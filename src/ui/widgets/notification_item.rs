//! Notification item widget - displays a single notification.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::screens::notifications::helper::ProcessedNotification;
use crate::ui::screens::notifications::NotificationMessage;
use crate::ui::screens::settings::rule_engine::RuleAction;
use crate::github::types::SubjectType;
use crate::ui::{icons, theme};

/// Get color for subject type
fn get_subject_color(subject_type: SubjectType) -> Color {
    let p = theme::palette();
    match subject_type {
        SubjectType::Issue => p.accent_success,
        SubjectType::PullRequest => p.accent,
        SubjectType::Release => p.accent_purple,
        SubjectType::Discussion => p.accent,
        SubjectType::CheckSuite => p.accent_warning,
        SubjectType::RepositoryVulnerabilityAlert => p.accent_danger,
        _ => p.text_secondary,
    }
}

/// Get the icon for a subject type.
fn subject_type_icon(
    subject_type: SubjectType,
    icon_theme: IconTheme,
) -> Element<'static, NotificationMessage> {
    let color = get_subject_color(subject_type);
    let icon_size = theme::notification_scaled(14.0);
    match subject_type {
        SubjectType::Issue => icons::icon_issue(icon_size, color, icon_theme),
        SubjectType::PullRequest => icons::icon_pull_request(icon_size, color, icon_theme),
        SubjectType::Release => icons::icon_release(icon_size, color, icon_theme),
        SubjectType::Discussion => icons::icon_discussion(icon_size, color, icon_theme),
        SubjectType::CheckSuite => icons::icon_check_suite(icon_size, color, icon_theme),
        SubjectType::Commit => icons::icon_commit(icon_size, color, icon_theme),
        SubjectType::RepositoryVulnerabilityAlert => {
            icons::icon_security(icon_size, color, icon_theme)
        }
        SubjectType::Unknown => icons::icon_unknown(icon_size, color, icon_theme),
    }
}

