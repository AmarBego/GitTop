use crate::settings::AppTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    PowerMode,
    General,
    Accounts,
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
}
