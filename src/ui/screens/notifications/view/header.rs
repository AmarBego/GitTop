//! Content header view - title, sync status, filters, actions.

use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Color, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::{icons, theme};

use crate::ui::screens::notifications::messages::NotificationMessage;
use crate::ui::screens::notifications::screen::NotificationsScreen;

impl NotificationsScreen {
    /// Renders the content header with title, sync status, filter toggle, and actions.
    pub fn view_content_header(&self, icon_theme: IconTheme) -> Element<'_, NotificationMessage> {
        let p = theme::palette();
        let unread_count = self
            .filtered_notifications
            .iter()
            .filter(|n| n.unread)
            .count();

        let title = text("Notifications").size(18).color(p.text_primary);

        let sync_status: Element<'_, NotificationMessage> = if self.is_loading {
            row![
                icons::icon_refresh(11.0, p.text_muted, icon_theme),
                Space::new().width(4),
                text("Syncing...").size(11).color(p.text_muted),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                icons::icon_check(11.0, p.accent_success, icon_theme),
                Space::new().width(4),
                text("Synced").size(11).color(p.accent_success),
            ]
            .align_y(Alignment::Center)
            .into()
        };

        // Segmented control for filter selection (Unread | All)
        let is_unread_filter = !self.filters.show_all;

        let unread_btn = button(text("Unread").size(12).color(if is_unread_filter {
            p.text_primary
        } else {
            p.text_secondary
        }))
        .style(move |_theme, status| {
            let base_bg = if is_unread_filter {
                p.bg_active
            } else {
                Color::TRANSPARENT
            };
            let bg = match status {
                button::Status::Hovered if !is_unread_filter => p.bg_hover,
                button::Status::Pressed => p.bg_active,
                _ => base_bg,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: if is_unread_filter {
                    p.text_primary
                } else {
                    p.text_secondary
                },
                border: iced::Border {
                    radius: 0.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .padding([6, 12])
        .on_press(NotificationMessage::ToggleShowAll);

        let all_btn = button(text("All").size(12).color(if !is_unread_filter {
            p.text_primary
        } else {
            p.text_secondary
        }))
        .style(move |_theme, status| {
            let base_bg = if !is_unread_filter {
                p.bg_active
            } else {
                Color::TRANSPARENT
            };
            let bg = match status {
                button::Status::Hovered if is_unread_filter => p.bg_hover,
                button::Status::Pressed => p.bg_active,
                _ => base_bg,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: if !is_unread_filter {
                    p.text_primary
                } else {
                    p.text_secondary
                },
                border: iced::Border {
                    radius: 0.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .padding([6, 12])
        .on_press(NotificationMessage::ToggleShowAll);

        // Wrap in container with border
        let filter_segment =
            container(row![unread_btn, all_btn].spacing(0)).style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_control)),
                border: iced::Border {
                    radius: 4.0.into(),
                    color: p.border_subtle,
                    width: 1.0,
                },
                ..Default::default()
            });

        // Mark all read button with improved styling
        let mark_all_btn = if unread_count > 0 {
            button(
                row![
                    icons::icon_check(12.0, p.accent, icon_theme),
                    Space::new().width(6),
                    text("Mark all read").size(12).color(p.text_primary),
                ]
                .align_y(Alignment::Center),
            )
            .style(move |_theme, status| {
                let bg = match status {
                    button::Status::Hovered => p.bg_hover,
                    button::Status::Pressed => p.bg_active,
                    _ => Color::TRANSPARENT,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: p.text_primary,
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .padding([6, 10])
            .on_press(NotificationMessage::MarkAllAsRead)
        } else {
            button(
                row![
                    icons::icon_check(12.0, p.text_muted, icon_theme),
                    Space::new().width(6),
                    text("Mark all read").size(12).color(p.text_muted),
                ]
                .align_y(Alignment::Center),
            )
            .style(move |_theme, _status| button::Style {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                text_color: p.text_muted,
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .padding([6, 10])
        };

        // Refresh button with subtle styling
        let refresh_btn = button(icons::icon_refresh(14.0, p.text_secondary, icon_theme))
            .style(move |_theme, status| {
                let bg = match status {
                    button::Status::Hovered => p.bg_hover,
                    button::Status::Pressed => p.bg_active,
                    _ => Color::TRANSPARENT,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: p.text_secondary,
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .padding(8)
            .on_press(NotificationMessage::Refresh);

        let header_row = row![
            title,
            Space::new().width(12),
            sync_status,
            Space::new().width(Fill),
            filter_segment,
            Space::new().width(12),
            mark_all_btn,
            Space::new().width(4),
            refresh_btn,
        ]
        .align_y(Alignment::Center)
        .padding([14, 16]);

        // Header with subtle bottom border for visual separation
        container(header_row)
            .width(Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_card)),
                border: iced::Border {
                    color: p.border_subtle,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}
