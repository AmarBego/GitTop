//! Notification screen messages.
//!
//! This module defines the top-level message enum for the notifications screen.
//! Screen-level messages are routing wrappers only - actual behavior is handled by features.

use crate::github::{GitHubError, NotificationView, SubjectType};
use crate::ui::features::bulk_actions::BulkActionMessage;
use crate::ui::features::notification_details::NotificationDetailsMessage;
use crate::ui::features::thread_actions::ThreadActionMessage;

/// Top-level message for the notifications screen.
///
/// This enum routes messages to the appropriate handler.
/// Feature wrappers delegate to feature modules.
/// Screen-level messages handle routing and lifecycle only.
#[derive(Debug, Clone)]
pub enum NotificationMessage {
    // === Lifecycle Messages ===
    /// Trigger a refresh of notifications from the API.
    Refresh,
    /// Refresh completed with result.
    RefreshComplete(Result<Vec<NotificationView>, GitHubError>),

    // === Feature Wrappers ===
    /// Thread action (open, mark read, mark done).
    Thread(ThreadActionMessage),
    /// Bulk action (multi-select, bulk operations).
    Bulk(BulkActionMessage),
    /// Notification details (selection, details loading).
    Details(NotificationDetailsMessage),

    // === UI State Messages ===
    /// Filter messages for type/repo selection.
    Filter(FilterMessage),
    /// View messages for scroll/group toggle.
    View(ViewMessage),
    /// Navigation messages (handled by parent App).
    Navigation(NavigationMessage),
}

#[derive(Debug, Clone)]
pub enum FilterMessage {
    ToggleShowAll,
    SelectType(Option<SubjectType>),
    SelectRepo(Option<String>),
}

/// View-related messages for UI state only.
///
/// Note: Selection and details loading moved to NotificationDetailsMessage.
#[derive(Debug, Clone)]
pub enum ViewMessage {
    ToggleGroup(usize),
    OnScroll(iced::widget::scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub enum NavigationMessage {
    Logout,
    OpenSettings,
    OpenRuleEngine,
    SwitchAccount(String),
    TogglePowerMode,
}
