//! Linux-specific platform implementations.

use crate::settings::AppSettings;
use crate::ui::App;
use iced::{Font, daemon, window};

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

/// Focus an existing GitTop window from another process (single-instance detection).
/// Called when a second GitTop instance tries to launch.
///
/// Note: This is different from iced's `window::gain_focus()` used in app.rs,
/// which works within the same process for tray "Show" functionality.
pub fn focus_existing_window() {
    // Wayland doesn't support focusing windows from other processes.
}

/// Linux context menus follow GTK/Qt theme settings.
pub fn enable_dark_mode() {}

/// System tray implementation using ksni (pure-Rust StatusNotifierItem).
pub mod tray {
    use crate::tray::TrayCommand;
    use ksni::{self, Icon, Tray, menu::StandardItem};
    use std::sync::mpsc::{self, Receiver, Sender};
    use std::sync::{Mutex, OnceLock};

    /// Global receiver for tray commands (set during TrayManager::new).
    static COMMAND_RECEIVER: OnceLock<Mutex<Receiver<TrayCommand>>> = OnceLock::new();

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

            let (tx, rx) = mpsc::channel();

            // Store receiver in global so poll_global_events can access it
            COMMAND_RECEIVER
                .set(Mutex::new(rx))
                .map_err(|_| "TrayManager already initialized")?;

            let tray = GitTopTray { tx };

            // Check if running in Flatpak (file exists)
            let is_flatpak = std::path::Path::new("/.flatpak-info").exists();

            // Use blocking spawn API
            // For Flatpak, we must disable D-Bus name ownership as we can't own arbitrary names.
            let handle = tray.disable_dbus_name(is_flatpak).spawn()?;

            Ok(Self { handle })
        }

        pub fn poll_global_events() -> Option<TrayCommand> {
            COMMAND_RECEIVER.get()?.lock().ok()?.try_recv().ok()
        }
    }
}

/// Release memory back to the OS (glibc only).
pub fn trim_memory() {
    #[cfg(target_env = "gnu")]
    {
        unsafe extern "C" {
            safe fn malloc_trim(pad: usize) -> i32;
        }
        malloc_trim(0);
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
ExecStart="{EXEC_PATH}"
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

    fn has_systemd() -> bool {
        Command::new("systemctl")
            .arg("--user")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn is_enabled() -> bool {
        if !has_systemd() {
            return false;
        }

        Command::new("systemctl")
            .args(["--user", "is-enabled", "gittop.service"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn enable() -> Result<(), OnBootError> {
        if !has_systemd() {
            return Err(OnBootError::NotSupported);
        }

        let exec_path = std::env::current_exe()
            .map_err(OnBootError::Io)?
            .to_string_lossy()
            .to_string();

        let service_content = SYSTEMD_SERVICE_TEMPLATE.replace("{EXEC_PATH}", &exec_path);

        let service_dir = systemd_user_dir().ok_or(OnBootError::NotSupported)?;
        fs::create_dir_all(&service_dir)?;

        let service_path = systemd_service_path().ok_or(OnBootError::NotSupported)?;
        fs::write(&service_path, service_content)?;

        let reload = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output()?;

        if !reload.status.success() {
            return Err(OnBootError::CommandFailed(
                String::from_utf8_lossy(&reload.stderr).to_string(),
            ));
        }

        let enable = Command::new("systemctl")
            .args(["--user", "enable", "gittop.service"])
            .output()?;

        if !enable.status.success() {
            return Err(OnBootError::CommandFailed(
                String::from_utf8_lossy(&enable.stderr).to_string(),
            ));
        }

        Ok(())
    }

    pub fn disable() -> Result<(), OnBootError> {
        if !has_systemd() {
            return Err(OnBootError::NotSupported);
        }

        let disable = Command::new("systemctl")
            .args(["--user", "--quiet", "disable", "gittop.service"])
            .output()?;

        // With --quiet, systemctl returns success even if unit doesn't exist
        if !disable.status.success() {
            return Err(OnBootError::CommandFailed(
                String::from_utf8_lossy(&disable.stderr).to_string(),
            ));
        }

        if let Some(service_path) = systemd_service_path().filter(|p| p.exists()) {
            fs::remove_file(&service_path)?;
        }

        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output();

        Ok(())
    }
}
