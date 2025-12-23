//! System tray management for GitTop.
//!
//! Provides cross-platform tray icon with menu support using tray-icon crate.
//! Supports Windows, macOS, and Linux (GTK).

use std::sync::OnceLock;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItem},
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
};

/// Global storage for menu IDs so we can identify them in the global poll
static SHOW_MENU_ID: OnceLock<MenuId> = OnceLock::new();
static QUIT_MENU_ID: OnceLock<MenuId> = OnceLock::new();

/// Events from the system tray that the app should handle.
#[derive(Debug, Clone)]
pub enum TrayCommand {
    /// Show or focus the main window.
    ShowWindow,
    /// Quit the application.
    Quit,
}

/// Manages the system tray icon and menu.
pub struct TrayManager {
    #[allow(dead_code)]
    tray: TrayIcon,
}

impl TrayManager {
    /// Create a new tray icon with menu.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create menu items
        let show_item = MenuItem::new("Show GitTop", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        let show_item_id = show_item.id().clone();
        let quit_item_id = quit_item.id().clone();

        // Store IDs globally for poll_global_events
        let _ = SHOW_MENU_ID.set(show_item_id);
        let _ = QUIT_MENU_ID.set(quit_item_id);

        // Build menu
        let menu = Menu::new();
        menu.append(&show_item)?;
        menu.append(&quit_item)?;

        // Create icon from embedded PNG bytes
        // Using a simple 32x32 diamond icon
        let icon = Self::load_icon()?;

        // Build tray icon
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("GitTop - GitHub Notifications")
            .with_icon(icon)
            .build()?;

        Ok(Self { tray })
    }

    /// Load the tray icon from embedded bytes.
    fn load_icon() -> Result<Icon, Box<dyn std::error::Error>> {
        // Generate a simple 32x32 diamond icon programmatically
        // This avoids needing external icon files
        let size = 32u32;
        let mut rgba = vec![0u8; (size * size * 4) as usize];

        // Draw a filled diamond shape
        let center = size as i32 / 2;
        let radius = center - 2;

        for y in 0..size {
            for x in 0..size {
                let dx = (x as i32 - center).abs();
                let dy = (y as i32 - center).abs();

                // Diamond shape: |x - center| + |y - center| <= radius
                if dx + dy <= radius {
                    let idx = ((y * size + x) * 4) as usize;
                    // Blue-purple color matching app theme
                    rgba[idx] = 100; // R
                    rgba[idx + 1] = 149; // G
                    rgba[idx + 2] = 237; // B - cornflower blue
                    rgba[idx + 3] = 255; // A
                }
            }
        }

        Icon::from_rgba(rgba, size, size).map_err(|e| e.into())
    }

    /// Poll for tray events from global receivers (called from async context).
    pub fn poll_global_events() -> Option<TrayCommand> {
        // Check for menu events
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            // Check against globally stored menu IDs
            if let Some(show_id) = SHOW_MENU_ID.get() {
                if event.id == *show_id {
                    return Some(TrayCommand::ShowWindow);
                }
            }
            if let Some(quit_id) = QUIT_MENU_ID.get() {
                if event.id == *quit_id {
                    return Some(TrayCommand::Quit);
                }
            }
        }

        // Drain tray icon events but don't act on clicks
        // Window should only be shown via the menu "Show" item
        // Trim memory on Leave event to prevent accumulation
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if matches!(event, TrayIconEvent::Leave { .. }) {
                crate::platform::trim_memory();
            }
        }

        None
    }
}
