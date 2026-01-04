//! Notifications screen.
//!
//! Architecture:
//! - This screen is a routing and layout shell
//! - Business logic is delegated to features under `ui/features/`
//! - Screen owns: layout composition, feature state storage, message routing
//!
//! ## Temporary Exceptions
//! The following large functions remain in this file pending future extraction:
//! - `rebuild_groups()` - notification grouping logic
//! - `send_desktop_notifications()` - desktop notification logic
//! - `handle_refresh_complete()` - refresh result processing
//!   These are documented as technical debt and should be extracted when the patterns stabilize.

use iced::widget::row;
use iced::{Element, Fill, Task};

use crate::github::{GitHubClient, GitHubError, NotificationView, UserInfo};
use crate::settings::IconTheme;
use crate::ui::features::bulk_actions::{BulkActionState, update_bulk_action};
use crate::ui::features::notification_details::{
    NotificationDetailsState, update_notification_details,
};
use crate::ui::features::thread_actions::{ThreadActionState, update_thread_action};
use crate::ui::screens::settings::rule_engine::{NotificationRuleSet, RuleAction};
use crate::ui::window_state;

use super::engine::{DesktopNotificationBatch, NotificationEngine};
use super::helper::{
    FilterSettings, NotificationGroup, ProcessedNotification, api_url_to_web_url, apply_filters,
    count_by_repo, count_by_type, group_processed_notifications,
};
use super::messages::{FilterMessage, NotificationMessage, ViewMessage};
use super::view::{SidebarState, view_sidebar};

use std::collections::HashMap;

/// Notifications screen state.
///
/// This struct is a layout shell that holds feature states and shared data.
/// It routes messages to features and composes the view.
#[derive(Debug, Clone)]
pub struct NotificationsScreen {
    // === Shared Data ===
    pub client: GitHubClient,
    pub user: UserInfo,
    pub all_notifications: Vec<NotificationView>,
    pub filtered_notifications: Vec<NotificationView>,
    pub processed_notifications: Vec<ProcessedNotification>,
    pub groups: Vec<NotificationGroup>,
    pub filters: FilterSettings,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub type_counts: Vec<(crate::github::SubjectType, usize)>,
    pub repo_counts: Vec<(String, usize)>,

    // === Feature States ===
    pub thread_actions: ThreadActionState,
    pub bulk_actions: BulkActionState,
    pub notification_details: NotificationDetailsState,

    // === Internal State ===
    seen_notification_timestamps: HashMap<String, chrono::DateTime<chrono::Utc>>,
    rules: NotificationRuleSet,
    cross_account_priority: Vec<ProcessedNotification>,
    pub(crate) scroll_offset: f32,
    pub(crate) viewport_height: f32,
}

impl NotificationsScreen {
    pub fn new(client: GitHubClient, user: UserInfo) -> (Self, Task<NotificationMessage>) {
        let screen = Self {
            client,
            user,
            all_notifications: Vec::new(),
            filtered_notifications: Vec::new(),
            processed_notifications: Vec::new(),
            groups: Vec::new(),
            filters: FilterSettings::default(),
            is_loading: true,
            error_message: None,
            type_counts: Vec::new(),
            repo_counts: Vec::new(),
            thread_actions: ThreadActionState::new(),
            bulk_actions: BulkActionState::new(),
            notification_details: NotificationDetailsState::new(),
            seen_notification_timestamps: HashMap::new(),
            rules: NotificationRuleSet::load(),
            cross_account_priority: Vec::new(),
            scroll_offset: 0.0,
            viewport_height: 600.0,
        };
        let task = screen.fetch_notifications();
        (screen, task)
    }

    fn fetch_notifications(&self) -> Task<NotificationMessage> {
        let client = self.client.clone();
        let show_all = self.filters.show_all;
        let account = self.user.login.clone();
        Task::perform(
            async move { client.get_notification_views(show_all, &account).await },
            NotificationMessage::RefreshComplete,
        )
    }

    pub fn collapse_all_groups(&mut self) {
        for group in &mut self.groups {
            group.is_expanded = false;
        }
    }

    pub fn enter_low_memory_mode(&mut self) {
        self.all_notifications = Vec::new();
        self.filtered_notifications = Vec::new();
        self.processed_notifications = Vec::new();
        self.groups = Vec::new();
        self.type_counts = Vec::new();
        self.repo_counts = Vec::new();
        self.cross_account_priority = Vec::new();
        self.error_message = None;
        self.scroll_offset = 0.0;
        self.viewport_height = 600.0;

        if self.seen_notification_timestamps.len() > 500 {
            self.seen_notification_timestamps.shrink_to_fit();
        }
    }

    pub fn get_cross_account_priority(&self) -> Vec<ProcessedNotification> {
        self.cross_account_priority.clone()
    }

    pub fn set_cross_account_priority(&mut self, priority: Vec<ProcessedNotification>) {
        self.cross_account_priority = priority;
        self.rebuild_groups();
    }

    // === Message Routing ===

