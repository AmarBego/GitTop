//! Thread action update logic.

use iced::Task;

use crate::github::{GitHubClient, NotificationView};
use crate::ui::screens::notifications::helper::api_url_to_web_url;

use super::message::ThreadActionMessage;
use super::state::ThreadActionState;

/// Result of a thread action update.
pub struct ThreadActionResult {
    pub task: Task<ThreadActionMessage>,
    pub needs_rebuild: bool,
    pub needs_refresh: bool,
}

impl ThreadActionResult {
    fn none() -> Self {
        Self {
            task: Task::none(),
            needs_rebuild: false,
            needs_refresh: false,
        }
    }

    fn task(task: Task<ThreadActionMessage>) -> Self {
        Self {
            task,
            needs_rebuild: false,
            needs_refresh: false,
        }
    }

    fn rebuild() -> Self {
        Self {
            task: Task::none(),
            needs_rebuild: true,
            needs_refresh: false,
        }
    }

    fn rebuild_with_task(task: Task<ThreadActionMessage>) -> Self {
        Self {
            task,
            needs_rebuild: true,
            needs_refresh: false,
        }
    }
}

/// Update thread action state and return any side effects.
///
/// Takes mutable references to the notifications list to apply changes.
/// Returns a result indicating what further actions the screen should take.
pub fn update_thread_action(
    state: &mut ThreadActionState,
    message: ThreadActionMessage,
    notifications: &mut Vec<NotificationView>,
    client: &GitHubClient,
) -> ThreadActionResult {
    match message {
        ThreadActionMessage::Open(id) => {
            // Open in browser
            if let Some(notif) = notifications.iter().find(|n| n.id == id)
                && let Some(ref url) = notif.url
            {
                let web_url = api_url_to_web_url(url);
                let _ = open::that(&web_url);
            }

            // Mark as read
            state.pending_mark_read.insert(id.clone());
            let client = client.clone();
            let notif_id = id.clone();
            ThreadActionResult::task(Task::perform(
                async move { client.mark_as_read(&notif_id).await },
                move |result| ThreadActionMessage::MarkAsReadComplete(id.clone(), result),
            ))
        }

        ThreadActionMessage::MarkAsRead(id) => {
            state.pending_mark_read.insert(id.clone());
            let client = client.clone();
            let notif_id = id.clone();
            ThreadActionResult::task(Task::perform(
                async move { client.mark_as_read(&notif_id).await },
                move |result| ThreadActionMessage::MarkAsReadComplete(id.clone(), result),
            ))
        }

        ThreadActionMessage::MarkAsReadComplete(id, result) => {
            state.pending_mark_read.remove(&id);
            if result.is_ok() {
                if let Some(notif) = notifications.iter_mut().find(|n| n.id == id) {
                    notif.unread = false;
                }
                ThreadActionResult::rebuild()
            } else {
                ThreadActionResult::none()
            }
        }

        ThreadActionMessage::MarkAllAsRead => {
            state.pending_mark_all = true;
            // Optimistic update
            for notif in notifications.iter_mut() {
                notif.unread = false;
            }

            let client = client.clone();
            ThreadActionResult::rebuild_with_task(Task::perform(
                async move { client.mark_all_as_read().await },
                ThreadActionMessage::MarkAllAsReadComplete,
            ))
        }

        ThreadActionMessage::MarkAllAsReadComplete(_result) => {
            state.pending_mark_all = false;
            // Trigger a full refresh to sync with server
            ThreadActionResult {
                task: Task::none(),
                needs_rebuild: false,
                needs_refresh: true,
            }
        }

        ThreadActionMessage::MarkAsDone(id) => {
            state.pending_mark_done.insert(id.clone());
            let client = client.clone();
            let notif_id = id.clone();
            ThreadActionResult::task(Task::perform(
                async move { client.mark_thread_as_done(&notif_id).await },
                move |result| ThreadActionMessage::MarkAsDoneComplete(id.clone(), result),
            ))
        }

        ThreadActionMessage::MarkAsDoneComplete(id, result) => {
            state.pending_mark_done.remove(&id);
            if result.is_ok() {
                notifications.retain(|n| n.id != id);
                ThreadActionResult::rebuild()
            } else {
                ThreadActionResult::none()
            }
        }
    }
}
