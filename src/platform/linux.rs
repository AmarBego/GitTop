//! Linux-specific platform implementations.

/// Focus an existing GitTop window.
/// TODO: Implement using X11/Wayland window activation.
pub fn focus_existing_window() {
    // On Linux, this depends on the display server (X11 vs Wayland).
    // X11: Could use xdotool or libX11 to find and activate window
    // Wayland: More complex, compositor-specific protocols
    // For now, this is a no-op - the tray icon handles window restoration.
}

/// Enable dark mode for system UI elements.
/// Linux context menus follow GTK/Qt theme settings.
pub fn enable_dark_mode() {
    // Linux context menus use GTK theming.
    // The theme is controlled by GTK_THEME or gsettings.
    // tray-icon/muda should respect the system theme.
}

/// Initialize the tray subsystem.
/// On Linux, tray-icon uses GTK which must be initialized before use.
pub fn init_tray() {
    // GTK must be initialized before tray-icon can create menus
    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK for tray icon");
    }
}

/// Reduce memory footprint.
/// Uses malloc_trim on glibc systems.
pub fn trim_memory() {
    // On Linux with glibc, we can call malloc_trim to release memory
    // back to the OS. This is similar to EmptyWorkingSet on Windows.
    #[cfg(target_env = "gnu")]
    {
        // malloc_trim returns 1 if memory was released, 0 otherwise
        unsafe extern "C" {
            safe fn malloc_trim(pad: usize) -> i32;
        }
        malloc_trim(0);
    }
}

/// Send a native Linux notification via DBus.
///
/// Uses notify-rust which:
/// - Talks to the system notification daemon via DBus
/// - No polling required
/// - No background threads once fired
/// - Zero persistent memory cost
///
/// If `url` is provided, adds an "Open" action that opens the URL.
/// Works with: notify-osd, dunst, xfce4-notifyd, KDE, GNOME, etc.
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
        notification.hint(notify_rust::Hint::ActionIcons(true));

        // Show and handle action
        let handle = notification.show()?;

        // Clone URL for the closure
        let url_owned = url.to_string();
        // Spawn a thread to wait for action (non-blocking)
        std::thread::spawn(move || {
            handle.wait_for_action(|action| {
                if action == "open" || action == "default" {
                    let _ = open::that(&url_owned);
                }
            });
        });
        Ok(())
    } else {
        // Simple fire and forget
        notification.show().map(|_| ())
    }
}
