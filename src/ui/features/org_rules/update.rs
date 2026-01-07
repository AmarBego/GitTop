use super::message::OrgMessage;
use super::state::OrgRulesState;
use crate::ui::screens::settings::rule_engine::rules::NotificationRuleSet;
use iced::Task;

pub fn update(
    _state: &mut OrgRulesState,
    message: OrgMessage,
    rules: &mut NotificationRuleSet,
) -> Task<OrgMessage> {
    match message {
        OrgMessage::Toggle(id, enabled) => {
            if let Some(rule) = rules.org_rules.iter_mut().find(|r| r.id == id) {
                rule.enabled = enabled;
            }
            let _ = rules.save();
            tracing::info!(
                rule_id = %id,
                enabled,
                "Org rule enabled state updated"
            );
        }
        OrgMessage::Delete(id) => {
            rules.org_rules.retain(|r| r.id != id);
            let _ = rules.save();
            tracing::info!(rule_id = %id, "Org rule deleted");
        }
        OrgMessage::Duplicate(id) => {
            if let Some(rule) = rules.org_rules.iter().find(|r| r.id == id).cloned() {
                let mut new_rule = rule;
                new_rule.id = uuid::Uuid::new_v4().to_string();
                let new_id = new_rule.id.clone();
                rules.org_rules.push(new_rule);
                let _ = rules.save();
                tracing::info!(
                    source_rule_id = %id,
                    new_rule_id = %new_id,
                    "Org rule duplicated"
                );
            }
        }
    }
    Task::none()
}
