use iced::Task;

use super::{NotificationListMessage, NotificationListState};
use crate::ui::screens::notifications::helper::NotificationGroup;
use crate::ui::screens::notifications::messages::NotificationMessage;

pub fn update(
    state: &mut NotificationListState,
    message: NotificationListMessage,
    groups: &mut [NotificationGroup],
) -> Task<NotificationMessage> {
    match message {
        NotificationListMessage::ToggleGroup(index) => {
            if let Some(group) = groups.get_mut(index) {
                group.is_expanded = !group.is_expanded;
            }
            Task::none()
        }
        NotificationListMessage::OnScroll(viewport) => {
            state.update_viewport(&viewport);
            Task::none()
        }
    }
}
