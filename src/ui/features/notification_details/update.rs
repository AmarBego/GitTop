//! Notification details update logic.

use iced::Task;

use crate::github::subject_details::NotificationSubjectDetail;
use crate::github::{GitHubClient, NotificationView, SubjectType};
use crate::ui::screens::notifications::helper::api_url_to_web_url;

use super::message::NotificationDetailsMessage;
use super::state::NotificationDetailsState;

pub fn update_notification_details(
    state: &mut NotificationDetailsState,
    message: NotificationDetailsMessage,
    notifications: &[NotificationView],
    client: &GitHubClient,
) -> Task<NotificationDetailsMessage> {
    match message {
        NotificationDetailsMessage::Select(id) => {
            state.selected_id = Some(id.clone());
            state.details = None;
            state.is_loading = true;
            state.label_input.clear();
            state.label_error = None;
            state.available_labels.clear();
            state.pending_label_ops.clear();
            state.comments = None;
            state.comments_loading = false;
            state.comments_error = None;

            let client = client.clone();
            let subject_type = notifications
                .iter()
                .find(|n| n.id == id)
                .map(|n| n.subject_type)
                .unwrap_or(SubjectType::Unknown);
            let subject_url = notifications
                .iter()
                .find(|n| n.id == id)
                .and_then(|n| n.url.clone());
            let latest_comment_url = notifications
                .iter()
                .find(|n| n.id == id)
                .and_then(|n| n.latest_comment_url.clone());
            let reason = notifications
                .iter()
                .find(|n| n.id == id)
                .map(|n| n.reason)
                .unwrap_or(crate::github::NotificationReason::Unknown);
            let title = notifications
                .iter()
                .find(|n| n.id == id)
                .map(|n| n.title.clone())
                .unwrap_or_default();

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
        }

        NotificationDetailsMessage::SelectComplete(id, result) => {
            if state.selected_id.as_ref() == Some(&id) {
                state.is_loading = false;
                match result {
                    Ok(details) => {
                        let is_pr = matches!(&details, NotificationSubjectDetail::PullRequest(_));
                        state.details = Some(details.clone());

                        if is_pr && let Some(notif) = notifications.iter().find(|n| n.id == id) {
                            let (owner, repo) = split_repo_full_name(&notif.repo_full_name);
                            state.labels_loading = true;
                            state.checks_loading = true;

                            let client = client.clone();
                            let owner_s = owner.to_string();
                            let repo_s = repo.to_string();

                            let labels_client = client.clone();
                            let labels_owner = owner_s.clone();
                            let labels_repo = repo_s.clone();

                            let mut tasks = vec![Task::perform(
                                async move {
                                    labels_client
                                        .list_repo_labels(&labels_owner, &labels_repo)
                                        .await
                                },
                                NotificationDetailsMessage::RepoLabelsLoaded,
                            )];

                            if let NotificationSubjectDetail::PullRequest(ref pr) = details {
                                let sha = pr.head.sha.clone();
                                let checks_client = client.clone();
                                let checks_owner = owner_s.clone();
                                let checks_repo = repo_s.clone();
                                tasks.push(Task::perform(
                                    async move {
                                        checks_client
                                            .get_check_runs(&checks_owner, &checks_repo, &sha)
                                            .await
                                    },
                                    NotificationDetailsMessage::ChecksLoaded,
                                ));
                            }

                            return Task::batch(tasks);
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to fetch notification details");
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

        NotificationDetailsMessage::SetLabelInput(text) => {
            state.label_input = text;
            Task::none()
        }

        NotificationDetailsMessage::SubmitAddLabel(owner, repo, pr_number, label) => {
            let label = label.trim().to_string();
            if label.is_empty() {
                return Task::none();
            }
            state.pending_label_ops.insert(label.clone());
            state.label_input.clear();

            let client = client.clone();
            let label_clone = label.clone();
            let labels = vec![label];
            Task::perform(
                async move {
                    client
                        .add_pr_labels(&owner, &repo, pr_number, &labels)
                        .await
                },
                move |result| NotificationDetailsMessage::AddLabelComplete(label_clone, result),
            )
        }

        NotificationDetailsMessage::SubmitLabelFromInput(owner, repo, pr_number) => {
            let label = state.label_input.trim().to_string();
            if label.is_empty() {
                return Task::none();
            }
            state.pending_label_ops.insert(label.clone());
            state.label_input.clear();

            let client = client.clone();
            let label_clone = label.clone();
            let labels = vec![label];
            Task::perform(
                async move {
                    client
                        .add_pr_labels(&owner, &repo, pr_number, &labels)
                        .await
                },
                move |result| NotificationDetailsMessage::AddLabelComplete(label_clone, result),
            )
        }

        NotificationDetailsMessage::AddLabelComplete(label, result) => {
            state.pending_label_ops.remove(&label);
            match result {
                Ok(updated_labels) => {
                    if let Some(NotificationSubjectDetail::PullRequest(ref mut pr)) = state.details
                    {
                        pr.labels = updated_labels;

                        // Refresh checks as requested
                        let (owner, repo) = (state
                            .selected_id
                            .as_ref()
                            .and_then(|id| notifications.iter().find(|n| &n.id == id)))
                        .map(|n| split_repo_full_name(&n.repo_full_name))
                        .unwrap_or(("", ""));

                        if !owner.is_empty() {
                            let sha = pr.head.sha.clone();
                            let client = client.clone();
                            let owner = owner.to_string();
                            let repo = repo.to_string();
                            state.checks_loading = true;
                            return Task::perform(
                                async move { client.get_check_runs(&owner, &repo, &sha).await },
                                NotificationDetailsMessage::ChecksLoaded,
                            );
                        }
                    }
                }
                Err(e) => {
                    state.label_error = Some(format!("Failed to add label: {}", e));
                }
            }
            Task::none()
        }

        NotificationDetailsMessage::RemoveLabel(owner, repo, pr_number, name) => {
            state.pending_label_ops.insert(name.clone());
            let client = client.clone();
            let name_clone = name.clone();
            Task::perform(
                async move {
                    client
                        .remove_pr_label(&owner, &repo, pr_number, &name)
                        .await
                },
                move |result| NotificationDetailsMessage::RemoveLabelComplete(name_clone, result),
            )
        }

        NotificationDetailsMessage::RemoveLabelComplete(name, result) => {
            state.pending_label_ops.remove(&name);
            match result {
                Ok(()) => {
                    if let Some(NotificationSubjectDetail::PullRequest(ref mut pr)) = state.details
                    {
                        pr.labels.retain(|l| l.name != name);

                        // Refresh checks as requested
                        let (owner, repo) = (state
                            .selected_id
                            .as_ref()
                            .and_then(|id| notifications.iter().find(|n| &n.id == id)))
                        .map(|n| split_repo_full_name(&n.repo_full_name))
                        .unwrap_or(("", ""));

                        if !owner.is_empty() {
                            let sha = pr.head.sha.clone();
                            let client = client.clone();
                            let owner = owner.to_string();
                            let repo = repo.to_string();
                            state.checks_loading = true;
                            return Task::perform(
                                async move { client.get_check_runs(&owner, &repo, &sha).await },
                                NotificationDetailsMessage::ChecksLoaded,
                            );
                        }
                    }
                }
                Err(e) => {
                    state.label_error = Some(format!("Failed to remove label: {}", e));
                }
            }
            Task::none()
        }

        NotificationDetailsMessage::RepoLabelsLoaded(result) => {
            state.labels_loading = false;
            match result {
                Ok(labels) => {
                    state.available_labels = labels;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to load repo labels");
                }
            }
            Task::none()
        }

        NotificationDetailsMessage::RefreshChecks(owner, repo, sha) => {
            state.checks_loading = true;
            let client = client.clone();
            Task::perform(
                async move { client.get_check_runs(&owner, &repo, &sha).await },
                NotificationDetailsMessage::ChecksLoaded,
            )
        }

        NotificationDetailsMessage::ChecksLoaded(result) => {
            state.checks_loading = false;
            match result {
                Ok(response) => {
                    state.check_runs = response.check_runs;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to load check runs");
                }
            }
            Task::none()
        }

        NotificationDetailsMessage::LoadComments(subject_url) => {
            state.comments_loading = true;
            state.comments_error = None;
            let client = client.clone();
            Task::perform(
                async move { client.get_comments(&subject_url).await },
                NotificationDetailsMessage::CommentsLoaded,
            )
        }

        NotificationDetailsMessage::CommentsLoaded(result) => {
            state.comments_loading = false;
            match result {
                Ok(comments) => {
                    state.comments = Some(comments);
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to load comments");
                    state.comments_error = Some("Failed to load comments".to_string());
                }
            }
            Task::none()
        }
    }
}

fn split_repo_full_name(full_name: &str) -> (&str, &str) {
    let mut parts = full_name.splitn(2, '/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(repo)) => (owner, repo),
        _ => (full_name, ""),
    }
}
