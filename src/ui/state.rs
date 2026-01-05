//! Window state management helpers.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use iced::window::Id as WindowId;
use iced::{Task, window};

/// Power mode window dimensions
const POWER_MODE_WIDTH: f32 = 1410.0;
const POWER_MODE_HEIGHT: f32 = 700.0;

static MAIN_WINDOW_ID: Mutex<Option<WindowId>> = Mutex::new(None);
static IS_WINDOW_HIDDEN: AtomicBool = AtomicBool::new(false);
static IS_WINDOW_FOCUSED: AtomicBool = AtomicBool::new(true);

pub fn set_window_id(id: WindowId) {
    if let Ok(mut guard) = MAIN_WINDOW_ID.lock() {
        *guard = Some(id);
    }
}

pub fn get_window_id() -> Option<WindowId> {
    MAIN_WINDOW_ID.lock().ok().and_then(|guard| *guard)
}

pub fn is_hidden() -> bool {
    IS_WINDOW_HIDDEN.load(Ordering::Relaxed)
}

pub fn set_hidden(hidden: bool) {
    IS_WINDOW_HIDDEN.store(hidden, Ordering::Relaxed);
}

/// Set hidden to false and return the previous value.
pub fn restore_from_hidden() -> bool {
    IS_WINDOW_HIDDEN.swap(false, Ordering::Relaxed)
}

pub fn is_focused() -> bool {
    IS_WINDOW_FOCUSED.load(Ordering::Relaxed)
}

pub fn set_focused(focused: bool) {
    IS_WINDOW_FOCUSED.store(focused, Ordering::Relaxed);
}

pub fn resize_for_power_mode<T: Send + 'static>() -> Task<T> {
    get_window_id().map_or(Task::none(), |id| {
        window::resize::<T>(id, iced::Size::new(POWER_MODE_WIDTH, POWER_MODE_HEIGHT)).discard()
    })
}
