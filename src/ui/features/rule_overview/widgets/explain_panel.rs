//! Explain Decision component - logic moved to rule_overview feature.

use iced::widget::{Space, column, container, row, text};
use iced::{Alignment, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::icons;
use crate::ui::screens::settings::rule_engine::rules::{NotificationRuleSet, RuleAction};
use crate::ui::theme;

use chrono::Local;

/// View the explanation panel.
pub fn view_explain_panel<'a, Message>(
    rules: &NotificationRuleSet,
    test_type: &str,
    test_account: Option<&str>,
    icon_theme: IconTheme,
) -> Element<'a, Message>
where
    Message: 'a + Clone + 'static,
{
    let p = theme::palette();

    let header = row![
        icons::icon_alert(14.0, p.accent, icon_theme),
        Space::new().width(8),
        text("Explain Decision").size(14).color(p.text_primary),
    ]
    .align_y(Alignment::Center);

    let description = text("See which rules would match a notification of this type.")
        .size(11)
        .color(p.text_secondary);

    // Simulate matching using the actual engine logic
    let matches = rules.trace(test_type, None, test_account, &Local::now(), true);

    let result_content = if matches.is_empty() {
        column![
            Space::new().height(8),
            text("No rules match this notification type.")
                .size(12)
                .color(p.text_muted),
            Space::new().height(4),
            text("Default behavior: Show + Desktop Notification")
                .size(11)
                .color(p.text_secondary),
        ]
    } else {
        let mut col = column![Space::new().height(8),].spacing(6);

        // The first match is the winner because trace() sorts them.
        let winner = matches.first();

        for matched in matches.iter() {
            let is_winner = winner.as_ref().map(|w| w.id == matched.id).unwrap_or(false);

            let action_color = match matched.action {
                RuleAction::Hide => p.accent_warning,
                RuleAction::Important => p.accent,
                _ => p.text_primary,
            };

            let status_color = if matched.enabled {
                p.accent_success
            } else {
                p.text_muted
            };

            let badge_color = match matched.action {
                RuleAction::Hide => p.accent_warning,
                RuleAction::Important => p.accent,
                _ => p.accent,
            };

            let winner_badge = if is_winner {
                row![
                    icons::icon_zap(10.0, badge_color, icon_theme),
                    Space::new().width(4),
                    text("Winner").size(9).color(badge_color),
                    Space::new().width(8),
                ]
                .align_y(Alignment::Center)
            } else {
                row![]
            };

            // Clone strings to avoid lifetime issues
            let rule_type = matched.rule_source.clone();
            let name = matched.name.clone();
            let priority_str = format!("P:{}", matched.priority);
            let action_label = matched.action.display_label();
            let status_str = if matched.enabled { "ON" } else { "OFF" };

            let rule_row = row![
                winner_badge,
                text(rule_type).size(10).color(p.text_muted),
                Space::new().width(6),
                text(name).size(11).color(p.text_primary),
                Space::new().width(Fill),
                text(priority_str).size(10).color(p.text_muted),
                Space::new().width(8),
                text(action_label).size(10).color(action_color),
                Space::new().width(8),
                text(status_str).size(9).color(status_color),
            ]
            .align_y(Alignment::Center)
            .padding([6, 10]);

            let rule_container = container(rule_row).style(move |_| {
                let bg = if is_winner { p.bg_hover } else { p.bg_control };
                container::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });

            col = col.push(rule_container);
        }

        // Show final decision
        if let Some(w) = winner {
            col = col.push(Space::new().height(12));

            let final_action_color = match w.action {
                RuleAction::Hide => p.accent_warning,
                RuleAction::Important => p.accent,
                _ => p.text_primary,
            };

            let final_action_label = w.action.display_label();

            col = col.push(
                row![
                    text("Final Action:").size(12).color(p.text_secondary),
                    Space::new().width(8),
                    text(final_action_label).size(13).color(final_action_color),
                ]
                .align_y(Alignment::Center),
            );
        }

        col
    };

    let test_type_owned = test_type.to_string();

    container(
        column![
            header,
            Space::new().height(4),
            description,
            Space::new().height(12),
            text(format!("Testing: {}", test_type_owned))
                .size(12)
                .color(p.text_secondary),
            result_content,
        ]
        .padding(16),
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
