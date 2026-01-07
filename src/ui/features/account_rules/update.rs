//! Account rule update logic.

use chrono::NaiveTime;
use iced::Task;

use crate::ui::screens::settings::rule_engine::rules::NotificationRuleSet;

use super::message::AccountRuleMessage;
use super::state::AccountRulesState;

/// Update account rule state based on message.
///
/// Returns Task::none() since all operations are synchronous.
pub fn update_account_rule(
    state: &mut AccountRulesState,
    message: AccountRuleMessage,
    rules: &mut NotificationRuleSet,
) -> Task<AccountRuleMessage> {
    match message {
        AccountRuleMessage::Select(id) => {
            state.selected_account_id = Some(id);
        }

        AccountRuleMessage::ToggleEnabled(id, enabled) => {
            if let Some(rule) = rules.account_rules.iter_mut().find(|r| r.id == id) {
                rule.enabled = enabled;
                let _ = rules.save();
                tracing::info!(
                    rule_id = %id,
                    enabled,
                    "Account rule enabled state updated"
                );
            }
        }

        AccountRuleMessage::ToggleDay(id, day) => {
            if let Some(rule) = rules.account_rules.iter_mut().find(|r| r.id == id) {
                if rule.active_days.contains(&day) {
                    rule.active_days.remove(&day);
                } else {
                    rule.active_days.insert(day);
                }
                let _ = rules.save();
                tracing::info!(
                    rule_id = %id,
                    day = ?day,
                    "Account rule schedule day toggled"
                );
            }
        }

        AccountRuleMessage::SetTimeWindow(id, start_str, end_str) => {
            if let Some(rule) = rules.account_rules.iter_mut().find(|r| r.id == id) {
                let start = start_str.and_then(|s| NaiveTime::parse_from_str(&s, "%H:%M").ok());
                let end = end_str.and_then(|s| NaiveTime::parse_from_str(&s, "%H:%M").ok());
                rule.start_time = start;
                rule.end_time = end;
                let _ = rules.save();
                tracing::debug!(
                    rule_id = %id,
                    has_start = start.is_some(),
                    has_end = end.is_some(),
                    "Account rule time window updated"
                );
            }
        }

        AccountRuleMessage::SetTimeWindowExpanded(id, expanded) => {
            if expanded {
                state.expanded_time_windows.insert(id);
            } else {
                state.expanded_time_windows.remove(&id);
            }
        }

        AccountRuleMessage::SetOutsideBehavior(id, behavior) => {
            if let Some(rule) = rules.account_rules.iter_mut().find(|r| r.id == id) {
                rule.outside_behavior = behavior;
                let _ = rules.save();
                tracing::info!(
                    rule_id = %id,
                    behavior = ?behavior,
                    "Account rule outside-schedule behavior updated"
                );
            }
        }
    }

    Task::none()
}
