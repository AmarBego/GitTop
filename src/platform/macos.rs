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
