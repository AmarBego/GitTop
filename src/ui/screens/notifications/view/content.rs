//! Main content view - notification list with virtual scrolling.

use iced::widget::{button, column, container, row, scrollable, Space};
use iced::{Alignment, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::widgets::notification_item;
use crate::ui::{icons, theme};

use super::group::view_group_header;
use super::states::{view_empty, view_error, view_loading};

use crate::ui::screens::notifications::messages::NotificationMessage;
use crate::ui::screens::notifications::screen::NotificationsScreen;

impl NotificationsScreen {
    /// Renders the main content area (header or bulk bar + content).
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

        // Check processed notifications (after rule filtering) for empty state
        if self.processed_notifications.is_empty() {
            return view_empty(self.filters.show_all, icon_theme);
        }

        // === VIRTUAL SCROLLING ===
        // Constants for item height calculation
        let item_height: f32 = if power_mode { 48.0 } else { 64.0 };
        let header_height: f32 = 40.0;
        let group_spacing: f32 = 8.0;
        let buffer_items: usize = 5; // Extra items above/below viewport

        // Calculate visible range based on scroll position
        let first_visible_px = self.scroll_offset;
        let last_visible_px = self.scroll_offset + self.viewport_height;

        // Build content with groups, virtualizing items within each group
        let mut content = column![].spacing(8).padding([8, 8]);
        let mut cumulative_y: f32 = 8.0; // Start with top padding

        for (group_idx, group) in self.groups.iter().enumerate() {
            if group.notifications.is_empty() {
                continue;
            }

            // Always render group header (they're small and needed for interaction)
            content = content.push(view_group_header(group, group_idx, icon_theme));
            cumulative_y += header_height;

            if group.is_expanded {
                let group_items_start_y = cumulative_y;
                let total_group_height = group.notifications.len() as f32 * item_height;
                let group_items_end_y = group_items_start_y + total_group_height;

                // Check if this group overlaps with visible viewport
                if group_items_end_y >= first_visible_px && group_items_start_y <= last_visible_px {
                    // Calculate which items are visible within this group
                    let first_visible_in_group = if first_visible_px > group_items_start_y {
                        ((first_visible_px - group_items_start_y) / item_height) as usize
                    } else {
                        0
                    };

                    let last_visible_in_group = if last_visible_px < group_items_end_y {
                        ((last_visible_px - group_items_start_y) / item_height).ceil() as usize
                    } else {
                        group.notifications.len()
                    };

                    // Apply buffer
                    let start_idx = first_visible_in_group.saturating_sub(buffer_items);
                    let end_idx =
                        (last_visible_in_group + buffer_items).min(group.notifications.len());

                    // Add top spacer for items above visible area
                    if start_idx > 0 {
                        let top_space = start_idx as f32 * item_height;
                        content = content.push(Space::new().height(top_space));
                    }

                    // Render only visible items
                    let is_priority = group.is_priority;
                    for p in &group.notifications[start_idx..end_idx] {
                        let item = notification_item(p, icon_theme, power_mode, is_priority);

                        // In bulk mode, wrap with selection indicator
                        if self.bulk_mode && power_mode {
                            let id = p.notification.id.clone();
                            let is_selected = self.selected_ids.contains(&id);
                            let pp = theme::palette();

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

                            let wrapped = button(
                                row![checkbox_icon, Space::new().width(8), item,]
                                    .align_y(Alignment::Center),
                            )
                            .style(move |_theme, _status| button::Style {
                                background: None,
                                ..Default::default()
                            })
                            .padding(0)
                            .on_press(NotificationMessage::ToggleSelect(id));

                            content = content.push(wrapped);
                        } else {
                            content = content.push(item);
                        }
                    }

                    // Add bottom spacer for items below visible area
                    if end_idx < group.notifications.len() {
                        let bottom_space =
                            (group.notifications.len() - end_idx) as f32 * item_height;
                        content = content.push(Space::new().height(bottom_space));
                    }
                } else {
                    // Group is entirely off-screen, just add spacer for total height
                    content = content.push(Space::new().height(total_group_height));
                }

                cumulative_y += total_group_height;
            }

            content = content.push(Space::new().height(group_spacing));
            cumulative_y += group_spacing;
        }

        container(
            scrollable(content)
                .on_scroll(NotificationMessage::OnScroll)
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
