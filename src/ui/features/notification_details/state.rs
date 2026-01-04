//! Notification details state.
//!
//! This feature handles the details panel for viewing notification content:
//! - Selecting a notification to view details
//! - Loading details from the API
//! - Opening the notification in browser

use crate::github::NotificationSubjectDetail;

/// State for the notification details panel.
#[derive(Debug, Clone, Default)]
pub struct NotificationDetailsState {
    pub selected_id: Option<String>,
    pub details: Option<NotificationSubjectDetail>,
    pub is_loading: bool,
}

impl NotificationDetailsState {
    pub fn new() -> Self {
        Self::default()
    }
}
