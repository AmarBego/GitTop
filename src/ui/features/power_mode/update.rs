use super::message::PowerModeMessage;
use super::state::PowerModeState;
use crate::settings::AppSettings;
use iced::Task;

pub fn update(
    _state: &mut PowerModeState,
    message: PowerModeMessage,
    settings: &mut AppSettings,
) -> Task<PowerModeMessage> {
    match message {
        PowerModeMessage::Toggle(enabled) => {
            settings.power_mode = enabled;
            let _ = settings.save();
            Task::none()
        }
        PowerModeMessage::OpenRuleEngine => Task::none(),
    }
}
