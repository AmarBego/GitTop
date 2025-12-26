//! Content header view - title, sync status, filters, actions.

use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Color, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::{icons, theme};

use crate::ui::screens::notifications::messages::{
    FilterMessage, NotificationMessage, ThreadMessage,
};
use crate::ui::screens::notifications::screen::NotificationsScreen;

impl NotificationsScreen {
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

        let unread_btn = view_filter_pill("Unread", is_unread_filter, FilterMessage::ToggleShowAll);
        let all_btn = view_filter_pill("All", !is_unread_filter, FilterMessage::ToggleShowAll);

        let filter_segment =
            container(row![unread_btn, all_btn].spacing(0)).style(theme::segment_container);

        let has_unread = unread_count > 0;
        let mark_all_btn = button(
            row![
                icons::icon_check(
                    12.0,
                    if has_unread { p.accent } else { p.text_muted },
                    icon_theme
                ),
                Space::new().width(6),
                text("Mark all read").size(12).color(if has_unread {
                    p.text_primary
                } else {
                    p.text_muted
                }),
            ]
            .align_y(Alignment::Center),
        )
        .style(move |t, s| {
            if has_unread {
                theme::ghost_button(t, s)
            } else {
                button::Style {
                    background: Some(iced::Background::Color(Color::TRANSPARENT)),
                    text_color: p.text_muted,
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }
        })
        .padding([6, 10])
        .on_press_maybe(
            has_unread.then_some(NotificationMessage::Thread(ThreadMessage::MarkAllAsRead)),
        );

        let refresh_btn = button(icons::icon_refresh(14.0, p.text_secondary, icon_theme))
            .style(theme::ghost_button)
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

        container(header_row)
            .width(Fill)
            .style(theme::header)
            .into()
    }
}

fn view_filter_pill<'a>(
    label: &'a str,
    is_active: bool,
    msg: FilterMessage,
) -> Element<'a, NotificationMessage> {
    button(text(label).size(12))
        .style(theme::segment_button(is_active))
        .padding([6, 12])
        .on_press(NotificationMessage::Filter(msg))
        .into()
}