/// Creates a notification item widget
pub fn notification_item(
    processed: &ProcessedNotification,
    icon_theme: IconTheme,
    dense: bool,
    is_priority_group: bool,
) -> Element<'_, NotificationMessage> {
    let notif = &processed.notification;
    let action = processed.action;
    let p = theme::palette();

    // Only apply priority styling if we're in the priority group
    // (in "All" mode, is_priority_group will be false even for priority notifications)
    let show_priority_style = is_priority_group && action == RuleAction::Priority;

    // Priority notifications use warning color, others use subject-type color
    let type_color = if show_priority_style {
        p.accent_warning
    } else {
        get_subject_color(notif.subject_type)
    };
    let is_unread = notif.unread;

    // --- SIZING & SPACING ---
    let padding_y = if dense { 8.0 } else { 14.0 };
    let padding_x = if dense { 12.0 } else { 16.0 };
    let content_spacing = if dense { 2.0 } else { 6.0 };
    let row_spacing = if dense { 8.0 } else { 8.0 };

    // Title row
    // In dense mode, title is slightly smaller or just cleaner
    let title_size = theme::notification_scaled(if dense { 13.0 } else { 14.0 });
    let meta_size = theme::notification_scaled(12.0);
    let reason_size = theme::notification_scaled(11.0);
    let account_size = theme::notification_scaled(10.0);

    let title = text(&notif.title).size(title_size).color(p.text_primary);

    // Account badge - show which account this notification belongs to
    let show_account_badge = !notif.account.is_empty();
    let account_badge: Option<Element<'_, NotificationMessage>> = if show_account_badge {
        Some(
            container(
                text(format!("@{}", notif.account))
                    .size(account_size)
                    .color(p.text_muted),
            )
            .padding([2, 6])
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    p.text_muted.r,
                    p.text_muted.g,
                    p.text_muted.b,
                    0.1,
                ))),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into(),
        )
    } else {
        None
    };

    // Silent indicator - show muted bell for silent notifications
    // Only relevant when rules are active (not in "All" mode where is_priority_group context indicates Unread mode)
    let silent_indicator: Option<Element<'_, NotificationMessage>> = if action == RuleAction::Silent {
        Some(
            container(text("🔕").size(account_size))
                .padding([2, 4])
                .into(),
        )
    } else {
        None
    };

    // Priority indicator for priority group items (only show the ⚡ in the priority group)
    let priority_indicator: Option<Element<'_, NotificationMessage>> = if show_priority_style {
        Some(
            container(text("⚡").size(meta_size))
                .padding([0, 4])
                .into(),
        )
    } else {
        None
    };

    // Build meta row with account badge
    let mut meta_row = row![
        subject_type_icon(notif.subject_type, icon_theme),
        Space::new().width(6),
        text(&notif.repo_full_name)
            .size(meta_size)
            .color(p.text_secondary),
        Space::new().width(8),
        text(notif.reason.label())
            .size(reason_size)
            .color(p.text_muted),
    ]
    .align_y(Alignment::Center);

    if let Some(badge) = account_badge {
        meta_row = meta_row.push(Space::new().width(8));
        meta_row = meta_row.push(badge);
    }

    if let Some(indicator) = silent_indicator {
        meta_row = meta_row.push(Space::new().width(4));
        meta_row = meta_row.push(indicator);
    }

    // Time with optional priority indicator
    let mut time_row = row![].align_y(Alignment::Center);
    if let Some(indicator) = priority_indicator {
        time_row = time_row.push(indicator);
    }
    time_row = time_row.push(text(&notif.time_ago).size(meta_size).color(p.text_muted));

    // Main content layout
    let content = if dense {
        // DENSE LAYOUT: Tighter spacing
        let mut title_row = row![
            subject_type_icon(notif.subject_type, icon_theme),
            Space::new().width(6),
            text(&notif.title).size(title_size).color(p.text_primary),
        ]
        .align_y(Alignment::Center);

        // Add account badge inline for dense mode
        if show_account_badge {
            title_row = title_row.push(Space::new().width(8));
            title_row = title_row.push(
                container(
                    text(format!("@{}", notif.account))
                        .size(account_size)
                        .color(p.text_muted),
                )
                .padding([2, 6])
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        p.text_muted.r,
                        p.text_muted.g,
                        p.text_muted.b,
                        0.1,
                    ))),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
        }

        row![
            column![
                title_row,
                row![
                    text(&notif.repo_full_name)
                        .size(meta_size)
                        .color(p.text_secondary),
                    Space::new().width(8),
                    text(notif.reason.label())
                        .size(reason_size)
                        .color(p.text_muted),
                ]
                .align_y(Alignment::Center)
                .padding([0, 20]) // Indent meta slightly
            ]
            .spacing(2)
            .width(Fill),
            container(time_row).padding([0, 8]),
        ]
        .align_y(Alignment::Center)
        .padding([padding_y, padding_x])
    } else {
        // STANDARD LAYOUT
        row![
            column![title, meta_row].spacing(content_spacing).width(Fill),
            container(time_row).padding([4, 8]),
        ]
        .spacing(row_spacing)
        .align_y(Alignment::Center)
        .padding([padding_y, padding_x])
    };

    // Wrap in button for click handling
    let item_button = button(content)
        .style(theme::notification_button)
        .on_press(NotificationMessage::Open(notif.id.clone()))
        .width(Fill);

    // Accent bar - priority uses warning color (only if in priority group), others use type color
    let bar_color = if show_priority_style {
        p.accent_warning
    } else if is_unread {
        type_color
    } else {
        Color::TRANSPARENT
    };

    let accent_bar = container(Space::new().width(3).height(Fill))
        .height(Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bar_color)),
            ..Default::default()
        });

    // Card styling - priority notifications have special background (only if in priority group)
    let card_bg = if show_priority_style {
        Color::from_rgba(p.accent_warning.r, p.accent_warning.g, p.accent_warning.b, 0.08)
    } else if is_unread {
        Color::from_rgba(type_color.r, type_color.g, type_color.b, 0.05)
    } else {
        Color::TRANSPARENT
    };

    let border_color = if show_priority_style {
        Color::from_rgba(p.accent_warning.r, p.accent_warning.g, p.accent_warning.b, 0.2)
    } else if is_unread {
        Color::from_rgba(type_color.r, type_color.g, type_color.b, 0.12)
    } else {
        Color::TRANSPARENT
    };

    let has_border = show_priority_style || is_unread;

    container(
        row![accent_bar, item_button]
            .spacing(0)
            .align_y(Alignment::Center),
    )
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(card_bg)),
        border: iced::Border {
            radius: if dense { 0.0.into() } else { 6.0.into() },
            color: border_color,
            width: if has_border { 1.0 } else { 0.0 },
        },
        ..Default::default()
    })
    .width(Fill)
    .into()
}
