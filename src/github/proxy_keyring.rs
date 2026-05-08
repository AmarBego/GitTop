//! Proxy credential storage.
//!
//! Stores proxy username/password pairs through the central credential_store
//! abstraction, which dispatches between the OS keyring and an encrypted
//! file based on the user's `CredentialStorage` setting.
//!
//! Format: service="gittop", account="proxy-{proxy_url_hash}". Value is
//! "username:password".

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use thiserror::Error;

use super::credential_store::{self, CredentialError};

const SERVICE_NAME: &str = "gittop";
const PROXY_KEY_PREFIX: &str = "proxy-";

#[derive(Debug, Error, Clone)]
pub enum ProxyKeyringError {
    #[error("Keyring error: {0}")]
    Internal(String),
}

impl From<CredentialError> for ProxyKeyringError {
    fn from(e: CredentialError) -> Self {
        ProxyKeyringError::Internal(e.to_string())
    }
}

/// Hash the proxy URL to a stable account key. Uses DefaultHasher because
/// the resulting value is opaque storage-only — collision resistance against
/// adversaries isn't required, only that the same URL produces the same key.
fn account_key(proxy_url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    proxy_url.hash(&mut hasher);
    format!("{}{:x}", PROXY_KEY_PREFIX, hasher.finish())
}

pub fn save_proxy_credentials(
    proxy_url: &str,
    username: &str,
    password: &str,
) -> Result<(), ProxyKeyringError> {
    let credentials = format!("{}:{}", username, password);
    credential_store::save(SERVICE_NAME, &account_key(proxy_url), &credentials)
        .map_err(Into::into)
}

pub fn load_proxy_credentials(
    proxy_url: &str,
) -> Result<Option<(String, String)>, ProxyKeyringError> {
    let raw = credential_store::load(SERVICE_NAME, &account_key(proxy_url))?;
    let Some(credentials) = raw else {
        return Ok(None);
    };
    Ok(credentials
        .split_once(':')
        .map(|(u, p)| (u.to_string(), p.to_string())))
}

pub fn delete_proxy_credentials(proxy_url: &str) -> Result<(), ProxyKeyringError> {
    credential_store::delete(SERVICE_NAME, &account_key(proxy_url)).map_err(Into::into)
}
