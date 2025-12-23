//! Window state management helpers.
//!
//! Provides thread-safe access to window state like ID and visibility.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use iced::window::Id as WindowId;
use iced::{Task, window};

/// Global storage for the main window ID.
static MAIN_WINDOW_ID: OnceLock<WindowId> = OnceLock::new();

/// Track if window is currently hidden (minimized to tray).
static IS_WINDOW_HIDDEN: AtomicBool = AtomicBool::new(false);

/// Store the main window ID (called on first window event).
pub fn set_window_id(id: WindowId) {
    let _ = MAIN_WINDOW_ID.set(id);
}

/// Get the main window ID if set.
pub fn get_window_id() -> Option<WindowId> {
    MAIN_WINDOW_ID.get().copied()
}

/// Check if the window is currently hidden.
pub fn is_hidden() -> bool {
    IS_WINDOW_HIDDEN.load(Ordering::Relaxed)
}

/// Set the window as hidden.
pub fn set_hidden(hidden: bool) {
    IS_WINDOW_HIDDEN.store(hidden, Ordering::Relaxed);
}

/// Set hidden to false and return the previous value.
/// Useful for detecting if we're restoring from hidden state.
pub fn restore_from_hidden() -> bool {
    IS_WINDOW_HIDDEN.swap(false, Ordering::Relaxed)
}

/// Resize window for power mode layout (1410×700).
pub fn resize_for_power_mode<T: Send + 'static>() -> Task<T> {
    if let Some(id) = get_window_id() {
        window::resize::<T>(id, iced::Size::new(1410.0, 700.0)).discard()
    } else {
        Task::none()
    }
}
