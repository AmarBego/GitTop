//! Notifications screen - main notification list view.
//!
//! Layout: Sidebar | Main Content
//! - Sidebar: Types filter, Repositories filter, User info
//! - Main: Content header + notification list

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Fill, Task};

use crate::github::{GitHubClient, GitHubError, NotificationView, SubjectType, UserInfo};
use crate::settings::IconTheme;
use crate::ui::{icons, theme, window_state};

use super::group::{view_group_header, view_group_items};
use super::helper::{
    api_url_to_web_url, apply_filters, count_by_repo, count_by_type, group_by_time, FilterSettings,
    NotificationGroup,
};
use super::sidebar::view_sidebar;
use super::states::{view_empty, view_error, view_loading};

use std::collections::HashMap;

/// Notifications screen state.
#[derive(Debug, Clone)]
pub struct NotificationsScreen {
    pub client: GitHubClient,
    pub user: UserInfo,
    pub all_notifications: Vec<NotificationView>,
    pub filtered_notifications: Vec<NotificationView>,
    pub groups: Vec<NotificationGroup>,
    pub filters: FilterSettings,
    pub is_loading: bool,
    pub error_message: Option<String>,
    /// Cached counts by subject type (computed on data change).
    pub type_counts: Vec<(SubjectType, usize)>,
    /// Cached counts by repository (computed on data change).
    pub repo_counts: Vec<(String, usize)>,
    /// Track seen notifications by ID -> updated_at timestamp.
    /// This detects both new notifications AND updates to existing ones.
    seen_notification_timestamps: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

/// Notifications screen messages.
#[derive(Debug, Clone)]
#[allow(dead_code)] // MarkAsRead/MarkAsDone/MuteThread have handlers, pending UI buttons
pub enum NotificationMessage {
    Refresh,
    RefreshComplete(Result<Vec<NotificationView>, GitHubError>),
    Open(String),
    MarkAsRead(String),
    MarkAsReadComplete(String, Result<(), GitHubError>),
    MarkAllAsRead,
    MarkAllAsReadComplete(Result<(), GitHubError>),
    ToggleShowAll,
    Logout,
    ToggleGroup(usize),
    // Filter actions
    SelectType(Option<SubjectType>),
    SelectRepo(Option<String>),
    // Thread actions
    MarkAsDone(String),
    MarkAsDoneComplete(String, Result<(), GitHubError>),
    MuteThread(String),
    MuteThreadComplete(String, Result<(), GitHubError>),
    // Navigation
    OpenSettings,
}

impl NotificationsScreen {
    pub fn new(client: GitHubClient, user: UserInfo) -> (Self, Task<NotificationMessage>) {
        let screen = Self {
            client,
            user,
            all_notifications: Vec::new(),
            filtered_notifications: Vec::new(),
            groups: Vec::new(),
            filters: FilterSettings::default(),
            is_loading: true,
            error_message: None,
            type_counts: Vec::new(),
            repo_counts: Vec::new(),
            seen_notification_timestamps: HashMap::new(),
        };
        let task = screen.fetch_notifications();
        (screen, task)
    }

    fn fetch_notifications(&self) -> Task<NotificationMessage> {
        let client = self.client.clone();
        let show_all = self.filters.show_all;
        Task::perform(
            async move { client.get_notification_views(show_all).await },
            NotificationMessage::RefreshComplete,
        )
    }

    fn rebuild_groups(&mut self) {
        // Recompute cached counts from all notifications
        self.type_counts = count_by_type(&self.all_notifications);
        self.repo_counts = count_by_repo(&self.all_notifications);
        // Apply filters
        self.filtered_notifications = apply_filters(&self.all_notifications, &self.filters);
        // Then group by time
        self.groups = group_by_time(&self.filtered_notifications);
    }

    /// Send desktop notifications for new or updated unread notifications.
    /// Only called when window is hidden in tray.
    fn send_desktop_notifications(&self, notifications: &[NotificationView]) {
        eprintln!(
            "[DEBUG] send_desktop_notifications called with {} notifications",
            notifications.len()
        );

        // Find new or updated unread notifications
        // A notification is "new" if:
        // 1. We've never seen this ID before, OR
        // 2. The updated_at timestamp is newer than what we recorded
        let new_notifications: Vec<_> = notifications
            .iter()
            .filter(|n| {
                if !n.unread {
                    return false;
                }
                match self.seen_notification_timestamps.get(&n.id) {
                    None => true,                                 // Never seen this ID
                    Some(last_seen) => n.updated_at > *last_seen, // Updated since last seen
                }
            })
            .collect();

        eprintln!(
            "[DEBUG] Found {} new/updated unread notifications (seen count: {})",
            new_notifications.len(),
            self.seen_notification_timestamps.len()
        );

        if new_notifications.is_empty() {
            eprintln!("[DEBUG] No new notifications to show, returning");
            return;
        }

        // If there's just one new notification, show it directly
        if new_notifications.len() == 1 {
            let notif = &new_notifications[0];
            let title = format!("{} - {}", notif.repo_full_name, notif.subject_type);
            let url = notif.url.as_ref().map(|u| api_url_to_web_url(u));

            // Include reason in body for context (e.g., "mentioned", "review requested")
            let body = format!("{}\n{}", notif.title, notif.reason.label());

            eprintln!("[DEBUG] Sending single notification: {:?}", title);
            crate::platform::notify(&title, &body, url.as_deref());
        } else {
            // Multiple notifications - show a summary
            let title = format!("{} new GitHub notifications", new_notifications.len());
            let body = new_notifications
                .iter()
                .take(3) // Show first 3
                .map(|n| format!("• {}", n.title))
                .collect::<Vec<_>>()
                .join("\n");

            let body = if new_notifications.len() > 3 {
                format!("{}\\n...and {} more", body, new_notifications.len() - 3)
            } else {
                body
            };

            eprintln!("[DEBUG] Sending summary notification: {:?}", title);
            // No specific URL for summary - just notify
            crate::platform::notify(&title, &body, None);
        }
    }

