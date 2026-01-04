use crate::ui::features::account_rules::AccountRuleMessage;
use crate::ui::features::type_rules::TypeRuleMessage;

#[derive(Debug, Clone)]
pub enum RuleEngineMessage {
    Back,
    SelectTab(RuleTab),
    ToggleEnabled(bool),
    ToggleHandbook,
    NoOp,
    Account(AccountRuleMessage),
    Org(OrgMessage),
    Type(TypeRuleMessage),
    Inspector(InspectorMessage),
    Explain(ExplainMessage),
}

// Re-export feature messages for convenience
pub use crate::ui::features::account_rules::AccountRuleMessage as AccountMessage;
pub use crate::ui::features::type_rules::TypeRuleMessage as TypeMessage;

#[derive(Debug, Clone)]
pub enum OrgMessage {
    Toggle(String, bool),
    Delete(String),
    Duplicate(String),
}

#[derive(Debug, Clone)]
pub enum InspectorMessage {
    Select(String),
    Close,
}

#[derive(Debug, Clone)]
pub enum ExplainMessage {
    SetTestType(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuleTab {
    #[default]
    Overview,
    AccountRules,
    OrgRules,
    TypeRules,
}
