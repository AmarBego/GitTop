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
use iced::{Font, Point, Size, application};
use settings::AppSettings;
use single_instance::SingleInstance;
use std::sync::atomic::{AtomicUsize, Ordering};
use ui::App;

/// Mutex name for single instance detection
const SINGLE_INSTANCE_MUTEX: &str = "GitTop-SingleInstance-Mutex-7a8b9c0d";

/// Global mock notification count (set via CLI)
pub static MOCK_NOTIFICATION_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Minimum valid window dimension (Windows reports 0x0 when minimized)
const MIN_VALID_WINDOW_SIZE: f32 = 100.0;
/// Minimum valid window position (Windows reports -32000 when minimized)
const MIN_VALID_WINDOW_POS: i32 = -10000;

fn parse_cli_args() {
    let mut args = std::env::args().skip(1).peekable();

    while let Some(arg) = args.next() {
        if matches!(arg.as_str(), "--mock-notifications" | "-m")
            && let Some(Ok(count)) = args.next().map(|s| s.parse::<usize>())
        {
            MOCK_NOTIFICATION_COUNT.store(count, Ordering::Relaxed);
        }
    }
}

/// Load the window icon for the title bar.
fn load_window_icon() -> Option<iced::window::Icon> {
    use std::io::Cursor;

    const ICON_BYTES: &[u8] = include_bytes!("../assets/images/favicon-32x32.png");

    let img = image::ImageReader::new(Cursor::new(ICON_BYTES))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();

    let (width, height) = img.dimensions();
    iced::window::icon::from_rgba(img.into_raw(), width, height).ok()
}

fn main() -> iced::Result {
    // Force OpenGL backend for wgpu to minimize memory footprint
    // OpenGL uses ~42MB vs Vulkan's ~164MB or DX12's ~133MB
    // Safety: This is called at program start before any threads are spawned
    unsafe { std::env::set_var("WGPU_BACKEND", "gl") };

    // Parse CLI arguments (e.g., --mock-notifications 1000)
    parse_cli_args();

    let instance =
        SingleInstance::new(SINGLE_INSTANCE_MUTEX).expect("Failed to create single-instance mutex");

    if !instance.is_single() {
        platform::focus_existing_window();
        return Ok(());
    }

    platform::enable_dark_mode();

    // Initialize tray subsystem (GTK on Linux)
    platform::init_tray();

    let _tray = match tray::TrayManager::new() {
        Ok(t) => Some(t),
        Err(e) => {
            eprintln!("Tray unavailable: {e}");
            None
        }
    };

    let settings = AppSettings::load();

    let window_size = if settings.window_width >= MIN_VALID_WINDOW_SIZE
        && settings.window_height >= MIN_VALID_WINDOW_SIZE
    {
        Size::new(settings.window_width, settings.window_height)
    } else {
        Size::new(800.0, 640.0)
    };

    let window_position = match (settings.window_x, settings.window_y) {
        (Some(x), Some(y)) if x > MIN_VALID_WINDOW_POS && y > MIN_VALID_WINDOW_POS => {
            Position::Specific(Point::new(x as f32, y as f32))
        }
        _ => Position::Centered,
    };

    // Load window icon for title bar
    let window_icon = load_window_icon();

    // Window settings with icon and platform-specific configuration
    let window_settings = iced::window::Settings {
        size: window_size,
        position: window_position,
        icon: window_icon,
        #[cfg(target_os = "linux")]
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: "gittop".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    application(App::new, App::update, App::view)
        .title(|app: &App| app.title())
        .theme(|app: &App| app.theme())
        .subscription(App::subscription)
        .window(window_settings)
        .antialiasing(true)
        .default_font(Font::DEFAULT)
        .exit_on_close_request(false)
        .run()
}
