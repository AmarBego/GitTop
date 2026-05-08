//! Unix-socket singleton + show-window IPC for Linux/FreeBSD.
//!
//! The socket itself is the lock: only one process can `bind()` a given
//! filesystem path. A second instance tries to `connect()` first; if a live
//! daemon answers, it sends `SHOW\n`, the daemon enqueues a `ShowWindow`
//! tray command (same path the tray menu uses), and the second instance
//! exits cleanly.
//!
//! Wayland refuses cross-process focus, but the daemon can open its own
//! window — that's why this works where `single_instance` + EnumWindows
//! does not.

use crate::tray::TrayCommand;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Duration;

const SOCKET_NAME: &str = "gittop.sock";
const COMMAND_SHOW: &str = "SHOW";
const READ_LIMIT: usize = 32;
const IO_TIMEOUT: Duration = Duration::from_secs(2);

/// Holds the bound socket path; unlinks on drop. The listener thread is
/// detached and reclaimed by the OS at process exit — unlinking the path
/// only prevents new clients from connecting, which is what we want.
pub struct Server {
    socket_path: PathBuf,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

pub enum AcquireResult {
    /// We bound the socket; we are the daemon.
    Primary(Server),
    /// An existing daemon was contacted and signaled; caller should exit.
    Secondary,
    /// IPC could not be set up. Caller should proceed without a singleton lock.
    Unavailable(String),
}

/// Per-user socket path. Prefers `$XDG_RUNTIME_DIR` (mode 0700, tmpfs);
/// falls back to the per-user cache dir if the runtime dir isn't exported.
fn socket_path() -> Option<PathBuf> {
    if let Some(rt) = dirs::runtime_dir() {
        return Some(rt.join(SOCKET_NAME));
    }
    let cache = dirs::cache_dir()?.join("gittop");
    let _ = std::fs::create_dir_all(&cache);
    Some(cache.join(SOCKET_NAME))
}

/// Try to claim the singleton. Either binds and spawns the listener, or
/// signals an existing daemon and tells the caller to exit.
pub fn acquire(tx: Sender<TrayCommand>) -> AcquireResult {
    let Some(path) = socket_path() else {
        return AcquireResult::Unavailable("no XDG_RUNTIME_DIR or cache dir".into());
    };

    // 1. Probe for a live daemon. ECONNREFUSED means a stale socket file
    //    from a previous crash; ENOENT means a clean slate.
    match UnixStream::connect(&path) {
        Ok(stream) => return signal_show(stream),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
            let _ = std::fs::remove_file(&path);
        }
        Err(e) => return AcquireResult::Unavailable(format!("connect probe failed: {e}")),
    }

    // 2. Bind. AddrInUse here means another instance won a startup race
    //    between our connect probe and our bind — try to signal it once
    //    more before giving up.
    let listener = match UnixListener::bind(&path) {
        Ok(l) => l,
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => match UnixStream::connect(&path) {
            Ok(stream) => return signal_show(stream),
            Err(_) => {
                return AcquireResult::Unavailable(format!("bind raced and reconnect failed: {e}"));
            }
        },
        Err(e) => return AcquireResult::Unavailable(format!("bind failed: {e}")),
    };

    spawn_listener(listener, tx);
    tracing::info!(path = %path.display(), "IPC singleton bound");
    AcquireResult::Primary(Server { socket_path: path })
}

fn signal_show(mut stream: UnixStream) -> AcquireResult {
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));

    if let Err(e) = stream.write_all(format!("{COMMAND_SHOW}\n").as_bytes()) {
        return AcquireResult::Unavailable(format!("signal write failed: {e}"));
    }

    // Best-effort ACK so we don't race ahead of the daemon enqueueing the
    // command. We don't gate behavior on the contents.
    let mut ack = [0u8; 8];
    let _ = stream.read(&mut ack);

    tracing::info!("Existing instance signaled via IPC; exiting");
    AcquireResult::Secondary
}

fn spawn_listener(listener: UnixListener, tx: Sender<TrayCommand>) {
    let _ = std::thread::Builder::new()
        .name("gittop-ipc".into())
        .spawn(move || {
            for conn in listener.incoming() {
                match conn {
                    Ok(stream) => handle_client(stream, &tx),
                    Err(e) => tracing::warn!(error = %e, "IPC accept failed"),
                }
            }
        });
}

fn handle_client(mut stream: UnixStream, tx: &Sender<TrayCommand>) {
    let _ = stream.set_read_timeout(Some(IO_TIMEOUT));
    let _ = stream.set_write_timeout(Some(IO_TIMEOUT));

    let mut buf = [0u8; READ_LIMIT];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return,
    };

    let request = std::str::from_utf8(&buf[..n]).unwrap_or("");
    let cmd = request.split('\n').next().unwrap_or("").trim();

    let response: &[u8] = match cmd {
        COMMAND_SHOW => {
            // tx.send only fails if the daemon already shut down its
            // receiver — in that case the process is exiting anyway.
            let _ = tx.send(TrayCommand::ShowWindow);
            b"OK\n"
        }
        _ => b"ERR\n",
    };
    let _ = stream.write_all(response);
}
