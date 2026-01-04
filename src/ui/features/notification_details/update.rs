//! Notification details update logic.

use iced::Task;

use crate::github::{GitHubClient, NotificationView};
use crate::ui::screens::notifications::helper::api_url_to_web_url;

use super::message::NotificationDetailsMessage;
use super::state::NotificationDetailsState;

/// Update notification details state and return any side effects.
pub fn update_notification_details(
    state: &mut NotificationDetailsState,
    message: NotificationDetailsMessage,
    notifications: &[NotificationView],
    client: &GitHubClient,
) -> Task<NotificationDetailsMessage> {
    match message {
        NotificationDetailsMessage::Select(id) => {
            if let Some(notif) = notifications.iter().find(|n| n.id == id) {
                state.selected_id = Some(id.clone());
                state.details = None;
                state.is_loading = true;

                let client = client.clone();
                let subject_type = notif.subject_type;
                let subject_url = notif.url.clone();
                let latest_comment_url = notif.latest_comment_url.clone();
                let reason = notif.reason;
                let title = notif.title.clone();

                Task::perform(
                    async move {
                        client
                            .get_notification_details(
                                subject_type,
                                subject_url.as_deref(),
                                latest_comment_url.as_deref(),
                                reason,
                                &title,
                            )
                            .await
                    },
                    move |result| NotificationDetailsMessage::SelectComplete(id.clone(), result),
                )
            } else {
                Task::none()
            }
        }

        NotificationDetailsMessage::SelectComplete(id, result) => {
            if state.selected_id.as_ref() == Some(&id) {
                state.is_loading = false;
                match result {
                    Ok(details) => {
                        state.details = Some(details);
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Failed to fetch notification details: {}", e);
                        state.details = None;
                    }
                }
            }
            Task::none()
        }

        NotificationDetailsMessage::OpenInBrowser => {
            if let Some(ref id) = state.selected_id
                && let Some(notif) = notifications.iter().find(|n| &n.id == id)
                && let Some(ref url) = notif.url
            {
                let web_url = api_url_to_web_url(url);
                let _ = open::that(&web_url);
            }
            Task::none()
        }
    }
}