    pub fn update(&mut self, message: NotificationMessage) -> Task<NotificationMessage> {
        match message {
            NotificationMessage::Refresh => {
                self.is_loading = true;
                self.error_message = None;
                self.fetch_notifications()
            }
            NotificationMessage::RefreshComplete(result) => {
                self.is_loading = false;
                match result {
                    Ok(notifications) => {
                        eprintln!(
                            "[DEBUG] RefreshComplete: got {} notifications",
                            notifications.len()
                        );

                        // Check for new notifications to send desktop notifications
                        // Only notify when window is hidden (in tray)
                        let is_hidden = window_state::is_hidden();
                        eprintln!("[DEBUG] is_hidden = {}", is_hidden);

                        if is_hidden {
                            self.send_desktop_notifications(&notifications);
                        } else {
                            eprintln!("[DEBUG] Window is visible, skipping desktop notifications");
                        }

                        // Update seen timestamps with current notifications
                        for n in &notifications {
                            self.seen_notification_timestamps
                                .insert(n.id.clone(), n.updated_at);
                        }

                        // If hidden, don't store the data - keep memory minimal
                        // The data will be fetched fresh when window is restored
                        if is_hidden {
                            // Don't update all_notifications - keep it empty
                            // Aggressively trim memory after the API call
                            crate::platform::trim_memory();
                        } else {
                            self.all_notifications = notifications;
                            self.rebuild_groups();
                        }
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.error_message = Some(e.to_string());
                    }
                }
                Task::none()
            }
            NotificationMessage::Open(id) => {
                if let Some(notif) = self.all_notifications.iter().find(|n| n.id == id) {
                    if let Some(ref url) = notif.url {
                        let web_url = api_url_to_web_url(url);
                        let _ = open::that(&web_url);
                    }
                }
                let client = self.client.clone();
                let notif_id = id.clone();
                Task::perform(
                    async move { client.mark_as_read(&notif_id).await },
                    move |result| NotificationMessage::MarkAsReadComplete(id.clone(), result),
                )
            }
            NotificationMessage::MarkAsRead(id) => {
                let client = self.client.clone();
                let notif_id = id.clone();
                Task::perform(
                    async move { client.mark_as_read(&notif_id).await },
                    move |result| NotificationMessage::MarkAsReadComplete(id.clone(), result),
                )
            }
            NotificationMessage::MarkAsReadComplete(id, result) => {
                if result.is_ok() {
                    if let Some(notif) = self.all_notifications.iter_mut().find(|n| n.id == id) {
                        notif.unread = false;
                        self.rebuild_groups();
                    }
                }
                Task::none()
            }
            NotificationMessage::MarkAllAsRead => {
                let client = self.client.clone();
                Task::perform(
                    async move { client.mark_all_as_read().await },
                    NotificationMessage::MarkAllAsReadComplete,
                )
            }
            NotificationMessage::MarkAllAsReadComplete(result) => {
                if result.is_ok() {
                    for notif in &mut self.all_notifications {
                        notif.unread = false;
                    }
                    self.rebuild_groups();
                }
                Task::none()
            }
            NotificationMessage::ToggleShowAll => {
                self.filters.show_all = !self.filters.show_all;
                self.is_loading = true;
                self.fetch_notifications()
            }
            NotificationMessage::Logout => Task::none(),
            NotificationMessage::ToggleGroup(index) => {
                if let Some(group) = self.groups.get_mut(index) {
                    group.is_expanded = !group.is_expanded;
                }
                Task::none()
            }
            NotificationMessage::SelectType(subject_type) => {
                self.filters.selected_type = subject_type;
                self.filters.selected_repo = None; // Clear repo filter
                self.rebuild_groups();
                Task::none()
            }
            NotificationMessage::SelectRepo(repo) => {
                self.filters.selected_repo = repo;
                self.filters.selected_type = None; // Clear type filter
                self.rebuild_groups();
                Task::none()
            }
            NotificationMessage::MarkAsDone(id) => {
                let client = self.client.clone();
                let notif_id = id.clone();
                Task::perform(
                    async move { client.mark_thread_as_done(&notif_id).await },
                    move |result| NotificationMessage::MarkAsDoneComplete(id.clone(), result),
                )
            }
            NotificationMessage::MarkAsDoneComplete(id, result) => {
                if result.is_ok() {
                    self.all_notifications.retain(|n| n.id != id);
                    self.rebuild_groups();
                }
                Task::none()
            }
            NotificationMessage::MuteThread(id) => {
                let client = self.client.clone();
                let notif_id = id.clone();
                Task::perform(
                    async move { client.delete_thread_subscription(&notif_id).await },
                    move |result| NotificationMessage::MuteThreadComplete(id.clone(), result),
                )
            }
            NotificationMessage::MuteThreadComplete(id, result) => {
                if result.is_ok() {
                    self.all_notifications.retain(|n| n.id != id);
                    self.rebuild_groups();
                }
                Task::none()
            }
            NotificationMessage::OpenSettings => {
                // Handled by parent (app.rs)
                Task::none()
            }
        }
    }

