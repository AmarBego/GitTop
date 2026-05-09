//! Credential persistence layer.
//!
//! Dispatches GitHub token and proxy-credential storage between two backends:
//!
//! 1. **System keyring** — Secret Service (Linux/FreeBSD), Credential Manager
//!    (Windows), Keychain (macOS). The default. Honored when the user picks
//!    `CredentialStorage::Keyring` in settings.
//!
//! 2. **Encrypted file** — `credentials.enc` in the platform data directory,
//!    encrypted with ChaCha20-Poly1305. The fallback when no secret service
//!    is available; the user opts in explicitly via settings. The encryption
//!    key sits next to the data, so this is closer to "obfuscated at rest"
//!    than real protection.
//!
//! Both `keyring.rs` and `proxy_keyring.rs` route through this module so the
//! backend choice is honored uniformly.

use crate::settings::{AppSettings, CredentialStorage};

use super::redaction::redact_secrets;

/// Errors common to both backends. Internally tagged so the caller can
/// distinguish "no entry" (a normal absence) from real failures.
#[derive(Debug, thiserror::Error, Clone)]
pub enum CredentialError {
    #[error("Credential backend error: {0}")]
    Backend(String),
}

/// Read+write+delete a single secret keyed by `(service, account)`.
///
/// `service` and `account` mirror the keyring crate's conventions: `service`
/// names the application (`gittop`), `account` names the entry inside it
/// (`gittop-<username>`, `proxy-<hash>`, etc).
pub trait Backend {
    fn save(&self, service: &str, account: &str, secret: &str) -> Result<(), CredentialError>;
    fn load(&self, service: &str, account: &str) -> Result<Option<String>, CredentialError>;
    fn delete(&self, service: &str, account: &str) -> Result<(), CredentialError>;
}

/// Pick a backend based on the current settings. Re-evaluated on every call
/// so a user toggling the option in the UI takes effect immediately, without
/// needing a restart.
fn select_backend() -> Box<dyn Backend> {
    match AppSettings::load().credential_storage {
        CredentialStorage::Keyring => Box::new(keyring_backend::KeyringBackend),
        CredentialStorage::EncryptedFile => Box::new(file_backend::FileBackend),
    }
}

pub fn save(service: &str, account: &str, secret: &str) -> Result<(), CredentialError> {
    select_backend().save(service, account, secret)
}

pub fn load(service: &str, account: &str) -> Result<Option<String>, CredentialError> {
    select_backend().load(service, account)
}

pub fn delete(service: &str, account: &str) -> Result<(), CredentialError> {
    select_backend().delete(service, account)
}

/// Human-readable name of the native backend the `keyring` crate was compiled
/// against on this platform. Derived from the `cfg(target_os)` + Cargo feature
/// pairing in `Cargo.toml`. Surfaced in startup logs so it's obvious whether
/// "Keyring" storage means kernel keyutils, Secret Service, Credential Manager,
/// or Keychain — they have very different failure modes.
pub fn native_backend_name() -> &'static str {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        "linux-keyutils (kernel session keyring)"
    }
    #[cfg(target_os = "windows")]
    {
        "Windows Credential Manager"
    }
    #[cfg(target_os = "macos")]
    {
        "macOS Keychain"
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        "unknown"
    }
}

/// Probe whether the system keyring is reachable for writes. Used at startup
/// to decide whether to surface a "tokens won't persist" warning to the user.
///
/// The probe writes and deletes a sentinel entry under a dedicated service
/// name so it can't collide with real data. Returns `false` for any error.
///
/// Cached per-session via `OnceLock` because the underlying D-Bus / system
/// call can take tens of milliseconds, and the answer doesn't change during
/// a run (a user can't start gnome-keyring after launch in a way GitTop
/// would automatically pick up).
pub fn keyring_available() -> bool {
    use std::sync::OnceLock;
    static CACHED: OnceLock<bool> = OnceLock::new();
    *CACHED.get_or_init(|| {
        use ::keyring::Entry;
        const PROBE_SERVICE: &str = "gittop-keyring-probe";
        const PROBE_ACCOUNT: &str = "probe";
        let Ok(entry) = Entry::new(PROBE_SERVICE, PROBE_ACCOUNT) else {
            return false;
        };
        let ok = entry.set_password("ok").is_ok();
        if ok {
            // Best-effort cleanup. Failing here doesn't change the answer.
            let _ = entry.delete_credential();
        } else {
            tracing::warn!(
                "System keyring not writable; tokens will not persist with `Keyring` storage. \
                 Switch to `EncryptedFile` storage in Settings → Accounts to enable persistence."
            );
        }
        ok
    })
}

// ============================================================================
// Keyring backend — wraps the existing `keyring` crate behavior.
// ============================================================================

mod keyring_backend {
    use super::{Backend, CredentialError, redact_secrets};
    use ::keyring::Entry;

    pub struct KeyringBackend;

    fn entry(service: &str, account: &str) -> Result<Entry, CredentialError> {
        Entry::new(service, account)
            .map_err(|e| CredentialError::Backend(redact_secrets(&e.to_string())))
    }

    impl Backend for KeyringBackend {
        fn save(&self, service: &str, account: &str, secret: &str) -> Result<(), CredentialError> {
            entry(service, account)?
                .set_password(secret)
                .map_err(|e| CredentialError::Backend(redact_secrets(&e.to_string())))
        }

