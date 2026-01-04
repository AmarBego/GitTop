use iced::widget::{Space, button, column, container, row, text};
use iced::{Element, Fill, Length};

use crate::github::types::NotificationReason;
use crate::settings::IconTheme;
use crate::ui::icons;
use crate::ui::screens::settings::rule_engine::rules::NotificationRuleSet;
use crate::ui::theme;

use super::message::OverviewMessage;
use super::state::RuleOverviewState;

pub fn view(
    rules: &NotificationRuleSet,
    icon_theme: IconTheme,
    state: &RuleOverviewState,
) -> Element<'static, OverviewMessage> {
    let p = theme::palette();
    let explain_test_type = &state.explain_test_type;

    // System health stats
    let active_count = rules.active_rule_count();
    let hidden_count = rules.count_suppress_rules();
    let important_count = rules.count_high_priority_rules();

    // ========================================================================
    // Header & Greeting
    // ========================================================================
    let header = column![
        text("Overview").size(24).color(p.text_primary),
        text("System health and rule performance metrics.")
            .size(14)
            .color(p.text_secondary),
    ]
    .spacing(4);

    // ========================================================================
    // 1. Status Strip (Inline, no cards)
    // ========================================================================

    // Helper for status items
    let status_item = |label: &'static str,
                       value: String,
                       icon: Element<'static, OverviewMessage>,
                       color: iced::Color| {
        row![
            icon,
            Space::new().width(6),
            text(value)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .size(14)
                .color(color),
            Space::new().width(6),
            text(label).size(14).color(p.text_secondary),
        ]
        .align_y(iced::Alignment::Center)
    };

    let divider = || text("•").size(14).color(p.text_muted);

    let status_strip = row![
        // Active Rules
        status_item(
            "Active",
            active_count.to_string(),
            icons::icon_check::<OverviewMessage>(16.0, p.accent_success, icon_theme),
            p.text_primary
        ),
        Space::new().width(16),
        divider(),
        Space::new().width(16),
        // Hidden
        status_item(
            "Hidden",
            hidden_count.to_string(),
            if hidden_count > 0 {
                icons::icon_eye_off::<OverviewMessage>(16.0, p.accent_warning, icon_theme)
            } else {
                icons::icon_check::<OverviewMessage>(16.0, p.text_muted, icon_theme)
            },
            if hidden_count > 0 {
                p.accent_warning
            } else {
                p.text_primary
            }
        ),
        Space::new().width(16),
        divider(),
        Space::new().width(16),
        // Important
        status_item(
            "Important",
            important_count.to_string(),
            if important_count > 0 {
                icons::icon_zap::<OverviewMessage>(16.0, p.accent, icon_theme)
            } else {
                icons::icon_check::<OverviewMessage>(16.0, p.text_muted, icon_theme)
            },
            p.text_primary
        ),
    ]
    .align_y(iced::Alignment::Center);

    // ========================================================================
    // 2. Rule Distribution (Text Row)
    // ========================================================================

    // Helper for distribution items
    let dist_item = |label: &'static str, count: usize| {
        let text_color = if count > 0 {
            p.text_primary
        } else {
            p.text_muted
        };
        row![
            text(label).size(13).color(p.text_secondary),
            Space::new().width(4),
            text(format!("({})", count))
                .size(13)
                .color(text_color)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
        ]
        .align_y(iced::Alignment::Center)
    };

    let dist_divider = || {
        container(Space::new().width(1).height(12)).style(move |_| container::Style {
            background: Some(iced::Background::Color(p.text_muted)),
            ..Default::default()
        })
    };

    let distribution_row = row![
        dist_item("Account", rules.account_rules.len()),
        Space::new().width(12),
        dist_divider(),
        Space::new().width(12),
        dist_item("Org", rules.org_rules.len()),
        Space::new().width(12),
        dist_divider(),
        Space::new().width(12),
        dist_item("Type", rules.type_rules.len()),
    ]
    .align_y(iced::Alignment::Center);

    // ========================================================================
    // 3. Test Lab (Primary Action Surface)
    // ========================================================================

    let type_options: Vec<(&'static str, &'static str)> = vec![
        (NotificationReason::Mention.label(), "Mention"),
        (NotificationReason::ReviewRequested.label(), "Review"),
        (NotificationReason::Assign.label(), "Assign"),
        (NotificationReason::Comment.label(), "Comment"),
        (NotificationReason::CiActivity.label(), "CI"),
        (NotificationReason::Author.label(), "Author"),
        (NotificationReason::TeamMention.label(), "Team"),
        (NotificationReason::StateChange.label(), "State"),
    ];

    let type_owned = explain_test_type.to_string();
    let mut type_buttons = row![].spacing(8);

    for (value, display_label) in type_options {
        let is_selected = value == explain_test_type;
        let value_owned = value.to_string();

        let btn = button(text(display_label).size(12).color(if is_selected {
            p.text_primary
        } else {
            p.text_secondary
        }))
        .style(if is_selected {
            theme::primary_button
        } else {
            theme::ghost_button
        })
        .padding([6, 12])
        .on_press(OverviewMessage::SetTestType(value_owned));

        type_buttons = type_buttons.push(btn);
    }

    // Explain panel
    // Explain panel
    let explain_panel = super::widgets::explain_panel::view_explain_panel::<OverviewMessage>(
        rules,
        &type_owned,
        None,
        icon_theme,
    );

    let test_lab = container(
        column![
            row![
                icons::icon_filter::<OverviewMessage>(16.0, p.accent, icon_theme),
                Space::new().width(8),
                text("Test Lab")
                    .size(16)
                    .color(p.text_primary)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
            ]
            .align_y(iced::Alignment::Center),
            Space::new().height(4),
            text("Simulate a notification to see which rules apply.")
                .size(13)
                .color(p.text_secondary),
            Space::new().height(20),
            type_buttons,
            Space::new().height(24),
            explain_panel,
        ]
        .padding(24),
    )
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(p.bg_card)),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: p.border_subtle,
        },
        ..Default::default()
    });

    // ========================================================================
    // 4. High Impact Sidebar (Flat List)
    // ========================================================================

    let high_impact_rules = rules.get_high_impact_rules();

    let list_content: Element<'static, OverviewMessage> = if high_impact_rules.is_empty() {
        column![
            text("No high-impact rules active.")
                .size(12)
                .color(p.text_muted),
        ]
        .into()
    } else {
        column(
            high_impact_rules
                .iter()
                .take(6)
                .map(|r| {
                    let action_label = r.action.display_label();
                    // Flat text row
                    button(
                        row![
                            text("•").size(14).color(p.text_secondary),
                            Space::new().width(8),
                            column![
                                text(r.name.clone()).size(13).color(p.text_primary),
                                text(action_label).size(11).color(p.text_muted)
                            ]
                        ]
                        .align_y(iced::Alignment::Start),
                    )
                    .padding(4)
                    .style(theme::ghost_button)
                    // Future: .on_press(NavigateToRule(id))
                    .into()
                })
                .collect::<Vec<Element<'_, OverviewMessage>>>(),
        )
        .spacing(8)
        .into()
    };

    let high_impact_section = column![
        text("HIGH IMPACT")
            .size(11)
            .color(p.text_muted)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
        Space::new().height(12),
        list_content
    ]
    .width(Length::Fixed(240.0));

    // ========================================================================
    // Final Layout Assembly
    // ========================================================================

    let left_column = column![
        status_strip,
        Space::new().height(24),
        distribution_row,
        Space::new().height(24),
        test_lab
    ]
    .width(Fill);

    column![
        header,
        Space::new().height(32),
        row![left_column, Space::new().width(48), high_impact_section]
    ]
    .padding(40) // More outer padding
    .width(Fill)
    .into()
}
