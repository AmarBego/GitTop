//! Settings screen - main screen with tab navigation.

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length, Task};

use crate::settings::AppSettings;
use crate::ui::context::AppContext;
use crate::ui::effects::{AppEffect, NavigateTo, SessionEffect};
use crate::ui::features::account_management::AccountMessage;
use crate::ui::features::power_mode::PowerModeMessage;
use crate::ui::features::{account_management, general_settings, network_proxy, power_mode};
use crate::ui::{icons, theme};

use super::messages::{SettingsMessage, SettingsTab};

/// Settings screen state.
#[derive(Debug, Clone)]
pub struct SettingsScreen {
    pub settings: AppSettings,
    pub selected_tab: SettingsTab,
    pub accounts: account_management::AccountManagementState,
    pub proxy: network_proxy::NetworkProxyState,
    pub general: general_settings::GeneralSettingsState,
    pub power_mode: power_mode::PowerModeState,
}

impl SettingsScreen {
    pub fn new(settings: AppSettings) -> Self {
        let proxy = network_proxy::NetworkProxyState::new(&settings);
        let general = general_settings::GeneralSettingsState::new();
        let power_mode = power_mode::PowerModeState::new();
        let accounts = account_management::AccountManagementState::default();

        Self {
            settings,
            selected_tab: SettingsTab::default(),
            accounts,
            proxy,
            general,
            power_mode,
        }
    }

    pub fn update(&mut self, message: SettingsMessage) -> Task<SettingsMessage> {
        match message {
            SettingsMessage::Back => Task::none(),
            SettingsMessage::SelectTab(tab) => {
                self.selected_tab = tab;
                // Reset states if needed when switching tabs
                self.accounts.status = account_management::state::SubmissionStatus::Idle;
                Task::none()
            }
            SettingsMessage::OpenRuleEngine => Task::none(),
            SettingsMessage::Account(msg) => {
                account_management::update(&mut self.accounts, msg, &mut self.settings)
                    .map(SettingsMessage::Account)
            }
            SettingsMessage::General(msg) => {
                general_settings::update(&mut self.general, msg, &mut self.settings)
                    .map(SettingsMessage::General)
            }
            SettingsMessage::Proxy(msg) => {
                network_proxy::update(&mut self.proxy, msg, &mut self.settings)
                    .map(SettingsMessage::Proxy)
            }
            SettingsMessage::PowerMode(msg) => {
                // Intercept OpenRuleEngine from PowerMode if needed, or let it propagate via update return
                if let power_mode::PowerModeMessage::OpenRuleEngine = msg {
                    return Task::done(SettingsMessage::OpenRuleEngine);
                }

                power_mode::update(&mut self.power_mode, msg, &mut self.settings)
                    .map(SettingsMessage::PowerMode)
            }
        }
    }

    /// Update with effect pattern - returns task and any app-level effect.
    pub fn update_with_effect(
        &mut self,
        message: SettingsMessage,
        _ctx: &mut AppContext,
    ) -> (Task<SettingsMessage>, AppEffect) {
        match &message {
            // Navigation becomes effects
            SettingsMessage::Back => (Task::none(), AppEffect::Navigate(NavigateTo::Back)),
            SettingsMessage::OpenRuleEngine => (
                Task::none(),
                AppEffect::Navigate(NavigateTo::RuleEngine {
                    from_settings: true,
                }),
            ),

            // Account operations that affect session
            SettingsMessage::Account(AccountMessage::RemoveAccount(username)) => {
                let username = username.clone();
                (
                    Task::none(),
                    AppEffect::Session(SessionEffect::RemoveAccount(username)),
                )
            }
            SettingsMessage::Account(AccountMessage::TokenValidated(Ok(_username))) => {
                // For token validation, we need to spawn the async restore task
                // We handle this in the screen's normal update, not as an effect
                let task = self.update(message);
                (task, AppEffect::None)
            }

            // Power mode toggle with window resize - handled by app.rs effect executor
            SettingsMessage::PowerMode(PowerModeMessage::Toggle(_)) => {
                let task = self.update(message);
                (task, AppEffect::None)
            }

            // Other messages handled normally
            _ => (self.update(message), AppEffect::None),
        }
    }

