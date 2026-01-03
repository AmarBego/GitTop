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

/// GTK must be initialized before tray-icon can create menus.
pub fn init_tray() {
    match gtk::init() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Failed to initialize GTK for tray icon: {:?}", e);
            eprintln!("Ensure GTK3 is installed: gtk3, libappindicator-gtk3");
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
    use std::fmt;
    use std::fs;
    use std::io;
    use std::path::PathBuf;
    use std::process::Command;

    /// Error type for on_boot operations.
    #[derive(Debug)]
    pub enum OnBootError {
        /// The operation is not supported on this platform/init system.
        NotSupported,
        /// An I/O error occurred.
        Io(io::Error),
        /// A command failed to execute.
        CommandFailed(String),
    }

    impl fmt::Display for OnBootError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                OnBootError::NotSupported => write!(f, "on-boot is not supported on this system"),
                OnBootError::Io(e) => write!(f, "I/O error: {}", e),
                OnBootError::CommandFailed(msg) => write!(f, "command failed: {}", msg),
            }
        }
    }

    impl std::error::Error for OnBootError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                OnBootError::Io(e) => Some(e),
                _ => None,
            }
        }
    }

    impl From<io::Error> for OnBootError {
        fn from(e: io::Error) -> Self {
            OnBootError::Io(e)
        }
    }

    /// The systemd user service unit file content.
    const SYSTEMD_SERVICE_TEMPLATE: &str = r#"[Unit]
Description=GitTop - GitHub Notifications Manager
After=graphical-session.target

[Service]
Type=simple
ExecStart={EXEC_PATH}
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
            .args(["--user", "disable", "gittop.service"])
            .output()?;

        // Ignore "not found" errors - service may not exist yet.
        if !disable.status.success() {
            let stderr = String::from_utf8_lossy(&disable.stderr);
            if !stderr.contains("not found") && !stderr.contains("No such file") {
                eprintln!("Warning: systemctl disable failed: {}", stderr);
            }
        }

        if let Some(service_path) = systemd_service_path() {
            if service_path.exists() {
                fs::remove_file(&service_path)?;
            }
        }

        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output();

        Ok(())
    }
}
