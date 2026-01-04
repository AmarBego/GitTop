#[derive(Debug, Clone)]
pub enum GeneralMessage {
    ChangeTheme(crate::settings::AppTheme),
    ToggleIconTheme(bool),
    ToggleMinimizeToTray(bool),
    SetNotificationFontScale(f32),
    SetSidebarFontScale(f32),
    SetSidebarWidth(f32),
    ToggleStartOnBoot(bool),
    StartOnBootResult(Result<bool, String>),
}
