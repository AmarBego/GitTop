//! Session Manager - Multi-account GitHub session handling.
//!
//! Manages multiple authenticated GitHub clients simultaneously.

use std::collections::HashMap;

use super::client::{GitHubClient, GitHubError};
use super::keyring::{self, KeyringError};
use super::types::UserInfo;
use thiserror::Error;

/// Session-related errors.
#[derive(Debug, Error, Clone)]
pub enum SessionError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] KeyringError),

    #[error("GitHub API error: {0}")]
    GitHub(#[from] GitHubError),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

/// An authenticated session for a single account.
#[derive(Debug, Clone)]
pub struct Session {
    pub username: String,
    pub client: GitHubClient,
    pub user: UserInfo,
}

/// Manages multiple GitHub sessions.
#[derive(Debug, Clone, Default)]
pub struct SessionManager {
    sessions: HashMap<String, Session>,
    primary: Option<String>,
}

impl SessionManager {
    /// Creates a new empty session manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Restore a session for a known username (loads token from keyring).
    pub async fn restore_account(&mut self, username: &str) -> Result<(), SessionError> {
        let token = keyring::load_token(username)?
            .ok_or_else(|| SessionError::AccountNotFound(username.to_string()))?;

        // Load proxy settings
        let settings = crate::settings::AppSettings::load();
        let proxy_settings = &settings.proxy;

        // Validate the token using GitHubClient with proxy
        let (client, user) =
            match GitHubClient::validate_token_with_proxy(&token, proxy_settings).await {
                Ok((client, user)) => (client, user),
                Err(GitHubError::Unauthorized) => {
                    // Token expired/revoked from GitHub (401), clean up
                    let _ = keyring::delete_token(username);
                    return Err(SessionError::AccountNotFound(username.to_string()));
                }
                Err(GitHubError::Request(msg)) => {
                    // Connection/network error - keep account, report network issue
                    return Err(SessionError::NetworkError(msg));
                }
                Err(GitHubError::Api { status, message }) => {
                    // API error that's NOT from GitHub auth:
                    // - 407 = Proxy authentication required
                    // - Other statuses could be proxy/network issues
                    // Don't delete token for these
                    return Err(SessionError::NetworkError(format!(
                        "API error (status {}): {}",
                        status, message
                    )));
                }
                Err(GitHubError::RateLimited) => {
                    // Rate limited - definitely keep account, just can't fetch now
                    return Err(SessionError::NetworkError(
                        "GitHub rate limit exceeded".to_string(),
                    ));
                }
            };

        // Create session
        let session = Session {
            username: username.to_string(),
            client,
            user,
        };

        // If this is the first account, make it primary
        if self.sessions.is_empty() {
            self.primary = Some(username.to_string());
        }

        self.sessions.insert(username.to_string(), session);

        Ok(())
    }

    /// Remove an account (also deletes from keyring).
    pub fn remove_account(&mut self, username: &str) -> Result<(), SessionError> {
        self.sessions.remove(username);
        keyring::delete_token(username)?;

        // If we removed the primary, pick a new one
        if self.primary.as_deref() == Some(username) {
            self.primary = self.sessions.keys().next().cloned();
        }

        Ok(())
    }

    /// Add a session manually (e.g. after restoration).
    pub fn add_session(&mut self, session: Session) {
        let username = session.username.clone();
        // If this is the first account, make it primary
        if self.sessions.is_empty() {
            self.primary = Some(username.clone());
        }
        self.sessions.insert(username, session);
    }

    /// Get the primary session.
    pub fn primary(&self) -> Option<&Session> {
        self.primary
            .as_ref()
            .and_then(|name| self.sessions.get(name))
    }

    /// Get mutable primary session.
    #[allow(dead_code)]
    pub fn primary_mut(&mut self) -> Option<&mut Session> {
        let name = self.primary.as_ref()?;
        self.sessions.get_mut(name)
    }

    /// Set which account is primary.
    #[allow(dead_code)]
    pub fn set_primary(&mut self, username: &str) {
        if self.sessions.contains_key(username) {
            self.primary = Some(username.to_string());
        }
    }

    /// Get all active session usernames.
    pub fn usernames(&self) -> impl Iterator<Item = &str> {
        self.sessions.keys().map(String::as_str)
    }

    /// Get a specific session by username.
    #[allow(dead_code)]
    pub fn get(&self, username: &str) -> Option<&Session> {
        self.sessions.get(username)
    }

    /// Rebuild all clients with updated proxy settings.
    /// Call this after proxy settings have changed to apply them to existing sessions.
    pub fn rebuild_clients_with_proxy(
        &mut self,
        proxy_settings: &crate::settings::ProxySettings,
    ) -> Result<(), GitHubError> {
        for session in self.sessions.values_mut() {
            // Load token from keyring for this user
            let token = match super::keyring::load_token(&session.username) {
                Ok(Some(t)) => t,
                Ok(None) => continue, // Skip if no token found
                Err(_) => continue,   // Skip on keyring error
            };

            // Rebuild client with new proxy settings
            let new_client = GitHubClient::new_with_proxy(&token, proxy_settings)?;
            session.client = new_client;

            eprintln!(
                "[PROXY] Rebuilt client for user '{}' with proxy enabled={}",
                session.username, proxy_settings.enabled
            );
        }
        Ok(())
    }

    /// Number of active sessions.
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.sessions.len()
    }
}
