//! Notification item widget - displays a single notification.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Color, Element, Fill};

use crate::github::types::{NotificationView, SubjectType};
use crate::ui::screens::notifications::NotificationMessage;
use crate::ui::{icons, theme};

/// Get color for subject type
fn get_subject_color(subject_type: SubjectType) -> Color {
    match subject_type {
        SubjectType::Issue => theme::ACCENT_GREEN,
        SubjectType::PullRequest => theme::ACCENT_BLUE,
        SubjectType::Release => theme::ACCENT_PURPLE,
        SubjectType::Discussion => theme::ACCENT_BLUE,
        SubjectType::CheckSuite => theme::ACCENT_ORANGE,
        SubjectType::RepositoryVulnerabilityAlert => theme::ACCENT_RED,
        _ => theme::TEXT_SECONDARY,
    }
}

/// Get the SVG icon for a subject type.
fn subject_type_icon(subject_type: SubjectType) -> iced::widget::Svg<'static> {
    let color = get_subject_color(subject_type);
    match subject_type {
        SubjectType::Issue => icons::icon_issue(14.0, color),
        SubjectType::PullRequest => icons::icon_pull_request(14.0, color),
        SubjectType::Release => icons::icon_release(14.0, color),
        SubjectType::Discussion => icons::icon_discussion(14.0, color),
        SubjectType::CheckSuite => icons::icon_check_suite(14.0, color),
        SubjectType::Commit => icons::icon_commit(14.0, color),
        SubjectType::RepositoryVulnerabilityAlert => icons::icon_security(14.0, color),
        SubjectType::Unknown => icons::icon_unknown(14.0, color),
    }
}

/// Creates a notification item widget - optimized for minimal allocations.
pub fn notification_item(notif: &NotificationView) -> Element<'_, NotificationMessage> {
    // Title row
    let title = text(&notif.title).size(13).color(theme::TEXT_PRIMARY);

    // Meta row: icon + repo + reason
    let meta = row![
        subject_type_icon(notif.subject_type),
        Space::new().width(4),
        text(&notif.repo_full_name)
            .size(11)
            .color(theme::TEXT_SECONDARY),
        Space::new().width(8),
        text(notif.reason.label()).size(10).color(theme::TEXT_MUTED),
    ]
    .align_y(Alignment::Center);

    // Time
    let time = text(&notif.time_ago).size(11).color(theme::TEXT_MUTED);

    // Unread dot (only render container if unread)
    let left: Element<'_, NotificationMessage> = if notif.unread {
        container(Space::new().width(6).height(6))
            .style(|_| container::Style {
                background: Some(iced::Background::Color(theme::ACCENT_BLUE)),
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .width(20)
            .align_y(Alignment::Center)
            .into()
    } else {
        Space::new().width(20).into()
    };

    // Main content
    let content = row![
        left,
        column![title, meta].spacing(4).width(Fill),
        container(time).padding([4, 0]),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(12);

    button(content)
        .style(theme::notification_button)
        .on_press(NotificationMessage::Open(notif.id.clone()))
        .width(Fill)
        .into()
}
