//! Notifications screen.
//!
//! Architecture:
//! - uses `NotificationEngine` for safe, centralized rule evaluation
//! - `rebuild_groups()` operates on already-processed notifications to avoid redundant work
//! - `send_desktop_notifications()` reuses this data for consistency

use iced::widget::row;
use iced::{Element, Fill, Task};

use crate::github::{GitHubClient, GitHubError, NotificationView, SubjectType, UserInfo};
use crate::settings::IconTheme;
use crate::ui::screens::settings::rule_engine::{NotificationRuleSet, RuleAction};
use crate::ui::window_state;

use super::engine::{DesktopNotificationBatch, NotificationEngine};
use super::helper::{
    FilterSettings, NotificationGroup, ProcessedNotification, api_url_to_web_url, apply_filters,
    count_by_repo, count_by_type, group_processed_notifications,
};
use super::messages::{
    BulkMessage, FilterMessage, NavigationMessage, NotificationMessage, ThreadMessage, ViewMessage,
};
use super::view::{SidebarState, view_sidebar};

use std::collections::{HashMap, HashSet};

/// Notifications screen state.
#[derive(Debug, Clone)]
pub struct NotificationsScreen {
    pub client: GitHubClient,
    pub user: UserInfo,
    pub all_notifications: Vec<NotificationView>,
    pub filtered_notifications: Vec<NotificationView>,
    /// Processed notifications with rule actions applied (Silent, Important, etc).
    pub processed_notifications: Vec<ProcessedNotification>,
    pub groups: Vec<NotificationGroup>,
    pub filters: FilterSettings,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub type_counts: Vec<(SubjectType, usize)>,
    pub repo_counts: Vec<(String, usize)>,
    /// Tracks notification timestamps to detect updates vs new items.
    seen_notification_timestamps: HashMap<String, chrono::DateTime<chrono::Utc>>,
    rules: NotificationRuleSet,
    /// Important notifications from ALL accounts.
    /// Always pinned to top regardless of current account.
    cross_account_priority: Vec<ProcessedNotification>,
    pub(crate) scroll_offset: f32,
    pub(crate) viewport_height: f32,
    selected_notification_id: Option<String>,
    selected_notification_details: Option<crate::github::NotificationSubjectDetail>,
    /// Loading state for the details panel.
    pub is_loading_details: bool,
    /// Set of selected notification IDs for bulk actions (Power Mode only).
    pub selected_ids: HashSet<String>,
    /// Whether bulk selection mode is active.
    pub bulk_mode: bool,
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
            seen_notification_timestamps: HashMap::new(),
            rules: NotificationRuleSet::load(),
            cross_account_priority: Vec::new(),
            scroll_offset: 0.0,
            viewport_height: 600.0, // Default, updated on first scroll
            selected_notification_id: None,
            selected_notification_details: None,
            is_loading_details: false,
            selected_ids: HashSet::new(),
            bulk_mode: false,
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

    /// Aggressively free memory for tray mode.
    ///
    /// Iced doesn't fully destroy the GPU context without closing the window,
    /// but we can minimize VRAM usage by clearing widget data and scroll state.
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

