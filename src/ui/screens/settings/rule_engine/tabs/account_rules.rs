//! Account Rules tab - New 3-pane design for schedule management.

use chrono::{Local, Weekday};
use iced::widget::{
    Space, button, column, container, radio, row, scrollable, text, text_input, toggler,
};
use iced::{Alignment, Element, Fill, Length};
use std::collections::HashSet;

use crate::settings::IconTheme;
use crate::ui::icons;
use crate::ui::screens::settings::rule_engine::rules::{
    AccountRule, NotificationRuleSet, OutsideScheduleBehavior,
};
use crate::ui::theme;

use super::super::messages::{AccountMessage, RuleEngineMessage};

pub fn view_account_rules_tab<'a>(
    rules: &'a NotificationRuleSet,
    icon_theme: IconTheme,
    selected_account_id: &Option<String>,
    expanded_time_windows: &HashSet<String>,
    _accounts: &[String], // unused for now, we rely on rules.account_rules
) -> Element<'a, RuleEngineMessage> {
    let p = theme::palette();

    // 1. Left Pane: Account List
    let list_pane = view_account_list(rules, selected_account_id, icon_theme);

    // 2. Center Pane: Schedule Configuration
    let center_pane = if let Some(id) = selected_account_id {
        if let Some(rule) = rules.account_rules.iter().find(|r| r.id == *id) {
            let is_expanded = expanded_time_windows.contains(id);
            view_schedule_config(rule, is_expanded, icon_theme)
        } else {
            container(text("Account not found").color(p.text_muted)).into()
        }
    } else {
        container(
            column![
                icons::icon_user(48.0, p.border_subtle, icon_theme),
                Space::new().height(16),
                text("Select an account to configure schedule").color(p.text_secondary)
            ]
            .align_x(Alignment::Center),
        )
        .width(Fill)
        .height(Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
    };

    // 3. Right Pane: Details
    let details_pane = if let Some(id) = selected_account_id {
        if let Some(rule) = rules.account_rules.iter().find(|r| r.id == *id) {
            view_account_details(rule, icon_theme)
        } else {
            Space::new().into()
        }
    } else {
        Space::new().into()
    };

    row![
        list_pane,
        container(center_pane)
            .width(Fill)
            .height(Fill)
            .padding(24)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_base)),
                ..Default::default()
            }),
        container(details_pane)
            .width(Length::Fixed(300.0)) // Fixed width for details
            .height(Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_sidebar)), // Slightly different bg
                border: iced::Border {
                    width: 1.0,
                    color: p.border_subtle,
                    ..Default::default()
                },
                ..Default::default()
            })
    ]
    .spacing(0)
    .width(Fill)
    .height(Fill)
    .into()
}

fn view_account_list<'a>(
    rules: &'a NotificationRuleSet,
    selected_id: &Option<String>,
    icon_theme: IconTheme,
) -> Element<'a, RuleEngineMessage> {
    let p = theme::palette();

    let list_items = rules.account_rules.iter().map(|rule| {
        let is_selected = selected_id.as_ref() == Some(&rule.id);
        let now = Local::now();

        let status_color = if rule.is_active(&now) && rule.enabled {
            p.accent_success
        } else {
            p.text_muted
        };

        let status_text = if !rule.enabled {
            "Disabled"
        } else if rule.is_active(&now) {
            "Active"
        } else {
            "Outside Schedule"
        };

        let content = row![
            column![
                text(&rule.account)
                    .size(14)
                    .color(p.text_primary)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                row![
                    text("●").size(10).color(status_color),
                    Space::new().width(6),
                    text(status_text).size(12).color(p.text_secondary)
                ]
                .align_y(Alignment::Center)
            ],
            Space::new().width(Fill),
            icons::icon_chevron_right(
                14.0,
                if is_selected {
                    p.accent
                } else {
                    p.border_subtle
                },
                icon_theme
            )
        ]
        .padding(12)
        .align_y(Alignment::Center);

        button(content)
            .style(move |theme, status| {
                if is_selected {
                    theme::sidebar_button(true)(theme, status)
                } else {
                    theme::ghost_button(theme, status)
                }
            })
            .width(Fill)
            .on_press(RuleEngineMessage::Account(AccountMessage::Select(
                rule.id.clone(),
            )))
            .into()
    });

    container(scrollable(
        column(list_items.collect::<Vec<_>>())
            .spacing(4)
            .padding(12),
    ))
    .width(Length::Fixed(250.0))
    .height(Fill)
    .style(move |_| container::Style {
        border: iced::Border {
            width: 1.0,
            color: p.border_subtle,
            ..Default::default()
        },
        background: Some(iced::Background::Color(p.bg_sidebar)),
        ..Default::default()
    })
    .into()
}

