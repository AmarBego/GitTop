use iced::widget::{Space, button, column, container, row, scrollable};
use iced::{Alignment, Element, Fill};

use super::widgets::notification_item;
use crate::settings::IconTheme;
use crate::ui::features::bulk_actions::{BulkActionMessage, BulkActionState};
use crate::ui::features::sidebar::SidebarState;
use crate::ui::screens::notifications::components::group::view_group_header;
use crate::ui::screens::notifications::components::states::{
    EmptyState, view_empty, view_error, view_loading,
};
use crate::ui::screens::notifications::helper::{NotificationGroup, ProcessedNotification};
use crate::ui::screens::notifications::messages::NotificationMessage;
use crate::ui::{icons, theme};

use super::{NotificationListMessage, NotificationListState};

pub struct ListArgs<'a> {
    pub groups: &'a [NotificationGroup],
    pub is_loading: bool,
    pub has_notifications: bool, // or check groups.is_empty?
    pub error_message: Option<&'a String>,
    pub filters: &'a SidebarState,
    pub bulk_actions: &'a BulkActionState,

    pub list_state: &'a NotificationListState,
    pub icon_theme: IconTheme,
    pub power_mode: bool,
}

pub fn view<'a>(args: ListArgs<'a>) -> Element<'a, NotificationMessage> {
    if args.is_loading && !args.has_notifications {
        return view_loading();
    }

    if let Some(error) = args.error_message {
        return view_error(error, args.icon_theme);
    }

    // Check if there are any notifications to display
    let has_content = args.groups.iter().any(|g| !g.notifications.is_empty());
    if !has_content {
        let empty_state = if args.filters.show_all {
            EmptyState::NoNotifications
        } else {
            EmptyState::AllCaughtUp
        };
        return view_empty(empty_state, args.icon_theme);
    }

    let in_bulk_mode = args.bulk_actions.bulk_mode && args.power_mode;
    let pp = theme::palette();

    // === HEIGHT ESTIMATES FOR VIRTUAL SCROLLING ===
    let item_height: f32 = if args.power_mode { 56.0 } else { 72.0 };
    let header_height: f32 = 32.0;
    let column_spacing: f32 = 8.0;
    let content_padding: f32 = 8.0;
    let buffer_items: usize = 10;

    let first_visible_px = args.list_state.scroll_offset.max(0.0);
    let last_visible_px = args.list_state.scroll_offset + args.list_state.viewport_height + 100.0;

    let mut content = column![]
        .spacing(column_spacing)
        .padding([content_padding, content_padding]);
    let mut current_y: f32 = content_padding;

    for (group_idx, group) in args.groups.iter().enumerate() {
        if group.notifications.is_empty() {
            continue;
        }

        let header_end_y = current_y + header_height;
        let header =
            container(view_group_header(group, group_idx, args.icon_theme)).height(header_height);
        content = content.push(header);
        current_y = header_end_y + column_spacing;

        if group.is_expanded {
            let items_start_y = current_y;
            let items_count = group.notifications.len();
            let total_items_height =
                items_count as f32 * (item_height + column_spacing) - column_spacing;
            let items_end_y = items_start_y + total_items_height;

            if items_end_y >= first_visible_px && items_start_y <= last_visible_px {
                let (render_start, render_end) = args.list_state.calculate_visible_range(
                    item_height,
                    column_spacing,
                    buffer_items,
                    items_start_y,
                    items_count,
                );

                if render_start > 0 {
                    let top_spacer_height = render_start as f32 * (item_height + column_spacing);
                    content = content.push(Space::new().height(top_spacer_height).width(Fill));
                }

                let is_priority = group.is_priority;
                for p in &group.notifications[render_start..render_end] {
                    let item_element = item_view(
                        p,
                        in_bulk_mode,
                        args.bulk_actions,
                        args.icon_theme,
                        args.power_mode,
                        is_priority,
                        pp,
                    );
                    content = content.push(item_element);
                }

                if render_end < items_count {
                    let remaining = items_count - render_end;
                    let bottom_spacer_height = remaining as f32 * (item_height + column_spacing);
                    content = content.push(Space::new().height(bottom_spacer_height).width(Fill));
                }
            } else {
                content = content.push(Space::new().height(total_items_height).width(Fill));
            }

            current_y = items_end_y + column_spacing;
        }
    }

    content = content.push(Space::new().height(content_padding));

    container(
        scrollable(content)
            .on_scroll(|v| NotificationMessage::List(NotificationListMessage::OnScroll(v)))
            .height(Fill)
            .width(Fill)
            .style(theme::scrollbar),
    )
    .style(theme::app_container)
    .height(Fill)
    .width(Fill)
    .into()
}

fn item_view<'a>(
    p: &'a ProcessedNotification,
    in_bulk_mode: bool,
    bulk_actions: &'a BulkActionState,
    icon_theme: IconTheme,
    power_mode: bool,
    is_priority: bool,
    pp: theme::ThemePalette,
) -> Element<'a, NotificationMessage> {
    if in_bulk_mode {
        // Bulk mode: checkbox + notification item
        let item = notification_item(p, icon_theme, power_mode, is_priority, false);
        let id = p.notification.id.clone();
        let is_selected = bulk_actions.is_selected(&id);

        let checkbox_icon: Element<'_, NotificationMessage> = if is_selected {
            container(icons::icon_check(12.0, iced::Color::WHITE, icon_theme))
                .padding(2)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(pp.accent)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .into()
        } else {
            container(Space::new().width(16).height(16))
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(pp.bg_control)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: pp.border,
                    },
                    ..Default::default()
                })
                .into()
        };

        button(
            row![checkbox_icon, Space::new().width(8), item]
                .align_y(Alignment::Center)
                .width(Fill),
        )
        .style(move |_theme, status| {
            let bg = match status {
                button::Status::Hovered => Some(iced::Background::Color(iced::Color::from_rgba(
                    1.0, 1.0, 1.0, 0.03,
                ))),
                button::Status::Pressed => Some(iced::Background::Color(iced::Color::from_rgba(
                    1.0, 1.0, 1.0, 0.05,
                ))),
                _ => None,
            };
            button::Style {
                background: bg,
                text_color: pp.text_primary,
                ..Default::default()
            }
        })
        .padding(0)
        .on_press(NotificationMessage::Bulk(BulkActionMessage::ToggleSelect(
            id,
        )))
        .width(Fill)
        .into()
    } else {
        // Normal mode: just the notification item
        notification_item(p, icon_theme, power_mode, is_priority, true)
    }
}
