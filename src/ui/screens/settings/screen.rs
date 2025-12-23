//! Settings screen - main screen with tab navigation.

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Fill, Length, Task};

use crate::github::{AccountKeyring, GitHubClient};
use crate::settings::{AppSettings, IconTheme};
use crate::ui::{icons, theme};

use super::messages::{SettingsMessage, SettingsTab};
use super::tabs::{accounts, general, power_mode};

/// Settings screen state.
#[derive(Debug, Clone)]
pub struct SettingsScreen {
    pub settings: AppSettings,
    pub selected_tab: SettingsTab,
    pub accounts_state: accounts::AccountsTabState,
}

impl SettingsScreen {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            settings,
            selected_tab: SettingsTab::default(),
            accounts_state: accounts::AccountsTabState::default(),
        }
    }

    pub fn update(&mut self, message: SettingsMessage) -> Task<SettingsMessage> {
        match message {
            SettingsMessage::Back => Task::none(),
            SettingsMessage::SelectTab(tab) => {
                self.selected_tab = tab;
                self.accounts_state.error_message = None;
                self.accounts_state.success_message = None;
                Task::none()
            }
            SettingsMessage::ChangeTheme(new_theme) => {
                self.settings.theme = new_theme;
                theme::set_theme(new_theme);
                let _ = self.settings.save();
                crate::platform::trim_memory();
                Task::none()
            }
            SettingsMessage::ToggleIconTheme(use_svg) => {
                self.settings.icon_theme = if use_svg {
                    IconTheme::Svg
                } else {
                    IconTheme::Emoji
                };
                let _ = self.settings.save();
                crate::platform::trim_memory();
                Task::none()
            }
            SettingsMessage::ToggleMinimizeToTray(enabled) => {
                self.settings.minimize_to_tray = enabled;
                let _ = self.settings.save();
                Task::none()
            }
            SettingsMessage::RemoveAccount(username) => {
                self.settings.remove_account(&username);
                let _ = self.settings.save();
                let _ = AccountKeyring::delete_token(&username);
                Task::none()
            }
            SettingsMessage::SetNotificationFontScale(scale) => {
                let clamped = scale.clamp(0.8, 1.5);
                self.settings.notification_font_scale = clamped;
                theme::set_notification_font_scale(clamped);
                let _ = self.settings.save();
                crate::platform::trim_memory();
                Task::none()
            }
            SettingsMessage::SetSidebarFontScale(scale) => {
                let clamped = scale.clamp(0.8, 1.5);
                self.settings.sidebar_font_scale = clamped;
                theme::set_sidebar_font_scale(clamped);
                let _ = self.settings.save();
                crate::platform::trim_memory();
                Task::none()
            }
            SettingsMessage::SetSidebarWidth(width) => {
                let clamped = width.clamp(180.0, 400.0);
                self.settings.sidebar_width = clamped;
                let _ = self.settings.save();
                crate::platform::trim_memory();
                Task::none()
            }
            SettingsMessage::TogglePowerMode(enabled) => {
                self.settings.power_mode = enabled;
                let _ = self.settings.save();
                Task::none()
            }
            SettingsMessage::OpenRuleEngine => Task::none(),
            SettingsMessage::TokenInputChanged(token) => {
                self.accounts_state.token_input = token;
                self.accounts_state.error_message = None;
                Task::none()
            }
            SettingsMessage::SubmitToken => {
                let token = self.accounts_state.token_input.clone();
                if token.is_empty() {
                    self.accounts_state.error_message = Some("Token cannot be empty".to_string());
                    return Task::none();
                }

                if !token.starts_with("ghp_") && !token.starts_with("github_pat_") {
                    self.accounts_state.error_message =
                        Some("Token must start with 'ghp_' or 'github_pat_'".to_string());
                    return Task::none();
                }

                self.accounts_state.is_validating = true;
                self.accounts_state.error_message = None;

                Task::perform(
                    async move {
                        match GitHubClient::new(&token) {
                            Ok(client) => match client.get_authenticated_user().await {
                                Ok(user) => {
                                    if let Err(e) = AccountKeyring::save_token(&user.login, &token)
                                    {
                                        return Err(format!("Failed to save token: {}", e));
                                    }
                                    Ok(user.login)
                                }
                                Err(e) => Err(format!("Validation failed: {}", e)),
                            },
                            Err(e) => Err(format!("Invalid token: {}", e)),
                        }
                    },
                    SettingsMessage::TokenValidated,
                )
            }
            SettingsMessage::TokenValidated(result) => {
                self.accounts_state.is_validating = false;
                match result {
                    Ok(username) => {
                        self.settings.set_active_account(&username);
                        let _ = self.settings.save();
                        self.accounts_state.token_input.clear();
                        self.accounts_state.success_message =
                            Some(format!("Account '{}' added successfully!", username));
                    }
                    Err(error) => {
                        self.accounts_state.error_message = Some(error);
                    }
                }
                Task::none()
            }
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

        let content = match self.selected_tab {
            SettingsTab::PowerMode => power_mode::view(&self.settings),
            SettingsTab::General => general::view(&self.settings),
            SettingsTab::Accounts => accounts::view(&self.settings, &self.accounts_state),
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
