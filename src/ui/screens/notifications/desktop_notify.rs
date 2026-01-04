use crate::ui::screens::notifications::engine::DesktopNotificationBatch;
use crate::ui::screens::notifications::helper::{ProcessedNotification, api_url_to_web_url};
use std::collections::HashMap;

/// Send desktop notifications for a batch of processed notifications.
pub fn send_desktop_notifications(
    processed: &[ProcessedNotification],
    seen_timestamps: &HashMap<String, chrono::DateTime<chrono::Utc>>,
) {
    let batch = DesktopNotificationBatch::from_processed(processed, seen_timestamps);

    if batch.is_empty() {
        return;
    }

    // Send priority notifications individually
    for p in &batch.priority {
        let notif = &p.notification;
        let title = format!(
            "Important: {} - {}",
            notif.repo_full_name, notif.subject_type
        );
        let url = notif.url.as_ref().map(|u| api_url_to_web_url(u));
        let body = format!("{}\n{}", notif.title, notif.reason.label());
        if let Err(e) = crate::platform::notify(&title, &body, url.as_deref()) {
            eprintln!("Failed to send notification: {}", e);
        }
    }

    if batch.regular.is_empty() {
        return;
    }

    // Send regular notifications
    if batch.regular.len() == 1 {
        let notif = &batch.regular[0].notification;
        let title = format!("{} - {}", notif.repo_full_name, notif.subject_type);
        let url = notif.url.as_ref().map(|u| api_url_to_web_url(u));
        let body = format!("{}\n{}", notif.title, notif.reason.label());

        if let Err(e) = crate::platform::notify(&title, &body, url.as_deref()) {
            eprintln!("Failed to send notification: {}", e);
        }
    } else {
        let title = format!("{} new GitHub notifications", batch.regular.len());
        let body = batch
            .regular
            .iter()
            .take(3)
            .map(|p| format!("• {}", p.notification.title))
            .collect::<Vec<_>>()
            .join("\n");

        let body = if batch.regular.len() > 3 {
            format!("{}\n...and {} more", body, batch.regular.len() - 3)
        } else {
            body
        };

        if let Err(e) = crate::platform::notify(&title, &body, None) {
            eprintln!("Failed to send notification: {}", e);
        }
    }

    crate::platform::trim_memory();
}
