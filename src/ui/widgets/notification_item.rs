//! Notification item widget - displays a single notification.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Fill};

use crate::github::types::{NotificationView, SubjectType};
use crate::settings::IconTheme;
use crate::ui::screens::notifications::NotificationMessage;
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
    let icon_size = theme::scaled(14.0);
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

/// Creates a notification item widget - optimized for minimal allocations.
pub fn notification_item(
    notif: &NotificationView,
    icon_theme: IconTheme,
) -> Element<'_, NotificationMessage> {
    let p = theme::palette();

    // Title row - uses scaled font size (f32 for iced Pixels)
    let title_size = theme::scaled(14.0);
    let meta_size = theme::scaled(12.0);
    let reason_size = theme::scaled(11.0);

    let title = text(&notif.title).size(title_size).color(p.text_primary);

    // Meta row: icon + repo + reason
    let meta = row![
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

    // Time
    let time = text(&notif.time_ago).size(meta_size).color(p.text_muted);

    // Unread dot (only render container if unread)
    let left: Element<'_, NotificationMessage> = if notif.unread {
        container(Space::new().width(8).height(8))
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.accent)),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .width(24)
            .align_y(Alignment::Center)
            .into()
    } else {
        Space::new().width(24).into()
    };

    // Main content
    let content = row![
        left,
        column![title, meta].spacing(6).width(Fill),
        container(time).padding([4, 8]),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding([14, 12]);

    button(content)
        .style(theme::notification_button)
        .on_press(NotificationMessage::Open(notif.id.clone()))
        .width(Fill)
        .into()
}
