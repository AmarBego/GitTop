//! Rule Engine screen - main state and layout.

use iced::widget::{Space, button, column, container, row, scrollable, text, toggler};
use iced::{Alignment, Element, Fill, Length, Task};

use crate::settings::{AppSettings, IconTheme};
use crate::ui::screens::settings::rule_engine::rules::NotificationRuleSet;
use crate::ui::{icons, theme};

use super::messages::{ExplainMessage, InspectorMessage, OrgMessage, RuleEngineMessage, RuleTab};
// use super::tabs; // Removed
use super::view;

use crate::ui::features::account_rules::{
    AccountRulesState, update_account_rule, view_account_rules_tab,
};
use crate::ui::features::type_rules::{TypeRuleFormState, update_type_rule, view_type_rules_tab};

/// Rule Engine screen state.
#[derive(Debug, Clone)]
pub struct RuleEngineScreen {
    pub rules: NotificationRuleSet,
    pub selected_tab: RuleTab,
    pub icon_theme: IconTheme,
    pub sidebar_width: f32,
    pub sidebar_font_scale: f32,
    pub accounts: Vec<String>,

    // Feature States
    pub account_rules: AccountRulesState,
    pub type_rules: TypeRuleFormState,

    // Rule Inspector State
    pub selected_rule_id: Option<String>,

    // Explain Decision State
    pub explain_test_type: String,

    // Handbook/Help State
    pub show_handbook: bool,
}

impl RuleEngineScreen {
    pub fn new(mut rules: NotificationRuleSet, settings: AppSettings) -> Self {
        let accounts: Vec<String> = settings
            .accounts
            .iter()
            .map(|a| a.username.clone())
            .collect();

        // Ensure every signed-in account has a rule entry
        for account in &accounts {
            if !rules
                .account_rules
                .iter()
                .any(|r| r.account.eq_ignore_ascii_case(account))
            {
                use crate::ui::screens::settings::rule_engine::rules::AccountRule;
                rules.account_rules.push(AccountRule::new(account));
            }
        }

        Self {
            rules,
            selected_tab: RuleTab::default(),
            icon_theme: settings.icon_theme,
            sidebar_width: settings.sidebar_width,
            sidebar_font_scale: settings.sidebar_font_scale,
            accounts,

            // Feature States
            account_rules: AccountRulesState::new(),
            type_rules: TypeRuleFormState::new(),

            selected_rule_id: None,
            explain_test_type: "Mentioned".to_string(),
            show_handbook: false,
        }
    }

    pub fn update(&mut self, message: RuleEngineMessage) -> Task<RuleEngineMessage> {
        match message {
            RuleEngineMessage::Back => Task::none(),
            RuleEngineMessage::NoOp => Task::none(),
            RuleEngineMessage::SelectTab(tab) => {
                self.selected_tab = tab;
                Task::none()
            }
            RuleEngineMessage::ToggleEnabled(enabled) => {
                self.rules.enabled = enabled;
                let _ = self.rules.save();
                Task::none()
            }
            RuleEngineMessage::ToggleHandbook => {
                self.show_handbook = !self.show_handbook;
                Task::none()
            }
            RuleEngineMessage::Account(msg) => {
                update_account_rule(&mut self.account_rules, msg, &mut self.rules)
                    .map(RuleEngineMessage::Account)
            }
            RuleEngineMessage::Org(msg) => self.update_org(msg),
            RuleEngineMessage::Type(msg) => {
                update_type_rule(&mut self.type_rules, msg, &mut self.rules)
                    .map(RuleEngineMessage::Type)
            }
            RuleEngineMessage::Inspector(msg) => self.update_inspector(msg),
            RuleEngineMessage::Explain(msg) => self.update_explain(msg),
        }
    }

