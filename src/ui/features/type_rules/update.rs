//! Type rule update logic.

use iced::Task;

use crate::ui::screens::settings::rule_engine::rules::{NotificationRuleSet, TypeRule};

use super::message::TypeRuleMessage;
use super::state::TypeRuleFormState;

/// Update type rule state based on message.
///
/// Returns Task::none() since all operations are synchronous.
pub fn update_type_rule(
    state: &mut TypeRuleFormState,
    message: TypeRuleMessage,
    rules: &mut NotificationRuleSet,
) -> Task<TypeRuleMessage> {
    match message {
        TypeRuleMessage::Toggle(id, enabled) => {
            if let Some(rule) = rules.type_rules.iter_mut().find(|r| r.id == id) {
                rule.enabled = enabled;
            }
            let _ = rules.save();
            tracing::info!(
                rule_id = %id,
                enabled,
                "Type rule enabled state updated"
            );
        }

        TypeRuleMessage::Delete(id) => {
            rules.type_rules.retain(|r| r.id != id);
            let _ = rules.save();
            tracing::info!(rule_id = %id, "Type rule deleted");
        }

        TypeRuleMessage::Duplicate(id) => {
            if let Some(rule) = rules.type_rules.iter().find(|r| r.id == id).cloned() {
                let mut new_rule = rule;
                new_rule.id = uuid::Uuid::new_v4().to_string();
                let new_id = new_rule.id.clone();
                rules.type_rules.push(new_rule);
                let _ = rules.save();
                tracing::info!(
                    source_rule_id = %id,
                    new_rule_id = %new_id,
                    "Type rule duplicated"
                );
            }
        }

        TypeRuleMessage::ToggleGroup(group_name) => {
            if state.expanded_groups.contains(&group_name) {
                state.expanded_groups.remove(&group_name);
            } else {
                state.expanded_groups.insert(group_name);
            }
        }

        TypeRuleMessage::FormTypeChanged(s) => {
            state.notification_type = s;
        }

        TypeRuleMessage::FormAccountChanged(s) => {
            state.account = if s == "Global" || s.trim().is_empty() {
                None
            } else {
                Some(s)
            };
        }

        TypeRuleMessage::FormPriorityChanged(p) => {
            state.priority = p;
        }

        TypeRuleMessage::FormActionChanged(a) => {
            state.action = a;
        }

        TypeRuleMessage::Add => {
            let mut rule = TypeRule::new(
                state.notification_type.label(),
                state.account.clone(),
                state.priority,
            );
            rule.action = state.action;

            let account_scoped = rule.account.is_some();
            let rule_id = rule.id.clone();
            let action = rule.action;
            let priority = rule.priority;
            let notification_type = rule.notification_type.clone();

            rules.type_rules.push(rule);
            let _ = rules.save();

            // Reset form
            state.reset_form();

            tracing::info!(
                rule_id = %rule_id,
                notification_type = %notification_type,
                action = ?action,
                priority,
                account_scoped,
                "Type rule added"
            );
        }
    }

    Task::none()
}
