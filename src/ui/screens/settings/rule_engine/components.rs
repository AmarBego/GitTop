//! Shared UI components for Rule Engine.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::{icons, theme};

// ============================================================================
// Empty State
// ============================================================================

pub fn view_empty_state<'a, Message>(
    message: &'static str,
    icon_theme: IconTheme,
) -> Element<'a, Message>
where
    Message: 'a + Clone + 'static,
{
    let p = theme::palette();

    container(
        column![
            icons::icon_inbox_empty(32.0, p.text_muted, icon_theme),
            Space::new().height(8),
            text(message).size(12).color(p.text_muted),
        ]
        .align_x(Alignment::Center)
        .padding(32),
    )
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(p.bg_card)),
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

// ============================================================================
// Context Menu Helpers
// ============================================================================

pub fn view_context_menu_item<'a, Message>(
    label: &'static str,
    message: Message,
) -> Element<'a, Message>
where
    Message: 'a + Clone + 'static,
{
    let p = theme::palette();

    button(text(label).size(12).color(p.text_primary))
        .style(move |_theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => p.bg_hover,
                iced::widget::button::Status::Pressed => p.bg_active,
                _ => p.bg_control,
            };
            iced::widget::button::Style {
                background: Some(iced::Background::Color(bg)),
                ..Default::default()
            }
        })
        .padding([6, 12])
        .width(Fill)
        .on_press(message)
        .into()
}

// ============================================================================
// Warning Row Helper
// ============================================================================

pub fn view_warning_row<'a, Message>(
    message: &'static str,
    icon_theme: IconTheme,
) -> Element<'a, Message>
where
    Message: 'a + Clone + 'static,
{
    let p = theme::palette();
    row![
        icons::icon_alert(12.0, p.accent_warning, icon_theme),
        Space::new().width(4),
        text(message).size(11).color(p.accent_warning),
    ]
    .align_y(Alignment::Center)
    .into()
}
