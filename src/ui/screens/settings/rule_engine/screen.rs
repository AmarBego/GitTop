//! Rule Engine screen - main state and layout.

use iced::widget::{Space, button, column, container, row, scrollable, text, toggler};
use iced::{Alignment, Element, Fill, Length, Task};

use crate::settings::{AppSettings, IconTheme};
use crate::ui::screens::settings::rule_engine::rules::{NotificationRuleSet, RuleAction, TypeRule};
use crate::ui::{icons, theme};
use chrono::NaiveTime;

use super::messages::{
    AccountMessage, ExplainMessage, InspectorMessage, OrgMessage, RuleEngineMessage, RuleTab,
    TypeMessage,
};
use super::tabs;

/// Rule Engine screen state.
#[derive(Debug, Clone)]
pub struct RuleEngineScreen {
    pub rules: NotificationRuleSet,
    pub selected_tab: RuleTab,
    pub icon_theme: IconTheme,
    pub sidebar_width: f32,
    pub sidebar_font_scale: f32,
    pub accounts: Vec<String>,

    pub selected_account_id: Option<String>,
    pub expanded_account_time_windows: std::collections::HashSet<String>,

    // Type Rule Creation State
    pub new_type_rule_type: crate::github::types::NotificationReason,
    pub new_type_rule_account: Option<String>,
    pub new_type_rule_priority: i32,
    pub new_type_rule_action: RuleAction,

    // Type Rules Grouping State
    pub expanded_type_groups: std::collections::HashSet<String>,

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

            selected_account_id: None,
            expanded_account_time_windows: std::collections::HashSet::new(),

