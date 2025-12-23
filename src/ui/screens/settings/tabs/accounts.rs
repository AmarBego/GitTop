//! Accounts tab - GitHub account management.

use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Fill};

use crate::settings::{AppSettings, StoredAccount};
use crate::ui::{icons, theme};

use super::super::components::{setting_card, tab_title};
use super::super::messages::SettingsMessage;

/// State for the accounts tab (token input, validation status).
#[derive(Debug, Clone, Default)]
pub struct AccountsTabState {
    pub token_input: String,
    pub is_validating: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
}

/// Render the accounts tab content.
pub fn view<'a>(
    settings: &'a AppSettings,
    state: &'a AccountsTabState,
) -> Element<'a, SettingsMessage> {
    let p = theme::palette();

    column![
        tab_title("Accounts"),
        text("Manage your GitHub accounts.")
            .size(12)
            .color(p.text_secondary),
        Space::new().height(16),
        view_add_account_section(settings, state),
        Space::new().height(16),
        view_accounts_list(settings),
    ]
    .spacing(4)
    .padding(24)
    .width(Fill)
    .into()
}

fn view_add_account_section<'a>(
    settings: &'a AppSettings,
    state: &'a AccountsTabState,
) -> Element<'a, SettingsMessage> {
    let p = theme::palette();
    let icon_theme = settings.icon_theme;

    let mut content = column![
        row![
            icons::icon_plus(14.0, p.accent, icon_theme),
            Space::new().width(8),
            text("Add Account").size(14).color(p.text_primary),
        ]
        .align_y(Alignment::Center),
        Space::new().height(8),
        text("Enter a GitHub Personal Access Token with 'notifications' scope.")
            .size(11)
            .color(p.text_secondary),
        Space::new().height(12),
        row![
            text_input("ghp_xxxxxxxxxxxx", &state.token_input)
                .on_input(SettingsMessage::TokenInputChanged)
                .padding([8, 12])
                .size(13)
                .width(Fill)
                .style(theme::text_input_style),
            Space::new().width(8),
            button(if state.is_validating {
                text("Validating...").size(13).color(iced::Color::WHITE)
            } else {
                text("Add").size(13).color(iced::Color::WHITE)
            })
            .style(theme::primary_button)
            .padding([8, 16])
            .on_press_maybe(if state.is_validating || state.token_input.is_empty() {
                None
            } else {
                Some(SettingsMessage::SubmitToken)
            }),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    // Show error or success message
    if let Some(ref error) = state.error_message {
        content = content.push(Space::new().height(8));
        content = content.push(text(error).size(12).color(p.accent_danger));
    }
    if let Some(ref success) = state.success_message {
        content = content.push(Space::new().height(8));
        content = content.push(text(success).size(12).color(p.accent_success));
    }

    setting_card(content)
}

fn view_accounts_list(settings: &AppSettings) -> Element<'static, SettingsMessage> {
    let p = theme::palette();

    if settings.accounts.is_empty() {
        return container(text("No accounts added yet").size(12).color(p.text_muted))
            .padding(14)
            .into();
    }

    let mut col = column![
        text("Connected Accounts").size(13).color(p.text_secondary),
        Space::new().height(8),
    ]
    .spacing(8);

    for account in &settings.accounts {
        col = col.push(view_account_item(account, settings));
    }

    col.into()
}

fn view_account_item(
    account: &StoredAccount,
    settings: &AppSettings,
) -> Element<'static, SettingsMessage> {
    let p = theme::palette();
    let icon_theme = settings.icon_theme;
    let username = account.username.clone();
    let username_for_button = account.username.clone();

    container(
        row![
            icons::icon_user(14.0, p.text_secondary, icon_theme),
            Space::new().width(8),
            text(username).size(13).color(p.text_primary),
            Space::new().width(8),
            Space::new().width(Fill),
            button(icons::icon_trash(14.0, p.text_muted, icon_theme))
                .style(theme::ghost_button)
                .padding(6)
                .on_press(SettingsMessage::RemoveAccount(username_for_button)),
        ]
        .align_y(Alignment::Center)
        .padding(14),
    )
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(p.bg_card)),
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
