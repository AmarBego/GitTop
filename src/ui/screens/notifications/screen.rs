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

use super::desktop_notify;
use super::helper::ProcessedNotification;
use super::messages::{FilterMessage, NavigationMessage, NotificationMessage};
use super::processing::ProcessingState;
use crate::github::{GitHubClient, GitHubError, NotificationView, UserInfo};
use crate::settings::IconTheme;
use crate::ui::context::AppContext;
use crate::ui::effects::{AppEffect, NavigateTo, SessionEffect};
use crate::ui::features::bulk_actions::{BulkActionState, update_bulk_action};
use crate::ui::features::notification_details::{
    NotificationDetailsState, update_notification_details,
};
use crate::ui::features::notification_list::{self, ListArgs, NotificationListMessage};
use crate::ui::features::sidebar::{self, SidebarState, SidebarViewArgs, view as view_sidebar};
use crate::ui::features::thread_actions::{ThreadActionState, update_thread_action};
use crate::ui::state;

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
    pub processing: ProcessingState,
    pub sidebar_state: SidebarState,
    pub is_loading: bool,
    pub error_message: Option<String>,

    // === Feature States ===
    pub thread_actions: ThreadActionState,
    pub bulk_actions: BulkActionState,
    pub notification_details: NotificationDetailsState,

    // === Internal State ===
    seen_notification_timestamps: HashMap<String, chrono::DateTime<chrono::Utc>>,

    pub(crate) list_state: notification_list::NotificationListState,
}

impl NotificationsScreen {
    pub fn new(client: GitHubClient, user: UserInfo) -> (Self, Task<NotificationMessage>) {
        let screen = Self {
            client,
            user,
            processing: ProcessingState::new(),
            sidebar_state: SidebarState::default(),
            is_loading: true,
            error_message: None,
            thread_actions: ThreadActionState::new(),
            bulk_actions: BulkActionState::new(),
            notification_details: NotificationDetailsState::new(),
            seen_notification_timestamps: HashMap::new(),
            list_state: notification_list::NotificationListState::new(),
        };
        let task = screen.fetch_notifications();
        (screen, task)
    }

    fn fetch_notifications(&self) -> Task<NotificationMessage> {
        let client = self.client.clone();
        let show_all = self.sidebar_state.show_all;
        let account = self.user.login.clone();
        Task::perform(
            async move { client.get_notification_views(show_all, &account).await },
            NotificationMessage::RefreshComplete,
        )
    }

    pub fn collapse_all_groups(&mut self) {
        for group in &mut self.processing.groups {
            group.is_expanded = false;
        }
    }

    pub fn enter_low_memory_mode(&mut self) {
        self.processing.enter_low_memory_mode();
        self.error_message = None;
        self.error_message = None;
        self.list_state.reset();

        if self.seen_notification_timestamps.len() > 500 {
            self.seen_notification_timestamps.shrink_to_fit();
        }
    }

    pub fn get_cross_account_priority(&self) -> Vec<ProcessedNotification> {
        self.processing.cross_account_priority.clone()
    }

