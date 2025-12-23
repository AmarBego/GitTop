//! Notifications screen module.

mod engine;
pub mod helper;
pub mod messages;
mod screen;
mod view;

// Public API exports for external consumers
#[allow(unused_imports)]
pub use engine::{DesktopNotificationBatch, NotificationEngine};
pub use messages::NotificationMessage;
pub use screen::NotificationsScreen;
