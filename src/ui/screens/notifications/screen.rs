//! Notifications screen - main notification list view.
//!
//! Layout: Sidebar | Main Content
//! - Sidebar: Types filter, Repositories filter, User info
//! - Main: Content header + notification list
//!
//! Architecture:
//! - Uses `NotificationEngine` for centralized rule evaluation (single pass)
//! - `rebuild_groups()` operates on already-processed notifications
//! - `send_desktop_notifications()` uses the same processed data

use iced::widget::row;
use iced::{Element, Fill, Task};

use crate::github::{GitHubClient, GitHubError, NotificationView, SubjectType, UserInfo};
use crate::settings::IconTheme;
use crate::ui::screens::settings::rule_engine::{NotificationRuleSet, RuleAction};
use crate::ui::window_state;

use super::engine::{DesktopNotificationBatch, NotificationEngine};
use super::helper::{
    api_url_to_web_url, apply_filters, count_by_repo, count_by_type, group_processed_notifications,
    FilterSettings, NotificationGroup, ProcessedNotification,
};
use super::messages::NotificationMessage;
use super::view::{view_sidebar, SidebarState};

use std::collections::{HashMap, HashSet};

/// Notifications screen state.
#[derive(Debug, Clone)]
pub struct NotificationsScreen {
    pub client: GitHubClient,
    pub user: UserInfo,
    pub all_notifications: Vec<NotificationView>,
    /// Notifications after filtering (by type, repo, read status).
    pub filtered_notifications: Vec<NotificationView>,
    /// Processed notifications with rule actions applied.
    pub processed_notifications: Vec<ProcessedNotification>,
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
    /// Cached rule set for evaluation.
    rules: NotificationRuleSet,
    /// Priority notifications from ALL accounts (persists across account switches).
    /// These are always shown at the top, regardless of current account.
    cross_account_priority: Vec<ProcessedNotification>,
    /// Virtual scrolling: current scroll offset in pixels.
    pub(crate) scroll_offset: f32,
    /// Virtual scrolling: viewport height in pixels.
    pub(crate) viewport_height: f32,
    /// Currently selected notification ID (for power mode details panel).
    selected_notification_id: Option<String>,
    /// Fetched details for the selected notification.
    selected_notification_details: Option<crate::github::NotificationSubjectDetail>,
    /// Whether we're currently loading details for a selected notification.
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

    /// Collapse all groups to reset view state (e.g. when switching modes).
    pub fn collapse_all_groups(&mut self) {
        for group in &mut self.groups {
            group.is_expanded = false;
        }
    }

    /// Aggressively free all memory for tray mode.
    ///
    /// Clears all cached data while preserving only essential state
    /// (client credentials, user info, seen timestamps for desktop notifications).
    ///
    /// Note: The GPU/OpenGL context cannot be fully destroyed without closing
    /// the window in Iced. However, clearing widget data and resetting scroll
    /// state reduces GPU memory usage by minimizing cached rendering data.
    pub fn enter_low_memory_mode(&mut self) {
        // Clear all notification data
        self.all_notifications = Vec::new();
        self.filtered_notifications = Vec::new();
        self.processed_notifications = Vec::new();
        self.groups = Vec::new();
        self.type_counts = Vec::new();
        self.repo_counts = Vec::new();
        self.cross_account_priority = Vec::new();
        self.error_message = None;

        // Reset scroll state to minimize GPU cached rendering data
        self.scroll_offset = 0.0;
        self.viewport_height = 600.0;

        // Keep seen_notification_timestamps - needed for desktop notification deduplication
        // But shrink it if it's grown too large (keep last 500 entries)
        if self.seen_notification_timestamps.len() > 500 {
            // Keep the HashMap but it will naturally be cleaned up
            // when we next refresh (only current notifications are tracked)
            self.seen_notification_timestamps.shrink_to_fit();
        }
    }

    /// Get the cross-account priority notifications (for passing to new screen on account switch).
    pub fn get_cross_account_priority(&self) -> Vec<ProcessedNotification> {
        self.cross_account_priority.clone()
    }

    /// Set cross-account priority notifications (from previous screen on account switch).
    pub fn set_cross_account_priority(&mut self, priority: Vec<ProcessedNotification>) {
        self.cross_account_priority = priority;
        self.rebuild_groups();
    }

    /// Extract priority notifications from current account and add to cross-account store.
    /// Only tracks UNREAD priority notifications.
    fn update_cross_account_priority(&mut self) {
        // Get unread priority notifications from current account's processed list
        let current_priority: Vec<ProcessedNotification> = self
            .processed_notifications
            .iter()
            .filter(|p| p.action == RuleAction::Priority && p.notification.unread)
            .cloned()
            .collect();

        // Merge with existing cross-account priority (remove duplicates by ID)
        // and remove old entries from the same account (they'll be replaced)
        let current_account = &self.user.login;
        self.cross_account_priority
            .retain(|p| p.notification.account != *current_account);

        // Add current account's unread priority notifications
        self.cross_account_priority.extend(current_priority);
    }

