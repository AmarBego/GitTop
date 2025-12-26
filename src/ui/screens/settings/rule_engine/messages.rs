use crate::github::types::NotificationReason;
use crate::ui::screens::settings::rule_engine::rules::{OutsideScheduleBehavior, RuleAction};
use chrono::Weekday;

#[derive(Debug, Clone)]
pub enum RuleEngineMessage {
    Back,
    SelectTab(RuleTab),
    ToggleEnabled(bool),
    ToggleHandbook,
    NoOp,
    Account(AccountMessage),
    Org(OrgMessage),
    Type(TypeMessage),
    Inspector(InspectorMessage),
    Explain(ExplainMessage),
}

#[derive(Debug, Clone)]
pub enum AccountMessage {
    Select(String),
    ToggleEnabled(String, bool),
    ToggleDay(String, Weekday),
    SetTimeWindow(String, Option<String>, Option<String>),
    SetTimeWindowExpanded(String, bool),
    SetOutsideBehavior(String, OutsideScheduleBehavior),
}

#[derive(Debug, Clone)]
pub enum OrgMessage {
    Toggle(String, bool),
    Delete(String),
    Duplicate(String),
}

/// Type rule messages.
#[derive(Debug, Clone)]
pub enum TypeMessage {
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
