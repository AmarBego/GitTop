//! Bulk action update logic.

use iced::Task;

use crate::github::{GitHubClient, GitHubError, NotificationView};

use super::message::BulkActionMessage;
use super::state::BulkActionState;

/// Result of a bulk action update.
pub struct BulkActionResult {
    pub task: Task<BulkActionMessage>,
    pub needs_rebuild: bool,
}

impl BulkActionResult {
    fn none() -> Self {
        Self {
            task: Task::none(),
            needs_rebuild: false,
        }
    }

    fn rebuild_with_task(task: Task<BulkActionMessage>) -> Self {
        Self {
            task,
            needs_rebuild: true,
        }
    }
}

/// Update bulk action state and return any side effects.
pub fn update_bulk_action(
    state: &mut BulkActionState,
    message: BulkActionMessage,
    notifications: &mut Vec<NotificationView>,
    client: &GitHubClient,
) -> BulkActionResult {
    match message {
        BulkActionMessage::ToggleMode => {
            state.bulk_mode = !state.bulk_mode;
            if !state.bulk_mode {
                state.selected_ids.clear();
                state.selected_ids.shrink_to_fit();
            }
            BulkActionResult::none()
        }

        BulkActionMessage::ToggleSelect(id) => {
            if state.selected_ids.contains(&id) {
                state.selected_ids.remove(&id);
            } else {
                state.selected_ids.insert(id);
            }
            BulkActionResult::none()
        }

        BulkActionMessage::SelectAll(ids) => {
            for id in ids {
                state.selected_ids.insert(id);
            }
            BulkActionResult::none()
        }

        BulkActionMessage::Clear => {
            state.selected_ids.clear();
            BulkActionResult::none()
        }

        BulkActionMessage::MarkAsRead => {
            // Optimistic update
            for id in &state.selected_ids {
                if let Some(notif) = notifications.iter_mut().find(|n| &n.id == id) {
                    notif.unread = false;
                }
            }

            let client = client.clone();
            let ids: Vec<String> = state.selected_ids.iter().cloned().collect();
            state.clear();

            BulkActionResult::rebuild_with_task(Task::perform(
                async move {
                    for id in ids {
                        let _ = client.mark_as_read(&id).await;
                    }
                    Ok::<(), GitHubError>(())
                },
                BulkActionMessage::Complete,
            ))
        }

        BulkActionMessage::MarkAsDone => {
            // Optimistic update - remove from list
            let ids_to_remove: Vec<String> = state.selected_ids.iter().cloned().collect();
            notifications.retain(|n| !state.selected_ids.contains(&n.id));

            let client = client.clone();
            state.clear();

            BulkActionResult::rebuild_with_task(Task::perform(
                async move {
                    for id in ids_to_remove {
                        let _ = client.mark_thread_as_done(&id).await;
                    }
                    Ok::<(), GitHubError>(())
                },
                BulkActionMessage::Complete,
            ))
        }

        BulkActionMessage::Complete(_result) => BulkActionResult::none(),
    }
}
