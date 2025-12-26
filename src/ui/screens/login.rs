//! Login screen - Personal Access Token entry.

use iced::widget::{Space, button, column, container, text, text_input};
use iced::{Alignment, Element, Fill, Length, Task};

use crate::github::{GitHubClient, UserInfo, auth};
use crate::ui::theme;

#[derive(Debug, Clone, Default)]
pub struct LoginScreen {
    token_input: String,
    is_loading: bool,
    error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum LoginMessage {
    TokenInputChanged(String),
    Submit,
    LoginSuccess(GitHubClient, UserInfo),
    LoginFailed(String),
    OpenTokenUrl,
    TokenUrlOpened,
}

impl LoginScreen {
    pub fn new() -> Self {
        Self::default()
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

                self.is_loading = true;
                self.error_message = None;

                let token = self.token_input.clone();
                Task::perform(
                    async move { auth::authenticate(&token).await },
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
        }
    }

    pub fn view(&self) -> Element<'_, LoginMessage> {
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
}
