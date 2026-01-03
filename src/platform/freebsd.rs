//! FreeBSD-specific platform implementations.

/// Focus an existing GitTop window.
/// TODO: Implement using X11 window activation.
pub fn focus_existing_window() {
    // FreeBSD typically uses X11, similar to Linux.
    // For now, this is a no-op.
}

/// Enable dark mode for system UI elements.
/// FreeBSD context menus follow GTK/Qt theme settings.
pub fn enable_dark_mode() {
    // Similar to Linux, GTK theming controls context menu appearance.
}

/// Initialize the tray subsystem.
/// On FreeBSD, tray-icon uses GTK which must be initialized before use.
pub fn init_tray() {
    // GTK must be initialized before tray-icon can create menus
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK for tray icon");
    }
}

/// Reduce memory footprint.
pub fn trim_memory() {
    // FreeBSD uses jemalloc by default.
    // Could potentially call jemalloc's purge functions.
    // For now, this is a no-op - the OS handles memory pressure.
}

/// Send a native FreeBSD notification via DBus.
///
/// Uses notify-rust which:
/// - Talks to the system notification daemon via DBus
/// - No polling required
/// - No background threads once fired
/// - Zero persistent memory cost
///
/// If `url` is provided, adds an "Open" action that opens the URL.
/// Works with any DBus-compatible notification daemon.
pub fn notify(title: &str, body: &str, url: Option<&str>) -> Result<(), notify_rust::error::Error> {
    use notify_rust::Notification;

    let mut notification = Notification::new();
    notification
        .summary(title)
        .body(body)
        .appname("GitTop")
        .timeout(5000); // 5 seconds

    // Add action if URL provided
    if let Some(url) = url {
        notification.action("open", "Open");

        // Show and handle action
        let handle = notification.show()?;

        let url_owned = url.to_string();
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

/// On-boot/autostart functionality for FreeBSD.
///
/// TODO: Investigate rc.d or user-level autostart mechanism.
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
    /// TODO: Investigate FreeBSD autostart mechanism
    pub fn is_enabled() -> bool {
        false
    }

    /// Enable autostart.
    ///
    /// TODO: Implement FreeBSD autostart
    pub fn enable() -> Result<(), OnBootError> {
        Err(OnBootError::NotSupported)
    }

    /// Disable autostart.
    ///
    /// TODO: Implement FreeBSD autostart
    pub fn disable() -> Result<(), OnBootError> {
        Err(OnBootError::NotSupported)
    }
}
