//! Notification Engine - centralized rule evaluation and notification processing.
//!
//! This module addresses the "Double Evaluation of Rules" architectural issue by:
//! 1. Providing a single entry point for rule evaluation
//! 2. Processing notifications once and storing the results
//! 3. Offering unified access for both desktop notifications and UI display
//!
//! Architecture:
//! - `NotificationEngine`: Owns the RuleEngine and provides evaluation methods
//! - `process_all()`: Single-pass processing of all notifications
//! - `should_notify_desktop()`: Checks if a processed notification should trigger desktop alert

use chrono::{DateTime, Local, Utc};
use std::collections::HashMap;

use crate::github::types::NotificationView;
use crate::ui::screens::settings::rule_engine::{NotificationRuleSet, RuleAction, RuleEngine};

use super::helper::ProcessedNotification;

// ============================================================================
// Notification Engine
// ============================================================================

/// Centralized notification processing engine.
///
/// Encapsulates all rule evaluation logic, ensuring notifications are
/// processed exactly once per refresh cycle.
pub struct NotificationEngine {
    engine: RuleEngine,
    /// Cached timestamp for consistent evaluation within a cycle.
    evaluation_time: DateTime<Local>,
}

impl NotificationEngine {
    /// Create a new engine from a rule set.
    pub fn new(rules: NotificationRuleSet) -> Self {
        Self {
            engine: RuleEngine::new(rules),
            evaluation_time: Local::now(),
        }
    }

    /// Process all notifications through the rule engine in a single pass.
    ///
    /// Returns processed notifications with their actions, filtering out hidden ones.
    /// This is the primary entry point - call this ONCE per refresh cycle.
    pub fn process_all(&self, notifications: &[NotificationView]) -> Vec<ProcessedNotification> {
        notifications
            .iter()
            .filter_map(|n| self.evaluate_single(n))
            .collect()
    }

    /// Evaluate a single notification and return its processed form.
    ///
    /// Returns `None` if the notification should be hidden.
    fn evaluate_single(&self, notification: &NotificationView) -> Option<ProcessedNotification> {
        // Extract reason label consistently - this is the canonical way to get
        // the notification type for rule matching
        let reason_label = Self::extract_reason_label(notification);

        let (action, _decision) = self.engine.evaluate_detailed(
            reason_label,
            Some(notification.repo_owner()),
            Some(&notification.account),
            &self.evaluation_time,
        );

        // Filter out hidden notifications
        if action == RuleAction::Hide {
            None
        } else {
            Some(ProcessedNotification {
                notification: notification.clone(),
                action,
            })
        }
    }

    /// Extract the reason label from a notification.
    ///
    /// This is the single source of truth for how notification reasons
    /// are converted to strings for rule matching. Previously this logic
    /// was duplicated in `process_with_rules` and `send_desktop_notifications`.
    #[inline]
    pub fn extract_reason_label(notification: &NotificationView) -> &str {
        notification.reason.label()
    }

    /// Check if a processed notification should trigger a desktop notification.
    ///
    /// A notification triggers a desktop alert if:
    /// 1. It's unread
    /// 2. It's new or updated (based on seen_timestamps)
    /// 3. Its action is Show or Priority (not Silent or Hide)
    pub fn should_notify_desktop(
        processed: &ProcessedNotification,
        seen_timestamps: &HashMap<String, DateTime<Utc>>,
    ) -> bool {
        let notif = &processed.notification;

        // Must be unread
        if !notif.unread {
            return false;
        }

        // Check if notification is new/updated
        let is_new = match seen_timestamps.get(&notif.id) {
            None => true,                                     // Never seen this ID
            Some(last_seen) => notif.updated_at > *last_seen, // Updated since last seen
        };

        if !is_new {
            return false;
        }

        // Only Show and Priority actions trigger desktop notifications
        matches!(processed.action, RuleAction::Show | RuleAction::Priority)
    }

    /// Check if a processed notification is a priority notification.
    #[inline]
    pub fn is_priority(processed: &ProcessedNotification) -> bool {
        processed.action == RuleAction::Priority
    }
}

// ============================================================================
// Desktop Notification Helpers
// ============================================================================

