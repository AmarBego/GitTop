mod engine;
pub mod helper;
pub mod messages;
mod screen;
mod view;

// Public API exports for external consumers
#[allow(unused_imports)]
pub use engine::{DesktopNotificationBatch, NotificationEngine};
pub use screen::NotificationsScreen;
