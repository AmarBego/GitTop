//! macOS-specific platform implementations.
//! the notes are to help later this is all i could find from  documentations and resources so not complete

/// Focus an existing GitTop window.
/// TODO: Implement using NSRunningApplication or AppleScript.
pub fn focus_existing_window() {
    // On macOS, the system typically handles single-instance apps
    // through the application delegate. For now, this is a no-op.
    // Future: Use objc2 crate to call [[NSRunningApplication currentApplication] activateWithOptions:]
}

/// Enable dark mode for system UI elements.
/// macOS respects the system appearance automatically.
pub fn enable_dark_mode() {
    // macOS context menus automatically follow system appearance.
    // No action needed.
}

/// Initialize the tray subsystem.
/// macOS doesn't require special initialization.
pub fn init_tray() {
    // No-op on macOS - tray-icon works without GTK
}

/// Reduce memory footprint.
/// TODO: Could potentially use madvise or similar.
pub fn trim_memory() {
    // macOS doesn't have a direct equivalent to EmptyWorkingSet.
    // The system manages memory pressure automatically.
    // Could potentially use jemalloc's purge or madvise(MADV_FREE).
}

/// Send a native macOS notification.
///
/// Uses mac-notification-sys which wraps NSUserNotificationCenter.
/// Notifications are:
/// - Lightweight
/// - Don't require daemons
/// - Don't require keeping handles alive
/// - Zero memory impact after send
///
/// Note: macOS doesn't support click-to-open-URL natively via this API.
/// The URL is included in the notification body as a fallback.
pub fn notify(
    title: &str,
    body: &str,
    url: Option<&str>,
) -> Result<(), mac_notification_sys::error::Error> {
    use mac_notification_sys::*;

    // Include URL in body if provided (macOS notification click handling is limited)
    let display_body = if let Some(url) = url {
        format!("{}\n{}", body, url)
    } else {
        body.to_string()
    };

    // Fire and forget - allocates nothing long-lived
    send_notification(
        title,
        None, // No subtitle
        &display_body,
        None, // No sound (use default)
    )
    .map(|_| ())
}

/// On-boot/autostart functionality for macOS.
///
/// TODO: Implement using LaunchAgents.
/// - Create plist at ~/Library/LaunchAgents/com.gittop.plist
pub mod on_boot {
    use std::fmt;
    use std::io;

    /// Error type for on_boot operations.
    #[derive(Debug)]
    pub enum OnBootError {
        /// The operation is not supported on this platform.
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
    /// TODO: Check if ~/Library/LaunchAgents/com.gittop.plist exists
    pub fn is_enabled() -> bool {
        false
    }

    /// Enable autostart.
    ///
    /// TODO: Create ~/Library/LaunchAgents/com.gittop.plist
    pub fn enable() -> Result<(), OnBootError> {
        Err(OnBootError::NotSupported)
    }

    /// Disable autostart.
    ///
    /// TODO: Remove ~/Library/LaunchAgents/com.gittop.plist
    pub fn disable() -> Result<(), OnBootError> {
        Err(OnBootError::NotSupported)
    }
}
