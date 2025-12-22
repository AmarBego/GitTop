//! GitHub notification types matching the GitHub API v3 schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// GitHub user information returned after successful authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
}

/// A GitHub notification from the notifications API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub unread: bool,
    pub reason: NotificationReason,
    pub updated_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub subject: NotificationSubject,
    pub repository: Repository,
    pub url: String,
}

/// The subject of a notification (issue, PR, release, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSubject {
    pub title: String,
    pub url: Option<String>,
    pub latest_comment_url: Option<String>,
    #[serde(rename = "type")]
    pub subject_type: SubjectType,
}

/// The type of notification subject.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, Hash)]
pub enum SubjectType {
    Issue,
    PullRequest,
    Release,
    Discussion,
    CheckSuite,
    Commit,
    RepositoryVulnerabilityAlert,
    /// Catch-all for unknown types from GitHub.
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for SubjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl SubjectType {
    /// Returns a human-readable label for the subject type.
    #[allow(unused)]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Issue => "Issue",
            Self::PullRequest => "Pull Request",
            Self::Release => "Release",
            Self::Discussion => "Discussion",
            Self::CheckSuite => "CI",
            Self::Commit => "Commit",
            Self::RepositoryVulnerabilityAlert => "Security",
            Self::Unknown => "Notification",
        }
    }
}

/// Why the user received this notification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum NotificationReason {
    /// You were requested to review and approve a deployment.
    ApprovalRequested,
    /// You were assigned to the issue.
    Assign,
    /// You created the thread.
    Author,
    /// A GitHub Actions workflow run that you triggered was completed.
    CiActivity,
    /// You commented on the thread.
    Comment,
    /// You accepted an invitation to contribute to the repository.
    Invitation,
    /// You subscribed to the thread (via an issue or pull request).
    Manual,
    /// Organization members have requested to enable a feature such as Copilot.
    MemberFeatureRequested,
    /// You were specifically @mentioned in the content.
    Mention,
    /// You, or a team you're a member of, were requested to review a pull request.
    ReviewRequested,
    /// You were credited for contributing to a security advisory.
    SecurityAdvisoryCredit,
    /// GitHub discovered a security vulnerability in your repository.
    SecurityAlert,
    /// You changed the thread state (for example, closing an issue or merging a PR).
    StateChange,
    /// You're watching the repository.
    Subscribed,
    /// You were on a team that was mentioned.
    TeamMention,
    /// Catch-all for unknown reasons.
    #[serde(other)]
    Unknown,
}

impl NotificationReason {
    pub const ALL: &'static [Self] = &[
        Self::ApprovalRequested,
        Self::Assign,
        Self::Author,
        Self::CiActivity,
        Self::Comment,
        Self::Invitation,
        Self::Manual,
        Self::MemberFeatureRequested,
        Self::Mention,
        Self::ReviewRequested,
        Self::SecurityAdvisoryCredit,
        Self::SecurityAlert,
        Self::StateChange,
        Self::Subscribed,
        Self::TeamMention,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::ApprovalRequested => "Approval Requested",
            Self::Assign => "Assigned",
            Self::Author => "Author",
            Self::CiActivity => "CI Activity",
            Self::Comment => "Commented",
            Self::Invitation => "Invited",
            Self::Manual => "Subscribed",
            Self::MemberFeatureRequested => "Feature Requested",
            Self::Mention => "Mentioned",
            Self::ReviewRequested => "Review Requested",
            Self::SecurityAdvisoryCredit => "Security Credit",
            Self::SecurityAlert => "Security Alert",
            Self::StateChange => "State Change",
            Self::Subscribed => "Watching",
            Self::TeamMention => "Team Mention",
            Self::Unknown => "Notification",
        }
    }
}

impl std::fmt::Display for NotificationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Repository information from the notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub owner: Owner,
    pub html_url: String,
    pub private: bool,
}

/// Repository owner (user or organization).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub login: String,
    pub avatar_url: String,
}

/// Frontend-friendly notification format for the UI.
#[derive(Debug, Clone, PartialEq)]
pub struct NotificationView {
    pub id: String,
    pub title: String,
    #[allow(dead_code)] // Reserved for enhanced UI (repo badge)
    pub repo_name: String,
    pub repo_full_name: String,
    pub subject_type: SubjectType,
    pub reason: NotificationReason,
    pub unread: bool,
    pub updated_at: DateTime<Utc>,
    pub time_ago: String,
    pub url: Option<String>,
    #[allow(dead_code)] // Reserved for comment preview feature
    pub latest_comment_url: Option<String>,
    #[allow(dead_code)] // Reserved for avatar display
    pub avatar_url: String,
    #[allow(dead_code)] // Reserved for private repo indicator
    pub is_private: bool,
    /// The GitHub account (username) this notification belongs to.
    pub account: String,
}

impl NotificationView {
    /// Create a NotificationView from a Notification with the account name.
    pub fn from_notification(n: Notification, account: impl Into<String>) -> Self {
        Self {
            id: n.id,
            title: n.subject.title,
            repo_name: n.repository.name,
            repo_full_name: n.repository.full_name.clone(),
            subject_type: n.subject.subject_type,
            reason: n.reason,
            unread: n.unread,
            updated_at: n.updated_at,
            time_ago: format_time_ago(n.updated_at),
            url: n.subject.url,
            latest_comment_url: n.subject.latest_comment_url,
            avatar_url: n.repository.owner.avatar_url,
            is_private: n.repository.private,
            account: account.into(),
        }
    }

    /// Get the repository owner (org or user) from the full name.
    pub fn repo_owner(&self) -> &str {
        self.repo_full_name
            .split('/')
            .next()
            .unwrap_or(&self.repo_full_name)
    }
}

impl From<Notification> for NotificationView {
    fn from(n: Notification) -> Self {
        Self::from_notification(n, String::new())
    }
}

fn format_time_ago(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_minutes() < 1 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d", duration.num_days())
    } else {
        dt.format("%b %d").to_string()
    }
}
