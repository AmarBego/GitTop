// Temporarily disabled for debugging - enables console output in release builds
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! GitTop - A beautiful native GitHub notification manager
//! No browser engine required. Pure Rust. Pure performance.

mod github;
mod platform;
mod settings;
mod tray;
mod ui;

use iced::{application, Font, Size};
use single_instance::SingleInstance;
use ui::App;

/// Mutex name for single instance detection
const SINGLE_INSTANCE_MUTEX: &str = "GitTop-SingleInstance-Mutex-7a8b9c0d";

fn main() -> iced::Result {
    // Check for existing instance
    let instance = SingleInstance::new(SINGLE_INSTANCE_MUTEX).unwrap();

    if !instance.is_single() {
        // Another instance is running - try to focus it and exit
        platform::focus_existing_window();
        return Ok(());
    }

    // Enable dark mode for context menus
    platform::enable_dark_mode();

    // Initialize tray icon on main thread (required for macOS)
    // The tray must be kept alive for the duration of the app
    let _tray = tray::TrayManager::new().ok();

    application(App::new, App::update, App::view)
        .title(|app: &App| app.title())
        .theme(|app: &App| app.theme())
        .subscription(App::subscription)
        .window_size(Size::new(800.0, 640.0))
        .antialiasing(true)
        .default_font(Font::DEFAULT)
        .exit_on_close_request(false)
        .run()
}