        // needed for desktop notification deduplication
        // shrink if it's grown too large (keep last 500 entries)
        if self.seen_notification_timestamps.len() > 500 {
            self.seen_notification_timestamps.shrink_to_fit();
        }
    }

    /// Get the cross-account priority notifications (for passing to new screen on account switch).
    pub fn get_cross_account_priority(&self) -> Vec<ProcessedNotification> {
        self.cross_account_priority.clone()
    }

    /// Set cross-account priority notifications (from previous screen on account switch).
    pub fn set_cross_account_priority(&mut self, priority: Vec<ProcessedNotification>) {
        eprintln!(
            "[DEBUG] set_cross_account_priority: received {} priority notifications",
            priority.len()
        );
        for p in &priority {
            eprintln!(
                "  - {} from @{} (unread={})",
                p.notification.title, p.notification.account, p.notification.unread
            );
        }
        self.cross_account_priority = priority;
        self.rebuild_groups();
    }

    /// Extract Important notifications from current account and add to cross-account store.
    fn update_cross_account_priority(&mut self) {
        // Get unread Important notifications from current account's processed list
        let current_priority: Vec<ProcessedNotification> = self
            .processed_notifications
            .iter()
            .filter(|p| p.action == RuleAction::Important && p.notification.unread)
            .cloned()
            .collect();

        eprintln!(
            "[DEBUG] update_cross_account_priority: found {} important from current account @{}",
            current_priority.len(),
            self.user.login
        );

        // Merge with existing cross-account priority (remove duplicates by ID)
        // and remove old entries from the same account (they'll be replaced)
        let current_account = &self.user.login;
        self.cross_account_priority
            .retain(|p| p.notification.account != *current_account);

        // Add current account's unread Important notifications
        self.cross_account_priority.extend(current_priority);

        eprintln!(
            "[DEBUG] cross_account_priority now has {} total notifications",
            self.cross_account_priority.len()
        );
    }

    fn process_notifications(&mut self) {
        let engine = NotificationEngine::new(self.rules.clone());

        self.filtered_notifications = apply_filters(&self.all_notifications, &self.filters);
        self.processed_notifications = engine.process_all(&self.filtered_notifications);
    }

    fn rebuild_groups(&mut self) {
        // Dynamic Counts Logic:
        // - type_counts: filtered by selected_repo (show types available in that repo)
        // - repo_counts: filtered by selected_type (show repos containing that type)

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

        // Clear selections if they become invalid (e.g. selected type no longer exists in selected repo)
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

        // Only show cross-account priority in "Unread" mode.
        let all_processed = if self.filters.show_all {
            eprintln!("[DEBUG] rebuild_groups: show_all mode, skipping cross-account priority");
            self.processed_notifications.clone()
        } else {
            let current_account = &self.user.login;
            let other_account_priority: Vec<ProcessedNotification> = self
                .cross_account_priority
                .iter()
                .filter(|p| p.notification.account != *current_account && p.notification.unread)
                .cloned()
                .collect();

            eprintln!(
                "[DEBUG] rebuild_groups: merging {} priority from other accounts (current=@{})",
                other_account_priority.len(),
                current_account
            );
            for p in &other_account_priority {
                eprintln!(
                    "  - Adding: {} from @{}",
                    p.notification.title, p.notification.account
                );
            }

            // Combine current account's processed notifications with other accounts' priority
            let mut combined = self.processed_notifications.clone();

            // Add other account Important notifications (they're already marked as Important action)
            for p in other_account_priority {
                // Avoid duplicates by ID
                if !combined
                    .iter()
                    .any(|existing| existing.notification.id == p.notification.id)
                {
                    combined.push(p);
                }
            }
            combined
        };

        // Preserve expansion state
        let previous_expansion: std::collections::HashMap<String, bool> = self
            .groups
            .iter()
            .map(|g| (g.title.clone(), g.is_expanded))
            .collect();

        // Important group only shown in "Unread" mode.
        let show_priority_group = !self.filters.show_all;
        self.groups = group_processed_notifications(&all_processed, show_priority_group);

        // Restore expansion state for groups that existed before
        for group in &mut self.groups {
            if let Some(&was_expanded) = previous_expansion.get(&group.title) {
                group.is_expanded = was_expanded;
            }
        }

        // Log resulting priority group
        if let Some(priority_group) = self.groups.iter().find(|g| g.is_priority) {
            eprintln!(
                "[DEBUG] rebuild_groups: Important group has {} items",
                priority_group.notifications.len()
            );
        } else {
            eprintln!("[DEBUG] rebuild_groups: No Important group");
        }
    }

    /// Send desktop notifications for new/updated unread items.
    /// Only called when window is hidden.
    fn send_desktop_notifications(&self, processed: &[ProcessedNotification]) {
        eprintln!(
            "[DEBUG] send_desktop_notifications called with {} processed notifications",
            processed.len()
        );

        let batch =
            DesktopNotificationBatch::from_processed(processed, &self.seen_notification_timestamps);

        eprintln!(
            "[DEBUG] Found {} new notifications ({} priority) (seen count: {})",
            batch.total_count(),
            batch.priority.len(),
            self.seen_notification_timestamps.len()
        );

        if batch.is_empty() {
            eprintln!("[DEBUG] No new notifications to show, returning");
            return;
        }

        // Send priority notifications first (always shown prominently)
        for p in &batch.priority {
            let notif = &p.notification;
            let title = format!(
                "Important: {} - {}",
                notif.repo_full_name, notif.subject_type
            );
            let url = notif.url.as_ref().map(|u| api_url_to_web_url(u));
            let body = format!("{}\n{}", notif.title, notif.reason.label());
            eprintln!("[DEBUG] Sending priority notification: {:?}", title);
            if let Err(e) = crate::platform::notify(&title, &body, url.as_deref()) {
                eprintln!("Failed to send notification: {}", e);
            }
        }

        // If all notifications are priority, we're done
        if batch.regular.is_empty() {
            return;
        }

        // Handle regular notifications
        if batch.regular.len() == 1 {
            let notif = &batch.regular[0].notification;
            let title = format!("{} - {}", notif.repo_full_name, notif.subject_type);
            let url = notif.url.as_ref().map(|u| api_url_to_web_url(u));
            let body = format!("{}\n{}", notif.title, notif.reason.label());

            eprintln!("[DEBUG] Sending single notification: {:?}", title);
            if let Err(e) = crate::platform::notify(&title, &body, url.as_deref()) {
                eprintln!("Failed to send notification: {}", e);
            }
        } else {
            // Multiple notifications - show a summary
            let title = format!("{} new GitHub notifications", batch.regular.len());
            let body = batch
                .regular
                .iter()
                .take(3) // Show first 3
                .map(|p| format!("• {}", p.notification.title))
                .collect::<Vec<_>>()
                .join("\n");

            let body = if batch.regular.len() > 3 {
                format!("{}\\n...and {} more", body, batch.regular.len() - 3)
            } else {
                body
            };

            eprintln!("[DEBUG] Sending summary notification: {:?}", title);
            if let Err(e) = crate::platform::notify(&title, &body, None) {
                eprintln!("Failed to send notification: {}", e);
            }
        }

        // Trim memory after sending desktop notifications to prevent accumulation
        crate::platform::trim_memory();
    }

    pub fn update(&mut self, message: NotificationMessage) -> Task<NotificationMessage> {
        match message {
            NotificationMessage::Refresh => {
                self.is_loading = true;
                self.error_message = None;
                self.fetch_notifications()
            }
            NotificationMessage::RefreshComplete(result) => self.handle_refresh_complete(result),
            NotificationMessage::Filter(msg) => self.update_filter(msg),
            NotificationMessage::Thread(msg) => self.update_thread(msg),
            NotificationMessage::Bulk(msg) => self.update_bulk(msg),
            NotificationMessage::View(msg) => self.update_view(msg),
            NotificationMessage::Navigation(msg) => self.update_navigation(msg),
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

    fn update_thread(&mut self, message: ThreadMessage) -> Task<NotificationMessage> {
        match message {
            ThreadMessage::Open(id) => {
                if let Some(notif) = self.all_notifications.iter().find(|n| n.id == id)
                    && let Some(ref url) = notif.url
                {
                    let web_url = api_url_to_web_url(url);
                    let _ = open::that(&web_url);
                }
                let client = self.client.clone();
                let notif_id = id.clone();
                Task::perform(
                    async move { client.mark_as_read(&notif_id).await },
                    move |result| {
                        NotificationMessage::Thread(ThreadMessage::MarkAsReadComplete(
                            id.clone(),
                            result,
                        ))
                    },
                )
            }
            ThreadMessage::MarkAsRead(id) => {
                let client = self.client.clone();
                let notif_id = id.clone();
                Task::perform(
                    async move { client.mark_as_read(&notif_id).await },
                    move |result| {
                        NotificationMessage::Thread(ThreadMessage::MarkAsReadComplete(
                            id.clone(),
                            result,
                        ))
                    },
                )
            }
            ThreadMessage::MarkAsReadComplete(id, result) => {
                if result.is_ok()
                    && let Some(notif) = self.all_notifications.iter_mut().find(|n| n.id == id)
                {
                    notif.unread = false;
                    self.rebuild_groups();
                }
                Task::none()
            }
            ThreadMessage::MarkAllAsRead => {
                for notif in &mut self.all_notifications {
                    notif.unread = false;
                }
                self.rebuild_groups();

                let client = self.client.clone();
                Task::perform(async move { client.mark_all_as_read().await }, |result| {
                    NotificationMessage::Thread(ThreadMessage::MarkAllAsReadComplete(result))
                })
            }
            ThreadMessage::MarkAllAsReadComplete(_result) => {
                self.is_loading = true;
                self.fetch_notifications()
            }
            ThreadMessage::MarkAsDone(id) => {
                let client = self.client.clone();
                let notif_id = id.clone();
                Task::perform(
                    async move { client.mark_thread_as_done(&notif_id).await },
                    move |result| {
                        NotificationMessage::Thread(ThreadMessage::MarkAsDoneComplete(
                            id.clone(),
                            result,
                        ))
                    },
                )
            }
            ThreadMessage::MarkAsDoneComplete(id, result) => {
                if result.is_ok() {
                    self.all_notifications.retain(|n| n.id != id);
                    self.rebuild_groups();
                }
                Task::none()
            }
        }
    }

    fn update_bulk(&mut self, message: BulkMessage) -> Task<NotificationMessage> {
        match message {
            BulkMessage::ToggleMode => {
                self.bulk_mode = !self.bulk_mode;
                if !self.bulk_mode {
                    self.selected_ids.clear();
                    self.selected_ids.shrink_to_fit();
                }
                Task::none()
            }
            BulkMessage::ToggleSelect(id) => {
                if self.selected_ids.contains(&id) {
                    self.selected_ids.remove(&id);
                } else {
                    self.selected_ids.insert(id);
                }
                Task::none()
            }
            BulkMessage::SelectAll => {
                for notif in &self.filtered_notifications {
                    self.selected_ids.insert(notif.id.clone());
                }
                Task::none()
            }
            BulkMessage::Clear => {
                self.selected_ids.clear();
                Task::none()
            }
            BulkMessage::MarkAsRead => {
                for id in &self.selected_ids {
                    if let Some(notif) = self.all_notifications.iter_mut().find(|n| &n.id == id) {
                        notif.unread = false;
                    }
                }
                self.rebuild_groups();

                let client = self.client.clone();
                let ids: Vec<String> = self.selected_ids.iter().cloned().collect();
                self.selected_ids.clear();
                self.bulk_mode = false;

                Task::perform(
                    async move {
                        for id in ids {
                            let _ = client.mark_as_read(&id).await;
                        }
                        Ok::<(), GitHubError>(())
                    },
                    |_| NotificationMessage::Bulk(BulkMessage::Complete),
                )
            }
            BulkMessage::MarkAsDone => {
                let ids_to_remove: Vec<String> = self.selected_ids.iter().cloned().collect();
                self.all_notifications
                    .retain(|n| !self.selected_ids.contains(&n.id));
                self.rebuild_groups();

                let client = self.client.clone();
                self.selected_ids.clear();
                self.bulk_mode = false;

                Task::perform(
                    async move {
                        for id in ids_to_remove {
                            let _ = client.mark_thread_as_done(&id).await;
                        }
                        Ok::<(), GitHubError>(())
                    },
                    |_| NotificationMessage::Bulk(BulkMessage::Complete),
                )
            }
            BulkMessage::Complete => Task::none(),
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
            ViewMessage::SelectNotification(id) => {
                if let Some(notif) = self.all_notifications.iter().find(|n| n.id == id) {
                    self.selected_notification_id = Some(id.clone());
                    self.selected_notification_details = None;
                    self.is_loading_details = true;

                    let client = self.client.clone();
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
                        move |result| {
                            NotificationMessage::View(ViewMessage::SelectComplete(
                                id.clone(),
                                result,
                            ))
                        },
                    )
                } else {
                    Task::none()
                }
            }
            ViewMessage::SelectComplete(id, result) => {
                if self.selected_notification_id.as_ref() == Some(&id) {
                    self.is_loading_details = false;
                    match result {
                        Ok(details) => {
                            self.selected_notification_details = Some(details);
                        }
                        Err(e) => {
                            eprintln!("[ERROR] Failed to fetch notification details: {}", e);
                            self.selected_notification_details = None;
                        }
                    }
                }
                Task::none()
            }
            ViewMessage::OpenInBrowser => {
                if let Some(ref id) = self.selected_notification_id
                    && let Some(notif) = self.all_notifications.iter().find(|n| &n.id == id)
                    && let Some(ref url) = notif.url
                {
                    let web_url = api_url_to_web_url(url);
                    let _ = open::that(&web_url);
                }
                Task::none()
            }
        }
    }

    fn update_navigation(&mut self, message: NavigationMessage) -> Task<NotificationMessage> {
        match message {
            NavigationMessage::Logout => Task::none(),
            NavigationMessage::OpenSettings => Task::none(),
            NavigationMessage::OpenRuleEngine => Task::none(),
            NavigationMessage::SwitchAccount(_) => Task::none(),
            NavigationMessage::TogglePowerMode => Task::none(),
        }
    }

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

                // Show desktop notifications when window is hidden or unfocused
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

    pub fn view<'a>(
        &'a self,
        accounts: Vec<String>,
        icon_theme: IconTheme,
        sidebar_width: f32,
        power_mode: bool,
    ) -> Element<'a, NotificationMessage> {
        // Compute dynamic sidebar totals
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
            // Sidebar
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
            // Main content area
            self.view_main_content(icon_theme, power_mode)
        ]
        .height(Fill)
        .into()
    }

    pub fn selected_notification(&self) -> Option<&NotificationView> {
        self.selected_notification_id
            .as_ref()
            .and_then(|id| self.all_notifications.iter().find(|n| &n.id == id))
    }

    pub fn selected_details(&self) -> Option<&crate::github::NotificationSubjectDetail> {
        self.selected_notification_details.as_ref()
    }
}
