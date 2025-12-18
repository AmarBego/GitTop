//! Notifications screen - main notification list view.
//!
//! Layout: Sidebar | Main Content
//! - Sidebar: Types filter, Repositories filter, User info
//! - Main: Content header + notification list

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Fill, Task};

use crate::github::{GitHubClient, GitHubError, NotificationView, SubjectType, UserInfo};
use crate::ui::{icons, theme};

use super::group::{view_group_header, view_group_items};
use super::helper::{
    api_url_to_web_url, apply_filters, count_by_repo, count_by_type, group_by_time, FilterSettings,
    NotificationGroup,
};
use super::sidebar::view_sidebar;
use super::states::{view_empty, view_error, view_loading};

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
}

/// Notifications screen messages.
#[derive(Debug, Clone)]
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
                        self.all_notifications = notifications;
                        self.rebuild_groups();
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
        }
    }

    pub fn view(&self) -> Element<'_, NotificationMessage> {
        row![
            // Sidebar
            view_sidebar(
                &self.user,
                &self.type_counts,
                &self.repo_counts,
                self.filters.selected_type,
                self.filters.selected_repo.as_deref(),
                self.all_notifications.len(),
            ),
            // Main content area
            self.view_main_content()
        ]
        .height(Fill)
        .into()
    }

    fn view_main_content(&self) -> Element<'_, NotificationMessage> {
        column![self.view_content_header(), self.view_content()]
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn view_content_header(&self) -> Element<'_, NotificationMessage> {
        let unread_count = self
            .filtered_notifications
            .iter()
            .filter(|n| n.unread)
            .count();

        let title = text("Notifications").size(18).color(theme::TEXT_PRIMARY);

        let sync_status: Element<'_, NotificationMessage> = if self.is_loading {
            row![
                icons::icon_refresh(11.0, theme::TEXT_MUTED),
                Space::new().width(4),
                text("Syncing...").size(11).color(theme::TEXT_MUTED),
            ]
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                icons::icon_check(11.0, theme::ACCENT_GREEN),
                Space::new().width(4),
                text("Synced").size(11).color(theme::ACCENT_GREEN),
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
            .size(11),
        )
        .style(theme::secondary_button)
        .padding([4, 8])
        .on_press(NotificationMessage::ToggleShowAll);

        let mark_all_btn = if unread_count > 0 {
            button(
                row![
                    icons::icon_check(11.0, theme::TEXT_SECONDARY),
                    Space::new().width(4),
                    text("Mark all read").size(11),
                ]
                .align_y(Alignment::Center),
            )
            .style(theme::ghost_button)
            .padding([4, 8])
            .on_press(NotificationMessage::MarkAllAsRead)
        } else {
            button(
                row![
                    icons::icon_check(11.0, theme::TEXT_MUTED),
                    Space::new().width(4),
                    text("Mark all read").size(11).color(theme::TEXT_MUTED),
                ]
                .align_y(Alignment::Center),
            )
            .style(theme::ghost_button)
            .padding([4, 8])
        };

        let refresh_btn = button(icons::icon_refresh(14.0, theme::TEXT_SECONDARY))
            .style(theme::ghost_button)
            .padding(6)
            .on_press(NotificationMessage::Refresh);

        let header_row = row![
            title,
            Space::new().width(12),
            sync_status,
            Space::new().width(Fill),
            filter_btn,
            Space::new().width(4),
            mark_all_btn,
            Space::new().width(4),
            refresh_btn,
        ]
        .align_y(Alignment::Center)
        .padding([12, 16]);

        container(header_row)
            .width(Fill)
            .style(theme::header)
            .into()
    }

    fn view_content(&self) -> Element<'_, NotificationMessage> {
        if self.is_loading && self.all_notifications.is_empty() {
            return view_loading();
        }

        if let Some(ref error) = self.error_message {
            return view_error(error);
        }

        if self.filtered_notifications.is_empty() {
            return view_empty(self.filters.show_all);
        }

        // Build content with groups
        let mut content = column![].spacing(8).padding([8, 8]);

        for (group_idx, group) in self.groups.iter().enumerate() {
            if group.notifications.is_empty() {
                continue;
            }

            content = content.push(view_group_header(group, group_idx));

            if group.is_expanded {
                content = content.push(view_group_items(group));
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