/// Categorized notifications for desktop alerts.
pub struct DesktopNotificationBatch<'a> {
    /// Priority notifications (always shown prominently)
    pub priority: Vec<&'a ProcessedNotification>,
    /// Regular notifications (Show action)
    pub regular: Vec<&'a ProcessedNotification>,
}

impl<'a> DesktopNotificationBatch<'a> {
    /// Create a batch from processed notifications, filtering for desktop-worthy ones.
    pub fn from_processed(
        processed: &'a [ProcessedNotification],
        seen_timestamps: &HashMap<String, DateTime<Utc>>,
    ) -> Self {
        let desktop_worthy: Vec<_> = processed
            .iter()
            .filter(|p| NotificationEngine::should_notify_desktop(p, seen_timestamps))
            .collect();

        let priority: Vec<_> = desktop_worthy
            .iter()
            .filter(|p| NotificationEngine::is_priority(p))
            .copied()
            .collect();

        let regular: Vec<_> = desktop_worthy
            .iter()
            .filter(|p| !NotificationEngine::is_priority(p))
            .copied()
            .collect();

        Self { priority, regular }
    }

    /// Check if there are any notifications to send.
    pub fn is_empty(&self) -> bool {
        self.priority.is_empty() && self.regular.is_empty()
    }

    /// Total count of notifications.
    pub fn total_count(&self) -> usize {
        self.priority.len() + self.regular.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::types::{NotificationReason, SubjectType};

    fn make_notification(id: &str, unread: bool, reason: NotificationReason) -> NotificationView {
        NotificationView {
            id: id.to_string(),
            unread,
            reason,
            updated_at: Utc::now(),
            title: "Test".to_string(),
            repo_name: "repo".to_string(),
            repo_full_name: "owner/repo".to_string(),
            url: None,
            latest_comment_url: None,
            avatar_url: "https://example.com/avatar.png".to_string(),
            is_private: false,
            subject_type: SubjectType::Issue,
            account: "testuser".to_string(),
            time_ago: "1m".to_string(),
        }
    }

    #[test]
    fn test_engine_filters_hidden() {
        let rules = NotificationRuleSet::default();
        let engine = NotificationEngine::new(rules);

        let notifications = vec![
            make_notification("1", true, NotificationReason::Mention),
            make_notification("2", true, NotificationReason::Subscribed),
        ];

        let processed = engine.process_all(&notifications);

        // Without any rules enabled, all should show
        assert_eq!(processed.len(), 2);
        assert!(processed.iter().all(|p| p.action == RuleAction::Show));
    }

    #[test]
    fn test_should_notify_desktop_new() {
        let rules = NotificationRuleSet::default();
        let engine = NotificationEngine::new(rules);
        let seen: HashMap<String, DateTime<Utc>> = HashMap::new();

        let notif = make_notification("1", true, NotificationReason::Mention);
        let processed = engine.process_all(&[notif]);

        // New unread notification should trigger desktop
        assert!(NotificationEngine::should_notify_desktop(
            &processed[0],
            &seen
        ));
    }

    #[test]
    fn test_should_notify_desktop_seen() {
        let rules = NotificationRuleSet::default();
        let engine = NotificationEngine::new(rules);

        let notif = make_notification("1", true, NotificationReason::Mention);
        let mut seen: HashMap<String, DateTime<Utc>> = HashMap::new();
        seen.insert("1".to_string(), notif.updated_at); // Already seen at current timestamp

        let processed = engine.process_all(&[notif]);

        // Already seen notification should NOT trigger desktop
        assert!(!NotificationEngine::should_notify_desktop(
            &processed[0],
            &seen
        ));
    }

    #[test]
    fn test_should_notify_desktop_read() {
        let rules = NotificationRuleSet::default();
        let engine = NotificationEngine::new(rules);
        let seen: HashMap<String, DateTime<Utc>> = HashMap::new();

        let notif = make_notification("1", false, NotificationReason::Mention); // Read
        let processed = engine.process_all(&[notif]);

        // Read notification should NOT trigger desktop
        assert!(!NotificationEngine::should_notify_desktop(
            &processed[0],
            &seen
        ));
    }
}
