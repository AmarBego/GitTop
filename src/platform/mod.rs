//! Platform-specific functionality.
//!
//! This module provides cross-platform abstractions for OS-specific features
//! like memory management, window focusing, theme settings, and notifications.

#[cfg(windows)]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub(crate) mod linux;

#[cfg(target_os = "freebsd")]
pub(crate) mod freebsd;

// Re-export platform functions with unified API

/// Focus an existing application window (for single-instance support).
/// Called when a second instance tries to launch.
pub fn focus_existing_window() {
    #[cfg(windows)]
    windows::focus_existing_window();

    #[cfg(target_os = "macos")]
    macos::focus_existing_window();

    #[cfg(target_os = "linux")]
    linux::focus_existing_window();

    #[cfg(target_os = "freebsd")]
    freebsd::focus_existing_window();
}

/// Enable dark mode for system UI elements (context menus, etc.).
/// Should be called early in app initialization.
pub fn enable_dark_mode() {
    #[cfg(windows)]
    windows::enable_dark_mode();

    #[cfg(target_os = "macos")]
    macos::enable_dark_mode();

    #[cfg(target_os = "linux")]
    linux::enable_dark_mode();

    #[cfg(target_os = "freebsd")]
    freebsd::enable_dark_mode();
}

/// Aggressively reduce memory footprint.
/// Trims working set on Windows, may trigger GC hints on other platforms.
/// Call when minimizing to tray.
pub fn trim_memory() {
    #[cfg(windows)]
    windows::trim_working_set();

    #[cfg(target_os = "macos")]
    macos::trim_memory();

    #[cfg(target_os = "linux")]
    linux::trim_memory();

    #[cfg(target_os = "freebsd")]
    freebsd::trim_memory();
}

/// Initialize the tray subsystem.
/// Must be called before creating TrayManager.
/// On Linux/FreeBSD, this initializes GTK which tray-icon requires.
pub fn init_tray() {
    #[cfg(windows)]
    windows::init_tray();

    #[cfg(target_os = "macos")]
    macos::init_tray();

    #[cfg(target_os = "linux")]
    linux::init_tray();

    #[cfg(target_os = "freebsd")]
    freebsd::init_tray();
}

/// Send a native desktop notification.
///
/// This is a fire-and-forget operation:
/// - Sends the notification to the system
/// - Returns immediately
/// - Allocates nothing long-lived
/// - Zero persistent memory cost
///
/// If `url` is provided, clicking the notification will open that URL.
///
/// Platform implementations:
/// - Windows: WinRT toast notifications
/// - macOS: NSUserNotificationCenter / UNUserNotificationCenter  
/// - Linux: DBus via notify-rust
/// - FreeBSD: DBus via notify-rust
pub fn notify(
    title: &str,
    body: &str,
    url: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    return windows::notify(title, body, url).map_err(|e| e.into());

    #[cfg(target_os = "macos")]
    return macos::notify(title, body, url).map_err(|e| e.into());

    #[cfg(target_os = "linux")]
    return linux::notify(title, body, url).map_err(|e| e.into());

    #[cfg(target_os = "freebsd")]
    return freebsd::notify(title, body, url).map_err(|e| e.into());
}

/// Run the iced application.
/// On Linux/FreeBSD, uses daemon mode to stay alive when window closes.
/// On Windows/macOS, uses normal application mode.
pub fn run_app() -> iced::Result {
    #[cfg(windows)]
    return windows::run_app();

    #[cfg(target_os = "macos")]
    return macos::run_app();

    #[cfg(target_os = "linux")]
    return linux::run_app();

    #[cfg(target_os = "freebsd")]
    return freebsd::run_app();
}

/// On-boot/autostart functionality.
///
/// Allows the application to start automatically when the user logs in.
///
/// Platform support:
/// - Linux: systemd user services (implemented), OpenRC (TODO)
/// - Windows: Registry (TODO)
/// - macOS: LaunchAgents (TODO)
/// - FreeBSD: (TODO)
pub mod on_boot {
    use std::fmt;
    use std::io;

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

    /// Check if autostart is currently enabled.
    ///
    /// Returns `true` if the application will start automatically on user login.
    pub fn is_enabled() -> bool {
        #[cfg(windows)]
        return super::windows::on_boot::is_enabled();

        #[cfg(target_os = "macos")]
        return super::macos::on_boot::is_enabled();

        #[cfg(target_os = "linux")]
        return super::linux::on_boot::is_enabled();

        #[cfg(target_os = "freebsd")]
        return super::freebsd::on_boot::is_enabled();
    }

    /// Enable autostart.
    ///
    /// Configures the system to start the application automatically on user login.
    pub fn enable() -> Result<(), OnBootError> {
        #[cfg(windows)]
        return super::windows::on_boot::enable().map_err(convert_error);

        #[cfg(target_os = "macos")]
        return super::macos::on_boot::enable().map_err(convert_error);

        #[cfg(target_os = "linux")]
        return super::linux::on_boot::enable().map_err(convert_error);

        #[cfg(target_os = "freebsd")]
        return super::freebsd::on_boot::enable().map_err(convert_error);
    }

    /// Disable autostart.
    ///
    /// Removes the autostart configuration so the application no longer starts on login.
    pub fn disable() -> Result<(), OnBootError> {
        #[cfg(windows)]
        return super::windows::on_boot::disable().map_err(convert_error);

        #[cfg(target_os = "macos")]
        return super::macos::on_boot::disable().map_err(convert_error);

        #[cfg(target_os = "linux")]
        return super::linux::on_boot::disable().map_err(convert_error);

        #[cfg(target_os = "freebsd")]
        return super::freebsd::on_boot::disable().map_err(convert_error);
    }

    // Helper to convert platform-specific error to unified error
    #[cfg(windows)]
    fn convert_error(e: super::windows::on_boot::OnBootError) -> OnBootError {
        match e {
            super::windows::on_boot::OnBootError::NotSupported => OnBootError::NotSupported,
            super::windows::on_boot::OnBootError::Io(io_err) => OnBootError::Io(io_err),
            super::windows::on_boot::OnBootError::CommandFailed(msg) => {
                OnBootError::CommandFailed(msg)
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn convert_error(e: super::macos::on_boot::OnBootError) -> OnBootError {
        match e {
            super::macos::on_boot::OnBootError::NotSupported => OnBootError::NotSupported,
            super::macos::on_boot::OnBootError::Io(io_err) => OnBootError::Io(io_err),
            super::macos::on_boot::OnBootError::CommandFailed(msg) => {
                OnBootError::CommandFailed(msg)
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn convert_error(e: super::linux::on_boot::OnBootError) -> OnBootError {
        match e {
            super::linux::on_boot::OnBootError::NotSupported => OnBootError::NotSupported,
            super::linux::on_boot::OnBootError::Io(io_err) => OnBootError::Io(io_err),
            super::linux::on_boot::OnBootError::CommandFailed(msg) => {
                OnBootError::CommandFailed(msg)
            }
        }
    }

    #[cfg(target_os = "freebsd")]
    fn convert_error(e: super::freebsd::on_boot::OnBootError) -> OnBootError {
        match e {
            super::freebsd::on_boot::OnBootError::NotSupported => OnBootError::NotSupported,
            super::freebsd::on_boot::OnBootError::Io(io_err) => OnBootError::Io(io_err),
            super::freebsd::on_boot::OnBootError::CommandFailed(msg) => {
                OnBootError::CommandFailed(msg)
            }
        }
    }
}
