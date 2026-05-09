//! Notification details state.
//!
//! This feature handles the details panel for viewing notification content:
//! - Selecting a notification to view details
//! - Loading details from the API
//! - Opening the notification in browser
//! - Managing PR/Issue labels (add and remove)

use crate::github::NotificationSubjectDetail;
use crate::github::subject_details::{CheckRun, CommentDetails, Label};
use iced::widget::image;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct NotificationDetailsState {
    pub selected_id: Option<String>,
    pub details: Option<NotificationSubjectDetail>,
    pub detail_error: Option<String>,
    pub is_loading: bool,
    pub label_input: String,
    pub available_labels: Vec<Label>,
    pub labels_loading: bool,
    pub pending_label_ops: HashSet<String>,
    pub label_error: Option<String>,
    pub check_runs: Vec<CheckRun>,
    pub checks_loading: bool,
    pub comments: Option<Vec<CommentDetails>>,
    pub comments_loading: bool,
    pub comments_error: Option<String>,
    pub avatar_handle: Option<image::Handle>,
}

impl NotificationDetailsState {
    pub fn new() -> Self {
        Self::default()
    }
}
