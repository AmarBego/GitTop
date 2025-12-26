//! Architecture Notes:
//! - `ProcessedNotification` and `NotificationGroup` hold our view data.
//! - `group_processed_notifications` handles the presentation logic (time buckets).
//! - `apply_filters`, `count_by_type`, `count_by_repo` are just pure data transformations.
//!
//! Note: For rule evaluation, check `engine.rs` instead.

use crate::github::{NotificationView, SubjectType};
use crate::ui::screens::settings::rule_engine::RuleAction;
use chrono::Local;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProcessedNotification {
    pub notification: NotificationView,
    pub action: RuleAction,
}

impl ProcessedNotification {
    pub fn is_priority(&self) -> bool {
        self.action == RuleAction::Important
    }
}

#[derive(Debug, Clone)]
pub struct NotificationGroup {
    pub title: String,
    pub notifications: Vec<ProcessedNotification>,
    pub is_expanded: bool,
    /// We flag this so the UI knows to give it special styling and keep it at the top.
    pub is_priority: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FilterSettings {
    pub show_all: bool,
    /// None means "All Types"
    pub selected_type: Option<SubjectType>,
    /// None means "All Repos"
    pub selected_repo: Option<String>,
}

pub fn group_processed_notifications(
    processed: &[ProcessedNotification],
    show_priority_group: bool,
) -> Vec<NotificationGroup> {
    let now_date = Local::now().date_naive();
    let one_week_ago = now_date - chrono::Duration::days(7);

    // We do a single pass fold here instead of multiple filters so we don't have to
    // iterate over the list 4 times.
    let (priority, today, this_week, older) = processed.iter().fold(
        (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
        |(mut p, mut t, mut w, mut o), notif| {
            if show_priority_group && notif.action == RuleAction::Important {
                p.push(notif.clone());
            } else {
                let notif_date = notif
                    .notification
                    .updated_at
                    .with_timezone(&Local)
                    .date_naive();

                if notif_date >= now_date {
                    t.push(notif.clone());
                } else if notif_date >= one_week_ago {
                    w.push(notif.clone());
                } else {
                    o.push(notif.clone());
                }
            }
            (p, t, w, o)
        },
    );

    let mut groups = Vec::with_capacity(4);

    if show_priority_group && !priority.is_empty() {
        groups.push(NotificationGroup {
            title: "Important".to_string(),
            notifications: priority,
            is_expanded: true,
            is_priority: true,
        });
    }

    groups.push(NotificationGroup {
        title: "Today".to_string(),
        notifications: today,
        is_expanded: true,
        is_priority: false,
    });

    groups.push(NotificationGroup {
        title: "This Week".to_string(),
        notifications: this_week,
        is_expanded: true,
        is_priority: false,
    });

    groups.push(NotificationGroup {
        title: "Older".to_string(),
        notifications: older,
        is_expanded: false,
        is_priority: false,
    });

    groups
}

pub fn apply_filters(
    notifications: &[NotificationView],
    filters: &FilterSettings,
) -> Vec<NotificationView> {
    notifications
        .iter()
        .filter(|n| {
            let passes_read = filters.show_all || n.unread;
            let passes_type = filters
                .selected_type
                .as_ref()
                .is_none_or(|t| &n.subject_type == t);
            let passes_repo = filters
                .selected_repo
                .as_ref()
                .is_none_or(|r| &n.repo_full_name == r);
            passes_read && passes_type && passes_repo
        })
        .cloned()
        .collect()
}

const SUBJECT_TYPE_ORDER: &[SubjectType] = &[
    SubjectType::PullRequest,
    SubjectType::Issue,
    SubjectType::Commit,
    SubjectType::CheckSuite,
    SubjectType::Discussion,
    SubjectType::Release,
    SubjectType::RepositoryVulnerabilityAlert,
];

pub fn count_by_type(notifications: &[NotificationView]) -> Vec<(SubjectType, usize)> {
    let counts = notifications.iter().fold(HashMap::new(), |mut acc, n| {
        *acc.entry(n.subject_type).or_insert(0) += 1;
        acc
    });

    SUBJECT_TYPE_ORDER
        .iter()
        .filter_map(|t| counts.get(t).map(|&c| (*t, c)))
        .collect()
}

pub fn count_by_repo(notifications: &[NotificationView]) -> Vec<(String, usize)> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for n in notifications {
        *counts.entry(&n.repo_full_name).or_insert(0) += 1;
    }

    let mut result: Vec<_> = counts.into_iter().map(|(k, v)| (k.to_owned(), v)).collect();

    result.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    result
}

pub fn api_url_to_web_url(api_url: &str) -> String {
    api_url
        .replace("api.github.com/repos", "github.com")
        .replace("/pulls/", "/pull/")
}
