//! GitHub API module - authentication, client, and types.

pub mod auth;
pub mod client;
pub mod keyring;
pub mod session;
pub mod types;

pub use auth::AuthManager;
pub use client::{GitHubClient, GitHubError};
pub use keyring::AccountKeyring;
pub use session::SessionManager;
pub use types::*;
