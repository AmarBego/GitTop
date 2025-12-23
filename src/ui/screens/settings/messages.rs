//! Settings screen messages and tab definitions.

use crate::settings::AppTheme;

/// Settings tabs for navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    PowerMode,
    General,
    Accounts,
}

/// Settings screen messages.
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    /// Go back to notifications.
    Back,
    /// Select a settings tab.
    SelectTab(SettingsTab),
    /// Change app theme.
    ChangeTheme(AppTheme),
    /// Toggle icon theme.
    ToggleIconTheme(bool),
    /// Toggle minimize to tray.
    ToggleMinimizeToTray(bool),
    /// Set notification text size (0.8 - 1.5).
    SetNotificationFontScale(f32),
    /// Set sidebar text size (0.8 - 1.5).
    SetSidebarFontScale(f32),
    /// Set sidebar width (180 - 400).
    SetSidebarWidth(f32),
    /// Remove an account.
    RemoveAccount(String),
    /// Toggle Power Mode (Enterprise Layout).
    TogglePowerMode(bool),
    /// Open the Rule Engine configuration window.
    OpenRuleEngine,
    /// Token input changed.
    TokenInputChanged(String),
    /// Submit token for validation.
    SubmitToken,
    /// Token validation result.
    TokenValidated(Result<String, String>),
}
