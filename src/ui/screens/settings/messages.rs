use crate::ui::features::account_management::AccountMessage;
use crate::ui::features::general_settings::GeneralMessage;
use crate::ui::features::network_proxy::ProxyMessage;
use crate::ui::features::power_mode::PowerModeMessage;

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
    OpenRuleEngine,
    Account(AccountMessage),
    General(GeneralMessage),
    Proxy(ProxyMessage),
    PowerMode(PowerModeMessage),
}
