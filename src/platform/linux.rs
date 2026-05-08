//! Linux-specific platform implementations.

use crate::settings::AppSettings;
use crate::tray::TrayCommand;
use crate::ui::App;
use iced::{Font, daemon, window};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

// ============================================================================
// Cross-thread tray command bus
// ============================================================================
//
// Both the ksni tray and the IPC listener push `TrayCommand`s here; the iced
// subscription drains them via `poll_command`. Lazily initialized so callers
// don't have to order tray init vs. IPC init.

struct CommandBus {
    tx: Sender<TrayCommand>,
    rx: Mutex<Receiver<TrayCommand>>,
}

static COMMAND_BUS: OnceLock<CommandBus> = OnceLock::new();

fn command_bus() -> &'static CommandBus {
    COMMAND_BUS.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        CommandBus {
            tx,
            rx: Mutex::new(rx),
        }
    })
}

pub(crate) fn command_sender() -> Sender<TrayCommand> {
    command_bus().tx.clone()
}

pub(crate) fn poll_command() -> Option<TrayCommand> {
    command_bus().rx.lock().ok()?.try_recv().ok()
}

/// Run the iced application using daemon mode.
/// Daemon mode allows the app to continue running with zero windows,
/// which is needed because Wayland doesn't support hiding windows.
pub fn run_app() -> iced::Result {
    daemon(App::new_for_daemon, App::update, App::view_for_daemon)
        .title(App::title_for_daemon)
        .theme(App::theme_for_daemon)
        .subscription(App::subscription)
        .antialiasing(true)
        .default_font(Font::DEFAULT)
        .run()
}

/// Build window settings for spawning from daemon.
pub fn build_initial_window_settings() -> (window::Id, iced::Task<crate::ui::app::Message>) {
    let settings = AppSettings::load();

    let size = iced::Size::new(
        if settings.window_width >= 100.0 {
            settings.window_width
        } else {
            800.0
        },
        if settings.window_height >= 100.0 {
            settings.window_height
        } else {
            640.0
        },
    );

    let position = match (settings.window_x, settings.window_y) {
        (Some(x), Some(y)) if x > -10000 && y > -10000 => {
            window::Position::Specific(iced::Point::new(x as f32, y as f32))
        }
        _ => window::Position::Centered,
    };

    let window_settings = window::Settings {
        size,
        position,
        platform_specific: window::settings::PlatformSpecific {
            application_id: "gittop".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let (id, task) = window::open(window_settings);
    (id, task.discard())
}

/// Linux context menus follow GTK/Qt theme settings.
pub fn enable_dark_mode() {}

/// System tray implementation using ksni (pure-Rust StatusNotifierItem).
pub mod tray {
    use crate::tray::TrayCommand;
    use ksni::{self, Icon, Tray, menu::StandardItem};
    use std::sync::mpsc::Sender;

    struct GitTopTray {
        tx: Sender<TrayCommand>,
    }

    impl Tray for GitTopTray {
        fn id(&self) -> String {
            "gittop".into()
        }

        fn category(&self) -> ksni::Category {
            ksni::Category::ApplicationStatus
        }

        fn title(&self) -> String {
            "GitTop".into()
        }

        fn icon_name(&self) -> String {
            "gittop".into()
        }

        fn icon_pixmap(&self) -> Vec<Icon> {
            // Fallback: embed the icon directly
            const ICON_BYTES: &[u8] = include_bytes!("../../assets/images/GitTop-256x256.png");

            if let Ok(icon) = Self::load_png_icon(ICON_BYTES) {
                vec![icon]
            } else {
                vec![]
            }
        }

        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: "GitTop - GitHub Notifications".into(),
                ..Default::default()
            }
        }

        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            vec![
                StandardItem {
                    label: "Show GitTop".into(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.tx.send(TrayCommand::ShowWindow);
                    }),
                    ..Default::default()
                }
                .into(),
                ksni::MenuItem::Separator,
                StandardItem {
                    label: "Quit".into(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.tx.send(TrayCommand::Quit);
                    }),
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    impl GitTopTray {
        fn load_png_icon(bytes: &[u8]) -> Result<Icon, Box<dyn std::error::Error>> {
            use image::ImageReader;
            use std::io::Cursor;

            let img = ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()?
                .decode()?
                .resize(32, 32, image::imageops::FilterType::Lanczos3)
                .into_rgba8();

            let (width, height) = img.dimensions();
            let raw = img.into_raw();

            // ksni expects ARGB format, convert from RGBA
            let argb: Vec<u8> = raw
                .chunks(4)
                .flat_map(|rgba| [rgba[3], rgba[0], rgba[1], rgba[2]])
                .collect();

            Ok(Icon {
                width: width as i32,
                height: height as i32,
                data: argb,
            })
        }
    }

    pub struct TrayManager {
        #[allow(dead_code)]
        handle: ksni::blocking::Handle<GitTopTray>,
    }

    impl TrayManager {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            use ksni::blocking::TrayMethods;

            let tray = GitTopTray {
                tx: super::command_sender(),
            };

            // Check if running in Flatpak (file exists)
            let is_flatpak = std::path::Path::new("/.flatpak-info").exists();

            // Use blocking spawn API
            // For Flatpak, we must disable D-Bus name ownership as we can't own arbitrary names.
            let handle = tray.disable_dbus_name(is_flatpak).spawn()?;

            Ok(Self { handle })
        }

        pub fn poll_global_events() -> Option<TrayCommand> {
            super::poll_command()
        }
    }
}

