//! GitHub API client using Personal Access Tokens.

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use thiserror::Error;

use super::types::{Notification, NotificationView, UserInfo};

/// GitHub API base URL.
const GITHUB_API_URL: &str = "https://api.github.com";

/// Errors that can occur when interacting with the GitHub API.
#[derive(Debug, Error, Clone)]
pub enum GitHubError {
    #[error("HTTP request failed: {0}")]
    Request(String),

    #[error("Invalid or expired token")]
    Unauthorized,

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("GitHub API error: {status} - {message}")]
    Api { status: u16, message: String },
}

impl From<reqwest::Error> for GitHubError {
    fn from(e: reqwest::Error) -> Self {
        GitHubError::Request(e.to_string())
    }
}

/// Raw GitHub user response.
#[derive(Debug, Deserialize)]
struct GitHubUser {
    login: String,
    name: Option<String>,
    avatar_url: String,
    html_url: String,
}

/// GitHub API client.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    client: reqwest::Client,
    token: String,
}

impl GitHubClient {
    /// Creates a new GitHub client with the given Personal Access Token.
    pub fn new(token: impl Into<String>) -> Result<Self, GitHubError> {
        let token = token.into();

        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("GitTop/0.1.0"));
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|_| GitHubError::Unauthorized)?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { client, token })
    }

    /// Fetches the authenticated user's information.
    /// This is used to validate the token and get user details.
    pub async fn get_authenticated_user(&self) -> Result<UserInfo, GitHubError> {
        let url = format!("{}/user", GITHUB_API_URL);

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if status.is_success() {
            let user: GitHubUser = response.json().await?;
            Ok(UserInfo {
                login: user.login,
                name: user.name,
                avatar_url: user.avatar_url,
                html_url: user.html_url,
            })
        } else if status.as_u16() == 401 {
            Err(GitHubError::Unauthorized)
        } else if status.as_u16() == 403 {
            Err(GitHubError::RateLimited)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(GitHubError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    /// Validates a token by creating a client and fetching user info.
    /// Returns the client and user info if valid.
    pub async fn validate_token(token: &str) -> Result<(Self, UserInfo), GitHubError> {
        // Basic format validation
        if !token.starts_with("ghp_") && !token.starts_with("github_pat_") {
            return Err(GitHubError::Api {
                status: 400,
                message: "Token must start with 'ghp_' or 'github_pat_'".to_string(),
            });
        }

        let client = Self::new(token)?;
        let user = client.get_authenticated_user().await?;
        Ok((client, user))
    }

    /// Fetches the user's notifications.
    pub async fn get_notifications(&self, all: bool) -> Result<Vec<Notification>, GitHubError> {
        let url = format!(
            "{}/notifications?all={}&participating=false",
            GITHUB_API_URL, all
        );

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(response.json().await?)
        } else if status.as_u16() == 401 {
            Err(GitHubError::Unauthorized)
        } else if status.as_u16() == 403 {
            Err(GitHubError::RateLimited)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(GitHubError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    /// Fetches notifications and converts them to frontend-friendly format.
    /// The account parameter identifies which GitHub account these notifications belong to.
    pub async fn get_notification_views(
        &self,
        all: bool,
        account: &str,
    ) -> Result<Vec<NotificationView>, GitHubError> {
        let notifications = self.get_notifications(all).await?;
        let account = account.to_string();
        Ok(notifications
            .into_iter()
            .map(|n| NotificationView::from_notification(n, account.clone()))
            .collect())
    }

    /// Marks a notification as read.
    pub async fn mark_as_read(&self, notification_id: &str) -> Result<(), GitHubError> {
        let url = format!(
            "{}/notifications/threads/{}",
            GITHUB_API_URL, notification_id
        );

        let response = self.client.patch(&url).send().await?;
        let status = response.status();

        if status.is_success() || status.as_u16() == 205 {
            Ok(())
        } else if status.as_u16() == 401 {
            Err(GitHubError::Unauthorized)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(GitHubError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    /// Marks all notifications as read.
    pub async fn mark_all_as_read(&self) -> Result<(), GitHubError> {
        let url = format!("{}/notifications", GITHUB_API_URL);

        let response = self
            .client
            .put(&url)
            .json(&serde_json::json!({}))
            .send()
            .await?;

        let status = response.status();

        if status.is_success() || status.as_u16() == 205 {
            Ok(())
        } else if status.as_u16() == 401 {
            Err(GitHubError::Unauthorized)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(GitHubError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    /// Marks a thread as "done" (removes it from inbox).
    pub async fn mark_thread_as_done(&self, thread_id: &str) -> Result<(), GitHubError> {
        let url = format!("{}/notifications/threads/{}", GITHUB_API_URL, thread_id);

        let response = self.client.delete(&url).send().await?;
        let status = response.status();

        if status.is_success() || status.as_u16() == 204 {
            Ok(())
        } else if status.as_u16() == 401 {
            Err(GitHubError::Unauthorized)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(GitHubError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    /// Deletes the subscription for a thread (mutes future notifications).
    pub async fn delete_thread_subscription(&self, thread_id: &str) -> Result<(), GitHubError> {
        let url = format!(
            "{}/notifications/threads/{}/subscription",
            GITHUB_API_URL, thread_id
        );

        let response = self.client.delete(&url).send().await?;
        let status = response.status();

        if status.is_success() || status.as_u16() == 204 {
            Ok(())
        } else if status.as_u16() == 401 {
            Err(GitHubError::Unauthorized)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(GitHubError::Api {
                status: status.as_u16(),
                message,
            })
        }
    }

    /// Returns the token for storage purposes.
    #[allow(unused)]
    pub fn token(&self) -> &str {
        &self.token
    }
}
