use crate::settings::AppTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    PowerMode,
    General,
    Accounts,
    NetworkProxy,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    Back,
    SelectTab(SettingsTab),
    ChangeTheme(AppTheme),
    ToggleIconTheme(bool),
    ToggleMinimizeToTray(bool),
    SetNotificationFontScale(f32),
    SetSidebarFontScale(f32),
    SetSidebarWidth(f32),
    RemoveAccount(String),
    TogglePowerMode(bool),
    OpenRuleEngine,
    TokenInputChanged(String),
    SubmitToken,
    TokenValidated(Result<String, String>),
    ToggleProxyEnabled(bool),
    ProxyUrlChanged(String),
    ProxyUsernameChanged(String),
    ProxyPasswordChanged(String),
    SaveProxySettings,
    ToggleStartOnBoot(bool),
    /// Result of an async start-on-boot enable/disable operation.
    /// Contains `Ok(new_state)` on success or `Err(error_message)` on failure.
    StartOnBootResult(Result<bool, String>),
}
