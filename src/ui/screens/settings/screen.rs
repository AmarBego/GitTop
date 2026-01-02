//! Settings screen - main screen with tab navigation.

use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Fill, Length, Task};

use crate::github::{GitHubClient, keyring, proxy_keyring};
use crate::settings::{AppSettings, IconTheme};
use crate::ui::{icons, theme};

use super::messages::{SettingsMessage, SettingsTab};
use super::tabs::{accounts, general, network_proxy, power_mode};

/// Settings screen state.
#[derive(Debug, Clone)]
pub struct SettingsScreen {
    pub settings: AppSettings,
    pub selected_tab: SettingsTab,
    pub accounts_state: accounts::AccountsTabState,
    pub proxy_enabled: bool,
    pub proxy_url: String,
    pub proxy_username: String,
    pub proxy_password: String,
    pub proxy_creds_dirty: bool,
    pub proxy_needs_rebuild: bool,
}

impl SettingsScreen {
    pub fn new(settings: AppSettings) -> Self {
        // Load proxy state from settings
        let proxy_enabled = settings.proxy.enabled;
        let proxy_url = settings.proxy.url.clone();

        // Load proxy credentials from keyring if they exist
        let (proxy_username, proxy_password) = if settings.proxy.has_credentials
            && let Ok(Some((user, pass))) =
                proxy_keyring::load_proxy_credentials(&settings.proxy.url)
        {
            (user, pass)
        } else {
            (String::new(), String::new())
        };

        Self {
            settings,
            selected_tab: SettingsTab::default(),
            accounts_state: accounts::AccountsTabState::default(),
            proxy_enabled,
            proxy_url,
            proxy_username,
            proxy_password,
            proxy_creds_dirty: false,
            proxy_needs_rebuild: false,
        }
    }

