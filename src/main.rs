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

use single_instance::SingleInstance;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Mutex name for single instance detection
const SINGLE_INSTANCE_MUTEX: &str = "GitTop-SingleInstance-Mutex-7a8b9c0d";

/// Global mock notification count (set via CLI)
pub static MOCK_NOTIFICATION_COUNT: AtomicUsize = AtomicUsize::new(0);

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

    let _tray = match tray::TrayManager::new() {
        Ok(t) => Some(t),
        Err(e) => {
            eprintln!("Tray unavailable: {e}");
            None
        }
    };

    platform::run_app()
}
