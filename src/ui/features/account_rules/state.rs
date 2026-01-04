//! Account rules state.

use std::collections::HashSet;

/// State for account rules UI.
#[derive(Debug, Clone, Default)]
pub struct AccountRulesState {
    pub selected_account_id: Option<String>,
    pub expanded_time_windows: HashSet<String>,
}

impl AccountRulesState {
    pub fn new() -> Self {
        Self::default()
    }
}
