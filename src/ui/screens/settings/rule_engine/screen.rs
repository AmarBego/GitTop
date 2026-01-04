//! Rule Engine screen - routing shell.

use iced::widget::{Space, button, column, container, row, text, toggler};
use iced::{Alignment, Element, Fill, Length, Task};

use crate::settings::{AppSettings, IconTheme};
use crate::ui::icons;
use crate::ui::screens::settings::rule_engine::rules::{AccountRule, NotificationRuleSet};
use crate::ui::theme;

use super::messages::{InspectorMessage, RuleEngineMessage, RuleTab};

// Feature imports
use crate::ui::features::account_rules::{self, AccountRulesState};
use crate::ui::features::org_rules::{self, OrgRulesState};
use crate::ui::features::rule_overview::{self, RuleOverviewState};
use crate::ui::features::type_rules::{self, TypeRuleFormState};

pub struct RuleEngineScreen {
    // Data Model
    pub rules: NotificationRuleSet,
    pub accounts: Vec<String>,
    pub icon_theme: IconTheme,

    // UI State
    active_tab: RuleTab,
    inspector_selected_rule: Option<String>,

    // Feature States
    account_rules: AccountRulesState,
    type_rules: TypeRuleFormState,
    org_rules: OrgRulesState,
    overview: RuleOverviewState,
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
                rules.account_rules.push(AccountRule::new(account));
            }
        }

        Self {
            rules,
            accounts,
            icon_theme: settings.icon_theme,
            active_tab: RuleTab::Overview, // Default tab
            inspector_selected_rule: None,

            account_rules: AccountRulesState::default(),
            type_rules: TypeRuleFormState::default(),
            org_rules: OrgRulesState::default(),
            overview: RuleOverviewState::default(),
        }
    }

    pub fn update(&mut self, message: RuleEngineMessage) -> Task<RuleEngineMessage> {
        match message {
            RuleEngineMessage::Back => Task::none(), // Handled by parent
            RuleEngineMessage::SelectTab(tab) => {
                self.active_tab = tab;
                Task::none()
            }
            RuleEngineMessage::ToggleEnabled(enabled) => {
                self.rules.enabled = enabled;
                let _ = self.rules.save();
                Task::none()
            }

            // Feature Delegation
            // Argument order: (state, message, rules) based on verified signatures
            RuleEngineMessage::Account(msg) => {
                let task = account_rules::update_account_rule(
                    &mut self.account_rules,
                    msg,
                    &mut self.rules,
                );
                task.map(RuleEngineMessage::Account)
            }
            RuleEngineMessage::Type(msg) => {
                let task = type_rules::update_type_rule(&mut self.type_rules, msg, &mut self.rules);
                task.map(RuleEngineMessage::Type)
            }
            RuleEngineMessage::Org(msg) => {
                let task = org_rules::update::update(&mut self.org_rules, msg, &mut self.rules);
                task.map(RuleEngineMessage::Org)
            }
            RuleEngineMessage::Overview(msg) => {
                // Overview update only requires state, not rules? Check signature.
                // Step 784: update(state, message) -> Task
                let task = rule_overview::update::update(&mut self.overview, msg);
                task.map(RuleEngineMessage::Overview)
            }

            RuleEngineMessage::Inspector(msg) => match msg {
                InspectorMessage::Select(id) => {
                    self.inspector_selected_rule = Some(id);
                    Task::none()
                }
                InspectorMessage::Close => {
                    self.inspector_selected_rule = None;
                    Task::none()
                }
            },
        }
    }

    pub fn view(&self) -> Element<'_, RuleEngineMessage> {
        let p = theme::palette();

        let back_btn = button(icons::icon_chevron_left(
            16.0,
            p.text_secondary,
            self.icon_theme,
        ))
        .style(theme::ghost_button)
        .padding(4)
        .on_press(RuleEngineMessage::Back);

        let header = row![
            back_btn,
            Space::new().width(8),
            icons::icon_filter(20.0, p.accent, self.icon_theme),
            Space::new().width(8),
            column![
                text("Rule Engine").size(16).color(p.text_primary),
                text(if self.rules.enabled {
                    "Active"
                } else {
                    "Paused"
                })
                .size(12)
                .color(if self.rules.enabled {
                    p.accent_success
                } else {
                    p.text_muted
                }),
            ],
            Space::new().width(Fill),
            toggler(self.rules.enabled)
                .on_toggle(RuleEngineMessage::ToggleEnabled)
                .width(Length::Shrink)
                .size(20),
        ]
        .align_y(Alignment::Center)
        .padding([16, 24]);

        // Tabs
        let tabs = row![
            view_tab_title(
                "Overview",
                self.active_tab == RuleTab::Overview,
                RuleEngineMessage::SelectTab(RuleTab::Overview)
            ),
            view_tab_title(
                "Type Rules",
                self.active_tab == RuleTab::TypeRules,
                RuleEngineMessage::SelectTab(RuleTab::TypeRules)
            ),
            view_tab_title(
                "Account Rules",
                self.active_tab == RuleTab::AccountRules,
                RuleEngineMessage::SelectTab(RuleTab::AccountRules)
            ),
            view_tab_title(
                "Org Rules",
                self.active_tab == RuleTab::OrgRules,
                RuleEngineMessage::SelectTab(RuleTab::OrgRules)
            ),
        ]
        .spacing(24)
        .padding([0, 24]);

        // Content
        let content = self.view_tab_content();

        // Main Layout
        let main_content = column![header, tabs, Space::new().height(16), content]
            .spacing(0)
            .width(Fill)
            .height(Fill);

        if let Some(rule_id) = &self.inspector_selected_rule {
            let inspector_view =
                super::inspector::view_inspector(&self.rules, rule_id, self.icon_theme);

            container(row![main_content, inspector_view])
                .style(theme::app_container)
                .width(Fill)
                .height(Fill)
                .into()
        } else {
            container(main_content)
                .style(theme::app_container)
                .width(Fill)
                .height(Fill)
                .into()
        }
    }

    fn view_tab_content(&self) -> Element<'_, RuleEngineMessage> {
        match self.active_tab {
            RuleTab::Overview => {
                // Signature: view(rules, icon_theme, state) -> OverviewMessage
                rule_overview::view(&self.rules, self.icon_theme, &self.overview)
                    .map(RuleEngineMessage::Overview)
            }
            RuleTab::TypeRules => {
                // Return RuleEngineMessage directly
                type_rules::view_type_rules_tab(
                    &self.rules,
                    self.icon_theme,
                    &self.type_rules,
                    &self.accounts,
                    &self.type_rules.expanded_groups,
                )
            }
            RuleTab::AccountRules => {
                // Return RuleEngineMessage directly
                account_rules::view_account_rules_tab(
                    &self.rules,
                    self.icon_theme, // Account rules view signature: (rules, icon_theme, selected_id, expanded_time, accounts)
                    &self.account_rules.selected_account_id,
                    &self.account_rules.expanded_time_windows,
                    &self.accounts,
                )
            }
            RuleTab::OrgRules => {
                // Returns OrgMessage -> map to RuleEngineMessage::Org
                org_rules::view(&self.rules, self.icon_theme).map(RuleEngineMessage::Org)
            }
        }
    }
}

// Helper for tabs
fn view_tab_title<'a>(
    title: &'static str,
    is_active: bool,
    on_press: RuleEngineMessage,
) -> Element<'a, RuleEngineMessage> {
    let p = theme::palette();

    let content = column![
        text(title).size(14).color(if is_active {
            p.text_primary
        } else {
            p.text_muted
        }),
        Space::new().height(8),
        iced::widget::container(Space::new())
            .width(Length::Fixed(20.0)) // simplified indicator
            .height(Length::Fixed(2.0))
            .style(move |_| iced::widget::container::Style {
                background: if is_active {
                    Some(iced::Background::Color(p.accent))
                } else {
                    None
                },
                ..Default::default()
            })
    ]
    .align_x(Alignment::Center);

    button(content)
        .style(theme::ghost_button)
        .padding(0)
        .on_press(on_press)
        .into()
}
