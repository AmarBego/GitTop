//! Notification screen state views (loading, error, empty).

use iced::widget::{button, column, container, text, Space};
use iced::{Alignment, Element, Fill};

use super::screen::NotificationMessage;
use crate::ui::{icons, theme};

/// Render the loading state.
pub fn view_loading<'a>() -> Element<'a, NotificationMessage> {
    container(
        text("Loading notifications...")
            .size(14)
            .color(theme::TEXT_SECONDARY),
    )
    .width(Fill)
    .height(Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(theme::app_container)
    .into()
}

/// Render the error state with retry button.
pub fn view_error<'a>(error: &'a str) -> Element<'a, NotificationMessage> {
    let content = column![
        icons::icon_alert(32.0, theme::ACCENT_ORANGE),
        Space::new().height(16),
        text("Failed to load notifications")
            .size(16)
            .color(theme::TEXT_PRIMARY),
        Space::new().height(8),
        text(error).size(12).color(theme::TEXT_SECONDARY),
        Space::new().height(24),
        button(text("Retry").size(14))
            .style(theme::primary_button)
            .padding([10, 24])
            .on_press(NotificationMessage::Refresh),
    ]
    .align_x(Alignment::Center);

    container(content)
        .width(Fill)
        .height(Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(theme::app_container)
        .into()
}

/// Render the empty state (no notifications).
pub fn view_empty<'a>(show_all: bool) -> Element<'a, NotificationMessage> {
    let message = if show_all {
        "No notifications yet"
    } else {
        "All caught up!"
    };

    let content = column![
        icons::icon_circle_check(48.0, theme::ACCENT_GREEN),
        Space::new().height(16),
        text(message).size(16).color(theme::TEXT_PRIMARY),
        Space::new().height(8),
        text("You have no unread notifications")
            .size(12)
            .color(theme::TEXT_SECONDARY),
    ]
    .align_x(Alignment::Center);

    container(content)
        .width(Fill)
        .height(Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(theme::app_container)
        .into()
}
