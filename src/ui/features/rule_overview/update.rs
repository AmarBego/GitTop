use super::message::OverviewMessage;
use super::state::RuleOverviewState;
use iced::Task;

pub fn update(state: &mut RuleOverviewState, message: OverviewMessage) -> Task<OverviewMessage> {
    match message {
        OverviewMessage::SetTestType(test_type) => {
            state.explain_test_type = test_type;
        }
    }
    Task::none()
}
