//! Thread action messages.

use crate::github::GitHubError;

#[derive(Debug, Clone)]
pub enum ThreadActionMessage {
    Open(String),
    MarkAsRead(String),
    MarkAsReadComplete(String, Result<(), GitHubError>),
    MarkAsDone(String),
    MarkAsDoneComplete(String, Result<(), GitHubError>),
    MarkAllAsRead,
    MarkAllAsReadComplete(Result<(), GitHubError>),
}
