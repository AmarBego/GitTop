//! Notification screen messages.
//!
//! This module defines the top-level message enum for the notifications screen.
//! Screen-level messages are routing wrappers only - actual behavior is handled by features.

use crate::github::{GitHubError, NotificationView};
use crate::ui::features::bulk_actions::BulkActionMessage;
use crate::ui::features::notification_details::NotificationDetailsMessage;
use crate::ui::features::notification_list::NotificationListMessage;
use crate::ui::features::sidebar::{SidebarAction, SidebarMessage};
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
    /// Sidebar messages (navigation, filters).
    Sidebar(SidebarMessage),
    /// Sidebar actions (results of updates).
    SidebarAction(SidebarAction),

    // === UI State Messages ===
    /// Filter messages for type/repo selection.
    Filter(FilterMessage),
    /// View messages for scroll/group toggle.
    List(NotificationListMessage),
    /// Navigation messages (handled by parent App).
    Navigation(NavigationMessage),
    /// Dismiss crash report notice banner.
    DismissCrashNotice,
    /// Dismiss update available banner for this session.
    DismissUpdateBanner,
    /// Open the GitHub release page for the new version.
    OpenReleasePage,
}

#[derive(Debug, Clone)]
pub enum FilterMessage {
    ToggleShowAll,
}

#[derive(Debug, Clone)]
pub enum NavigationMessage {
    Logout,
    OpenSettings,
    OpenRuleEngine,
    SwitchAccount(String),
    TogglePowerMode,
}
