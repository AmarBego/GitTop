use iced::widget::{Space, column, text};
use iced::{Element, Fill};

use crate::settings::IconTheme;
use crate::ui::screens::settings::rule_engine::components::view_empty_state;
use crate::ui::screens::settings::rule_engine::rules::NotificationRuleSet;
use crate::ui::theme;

use super::message::OrgMessage;

pub fn view(rules: &NotificationRuleSet, icon_theme: IconTheme) -> Element<'static, OrgMessage> {
    let p = theme::palette();

    let rules_list: Element<_> = if rules.org_rules.is_empty() {
        view_empty_state::<OrgMessage>("Coming soon", icon_theme)
    } else {
        column(rules.org_rules.iter().flat_map(|rule| {
            [
                view_org_rule_card(rule, icon_theme),
                Space::new().height(8).into(),
            ]
        }))
        .into()
    };

    column![
        text("Organization Rules").size(20).color(p.text_primary),
        text("Set priority levels for organizations.")
            .size(12)
            .color(p.text_secondary),
        Space::new().height(16),
        rules_list,
    ]
    .spacing(4)
    .padding(24)
    .width(Fill)
    .into()
}

// ============================================================================
// Org Rule Card
// ============================================================================

fn view_org_rule_card(
    rule: &crate::ui::screens::settings::rule_engine::rules::OrgRule,
    icon_theme: IconTheme,
) -> Element<'static, OrgMessage> {
    use crate::ui::icons;
    use iced::Alignment;
    use iced::widget::{button, container, row, toggler};

    let p = theme::palette();
    let id = rule.id.clone();
    let id_toggle = id.clone();
    let id_dup = id.clone();
    let id_delete = id.clone();
    let enabled = rule.enabled;

    let info_column = column![
        text(rule.org.clone()).size(14).color(p.text_primary),
        Space::new().height(4),
        text(format!("Action: {}", rule.action.display_label()))
            .size(11)
            .color(p.text_muted),
    ]
    .width(Fill);

    // Visible action buttons
    let dup_btn = button(icons::icon_plus(14.0, p.text_muted, icon_theme))
        .style(theme::ghost_button)
        .padding(6)
        .on_press(OrgMessage::Duplicate(id_dup));

    let delete_btn = button(icons::icon_trash(14.0, p.text_muted, icon_theme))
        .style(theme::ghost_button)
        .padding(6)
        .on_press(OrgMessage::Delete(id_delete));

    let action_buttons = row![dup_btn, delete_btn,].spacing(2);

    container(
        row![
            info_column,
            Space::new().width(8),
            action_buttons,
            Space::new().width(8),
            toggler(enabled)
                .on_toggle(move |e| OrgMessage::Toggle(id_toggle.clone(), e))
                .size(18),
        ]
        .align_y(Alignment::Center)
        .padding(14),
    )
    .style(|_| theme::rule_card_container())
    .into()
}
