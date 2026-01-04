//! Type Rules feature module for Rule Engine.
//!
//! Handles type-based notification rule creation and management.

mod message;
mod state;
mod update;
mod view;

pub use message::TypeRuleMessage;
pub use state::TypeRuleFormState;
pub use update::update_type_rule;
pub use view::view_type_rules_tab;
