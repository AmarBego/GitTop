//! Notification details messages.

use crate::github::{GitHubError, NotificationSubjectDetail};

#[derive(Debug, Clone)]
pub enum NotificationDetailsMessage {
    Select(String),
    SelectComplete(String, Result<NotificationSubjectDetail, GitHubError>),
    OpenInBrowser,
}