    // ========================================================================
    // Main Layout
    // ========================================================================

    pub fn view(&self) -> Element<'_, SettingsMessage> {
        let header = self.view_header();
        let sidebar = self.view_sidebar();
        let content = self.view_content();

        let main_area = row![sidebar, content].height(Fill);

        column![header, main_area]
            .spacing(0)
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn view_header(&self) -> Element<'_, SettingsMessage> {
        let p = theme::palette();
        let icon_theme = self.settings.icon_theme;

        let back_btn = button(
            row![
                icons::icon_chevron_left(16.0, p.text_secondary, icon_theme),
                Space::new().width(4),
                text("Back").size(13).color(p.text_secondary),
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::ghost_button)
        .padding([6, 10])
        .on_press(SettingsMessage::Back);

        let title = text("Settings").size(18).color(p.text_primary);

        let header_row = row![
            back_btn,
            Space::new().width(Fill),
            title,
            Space::new().width(Fill),
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

    fn view_sidebar(&self) -> Element<'_, SettingsMessage> {
        let icon_theme = self.settings.icon_theme;

        let nav = column![
            self.nav_item(
                "Power Mode",
                SettingsTab::PowerMode,
                icons::icon_power(16.0, self.icon_color(SettingsTab::PowerMode), icon_theme)
            ),
            self.nav_item(
                "General",
                SettingsTab::General,
                icons::icon_settings(16.0, self.icon_color(SettingsTab::General), icon_theme)
            ),
            self.nav_item(
                "Accounts",
                SettingsTab::Accounts,
                icons::icon_user(16.0, self.icon_color(SettingsTab::Accounts), icon_theme)
            ),
            self.nav_item(
                "Network Proxy",
                SettingsTab::NetworkProxy,
                icons::icon_wifi(16.0, self.icon_color(SettingsTab::NetworkProxy), icon_theme)
            ),
        ]
        .spacing(4)
        .padding([16, 8]);

        container(nav)
            .width(Length::Fixed(self.settings.sidebar_width))
            .height(Fill)
            .style(theme::sidebar)
            .into()
    }

    fn icon_color(&self, tab: SettingsTab) -> iced::Color {
        let p = theme::palette();
        if self.selected_tab == tab {
            p.accent
        } else {
            p.text_secondary
        }
    }

    fn nav_item<'a>(
        &self,
        label: &'static str,
        tab: SettingsTab,
        icon: Element<'a, SettingsMessage>,
    ) -> Element<'a, SettingsMessage> {
        let p = theme::palette();
        let selected = self.selected_tab == tab;
        let color = if selected { p.accent } else { p.text_primary };

        let content = row![
            icon,
            Space::new().width(10),
            text(label).size(theme::sidebar_scaled(13.0)).color(color),
        ]
        .align_y(Alignment::Center)
        .padding([10, 12]);

        button(content)
            .style(move |theme, status| (theme::sidebar_button(selected))(theme, status))
            .on_press(SettingsMessage::SelectTab(tab))
            .width(Fill)
            .into()
    }

    // ========================================================================
    // Tab Content
    // ========================================================================

    fn view_content(&self) -> Element<'_, SettingsMessage> {
        let p = theme::palette();

        // Each feature view returns its own Message type.
        // We map them to SettingsMessage wrapper using .map()
        let content: Element<'_, SettingsMessage> = match self.selected_tab {
            SettingsTab::PowerMode => {
                power_mode::view(&self.settings).map(SettingsMessage::PowerMode)
            }
            SettingsTab::General => {
                general_settings::view(&self.settings, &self.general).map(SettingsMessage::General)
            }
            SettingsTab::Accounts => account_management::view(&self.accounts, &self.settings)
                .map(SettingsMessage::Account),
            SettingsTab::NetworkProxy => {
                network_proxy::view(&self.proxy, &self.settings).map(SettingsMessage::Proxy)
            }
        };

        let scrollable_content = scrollable(content)
            .width(Fill)
            .height(Fill)
            .style(theme::scrollbar);

        container(scrollable_content)
            .width(Fill)
            .height(Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_base)),
                ..Default::default()
            })
            .into()
    }
}
