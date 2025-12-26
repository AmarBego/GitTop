//! Rule Inspector component - shows detailed rule information in a side panel.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Fill, Length};

use crate::settings::IconTheme;
use crate::ui::icons;
use crate::ui::screens::settings::rule_engine::rules::{
    AccountRule, NotificationRuleSet, OrgRule, OutsideScheduleBehavior, RuleAction, TypeRule,
};
use crate::ui::theme;
use chrono::Local;

use super::messages::{InspectorMessage, RuleEngineMessage};

/// Result of finding a rule by ID across all rule types.
#[derive(Debug, Clone)]
pub enum FoundRule {
    Account(AccountRule),
    Org(OrgRule),
    Type(TypeRule),
}

impl FoundRule {
    /// Get whether the rule is enabled.
    pub fn enabled(&self) -> bool {
        match self {
            FoundRule::Account(r) => r.enabled,
            FoundRule::Org(r) => r.enabled,
            FoundRule::Type(r) => r.enabled,
        }
    }

    /// Get the rule's action.
    pub fn action(&self) -> RuleAction {
        match self {
            FoundRule::Account(r) => {
                if r.is_active(&Local::now()) {
                    RuleAction::Show
                } else {
                    match r.outside_behavior {
                        OutsideScheduleBehavior::Suppress => RuleAction::Hide,
                        OutsideScheduleBehavior::Defer => RuleAction::Silent,
                    }
                }
            }
            FoundRule::Org(r) => r.action,
            FoundRule::Type(r) => r.action,
        }
    }

    /// Get the rule type label.
    pub fn type_label(&self) -> &'static str {
        match self {
            FoundRule::Account(_) => "Account Rule",
            FoundRule::Org(_) => "Org Rule",
            FoundRule::Type(_) => "Type Rule",
        }
    }
}

/// Find a rule by ID across all rule types.
pub fn find_rule_by_id(rules: &NotificationRuleSet, id: &str) -> Option<FoundRule> {
    if let Some(r) = rules.account_rules.iter().find(|r| r.id == id) {
        return Some(FoundRule::Account(r.clone()));
    }
    if let Some(r) = rules.org_rules.iter().find(|r| r.id == id) {
        return Some(FoundRule::Org(r.clone()));
    }
    if let Some(r) = rules.type_rules.iter().find(|r| r.id == id) {
        return Some(FoundRule::Type(r.clone()));
    }
    None
}

/// View the rule inspector panel.
pub fn view_inspector(
    rules: &NotificationRuleSet,
    selected_rule_id: &str,
    icon_theme: IconTheme,
) -> Element<'static, RuleEngineMessage> {
    let p = theme::palette();

    // Find the selected rule
    let Some(rule) = find_rule_by_id(rules, selected_rule_id) else {
        return container(
            column![
                text("Rule Not Found").size(14).color(p.text_muted),
                text("The selected rule may have been deleted.")
                    .size(12)
                    .color(p.text_muted),
            ]
            .spacing(8)
            .padding(16),
        )
        .width(Length::Fixed(280.0))
        .height(Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_card)),
            border: iced::Border {
                color: p.border_subtle,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into();
    };

    // Header with close button
    let close_btn = button(icons::icon_x(14.0, p.text_secondary, icon_theme))
        .style(theme::ghost_button)
        .padding(4)
        .on_press(RuleEngineMessage::Inspector(InspectorMessage::Close));

    let header = row![
        text("Rule Details").size(14).color(p.text_primary),
        Space::new().width(Fill),
        close_btn,
    ]
    .align_y(Alignment::Center);

    // Rule type badge
    let type_badge = container(text(rule.type_label()).size(10).color(p.text_secondary))
        .padding([2, 8])
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_control)),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    // Status indicator
    let status_row = if rule.enabled() {
        row![
            icons::icon_check(12.0, p.accent_success, icon_theme),
            Space::new().width(6),
            text("Enabled").size(12).color(p.accent_success),
        ]
        .align_y(Alignment::Center)
    } else {
        row![
            icons::icon_x(12.0, p.text_muted, icon_theme),
            Space::new().width(6),
            text("Disabled").size(12).color(p.text_muted),
        ]
        .align_y(Alignment::Center)
    };

    // Action with explanation
    let action = rule.action();
    let action_color = match action {
        RuleAction::Hide => p.accent_warning,
        RuleAction::Important => p.accent,
        _ => p.text_primary,
    };

    let action_explanation = match action {
        RuleAction::Show => "Notification appears normally and triggers desktop notification.",
        RuleAction::Silent => {
            "Notification appears in list but does not trigger desktop notification."
        }
        RuleAction::Hide => "Notification is completely hidden from the list.",
        RuleAction::Important => {
            "OVERRIDES all Hide/Silent rules. Always shown and triggers desktop notification."
        }
    };

    let action_section = column![
        text("Action").size(11).color(p.text_muted),
        Space::new().height(4),
        text(action.display_label()).size(14).color(action_color),
        Space::new().height(4),
        text(action_explanation).size(11).color(p.text_secondary),
    ];

    // Warning for dangerous rules
    let warning_section = if action == RuleAction::Hide {
        Some(
            container(
                row![
                    icons::icon_alert(12.0, p.accent_warning, icon_theme),
                    Space::new().width(8),
                    text("This rule hides notifications")
                        .size(11)
                        .color(p.accent_warning),
                ]
                .align_y(Alignment::Center),
            )
            .padding([8, 12])
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_control)),
                border: iced::Border {
                    radius: 4.0.into(),
                    color: p.accent_warning,
                    width: 1.0,
                },
                ..Default::default()
            }),
        )
    } else {
        None
    };

    // Rule-specific details
    let details_section = match &rule {
        FoundRule::Account(r) => {
            let account_name = r.account.clone();
            column![
                text("Account").size(11).color(p.text_muted),
                text(account_name).size(13).color(p.text_primary),
                Space::new().height(8),
                text("Outside Schedule").size(11).color(p.text_muted),
                text(format!("{}", r.outside_behavior))
                    .size(13)
                    .color(p.text_primary),
            ]
        }
        FoundRule::Org(r) => {
            let org_name = r.org.clone();
            let priority = r.priority;
            column![
                text("Organization").size(11).color(p.text_muted),
                text(org_name).size(13).color(p.text_primary),
                Space::new().height(8),
                text("Priority").size(11).color(p.text_muted),
                text(format!("{}", priority)).size(13).color(p.text_primary),
            ]
        }
        FoundRule::Type(r) => {
            let notification_type = r.notification_type.clone();
            let account_text = r.account.clone().unwrap_or_else(|| "Global".to_string());
            let priority = r.priority;
            column![
                text("Notification Type").size(11).color(p.text_muted),
                text(notification_type).size(13).color(p.text_primary),
                Space::new().height(8),
                text("Scope").size(11).color(p.text_muted),
                text(account_text).size(13).color(p.text_primary),
                Space::new().height(8),
                text("Priority").size(11).color(p.text_muted),
                text(format!("{}", priority)).size(13).color(p.text_primary),
            ]
        }
    };

    // Assemble content
    let mut content = column![
        header,
        Space::new().height(16),
        type_badge,
        Space::new().height(12),
        status_row,
        Space::new().height(16),
        action_section,
        Space::new().height(16),
    ]
    .spacing(0);

    if let Some(warning) = warning_section {
        content = content.push(warning);
        content = content.push(Space::new().height(16));
    }

    content = content.push(details_section);

    container(content.padding(16))
        .width(Length::Fixed(280.0))
        .height(Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_card)),
            border: iced::Border {
                color: p.border_subtle,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}