            new_type_rule_type: crate::github::types::NotificationReason::Mention,
            new_type_rule_account: None,
            new_type_rule_priority: 0,
            new_type_rule_action: RuleAction::Show,
            expanded_type_groups: std::collections::HashSet::new(),
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
            RuleEngineMessage::Account(msg) => self.update_account(msg),
            RuleEngineMessage::Org(msg) => self.update_org(msg),
            RuleEngineMessage::Type(msg) => self.update_type(msg),
            RuleEngineMessage::Inspector(msg) => self.update_inspector(msg),
            RuleEngineMessage::Explain(msg) => self.update_explain(msg),
        }
    }

    fn update_account(&mut self, message: AccountMessage) -> Task<RuleEngineMessage> {
        match message {
            AccountMessage::Select(id) => {
                self.selected_account_id = Some(id);
            }
            AccountMessage::ToggleEnabled(id, enabled) => {
                if let Some(rule) = self.rules.account_rules.iter_mut().find(|r| r.id == id) {
                    rule.enabled = enabled;
                    let _ = self.rules.save();
                }
            }
            AccountMessage::ToggleDay(id, day) => {
                if let Some(rule) = self.rules.account_rules.iter_mut().find(|r| r.id == id) {
                    if rule.active_days.contains(&day) {
                        rule.active_days.remove(&day);
                    } else {
                        rule.active_days.insert(day);
                    }
                    let _ = self.rules.save();
                }
            }
            AccountMessage::SetTimeWindow(id, start_str, end_str) => {
                if let Some(rule) = self.rules.account_rules.iter_mut().find(|r| r.id == id) {
                    let start = start_str.and_then(|s| NaiveTime::parse_from_str(&s, "%H:%M").ok());
                    let end = end_str.and_then(|s| NaiveTime::parse_from_str(&s, "%H:%M").ok());
                    rule.start_time = start;
                    rule.end_time = end;
                    let _ = self.rules.save();
                }
            }
            AccountMessage::SetTimeWindowExpanded(id, expanded) => {
                if expanded {
                    self.expanded_account_time_windows.insert(id);
                } else {
                    self.expanded_account_time_windows.remove(&id);
                }
            }
            AccountMessage::SetOutsideBehavior(id, behavior) => {
                if let Some(rule) = self.rules.account_rules.iter_mut().find(|r| r.id == id) {
                    rule.outside_behavior = behavior;
                    let _ = self.rules.save();
                }
            }
        }
        Task::none()
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

    fn update_type(&mut self, message: TypeMessage) -> Task<RuleEngineMessage> {
        match message {
            TypeMessage::Toggle(id, enabled) => {
                if let Some(rule) = self.rules.type_rules.iter_mut().find(|r| r.id == id) {
                    rule.enabled = enabled;
                }
                let _ = self.rules.save();
            }
            TypeMessage::Delete(id) => {
                self.rules.type_rules.retain(|r| r.id != id);
                let _ = self.rules.save();
            }
            TypeMessage::Duplicate(id) => {
                if let Some(rule) = self.rules.type_rules.iter().find(|r| r.id == id).cloned() {
                    let mut new_rule = rule;
                    new_rule.id = uuid::Uuid::new_v4().to_string();
                    self.rules.type_rules.push(new_rule);
                    let _ = self.rules.save();
                }
            }
            TypeMessage::ToggleGroup(group_name) => {
                if self.expanded_type_groups.contains(&group_name) {
                    self.expanded_type_groups.remove(&group_name);
                } else {
                    self.expanded_type_groups.insert(group_name);
                }
            }
            TypeMessage::FormTypeChanged(s) => {
                self.new_type_rule_type = s;
            }
            TypeMessage::FormAccountChanged(s) => {
                self.new_type_rule_account = if s == "Global" || s.trim().is_empty() {
                    None
                } else {
                    Some(s)
                };
            }
            TypeMessage::FormPriorityChanged(p) => {
                self.new_type_rule_priority = p;
            }
            TypeMessage::FormActionChanged(a) => {
                self.new_type_rule_action = a;
            }
            TypeMessage::Add => {
                let priority = self.new_type_rule_priority;
                let account = self.new_type_rule_account.clone();

                let mut rule = TypeRule::new(self.new_type_rule_type.label(), account, priority);
                rule.action = self.new_type_rule_action;

                self.rules.type_rules.push(rule);
                let _ = self.rules.save();

                // Reset form
                self.new_type_rule_account = None;
                self.new_type_rule_priority = 0;
                self.new_type_rule_action = RuleAction::Show;
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
            let handbook = self.view_handbook_modal();
            iced::widget::stack![base_layout, handbook]
                .width(Fill)
                .height(Fill)
                .into()
        } else {
            base_layout
        }
    }

    fn view_handbook_modal(&self) -> Element<'_, RuleEngineMessage> {
        let p = theme::palette();

        // Backdrop (clickable to close)
        let backdrop_btn = button(Space::new().width(Fill).height(Fill))
            .width(Fill)
            .height(Fill)
            .style(|_, _| button::Style {
                background: Some(iced::Background::Color(iced::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.6,
                })),
                ..Default::default()
            })
            .on_press(RuleEngineMessage::ToggleHandbook);

        // Handbook content
        let content = column![
            // Header
            row![
                text("Rule Engine Handbook")
                    .size(18)
                    .color(p.text_primary)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    }),
                Space::new().width(Fill),
                button(icons::icon_x(16.0, p.text_secondary, self.icon_theme))
                    .style(theme::ghost_button)
                    .padding(4)
                    .on_press(RuleEngineMessage::ToggleHandbook),
            ]
            .align_y(Alignment::Center),
            Space::new().height(16),
            // Core Principle
            text("Core Principle")
                .size(14)
                .color(p.accent)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
            text(
                "Notifications are SHOWN by default. \
Rules only exist to restrict, silence, hide, or elevate notifications."
            )
            .size(13)
            .color(p.text_secondary),
            Space::new().height(12),
            // Actions
            text("Actions").size(14).color(p.accent).font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }),
            text(
                "• Show - Visible in the in-app list and triggers a desktop notification. \
This is the default behavior when no rules apply."
            )
            .size(13)
            .color(p.text_secondary),
            text(
                "• Silent - Visible in the in-app list but does NOT trigger a desktop notification."
            )
            .size(13)
            .color(p.text_secondary),
            text(
                "• Hide - Completely hidden. The notification does NOT appear in the list \
and does NOT trigger a desktop notification."
            )
            .size(13)
            .color(p.text_secondary),
            text(
                "• Important - Always visible and always triggers a desktop notification. \
Important notifications bypass account rules, schedules, Hide, and Silent actions, \
and are shown across ALL configured accounts. Important notifications are pinned \
at the top of every notification list."
            )
            .size(13)
            .color(p.accent),
            Space::new().height(12),
            // Priority Value
            text("Priority Value (−100 to +100)")
                .size(14)
                .color(p.accent)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
            text(
                "The priority value ONLY affects the sort order of notifications \
within the in-app list. Higher values appear first."
            )
            .size(13)
            .color(p.text_secondary),
            text(
                "Priority does NOT affect desktop notifications and does NOT override \
Hide or Silent actions. Only the Important action can override suppression."
            )
            .size(13)
            .color(p.text_muted),
            Space::new().height(12),
            // Resolution Order
            text("Rule Resolution Order")
                .size(14)
                .color(p.accent)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                }),
            text(
                "1. Important always wins. \
If ANY matching rule marks a notification as Important, \
it is treated as Important regardless of other rules."
            )
            .size(13)
            .color(p.text_secondary),
            text("2. If no Important rule applies, the rule with the highest priority value wins.")
                .size(13)
                .color(p.text_secondary),
            text(
                "3. If priority values are equal, the most restrictive action wins \
(Hide > Silent > Show)."
            )
            .size(13)
            .color(p.text_secondary),
        ]
        .spacing(4)
        .padding(24)
        .width(450);

        let scrollable_content = scrollable(content)
            .style(theme::scrollbar)
            .height(Length::Shrink);

        let modal_card = container(scrollable_content)
            .style(theme::card)
            .max_width(500)
            .max_height(600); // Fixed max height to allow scrolling

        // Wrap modal in mouse_area to prevent clicks from bubbling to backdrop
        let modal_with_blocker =
            iced::widget::mouse_area(modal_card).on_press(RuleEngineMessage::NoOp);

        // Center the modal
        let centered = container(modal_with_blocker)
            .width(Fill)
            .height(Fill)
            .padding(40)
            .center_x(Fill)
            .center_y(Fill);

        // Stack backdrop + modal
        iced::widget::stack![backdrop_btn, centered]
            .width(Fill)
            .height(Fill)
            .into()
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
                let content = tabs::view_overview_tab(&self.rules, t, &self.explain_test_type);
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
                let content = tabs::view_account_rules_tab(
                    &self.rules,
                    t,
                    &self.selected_account_id,
                    &self.expanded_account_time_windows,
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
                let content = tabs::view_org_rules_tab(&self.rules, t);
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
                let content = tabs::view_type_rules_tab(
                    &self.rules,
                    t,
                    &tabs::TypeRuleFormState {
                        notification_type: self.new_type_rule_type,
                        account: self.new_type_rule_account.clone(),
                        priority: self.new_type_rule_priority,
                        action: self.new_type_rule_action,
                    },
                    &self.accounts,
                    &self.expanded_type_groups,
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
