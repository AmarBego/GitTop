//! Authentication module for secure credential storage and validation.

use keyring::Entry;
use thiserror::Error;

use crate::settings::AppSettings;

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
pub async fn authenticate(token: &str) -> Result<(GitHubClient, UserInfo), AuthError> {
    // Validate token format first
    validate_token_format(token)?;

    // Load proxy settings
    let settings = AppSettings::load();
    let proxy_settings = &settings.proxy;

    // Create client with proxy settings
    let client = GitHubClient::new_with_proxy(token, proxy_settings)?;

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
