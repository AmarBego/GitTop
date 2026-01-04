//! Account rule messages.

use chrono::Weekday;

use crate::ui::screens::settings::rule_engine::rules::OutsideScheduleBehavior;

/// Messages for account rule operations.
#[derive(Debug, Clone)]
pub enum AccountRuleMessage {
    Select(String),
    ToggleEnabled(String, bool),
    ToggleDay(String, Weekday),
    SetTimeWindow(String, Option<String>, Option<String>),
    SetTimeWindowExpanded(String, bool),
    SetOutsideBehavior(String, OutsideScheduleBehavior),
}