    pub fn update(&mut self, message: NotificationMessage) -> Task<NotificationMessage> {
        match message {
            // Lifecycle
            NotificationMessage::Refresh => {
                self.is_loading = true;
                self.error_message = None;
                self.fetch_notifications()
            }
            NotificationMessage::RefreshComplete(result) => self.handle_refresh_complete(result),

            // Feature routing
            NotificationMessage::Thread(msg) => {
                let result = update_thread_action(
                    &mut self.thread_actions,
                    msg,
                    &mut self.all_notifications,
                    &self.client,
                );
                if result.needs_rebuild {
                    self.rebuild_groups();
                }
                if result.needs_refresh {
                    self.is_loading = true;
                    return self.fetch_notifications();
                }
                result.task.map(NotificationMessage::Thread)
            }

            NotificationMessage::Bulk(msg) => {
                let result = update_bulk_action(
                    &mut self.bulk_actions,
                    msg,
                    &mut self.all_notifications,
                    &self.client,
                );
                if result.needs_rebuild {
                    self.rebuild_groups();
                }
                result.task.map(NotificationMessage::Bulk)
            }

            NotificationMessage::Details(msg) => {
                let task = update_notification_details(
                    &mut self.notification_details,
                    msg,
                    &self.all_notifications,
                    &self.client,
                );
                task.map(NotificationMessage::Details)
            }

            // UI state
            NotificationMessage::Filter(msg) => self.update_filter(msg),
            NotificationMessage::View(msg) => self.update_view(msg),
            NotificationMessage::Navigation(_msg) => Task::none(),
        }
    }

    fn update_filter(&mut self, message: FilterMessage) -> Task<NotificationMessage> {
        match message {
            FilterMessage::ToggleShowAll => {
                self.filters.show_all = !self.filters.show_all;
                self.scroll_offset = 0.0;
                self.is_loading = true;
                self.fetch_notifications()
            }
            FilterMessage::SelectType(subject_type) => {
                self.filters.selected_type = subject_type;
                self.scroll_offset = 0.0;
                self.rebuild_groups();
                Task::none()
            }
            FilterMessage::SelectRepo(repo) => {
                self.filters.selected_repo = repo;
                self.scroll_offset = 0.0;
                self.rebuild_groups();
                Task::none()
            }
        }
    }

    fn update_view(&mut self, message: ViewMessage) -> Task<NotificationMessage> {
        match message {
            ViewMessage::ToggleGroup(index) => {
                if let Some(group) = self.groups.get_mut(index) {
                    group.is_expanded = !group.is_expanded;
                }
                Task::none()
            }
            ViewMessage::OnScroll(viewport) => {
                self.scroll_offset = viewport.absolute_offset().y;
                self.viewport_height = viewport.bounds().height;
                Task::none()
            }
        }
    }

    // === View Composition ===

