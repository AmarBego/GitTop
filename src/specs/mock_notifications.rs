//! Mock notification generator for scroll performance testing.
//!
//! Usage: cargo run -- --mock-notifications 1000

use chrono::{Duration, Utc};

use crate::github::{NotificationReason, NotificationView, SubjectType};

/// Generate mock notifications for testing scroll performance.
pub fn generate_mock_notifications(count: usize, account: &str) -> Vec<NotificationView> {
    let subject_types = [
        SubjectType::Issue,
        SubjectType::PullRequest,
        SubjectType::Release,
        SubjectType::Discussion,
        SubjectType::CheckSuite,
        SubjectType::Commit,
    ];

    let reasons = [
        NotificationReason::Assign,
        NotificationReason::Author,
        NotificationReason::Comment,
        NotificationReason::Mention,
        NotificationReason::ReviewRequested,
        NotificationReason::Subscribed,
        NotificationReason::TeamMention,
    ];

    let repos = [
        ("rust-lang", "rust"),
        ("denoland", "deno"),
        ("tokio-rs", "tokio"),
        ("serde-rs", "serde"),
        ("iced-rs", "iced"),
        ("tauri-apps", "tauri"),
        ("microsoft", "vscode"),
        ("facebook", "react"),
        ("vercel", "next.js"),
        ("golang", "go"),
    ];

    let titles = [
        "Fix memory leak in async runtime",
        "Add support for dark mode",
        "Update documentation for v2.0",
        "Refactor error handling",
        "Implement lazy loading for lists",
        "Fix race condition in scheduler",
        "Add unit tests for parser",
        "Improve performance of hot path",
        "Migrate to new API version",
        "Fix typo in README",
        "Add internationalization support",
        "Implement caching layer",
        "Fix build on Windows",
        "Add CI/CD pipeline",
        "Update dependencies",
    ];

    let now = Utc::now();
    let mut notifications = Vec::with_capacity(count);

    for i in 0..count {
        let subject_type = subject_types[i % subject_types.len()];
        let reason = reasons[i % reasons.len()];
        let (owner, repo) = repos[i % repos.len()];
        let title = titles[i % titles.len()];

        // Spread updates over time: some recent, some older
        let age_minutes = match i % 10 {
            0..=2 => i as i64 * 2,           // Recent (0-6 min)
            3..=5 => 60 + (i as i64 * 10),   // Hours ago
            6..=8 => 1440 + (i as i64 * 30), // Days ago
            _ => 10080 + (i as i64 * 60),    // Weeks ago
        };
        let updated_at = now - Duration::minutes(age_minutes);

        notifications.push(NotificationView {
            id: format!("mock-{}", i),
            title: format!("{} (#{}) - {}", title, i, subject_type),
            repo_name: repo.to_string(),
            repo_full_name: format!("{}/{}", owner, repo),
            subject_type,
            reason,
            unread: i % 3 != 0, // ~67% unread
            updated_at,
            time_ago: format_time_ago(updated_at),
            url: Some(format!(
                "https://api.github.com/repos/{}/{}/issues/{}",
                owner, repo, i
            )),
            latest_comment_url: None,
            avatar_url: format!("https://github.com/{}.png", owner),
            is_private: i % 10 == 0, // 10% private
            account: account.to_string(),
        });
    }

    notifications
}

fn format_time_ago(dt: chrono::DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_minutes() < 1 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d", duration.num_days())
    } else {
        dt.format("%b %d").to_string()
    }
}
