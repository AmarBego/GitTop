//! GitHub API module - authentication, client, and types.

pub mod auth;
pub mod client;
pub mod credential_store;
pub mod keyring;
pub mod proxy_keyring;
pub mod redaction;
pub mod session;
pub mod subject_details;
pub mod types;

pub use client::{GitHubClient, GitHubError};
pub use credential_store::{init_keyring, keyring_available, native_backend_name, unset_keyring};
pub use session::SessionManager;
pub use subject_details::NotificationSubjectDetail;
pub use types::*;
