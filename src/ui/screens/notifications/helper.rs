//! Notification screen helpers - grouping, filtering, URL conversion.

use crate::github::{NotificationView, SubjectType};
use crate::ui::screens::settings::rule_engine::{NotificationRuleSet, RuleAction, RuleEngine};
use chrono::Local;
use std::collections::HashMap;

/// A notification with its evaluated rule action.
#[derive(Debug, Clone)]
pub struct ProcessedNotification {
    pub notification: NotificationView,
    pub action: RuleAction,
}

/// Group of notifications by time period.
#[derive(Debug, Clone)]
pub struct NotificationGroup {
    pub title: String,
    pub notifications: Vec<ProcessedNotification>,
    pub is_expanded: bool,
    /// True if this is the priority group (always shown first, special styling).
    pub is_priority: bool,
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

/// Process notifications through the rule engine.
/// Returns processed notifications with their actions, filtering out hidden ones.
pub fn process_with_rules(
    notifications: &[NotificationView],
    rules: &NotificationRuleSet,
) -> Vec<ProcessedNotification> {
    if !rules.enabled {
        // No rules active - all notifications are shown normally
        return notifications
            .iter()
            .map(|n| ProcessedNotification {
                notification: n.clone(),
                action: RuleAction::Show,
            })
            .collect();
    }

    let engine = RuleEngine::new(rules.clone());
    let now = Local::now();

    notifications
        .iter()
        .filter_map(|n| {
            // Use the reason's label for matching (e.g., "Mentioned", "Commented")
            // This matches what the rule editor UI shows/saves
            let reason_label = n.reason.label();
            let (action, _decision) = engine.evaluate_detailed(
                reason_label,
                Some(n.repo_owner()),
                Some(&n.account),
                &now,
            );

            // Filter out hidden notifications
            if action == RuleAction::Hide {
                None
            } else {
                Some(ProcessedNotification {
                    notification: n.clone(),
                    action,
                })
            }
        })
        .collect()
}

/// Group processed notifications by time period (Today, This Week, Older).
/// Priority notifications are extracted into a separate group shown first (only if `show_priority_group` is true).
pub fn group_processed_notifications(
    processed: &[ProcessedNotification],
    show_priority_group: bool,
) -> Vec<NotificationGroup> {
    let now_date = Local::now().date_naive();
    let one_week_ago = now_date - chrono::Duration::days(7);

    let mut priority = Vec::new();
    let mut today = Vec::new();
    let mut this_week = Vec::new();
    let mut older = Vec::new();

    for p in processed {
        // Priority notifications go to their own group only if priority grouping is enabled
        if show_priority_group && p.action == RuleAction::Priority {
            priority.push(p.clone());
            continue;
        }

        // Otherwise, group by time (priority notifications also go here in "All" mode)
        let notif_date = p.notification.updated_at.with_timezone(&Local).date_naive();
        if notif_date >= now_date {
            today.push(p.clone());
        } else if notif_date >= one_week_ago {
            this_week.push(p.clone());
        } else {
            older.push(p.clone());
        }
    }

    let mut groups = Vec::new();

    // Priority group always first (if not empty and enabled)
    if show_priority_group && !priority.is_empty() {
        groups.push(NotificationGroup {
            title: "Priority".to_string(),
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

/// Group notifications by time period (Today, This Week, Older).
/// Legacy function - wraps notifications without rule processing.
#[allow(dead_code)]
pub fn group_by_time(notifications: &[NotificationView]) -> Vec<NotificationGroup> {
    let processed: Vec<_> = notifications
        .iter()
        .map(|n| ProcessedNotification {
            notification: n.clone(),
            action: RuleAction::Show,
        })
        .collect();
    group_processed_notifications(&processed, false) // No priority grouping for legacy
}

/// Apply filters to a list of notifications.
pub fn apply_filters(
    notifications: &[NotificationView],
    filters: &FilterSettings,
) -> Vec<NotificationView> {
    notifications
        .iter()
        .filter(|n| {
            // If not showing all, only show unread notifications
            if !filters.show_all && !n.unread {
                return false;
            }
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

    // Sort by count descending, then by name ascending for stability
    let mut result: Vec<_> = counts.into_iter().collect();
    result.sort_by(|a, b| {
        match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => a.0.cmp(&b.0), // alphabetical when equal
            other => other,
        }
    });
    result
}

/// Convert GitHub API URL to web URL.
pub fn api_url_to_web_url(api_url: &str) -> String {
    api_url
        .replace("api.github.com/repos", "github.com")
        .replace("/pulls/", "/pull/")
}
