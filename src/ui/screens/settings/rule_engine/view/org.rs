//! Organization Rules view.

use iced::widget::{Space, column, text};
use iced::{Element, Fill};

use crate::settings::IconTheme;
use crate::ui::screens::settings::rule_engine::rules::NotificationRuleSet;
use crate::ui::theme;

use crate::ui::screens::settings::rule_engine::components::{view_empty_state, view_org_rule_card};
use crate::ui::screens::settings::rule_engine::messages::RuleEngineMessage;

pub fn view_org_rules_tab(
    rules: &NotificationRuleSet,
    icon_theme: IconTheme,
) -> Element<'static, RuleEngineMessage> {
    let p = theme::palette();

    let rules_list: Element<_> = if rules.org_rules.is_empty() {
        view_empty_state("Coming soon", icon_theme)
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
