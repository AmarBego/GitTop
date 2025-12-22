//! Rule Engine messages.

/// Rule Engine screen messages.
#[derive(Debug, Clone)]
pub enum RuleEngineMessage {
    /// Go back to settings.
    Back,
    /// Select a rule tab.
    SelectTab(RuleTab),
    /// Toggle the rule engine globally.
    ToggleEnabled(bool),

    // ========================================================================
    // Account Rules (New Design)
    // ========================================================================
    /// Select an account in the Accounts tab to view/edit schedule.
    SelectAccount(String), // account rule ID

    // Master switch
    ToggleAccountEnabled(String, bool),

    // Weekly Schedule Grid
    ToggleAccountDay(String, u8), // id, day(0-6)

    // Time Windows
    SetAccountTimeWindow(String, Option<String>, Option<String>), // id, start, end
    SetAccountTimeWindowExpanded(String, bool),                   // id, is_expanded

    // Outside Schedule Behavior
    SetOutsideScheduleBehavior(
        String,
        crate::ui::screens::settings::rule_engine::rules::OutsideScheduleBehavior,
    ),

    // ========================================================================
    // Org Rules
    // ========================================================================
    ToggleOrgRule(String, bool),
    DeleteOrgRule(String),
    DuplicateOrgRule(String),

    // ========================================================================
    // Type Rules
    // ========================================================================
    ToggleTypeRule(String, bool),
    DeleteTypeRule(String),
    DuplicateTypeRule(String),

    // Type Rule Creation
    NewTypeRuleTypeChanged(crate::github::types::NotificationReason),
    NewTypeRuleAccountChanged(String),
    NewTypeRulePriorityChanged(i32),
    NewTypeRuleActionChanged(crate::ui::screens::settings::rule_engine::rules::RuleAction),
    AddTypeRule,

    // Type Rule Grouping
    /// Toggle a type rule group (expand/collapse)
    ToggleTypeGroup(String),

    // Rule Inspector
    /// Select a rule for inspection
    SelectRule(String),
    /// Clear rule selection (close inspector)
    ClearRuleSelection,

    // Explain Decision
    /// Set the test notification type for explain decision
    SetExplainTestType(String),
}

/// Rule Engine tabs for navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RuleTab {
    #[default]
    Overview,
    AccountRules,
    OrgRules,
    TypeRules,
}
