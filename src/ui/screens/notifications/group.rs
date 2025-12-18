//! Notification group component - collapsible time-based groups.

use iced::widget::{button, keyed_column, row, text, Space};
use iced::{Alignment, Element, Fill};

use super::helper::NotificationGroup;
use super::screen::NotificationMessage;
use crate::ui::widgets::notification_item;
use crate::ui::{icons, theme};

/// Render a collapsible notification group header.
pub fn view_group_header<'a>(
    group: &'a NotificationGroup,
    group_index: usize,
) -> Element<'a, NotificationMessage> {
    let chevron = if group.is_expanded {
        icons::icon_colored(&icondata_lu::LuChevronDown, 12.0, theme::TEXT_MUTED)
    } else {
        icons::icon_colored(&icondata_lu::LuChevronRight, 12.0, theme::TEXT_MUTED)
    };

    button(
        row![
            chevron,
            Space::new().width(8),
            text(&group.title).size(12).color(theme::TEXT_SECONDARY),
            Space::new().width(4),
            text(format!("({})", group.notifications.len()))
                .size(11)
                .color(theme::TEXT_MUTED),
        ]
        .align_y(Alignment::Center),
    )
    .style(theme::ghost_button)
    .padding(4)
    .on_press(NotificationMessage::ToggleGroup(group_index))
    .width(Fill)
    .into()
}

/// Render the notification items within an expanded group.
pub fn view_group_items<'a>(group: &'a NotificationGroup) -> Element<'a, NotificationMessage> {
    let items = group
        .notifications
        .iter()
        .enumerate()
        .map(|(idx, n)| (idx, notification_item(n)));

    keyed_column(items).spacing(2).into()
}