    pub fn view(&self, icon_theme: IconTheme) -> Element<'_, NotificationMessage> {
        row![
            // Sidebar
            view_sidebar(
                &self.user,
                &self.type_counts,
                &self.repo_counts,
                self.filters.selected_type,
                self.filters.selected_repo.as_deref(),
                self.all_notifications.len(),
                icon_theme,
            ),
            // Main content area
            self.view_main_content(icon_theme)
        ]
        .height(Fill)
        .into()
    }

    fn view_main_content(&self, icon_theme: IconTheme) -> Element<'_, NotificationMessage> {
        column![
            self.view_content_header(icon_theme),
            self.view_content(icon_theme)
        ]
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn view_content_header(&self, icon_theme: IconTheme) -> Element<'_, NotificationMessage> {
        let p = theme::palette();
        let unread_count = self
            .filtered_notifications
            .iter()
            .filter(|n| n.unread)
            .count();

        let title = text("Notifications").size(18).color(p.text_primary);

        let sync_status: Element<'_, NotificationMessage> = if self.is_loading {
            row![
                icons::icon_refresh(11.0, p.text_muted, icon_theme),
                Space::new().width(4),
                text("Syncing...").size(11).color(p.text_muted),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                icons::icon_check(11.0, p.accent_success, icon_theme),
                Space::new().width(4),
                text("Synced").size(11).color(p.accent_success),
            ]
            .align_y(Alignment::Center)
            .into()
        };

        let filter_btn = button(
            text(if self.filters.show_all {
                "All"
            } else {
                "Unread"
            })
            .size(12),
        )
        .style(theme::secondary_button)
        .padding([6, 10])
        .on_press(NotificationMessage::ToggleShowAll);

        let mark_all_btn = if unread_count > 0 {
            button(
                row![
                    icons::icon_check(11.0, p.text_secondary, icon_theme),
                    Space::new().width(4),
                    text("Mark all read").size(12),
                ]
                .align_y(Alignment::Center),
            )
            .style(theme::ghost_button)
            .padding([6, 10])
            .on_press(NotificationMessage::MarkAllAsRead)
        } else {
            button(
                row![
                    icons::icon_check(11.0, p.text_muted, icon_theme),
                    Space::new().width(4),
                    text("Mark all read").size(12).color(p.text_muted),
                ]
                .align_y(Alignment::Center),
            )
            .style(theme::ghost_button)
            .padding([6, 10])
        };

        let refresh_btn = button(icons::icon_refresh(14.0, p.text_secondary, icon_theme))
            .style(theme::ghost_button)
            .padding(8)
            .on_press(NotificationMessage::Refresh);

        let header_row = row![
            title,
            Space::new().width(12),
            sync_status,
            Space::new().width(Fill),
            filter_btn,
            Space::new().width(6),
            mark_all_btn,
            Space::new().width(6),
            refresh_btn,
        ]
        .align_y(Alignment::Center)
        .padding([14, 16]);

        container(header_row)
            .width(Fill)
            .style(theme::header)
            .into()
    }

    fn view_content(&self, icon_theme: IconTheme) -> Element<'_, NotificationMessage> {
        if self.is_loading && self.all_notifications.is_empty() {
            return view_loading();
        }

        if let Some(ref error) = self.error_message {
            return view_error(error, icon_theme);
        }

        if self.filtered_notifications.is_empty() {
            return view_empty(self.filters.show_all, icon_theme);
        }

        // Build content with groups
        let mut content = column![].spacing(8).padding([8, 8]);

        for (group_idx, group) in self.groups.iter().enumerate() {
            if group.notifications.is_empty() {
                continue;
            }

            content = content.push(view_group_header(group, group_idx, icon_theme));

            if group.is_expanded {
                content = content.push(view_group_items(group, icon_theme));
            }

            content = content.push(Space::new().height(8));
        }

        container(
            scrollable(content)
                .height(Fill)
                .width(Fill)
                .style(theme::scrollbar),
        )
        .style(theme::app_container)
        .height(Fill)
        .width(Fill)
        .into()
    }
}
