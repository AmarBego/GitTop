//! Application effects - state changes that screens can request.
//!
//! This module implements the Effect Pattern: screens return effects describing
//! what should happen, and App.rs applies them. This decouples screens from
//! app-level state management.

/// Effects that screens can request from the App layer.
///
/// Instead of screens mutating app state directly or emitting messages that
/// app.rs must intercept, screens return effects describing the desired change.
#[derive(Debug, Clone, Default)]
pub enum AppEffect {
    #[default]
    None,
    Navigate(NavigateTo),
    Session(SessionEffect),
}

/// Navigation targets.
#[derive(Debug, Clone)]
pub enum NavigateTo {
    Notifications,
    Settings,
    RuleEngine { from_settings: bool },
    Login,
    Back,
}

/// Session/account operations.
#[derive(Debug, Clone)]
pub enum SessionEffect {
    Logout,
    SwitchAccount(String),
    RemoveAccount(String),
}
