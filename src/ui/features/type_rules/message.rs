//! Type rule messages.

use crate::github::types::NotificationReason;
use crate::ui::screens::settings::rule_engine::rules::RuleAction;

/// Messages for type rule operations.
#[derive(Debug, Clone)]
pub enum TypeRuleMessage {
    Toggle(String, bool),
    Delete(String),
    Duplicate(String),
    ToggleGroup(String),
    FormTypeChanged(NotificationReason),
    FormAccountChanged(String),
    FormPriorityChanged(i32),
    FormActionChanged(RuleAction),
    Add,
}