        fn load(&self, service: &str, account: &str) -> Result<Option<String>, CredentialError> {
            match entry(service, account)?.get_password() {
                Ok(v) => Ok(Some(v)),
                Err(::keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(CredentialError::Backend(redact_secrets(&e.to_string()))),
            }
        }

        fn delete(&self, service: &str, account: &str) -> Result<(), CredentialError> {
            match entry(service, account)?.delete_credential() {
                Ok(()) => Ok(()),
                Err(::keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(CredentialError::Backend(redact_secrets(&e.to_string()))),
            }
        }
    }
}

// ============================================================================
// File backend — ChaCha20-Poly1305 over a JSON map at credentials.enc.
// ============================================================================

mod file_backend {
    use super::{Backend, CredentialError, redact_secrets};
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD_NO_PAD as B64;
    use chacha20poly1305::{
        ChaCha20Poly1305, Key, KeyInit, Nonce,
        aead::{Aead, OsRng, rand_core::RngCore},
    };
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    pub struct FileBackend;

    /// On-disk envelope. Keys are `"{service}\0{account}"`. Values are
    /// base64(nonce || ciphertext+tag).
    #[derive(Debug, Default, Serialize, Deserialize)]
    struct Envelope {
        entries: BTreeMap<String, String>,
    }

    fn data_dir() -> Result<PathBuf, CredentialError> {
        let dir = dirs::data_local_dir()
            .ok_or_else(|| CredentialError::Backend("No data_local_dir available".into()))?
            .join("GitTop");
        std::fs::create_dir_all(&dir)
            .map_err(|e| CredentialError::Backend(format!("create data dir: {}", e)))?;
        Ok(dir)
    }

    fn key_path() -> Result<PathBuf, CredentialError> {
        Ok(data_dir()?.join(".cred-key"))
    }

    fn data_path() -> Result<PathBuf, CredentialError> {
        Ok(data_dir()?.join("credentials.enc"))
    }

    /// Read the per-machine 32-byte key, generating it on first use. The
    /// file is written with restrictive permissions on Unix; on Windows the
    /// access ACL on `%LOCALAPPDATA%` already restricts access to the user.
    fn load_or_create_key() -> Result<Key, CredentialError> {
        let path = key_path()?;
        if let Ok(bytes) = std::fs::read(&path)
            && bytes.len() == 32
        {
            let mut k = [0u8; 32];
            k.copy_from_slice(&bytes);
            return Ok(k.into());
        }

        let mut k = [0u8; 32];
        OsRng.fill_bytes(&mut k);
        write_secret_file(&path, &k)?;
        Ok(k.into())
    }

    /// Write a file with 0600 perms on Unix. Plain write on Windows; the
    /// containing dir's ACL already restricts access to the user.
    fn write_secret_file(path: &std::path::Path, bytes: &[u8]) -> Result<(), CredentialError> {
        std::fs::write(path, bytes)
            .map_err(|e| CredentialError::Backend(format!("write {}: {}", path.display(), e)))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    fn read_envelope() -> Result<Envelope, CredentialError> {
        let path = data_path()?;
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s)
                .map_err(|e| CredentialError::Backend(format!("parse credentials.enc: {}", e))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Envelope::default()),
            Err(e) => Err(CredentialError::Backend(format!(
                "read {}: {}",
                path.display(),
                e
            ))),
        }
    }

    fn write_envelope(env: &Envelope) -> Result<(), CredentialError> {
        let path = data_path()?;
        let bytes = serde_json::to_vec_pretty(env)
            .map_err(|e| CredentialError::Backend(format!("serialize envelope: {}", e)))?;
        write_secret_file(&path, &bytes)
    }

    fn compose_key(service: &str, account: &str) -> String {
        format!("{}\0{}", service, account)
    }

    fn cipher() -> Result<ChaCha20Poly1305, CredentialError> {
        let key = load_or_create_key()?;
        Ok(ChaCha20Poly1305::new(&key))
    }

    impl Backend for FileBackend {
        fn save(&self, service: &str, account: &str, secret: &str) -> Result<(), CredentialError> {
            let cipher = cipher()?;
            let mut nonce_bytes = [0u8; 12];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let ct = cipher
                .encrypt(nonce, secret.as_bytes())
                .map_err(|e| CredentialError::Backend(redact_secrets(&e.to_string())))?;

            let mut blob = Vec::with_capacity(12 + ct.len());
            blob.extend_from_slice(&nonce_bytes);
            blob.extend_from_slice(&ct);

            let mut env = read_envelope()?;
            env.entries
                .insert(compose_key(service, account), B64.encode(&blob));
            write_envelope(&env)
        }

        fn load(&self, service: &str, account: &str) -> Result<Option<String>, CredentialError> {
            let env = read_envelope()?;
            let Some(encoded) = env.entries.get(&compose_key(service, account)) else {
                return Ok(None);
            };
            let blob = B64
                .decode(encoded)
                .map_err(|e| CredentialError::Backend(format!("base64: {}", e)))?;
            if blob.len() < 12 + 16 {
                return Err(CredentialError::Backend("ciphertext too short".into()));
            }
            let (nonce_bytes, ct) = blob.split_at(12);
            let nonce = Nonce::from_slice(nonce_bytes);
            let pt = cipher()?
                .decrypt(nonce, ct)
                .map_err(|e| CredentialError::Backend(redact_secrets(&e.to_string())))?;
            let s = String::from_utf8(pt)
                .map_err(|e| CredentialError::Backend(format!("utf-8: {}", e)))?;
            Ok(Some(s))
        }

        fn delete(&self, service: &str, account: &str) -> Result<(), CredentialError> {
            let mut env = read_envelope()?;
            env.entries.remove(&compose_key(service, account));
            write_envelope(&env)
        }
    }
}
