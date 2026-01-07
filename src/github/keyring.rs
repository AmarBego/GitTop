//! Account Keyring - Secure per-account credential storage.
//!
//! Uses the system keyring to store GitHub PATs keyed by username.
//! Format: service="gittop", user="gittop-{username}"

use keyring::Entry;
use thiserror::Error;

use super::redaction::redact_secrets;

/// Service name for keyring storage.
const SERVICE_NAME: &str = "gittop";

/// Keyring-specific errors.
#[derive(Debug, Error, Clone)]
pub enum KeyringError {
    #[error("Keyring error: {0}")]
    Internal(String),
}

/// Creates a keyring entry for a specific username.
fn get_entry(username: &str) -> Result<Entry, KeyringError> {
    let key = format!("gittop-{}", username);
    Entry::new(SERVICE_NAME, &key)
        .map_err(|e| KeyringError::Internal(redact_secrets(&e.to_string())))
}

/// Saves a token for a specific account.
pub fn save_token(username: &str, token: &str) -> Result<(), KeyringError> {
    let entry = get_entry(username)?;
    entry
        .set_password(token)
        .map_err(|e| KeyringError::Internal(redact_secrets(&e.to_string())))?;
    Ok(())
}

/// Loads the token for a specific account.
pub fn load_token(username: &str) -> Result<Option<String>, KeyringError> {
    let entry = get_entry(username)?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(KeyringError::Internal(redact_secrets(&e.to_string()))),
    }
}

/// Deletes the token for a specific account.
pub fn delete_token(username: &str) -> Result<(), KeyringError> {
    let entry = get_entry(username)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(KeyringError::Internal(redact_secrets(&e.to_string()))),
    }
}
