//! Notification screen messages.

use crate::github::{GitHubError, NotificationView, SubjectType};

/// Notifications screen messages.
#[derive(Debug, Clone)]
#[allow(dead_code)] // MarkAsRead/MarkAsDone/MuteThread have handlers, pending UI buttons
pub enum NotificationMessage {
    Refresh,
    RefreshComplete(Result<Vec<NotificationView>, GitHubError>),
    Open(String),
    MarkAsRead(String),
    MarkAsReadComplete(String, Result<(), GitHubError>),
    MarkAllAsRead,
    MarkAllAsReadComplete(Result<(), GitHubError>),
    ToggleShowAll,
    Logout,
    ToggleGroup(usize),
    // Filter actions
    SelectType(Option<SubjectType>),
    SelectRepo(Option<String>),
    // Thread actions
    MarkAsDone(String),
    MarkAsDoneComplete(String, Result<(), GitHubError>),
    MuteThread(String),
    MuteThreadComplete(String, Result<(), GitHubError>),
    // Navigation
    OpenSettings,
    OpenRuleEngine,
    // Account switching (handled by app.rs)
    SwitchAccount(String),
    TogglePowerMode,
    /// Virtual scrolling: scroll position changed
    OnScroll(iced::widget::scrollable::Viewport),
    /// Select a notification to view in details panel (power mode)
    SelectNotification(String),
    /// Notification details fetch completed
    SelectComplete(
        String,
        Result<crate::github::NotificationSubjectDetail, GitHubError>,
    ),
    /// Open selected notification's URL in browser (from details panel)
    OpenInBrowser,
    // Bulk actions (Power Mode only)
    /// Toggle bulk selection mode
    ToggleBulkMode,
    /// Toggle selection of a single notification
    ToggleSelect(String),
    /// Select all visible notifications
    SelectAll,
    /// Clear all selections
    ClearSelection,
    /// Bulk mark selected as read
    BulkMarkAsRead,
    /// Bulk mark selected as done (archive)
    BulkMarkAsDone,
    /// Bulk action completed (no-op, just to complete the Task)
    BulkActionComplete,
}
