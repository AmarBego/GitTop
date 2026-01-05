//! Platform event handlers - tick, tray, window events.

use iced::window::Id as WindowId;
use iced::{Task, exit, window};

use crate::settings::AppSettings;
use crate::tray::{TrayCommand, TrayManager};
use crate::ui::screens::notifications::NotificationsScreen;
use crate::ui::screens::notifications::messages::NotificationMessage;
use crate::ui::state;

use super::super::app::Message;

// ============================================================================
// Constants
// ============================================================================

/// Windows reports these values when window is minimized.
pub const MINIMIZED_POSITION_THRESHOLD: f32 = -10000.0;
pub const MINIMIZED_SIZE_THRESHOLD: f32 = 100.0;

/// Polling intervals in milliseconds.
pub const TRAY_POLL_INTERVAL_HIDDEN_MS: u64 = 500;
pub const TRAY_POLL_INTERVAL_ACTIVE_MS: u64 = 100;

/// Auto-refresh interval for notifications.
pub const REFRESH_INTERVAL_SECS: u64 = 60;

// ============================================================================
// Tick Handler
// ============================================================================

/// Handle periodic refresh tick.
pub fn handle_tick(screen: &mut NotificationsScreen) -> Task<Message> {
    if screen.is_loading {
        return Task::none();
    }
    screen
        .update(NotificationMessage::Refresh)
        .map(Message::Notifications)
}

// ============================================================================
// Tray Handler
// ============================================================================

/// Handle tray icon events.
pub fn handle_tray_poll(notification_screen: Option<&mut NotificationsScreen>) -> Task<Message> {
    let Some(cmd) = TrayManager::poll_global_events() else {
        return Task::none();
    };

    match cmd {
        TrayCommand::ShowWindow => {
            let was_hidden = state::restore_from_hidden();

            #[cfg(target_os = "linux")]
            let window_task = if was_hidden {
                let (id, open_task) = crate::platform::linux::build_initial_window_settings();
                state::set_window_id(id);
                open_task
            } else {
                state::get_window_id()
                    .map(window::gain_focus)
                    .unwrap_or_else(Task::none)
            };

            #[cfg(not(target_os = "linux"))]
            let window_task = state::get_window_id()
                .map(|id| {
                    Task::batch([
                        window::set_mode(id, window::Mode::Windowed),
                        window::gain_focus(id),
                    ])
                })
                .unwrap_or_else(Task::none);

            let refresh_task = was_hidden
                .then_some(notification_screen)
                .flatten()
                .map(|screen| {
                    screen
                        .update(NotificationMessage::Refresh)
                        .map(Message::Notifications)
                })
                .unwrap_or_else(Task::none);

            Task::batch([window_task, refresh_task])
        }
        TrayCommand::Quit => exit(),
    }
}

// ============================================================================
// Window Event Handler
// ============================================================================

/// Context needed for window event handling.
pub struct WindowEventContext<'a> {
    pub settings: Option<&'a mut AppSettings>,
    pub minimize_to_tray: bool,
    pub notification_screen: Option<&'a mut NotificationsScreen>,
}

/// Handle window events (moved, resized, close, focus).
pub fn handle_window_event(
    id: WindowId,
    event: window::Event,
    ctx: WindowEventContext<'_>,
) -> Task<Message> {
    state::set_window_id(id);

    match event {
        window::Event::CloseRequested => {
            if ctx.minimize_to_tray {
                enter_tray_mode(id, ctx.notification_screen)
            } else {
                exit()
            }
        }

        window::Event::Moved(position) => {
            let valid = position.x > MINIMIZED_POSITION_THRESHOLD
                && position.y > MINIMIZED_POSITION_THRESHOLD;

            if let Some(s) = valid.then_some(ctx.settings).flatten() {
                s.window_x = Some(position.x as i32);
                s.window_y = Some(position.y as i32);
                s.save_silent();
            }
            Task::none()
        }

        window::Event::Resized(size) => {
            let valid =
                size.width > MINIMIZED_SIZE_THRESHOLD && size.height > MINIMIZED_SIZE_THRESHOLD;

            if let Some(s) = valid.then_some(ctx.settings).flatten() {
                s.window_width = size.width;
                s.window_height = size.height;
                s.save_silent();
            }
            Task::none()
        }

        #[cfg(target_os = "linux")]
        window::Event::Closed => {
            if ctx.minimize_to_tray {
                state::set_hidden(true);
                if let Some(screen) = ctx.notification_screen {
                    screen.enter_low_memory_mode();
                }
                crate::platform::trim_memory();
                Task::none()
            } else {
                exit()
            }
        }

        window::Event::Focused => {
            state::set_focused(true);
            Task::none()
        }

        window::Event::Unfocused => {
            state::set_focused(false);
            Task::none()
        }

        _ => Task::none(),
    }
}

// ============================================================================
// Tray Mode
// ============================================================================

/// Enter tray mode: hide window and free memory.
pub fn enter_tray_mode(
    window_id: WindowId,
    notification_screen: Option<&mut NotificationsScreen>,
) -> Task<Message> {
    state::set_hidden(true);

    if let Some(screen) = notification_screen {
        screen.enter_low_memory_mode();
    }

    crate::platform::trim_memory();

    #[cfg(target_os = "linux")]
    {
        window::close(window_id)
    }

    #[cfg(not(target_os = "linux"))]
    {
        window::set_mode(window_id, window::Mode::Hidden)
    }
}
