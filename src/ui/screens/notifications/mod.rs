pub mod components;
pub mod desktop_notify;
mod engine;
pub mod helper;
pub mod messages;
pub mod processing;
mod screen;

// Public API exports for external consumers
#[allow(unused_imports)]
pub use engine::{DesktopNotificationBatch, NotificationEngine};
pub use screen::NotificationsScreen;