    pub fn set_cross_account_priority(&mut self, priority: Vec<ProcessedNotification>) {
        self.processing.cross_account_priority = priority;
        self.processing
            .rebuild_groups(&mut self.sidebar_state, &self.user.login);
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
                    &mut self.processing.all_notifications,
                    &self.client,
                );
                if result.needs_rebuild {
                    self.processing
                        .rebuild_groups(&mut self.sidebar_state, &self.user.login);
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
                    &mut self.processing.all_notifications,
                    &self.client,
                );
                if result.needs_rebuild {
                    self.processing
                        .rebuild_groups(&mut self.sidebar_state, &self.user.login);
                }
                result.task.map(NotificationMessage::Bulk)
            }

            NotificationMessage::Details(msg) => {
                let task = update_notification_details(
                    &mut self.notification_details,
                    msg,
                    &self.processing.all_notifications,
                    &self.client,
                );
                task.map(NotificationMessage::Details)
            }

            // UI state
            NotificationMessage::Filter(msg) => self.update_filter(msg),
            NotificationMessage::List(msg) => self.update_view(msg),
            NotificationMessage::Sidebar(msg) => self.update_sidebar(msg),
            NotificationMessage::SidebarAction(action) => self.handle_sidebar_action(action),
            NotificationMessage::Navigation(_msg) => Task::none(),
        }
    }

    fn handle_sidebar_action(
        &mut self,
        action: crate::ui::features::sidebar::SidebarAction,
    ) -> Task<NotificationMessage> {
        use crate::ui::features::sidebar::SidebarAction;
        match action {
            SidebarAction::FilterChanged => {
                self.list_state.reset();
                self.processing
                    .rebuild_groups(&mut self.sidebar_state, &self.user.login);
                Task::none()
            }
            SidebarAction::SwitchAccount(u) => Task::done(NotificationMessage::Navigation(
                NavigationMessage::SwitchAccount(u),
            )),
            SidebarAction::OpenSettings => Task::done(NotificationMessage::Navigation(
                NavigationMessage::OpenSettings,
            )),
            SidebarAction::Logout => {
                Task::done(NotificationMessage::Navigation(NavigationMessage::Logout))
            }
        }
    }

    /// Update with effect pattern - returns task and any app-level effect.
    pub fn update_with_effect(
        &mut self,
        message: NotificationMessage,
        ctx: &mut AppContext,
    ) -> (Task<NotificationMessage>, AppEffect) {
        match message {
            // Navigation messages become effects
            NotificationMessage::Navigation(nav) => match nav {
                NavigationMessage::Logout => {
                    (Task::none(), AppEffect::Session(SessionEffect::Logout))
                }
                NavigationMessage::OpenSettings => {
                    (Task::none(), AppEffect::Navigate(NavigateTo::Settings))
                }
                NavigationMessage::OpenRuleEngine => (
                    Task::none(),
                    AppEffect::Navigate(NavigateTo::RuleEngine {
                        from_settings: false,
                    }),
                ),
                NavigationMessage::SwitchAccount(username) => (
                    Task::none(),
                    AppEffect::Session(SessionEffect::SwitchAccount(username)),
                ),
                NavigationMessage::TogglePowerMode => {
                    ctx.settings.power_mode = !ctx.settings.power_mode;
                    ctx.settings.save_silent();
                    self.collapse_all_groups();

                    let task: Task<NotificationMessage> = if ctx.settings.power_mode {
                        state::resize_for_power_mode()
                    } else {
                        Task::none()
                    };
                    (task, AppEffect::None)
                }
            },

            // Other messages handled normally
            other => (self.update(other), AppEffect::None),
        }
    }

    fn update_filter(&mut self, message: FilterMessage) -> Task<NotificationMessage> {
        match message {
            FilterMessage::ToggleShowAll => {
                self.sidebar_state.show_all = !self.sidebar_state.show_all;
                self.list_state.reset();
                self.is_loading = true;
                self.fetch_notifications()
            }
        }
    }

    fn update_view(&mut self, message: NotificationListMessage) -> Task<NotificationMessage> {
        notification_list::update(&mut self.list_state, message, &mut self.processing.groups)
    }

    // === View Composition ===

    pub fn view<'a>(
        &'a self,
        accounts: Vec<String>,
        icon_theme: IconTheme,
        sidebar_width: f32,
        power_mode: bool,
    ) -> Element<'a, NotificationMessage> {
        let total_count = if let Some(ref repo) = self.sidebar_state.selected_repo {
            self.processing
                .all_notifications
                .iter()
                .filter(|n| &n.repo_full_name == repo)
                .count()
        } else {
            self.processing.all_notifications.len()
        };

        let total_repo_count = if let Some(ref selected_type) = self.sidebar_state.selected_type {
            self.processing
                .all_notifications
                .iter()
                .filter(|n| &n.subject_type == selected_type)
                .count()
        } else {
            self.processing.all_notifications.len()
        };

        row![
            view_sidebar(SidebarViewArgs {
                user: &self.user,
                accounts: accounts.clone(),
                type_counts: &self.processing.type_counts,
                repo_counts: &self.processing.repo_counts,
                selected_type: self.sidebar_state.selected_type,
                selected_repo: self.sidebar_state.selected_repo.as_deref(),
                total_count,
                total_repo_count,
                icon_theme,
                width: sidebar_width,
                power_mode,
            })
            .map(NotificationMessage::Sidebar),
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
            iced::widget::column![
                crate::ui::features::bulk_actions::view(
                    &self.bulk_actions,
                    self.processing
                        .filtered_notifications
                        .iter()
                        .map(|n| n.id.clone())
                        .collect(),
                    icon_theme,
                )
                .map(NotificationMessage::Bulk),
                notification_list::view(ListArgs {
                    groups: &self.processing.groups,
                    is_loading: self.is_loading,
                    has_notifications: self
                        .processing
                        .groups
                        .iter()
                        .any(|g| !g.notifications.is_empty()),
                    error_message: self.error_message.as_ref(),
                    filters: &self.sidebar_state,
                    bulk_actions: &self.bulk_actions,
                    list_state: &self.list_state,
                    icon_theme,
                    power_mode,
                })
            ]
            .width(Fill)
            .height(Fill)
            .into()
        } else {
            iced::widget::column![
                super::components::header::view(
                    &self.processing.filtered_notifications,
                    self.is_loading,
                    &self.sidebar_state,
                    icon_theme
                ),
                notification_list::view(ListArgs {
                    groups: &self.processing.groups,
                    is_loading: self.is_loading,
                    has_notifications: self
                        .processing
                        .groups
                        .iter()
                        .any(|g| !g.notifications.is_empty()),
                    error_message: self.error_message.as_ref(),
                    filters: &self.sidebar_state,
                    bulk_actions: &self.bulk_actions,
                    list_state: &self.list_state,
                    icon_theme,
                    power_mode,
                })
            ]
            .width(Fill)
            .height(Fill)
            .into()
        }
    }

    pub fn selected_notification(&self) -> Option<&NotificationView> {
        self.notification_details
            .selected_id
            .as_ref()
            .and_then(|id| {
                self.processing
                    .all_notifications
                    .iter()
                    .find(|n| &n.id == id)
            })
    }

    pub fn selected_details(&self) -> Option<&crate::github::NotificationSubjectDetail> {
        self.notification_details.details.as_ref()
    }

    // === Extracted Logic ===

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

                // Process first to check for desktop notifications
                // We create a temporary engine just for desktop notification check if needed,
                // or we update state and then check.
                // Updating state:
                self.processing.all_notifications = notifications;
                // Rebuild groups will process notifications
                self.processing
                    .rebuild_groups(&mut self.sidebar_state, &self.user.login);

                let is_hidden = state::is_hidden();
                let should_notify = is_hidden || !state::is_focused();

                if should_notify {
                    // Send desktop notifications using processed data
                    desktop_notify::send_desktop_notifications(
                        &self.processing.processed_notifications,
                        &self.seen_notification_timestamps,
                    );
                }

                for n in &self.processing.all_notifications {
                    self.seen_notification_timestamps
                        .insert(n.id.clone(), n.updated_at);
                }
                if self.seen_notification_timestamps.len() > 500 {
                    let current_ids: std::collections::HashSet<_> = self
                        .processing
                        .all_notifications
                        .iter()
                        .map(|n| &n.id)
                        .collect();
                    self.seen_notification_timestamps
                        .retain(|id, _| current_ids.contains(id));
                }

                crate::platform::trim_memory();
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
            }
        }
        Task::none()
    }

    fn update_sidebar(
        &mut self,
        message: crate::ui::features::sidebar::SidebarMessage,
    ) -> Task<NotificationMessage> {
        sidebar::update(&mut self.sidebar_state, message).map(NotificationMessage::SidebarAction)
    }
}
