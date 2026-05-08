use iced::widget::{Space, button, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Element, Fill};

use crate::settings::{AppSettings, CredentialStorage, StoredAccount};
use crate::ui::screens::settings::components::{setting_card, tab_title};
use crate::ui::{icons, theme};

use super::message::AccountMessage;
use super::state::{AccountManagementState, SubmissionStatus};

/// Render the accounts tab content.
pub fn view<'a>(
    state: &'a AccountManagementState,
    settings: &'a AppSettings,
) -> Element<'a, AccountMessage> {
    let p = theme::palette();

    column![
        tab_title("Accounts"),
        text("Manage your GitHub accounts.")
            .size(12)
            .color(p.text_secondary),
        Space::new().height(16),
        view_add_account_section(state, settings),
        Space::new().height(16),
        view_accounts_list(settings),
        Space::new().height(16),
        view_credential_storage_section(settings),
    ]
    .spacing(4)
    .padding(24)
    .width(Fill)
    .into()
}

fn view_add_account_section<'a>(
    state: &'a AccountManagementState,
    settings: &'a AppSettings,
) -> Element<'a, AccountMessage> {
    let p = theme::palette();
    let icon_theme = settings.icon_theme;

    let is_validating = matches!(state.status, SubmissionStatus::Validating);

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
                .on_input(AccountMessage::TokenInputChanged)
                .padding([8, 12])
                .size(13)
                .width(Fill)
                .style(theme::text_input_style),
            Space::new().width(8),
            button(if is_validating {
                text("Validating...").size(13).color(iced::Color::WHITE)
            } else {
                text("Add").size(13).color(iced::Color::WHITE)
            })
            .style(theme::primary_button)
            .padding([8, 16])
            .on_press_maybe(if is_validating || state.token_input.is_empty() {
                None
            } else {
                Some(AccountMessage::SubmitToken)
            }),
        ]
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    // Show error or success message based on status
    match &state.status {
        SubmissionStatus::Error(error) => {
            content = content.push(Space::new().height(8));
            content = content.push(text(error).size(12).color(p.accent_danger));
        }
        SubmissionStatus::Success(success) => {
            content = content.push(Space::new().height(8));
            content = content.push(text(success).size(12).color(p.accent_success));
        }
        _ => {}
    }

    setting_card(content)
}

fn view_accounts_list(settings: &AppSettings) -> Element<'static, AccountMessage> {
    let p = theme::palette();

    if settings.accounts.is_empty() {
        return container(text("No accounts added yet").size(12).color(p.text_muted))
            .padding(14)
            .into();
    }

    let account_items = settings
        .accounts
        .iter()
        .map(|account| view_account_item(account, settings));

    column![
        text("Connected Accounts").size(13).color(p.text_secondary),
        Space::new().height(8),
    ]
    .spacing(8)
    .extend(account_items)
    .into()
}

/// Storage backend selector + warning when the chosen backend doesn't work.
fn view_credential_storage_section(settings: &AppSettings) -> Element<'_, AccountMessage> {
    let p = theme::palette();
    let icon_theme = settings.icon_theme;

    let options = [CredentialStorage::Keyring, CredentialStorage::EncryptedFile];
    let selector = pick_list(
        options,
        Some(settings.credential_storage),
        AccountMessage::CredentialStorageChanged,
    )
    .text_size(13)
    .padding([6, 12]);

    let mut content = column![
        row![
            icons::icon_settings(14.0, p.accent, icon_theme),
            Space::new().width(8),
            text("Credential Storage").size(14).color(p.text_primary),
        ]
        .align_y(Alignment::Center),
        Space::new().height(8),
        text(
            "Where GitHub tokens and proxy credentials are kept. \
             System Keyring uses Secret Service / Credential Manager / \
             Keychain. Encrypted File falls back to a local file when no \
             secret service is running."
        )
        .size(11)
        .color(p.text_secondary),
        Space::new().height(12),
        selector,
    ]
    .spacing(2);

    // Surface the same warning here so users browsing settings see why
    // their tokens aren't sticking, even if they skipped the login banner.
    let keyring_chosen = settings.credential_storage == CredentialStorage::Keyring;
    if keyring_chosen && !crate::github::keyring_available() {
        content = content.push(Space::new().height(10));
        content = content.push(
            text(
                "\u{26A0} System keyring not reachable on this machine. \
                 Switch to Encrypted File to make tokens persist across restarts."
            )
            .size(12)
            .color(p.accent_danger),
        );
    } else if settings.credential_storage == CredentialStorage::EncryptedFile {
        content = content.push(Space::new().height(10));
        content = content.push(
            text(
                "Tokens are encrypted at rest with a key stored in the GitTop \
                 data directory. Treat your home directory as sensitive — \
                 anyone who can read both files can decrypt the tokens."
            )
            .size(11)
            .color(p.text_secondary),
        );
    }

    setting_card(content)
}

fn view_account_item(
    account: &StoredAccount,
    settings: &AppSettings,
) -> Element<'static, AccountMessage> {
    let p = theme::palette();
    let icon_theme = settings.icon_theme;

    // We need owned strings for both output elements because we are returning Element<'static>
    let username_display = account.username.clone();
    let username_msg = account.username.clone();

    container(
        row![
            icons::icon_user(14.0, p.text_secondary, icon_theme),
            Space::new().width(8),
            text(username_display).size(13).color(p.text_primary),
            Space::new().width(8),
            Space::new().width(Fill),
            button(icons::icon_trash(14.0, p.text_muted, icon_theme))
                .style(theme::ghost_button)
                .padding(6)
                .on_press(AccountMessage::RemoveAccount(username_msg)),
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
