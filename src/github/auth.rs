//! Authentication module for secure credential storage and validation.

use keyring::Entry;
use thiserror::Error;

use super::client::{GitHubClient, GitHubError};
use super::types::UserInfo;

/// Service name for keyring storage.
const SERVICE_NAME: &str = "gittop";
const ACCOUNT_NAME: &str = "github_pat";

/// Authentication-specific errors.
#[derive(Debug, Error, Clone)]
pub enum AuthError {
    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("GitHub API error: {0}")]
    GitHub(#[from] GitHubError),
}

/// Creates a new keyring entry.
fn get_entry() -> Result<Entry, AuthError> {
    Entry::new(SERVICE_NAME, ACCOUNT_NAME).map_err(|e| AuthError::Keyring(e.to_string()))
}

/// Saves the token to secure storage.
pub fn save_token(token: &str) -> Result<(), AuthError> {
    let entry = get_entry()?;
    entry
        .set_password(token)
        .map_err(|e| AuthError::Keyring(e.to_string()))?;
    Ok(())
}

/// Deletes the stored token.
pub fn delete_token() -> Result<(), AuthError> {
    let entry = get_entry()?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(AuthError::Keyring(e.to_string())),
    }
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
        super::proxy_keyring::load_proxy_credentials(&proxy_settings.url)
            .map_err(|e| AuthError::Keyring(e.to_string()))?
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
    save_token(token)?;

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