    pub fn view<'a>(
        &'a self,
        accounts: Vec<String>,
        icon_theme: IconTheme,
        sidebar_width: f32,
        power_mode: bool,
    ) -> Element<'a, NotificationMessage> {
        let total_count = if let Some(ref repo) = self.filters.selected_repo {
            self.all_notifications
                .iter()
                .filter(|n| &n.repo_full_name == repo)
                .count()
        } else {
            self.all_notifications.len()
        };

        let total_repo_count = if let Some(ref selected_type) = self.filters.selected_type {
            self.all_notifications
                .iter()
                .filter(|n| &n.subject_type == selected_type)
                .count()
        } else {
            self.all_notifications.len()
        };

        row![
            view_sidebar(SidebarState {
                user: &self.user,
                accounts,
                type_counts: &self.type_counts,
                repo_counts: &self.repo_counts,
                selected_type: self.filters.selected_type,
                selected_repo: self.filters.selected_repo.as_deref(),
                total_count,
                total_repo_count,
                icon_theme,
                width: sidebar_width,
                power_mode,
            }),
            self.view_main_content(icon_theme, power_mode)
        ]
        .height(Fill)
        .into()
    }

    pub fn selected_notification(&self) -> Option<&NotificationView> {
        self.notification_details
            .selected_id
            .as_ref()
            .and_then(|id| self.all_notifications.iter().find(|n| &n.id == id))
    }

    pub fn selected_details(&self) -> Option<&crate::github::NotificationSubjectDetail> {
        self.notification_details.details.as_ref()
    }

    // === Temporary Exceptions (to be extracted) ===

    /// Process notifications with rule engine.
    /// TODO: Consider extracting to a notification_processing feature.
    fn process_notifications(&mut self) {
        let engine = NotificationEngine::new(self.rules.clone());
        self.filtered_notifications = apply_filters(&self.all_notifications, &self.filters);
        self.processed_notifications = engine.process_all(&self.filtered_notifications);
    }

    /// Update cross-account priority notifications.
    /// TODO: Consider extracting when cross-account feature scope is clearer.
    fn update_cross_account_priority(&mut self) {
        let current_priority: Vec<ProcessedNotification> = self
            .processed_notifications
            .iter()
            .filter(|p| p.action == RuleAction::Important && p.notification.unread)
            .cloned()
            .collect();

        let current_account = &self.user.login;
        self.cross_account_priority
            .retain(|p| p.notification.account != *current_account);
        self.cross_account_priority.extend(current_priority);
    }

    /// Rebuild notification groups.
    /// TODO: Large function (~80 lines). Extract when grouping logic stabilizes.
    fn rebuild_groups(&mut self) {
        let notifications_for_types: Vec<_> = if let Some(ref repo) = self.filters.selected_repo {
            self.all_notifications
                .iter()
                .filter(|n| &n.repo_full_name == repo)
                .cloned()
                .collect()
        } else {
            self.all_notifications.clone()
        };

        let notifications_for_repos: Vec<_> =
            if let Some(ref selected_type) = self.filters.selected_type {
                self.all_notifications
                    .iter()
                    .filter(|n| &n.subject_type == selected_type)
                    .cloned()
                    .collect()
            } else {
                self.all_notifications.clone()
            };

        self.type_counts = count_by_type(&notifications_for_types);
        self.repo_counts = count_by_repo(&notifications_for_repos);

        if let Some(ref selected_type) = self.filters.selected_type {
            let type_valid = self
                .type_counts
                .iter()
                .any(|(t, c)| t == selected_type && *c > 0);
            if !type_valid {
                self.filters.selected_type = None;
            }
        }
        if let Some(ref selected_repo) = self.filters.selected_repo {
            let repo_valid = self
                .repo_counts
                .iter()
                .any(|(r, c)| r == selected_repo && *c > 0);
            if !repo_valid {
                self.filters.selected_repo = None;
            }
        }

        self.process_notifications();
        self.update_cross_account_priority();

        let all_processed = if self.filters.show_all {
            self.processed_notifications.clone()
        } else {
            let current_account = &self.user.login;
            let other_account_priority: Vec<ProcessedNotification> = self
                .cross_account_priority
                .iter()
                .filter(|p| p.notification.account != *current_account && p.notification.unread)
                .cloned()
                .collect();

            let mut combined = self.processed_notifications.clone();
            for p in other_account_priority {
                if !combined
                    .iter()
                    .any(|existing| existing.notification.id == p.notification.id)
                {
                    combined.push(p);
                }
            }
            combined
        };

        let previous_expansion: std::collections::HashMap<String, bool> = self
            .groups
            .iter()
            .map(|g| (g.title.clone(), g.is_expanded))
            .collect();

        let show_priority_group = !self.filters.show_all;
        self.groups = group_processed_notifications(&all_processed, show_priority_group);

        for group in &mut self.groups {
            if let Some(&was_expanded) = previous_expansion.get(&group.title) {
                group.is_expanded = was_expanded;
            }
        }
    }

    /// Send desktop notifications.
    /// TODO: Extract to desktop_notifications feature when notification batching stabilizes.
    fn send_desktop_notifications(&self, processed: &[ProcessedNotification]) {
        let batch =
            DesktopNotificationBatch::from_processed(processed, &self.seen_notification_timestamps);

        if batch.is_empty() {
            return;
        }

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
                format!("{}\\n...and {} more", body, batch.regular.len() - 3)
            } else {
                body
            };

            if let Err(e) = crate::platform::notify(&title, &body, None) {
                eprintln!("Failed to send notification: {}", e);
            }
        }

        crate::platform::trim_memory();
    }

    /// Handle refresh completion.
    /// TODO: Large function. Consider extracting refresh logic to a feature.
    fn handle_refresh_complete(
        &mut self,
        result: Result<Vec<NotificationView>, GitHubError>,
    ) -> Task<NotificationMessage> {
        self.is_loading = false;
        match result {
            Ok(mut notifications) => {
                let mock_count =
                    crate::MOCK_NOTIFICATION_COUNT.load(std::sync::atomic::Ordering::Relaxed);
                if mock_count > 0 {
                    let mock =
                        crate::specs::generate_mock_notifications(mock_count, &self.user.login);
                    notifications.extend(mock);
                }

                let engine = NotificationEngine::new(self.rules.clone());
                let processed_for_desktop = engine.process_all(&notifications);
                let is_hidden = window_state::is_hidden();

                let should_notify = is_hidden || !window_state::is_focused();
                if should_notify {
                    self.send_desktop_notifications(&processed_for_desktop);
                }

                for n in &notifications {
                    self.seen_notification_timestamps
                        .insert(n.id.clone(), n.updated_at);
                }
                if self.seen_notification_timestamps.len() > 500 {
                    let current_ids: std::collections::HashSet<_> =
                        notifications.iter().map(|n| &n.id).collect();
                    self.seen_notification_timestamps
                        .retain(|id, _| current_ids.contains(id));
                }

                if is_hidden {
                    crate::platform::trim_memory();
                } else {
                    self.all_notifications = notifications;
                    self.rebuild_groups();
                    crate::platform::trim_memory();
                }
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
        Task::none()
    }
}
