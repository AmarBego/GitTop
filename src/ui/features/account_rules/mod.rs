//! Account Rules feature module for Rule Engine.
//!
//! Handles account-based rule selection and time window configuration.

mod message;
mod state;
mod update;
mod view;

pub use message::AccountRuleMessage;
pub use state::AccountRulesState;
pub use update::update_account_rule;
pub use view::view_account_rules_tab;
