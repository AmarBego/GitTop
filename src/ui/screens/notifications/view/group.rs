//! Notification group component - collapsible time-based groups.

use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::screens::notifications::helper::NotificationGroup;
use crate::ui::screens::notifications::messages::{NotificationMessage, ViewMessage};
use crate::ui::{icons, theme};

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

    // Priority groups get special styling
    let (title_color, count_color) = if group.is_priority {
        (p.accent_warning, p.accent_warning)
    } else {
        (p.text_secondary, p.text_muted)
    };

    let header_content = row![
        chevron,
        Space::new().width(8),
        text(&group.title).size(13).color(title_color),
        Space::new().width(6),
        text(format!("({})", group.notifications.len()))
            .size(12)
            .color(count_color),
    ]
    .align_y(Alignment::Center);

    let header_btn = button(header_content)
        .style(if group.is_priority {
            theme::priority_header_button
        } else {
            theme::ghost_button
        })
        .padding([6, 8])
        .on_press(NotificationMessage::View(ViewMessage::ToggleGroup(
            group_index,
        )))
        .width(Fill);

    // Wrap priority headers with subtle background from theme
    if group.is_priority {
        container(header_btn)
            .style(theme::priority_header_container)
            .into()
    } else {
        header_btn.into()
    }
}