fn view_schedule_config<'a>(
    rule: &'a AccountRule,
    is_expanded: bool,
    icon_theme: IconTheme,
) -> Element<'a, RuleEngineMessage> {
    let p = theme::palette();

    // Section 1: Availability
    let availability_section =
        column![
            text("Account Availability").size(16).font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
            container(
                row![
                    text("Enabled").size(14).color(p.text_primary),
                    Space::new().width(8),
                    toggler(rule.enabled)
                        .on_toggle(move |v| RuleEngineMessage::Account(
                            AccountMessage::ToggleEnabled(rule.id.clone(), v)
                        ))
                        .width(Length::Shrink),
                    Space::new().width(12),
                    text("Notifications are allowed for this account")
                        .size(13)
                        .color(p.text_secondary)
                ]
                .align_y(Alignment::Center)
            )
            .padding([12, 0])
        ];

    // Section 2: Weekly Schedule
    let days = [
        ("Sun", Weekday::Sun),
        ("Mon", Weekday::Mon),
        ("Tue", Weekday::Tue),
        ("Wed", Weekday::Wed),
        ("Thu", Weekday::Thu),
        ("Fri", Weekday::Fri),
        ("Sat", Weekday::Sat),
    ];

    let grid_items = days.iter().map(|(label, day)| {
        let is_active = rule.active_days.contains(day);
        let color = if is_active {
            p.accent_success
        } else {
            p.text_muted
        };
        let icon = if is_active {
            icons::icon_check(14.0, p.bg_base, icon_theme)
        } else {
            icons::icon_x(14.0, p.bg_base, icon_theme)
        };

        button(
            column![
                text(*label).size(12).color(p.text_secondary),
                Space::new().height(8),
                container(icon)
                    .width(Length::Fixed(24.0))
                    .height(Length::Fixed(24.0))
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(color)),
                        border: iced::Border {
                            radius: 12.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .align_x(iced::Alignment::Center)
                    .align_y(iced::Alignment::Center)
            ]
            .align_x(Alignment::Center),
        )
        .on_press(RuleEngineMessage::Account(AccountMessage::ToggleDay(
            rule.id.clone(),
            *day,
        )))
        .style(theme::ghost_button)
        .into()
    });

    let weekly_schedule = column![
        text("Weekly Schedule").size(14).font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        container(row(grid_items.collect::<Vec<_>>()).spacing(16)).padding([16, 0])
    ];

    // Section 3: Time Windows
    let expand_icon = if is_expanded {
        icons::icon_chevron_down(16.0, p.text_secondary, icon_theme)
    } else {
        icons::icon_chevron_right(16.0, p.text_secondary, icon_theme)
    };

    let time_windows_header = button(
        row![
            expand_icon,
            Space::new().width(8),
            text("Time Windows")
                .size(14)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(p.text_primary),
            Space::new().width(8),
            text("Control when notifications are delivered during enabled days")
                .size(13)
                .color(p.text_muted)
        ]
        .align_y(Alignment::Center),
    )
    .style(theme::ghost_button)
    .on_press(RuleEngineMessage::Account(
        AccountMessage::SetTimeWindowExpanded(rule.id.clone(), !is_expanded),
    ))
    .padding(0);

    let time_windows_content =
        if is_expanded {
            let start_val = rule
                .start_time
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_else(|| "09:00".to_string());
            let end_val = rule
                .end_time
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or_else(|| "17:00".to_string());
            // For the setters, we just pass the current value (formatted) if the other one is changed.
            // But since we are inside a value binding, we simply get the string value to display.
            // We know we need to pass a valid string to SetAccountTimeWindow.
            // However, SetAccountTimeWindow expects Option<String>.

            let current_start_str = start_val.clone();
            let current_end_str = end_val.clone();

            column![
            Space::new().height(12),
            text("Default Window").size(13).color(p.text_secondary),
            Space::new().height(8),
            row![
                text("From:").size(13).color(p.text_muted),
                text_input("09:00", &start_val)
                    .on_input(move |s| RuleEngineMessage::Account(AccountMessage::SetTimeWindow(
                        rule.id.clone(),
                        Some(s),
                        Some(current_end_str.clone())
                    )))
                    .width(Length::Fixed(80.0))
                    .padding(6),
                Space::new().width(16),
                text("To:").size(13).color(p.text_muted),
                text_input("17:00", &end_val)
                    .on_input(move |s| RuleEngineMessage::Account(AccountMessage::SetTimeWindow(
                        rule.id.clone(),
                        Some(current_start_str.clone()),
                        Some(s)
                    )))
                    .width(Length::Fixed(80.0))
                    .padding(6),
            ]
            .align_y(Alignment::Center),
            Space::new().height(8),
            text("Notifications outside this range will be handled according to Behavior setting.")
                .size(12)
                .color(p.text_muted)
        ]
            .padding([0, 24])
        } else {
            column![]
        };

    // Section 4: Outside Schedule Behavior
    let behavior_section = column![
        text("Outside Schedule Behavior").size(14).font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        Space::new().height(12),
        column![
            radio(
                "Hide notifications",
                OutsideScheduleBehavior::Suppress,
                Some(rule.outside_behavior),
                move |b| RuleEngineMessage::Account(AccountMessage::SetOutsideBehavior(
                    rule.id.clone(),
                    b
                ))
            )
            .size(14)
            .spacing(8),
            text("Notifications are completely hidden from the list.")
                .size(12)
                .color(p.text_muted),
            Space::new().height(8),
            radio(
                "Silent (no desktop notification)",
                OutsideScheduleBehavior::Defer,
                Some(rule.outside_behavior),
                move |b| RuleEngineMessage::Account(AccountMessage::SetOutsideBehavior(
                    rule.id.clone(),
                    b
                ))
            )
            .size(14)
            .spacing(8),
            text("Notifications appear in the list but don't trigger desktop alerts.")
                .size(12)
                .color(p.text_muted),
        ]
    ];

    scrollable(column![
        availability_section,
        container(Space::new().height(1).width(Fill)).style(move |_| container::Style {
            background: Some(iced::Background::Color(p.border_subtle)),
            ..Default::default()
        }),
        Space::new().height(24),
        weekly_schedule,
        Space::new().height(12),
        container(Space::new().height(1).width(Fill)).style(move |_| container::Style {
            background: Some(iced::Background::Color(p.border_subtle)),
            ..Default::default()
        }),
        Space::new().height(24),
        time_windows_header,
        time_windows_content,
        Space::new().height(24),
        container(Space::new().height(1).width(Fill)).style(move |_| container::Style {
            background: Some(iced::Background::Color(p.border_subtle)),
            ..Default::default()
        }),
        Space::new().height(24),
        behavior_section,
    ])
    .into()
}

