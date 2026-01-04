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
        }

        TypeRuleMessage::Delete(id) => {
            rules.type_rules.retain(|r| r.id != id);
            let _ = rules.save();
        }

        TypeRuleMessage::Duplicate(id) => {
            if let Some(rule) = rules.type_rules.iter().find(|r| r.id == id).cloned() {
                let mut new_rule = rule;
                new_rule.id = uuid::Uuid::new_v4().to_string();
                rules.type_rules.push(new_rule);
                let _ = rules.save();
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

            rules.type_rules.push(rule);
            let _ = rules.save();

            // Reset form
            state.reset_form();
        }
    }

    Task::none()
}
