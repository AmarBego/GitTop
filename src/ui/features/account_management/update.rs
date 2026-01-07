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
                    let client =
                        GitHubClient::new(&token).map_err(|e| format!("Invalid token: {}", e))?;

                    let user = client
                        .get_authenticated_user()
                        .await
                        .map_err(|e| format!("Validation failed: {}", e))?;

                    keyring::save_token(&user.login, &token)
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
            let _ = keyring::delete_token(&username);
            tracing::info!(account_count = settings.accounts.len(), "Account removed");
            Task::none()
        }
    }
}
