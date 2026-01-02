//! GitHub API client using Personal Access Tokens.

use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
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
        Self::new_with_proxy(token, &crate::settings::AppSettings::load().proxy)
    }

    /// Creates a new GitHub client with the given Personal Access Token and proxy settings.
    /// Loads credentials from keyring if indicated by settings.
    pub fn new_with_proxy(
        token: impl Into<String>,
        proxy_settings: &crate::settings::ProxySettings,
    ) -> Result<Self, GitHubError> {
        let (username, password) = if proxy_settings.has_credentials {
            crate::github::proxy_keyring::load_proxy_credentials(&proxy_settings.url)
                .map_err(|e| {
                    GitHubError::Request(format!("Failed to load proxy credentials: {}", e))
                })?
                .map(|(u, p)| (Some(u), Some(p)))
                .unwrap_or((None, None))
        } else {
            (None, None)
        };

        Self::new_with_proxy_and_credentials(token, proxy_settings, username, password)
    }

    /// Creates a new GitHub client with explicit proxy credentials.
    pub fn new_with_proxy_and_credentials(
        token: impl Into<String>,
        proxy_settings: &crate::settings::ProxySettings,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<Self, GitHubError> {
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

        let mut client_builder = reqwest::Client::builder()
            .default_headers(headers)
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(1);

        // Configure proxy if enabled
        if proxy_settings.enabled && !proxy_settings.url.is_empty() {
            let mut proxy_builder = reqwest::Proxy::all(&proxy_settings.url)
                .map_err(|e| GitHubError::Request(format!("Invalid proxy URL: {}", e)))?;

            if let Some(user) = username
                && !user.is_empty()
            {
                let pass = password.as_deref().unwrap_or("");
                proxy_builder = proxy_builder.basic_auth(&user, pass);
            }

            client_builder = client_builder.proxy(proxy_builder);
        }

        let client = client_builder.build()?;

        Ok(Self { client, token })
    }

    /// Validates and handles the response status.
    async fn handle_response(
        response: reqwest::Response,
    ) -> Result<reqwest::Response, GitHubError> {
        let status = response.status();

        if status.is_success() {
            Ok(response)
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

    /// Fetches the authenticated user's information.
    /// This is used to validate the token and get user details.
    pub async fn get_authenticated_user(&self) -> Result<UserInfo, GitHubError> {
        let url = format!("{}/user", GITHUB_API_URL);

        let response = self.client.get(&url).send().await?;
        let response = Self::handle_response(response).await?;

        let user: GitHubUser = response.json().await?;
        Ok(UserInfo {
            login: user.login,
            name: user.name,
            avatar_url: user.avatar_url,
            html_url: user.html_url,
        })
    }

    /// Validates a token by creating a client and fetching user info.
    /// Returns the client and user info if valid.
    #[allow(dead_code)]
    pub async fn validate_token(token: &str) -> Result<(Self, UserInfo), GitHubError> {
        // Load proxy settings
        let settings = crate::settings::AppSettings::load();
        let proxy_settings = &settings.proxy;

        Self::validate_token_with_proxy(token, proxy_settings).await
    }

    /// Validates a token by creating a client with proxy settings and fetching user info.
    /// Returns the client and user info if valid.
    pub async fn validate_token_with_proxy(
        token: &str,
        proxy_settings: &crate::settings::ProxySettings,
    ) -> Result<(Self, UserInfo), GitHubError> {
        // Basic format validation
        if let Err(e) = super::auth::validate_token_format(token) {
            return Err(GitHubError::Api {
                status: 400,
                message: e.to_string(),
            });
        }

        let client = Self::new_with_proxy(token, proxy_settings)?;
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
        let response = Self::handle_response(response).await?;
        Ok(response.json().await?)
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
        Self::handle_response(response).await.map(|_| ())
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

        Self::handle_response(response).await.map(|_| ())
    }

    /// Marks a thread as "done" (removes it from inbox).
    pub async fn mark_thread_as_done(&self, thread_id: &str) -> Result<(), GitHubError> {
        let url = format!("{}/notifications/threads/{}", GITHUB_API_URL, thread_id);

        let response = self.client.delete(&url).send().await?;
        Self::handle_response(response).await.map(|_| ())
    }

    /// Fetches Issue details from an API URL.
    ///
    /// The URL comes from `notification.subject.url` and is in the format:
    /// `https://api.github.com/repos/{owner}/{repo}/issues/{number}`
    pub async fn get_issue(
        &self,
        url: &str,
    ) -> Result<super::subject_details::IssueDetails, GitHubError> {
        let response = self.client.get(url).send().await?;

        let status = response.status();
        if status.as_u16() == 404 {
            return Err(GitHubError::Api {
                status: 404,
                message: "Issue not found".to_string(),
            });
        }

        let response = Self::handle_response(response).await?;
        Ok(response.json().await?)
    }

    /// Fetches Pull Request details from an API URL.
    ///
    /// The URL comes from `notification.subject.url` and is in the format:
    /// `https://api.github.com/repos/{owner}/{repo}/pulls/{number}`
    pub async fn get_pull_request(
        &self,
        url: &str,
    ) -> Result<super::subject_details::PullRequestDetails, GitHubError> {
        let response = self.client.get(url).send().await?;

        let status = response.status();
        if status.as_u16() == 404 {
            return Err(GitHubError::Api {
                status: 404,
                message: "Pull request not found".to_string(),
            });
        }

        let response = Self::handle_response(response).await?;
        Ok(response.json().await?)
    }

    /// Fetches Comment details from an API URL.
    ///
    /// The URL comes from `notification.subject.latest_comment_url`.
    pub async fn get_comment(
        &self,
        url: &str,
    ) -> Result<super::subject_details::CommentDetails, GitHubError> {
        let response = self.client.get(url).send().await?;

        let status = response.status();
        if status.as_u16() == 404 {
            return Err(GitHubError::Api {
                status: 404,
                message: "Comment not found".to_string(),
            });
        }

        let response = Self::handle_response(response).await?;
        Ok(response.json().await?)
    }

    /// Fetches notification subject details based on type.
    ///
    /// This is the high-level method that determines what to fetch based on:
    /// - subject_type: Issue, PullRequest, etc.
    /// - reason: If "mention", fetch the comment instead of the issue
    /// - subject_url: API URL for the Issue/PR
    /// - latest_comment_url: API URL for the latest comment (for mentions)
    pub async fn get_notification_details(
        &self,
        subject_type: super::types::SubjectType,
        subject_url: Option<&str>,
        latest_comment_url: Option<&str>,
        reason: super::types::NotificationReason,
        title: &str,
    ) -> Result<super::subject_details::NotificationSubjectDetail, GitHubError> {
        use super::subject_details::NotificationSubjectDetail;
        use super::types::{NotificationReason, SubjectType};

        // For "mention" reason, prioritize showing the comment that mentioned the user
        if reason == NotificationReason::Mention
            && let Some(comment_url) = latest_comment_url
        {
            let comment = self.get_comment(comment_url).await?;
            return Ok(NotificationSubjectDetail::Comment {
                comment,
                context_title: title.to_string(),
            });
        }

        match subject_type {
            SubjectType::Issue => {
                if let Some(url) = subject_url {
                    let issue = self.get_issue(url).await?;
                    Ok(NotificationSubjectDetail::Issue(issue))
                } else {
                    Ok(NotificationSubjectDetail::Unsupported {
                        subject_type: "Issue".to_string(),
                    })
                }
            }
            SubjectType::PullRequest => {
                if let Some(url) = subject_url {
                    let pr = self.get_pull_request(url).await?;
                    Ok(NotificationSubjectDetail::PullRequest(pr))
                } else {
                    Ok(NotificationSubjectDetail::Unsupported {
                        subject_type: "PullRequest".to_string(),
                    })
                }
            }
            SubjectType::RepositoryVulnerabilityAlert => {
                // Security alerts don't expose full content via REST API
                Ok(NotificationSubjectDetail::SecurityAlert {
                    title: title.to_string(),
                    severity: None,
                })
            }
            SubjectType::Discussion => {
                // Try to extract owner/repo/number from subject URL
                // Format: https://api.github.com/repos/{owner}/{repo}/discussions/{number}
                if let Some(url) = subject_url
                    && let Some((owner, repo, number)) = parse_discussion_url(url)
                    && let Ok(discussion) = self.get_discussion(&owner, &repo, number).await
                {
                    return Ok(NotificationSubjectDetail::Discussion(discussion));
                }
                // Fallback: minimal discussion details
                Ok(NotificationSubjectDetail::Discussion(
                    super::subject_details::DiscussionDetails {
                        title: title.to_string(),
                        body: None,
                        author: None,
                        category: None,
                        answer_chosen: false,
                        comments_count: 0,
                    },
                ))
            }
            _ => {
                // Release, CheckSuite, Commit, Unknown - unsupported for now
                Ok(NotificationSubjectDetail::Unsupported {
                    subject_type: format!("{:?}", subject_type),
                })
            }
        }
    }

    /// Fetches Discussion details via GraphQL API.
    ///
    /// Discussions are not available via REST API, so we use the GraphQL endpoint.
    pub async fn get_discussion(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<super::subject_details::DiscussionDetails, GitHubError> {
        use super::subject_details::{DiscussionCategory, DiscussionDetails};

        const GRAPHQL_URL: &str = "https://api.github.com/graphql";

        let query = format!(
            r#"{{
              repository(owner: "{}", name: "{}") {{
                discussion(number: {}) {{
                  number
                  title
                  body
                  url
                  author {{ login }}
                  category {{ name emoji }}
                  answerChosenAt
                  comments {{ totalCount }}
                  createdAt
                  updatedAt
                }}
              }}
            }}"#,
            owner, repo, number
        );

        let body = serde_json::json!({ "query": query });

        let response = self.client.post(GRAPHQL_URL).json(&body).send().await?;

        let status = response.status();
        if !status.is_success() {
            return Err(GitHubError::Api {
                status: status.as_u16(),
                message: "GraphQL request failed".to_string(),
            });
        }

        let json: serde_json::Value = response.json().await?;

        // Check for GraphQL errors
        if let Some(errors) = json.get("errors") {
            let msg = errors
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown GraphQL error");
            return Err(GitHubError::Api {
                status: 400,
                message: msg.to_string(),
            });
        }

        // Parse the discussion data
        let discussion = &json["data"]["repository"]["discussion"];
        if discussion.is_null() {
            return Err(GitHubError::Api {
                status: 404,
                message: "Discussion not found".to_string(),
            });
        }

        Ok(DiscussionDetails {
            title: discussion["title"].as_str().unwrap_or("").to_string(),
            body: discussion["body"].as_str().map(String::from),
            author: discussion["author"]["login"].as_str().map(String::from),
            category: discussion["category"]["name"]
                .as_str()
                .map(|name| DiscussionCategory {
                    name: name.to_string(),
                    emoji: discussion["category"]["emoji"].as_str().map(String::from),
                }),
            answer_chosen: discussion["answerChosenAt"].as_str().is_some(),
            comments_count: discussion["comments"]["totalCount"].as_u64().unwrap_or(0),
        })
    }

    /// Returns the token for storage purposes.
    #[allow(unused)]
    pub fn token(&self) -> &str {
        &self.token
    }
}

/// Parse discussion URL to extract owner, repo, and number.
/// Format: https://api.github.com/repos/{owner}/{repo}/discussions/{number}
/// Parse discussion URL to extract owner, repo, and number.
/// Format: https://api.github.com/repos/{owner}/{repo}/discussions/{number}
fn parse_discussion_url(url: &str) -> Option<(String, String, u64)> {
    let mut parts = url.split('/');
    // Expected: ["https:", "", "api.github.com", "repos", "{owner}", "{repo}", "discussions", "{number}"]

    // Skip protocol, empty, host, "repos" -> 4 items
    if parts.next()? != "https:" {
        return None;
    }
    if !parts.next()?.is_empty() {
        return None;
    }
    if parts.next()? != "api.github.com" {
        return None;
    }
    if parts.next()? != "repos" {
        return None;
    }

    let owner = parts.next()?.to_string();
    let repo = parts.next()?.to_string();

    if parts.next()? != "discussions" {
        return None;
    }

    let number = parts.next()?.parse().ok()?;

    Some((owner, repo, number))
}
