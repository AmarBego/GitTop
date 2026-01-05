//! Screen routing and state management.

use crate::ui::screens::notifications::NotificationsScreen;
use crate::ui::screens::settings::SettingsScreen;
use crate::ui::screens::settings::rule_engine::RuleEngineScreen;

/// Current screen state in the authenticated app.
pub enum Screen {
    /// Main notifications screen.
    Notifications(Box<NotificationsScreen>),
    /// Settings screen.
    Settings(Box<SettingsScreen>),
    /// Rule Engine screen with back navigation context.
    RuleEngine(Box<RuleEngineScreen>, RuleEngineOrigin),
}

impl Screen {
    /// Get the window title for this screen.
    pub fn title(&self) -> String {
        match self {
            Screen::Notifications(screen) => {
                let unread = screen
                    .processing
                    .all_notifications
                    .iter()
                    .filter(|n| n.unread)
                    .count();
                if unread > 0 {
                    format!("GitTop ({unread} unread)")
                } else {
                    "GitTop".into()
                }
            }
            Screen::Settings(_) => "GitTop - Settings".into(),
            Screen::RuleEngine(_, _) => "GitTop - Rule Engine".into(),
        }
    }
}

/// Where the Rule Engine was opened from (for back navigation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleEngineOrigin {
    Settings,
    Notifications,
}

impl RuleEngineOrigin {
    /// Create from the `from_settings` flag in NavigateTo::RuleEngine.
    pub fn from_settings_flag(from_settings: bool) -> Self {
        if from_settings {
            Self::Settings
        } else {
            Self::Notifications
        }
    }
}
