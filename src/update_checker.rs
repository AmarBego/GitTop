//! Update checker for GitTop.
//!
//! Checks GitHub releases API for newer stable versions.

use serde::Deserialize;

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// Current version (from Cargo.toml)
    pub current: String,
    /// Latest version available
    pub latest: String,
    /// URL to the release page
    pub release_url: String,
}

/// GitHub release response (minimal fields we need)
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    prerelease: bool,
    draft: bool,
}

/// Check for updates by querying GitHub releases API.
///
/// Returns `Some(UpdateInfo)` if a newer stable version is available,
/// `None` if current version is up-to-date or on any error (fail silently).
pub async fn check_for_update() -> Option<UpdateInfo> {
    let current = env!("CARGO_PKG_VERSION");

    // Use reqwest to fetch latest release
    // We use a simple client without auth (60 req/hour rate limit is plenty)
    let client = reqwest::Client::builder()
        .user_agent(concat!("GitTop/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let response = client
        .get("https://api.github.com/repos/AmarBego/GitTop/releases/latest")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        tracing::debug!(
            status = %response.status(),
            "Update check: GitHub API returned non-success status"
        );
        return None;
    }

    let release: GitHubRelease = response.json().await.ok()?;

    // Skip prereleases and drafts
    if release.prerelease || release.draft {
        tracing::debug!("Update check: Latest release is prerelease/draft, skipping");
        return None;
    }

    // Parse version from tag (strip leading 'v' if present)
    let latest = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);

    // Compare versions using semver
    let current_ver = semver::Version::parse(current).ok()?;
    let latest_ver = semver::Version::parse(latest).ok()?;

    if latest_ver > current_ver {
        tracing::info!(
            current = %current,
            latest = %latest,
            "Update available"
        );
        Some(UpdateInfo {
            current: current.to_string(),
            latest: latest.to_string(),
            release_url: release.html_url,
        })
    } else {
        tracing::debug!(
            current = %current,
            latest = %latest,
            "Already up to date"
        );
        None
    }
}
