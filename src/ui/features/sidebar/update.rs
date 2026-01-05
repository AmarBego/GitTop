use iced::Task;

use super::messages::SidebarMessage;
use super::state::SidebarState;

#[derive(Debug, Clone, PartialEq)]
pub enum SidebarAction {
    FilterChanged,
    SwitchAccount(String),
    OpenSettings,
    Logout,
}

pub fn update(state: &mut SidebarState, message: SidebarMessage) -> Task<SidebarAction> {
    match message {
        SidebarMessage::SelectType(t) => {
            state.selected_type = t;
            Task::done(SidebarAction::FilterChanged)
        }
        SidebarMessage::SelectRepo(r) => {
            state.selected_repo = r;
            Task::done(SidebarAction::FilterChanged)
        }
        SidebarMessage::SwitchAccount(u) => Task::done(SidebarAction::SwitchAccount(u)),
        SidebarMessage::OpenSettings => Task::done(SidebarAction::OpenSettings),
        SidebarMessage::Logout => Task::done(SidebarAction::Logout),
    }
}
