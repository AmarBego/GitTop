//! Shared UI components for Rule Engine.

use iced::widget::{Space, button, column, container, row, text, toggler};
use iced::{Alignment, Element, Fill};
use iced_aw::ContextMenu;

use crate::settings::IconTheme;
use crate::ui::screens::settings::rule_engine::rules::{OrgRule, RuleAction, TypeRule};
use crate::ui::{icons, theme};

use super::messages::{InspectorMessage, OrgMessage, RuleEngineMessage, TypeMessage};

// ============================================================================
// Empty State
// ============================================================================

pub fn view_empty_state(
    message: &'static str,
    icon_theme: IconTheme,
) -> Element<'static, RuleEngineMessage> {
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

fn view_context_menu_item(
    label: &'static str,
    message: RuleEngineMessage,
) -> Element<'static, RuleEngineMessage> {
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

fn view_warning_row(
    message: &'static str,
    icon_theme: IconTheme,
) -> Element<'static, RuleEngineMessage> {
    let p = theme::palette();
    row![
        icons::icon_alert(12.0, p.accent_warning, icon_theme),
        Space::new().width(4),
        text(message).size(11).color(p.accent_warning),
    ]
    .align_y(Alignment::Center)
    .into()
}

// ============================================================================
// Org Rule Card
// ============================================================================

pub fn view_org_rule_card(
    rule: &OrgRule,
    icon_theme: IconTheme,
) -> Element<'static, RuleEngineMessage> {
    let p = theme::palette();
    let id = rule.id.clone();
    let id_toggle = id.clone();
    let id_dup = id.clone();
    let id_dup2 = id.clone();
    let id_delete = id.clone();
    let id_delete2 = id.clone();
    let id_select = id;
    let enabled = rule.enabled;

    let priority = format!("Priority: {}", rule.priority);
    let action_str = format!("Action: {}", rule.action.display_label());

    let mut info_column = column![
        text(rule.org.clone()).size(14).color(p.text_primary),
        text(priority).size(12).color(p.text_secondary),
        text(action_str).size(11).color(p.text_muted),
    ]
    .width(Fill);

    if rule.priority > 100 || rule.priority < -100 {
        info_column = info_column.push(Space::new().height(4));
        info_column = info_column.push(view_warning_row("Non-standard priority", icon_theme));
    }
    if rule.action == RuleAction::Hide {
        info_column = info_column.push(Space::new().height(4));
        info_column = info_column.push(view_warning_row("Hides notifications", icon_theme));
    }

    // Make info content clickable to open inspector
    let clickable_info = button(info_column)
        .style(theme::ghost_button)
        .padding(0)
        .on_press(RuleEngineMessage::Inspector(InspectorMessage::Select(
            id_select,
        )));

    // Visible action buttons
    let dup_btn = button(icons::icon_plus(14.0, p.text_muted, icon_theme))
        .style(theme::ghost_button)
        .padding(6)
        .on_press(RuleEngineMessage::Org(OrgMessage::Duplicate(id_dup)));

    let delete_btn = button(icons::icon_trash(14.0, p.text_muted, icon_theme))
        .style(theme::ghost_button)
        .padding(6)
        .on_press(RuleEngineMessage::Org(OrgMessage::Delete(id_delete)));

    let action_buttons = row![dup_btn, delete_btn,].spacing(2);

    let card_content = container(
        row![
            clickable_info,
            Space::new().width(Fill),
            action_buttons,
            Space::new().width(8),
            toggler(enabled)
                .on_toggle(move |e| RuleEngineMessage::Org(OrgMessage::Toggle(
                    id_toggle.clone(),
                    e
                )))
                .size(18),
        ]
        .align_y(Alignment::Center)
        .padding(14),
    )
    .style(|_| theme::rule_card_container());

    ContextMenu::new(card_content, move || {
        container(
            column![
                view_context_menu_item(
                    "Duplicate",
                    RuleEngineMessage::Org(OrgMessage::Duplicate(id_dup2.clone()))
                ),
                view_context_menu_item(
                    "Delete",
                    RuleEngineMessage::Org(OrgMessage::Delete(id_delete2.clone()))
                ),
            ]
            .spacing(2),
        )
        .style(|_| theme::context_menu_container())
        .padding(4)
        .width(140)
        .into()
    })
    .into()
}

// ============================================================================
// Type Rule Card
// ============================================================================

pub fn view_type_rule_card(
    rule: &TypeRule,
    icon_theme: IconTheme,
) -> Element<'static, RuleEngineMessage> {
    let p = theme::palette();
    let id = rule.id.clone();
    let id_toggle = id.clone();
    let id_dup = id.clone();
    let id_dup2 = id.clone();
    let id_delete = id.clone();
    let id_delete2 = id.clone();
    let id_select = id;
    let enabled = rule.enabled;

    let account = rule.account.clone().unwrap_or_else(|| "Global".to_string());
    let priority = format!("Priority: {}", rule.priority);
    let action_str = format!("Action: {}", rule.action.display_label());

    let mut info_column = column![
        text(rule.notification_type.clone())
            .size(14)
            .color(p.text_primary),
        Space::new().height(4),
        row![
            text(account).size(12).color(p.text_secondary),
            text("•").size(12).color(p.text_muted),
            text(priority).size(12).color(p.text_secondary),
        ]
        .spacing(6),
        text(action_str).size(11).color(p.text_muted),
    ]
    .width(Fill);

    if rule.priority > 100 || rule.priority < -100 {
        info_column = info_column.push(Space::new().height(4));
        info_column = info_column.push(view_warning_row("Non-standard priority", icon_theme));
    }
    if rule.action == RuleAction::Hide {
        info_column = info_column.push(Space::new().height(4));
        info_column = info_column.push(view_warning_row("Hides notifications", icon_theme));
    }

    // Make info content clickable to open inspector
    let clickable_info = button(info_column)
        .style(theme::ghost_button)
        .padding(0)
        .on_press(RuleEngineMessage::Inspector(InspectorMessage::Select(
            id_select,
        )));

    // Visible action buttons
    let dup_btn = button(icons::icon_plus(14.0, p.text_muted, icon_theme))
        .style(theme::ghost_button)
        .padding(6)
        .on_press(RuleEngineMessage::Type(TypeMessage::Duplicate(id_dup)));

    let delete_btn = button(icons::icon_trash(14.0, p.text_muted, icon_theme))
        .style(theme::ghost_button)
        .padding(6)
        .on_press(RuleEngineMessage::Type(TypeMessage::Delete(id_delete)));

    let action_buttons = row![dup_btn, delete_btn,].spacing(2);

    let card_content = container(
        row![
            clickable_info,
            Space::new().width(Fill),
            action_buttons,
            Space::new().width(8),
            toggler(enabled)
                .on_toggle(move |e| RuleEngineMessage::Type(TypeMessage::Toggle(
                    id_toggle.clone(),
                    e
                )))
                .size(18),
        ]
        .align_y(Alignment::Center)
        .padding(14),
    )
    .style(|_| theme::rule_card_container());

    ContextMenu::new(card_content, move || {
        container(
            column![
                view_context_menu_item(
                    "Duplicate",
                    RuleEngineMessage::Type(TypeMessage::Duplicate(id_dup2.clone()))
                ),
                view_context_menu_item(
                    "Delete",
                    RuleEngineMessage::Type(TypeMessage::Delete(id_delete2.clone()))
                ),
            ]
            .spacing(2),
        )
        .style(|_| theme::context_menu_container())
        .padding(4)
        .width(140)
        .into()
    })
    .into()
}