/// Release memory back to the OS.
pub fn trim_memory() {
    #[cfg(target_env = "gnu")]
    {
        // glibc's malloc_trim can release free memory back to the system.
        unsafe extern "C" {
            safe fn malloc_trim(pad: usize) -> i32;
        }
        malloc_trim(0);
    }

    #[cfg(target_env = "musl")]
    {
        // musl's allocator (mallocng) is more aggressive at returning memory to the OS,
        // and doesn't provide a direct equivalent to malloc_trim.
    }
}

/// Send a native Linux notification via DBus.
pub fn notify(title: &str, body: &str, url: Option<&str>) -> Result<(), notify_rust::error::Error> {
    use notify_rust::Notification;

    let mut notification = Notification::new();
    notification
        .summary(title)
        .body(body)
        .appname("GitTop")
        .icon("gittop")
        .timeout(5000);

    if let Some(url) = url {
        notification.action("open", "Open");
        notification.hint(notify_rust::Hint::ActionIcons(true));

        let handle = notification.show()?;
        let url_owned = url.to_string();

        // Thread required because wait_for_action blocks.
        std::thread::spawn(move || {
            handle.wait_for_action(|action| {
                if action == "open" || action == "default" {
                    let _ = open::that(&url_owned);
                }
            });
        });
        Ok(())
    } else {
        notification.show().map(|_| ())
    }
}

pub mod on_boot {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    // Re-export the shared error type from the parent module
    pub use crate::platform::on_boot::OnBootError;

    /// The systemd user service unit file content.
    ///
    /// PassEnvironment inherits display variables from the user session,
    /// which are required for GUI applications to connect to the display server.
    const SYSTEMD_SERVICE_TEMPLATE: &str = r#"[Unit]
Description=GitTop - GitHub Notifications Manager
After=graphical-session.target

[Service]
Type=simple
ExecStart="{EXEC_PATH}" --hidden
PassEnvironment=DISPLAY WAYLAND_DISPLAY XDG_RUNTIME_DIR
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
"#;

    fn systemd_user_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("systemd/user"))
    }

    fn systemd_service_path() -> Option<PathBuf> {
        systemd_user_dir().map(|p| p.join("gittop.service"))
    }

    fn xdg_autostart_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("autostart"))
    }

    fn xdg_autostart_path() -> Option<PathBuf> {
        xdg_autostart_dir().map(|p| p.join("gittop.desktop"))
    }

    fn has_systemd() -> bool {
        #[cfg(target_env = "musl")]
        {
            // Musl-based systems like Void Linux typically don't use systemd.
            false
        }
        #[cfg(not(target_env = "musl"))]
        {
            Command::new("systemctl")
                .arg("--user")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    pub fn is_enabled() -> bool {
        // Check systemd first
        if has_systemd() {
            let systemd_enabled = Command::new("systemctl")
                .args(["--user", "is-enabled", "gittop.service"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if systemd_enabled {
                return true;
            }
        }

        // Fallback to XDG autostart
        xdg_autostart_path().map(|p| p.exists()).unwrap_or(false)
    }

    pub fn enable() -> Result<(), OnBootError> {
        let exec_path = std::env::current_exe()
            .map_err(OnBootError::Io)?
            .to_string_lossy()
            .to_string();

        if has_systemd() {
            let service_content = SYSTEMD_SERVICE_TEMPLATE.replace("{EXEC_PATH}", &exec_path);
            let service_dir = systemd_user_dir().ok_or(OnBootError::NotSupported)?;
            fs::create_dir_all(&service_dir)?;
            let service_path = systemd_service_path().ok_or(OnBootError::NotSupported)?;
            fs::write(&service_path, service_content)?;

            let reload = Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .output()?;

            if reload.status.success() {
                let enable = Command::new("systemctl")
                    .args(["--user", "enable", "gittop.service"])
                    .output()?;

                if enable.status.success() {
                    return Ok(());
                }
            }
            // If systemd fails for some reason (e.g. broken user session), fall through to XDG
        }

        // XDG Autostart Fallback (Universal)
        let autostart_dir = xdg_autostart_dir().ok_or(OnBootError::NotSupported)?;
        fs::create_dir_all(&autostart_dir)?;

        let desktop_content = format!(
            r#"[Desktop Entry]
Name=GitTop
Comment=GitHub Notifications Manager
Exec={} --hidden
Icon=gittop
Terminal=false
Type=Application
Categories=Development;Utility;
X-GNOME-Autostart-enabled=true
"#,
            exec_path
        );

        let desktop_path = xdg_autostart_path().ok_or(OnBootError::NotSupported)?;
        fs::write(&desktop_path, desktop_content)?;

        Ok(())
    }

    pub fn disable() -> Result<(), OnBootError> {
        let mut error = None;

        // Try disabling systemd
        if has_systemd() {
            let disable = Command::new("systemctl")
                .args(["--user", "--quiet", "disable", "gittop.service"])
                .output()?;

            if disable.status.success() {
                if let Some(service_path) = systemd_service_path().filter(|p| p.exists()) {
                    let _ = fs::remove_file(&service_path);
                }
                let _ = Command::new("systemctl")
                    .args(["--user", "daemon-reload"])
                    .output();
            } else {
                error = Some(OnBootError::CommandFailed(
                    String::from_utf8_lossy(&disable.stderr).to_string(),
                ));
            }
        }

        // Always try to remove XDG autostart file
        if let Some(desktop_path) = xdg_autostart_path().filter(|p| p.exists())
            && let Err(e) = fs::remove_file(&desktop_path)
        {
            error = Some(OnBootError::Io(e));
        }

        if let Some(e) = error { Err(e) } else { Ok(()) }
    }
}
