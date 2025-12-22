//! Notifications screen - main notification list view.
//!
//! Layout: Sidebar | Main Content
//! - Sidebar: Types filter, Repositories filter, User info
//! - Main: Content header + notification list

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Color, Element, Fill, Task};

use crate::github::{GitHubClient, GitHubError, NotificationView, SubjectType, UserInfo};
use crate::settings::IconTheme;
use crate::ui::screens::settings::rule_engine::NotificationRuleSet;
use crate::ui::{icons, theme, window_state};

use super::group::{view_group_header, view_group_items};
use super::helper::{
    api_url_to_web_url, apply_filters, count_by_repo, count_by_type, group_processed_notifications,
    process_with_rules, FilterSettings, NotificationGroup, ProcessedNotification,
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
    OpenRuleEngine,
    // Account switching (handled by app.rs)
    // Account switching (handled by app.rs)
    SwitchAccount(String),
    TogglePowerMode,
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
        use crate::ui::screens::settings::rule_engine::RuleAction;

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

    fn rebuild_groups(&mut self) {
        // Recompute cached counts from all notifications
        self.type_counts = count_by_type(&self.all_notifications);
        self.repo_counts = count_by_repo(&self.all_notifications);
        // Apply filters (type, repo, read status)
        self.filtered_notifications = apply_filters(&self.all_notifications, &self.filters);
        // Process through rule engine (applies actions, filters hidden)
        self.processed_notifications =
            process_with_rules(&self.filtered_notifications, &self.rules);

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
    /// Respects rule engine: Silent/Hide actions suppress desktop notifications.
    fn send_desktop_notifications(&self, notifications: &[NotificationView]) {
        use crate::ui::screens::settings::rule_engine::{RuleAction, RuleEngine};

        eprintln!(
            "[DEBUG] send_desktop_notifications called with {} notifications",
            notifications.len()
        );

        // Process through rule engine to determine which should trigger desktop notifications
        let engine = RuleEngine::new(self.rules.clone());
        let now = chrono::Local::now();

        // Find new or updated unread notifications that should trigger desktop notifications
        // A notification is "new" if:
        // 1. We've never seen this ID before, OR
        // 2. The updated_at timestamp is newer than what we recorded
        // AND:
        // 3. The rule action is Show or Priority (not Silent or Hide)
        let new_notifications: Vec<_> = notifications
            .iter()
            .filter(|n| {
                if !n.unread {
                    return false;
                }

                // Check if notification is new/updated
                let is_new = match self.seen_notification_timestamps.get(&n.id) {
                    None => true,                                 // Never seen this ID
                    Some(last_seen) => n.updated_at > *last_seen, // Updated since last seen
                };

                if !is_new {
                    return false;
                }

                // Check rule engine for action
                // Use the reason's label for matching (e.g., "Mentioned", "Commented")
                let reason_label = n.reason.label();
                let (action, _) = engine.evaluate_detailed(
                    reason_label,
                    Some(n.repo_owner()),
                    Some(&n.account),
                    &now,
                );

                // Only show desktop notification for Show and Priority actions
                // Priority always triggers notification (that's the point!)
                matches!(action, RuleAction::Show | RuleAction::Priority)
            })
            .collect();

        // Separate priority notifications (they should always be prominent)
        let priority_notifications: Vec<_> = new_notifications
            .iter()
            .filter(|n| {
                let reason_label = n.reason.label();
                let (action, _) = engine.evaluate_detailed(
                    reason_label,
                    Some(n.repo_owner()),
                    Some(&n.account),
                    &now,
                );
                action == RuleAction::Priority
            })
            .collect();

        eprintln!(
            "[DEBUG] Found {} new notifications ({} priority) (seen count: {})",
            new_notifications.len(),
            priority_notifications.len(),
            self.seen_notification_timestamps.len()
        );

        if new_notifications.is_empty() {
            eprintln!("[DEBUG] No new notifications to show, returning");
            return;
        }

        // If there are priority notifications, show them first/prominently
        if !priority_notifications.is_empty() {
            for notif in &priority_notifications {
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
            if priority_notifications.len() == new_notifications.len() {
                return;
            }
        }

        // Handle remaining (non-priority) notifications
        let regular_notifications: Vec<_> = new_notifications
            .iter()
            .filter(|n| !priority_notifications.contains(n))
            .collect();

        if regular_notifications.is_empty() {
            return;
        }

        // If there's just one regular notification, show it directly
        if regular_notifications.len() == 1 {
            let notif = regular_notifications[0];
            let title = format!("{} - {}", notif.repo_full_name, notif.subject_type);
            let url = notif.url.as_ref().map(|u| api_url_to_web_url(u));

            // Include reason in body for context (e.g., "mentioned", "review requested")
            let body = format!("{}\n{}", notif.title, notif.reason.label());

            eprintln!("[DEBUG] Sending single notification: {:?}", title);
            crate::platform::notify(&title, &body, url.as_deref());
        } else {
            // Multiple notifications - show a summary
            let title = format!("{} new GitHub notifications", regular_notifications.len());
            let body = regular_notifications
                .iter()
                .take(3) // Show first 3
                .map(|n| format!("• {}", n.title))
                .collect::<Vec<_>>()
                .join("\n");

            let body = if regular_notifications.len() > 3 {
                format!("{}\\n...and {} more", body, regular_notifications.len() - 3)
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
        }
    }

    pub fn view(
        &self,
        accounts: &[String],
        icon_theme: IconTheme,
        sidebar_width: f32,
        power_mode: bool,
    ) -> Element<'_, NotificationMessage> {
        row![
            // Sidebar
            view_sidebar(
                &self.user,
                accounts,
                &self.type_counts,
                &self.repo_counts,
                self.filters.selected_type,
                self.filters.selected_repo.as_deref(),
                self.all_notifications.len(),
                icon_theme,
                sidebar_width,
                power_mode,
            ),
            // Main content area
            self.view_main_content(icon_theme, power_mode)
        ]
        .height(Fill)
        .into()
    }

    fn view_main_content(
        &self,
        icon_theme: IconTheme,
        power_mode: bool,
    ) -> Element<'_, NotificationMessage> {
        if power_mode {
            // In power mode, header controls are in top bar
            column![self.view_content(icon_theme, power_mode)]
                .width(Fill)
                .height(Fill)
                .into()
        } else {
            column![
                self.view_content_header(icon_theme),
                self.view_content(icon_theme, power_mode)
            ]
            .width(Fill)
            .height(Fill)
            .into()
        }
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

        // Segmented control for filter selection (Unread | All)
        // Clear visual indication of current state, scales for future filters
        let is_unread_filter = !self.filters.show_all;

        let unread_btn = button(text("Unread").size(12).color(if is_unread_filter {
            p.text_primary
        } else {
            p.text_secondary
        }))
        .style(move |_theme, status| {
            let base_bg = if is_unread_filter {
                p.bg_active
            } else {
                Color::TRANSPARENT
            };
            let bg = match status {
                button::Status::Hovered if !is_unread_filter => p.bg_hover,
                button::Status::Pressed => p.bg_active,
                _ => base_bg,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: if is_unread_filter {
                    p.text_primary
                } else {
                    p.text_secondary
                },
                border: iced::Border {
                    radius: 0.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .padding([6, 12])
        .on_press(NotificationMessage::ToggleShowAll);

        let all_btn = button(text("All").size(12).color(if !is_unread_filter {
            p.text_primary
        } else {
            p.text_secondary
        }))
        .style(move |_theme, status| {
            let base_bg = if !is_unread_filter {
                p.bg_active
            } else {
                Color::TRANSPARENT
            };
            let bg = match status {
                button::Status::Hovered if is_unread_filter => p.bg_hover,
                button::Status::Pressed => p.bg_active,
                _ => base_bg,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: if !is_unread_filter {
                    p.text_primary
                } else {
                    p.text_secondary
                },
                border: iced::Border {
                    radius: 0.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .padding([6, 12])
        .on_press(NotificationMessage::ToggleShowAll);

        // Wrap in container with border
        let filter_segment =
            container(row![unread_btn, all_btn].spacing(0)).style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_control)),
                border: iced::Border {
                    radius: 4.0.into(),
                    color: p.border_subtle,
                    width: 1.0,
                },
                ..Default::default()
            });

        // Mark all read button with improved styling
        let mark_all_btn = if unread_count > 0 {
            button(
                row![
                    icons::icon_check(12.0, p.accent, icon_theme),
                    Space::new().width(6),
                    text("Mark all read").size(12).color(p.text_primary),
                ]
                .align_y(Alignment::Center),
            )
            .style(move |_theme, status| {
                let bg = match status {
                    button::Status::Hovered => p.bg_hover,
                    button::Status::Pressed => p.bg_active,
                    _ => Color::TRANSPARENT,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: p.text_primary,
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .padding([6, 10])
            .on_press(NotificationMessage::MarkAllAsRead)
        } else {
            button(
                row![
                    icons::icon_check(12.0, p.text_muted, icon_theme),
                    Space::new().width(6),
                    text("Mark all read").size(12).color(p.text_muted),
                ]
                .align_y(Alignment::Center),
            )
            .style(move |_theme, _status| button::Style {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                text_color: p.text_muted,
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .padding([6, 10])
        };

        // Refresh button with subtle styling
        let refresh_btn = button(icons::icon_refresh(14.0, p.text_secondary, icon_theme))
            .style(move |_theme, status| {
                let bg = match status {
                    button::Status::Hovered => p.bg_hover,
                    button::Status::Pressed => p.bg_active,
                    _ => Color::TRANSPARENT,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: p.text_secondary,
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .padding(8)
            .on_press(NotificationMessage::Refresh);

        let header_row = row![
            title,
            Space::new().width(12),
            sync_status,
            Space::new().width(Fill),
            filter_segment,
            Space::new().width(12),
            mark_all_btn,
            Space::new().width(4),
            refresh_btn,
        ]
        .align_y(Alignment::Center)
        .padding([14, 16]);

        // Header with subtle bottom border for visual separation
        container(header_row)
            .width(Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_card)),
                border: iced::Border {
                    color: p.border_subtle,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn view_content(
        &self,
        icon_theme: IconTheme,
        power_mode: bool,
    ) -> Element<'_, NotificationMessage> {
        if self.is_loading && self.all_notifications.is_empty() {
            return view_loading();
        }

        if let Some(ref error) = self.error_message {
            return view_error(error, icon_theme);
        }

        // Check processed notifications (after rule filtering) for empty state
        if self.processed_notifications.is_empty() {
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
                content = content.push(view_group_items(group, icon_theme, power_mode));
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
