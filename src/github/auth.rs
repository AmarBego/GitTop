//! Authentication module for credential validation and the legacy single-token
//! storage slot.
//!
//! Most token persistence now lives in `keyring.rs` (multi-account, keyed by
//! username). This module retains the older single-token entry for backward
//! compatibility — it's written on every successful auth and cleared on
//! logout, but no code reads it back. Both writes and the delete dispatch
//! through the central `credential_store` so they honor the user's
//! `CredentialStorage` setting.

use thiserror::Error;

use super::client::{GitHubClient, GitHubError};
use super::credential_store;
use super::redaction::redact_secrets;
use super::types::UserInfo;

const SERVICE_NAME: &str = "gittop";
const ACCOUNT_NAME: &str = "github_pat";

#[derive(Debug, Error, Clone)]
pub enum AuthError {
    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("GitHub API error: {0}")]
    GitHub(#[from] GitHubError),
}

/// Saves the token to the legacy single-account slot.
pub fn save_token(token: &str) -> Result<(), AuthError> {
    credential_store::save(SERVICE_NAME, ACCOUNT_NAME, token)
        .map_err(|e| AuthError::Keyring(redact_secrets(&e.to_string())))
}

/// Deletes the legacy single-account slot.
pub fn delete_token() -> Result<(), AuthError> {
    credential_store::delete(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| AuthError::Keyring(redact_secrets(&e.to_string())))
}

/// Full authentication flow: validate token, save to keyring, return user info.
pub async fn authenticate(
    token: &str,
    proxy_settings: Option<&crate::settings::ProxySettings>,
) -> Result<(GitHubClient, UserInfo), AuthError> {
    // Validate token format first
    validate_token_format(token)?;

    // Load proxy settings from AppSettings if not provided
    let proxy_settings: crate::settings::ProxySettings = match proxy_settings {
        Some(settings) => settings.clone(),
        None => {
            let app_settings = crate::settings::AppSettings::load();
            app_settings.proxy
        }
    };

    // Load proxy credentials from keyring if settings indicate they exist
    let (username, password) = if proxy_settings.has_credentials {
        let url = proxy_settings.url.clone();
        tokio::task::spawn_blocking(move || super::proxy_keyring::load_proxy_credentials(&url))
            .await
            .map_err(|e| AuthError::Keyring(format!("Spawn blocking failed: {}", e)))?
            .map_err(|e| AuthError::Keyring(redact_secrets(&e.to_string())))?
            .map(|(u, p)| (Some(u), Some(p)))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    // Create client with proxy settings and credentials
    let client =
        GitHubClient::new_with_proxy_and_credentials(token, &proxy_settings, username, password)?;

    // Fetch user info
    let user = client.get_authenticated_user().await?;

    // Save to secure storage
    let token_clone = token.to_string();
    tokio::task::spawn_blocking(move || save_token(&token_clone))
        .await
        .map_err(|e| AuthError::Keyring(format!("Spawn blocking failed: {}", e)))??;

    Ok((client, user))
}

/// Validates the format of a GitHub Personal Access Token.
/// Checks for 'ghp_' or 'github_pat_' prefix and non-empty content.
pub fn validate_token_format(token: &str) -> Result<(), AuthError> {
    if token.is_empty() {
        return Err(AuthError::Keyring("Token cannot be empty".to_string()));
    }
    if !token.starts_with("ghp_") && !token.starts_with("github_pat_") {
        return Err(AuthError::Keyring(
            "Token must start with 'ghp_' or 'github_pat_'".to_string(),
        ));
    }
    Ok(())
}
