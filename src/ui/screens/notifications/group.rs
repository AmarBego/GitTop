//! Notification group component - collapsible time-based groups.

use iced::widget::{button, keyed_column, row, text, Space};
use iced::{Alignment, Element, Fill};

use super::helper::NotificationGroup;
use super::screen::NotificationMessage;
use crate::settings::IconTheme;
use crate::ui::widgets::notification_item;
use crate::ui::{icons, theme};

/// Render a collapsible notification group header.
pub fn view_group_header<'a>(
    group: &'a NotificationGroup,
    group_index: usize,
    icon_theme: IconTheme,
) -> Element<'a, NotificationMessage> {
    let p = theme::palette();

    let chevron = if group.is_expanded {
        icons::icon_chevron_down(12.0, p.text_muted, icon_theme)
    } else {
        icons::icon_chevron_right(12.0, p.text_muted, icon_theme)
    };

    button(
        row![
            chevron,
            Space::new().width(8),
            text(&group.title).size(13).color(p.text_secondary),
            Space::new().width(6),
            text(format!("({})", group.notifications.len()))
                .size(12)
                .color(p.text_muted),
        ]
        .align_y(Alignment::Center),
    )
    .style(theme::ghost_button)
    .padding([6, 8])
    .on_press(NotificationMessage::ToggleGroup(group_index))
    .width(Fill)
    .into()
}

/// Render the notification items within an expanded group.
pub fn view_group_items<'a>(
    group: &'a NotificationGroup,
    icon_theme: IconTheme,
) -> Element<'a, NotificationMessage> {
    let items = group
        .notifications
        .iter()
        .enumerate()
        .map(|(idx, n)| (idx, notification_item(n, icon_theme)));

    keyed_column(items).spacing(4).into()
}
