//! Notification group component - collapsible time-based groups.

use iced::widget::{button, container, keyed_column, row, text, Space};
use iced::{Alignment, Color, Element, Fill};

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
        .on_press(NotificationMessage::ToggleGroup(group_index))
        .width(Fill);

    // Wrap priority headers with subtle background
    if group.is_priority {
        container(header_btn)
            .style(move |_| {
                let p = theme::palette();
                container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        p.accent_warning.r,
                        p.accent_warning.g,
                        p.accent_warning.b,
                        0.05,
                    ))),
                    border: iced::Border {
                        radius: 6.0.into(),
                        color: Color::from_rgba(
                            p.accent_warning.r,
                            p.accent_warning.g,
                            p.accent_warning.b,
                            0.15,
                        ),
                        width: 1.0,
                    },
                    ..Default::default()
                }
            })
            .into()
    } else {
        header_btn.into()
    }
}

/// Render the notification items within an expanded group.
pub fn view_group_items<'a>(
    group: &'a NotificationGroup,
    icon_theme: IconTheme,
    dense: bool,
) -> Element<'a, NotificationMessage> {
    let is_priority = group.is_priority;
    let items = group
        .notifications
        .iter()
        .enumerate()
        .map(|(idx, p)| (idx, notification_item(p, icon_theme, dense, is_priority)));

    keyed_column(items)
        .spacing(if dense { 0 } else { 4 }) // No spacing in dense mode for list feel
        .into()
}
