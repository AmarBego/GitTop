//! Type rule form state.

use std::collections::HashSet;

use crate::github::types::NotificationReason;
use crate::ui::screens::settings::rule_engine::rules::RuleAction;

/// State for the type rule creation form and grouping.
#[derive(Debug, Clone)]
pub struct TypeRuleFormState {
    pub notification_type: NotificationReason,
    pub account: Option<String>,
    pub priority: i32,
    pub action: RuleAction,
    pub expanded_groups: HashSet<String>,
}

impl Default for TypeRuleFormState {
    fn default() -> Self {
        Self {
            notification_type: NotificationReason::Mention,
            account: None,
            priority: 0,
            action: RuleAction::Show,
            expanded_groups: HashSet::new(),
        }
    }
}

impl TypeRuleFormState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset form to defaults after adding a rule.
    pub fn reset_form(&mut self) {
        self.account = None;
        self.priority = 0;
        self.action = RuleAction::Show;
    }
}
