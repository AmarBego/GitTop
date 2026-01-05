//! Shared application context across authenticated screens.

use crate::github::SessionManager;
use crate::settings::AppSettings;

/// Shared state across all authenticated screens.
///
/// This is passed to screens and provides access to settings and sessions
/// without screens needing to own or mutate this state directly.
#[derive(Clone)]
pub struct AppContext {
    pub settings: AppSettings,
    pub sessions: SessionManager,
}

impl AppContext {
    /// Create a new context.
    pub fn new(settings: AppSettings, sessions: SessionManager) -> Self {
        Self { settings, sessions }
    }

    /// Clone with updated settings.
    pub fn with_settings(&self, settings: AppSettings) -> Self {
        Self {
            settings,
            sessions: self.sessions.clone(),
        }
    }

    /// Get list of account usernames.
    pub fn account_names(&self) -> Vec<String> {
        self.sessions.usernames().map(String::from).collect()
    }
}
