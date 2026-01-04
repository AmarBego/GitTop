use crate::ui::features::account_rules::AccountRuleMessage;
use crate::ui::features::type_rules::TypeRuleMessage;

#[derive(Debug, Clone)]
pub enum RuleEngineMessage {
    Back,
    SelectTab(RuleTab),
    ToggleEnabled(bool),
    Account(AccountRuleMessage),
    Org(OrgMessage),
    Type(TypeRuleMessage),
    Inspector(InspectorMessage),
    Overview(OverviewMessage),
}

// Re-export feature messages for convenience
pub use crate::ui::features::account_rules::AccountRuleMessage as AccountMessage;
pub use crate::ui::features::org_rules::OrgMessage;
pub use crate::ui::features::rule_overview::OverviewMessage;
pub use crate::ui::features::type_rules::TypeRuleMessage as TypeMessage;

#[derive(Debug, Clone)]
pub enum InspectorMessage {
    Select(String),
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuleTab {
    #[default]
    Overview,
    AccountRules,
    OrgRules,
    TypeRules,
}
