//! Bulk action bar view for Power Mode.

use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::{icons, theme};

use crate::ui::screens::notifications::messages::NotificationMessage;
use crate::ui::screens::notifications::screen::NotificationsScreen;

impl NotificationsScreen {
    /// Renders the bulk action bar for Power Mode.
    /// Shows "Select" button when not in bulk mode, or full action bar when in bulk mode.
    pub fn view_bulk_action_bar(&self, icon_theme: IconTheme) -> Element<'_, NotificationMessage> {
        let p = theme::palette();
        let selection_count = self.selected_ids.len();

        // Only show action bar when in bulk mode (triggered from top bar Select button)
        if !self.bulk_mode {
            return Space::new().height(0).into();
        }

        // In bulk mode: show action bar
        let selection_text = if selection_count == 0 {
            "Select items".to_string()
        } else {
            format!("{} selected", selection_count)
        };

        let select_all_btn = button(text("Select All").size(12).color(p.text_secondary))
            .style(theme::ghost_button)
            .padding([6, 10])
            .on_press(NotificationMessage::SelectAll);

        let clear_btn = button(text("Clear").size(12).color(p.text_secondary))
            .style(theme::ghost_button)
            .padding([6, 10])
            .on_press_maybe(if selection_count > 0 {
                Some(NotificationMessage::ClearSelection)
            } else {
                None
            });

        let cancel_btn = button(
            row![
                icons::icon_x(12.0, p.text_secondary, icon_theme),
                Space::new().width(4),
                text("Cancel").size(12).color(p.text_secondary),
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::ghost_button)
        .padding([6, 10])
        .on_press(NotificationMessage::ToggleBulkMode);

        // Action buttons (only enabled when items selected)
        let mark_read_btn = button(
            row![
                icons::icon_check(12.0, iced::Color::WHITE, icon_theme),
                Space::new().width(6),
                text("Mark Read").size(12).color(iced::Color::WHITE),
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::primary_button)
        .padding([6, 12])
        .on_press_maybe(if selection_count > 0 {
            Some(NotificationMessage::BulkMarkAsRead)
        } else {
            None
        });

        let archive_btn = button(
            row![
                icons::icon_inbox(12.0, p.accent_warning, icon_theme),
                Space::new().width(6),
                text("Mark as Done").size(12).color(p.text_primary),
            ]
            .align_y(Alignment::Center),
        )
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered => p.bg_hover,
                button::Status::Pressed => p.bg_active,
                _ => p.bg_control,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: p.text_primary,
                border: iced::Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: p.border_subtle,
                },
                ..Default::default()
            }
        })
        .padding([6, 12])
        .on_press_maybe(if selection_count > 0 {
            Some(NotificationMessage::BulkMarkAsDone)
        } else {
            None
        });

        container(
            row![
                text(selection_text).size(13).color(p.text_primary),
                Space::new().width(16),
                select_all_btn,
                clear_btn,
                Space::new().width(Fill),
                mark_read_btn,
                Space::new().width(8),
                archive_btn,
                Space::new().width(16),
                cancel_btn,
            ]
            .align_y(Alignment::Center)
            .padding([8, 16]),
        )
        .width(Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.accent.scale_alpha(0.1))),
            border: iced::Border {
                color: p.accent.scale_alpha(0.3),
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}
