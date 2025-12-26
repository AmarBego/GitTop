//! Main content view - notification list with virtual scrolling.

use iced::widget::{Space, button, column, container, row, scrollable};
use iced::{Alignment, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::widgets::notification_item;
use crate::ui::{icons, theme};

use super::group::view_group_header;
use super::states::{view_empty, view_error, view_loading};

use crate::ui::screens::notifications::messages::{BulkMessage, NotificationMessage, ViewMessage};
use crate::ui::screens::notifications::screen::NotificationsScreen;

impl NotificationsScreen {
    pub fn view_main_content(
        &self,
        icon_theme: IconTheme,
        power_mode: bool,
    ) -> Element<'_, NotificationMessage> {
        if power_mode {
            // In power mode, add bulk action bar above content
            column![
                self.view_bulk_action_bar(icon_theme),
                self.view_content(icon_theme, power_mode)
            ]
            .width(Fill)
            .height(Fill)
            .into()
        } else {
            column![
                self.view_content_header(icon_theme),
                self.view_content(icon_theme, power_mode)
            ]
            .width(Fill)
            .height(Fill)
            .into()
        }
    }

    /// Renders the notification list with virtual scrolling.
    pub fn view_content(
        &self,
        icon_theme: IconTheme,
        power_mode: bool,
    ) -> Element<'_, NotificationMessage> {
        if self.is_loading && self.all_notifications.is_empty() {
            return view_loading();
        }

        if let Some(ref error) = self.error_message {
            return view_error(error, icon_theme);
        }

        // Check if there are any notifications to display
        let has_notifications = self.groups.iter().any(|g| !g.notifications.is_empty());
        if !has_notifications {
            let empty_state = if self.filters.show_all {
                super::states::EmptyState::NoNotifications
            } else {
                super::states::EmptyState::AllCaughtUp
            };
            return view_empty(empty_state, icon_theme);
        }

        let in_bulk_mode = self.bulk_mode && power_mode;
        let pp = theme::palette();

        // === HEIGHT ESTIMATES FOR VIRTUAL SCROLLING ===
        let item_height: f32 = if power_mode { 56.0 } else { 72.0 };
        let header_height: f32 = 32.0;
        let column_spacing: f32 = 8.0;
        let content_padding: f32 = 8.0;
        let buffer_items: usize = 10;

        let first_visible_px = self.scroll_offset.max(0.0);
        let last_visible_px = self.scroll_offset + self.viewport_height + 100.0;

        let mut content = column![]
            .spacing(column_spacing)
            .padding([content_padding, content_padding]);
        let mut current_y: f32 = content_padding;

        for (group_idx, group) in self.groups.iter().enumerate() {
            if group.notifications.is_empty() {
                continue;
            }

            let header_end_y = current_y + header_height;
            let header =
                container(view_group_header(group, group_idx, icon_theme)).height(header_height);
            content = content.push(header);
            current_y = header_end_y + column_spacing;

            if group.is_expanded {
                let items_start_y = current_y;
                let items_count = group.notifications.len();
                let total_items_height =
                    items_count as f32 * (item_height + column_spacing) - column_spacing;
                let items_end_y = items_start_y + total_items_height;

                if items_end_y >= first_visible_px && items_start_y <= last_visible_px {
                    let first_visible_idx = if first_visible_px > items_start_y {
                        ((first_visible_px - items_start_y) / (item_height + column_spacing))
                            as usize
                    } else {
                        0
                    };

                    let last_visible_idx = if last_visible_px < items_end_y {
                        ((last_visible_px - items_start_y) / (item_height + column_spacing)).ceil()
                            as usize
                            + 1
                    } else {
                        items_count
                    };

                    let render_start = first_visible_idx.saturating_sub(buffer_items);
                    let render_end = (last_visible_idx + buffer_items).min(items_count);

                    if render_start > 0 {
                        let top_spacer_height =
                            render_start as f32 * (item_height + column_spacing);
                        content = content.push(Space::new().height(top_spacer_height).width(Fill));
                    }

                    let is_priority = group.is_priority;
                    for p in &group.notifications[render_start..render_end] {
                        let item_element: Element<'_, NotificationMessage> = if in_bulk_mode {
                            // Bulk mode: checkbox + notification item
                            let item =
                                notification_item(p, icon_theme, power_mode, is_priority, false);
                            let id = p.notification.id.clone();
                            let is_selected = self.selected_ids.contains(&id);

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
                                    button::Status::Hovered => Some(iced::Background::Color(
                                        iced::Color::from_rgba(1.0, 1.0, 1.0, 0.03),
                                    )),
                                    button::Status::Pressed => Some(iced::Background::Color(
                                        iced::Color::from_rgba(1.0, 1.0, 1.0, 0.05),
                                    )),
                                    _ => None,
                                };
                                button::Style {
                                    background: bg,
                                    text_color: pp.text_primary,
                                    ..Default::default()
                                }
                            })
                            .padding(0)
                            .on_press(NotificationMessage::Bulk(BulkMessage::ToggleSelect(id)))
                            .width(Fill)
                            .into()
                        } else {
                            // Normal mode: just the notification item
                            notification_item(p, icon_theme, power_mode, is_priority, true)
                        };

                        content = content.push(item_element);
                    }

                    if render_end < items_count {
                        let remaining = items_count - render_end;
                        let bottom_spacer_height =
                            remaining as f32 * (item_height + column_spacing);
                        content =
                            content.push(Space::new().height(bottom_spacer_height).width(Fill));
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
                .on_scroll(|v| NotificationMessage::View(ViewMessage::OnScroll(v)))
                .height(Fill)
                .width(Fill)
                .style(theme::scrollbar),
        )
        .style(theme::app_container)
        .height(Fill)
        .width(Fill)
        .into()
    }

    /// Simple content rendering without virtual scrolling.
    /// Used as fallback if virtual scrolling causes issues.
    #[allow(dead_code)]
    fn view_content_simple(
        &self,
        icon_theme: IconTheme,
        power_mode: bool,
    ) -> Element<'_, NotificationMessage> {
        let column_spacing: f32 = 8.0;
        let content_padding: f32 = 8.0;
        let pp = theme::palette();
        let in_bulk_mode = self.bulk_mode && power_mode;

        let mut content = column![]
            .spacing(column_spacing)
            .padding([content_padding, content_padding]);

        for (group_idx, group) in self.groups.iter().enumerate() {
            if group.notifications.is_empty() {
                continue;
            }

            content = content.push(view_group_header(group, group_idx, icon_theme));

            if group.is_expanded {
                let is_priority = group.is_priority;

                for p in &group.notifications {
                    let item_element: Element<'_, NotificationMessage> = if in_bulk_mode {
                        let item = notification_item(p, icon_theme, power_mode, is_priority, false);
                        let id = p.notification.id.clone();
                        let is_selected = self.selected_ids.contains(&id);

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
                                button::Status::Hovered => Some(iced::Background::Color(
                                    iced::Color::from_rgba(1.0, 1.0, 1.0, 0.03),
                                )),
                                button::Status::Pressed => Some(iced::Background::Color(
                                    iced::Color::from_rgba(1.0, 1.0, 1.0, 0.05),
                                )),
                                _ => None,
                            };
                            button::Style {
                                background: bg,
                                text_color: pp.text_primary,
                                ..Default::default()
                            }
                        })
                        .padding(0)
                        .on_press(NotificationMessage::Bulk(BulkMessage::ToggleSelect(id)))
                        .width(Fill)
                        .into()
                    } else {
                        notification_item(p, icon_theme, power_mode, is_priority, true)
                    };

                    content = content.push(item_element);
                }
            }
        }

        content = content.push(Space::new().height(content_padding));

        container(
            scrollable(content)
                .on_scroll(|v| NotificationMessage::View(ViewMessage::OnScroll(v)))
                .height(Fill)
                .width(Fill)
                .style(theme::scrollbar),
        )
        .style(theme::app_container)
        .height(Fill)
        .width(Fill)
        .into()
    }
}
