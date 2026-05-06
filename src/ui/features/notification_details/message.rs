//! Notification details messages.

use crate::github::subject_details::{CheckRunsResponse, CommentDetails, Label};
use crate::github::{GitHubError, NotificationSubjectDetail};

#[derive(Debug, Clone)]
pub enum NotificationDetailsMessage {
    Select(String),
    SelectComplete(String, Result<NotificationSubjectDetail, GitHubError>),
    OpenInBrowser,
    SetLabelInput(String),
    SubmitAddLabel(String, String, u64, String),
    SubmitLabelFromInput(String, String, u64),
    AddLabelComplete(String, Result<Vec<Label>, GitHubError>),
    RemoveLabel(String, String, u64, String),
    RemoveLabelComplete(String, Result<(), GitHubError>),
    RepoLabelsLoaded(Result<Vec<Label>, GitHubError>),
    RefreshChecks(String, String, String),
    ChecksLoaded(Result<CheckRunsResponse, GitHubError>),
    LoadComments(String),
    CommentsLoaded(Result<Vec<CommentDetails>, GitHubError>),
}