    /// Process all notifications through the rule engine (single pass).
    /// This is called once after fetching, and the results are reused.
    fn process_notifications(&mut self) {
        let engine = NotificationEngine::new(self.rules.clone());

        // Apply filters first (type, repo, read status)
        self.filtered_notifications = apply_filters(&self.all_notifications, &self.filters);

        // Process through rule engine once (applies actions, filters hidden)
        self.processed_notifications = engine.process_all(&self.filtered_notifications);
    }

    fn rebuild_groups(&mut self) {
        // Recompute cached counts from all notifications
        self.type_counts = count_by_type(&self.all_notifications);
        self.repo_counts = count_by_repo(&self.all_notifications);

        // Process notifications through rule engine (single pass)
        self.process_notifications();

        // Update cross-account priority store with current account's priority notifications
        // (only track unread priority notifications)
        self.update_cross_account_priority();

        // Only show cross-account priority in "Unread" mode, not "All"
        let all_processed = if self.filters.show_all {
            // In "All" mode, just show current account's notifications without cross-account priority
            self.processed_notifications.clone()
        } else {
            // In "Unread" mode, merge cross-account priority notifications from other accounts
            let current_account = &self.user.login;
            let other_account_priority: Vec<ProcessedNotification> = self
                .cross_account_priority
                .iter()
                .filter(|p| p.notification.account != *current_account && p.notification.unread)
                .cloned()
                .collect();

            // Combine current account's processed notifications with other accounts' priority
            let mut combined = self.processed_notifications.clone();

            // Add other account priority notifications (they're already marked as Priority action)
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

        // Group by time. Priority group only shown in "Unread" mode (not "All").
        let show_priority_group = !self.filters.show_all;
        self.groups = group_processed_notifications(&all_processed, show_priority_group);
    }

    /// Send desktop notifications for new or updated unread notifications.
    /// Only called when window is hidden in tray.
    ///
    /// Uses the already-processed notifications to avoid re-running rules.
    /// Respects rule engine: Silent/Hide actions suppress desktop notifications.
    fn send_desktop_notifications(&self, processed: &[ProcessedNotification]) {
        eprintln!(
            "[DEBUG] send_desktop_notifications called with {} processed notifications",
            processed.len()
        );

        // Use DesktopNotificationBatch to categorize notifications (uses already-processed data)
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
                "Priority: {} - {}",
                notif.repo_full_name, notif.subject_type
            );
            let url = notif.url.as_ref().map(|u| api_url_to_web_url(u));
            let body = format!("{}\n{}", notif.title, notif.reason.label());
            eprintln!("[DEBUG] Sending priority notification: {:?}", title);
            crate::platform::notify(&title, &body, url.as_deref());
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
            crate::platform::notify(&title, &body, url.as_deref());
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
            crate::platform::notify(&title, &body, None);
        }
    }

    pub fn update(&mut self, message: NotificationMessage) -> Task<NotificationMessage> {
        match message {
            NotificationMessage::TogglePowerMode => Task::none(), // Handled by app.rs
            NotificationMessage::Refresh => {
                self.is_loading = true;
                self.error_message = None;
                self.fetch_notifications()
            }
            NotificationMessage::RefreshComplete(result) => {
                self.is_loading = false;
                match result {
                    Ok(mut notifications) => {
                        // Inject mock notifications if --mock-notifications N was passed
                        let mock_count = crate::MOCK_NOTIFICATION_COUNT
                            .load(std::sync::atomic::Ordering::Relaxed);
                        if mock_count > 0 {
                            let mock = crate::specs::generate_mock_notifications(
                                mock_count,
                                &self.user.login,
                            );
                            eprintln!(
                                "[SPECS] Injecting {} mock notifications for scroll testing",
                                mock_count
                            );
                            notifications.extend(mock);
                        }

                        eprintln!(
                            "[DEBUG] RefreshComplete: got {} notifications",
                            notifications.len()
                        );

                        // === PROCESS ONCE PIPELINE ===
                        // 1. Process all notifications through rule engine (single pass)
                        let engine = NotificationEngine::new(self.rules.clone());
                        let processed_for_desktop = engine.process_all(&notifications);

                        // 2. Check for new notifications to send desktop notifications
                        //    Uses already-processed list (no re-evaluation)
                        let is_hidden = window_state::is_hidden();
                        eprintln!("[DEBUG] is_hidden = {}", is_hidden);

                        if is_hidden {
                            self.send_desktop_notifications(&processed_for_desktop);
                        } else {
                            eprintln!("[DEBUG] Window is visible, skipping desktop notifications");
                        }

                        // 3. Update seen timestamps with current notifications
                        //    Cap size to prevent unbounded memory growth
                        for n in &notifications {
                            self.seen_notification_timestamps
                                .insert(n.id.clone(), n.updated_at);
                        }
                        // Prune old entries if over limit (keep only current + some buffer)
                        if self.seen_notification_timestamps.len() > 500 {
                            // Keep only IDs that are in the current notification set
                            let current_ids: std::collections::HashSet<_> =
                                notifications.iter().map(|n| &n.id).collect();
                            self.seen_notification_timestamps
                                .retain(|id, _| current_ids.contains(id));
                        }

                        // 4. Store data and rebuild groups (will re-process with filters applied)
                        //    If hidden, don't store the data - keep memory minimal
                        if is_hidden {
                            // Don't update all_notifications - keep it empty
                            // Aggressively trim memory after the API call
                            crate::platform::trim_memory();
                        } else {
                            self.all_notifications = notifications;
                            // rebuild_groups() will process with current filters
                            self.rebuild_groups();
                            // Trim memory after render to release wgpu initialization buffers
                            // This reduces baseline memory from ~100MB to ~15MB
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
                // Optimistic update: immediately mark all as read in UI
                for notif in &mut self.all_notifications {
                    notif.unread = false;
                }
                self.rebuild_groups();

                // Fire API call in background
                let client = self.client.clone();
                Task::perform(
                    async move { client.mark_all_as_read().await },
                    NotificationMessage::MarkAllAsReadComplete,
                )
            }
            NotificationMessage::MarkAllAsReadComplete(_result) => {
                // Resync from API
                self.is_loading = true;
                self.fetch_notifications()
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
            NotificationMessage::OpenRuleEngine => {
                // Handled by parent (app.rs)
                Task::none()
            }
            NotificationMessage::SwitchAccount(_) => {
                // Handled by parent (app.rs)
                Task::none()
            }
            NotificationMessage::OnScroll(viewport) => {
                // Update scroll state for virtual scrolling
                self.scroll_offset = viewport.absolute_offset().y;
                self.viewport_height = viewport.bounds().height;
                Task::none()
            }
            NotificationMessage::SelectNotification(id) => {
                // Find the notification
                if let Some(notif) = self.all_notifications.iter().find(|n| n.id == id) {
                    self.selected_notification_id = Some(id.clone());
                    self.selected_notification_details = None;
                    self.is_loading_details = true;

                    // Fetch the details
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
                        move |result| NotificationMessage::SelectComplete(id.clone(), result),
                    )
                } else {
                    Task::none()
                }
            }
            NotificationMessage::SelectComplete(id, result) => {
                // Only update if this is still the selected notification
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
            NotificationMessage::OpenInBrowser => {
                // Open the selected notification's URL in browser
                if let Some(ref id) = self.selected_notification_id {
                    if let Some(notif) = self.all_notifications.iter().find(|n| &n.id == id) {
                        if let Some(ref url) = notif.url {
                            let web_url = api_url_to_web_url(url);
                            let _ = open::that(&web_url);
                        }
                    }
                }
                Task::none()
            }
            // Bulk action handlers
            NotificationMessage::ToggleBulkMode => {
                self.bulk_mode = !self.bulk_mode;
                if !self.bulk_mode {
                    self.selected_ids.clear();
                }
                Task::none()
            }
            NotificationMessage::ToggleSelect(id) => {
                if self.selected_ids.contains(&id) {
                    self.selected_ids.remove(&id);
                } else {
                    self.selected_ids.insert(id);
                }
                Task::none()
            }
            NotificationMessage::SelectAll => {
                // Select all filtered notifications
                for notif in &self.filtered_notifications {
                    self.selected_ids.insert(notif.id.clone());
                }
                Task::none()
            }
            NotificationMessage::ClearSelection => {
                self.selected_ids.clear();
                Task::none()
            }
            NotificationMessage::BulkMarkAsRead => {
                // Optimistic update: immediately mark selected as read in UI
                for id in &self.selected_ids {
                    if let Some(notif) = self.all_notifications.iter_mut().find(|n| &n.id == id) {
                        notif.unread = false;
                    }
                }
                self.rebuild_groups();

                // Fire API calls in background for each selected
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
                    |_| NotificationMessage::Refresh,
                )
            }
            NotificationMessage::BulkMarkAsDone => {
                // Optimistic update: immediately remove selected from UI
                let ids_to_remove: Vec<String> = self.selected_ids.iter().cloned().collect();
                self.all_notifications
                    .retain(|n| !self.selected_ids.contains(&n.id));
                self.rebuild_groups();

                // Fire API calls in background
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
                    |_| NotificationMessage::Refresh,
                )
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        accounts: Vec<String>,
        icon_theme: IconTheme,
        sidebar_width: f32,
        power_mode: bool,
    ) -> Element<'a, NotificationMessage> {
        row![
            // Sidebar
            view_sidebar(SidebarState {
                user: &self.user,
                accounts,
                type_counts: &self.type_counts,
                repo_counts: &self.repo_counts,
                selected_type: self.filters.selected_type,
                selected_repo: self.filters.selected_repo.as_deref(),
                total_count: self.all_notifications.len(),
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

    /// Get the currently selected notification (for details panel).
    pub fn selected_notification(&self) -> Option<&NotificationView> {
        self.selected_notification_id
            .as_ref()
            .and_then(|id| self.all_notifications.iter().find(|n| &n.id == id))
    }

    /// Get the fetched details for the selected notification.
    pub fn selected_details(&self) -> Option<&crate::github::NotificationSubjectDetail> {
        self.selected_notification_details.as_ref()
    }
}