    pub fn update(&mut self, message: SettingsMessage) -> Task<SettingsMessage> {
        match message {
            SettingsMessage::Back => Task::none(),
            SettingsMessage::SelectTab(tab) => {
                self.selected_tab = tab;
                self.accounts_state.status = accounts::SubmissionStatus::Idle;
                Task::none()
            }
            SettingsMessage::ChangeTheme(new_theme) => {
                self.settings.theme = new_theme;
                theme::set_theme(new_theme);
                self.persist_settings();
                Task::none()
            }
            SettingsMessage::ToggleIconTheme(use_svg) => {
                self.settings.icon_theme = if use_svg {
                    IconTheme::Svg
                } else {
                    IconTheme::Emoji
                };
                self.persist_settings();
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
                let _ = keyring::delete_token(&username);
                Task::none()
            }
            SettingsMessage::SetNotificationFontScale(scale) => {
                let clamped = scale.clamp(0.8, 1.5);
                self.settings.notification_font_scale = clamped;
                theme::set_notification_font_scale(clamped);
                self.persist_settings();
                Task::none()
            }
            SettingsMessage::SetSidebarFontScale(scale) => {
                let clamped = scale.clamp(0.8, 1.5);
                self.settings.sidebar_font_scale = clamped;
                theme::set_sidebar_font_scale(clamped);
                self.persist_settings();
                Task::none()
            }
            SettingsMessage::SetSidebarWidth(width) => {
                let clamped = width.clamp(180.0, 400.0);
                self.settings.sidebar_width = clamped;
                self.persist_settings();
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
                self.accounts_state.status = accounts::SubmissionStatus::Idle;
                Task::none()
            }
            SettingsMessage::SubmitToken => {
                let token = self.accounts_state.token_input.clone();
                if let Err(e) = crate::github::auth::validate_token_format(&token) {
                    self.accounts_state.status = accounts::SubmissionStatus::Error(e.to_string());
                    return Task::none();
                }

                self.accounts_state.status = accounts::SubmissionStatus::Validating;

                Task::perform(
                    async move {
                        let client = GitHubClient::new(&token)
                            .map_err(|e| format!("Invalid token: {}", e))?;

                        let user = client
                            .get_authenticated_user()
                            .await
                            .map_err(|e| format!("Validation failed: {}", e))?;

                        keyring::save_token(&user.login, &token)
                            .map_err(|e| format!("Failed to save token: {}", e))?;

                        Ok(user.login)
                    },
                    SettingsMessage::TokenValidated,
                )
            }
            SettingsMessage::TokenValidated(result) => {
                match result {
                    Ok(username) => {
                        self.settings.set_active_account(&username);
                        let _ = self.settings.save();
                        self.accounts_state.token_input.clear();
                        self.accounts_state.status = accounts::SubmissionStatus::Success(format!(
                            "Account '{}' added successfully!",
                            username
                        ));
                    }
                    Err(error) => {
                        self.accounts_state.status = accounts::SubmissionStatus::Error(error);
                    }
                }
                Task::none()
            }
            SettingsMessage::ToggleProxyEnabled(enabled) => {
                self.proxy_enabled = enabled;
                Task::none()
            }
            SettingsMessage::ProxyUrlChanged(url) => {
                self.proxy_url = url;
                Task::none()
            }
            SettingsMessage::ProxyUsernameChanged(username) => {
                self.proxy_username = username;
                self.proxy_creds_dirty = true;
                Task::none()
            }
            SettingsMessage::ProxyPasswordChanged(password) => {
                self.proxy_password = password;
                self.proxy_creds_dirty = true;
                Task::none()
            }
            SettingsMessage::SaveProxySettings => {
                self.update_proxy_credentials();
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

        let content = match self.selected_tab {
            SettingsTab::PowerMode => power_mode::view(&self.settings),
            SettingsTab::General => general::view(&self.settings),
            SettingsTab::Accounts => accounts::view(&self.settings, &self.accounts_state),
            SettingsTab::NetworkProxy => network_proxy::view(self),
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

    fn persist_settings(&mut self) {
        let _ = self.settings.save();
        crate::platform::trim_memory();
    }

    /// Check if proxy settings have unsaved changes
    pub fn has_unsaved_proxy_changes(&self) -> bool {
        let enabled_changed = self.proxy_enabled != self.settings.proxy.enabled;
        let url_changed = self.proxy_url != self.settings.proxy.url;
        let new_has_creds = !self.proxy_username.is_empty() || !self.proxy_password.is_empty();
        let creds_status_changed = new_has_creds != self.settings.proxy.has_credentials;

        enabled_changed || url_changed || creds_status_changed || self.proxy_creds_dirty
    }

    fn update_proxy_credentials(&mut self) {
        let old_url = self.settings.proxy.url.clone();
        let new_url = self.proxy_url.clone();

        // Sync all proxy settings from temp fields
        self.settings.proxy.enabled = self.proxy_enabled;
        self.settings.proxy.url = new_url.clone();

        // Update has_credentials flag
        self.settings.proxy.has_credentials =
            !self.proxy_username.is_empty() || !self.proxy_password.is_empty();

        // Case 1: URL changed - handle both old and new URLs
        if old_url != new_url {
            eprintln!(
                "[PROXY] URL changed: '{}' -> '{}' (enabled: {})",
                old_url, new_url, self.proxy_enabled
            );

            // Delete credentials for old URL to prevent orphaned data
            if !old_url.is_empty() {
                eprintln!("[PROXY] Deleting credentials for old proxy URL");
                let _ = proxy_keyring::delete_proxy_credentials(&old_url);
            }

            // Save credentials for new URL if provided
            if !self.proxy_username.is_empty() && !self.proxy_password.is_empty() {
                eprintln!("[PROXY] Saving credentials for new proxy URL");
                let username = self.proxy_username.as_str();
                let password = self.proxy_password.as_str();
                let _ = proxy_keyring::save_proxy_credentials(&new_url, username, password);
            }
        }
        // Case 2: URL unchanged - only handle credential changes
        else if self.proxy_username.is_empty() && self.proxy_password.is_empty() {
            eprintln!("[PROXY] Credentials cleared, deleting from keyring");
            let _ = proxy_keyring::delete_proxy_credentials(&old_url);
        } else {
            // Check if credentials actually changed
            let should_save = if let Ok(Some((saved_username, saved_password))) =
                proxy_keyring::load_proxy_credentials(&old_url)
            {
                saved_username != self.proxy_username || saved_password != self.proxy_password
            } else {
                // No existing credentials, so save the new ones
                true
            };

            if should_save {
                eprintln!("[PROXY] Credentials changed, saving to keyring");
                let username = self.proxy_username.as_str();
                let password = self.proxy_password.as_str();
                let _ = proxy_keyring::save_proxy_credentials(&new_url, username, password);
            } else {
                eprintln!("[PROXY] Credentials unchanged, skipping keyring write");
            }
        }

        eprintln!(
            "[PROXY] Settings saved: enabled={}, url='{}', has_credentials={}",
            self.settings.proxy.enabled,
            self.settings.proxy.url,
            self.settings.proxy.has_credentials
        );

        // Signal that clients need rebuild when leaving settings
        self.proxy_needs_rebuild = true;
        // Reset dirty flag since we just saved
        self.proxy_creds_dirty = false;

        // Persist settings
        self.persist_settings();
    }
}
