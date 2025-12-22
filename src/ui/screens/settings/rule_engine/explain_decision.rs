//! Explain Decision component - shows why notifications are handled a certain way.
//!
//! This component allows users to test their rules by simulating a notification
//! and seeing which rules would match and in what priority order.

use iced::widget::{column, container, row, text, Space};
use iced::{Alignment, Element, Fill};

use crate::settings::IconTheme;
use crate::ui::icons;
use crate::ui::screens::settings::rule_engine::rules::{
    NotificationRuleSet, OutsideScheduleBehavior, RuleAction,
};
use crate::ui::theme;

use super::messages::RuleEngineMessage;

/// A matched rule for explanation display.
#[derive(Debug, Clone)]
pub struct MatchedRule {
    pub id: String,
    pub rule_type: String,
    pub name: String,
    pub action: RuleAction,
    pub priority: i32,
    pub enabled: bool,
}

/// Simulate matching rules for a given notification type.
pub fn simulate_matching_rules(
    rules: &NotificationRuleSet,
    notification_type: &str,
    account: Option<&str>,
) -> Vec<MatchedRule> {
    if !rules.enabled {
        return vec![];
    }

    let mut matches = vec![];

    // Check type rules
    for rule in &rules.type_rules {
        if rule.notification_type == notification_type {
            // Check account scope
            // When testing without a specific account (None), show all rules
            // but note that account-specific rules would only apply to that account
            let account_match = match (&rule.account, account) {
                (None, _) => true, // Global rule matches all
                (Some(rule_acc), Some(notif_acc)) => rule_acc == notif_acc, // Account-specific match
                (Some(_), None) => true, // Show account rules when no test account (informational)
            };

            if account_match {
                matches.push(MatchedRule {
                    id: rule.id.clone(),
                    rule_type: "Type".to_string(),
                    name: format!(
                        "{} ({})",
                        rule.notification_type,
                        rule.account.as_deref().unwrap_or("Global")
                    ),
                    action: rule.action,
                    priority: rule.priority,
                    enabled: rule.enabled,
                });
            }
        }
    }

    // Check account rules
    if let Some(acc) = account {
        for rule in &rules.account_rules {
            if rule.account == acc {
                let action = if rule.is_active_now() {
                    RuleAction::Show
                } else {
                    match rule.outside_behavior {
                        OutsideScheduleBehavior::Suppress => RuleAction::Hide,
                        OutsideScheduleBehavior::Defer => RuleAction::Silent,
                    }
                };
                matches.push(MatchedRule {
                    id: rule.id.clone(),
                    rule_type: "Account".to_string(),
                    name: rule.account.clone(),
                    action,
                    priority: 0,
                    enabled: rule.enabled,
                });
            }
        }
    }

    // Sort by priority (highest first) then by enabled status
    matches.sort_by(|a, b| {
        if a.enabled != b.enabled {
            return b.enabled.cmp(&a.enabled);
        }
        // Prioritize Priority action rules to top
        if a.action == RuleAction::Priority && b.action != RuleAction::Priority {
            return std::cmp::Ordering::Less;
        }
        if b.action == RuleAction::Priority && a.action != RuleAction::Priority {
            return std::cmp::Ordering::Greater;
        }
        b.priority.cmp(&a.priority)
    });

    matches
}

/// View the explanation panel.
pub fn view_explain_panel(
    rules: &NotificationRuleSet,
    test_type: &str,
    test_account: Option<&str>,
    icon_theme: IconTheme,
) -> Element<'static, RuleEngineMessage> {
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

    // Simulate matching
    let matches = simulate_matching_rules(rules, test_type, test_account);

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

        // Get the winning rule
        // 1. If any rule has Action::Priority, it overrides everything else
        let priority_override_rule = matches
            .iter()
            .find(|m| m.enabled && m.action == RuleAction::Priority)
            .cloned();

        let winner = if let Some(rule) = priority_override_rule {
            Some(rule)
        } else {
            // 2. Otherwise standard logic: Highest priority enabled rule
            matches.iter().find(|m| m.enabled).cloned()
        };

        for (_, matched) in matches.iter().enumerate() {
            let is_winner = winner.as_ref().map(|w| w.id == matched.id).unwrap_or(false);

            let action_color = match matched.action {
                RuleAction::Hide => p.accent_warning,
                RuleAction::Priority => p.accent,
                _ => p.text_primary,
            };

            let status_color = if matched.enabled {
                p.accent_success
            } else {
                p.text_muted
            };

            let badge_color = match matched.action {
                RuleAction::Hide => p.accent_warning,
                RuleAction::Priority => p.accent,
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
            let rule_type = matched.rule_type.clone();
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
                RuleAction::Priority => p.accent,
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
