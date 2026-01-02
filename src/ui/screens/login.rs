//! Login screen - Personal Access Token entry.

use iced::widget::{Space, button, column, container, text, text_input, toggler};
use iced::{Alignment, Element, Fill, Length, Task};

use crate::github::{GitHubClient, UserInfo, auth, proxy_keyring};
use crate::settings::AppSettings;
use crate::ui::theme;

#[derive(Debug, Clone, Default)]
pub struct LoginScreen {
    token_input: String,
    is_loading: bool,
    pub error_message: Option<String>,
    showing_proxy_settings: bool,
    proxy_enabled: bool,
    proxy_url: String,
    proxy_username: String,
    proxy_password: String,
}

#[derive(Debug, Clone)]
pub enum LoginMessage {
    TokenInputChanged(String),
    Submit,
    LoginSuccess(GitHubClient, UserInfo),
    LoginFailed(String),
    OpenTokenUrl,
    TokenUrlOpened,
    ToggleProxySettings,
    ProxyEnabledChanged(bool),
    ProxyUrlChanged(String),
    ProxyUsernameChanged(String),
    ProxyPasswordChanged(String),
    SubmitProxySettings,
}

impl LoginScreen {
    pub fn new() -> Self {
        let settings = AppSettings::load();
        let proxy = &settings.proxy;

        // Load credentials from keyring if they exist
        let (proxy_username, proxy_password) = if proxy.has_credentials
            && let Ok(Some((user, pass))) = proxy_keyring::load_proxy_credentials(&proxy.url)
        {
            (user, pass)
        } else {
            (String::new(), String::new())
        };

        Self {
            token_input: String::new(),
            is_loading: false,
            error_message: None,
            showing_proxy_settings: false,
            proxy_enabled: proxy.enabled,
            proxy_url: proxy.url.clone(),
            proxy_username,
            proxy_password,
        }
    }

    /// Build ProxySettings from current UI state
    fn build_proxy_settings(&self) -> AppSettings {
        let mut settings = AppSettings::load();
        settings.proxy.enabled = self.proxy_enabled;
        settings.proxy.url = self.proxy_url.clone();

        // Determine if we have credentials to store
        settings.proxy.has_credentials =
            !self.proxy_username.is_empty() || !self.proxy_password.is_empty();

        settings
    }

    pub fn update(&mut self, message: LoginMessage) -> Task<LoginMessage> {
        match message {
            LoginMessage::TokenInputChanged(value) => {
                self.token_input = value;
                self.error_message = None;
                Task::none()
            }
            LoginMessage::Submit => {
                if self.token_input.trim().is_empty() {
                    self.error_message = Some("Please enter your token".to_string());
                    return Task::none();
                }

                if let Err(e) = auth::validate_token_format(&self.token_input) {
                    self.error_message = Some(e.to_string());
                    return Task::none();
                }

                // Build proxy settings from current state (read only, no saving)
                let proxy_settings = self.build_proxy_settings();

                self.is_loading = true;
                self.error_message = None;

                let token = self.token_input.clone();
                Task::perform(
                    async move { auth::authenticate(&token, Some(&proxy_settings.proxy)).await },
                    |result| match result {
                        Ok((client, user)) => LoginMessage::LoginSuccess(client, user),
                        Err(e) => LoginMessage::LoginFailed(e.to_string()),
                    },
                )
            }
            LoginMessage::LoginSuccess(_, _) => {
                // Handled by parent
                self.is_loading = false;
                Task::none()
            }
            LoginMessage::LoginFailed(error) => {
                self.is_loading = false;
                self.error_message = Some(error);
                Task::none()
            }
            LoginMessage::OpenTokenUrl => {
                let date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
                let description = format!("GitTop (Created on {})", date);

                let scopes = "notifications,repo";

                let mut url = reqwest::Url::parse("https://github.com/settings/tokens/new")
                    .expect("Base URL is valid");
                url.query_pairs_mut()
                    .append_pair("scopes", scopes)
                    .append_pair("description", &description);

                let url_string = url.to_string();

                Task::perform(
                    async move {
                        let _ = open::that(url_string);
                    },
                    |_| LoginMessage::TokenUrlOpened,
                )
            }
            LoginMessage::TokenUrlOpened => Task::none(),
            LoginMessage::ToggleProxySettings => {
                self.showing_proxy_settings = !self.showing_proxy_settings;
                Task::none()
            }
            LoginMessage::ProxyEnabledChanged(enabled) => {
                self.proxy_enabled = enabled;
                Task::none()
            }
            LoginMessage::ProxyUrlChanged(url) => {
                self.proxy_url = url;
                Task::none()
            }
            LoginMessage::ProxyUsernameChanged(username) => {
                self.proxy_username = username;
                Task::none()
            }
            LoginMessage::ProxyPasswordChanged(password) => {
                self.proxy_password = password;
                Task::none()
            }
            LoginMessage::SubmitProxySettings => {
                // Save proxy settings to disk and credentials to keyring
                self.save_proxy_settings();
                // Go back to login screen
                self.showing_proxy_settings = false;
                Task::none()
            }
        }
    }

