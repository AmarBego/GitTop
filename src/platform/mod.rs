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

// ============================================================================
// Platform dispatch macros
// ============================================================================

/// Dispatch a no-op function call to the correct platform module.
macro_rules! platform_call {
    ($fn:ident) => {{
        #[cfg(windows)]
        windows::$fn();

        #[cfg(target_os = "macos")]
        macos::$fn();

        #[cfg(target_os = "linux")]
        linux::$fn();

        #[cfg(target_os = "freebsd")]
        freebsd::$fn();
    }};
}

/// Dispatch a function call with a return value to the correct platform module.
macro_rules! platform_return {
    ($fn:ident $(, $arg:expr)*) => {{
        #[cfg(windows)]
        return windows::$fn($($arg),*);

        #[cfg(target_os = "macos")]
        return macos::$fn($($arg),*);

        #[cfg(target_os = "linux")]
        return linux::$fn($($arg),*);

        #[cfg(target_os = "freebsd")]
        return freebsd::$fn($($arg),*);
    }};
}

// ============================================================================
// Public platform API
// ============================================================================

/// Focus an existing application window (for single-instance support).
/// Called when a second instance tries to launch.
pub fn focus_existing_window() {
    platform_call!(focus_existing_window);
}

/// Enable dark mode for system UI elements (context menus, etc.).
/// Should be called early in app initialization.
pub fn enable_dark_mode() {
    platform_call!(enable_dark_mode);
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

// Re-export platform-specific tray module
#[cfg(target_os = "linux")]
pub use linux::tray;

#[cfg(target_os = "freebsd")]
pub use freebsd::tray;

#[cfg(windows)]
pub use windows::tray;

#[cfg(target_os = "macos")]
pub use macos::tray;

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
    platform_return!(run_app);
}

// ============================================================================
// On-boot/autostart functionality
// ============================================================================

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

    /// Error type for on_boot operations (shared across all platforms).
    #[derive(Debug)]
    #[allow(dead_code)] // Variants used on different platforms (e.g., CommandFailed on Linux)
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

        #[cfg(not(any(
            windows,
            target_os = "macos",
            target_os = "linux",
            target_os = "freebsd"
        )))]
        return false;
    }

    /// Enable autostart.
    ///
    /// Configures the system to start the application automatically on user login.
    pub fn enable() -> Result<(), OnBootError> {
        #[cfg(windows)]
        return super::windows::on_boot::enable();

        #[cfg(target_os = "macos")]
        return super::macos::on_boot::enable();

        #[cfg(target_os = "linux")]
        return super::linux::on_boot::enable();

        #[cfg(target_os = "freebsd")]
        return super::freebsd::on_boot::enable();

        #[cfg(not(any(
            windows,
            target_os = "macos",
            target_os = "linux",
            target_os = "freebsd"
        )))]
        return Err(OnBootError::NotSupported);
    }

    /// Disable autostart.
    ///
    /// Removes the autostart configuration so the application no longer starts on login.
    pub fn disable() -> Result<(), OnBootError> {
        #[cfg(windows)]
        return super::windows::on_boot::disable();

        #[cfg(target_os = "macos")]
        return super::macos::on_boot::disable();

        #[cfg(target_os = "linux")]
        return super::linux::on_boot::disable();

        #[cfg(target_os = "freebsd")]
        return super::freebsd::on_boot::disable();

        #[cfg(not(any(
            windows,
            target_os = "macos",
            target_os = "linux",
            target_os = "freebsd"
        )))]
        return Err(OnBootError::NotSupported);
    }
}
