#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! GitTop - A beautiful native GitHub notification manager
//! No browser engine required. Pure Rust. Pure performance.

mod cache;
mod github;
mod platform;
mod settings;
mod specs;
mod tray;
mod ui;

use iced::window::Position;
use iced::{application, Font, Point, Size};
use settings::AppSettings;
use single_instance::SingleInstance;
use std::sync::atomic::{AtomicUsize, Ordering};
use ui::App;

/// Mutex name for single instance detection
const SINGLE_INSTANCE_MUTEX: &str = "GitTop-SingleInstance-Mutex-7a8b9c0d";

/// Global mock notification count (set via CLI)
pub static MOCK_NOTIFICATION_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Parse CLI arguments for mock data testing.
fn parse_cli_args() {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if (args[i] == "--mock-notifications" || args[i] == "-m") && i + 1 < args.len() {
            if let Ok(count) = args[i + 1].parse::<usize>() {
                MOCK_NOTIFICATION_COUNT.store(count, Ordering::Relaxed);
            }
            i += 1;
        }
        i += 1;
    }
}

fn main() -> iced::Result {
    // Force OpenGL backend for wgpu to minimize memory footprint
    // OpenGL uses ~42MB vs Vulkan's ~164MB or DX12's ~133MB
    // Safety: This is called at program start before any threads are spawned
    unsafe { std::env::set_var("WGPU_BACKEND", "gl") };

    // Parse CLI arguments (e.g., --mock-notifications 1000)
    parse_cli_args();

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

    // Load settings to restore window state
    let settings = AppSettings::load();

    // Validate window size (Windows reports 0x0 when minimized)
    let window_size = if settings.window_width >= 100.0 && settings.window_height >= 100.0 {
        Size::new(settings.window_width, settings.window_height)
    } else {
        Size::new(800.0, 640.0) // Default size
    };

    // Validate window position (Windows reports -32000 when minimized)
    let window_position = match (settings.window_x, settings.window_y) {
        (Some(x), Some(y)) if x > -10000 && y > -10000 => {
            Position::Specific(Point::new(x as f32, y as f32))
        }
        _ => Position::Centered,
    };

    application(App::new, App::update, App::view)
        .title(|app: &App| app.title())
        .theme(|app: &App| app.theme())
        .subscription(App::subscription)
        .window_size(window_size)
        .position(window_position)
        .antialiasing(true)
        .default_font(Font::DEFAULT)
        .exit_on_close_request(false)
        .run()
}