    fn save_proxy_settings(&self) {
        let settings = self.build_proxy_settings();

        // Save or delete credentials from keyring based on whether they're empty
        if self.proxy_username.is_empty() && self.proxy_password.is_empty() {
            // Delete credentials if both are empty
            let _ = proxy_keyring::delete_proxy_credentials(&self.proxy_url);
        } else {
            // Save credentials if at least one is not empty
            let _ = proxy_keyring::save_proxy_credentials(
                &self.proxy_url,
                &self.proxy_username,
                &self.proxy_password,
            );
        }

        // Save settings to disk
        let _ = settings.save();
    }

    pub fn view(&self) -> Element<'_, LoginMessage> {
        if self.showing_proxy_settings {
            self.proxy_settings_view()
        } else {
            self.login_view()
        }
    }

    fn login_view(&self) -> Element<'_, LoginMessage> {
        let p = theme::palette();

        let logo = text("GitTop").size(32).color(p.text_primary);

        let tagline = text("Runs lighter than your IDE's status bar.")
            .size(14)
            .style(theme::secondary_text);

        let token_label = text("GitHub Personal Access Token")
            .size(12)
            .style(theme::secondary_text);

        let token_input = text_input("ghp_xxxxxxxxxxxx", &self.token_input)
            .on_input(LoginMessage::TokenInputChanged)
            .on_submit(LoginMessage::Submit)
            .padding(12)
            .size(14)
            .style(theme::text_input_style)
            .width(Fill);

        let submit_button = if self.is_loading {
            button(
                text("Authenticating...")
                    .size(14)
                    .width(Fill)
                    .align_x(Alignment::Center),
            )
            .style(theme::primary_button)
            .width(Fill)
            .padding(12)
        } else {
            button(
                text("Sign In")
                    .size(14)
                    .width(Fill)
                    .align_x(Alignment::Center),
            )
            .style(theme::primary_button)
            .on_press(LoginMessage::Submit)
            .width(Fill)
            .padding(12)
        };

        let error_text: Element<'_, LoginMessage> = if let Some(ref error) = self.error_message {
            text(error).size(12).color(p.accent_danger).into()
        } else {
            Space::new().width(0).height(0).into()
        };

        let help_text = column![
            button(text("Generate New Token").size(12))
                .style(theme::ghost_button)
                .on_press(LoginMessage::OpenTokenUrl)
                .padding(4),
            text("Required scopes: notifications, repo")
                .size(11)
                .style(theme::muted_text),
            button(text("Proxy Settings").size(12))
                .style(theme::ghost_button)
                .on_press(LoginMessage::ToggleProxySettings)
                .padding(4),
        ]
        .spacing(4)
        .align_x(Alignment::Center);

        let form = column![
            token_label,
            Space::new().height(8),
            token_input,
            Space::new().height(8),
            error_text,
            Space::new().height(16),
            submit_button,
            Space::new().height(24),
            help_text,
        ]
        .align_x(Alignment::Center)
        .width(Length::Fixed(320.0));

        let content = column![
            logo,
            Space::new().height(8),
            tagline,
            Space::new().height(48),
            form,
        ]
        .align_x(Alignment::Center);

        container(content)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .padding(32)
            .style(theme::app_container)
            .into()
    }

    fn proxy_settings_view(&self) -> Element<'_, LoginMessage> {
        let p = theme::palette();

        let title = text("Network Proxy Settings")
            .size(24)
            .color(p.text_primary);

        let subtitle = text("Configure proxy settings for GitHub API requests")
            .size(13)
            .style(theme::secondary_text);

        let proxy_switch = toggler(self.proxy_enabled)
            .on_toggle(LoginMessage::ProxyEnabledChanged)
            .size(24);

        let url_label = text("Proxy URL").size(12).style(theme::secondary_text);

        let url_input = text_input("http://proxy.company.com:8080", &self.proxy_url)
            .on_input(LoginMessage::ProxyUrlChanged)
            .padding(12)
            .size(14)
            .style(theme::text_input_style)
            .width(Fill);

        let username_label = text("Username (optional)")
            .size(12)
            .style(theme::secondary_text);

        let username_input = text_input("", &self.proxy_username)
            .on_input(LoginMessage::ProxyUsernameChanged)
            .padding(12)
            .size(14)
            .style(theme::text_input_style)
            .width(Fill);

        let password_label = text("Password (optional)")
            .size(12)
            .style(theme::secondary_text);

        let password_input = text_input("", &self.proxy_password)
            .secure(true)
            .on_input(LoginMessage::ProxyPasswordChanged)
            .padding(12)
            .size(14)
            .style(theme::text_input_style)
            .width(Fill);

        let settings_form = column![
            url_label,
            Space::new().height(4),
            url_input,
            Space::new().height(16),
            username_label,
            Space::new().height(4),
            username_input,
            Space::new().height(16),
            password_label,
            Space::new().height(4),
            password_input,
        ]
        .align_x(Alignment::Center)
        .width(Length::Fixed(320.0));

        let back_button = button(
            text("Save and Back")
                .size(14)
                .width(Fill)
                .align_x(Alignment::Center),
        )
        .style(theme::primary_button)
        .on_press(LoginMessage::SubmitProxySettings)
        .width(Fill)
        .padding(12);

        let content = column![
            title,
            Space::new().height(4),
            subtitle,
            Space::new().height(32),
            proxy_switch,
            Space::new().height(24),
            settings_form,
            Space::new().height(32),
            back_button,
        ]
        .align_x(Alignment::Center)
        .width(Length::Fixed(320.0));

        container(content)
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .padding(32)
            .style(theme::app_container)
            .into()
    }
}
