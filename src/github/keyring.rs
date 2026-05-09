//! Account credential storage — secure per-account token persistence.
//!
//! Stores GitHub PATs keyed by username via the central credential_store
//! abstraction, which dispatches between the OS keyring and an encrypted
//! file based on the user's `CredentialStorage` setting.
//!
//! Format: service="gittop", account="gittop-{username}".

use thiserror::Error;

use super::credential_store::{self, CredentialError};

const SERVICE_NAME: &str = "gittop";

/// Account-credential errors. Kept as a distinct type for callers that
/// want to type-discriminate, but it just wraps `CredentialError`.
#[derive(Debug, Error, Clone)]
pub enum KeyringError {
    #[error("Keyring error: {0}")]
    Internal(String),
}

impl From<CredentialError> for KeyringError {
    fn from(e: CredentialError) -> Self {
        KeyringError::Internal(e.to_string())
    }
}

fn account_key(username: &str) -> String {
    format!("gittop-{}", username)
}

/// Saves a token for a specific account.
pub fn save_token(username: &str, token: &str) -> Result<(), KeyringError> {
    credential_store::save(SERVICE_NAME, &account_key(username), token).map_err(Into::into)
}

/// Loads the token for a specific account.
pub fn load_token(username: &str) -> Result<Option<String>, KeyringError> {
    credential_store::load(SERVICE_NAME, &account_key(username)).map_err(Into::into)
}

/// Deletes the token for a specific account.
pub fn delete_token(username: &str) -> Result<(), KeyringError> {
    credential_store::delete(SERVICE_NAME, &account_key(username)).map_err(Into::into)
}
