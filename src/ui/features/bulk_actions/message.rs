//! Bulk action messages.

use crate::github::GitHubError;

#[derive(Debug, Clone)]
pub enum BulkActionMessage {
    ToggleMode,
    ToggleSelect(String),
    SelectAll(Vec<String>),
    Clear,
    MarkAsRead,
    MarkAsDone,
    Complete(Result<(), GitHubError>),
}