fn view_account_details<'a>(
    rule: &'a AccountRule,
    _icon_theme: IconTheme,
) -> Element<'a, RuleEngineMessage> {
    let p = theme::palette();

    let now = Local::now();
    let status_label = if !rule.enabled {
        "Disabled"
    } else if rule.is_active(&now) {
        "Active"
    } else {
        "Outside Schedule"
    };

    let reason_label = if !rule.enabled {
        "Account is disabled"
    } else if rule.is_active(&now) {
        "Within active schedule"
    } else {
        "Outside active window"
    };

    let active_days_count = rule.active_days.len();
    let hours_label = if let (Some(s), Some(e)) = (rule.start_time, rule.end_time) {
        format!("{} - {}", s.format("%H:%M"), e.format("%H:%M"))
    } else {
        "All day".to_string()
    };

    column![
        text("Account Status").size(14).font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        Space::new().height(8),
        text(format!("Currently: {}", status_label))
            .size(13)
            .color(p.text_primary),
        text(format!("Reason: {}", reason_label))
            .size(13)
            .color(p.text_muted),
        Space::new().height(24),
        text("Effective Schedule").size(14).font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        Space::new().height(8),
        text(format!("Active days: {}/7", active_days_count))
            .size(13)
            .color(p.text_primary),
        text(format!("Active hours: {}", hours_label))
            .size(13)
            .color(p.text_primary),
        Space::new().height(24),
        text("Interaction with Rules").size(14).font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        }),
        Space::new().height(8),
        text("• Important rules (e.g. Org, Type) still apply")
            .size(13)
            .color(p.text_muted),
        text("• Notifications will be delivered when account becomes active")
            .size(13)
            .color(p.text_muted),
    ]
    .padding(24)
    .into()
}
