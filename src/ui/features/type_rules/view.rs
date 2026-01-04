use std::collections::HashSet;

use iced::widget::{Space, button, column, container, pick_list, row, slider, text, toggler};
use iced::{Alignment, Element, Fill, Length};
use iced_aw::ContextMenu;

use crate::settings::IconTheme;
use crate::ui::icons;
use crate::ui::screens::settings::rule_engine::rules::{NotificationRuleSet, RuleAction, TypeRule};
use crate::ui::theme;

use super::TypeRuleFormState;
use crate::ui::screens::settings::rule_engine::components::{
    view_context_menu_item, view_empty_state, view_warning_row,
};
use crate::ui::screens::settings::rule_engine::messages::{
    InspectorMessage, RuleEngineMessage, TypeMessage,
};

/// Groups type rules by notification_type using BTreeMap to avoid allocations and sorting.
fn view_grouped_rules<'a>(
    rules: &'a [TypeRule],
    expanded_groups: &HashSet<String>,
    icon_theme: IconTheme,
) -> Element<'a, RuleEngineMessage> {
    use std::collections::BTreeMap;
    let p = theme::palette();

    if rules.is_empty() {
        return view_empty_state("No type rules configured.", icon_theme);
    }

    // Group by (NotificationReason, String) -> Vec<&TypeRule>
    // We use BTreeMap to automatically sort by keys
    let mut groups: BTreeMap<String, Vec<&TypeRule>> = BTreeMap::new();
    for rule in rules {
        groups
            .entry(rule.notification_type.clone())
            .or_default()
            .push(rule);
    }

    column(groups.into_iter().flat_map(|(group_name, group_rules)| {
        let is_expanded = expanded_groups.contains(&group_name);
        let count = group_rules.len();

        let chevron = if is_expanded {
            icons::icon_chevron_down(12.0, p.text_secondary, icon_theme)
        } else {
            icons::icon_chevron_right(12.0, p.text_secondary, icon_theme)
        };

        let header = button(
            row![
                chevron,
                Space::new().width(8),
                text(group_name.clone()).size(14).color(p.text_primary),
                Space::new().width(8),
                text(format!("({} rules)", count))
                    .size(12)
                    .color(p.text_muted),
            ]
            .align_y(Alignment::Center)
            .padding([8, 12]),
        )
        .style(theme::ghost_button)
        .on_press(RuleEngineMessage::Type(TypeMessage::ToggleGroup(
            group_name.clone(),
        )))
        .width(Fill);

        let header_container = container(header).style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_control)),
            border: iced::Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let mut elements = vec![header_container.into(), Space::new().height(4).into()];

        if is_expanded {
            let mut rules_column = column![].spacing(8);
            for rule in group_rules {
                rules_column = rules_column.push(view_type_rule_card(rule, icon_theme));
            }

            elements.push(row![Space::new().width(24), rules_column].into());
            elements.push(Space::new().height(8).into());
        }

        elements
    }))
    .spacing(0)
    .into()
}

pub fn view_type_rules_tab<'a>(
    rules: &'a NotificationRuleSet,
    icon_theme: IconTheme,
    form_state: &TypeRuleFormState,
    available_accounts: &[String],
    expanded_groups: &HashSet<String>,
) -> Element<'a, RuleEngineMessage> {
    let p = theme::palette();

    // ========================================================================
    // Form Section
    // ========================================================================
    let type_input = container(
        column![
            text("Type").size(12).color(p.text_secondary),
            pick_list(
                crate::github::types::NotificationReason::ALL,
                Some(form_state.notification_type),
                |r| RuleEngineMessage::Type(TypeMessage::FormTypeChanged(r))
            )
            .width(Length::Fixed(180.0))
            .style(theme::pick_list_style)
            .menu_style(theme::menu_style),
        ]
        .spacing(4),
    );

    let selected_account = form_state
        .account
        .clone()
        .unwrap_or_else(|| "Global".to_string());

    let account_input = container(
        column![
            text("Account").size(12).color(p.text_secondary),
            pick_list(
                {
                    let mut options = vec!["Global".to_string()];
                    options.extend_from_slice(available_accounts);
                    options
                },
                Some(selected_account),
                |s| RuleEngineMessage::Type(TypeMessage::FormAccountChanged(s))
            )
            .width(Length::Fixed(150.0))
            .style(theme::pick_list_style)
            .menu_style(theme::menu_style),
        ]
        .spacing(4),
    );

    let priority_input = container(
        column![
            row![
                text("Priority").size(12).color(p.text_secondary),
                Space::new().width(8),
                text(format!("{}", form_state.priority))
                    .size(12)
                    .color(p.text_primary),
            ]
            .align_y(Alignment::Center),
            slider(
                -100..=100,
                form_state.priority,
                |p| RuleEngineMessage::Type(TypeMessage::FormPriorityChanged(p))
            )
            .width(Length::Fixed(150.0)),
        ]
        .spacing(4),
    );

    // Action Input with Warning
    let action_label_row = if form_state.action == RuleAction::Hide {
        row![
            text("Action").size(12).color(p.text_secondary),
            Space::new().width(4),
            icons::icon_alert(12.0, p.accent_warning, icon_theme),
        ]
        .align_y(Alignment::Center)
    } else {
        row![text("Action").size(12).color(p.text_secondary)]
    };

    let action_input = container(
        column![
            action_label_row,
            pick_list(RuleAction::ALL, Some(form_state.action), |a| {
                RuleEngineMessage::Type(TypeMessage::FormActionChanged(a))
            })
            .width(Length::Fixed(100.0))
            .style(theme::pick_list_style)
            .menu_style(theme::menu_style),
        ]
        .spacing(4),
    );

    let add_btn = button(text("Add Rule").size(13))
        .style(theme::primary_button)
        .on_press(RuleEngineMessage::Type(TypeMessage::Add))
        .padding([8, 16]);

    let form_row = row![
        type_input,
        account_input,
        priority_input,
        action_input,
        Space::new().width(Fill),
        column![Space::new().height(19), add_btn].spacing(0),
    ]
    .spacing(12)
    .align_y(Alignment::End);

    let form_section = container(form_row)
        .padding(16)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_control)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    let header = column![
        text("Type Rules").size(20).color(p.text_primary),
        text("Filter notifications by type, account, and priority.")
            .size(12)
            .color(p.text_secondary),
    ]
    .spacing(4);

    column![
        header,
        Space::new().height(16),
        form_section,
        Space::new().height(24),
        view_grouped_rules(&rules.type_rules, expanded_groups, icon_theme)
    ]
    .padding(24)
    .width(Fill)
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