    fn update_org(&mut self, message: OrgMessage) -> Task<RuleEngineMessage> {
        match message {
            OrgMessage::Toggle(id, enabled) => {
                if let Some(rule) = self.rules.org_rules.iter_mut().find(|r| r.id == id) {
                    rule.enabled = enabled;
                }
                let _ = self.rules.save();
            }
            OrgMessage::Delete(id) => {
                self.rules.org_rules.retain(|r| r.id != id);
                let _ = self.rules.save();
            }
            OrgMessage::Duplicate(id) => {
                if let Some(rule) = self.rules.org_rules.iter().find(|r| r.id == id).cloned() {
                    let mut new_rule = rule;
                    new_rule.id = uuid::Uuid::new_v4().to_string();
                    self.rules.org_rules.push(new_rule);
                    let _ = self.rules.save();
                }
            }
        }
        Task::none()
    }

    fn update_inspector(&mut self, message: InspectorMessage) -> Task<RuleEngineMessage> {
        match message {
            InspectorMessage::Select(rule_id) => {
                self.selected_rule_id = Some(rule_id);
            }
            InspectorMessage::Close => {
                self.selected_rule_id = None;
            }
        }
        Task::none()
    }

    fn update_explain(&mut self, message: ExplainMessage) -> Task<RuleEngineMessage> {
        match message {
            ExplainMessage::SetTestType(test_type) => {
                self.explain_test_type = test_type;
            }
        }
        Task::none()
    }

    // ========================================================================
    // Main Layout
    // ========================================================================

    pub fn view(&self) -> Element<'_, RuleEngineMessage> {
        let header = self.view_header();
        let sidebar = self.view_sidebar();
        let content = self.view_tab_content();

        // Build main area with optional inspector
        let main_area = if let Some(ref rule_id) = self.selected_rule_id {
            let inspector = super::inspector::view_inspector(&self.rules, rule_id, self.icon_theme);
            row![sidebar, content, inspector].height(Fill)
        } else {
            row![sidebar, content].height(Fill)
        };

        let base_layout: Element<'_, RuleEngineMessage> = column![header, main_area]
            .spacing(0)
            .width(Fill)
            .height(Fill)
            .into();

