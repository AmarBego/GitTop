//! Subject detail types for fetched notification content.
//!
//! When a notification is clicked in power mode, we fetch the actual
//! Issue/PR/Comment content from the GitHub API and display it in the
//! details panel.

use serde::Deserialize;

/// GitHub user info (author, assignee, etc.)
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub login: String,
}

/// Issue/PR label
#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub name: String,
    pub color: String,
}

/// Fetched content for an Issue
#[derive(Debug, Clone, Deserialize)]
pub struct IssueDetails {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(rename = "comments")]
    pub comments_count: u64,
    pub user: User,
}

/// Fetched content for a Pull Request
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestDetails {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    #[serde(default)]
    pub merged: bool,
    #[serde(default)]
    pub additions: u64,
    #[serde(default)]
    pub deletions: u64,
    #[serde(default)]
    pub changed_files: u64,
    #[serde(default)]
    pub commits: u64,
    pub user: User,
}

/// Fetched content for a Comment
#[derive(Debug, Clone, Deserialize)]
pub struct CommentDetails {
    pub body: String,
    pub user: User,
}

/// Discussion details (fetched via GraphQL API)
#[derive(Debug, Clone)]
pub struct DiscussionDetails {
    pub title: String,
    pub body: Option<String>,
    pub author: Option<String>,
    pub category: Option<DiscussionCategory>,
    pub answer_chosen: bool,
    pub comments_count: u64,
}

/// Discussion category
#[derive(Debug, Clone)]
pub struct DiscussionCategory {
    pub name: String,
    pub emoji: Option<String>,
}

/// Unified notification subject detail
#[derive(Debug, Clone)]
pub enum NotificationSubjectDetail {
    /// Full issue content
    Issue(IssueDetails),
    /// Full pull request content
    PullRequest(PullRequestDetails),
    /// Comment with context (for "mention" reason)
    Comment {
        comment: CommentDetails,
        context_title: String,
    },
    /// Discussion content
    Discussion(DiscussionDetails),
    /// Security alert (limited API - can't fetch full body)
    SecurityAlert {
        title: String,
        severity: Option<String>,
    },
    /// Unsupported subject type (show link only)
    Unsupported { subject_type: String },
}
