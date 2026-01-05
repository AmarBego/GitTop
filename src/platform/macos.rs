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

/// System tray implementation using tray-icon (native macOS APIs).
pub mod tray {
    use crate::tray::TrayCommand;
    use std::sync::OnceLock;
    use tray_icon::{
        Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
        menu::{Menu, MenuEvent, MenuId, MenuItem},
    };

    static MENU_IDS: OnceLock<MenuIds> = OnceLock::new();

    #[derive(Debug)]
    struct MenuIds {
        show: MenuId,
        quit: MenuId,
    }

    pub struct TrayManager {
        #[allow(dead_code)]
        tray: TrayIcon,
    }

    impl TrayManager {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let show_item = MenuItem::new("Show GitTop", true, None);
            let quit_item = MenuItem::new("Quit", true, None);

            MENU_IDS
                .set(MenuIds {
                    show: show_item.id().clone(),
                    quit: quit_item.id().clone(),
                })
                .expect("TrayManager initialized twice");

            let menu = Menu::new();
            menu.append(&show_item)?;
            menu.append(&quit_item)?;

            let icon = Self::create_icon()?;
            let tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("GitTop - GitHub Notifications")
                .with_icon(icon)
                .build()?;

            Ok(Self { tray })
        }

        fn create_icon() -> Result<Icon, Box<dyn std::error::Error>> {
            use image::ImageReader;
            use std::io::Cursor;

            const ICON_BYTES: &[u8] = include_bytes!("../../assets/images/GitTop-256x256.png");

            let img = ImageReader::new(Cursor::new(ICON_BYTES))
                .with_guessed_format()?
                .decode()?
                .resize(32, 32, image::imageops::FilterType::Lanczos3)
                .into_rgba8();

            let (width, height) = img.dimensions();
            Icon::from_rgba(img.into_raw(), width, height).map_err(Into::into)
        }

        pub fn poll_global_events() -> Option<TrayCommand> {
            let command = Self::poll_menu_events();
            Self::drain_tray_icon_events();
            command
        }

        fn poll_menu_events() -> Option<TrayCommand> {
            let event = MenuEvent::receiver().try_recv().ok()?;
            let ids = MENU_IDS.get()?;

            [
                (&ids.show, TrayCommand::ShowWindow),
                (&ids.quit, TrayCommand::Quit),
            ]
            .into_iter()
            .find_map(|(id, cmd)| (event.id == *id).then_some(cmd))
        }

        fn drain_tray_icon_events() {
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if matches!(event, TrayIconEvent::Leave { .. }) {
                    super::trim_memory();
                }
            }
        }
    }
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
    // Re-export the shared error type from the parent module
    pub use crate::platform::on_boot::OnBootError;

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
