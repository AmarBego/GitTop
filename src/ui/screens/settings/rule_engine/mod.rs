//! Rule Engine screen module.
//!
//! Modular structure for the Rule Engine configuration interface.
//!
//! Components:
//! - `screen.rs` - Main screen state and layout
//! - `tabs/` - Tab-specific views (overview, time, schedule, etc.)
//! - `components.rs` - Shared UI components (rule cards, empty states)
//! - `messages.rs` - All RuleEngineMessage variants
//! - `rules.rs` - Core rule types and evaluation engine

mod components;
mod explain_decision;
mod inspector;
mod messages;
mod screen;
mod tabs;

pub mod rules;

pub use messages::RuleEngineMessage;
pub use rules::{NotificationRuleSet, RuleAction, RuleEngine};
pub use screen::RuleEngineScreen;
