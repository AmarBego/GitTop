use super::message::AccountMessage;
use super::state::{AccountManagementState, SubmissionStatus};
use crate::github::{GitHubClient, keyring};
use crate::settings::AppSettings;
use iced::Task;

pub fn update(
    state: &mut AccountManagementState,
    message: AccountMessage,
    settings: &mut AppSettings,
) -> Task<AccountMessage> {
    match message {
        AccountMessage::TokenInputChanged(token) => {
            state.token_input = token;
            state.status = SubmissionStatus::Idle;
            Task::none()
        }
        AccountMessage::SubmitToken => {
            let token = state.token_input.clone();
            if let Err(e) = crate::github::auth::validate_token_format(&token) {
                state.status = SubmissionStatus::Error(e.to_string());
                tracing::warn!(error = %e, "Token format validation failed");
                return Task::none();
            }

            state.status = SubmissionStatus::Validating;
            tracing::info!("Account validation requested");

            Task::perform(
                async move {
                    let token_clone = token.clone();
                    let client = tokio::task::spawn_blocking(move || {
                        GitHubClient::new(token_clone)
                    })
                    .await
                    .map_err(|e| format!("Failed to spawn blocking task: {}", e))?
                    .map_err(|e| format!("Invalid token: {}", e))?;

                    let user = client
                        .get_authenticated_user()
                        .await
                        .map_err(|e| format!("Validation failed: {}", e))?;

                    let login_clone = user.login.clone();
                    let token_clone = token.clone();
                    tokio::task::spawn_blocking(move || {
                        keyring::save_token(&login_clone, &token_clone)
                    })
                    .await
                    .map_err(|e| format!("Failed to spawn blocking task: {}", e))?
                    .map_err(|e| format!("Failed to save token: {}", e))?;

                    Ok(user.login)
                },
                AccountMessage::TokenValidated,
            )
        }
        AccountMessage::TokenValidated(result) => {
            match result {
                Ok(username) => {
                    settings.set_active_account(&username);
                    let _ = settings.save();
                    state.token_input.clear();
                    state.status = SubmissionStatus::Success(format!(
                        "Account '{}' added successfully!",
                        username
                    ));
                    tracing::info!(account_count = settings.accounts.len(), "Account added");
                }
                Err(error) => {
                    let error_msg = error.clone();
                    state.status = SubmissionStatus::Error(error);
                    tracing::warn!(error = %error_msg, "Account validation failed");
                }
            }
            Task::none()
        }
        AccountMessage::RemoveAccount(username) => {
            settings.remove_account(&username);
            let _ = settings.save();
            if let Err(e) = keyring::delete_token(&username) {
                tracing::warn!(
                    username = %username,
                    error = %e,
                    "Failed to delete token from keyring during account removal"
                );
            }
            tracing::info!(account_count = settings.accounts.len(), "Account removed");
            Task::none()
        }
        AccountMessage::CredentialStorageChanged(storage) => {
            // Switching backends does not migrate existing credentials —
            // entries written to the old backend stay there. Affected users
            // need to re-add their accounts; the view shows a static notice
            // explaining that.
            if settings.credential_storage == storage {
                return Task::none();
            }
            tracing::info!(
                from = %settings.credential_storage,
                to = %storage,
                "Credential storage backend changed"
            );
            settings.credential_storage = storage;
            let _ = settings.save();
            Task::none()
        }
    }
}
