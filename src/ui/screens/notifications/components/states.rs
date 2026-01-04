//! Notification screen state views (loading, error, empty).

use iced::widget::{Space, button, column, container, text};
use iced::{Alignment, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::screens::notifications::messages::NotificationMessage;
use crate::ui::{icons, theme};

pub fn view_loading<'a>() -> Element<'a, NotificationMessage> {
    let p = theme::palette();
    container(
        text("Loading notifications...")
            .size(14)
            .color(p.text_secondary),
    )
    .width(Fill)
    .height(Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .style(theme::app_container)
    .into()
}

pub fn view_error<'a>(error: &'a str, icon_theme: IconTheme) -> Element<'a, NotificationMessage> {
    let p = theme::palette();
    let content = column![
        icons::icon_alert(32.0, p.accent_warning, icon_theme),
        Space::new().height(16),
        text("Failed to load notifications")
            .size(16)
            .color(p.text_primary),
        Space::new().height(8),
        text(error).size(12).color(p.text_secondary),
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

pub enum EmptyState {
    NoNotifications,
    AllCaughtUp,
}

pub fn view_empty<'a>(
    state: EmptyState,
    icon_theme: IconTheme,
) -> Element<'a, NotificationMessage> {
    let p = theme::palette();
    let message = match state {
        EmptyState::NoNotifications => "No notifications yet",
        EmptyState::AllCaughtUp => "All caught up!",
    };

    let content = column![
        icons::icon_circle_check(48.0, p.accent_success, icon_theme),
        Space::new().height(16),
        text(message).size(16).color(p.text_primary),
        Space::new().height(8),
        text("You have no unread notifications")
            .size(12)
            .color(p.text_secondary),
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
