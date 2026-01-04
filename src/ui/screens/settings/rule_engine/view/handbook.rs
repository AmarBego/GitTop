//! Handbook modal view for Rule Engine.

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length};

use crate::settings::IconTheme;
use crate::ui::{icons, theme};

use crate::ui::screens::settings::rule_engine::messages::RuleEngineMessage;

/// View the handbook/help modal overlay.
pub fn view_handbook_modal(icon_theme: IconTheme) -> Element<'static, RuleEngineMessage> {
    let p = theme::palette();

    // Backdrop (clickable to close)
    let backdrop_btn = button(Space::new().width(Fill).height(Fill))
        .width(Fill)
        .height(Fill)
        .style(|_, _| button::Style {
            background: Some(iced::Background::Color(iced::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.6,
            })),
            ..Default::default()
        })
        .on_press(RuleEngineMessage::ToggleHandbook);

    // Handbook content
    let content = column![
        // Header
        row![
            text("Rule Engine Handbook")
                .size(18)
                .color(p.text_primary)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
            Space::new().width(Fill),
            button(icons::icon_x(16.0, p.text_secondary, icon_theme))
                .style(theme::ghost_button)
                .padding(4)
                .on_press(RuleEngineMessage::ToggleHandbook),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Core Principle
        text("Core Principle")
            .size(14)
            .color(p.accent)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
        text(
            "Notifications are SHOWN by default. \
Rules only exist to restrict, silence, hide, or elevate notifications."
        )
        .size(13)
        .color(p.text_secondary),
        Space::new().height(12),
        // Actions
        text("Actions").size(14).color(p.accent).font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        text(
            "• Show - Visible in the in-app list and triggers a desktop notification. \
This is the default behavior when no rules apply."
        )
        .size(13)
        .color(p.text_secondary),
        text("• Silent - Visible in the in-app list but does NOT trigger a desktop notification.")
            .size(13)
            .color(p.text_secondary),
        text(
            "• Hide - Completely hidden. The notification does NOT appear in the list \
and does NOT trigger a desktop notification."
        )
        .size(13)
        .color(p.text_secondary),
        text(
            "• Important - Always visible and always triggers a desktop notification. \
Important notifications bypass account rules, schedules, Hide, and Silent actions, \
and are shown across ALL configured accounts. Important notifications are pinned \
at the top of every notification list."
        )
        .size(13)
        .color(p.accent),
        Space::new().height(12),
        // Priority Value
        text("Priority Value (−100 to +100)")
            .size(14)
            .color(p.accent)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
        text(
            "The priority value ONLY affects the sort order of notifications \
within the in-app list. Higher values appear first."
        )
        .size(13)
        .color(p.text_secondary),
        text(
            "Priority does NOT affect desktop notifications and does NOT override \
Hide or Silent actions. Only the Important action can override suppression."
        )
        .size(13)
        .color(p.text_muted),
        Space::new().height(12),
        // Resolution Order
        text("Rule Resolution Order")
            .size(14)
            .color(p.accent)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
        text(
            "1. Important always wins. \
If ANY matching rule marks a notification as Important, \
it is treated as Important regardless of other rules."
        )
        .size(13)
        .color(p.text_secondary),
        text("2. If no Important rule applies, the rule with the highest priority value wins.")
            .size(13)
            .color(p.text_secondary),
        text(
            "3. If priority values are equal, the most restrictive action wins \
(Hide > Silent > Show)."
        )
        .size(13)
        .color(p.text_secondary),
    ]
    .spacing(4)
    .padding(24)
    .width(450);

    let scrollable_content = scrollable(content)
        .style(theme::scrollbar)
        .height(Length::Shrink);

    let modal_card = container(scrollable_content)
        .style(theme::card)
        .max_width(500)
        .max_height(600);

    // Wrap modal in mouse_area to prevent clicks from bubbling to backdrop
    let modal_with_blocker = iced::widget::mouse_area(modal_card).on_press(RuleEngineMessage::NoOp);

    // Center the modal
    let centered = container(modal_with_blocker)
        .width(Fill)
        .height(Fill)
        .padding(40)
        .center_x(Fill)
        .center_y(Fill);

    // Stack backdrop + modal
    iced::widget::stack![backdrop_btn, centered]
        .width(Fill)
        .height(Fill)
        .into()
}
