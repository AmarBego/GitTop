#[derive(Debug, Clone)]
pub struct GeneralSettingsState {
    pub start_on_boot_enabled: bool,
}

impl GeneralSettingsState {
    pub fn new() -> Self {
        // Cache start-on-boot state to avoid querying systemctl on every render
        let start_on_boot_enabled = crate::platform::on_boot::is_enabled();
        Self {
            start_on_boot_enabled,
        }
    }
}
