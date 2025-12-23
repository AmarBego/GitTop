//! Subject detail types for fetched notification content.
//!
//! When a notification is clicked in power mode, we fetch the actual
//! Issue/PR/Comment content from the GitHub API and display it in the
//! details panel.

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// GitHub user info (author, assignee, etc.)
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub login: String,
    pub avatar_url: String,
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
    #[serde(default)]
    pub assignees: Vec<User>,
    #[serde(rename = "comments")]
    pub comments_count: u64,
    pub html_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub mergeable: Option<bool>,
    #[serde(default)]
    pub additions: u64,
    #[serde(default)]
    pub deletions: u64,
    #[serde(default)]
    pub changed_files: u64,
    #[serde(default)]
    pub commits: u64,
    pub html_url: String,
    pub user: User,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fetched content for a Comment
#[derive(Debug, Clone, Deserialize)]
pub struct CommentDetails {
    pub body: String,
    pub html_url: String,
    pub created_at: DateTime<Utc>,
    pub user: User,
}

/// Discussion details (limited API support)
#[derive(Debug, Clone)]
pub struct DiscussionDetails {
    pub title: String,
    pub body: Option<String>,
    pub html_url: String,
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
        html_url: String,
    },
    /// Unsupported subject type (show link only)
    Unsupported {
        subject_type: String,
        html_url: Option<String>,
    },
}

impl NotificationSubjectDetail {
    /// Get the HTML URL for opening in browser
    pub fn html_url(&self) -> Option<&str> {
        match self {
            Self::Issue(i) => Some(&i.html_url),
            Self::PullRequest(pr) => Some(&pr.html_url),
            Self::Comment { comment, .. } => Some(&comment.html_url),
            Self::Discussion(d) => Some(&d.html_url),
            Self::SecurityAlert { html_url, .. } => Some(html_url),
            Self::Unsupported { html_url, .. } => html_url.as_deref(),
        }
    }
}
