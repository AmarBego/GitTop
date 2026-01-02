//! Proxy credential storage using system keyring.
//!
//! Provides secure storage for proxy authentication credentials.
//! Format: service="gittop", user="proxy-{proxy_url_hash}"

use keyring::Entry;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use thiserror::Error;

/// Service name for keyring storage.
const SERVICE_NAME: &str = "gittop";

/// Prefix for proxy credential entries in keyring.
const PROXY_KEY_PREFIX: &str = "proxy-";

/// Keyring-specific errors for proxy credentials.
#[derive(Debug, Error, Clone)]
pub enum ProxyKeyringError {
    #[error("Keyring error: {0}")]
    Internal(String),
}

/// Creates a unique key for a proxy URL by hashing it.
fn hash_proxy_url(proxy_url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    proxy_url.hash(&mut hasher);
    format!("{}{:x}", PROXY_KEY_PREFIX, hasher.finish())
}

/// Creates a keyring entry for proxy credentials.
fn get_entry(proxy_url: &str) -> Result<Entry, ProxyKeyringError> {
    let key = hash_proxy_url(proxy_url);
    Entry::new(SERVICE_NAME, &key).map_err(|e| ProxyKeyringError::Internal(e.to_string()))
}

/// Saves proxy credentials to secure storage.
///
/// # Arguments
/// * `proxy_url` - The proxy URL (used as identifier)
/// * `username` - The proxy authentication username
/// * `password` - The proxy authentication password
pub fn save_proxy_credentials(
    proxy_url: &str,
    username: &str,
    password: &str,
) -> Result<(), ProxyKeyringError> {
    let entry = get_entry(proxy_url)?;
    // Store as "username:password" format
    let credentials = format!("{}:{}", username, password);
    entry
        .set_password(&credentials)
        .map_err(|e| ProxyKeyringError::Internal(e.to_string()))?;
    Ok(())
}

/// Loads proxy credentials from secure storage.
///
/// # Arguments
/// * `proxy_url` - The proxy URL used when credentials were saved
///
/// # Returns
/// A tuple of (username, password) if credentials exist, None otherwise
pub fn load_proxy_credentials(
    proxy_url: &str,
) -> Result<Option<(String, String)>, ProxyKeyringError> {
    let entry = get_entry(proxy_url)?;
    match entry.get_password() {
        Ok(credentials) => {
            // Parse "username:password" format
            if let Some((username, password)) = credentials.split_once(':') {
                Ok(Some((username.to_string(), password.to_string())))
            } else {
                // Malformed data, return None
                Ok(None)
            }
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(ProxyKeyringError::Internal(e.to_string())),
    }
}

/// Deletes proxy credentials from secure storage.
///
/// # Arguments
/// * `proxy_url` - The proxy URL used when credentials were saved
pub fn delete_proxy_credentials(proxy_url: &str) -> Result<(), ProxyKeyringError> {
    let entry = get_entry(proxy_url)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
        Err(e) => Err(ProxyKeyringError::Internal(e.to_string())),
    }
}