        // Overlay handbook modal if visible
        if self.show_handbook {
            let handbook = view::view_handbook_modal(self.icon_theme);
            iced::widget::stack![base_layout, handbook]
                .width(Fill)
                .height(Fill)
                .into()
        } else {
            base_layout
        }
    }

    fn view_header(&self) -> Element<'_, RuleEngineMessage> {
        let p = theme::palette();

        let back_btn = button(
            row![
                icons::icon_chevron_left(16.0, p.text_secondary, self.icon_theme),
                Space::new().width(4),
                text("Back").size(13).color(p.text_secondary),
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::ghost_button)
        .padding([6, 10])
        .on_press(RuleEngineMessage::Back);

        let title = row![
            icons::icon_filter(18.0, p.accent, self.icon_theme),
            Space::new().width(8),
            text("Rule Engine").size(18).color(p.text_primary),
        ]
        .align_y(Alignment::Center);

        // Help/Handbook button
        let help_btn = button(
            row![
                icons::icon_info(16.0, p.text_secondary, self.icon_theme),
                Space::new().width(4),
                text("Handbook").size(12).color(p.text_secondary),
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::ghost_button)
        .padding([6, 10])
        .on_press(RuleEngineMessage::ToggleHandbook);

        let enabled_toggle = row![
            text("Enabled").size(12).color(p.text_secondary),
            Space::new().width(8),
            toggler(self.rules.enabled)
                .on_toggle(RuleEngineMessage::ToggleEnabled)
                .size(18),
        ]
        .align_y(Alignment::Center);

        let header_row = row![
            back_btn,
            Space::new().width(16),
            title,
            Space::new().width(Fill),
            help_btn,
            Space::new().width(16),
            enabled_toggle,
        ]
        .align_y(Alignment::Center)
        .padding([12, 16]);

        container(header_row)
            .width(Fill)
            .style(theme::header)
            .into()
    }

    // ========================================================================
    // Sidebar Navigation
    // ========================================================================

    fn view_sidebar(&self) -> Element<'_, RuleEngineMessage> {
        let p = theme::palette();
        let t = self.icon_theme;
        let scale = self.sidebar_font_scale;

        // Base sizes
        let icon_size = 14.0 * scale;

        let nav_items = column![
            self.view_nav_item(
                "Overview",
                RuleTab::Overview,
                icons::icon_chart(icon_size, self.nav_icon_color(RuleTab::Overview), t)
            ),
            // Removed Time and Schedule items
            self.view_nav_item(
                "Accounts",
                RuleTab::AccountRules,
                icons::icon_user(icon_size, self.nav_icon_color(RuleTab::AccountRules), t)
            ),
            self.view_nav_item(
                "Organizations",
                RuleTab::OrgRules,
                icons::icon_building(icon_size, self.nav_icon_color(RuleTab::OrgRules), t)
            ),
            self.view_nav_item(
                "Types",
                RuleTab::TypeRules,
                icons::icon_tag(icon_size, self.nav_icon_color(RuleTab::TypeRules), t)
            ),
        ]
        .spacing(4)
        .padding([16, 8]);

        container(nav_items)
            .width(Length::Fixed(self.sidebar_width))
            .height(Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_sidebar)),
                border: iced::Border {
                    color: p.border_subtle,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn nav_icon_color(&self, tab: RuleTab) -> iced::Color {
        let p = theme::palette();
        if self.selected_tab == tab {
            p.accent
        } else {
            p.text_muted
        }
    }

    fn view_nav_item(
        &self,
        label: &'static str,
        tab: RuleTab,
        icon: Element<'static, RuleEngineMessage>,
    ) -> Element<'static, RuleEngineMessage> {
        let p = theme::palette();
        let is_selected = self.selected_tab == tab;
        let scale = self.sidebar_font_scale;
        let text_size = 13.0 * scale;

        let text_color = if is_selected {
            p.accent
        } else {
            p.text_primary
        };

        let content = row![
            icon,
            Space::new().width(8.0 * scale), // Scale spacing too? Maybe not explicitly requested but looks better.
            text(label).size(text_size).color(text_color),
        ]
        .align_y(Alignment::Center)
        .padding([8, 10]);

        button(content)
            .style(move |theme, status| (theme::sidebar_button(is_selected))(theme, status))
            .on_press(RuleEngineMessage::SelectTab(tab))
            .width(Fill)
            .into()
    }

    // ========================================================================
    // Tab Content
    // ========================================================================

    fn view_tab_content(&self) -> Element<'_, RuleEngineMessage> {
        let p = theme::palette();
        let t = self.icon_theme;

        match self.selected_tab {
            RuleTab::Overview => {
                let content =
                    view::overview::view_overview_tab(&self.rules, t, &self.explain_test_type);
                container(
                    scrollable(content)
                        .width(Fill)
                        .height(Fill)
                        .style(theme::scrollbar),
                )
                .width(Fill)
                .height(Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(p.bg_base)),
                    ..Default::default()
                })
                .into()
            }
            RuleTab::AccountRules => {
                // Account rules tab handles its own scrolling internally (3-pane layout)
                let content = view_account_rules_tab(
                    &self.rules,
                    t,
                    &self.account_rules.selected_account_id,
                    &self.account_rules.expanded_time_windows,
                    &self.accounts,
                );
                container(content)
                    .width(Fill)
                    .height(Fill)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(p.bg_base)),
                        ..Default::default()
                    })
                    .into()
            }
            RuleTab::OrgRules => {
                let content = view::org::view_org_rules_tab(&self.rules, t);
                container(
                    scrollable(content)
                        .width(Fill)
                        .height(Fill)
                        .style(theme::scrollbar),
                )
                .width(Fill)
                .height(Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(p.bg_base)),
                    ..Default::default()
                })
                .into()
            }
            RuleTab::TypeRules => {
                let content = view_type_rules_tab(
                    &self.rules,
                    t,
                    &self.type_rules, // Pass the whole state struct
                    &self.accounts,
                    &self.type_rules.expanded_groups,
                );
                container(
                    scrollable(content)
                        .width(Fill)
                        .height(Fill)
                        .style(theme::scrollbar),
                )
                .width(Fill)
                .height(Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(p.bg_base)),
                    ..Default::default()
                })
                .into()
            }
        }
    }
}
