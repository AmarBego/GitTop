//! Notification screen helpers - grouping, filtering, URL conversion.

use crate::github::{NotificationView, SubjectType};
use chrono::Local;
use std::collections::HashMap;

/// Group of notifications by time period.
#[derive(Debug, Clone)]
pub struct NotificationGroup {
    pub title: String,
    pub notifications: Vec<NotificationView>,
    pub is_expanded: bool,
}

/// Filter settings for notification list.
#[derive(Debug, Clone, Default)]
pub struct FilterSettings {
    /// Show all notifications including read ones.
    pub show_all: bool,
    /// Filter by specific subject type (None = show all).
    pub selected_type: Option<SubjectType>,
    /// Filter by specific repository (None = show all).
    pub selected_repo: Option<String>,
}

/// Group notifications by time period (Today, This Week, Older).
pub fn group_by_time(notifications: &[NotificationView]) -> Vec<NotificationGroup> {
    let now_date = Local::now().date_naive();
    let one_week_ago = now_date - chrono::Duration::days(7);

    let mut today = Vec::new();
    let mut this_week = Vec::new();
    let mut older = Vec::new();

    for notif in notifications {
        let notif_date = notif.updated_at.with_timezone(&Local).date_naive();
        if notif_date >= now_date {
            today.push(notif.clone());
        } else if notif_date >= one_week_ago {
            this_week.push(notif.clone());
        } else {
            older.push(notif.clone());
        }
    }

    vec![
        NotificationGroup {
            title: "Today".to_string(),
            notifications: today,
            is_expanded: true,
        },
        NotificationGroup {
            title: "This Week".to_string(),
            notifications: this_week,
            is_expanded: true,
        },
        NotificationGroup {
            title: "Older".to_string(),
            notifications: older,
            is_expanded: false,
        },
    ]
}

/// Apply filters to a list of notifications.
pub fn apply_filters(
    notifications: &[NotificationView],
    filters: &FilterSettings,
) -> Vec<NotificationView> {
    notifications
        .iter()
        .filter(|n| {
            // Filter by subject type if specified
            if let Some(ref selected_type) = filters.selected_type {
                if &n.subject_type != selected_type {
                    return false;
                }
            }
            // Filter by repository if specified
            if let Some(ref selected_repo) = filters.selected_repo {
                if &n.repo_full_name != selected_repo {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

/// Count notifications by subject type.
pub fn count_by_type(notifications: &[NotificationView]) -> Vec<(SubjectType, usize)> {
    let mut counts: HashMap<SubjectType, usize> = HashMap::new();

    for n in notifications {
        *counts.entry(n.subject_type).or_insert(0) += 1;
    }

    // Return in a consistent order
    let order = [
        SubjectType::PullRequest,
        SubjectType::Issue,
        SubjectType::Commit,
        SubjectType::CheckSuite,
        SubjectType::Discussion,
        SubjectType::Release,
        SubjectType::RepositoryVulnerabilityAlert,
    ];

    order
        .iter()
        .filter_map(|t| counts.get(t).map(|c| (*t, *c)))
        .collect()
}

/// Count notifications by repository.
pub fn count_by_repo(notifications: &[NotificationView]) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for n in notifications {
        *counts.entry(n.repo_full_name.clone()).or_insert(0) += 1;
    }

    // Sort by count descending
    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1));
    result
}

/// Convert GitHub API URL to web URL.
pub fn api_url_to_web_url(api_url: &str) -> String {
    api_url
        .replace("api.github.com/repos", "github.com")
        .replace("/pulls/", "/pull/")
}
