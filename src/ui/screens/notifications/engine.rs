//! Notification Engine - centralized rule evaluation.
//!
//! Architecture:
//! - `NotificationEngine`: Owns the RuleEngine and provides evaluation methods
//! - `process_all()`: Single-passes all notifications for a view refresh.
//! - `should_notify_desktop()`: Checks if we should annoy the user with a popup.
//!
//! Solves the "Double Evaluation" problem by processing once and storing results.

use chrono::{DateTime, Local, Utc};
use std::collections::HashMap;

use crate::github::types::NotificationView;
use crate::ui::screens::settings::rule_engine::{NotificationRuleSet, RuleAction, RuleEngine};

use super::helper::ProcessedNotification;

// ============================================================================
// Notification Engine
// ============================================================================

/// Encapsulates all rule evaluation logic, ensuring notifications are
/// processed exactly once per refresh cycle.
pub struct NotificationEngine {
    engine: RuleEngine,
    /// We cache this so every rule sees the EXACT same "now", avoiding race conditions
    /// or weird edge cases during a batch.
    evaluation_time: DateTime<Local>,
}

impl NotificationEngine {
    pub fn new(rules: NotificationRuleSet) -> Self {
        Self {
            engine: RuleEngine::new(rules),
            evaluation_time: Local::now(),
        }
    }

    /// Primary entry point. Call this ONCE per refresh cycle.
    pub fn process_all(&self, notifications: &[NotificationView]) -> Vec<ProcessedNotification> {
        notifications
            .iter()
            .filter_map(|n| self.evaluate_single(n))
            .collect()
    }

    fn evaluate_single(&self, notification: &NotificationView) -> Option<ProcessedNotification> {
        // This extraction is subtle we must use the exact same label as the rules expected.
        let reason_label = Self::extract_reason_label(notification);

        let (action, _decision) = self.engine.evaluate_detailed(
            reason_label,
            Some(notification.repo_owner()),
            Some(&notification.account),
            &self.evaluation_time,
        );

        // Filter out hidden notifications entirely from the UI view model
        if action == RuleAction::Hide {
            None
        } else {
            Some(ProcessedNotification {
                notification: notification.clone(),
                action,
            })
        }
    }

    /// Single source of truth for notification reason -> string conversion.
    #[inline]
    pub fn extract_reason_label(notification: &NotificationView) -> &str {
        notification.reason.label()
    }

    pub fn should_notify_desktop(
        processed: &ProcessedNotification,
        seen_timestamps: &HashMap<String, DateTime<Utc>>,
    ) -> bool {
        let notif = &processed.notification;

        // Logic: Unread AND (Never seen OR Updated since seen) AND (Show OR Important)
        notif.unread
            && seen_timestamps
                .get(&notif.id)
                .is_none_or(|last_seen| notif.updated_at > *last_seen)
            && matches!(processed.action, RuleAction::Show | RuleAction::Important)
    }
}

// ============================================================================
// Desktop Notification Helpers
// ============================================================================

pub struct DesktopNotificationBatch<'a> {
    /// Important notifications (always shown prominently)
    pub priority: Vec<&'a ProcessedNotification>,
    /// Regular notifications (Show action)
    pub regular: Vec<&'a ProcessedNotification>,
}

impl<'a> DesktopNotificationBatch<'a> {
    pub fn from_processed(
        processed: &'a [ProcessedNotification],
        seen_timestamps: &HashMap<String, DateTime<Utc>>,
    ) -> Self {
        let (priority, regular) = processed
            .iter()
            .filter(|p| NotificationEngine::should_notify_desktop(p, seen_timestamps))
            .partition(|p| (*p).is_priority());

        Self { priority, regular }
    }

    pub fn is_empty(&self) -> bool {
        self.priority.is_empty() && self.regular.is_empty()
    }

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
