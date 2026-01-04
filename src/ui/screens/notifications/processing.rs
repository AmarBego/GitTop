use crate::github::{NotificationView, SubjectType};
use crate::ui::screens::settings::rule_engine::{NotificationRuleSet, RuleAction};

use super::engine::NotificationEngine;
use super::helper::{
    FilterSettings, NotificationGroup, ProcessedNotification, apply_filters, count_by_repo,
    count_by_type, group_processed_notifications,
};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProcessingState {
    pub all_notifications: Vec<NotificationView>,
    pub filtered_notifications: Vec<NotificationView>,
    pub processed_notifications: Vec<ProcessedNotification>,
    pub groups: Vec<NotificationGroup>,
    pub rules: NotificationRuleSet,
    pub cross_account_priority: Vec<ProcessedNotification>,
    pub type_counts: Vec<(SubjectType, usize)>,
    pub repo_counts: Vec<(String, usize)>,
}

impl ProcessingState {
    pub fn new() -> Self {
        Self {
            all_notifications: Vec::new(),
            filtered_notifications: Vec::new(),
            processed_notifications: Vec::new(),
            groups: Vec::new(),
            rules: NotificationRuleSet::load(),
            cross_account_priority: Vec::new(),
            type_counts: Vec::new(),
            repo_counts: Vec::new(),
        }
    }

    pub fn enter_low_memory_mode(&mut self) {
        self.all_notifications = Vec::new();
        self.filtered_notifications = Vec::new();
        self.processed_notifications = Vec::new();
        self.groups = Vec::new();
        self.type_counts = Vec::new();
        self.repo_counts = Vec::new();
        self.cross_account_priority = Vec::new();
    }

    pub fn rebuild_groups(&mut self, filters: &mut FilterSettings, current_account: &str) {
        let notifications_for_types: Vec<_> = if let Some(ref repo) = filters.selected_repo {
            self.all_notifications
                .iter()
                .filter(|n| &n.repo_full_name == repo)
                .cloned()
                .collect()
        } else {
            self.all_notifications.clone()
        };

        let notifications_for_repos: Vec<_> = if let Some(ref selected_type) = filters.selected_type
        {
            self.all_notifications
                .iter()
                .filter(|n| &n.subject_type == selected_type)
                .cloned()
                .collect()
        } else {
            self.all_notifications.clone()
        };

        self.type_counts = count_by_type(&notifications_for_types);
        self.repo_counts = count_by_repo(&notifications_for_repos);

        if let Some(ref selected_type) = filters.selected_type {
            let type_valid = self
                .type_counts
                .iter()
                .any(|(t, c)| t == selected_type && *c > 0);
            if !type_valid {
                filters.selected_type = None;
            }
        }
        if let Some(ref selected_repo) = filters.selected_repo {
            let repo_valid = self
                .repo_counts
                .iter()
                .any(|(r, c)| r == selected_repo && *c > 0);
            if !repo_valid {
                filters.selected_repo = None;
            }
        }

        self.process_notifications(filters);
        self.update_cross_account_priority(current_account);

        let all_processed = if filters.show_all {
            self.processed_notifications.clone()
        } else {
            let other_account_priority: Vec<ProcessedNotification> = self
                .cross_account_priority
                .iter()
                .filter(|p| p.notification.account != *current_account && p.notification.unread)
                .cloned()
                .collect();

            let mut combined = self.processed_notifications.clone();
            for p in other_account_priority {
                if !combined
                    .iter()
                    .any(|existing| existing.notification.id == p.notification.id)
                {
                    combined.push(p);
                }
            }
            combined
        };

        let previous_expansion: HashMap<String, bool> = self
            .groups
            .iter()
            .map(|g| (g.title.clone(), g.is_expanded))
            .collect();

        let show_priority_group = !filters.show_all;
        self.groups = group_processed_notifications(&all_processed, show_priority_group);

        for group in &mut self.groups {
            if let Some(&was_expanded) = previous_expansion.get(&group.title) {
                group.is_expanded = was_expanded;
            }
        }
    }

    fn process_notifications(&mut self, filters: &FilterSettings) {
        let engine = NotificationEngine::new(self.rules.clone());
        self.filtered_notifications = apply_filters(&self.all_notifications, filters);
        self.processed_notifications = engine.process_all(&self.filtered_notifications);
    }

    fn update_cross_account_priority(&mut self, current_account: &str) {
        let current_priority: Vec<ProcessedNotification> = self
            .processed_notifications
            .iter()
            .filter(|p| p.action == RuleAction::Important && p.notification.unread)
            .cloned()
            .collect();

        self.cross_account_priority
            .retain(|p| p.notification.account != *current_account);
        self.cross_account_priority.extend(current_priority);
    }
}
