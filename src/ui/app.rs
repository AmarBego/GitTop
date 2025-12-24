//! Main application state and logic.

use std::time::Duration;

use iced::window::Id as WindowId;
use iced::{Element, Event, Subscription, Task, Theme, event, exit, time, window};

use crate::github::{SessionManager, auth};
use crate::settings::AppSettings;
use crate::tray::{TrayCommand, TrayManager};
use crate::ui::screens::settings::rule_engine::rules::NotificationRuleSet;
use crate::ui::screens::{
    login::{LoginMessage, LoginScreen},
    notifications::NotificationsScreen,
    notifications::messages::{NavigationMessage, NotificationMessage},
    settings::{
        SettingsMessage, SettingsScreen,
        rule_engine::{RuleEngineMessage, RuleEngineScreen},
    },
};
use crate::ui::widgets::power::{details_panel, status_bar, top_bar};
use crate::ui::window_state;

// ============================================================================
// Shared Application Context
// ============================================================================

/// Shared state across all authenticated screens.
pub struct AppContext {
    pub settings: AppSettings,
    pub sessions: SessionManager,
}

impl AppContext {
    /// Create a new context.
    pub fn new(settings: AppSettings, sessions: SessionManager) -> Self {
        Self { settings, sessions }
    }

    /// Clone with updated settings.
    pub fn with_settings(&self, settings: AppSettings) -> Self {
        Self {
            settings,
            sessions: self.sessions.clone(),
        }
    }

    /// Get list of account usernames.
    pub fn account_names(&self) -> Vec<String> {
        self.sessions.usernames().map(String::from).collect()
    }
}

// ============================================================================
// Application State Machine
// ============================================================================

/// Current screen state.
pub enum Screen {
    /// Main notifications screen.
    Notifications(Box<NotificationsScreen>),
    /// Settings screen.
    Settings(SettingsScreen),
    /// Rule Engine screen.
    RuleEngine(Box<RuleEngineScreen>, RuleEngineOrigin),
}

impl Screen {
    fn title(&self) -> String {
        match self {
            Screen::Notifications(screen) => {
                let unread = screen.all_notifications.iter().filter(|n| n.unread).count();
                if unread > 0 {
                    format!("GitTop ({unread} unread)")
                } else {
                    "GitTop".into()
                }
            }
            Screen::Settings(_) => "GitTop - Settings".into(),
            Screen::RuleEngine(_, _) => "GitTop - Rule Engine".into(),
        }
    }
}

/// Application state - which screen we're on.
pub enum App {
    /// Checking for existing auth on startup.
    Loading,
    /// Login screen - no auth.
    Login(LoginScreen),
    /// Authenticated state with shared context.
    Authenticated(Box<Screen>, AppContext),
}

/// Where the Rule Engine was opened from (for back navigation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleEngineOrigin {
    Settings,
    Notifications,
}

/// Top-level application messages.
#[derive(Debug, Clone)]
pub enum Message {
    // -- Lifecycle --
    /// Restore result (returns SessionManager with restored accounts).
    RestoreComplete(SessionManager),
    /// Session restored (for account addition).
    SessionRestored(Result<crate::github::session::Session, String>),

    // -- UI Screens --
    /// Login screen messages.
    Login(LoginMessage),
    /// Notifications screen messages.
    Notifications(NotificationMessage),
    /// Settings screen messages.
    Settings(SettingsMessage),
    /// Rule Engine screen messages.
    RuleEngine(RuleEngineMessage),

    // -- Platform/System --
    /// Periodic refresh tick.
    Tick,
    /// Tray event poll tick.
    TrayPoll,
    /// Window event.
    WindowEvent(WindowId, window::Event),
}

/// Windows reports these values when window is minimized.
const MINIMIZED_POSITION_THRESHOLD: f32 = -10000.0;
const MINIMIZED_SIZE_THRESHOLD: f32 = 100.0;

/// Polling intervals in milliseconds.
const TRAY_POLL_INTERVAL_HIDDEN_MS: u64 = 500;
const TRAY_POLL_INTERVAL_ACTIVE_MS: u64 = 100;

/// Auto-refresh interval for notifications.
const REFRESH_INTERVAL_SECS: u64 = 60;

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            App::Loading,
            Task::perform(
                async {
                    let mut sessions = SessionManager::new();
                    let mut settings = AppSettings::load();
                    let mut failed_accounts = Vec::new();

                    // Restore all accounts, track failures
                    for account in &settings.accounts {
                        if sessions.restore_account(&account.username).await.is_err() {
                            failed_accounts.push(account.username.clone());
                        }
                    }

                    // Remove corrupted/failed accounts from settings
                    if !failed_accounts.is_empty() {
                        for username in failed_accounts {
                            settings.remove_account(&username);
                        }
                        settings.save_silent();
                    }

                    // Set primary: prefer marked active, fallback to first available
                    let primary = settings
                        .accounts
                        .iter()
                        .find(|a| a.is_active)
                        .or_else(|| settings.accounts.first())
                        .map(|a| a.username.clone());

                    if let Some(username) = primary {
                        sessions.set_primary(&username);
                    }

                    sessions
                },
                Message::RestoreComplete,
            ),
        )
    }

    /// Update application state.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        // Handle global messages first
        match &message {
            Message::Tick => return self.handle_tick(),
            Message::TrayPoll => return self.handle_tray_poll(),
            Message::WindowEvent(id, event) => return self.handle_window_event(*id, event.clone()),
            _ => {}
        }

        // Dispatch to state-specific handlers
        match self {
            App::Loading => self.update_loading(message),
            App::Login(_) => self.update_login(message),
            App::Authenticated(boxed_screen, _) => match &mut **boxed_screen {
                Screen::Notifications(_) => self.update_notifications(message),
                Screen::Settings(_) => self.update_settings(message),
                Screen::RuleEngine(_, _) => self.update_rule_engine(message),
            },
        }
    }

    // ========================================================================
    // State-Specific Update Handlers
    // ========================================================================

    fn update_loading(&mut self, message: Message) -> Task<Message> {
        if let Message::RestoreComplete(sessions) = message {
            if let Some(session) = sessions.primary() {
                let mut settings = AppSettings::load();
                settings.set_active_account(&session.username);
                settings.save_silent();
                settings.apply_theme();

                let (notif_screen, task) =
                    NotificationsScreen::new(session.client.clone(), session.user.clone());
                let ctx = AppContext::new(settings, sessions);
                *self = App::Authenticated(
                    Box::new(Screen::Notifications(Box::new(notif_screen))),
                    ctx,
                );
                return task.map(Message::Notifications);
            } else {
                let settings = AppSettings::load();
                settings.apply_theme();
                *self = App::Login(LoginScreen::new());
                crate::platform::trim_memory();
            }
        }
        Task::none()
    }

    fn update_login(&mut self, message: Message) -> Task<Message> {
        let App::Login(screen) = self else {
            return Task::none();
        };

        let Message::Login(login_msg) = message else {
            return Task::none();
        };

        match login_msg {
            LoginMessage::LoginSuccess(client, user) => {
                let mut settings = AppSettings::load();
                settings.set_active_account(&user.login);
                settings.save_silent();
                settings.apply_theme();

                let token = client.token().to_string();
                let _ = crate::github::keyring::save_token(&user.login, &token);

                // Create session and add to manager so navigation works
                let mut sessions = SessionManager::new();
                sessions.add_session(crate::github::session::Session {
                    username: user.login.clone(),
                    client: client.clone(),
                    user: user.clone(),
                });
                let (notif_screen, task) = NotificationsScreen::new(client, user);
                let ctx = AppContext::new(settings, sessions);
                *self = App::Authenticated(
                    Box::new(Screen::Notifications(Box::new(notif_screen))),
                    ctx,
                );
                task.map(Message::Notifications)
            }
            other => screen.update(other).map(Message::Login),
        }
    }

    fn update_notifications(&mut self, message: Message) -> Task<Message> {
        let App::Authenticated(boxed_screen, ctx) = self else {
            return Task::none();
        };

        let Screen::Notifications(screen) = &mut **boxed_screen else {
            return Task::none();
        };

        let Message::Notifications(notif_msg) = message else {
            return Task::none();
        };

        match notif_msg {
            NotificationMessage::Navigation(NavigationMessage::Logout) => {
                // Remove only the current account, not all accounts
                let current_username = ctx.sessions.primary().map(|s| s.username.clone());

                if let Some(username) = current_username {
                    let _ = ctx.sessions.remove_account(&username);
                    ctx.settings.remove_account(&username);
                    ctx.settings.save_silent();
                }

                // If no accounts left, go to login
                if ctx.sessions.primary().is_none() {
                    let _ = auth::delete_token();
                    *self = App::Login(LoginScreen::new());
                    return Task::none();
                }

                // Switch to next available account
                if let Some(session) = ctx.sessions.primary() {
                    ctx.settings.set_active_account(&session.username);
                    ctx.settings.save_silent();

                    let (notif_screen, task) =
                        NotificationsScreen::new(session.client.clone(), session.user.clone());
                    let settings = ctx.settings.clone();
                    *self = App::Authenticated(
                        Box::new(Screen::Notifications(Box::new(notif_screen))),
                        ctx.with_settings(settings),
                    );
                    return task.map(Message::Notifications);
                }
                Task::none()
            }

            NotificationMessage::Navigation(NavigationMessage::OpenSettings) => {
                self.go_to_settings()
            }

            NotificationMessage::Navigation(NavigationMessage::OpenRuleEngine) => {
                self.go_to_rule_engine(RuleEngineOrigin::Notifications)
            }

            NotificationMessage::Navigation(NavigationMessage::SwitchAccount(username)) => {
                // Skip if already on this account
                if ctx
                    .sessions
                    .primary()
                    .is_some_and(|s| s.username == username)
                {
                    return Task::none();
                }

                // Preserve cross-account priority notifications
                let cross_account_priority = screen.get_cross_account_priority();
                ctx.sessions.set_primary(&username);

                // Persist the active account preference
                ctx.settings.set_active_account(&username);
                ctx.settings.save_silent();

                let Some(session) = ctx.sessions.primary() else {
                    return Task::none();
                };

                let (mut notif_screen, task) =
                    NotificationsScreen::new(session.client.clone(), session.user.clone());
                notif_screen.set_cross_account_priority(cross_account_priority);

                let settings = ctx.settings.clone();
                *self = App::Authenticated(
                    Box::new(Screen::Notifications(Box::new(notif_screen))),
                    ctx.with_settings(settings),
                );
                task.map(Message::Notifications)
            }

            NotificationMessage::Navigation(NavigationMessage::TogglePowerMode) => {
                ctx.settings.power_mode = !ctx.settings.power_mode;
                ctx.settings.save_silent();
                screen.collapse_all_groups();

                ctx.settings
                    .power_mode
                    .then(window_state::resize_for_power_mode)
                    .unwrap_or_else(Task::none)
            }

            other => screen.update(other).map(Message::Notifications),
        }
    }

    fn update_settings(&mut self, message: Message) -> Task<Message> {
        let App::Authenticated(boxed_screen, ctx) = self else {
            return Task::none();
        };

        let Screen::Settings(screen) = &mut **boxed_screen else {
            return Task::none();
        };

        // Handle session restoration
        if let Message::SessionRestored(result) = message {
            match result {
                Ok(session) => {
                    let username = session.username.clone();
                    ctx.sessions.add_session(session);
                    screen.settings.set_active_account(&username);
                    screen.settings.save_silent();
                }
                Err(e) => {
                    eprintln!("Failed to restore session: {}", e);
                    screen.accounts_state.status =
                        crate::ui::screens::settings::tabs::accounts::SubmissionStatus::Error(
                            format!("Failed to restore session: {}", e),
                        );
                }
            }
            return Task::none();
        }

        let Message::Settings(settings_msg) = message else {
            return Task::none();
        };

        match &settings_msg {
            SettingsMessage::Back => {
                return self.go_to_notifications();
            }

            SettingsMessage::OpenRuleEngine => {
                return self.go_to_rule_engine(RuleEngineOrigin::Settings);
            }

            SettingsMessage::TogglePowerMode(enabled) => {
                let enabling = *enabled;
                let settings_task = screen.update(settings_msg).map(Message::Settings);
                if enabling {
                    return Task::batch([window_state::resize_for_power_mode(), settings_task]);
                }
                return settings_task;
            }

            SettingsMessage::RemoveAccount(username) => {
                // Remove from active sessions
                let _ = ctx.sessions.remove_account(username);
                // Also remove from ctx.settings to keep them in sync
                ctx.settings.remove_account(username);
                ctx.settings.save_silent();

                // If no accounts left, logout
                if ctx.sessions.primary().is_none() {
                    let _ = auth::delete_token();
                    *self = App::Login(LoginScreen::new());
                    return Task::none();
                }

                // If we still have an account, ensure settings match the new primary
                if let Some(primary) = ctx.sessions.primary() {
                    ctx.settings.set_active_account(&primary.username);
                    ctx.settings.save_silent();
                }
                // Keep screen.settings in sync with ctx.settings
                screen.settings = ctx.settings.clone();
                return Task::none();
            }

            SettingsMessage::TokenValidated(Ok(username)) => {
                let username = username.clone();
                // First update the screen to show success status
                let screen_task = screen.update(settings_msg).map(Message::Settings);

                // Then start session restoration in parallel
                let restore_task = Task::perform(
                    async move {
                        let token = crate::github::keyring::load_token(&username)
                            .map_err(|e| e.to_string())?
                            .ok_or_else(|| "Token not found in keyring".to_string())?;

                        let client =
                            crate::github::GitHubClient::new(&token).map_err(|e| e.to_string())?;
                        let user = client
                            .get_authenticated_user()
                            .await
                            .map_err(|e| e.to_string())?;

                        Ok(crate::github::session::Session {
                            username,
                            client,
                            user,
                        })
                    },
                    Message::SessionRestored,
                );
                return Task::batch([screen_task, restore_task]);
            }

            _ => {}
        }

        screen.update(settings_msg).map(Message::Settings)
    }

    fn update_rule_engine(&mut self, message: Message) -> Task<Message> {
        let App::Authenticated(boxed_screen, ctx) = self else {
            return Task::none();
        };

        let Screen::RuleEngine(screen, origin) = &mut **boxed_screen else {
            return Task::none();
        };

        let Message::RuleEngine(rule_msg) = message else {
            return Task::none();
        };

        if let RuleEngineMessage::Back = rule_msg {
            let settings = ctx.settings.clone();
            match origin {
                RuleEngineOrigin::Settings => {
                    let settings_screen = SettingsScreen::new(settings.clone());
                    *self = App::Authenticated(
                        Box::new(Screen::Settings(settings_screen)),
                        ctx.with_settings(settings),
                    );
                }
                RuleEngineOrigin::Notifications => {
                    if let Some(session) = ctx.sessions.primary() {
                        let (notif_screen, task) =
                            NotificationsScreen::new(session.client.clone(), session.user.clone());
                        *self = App::Authenticated(
                            Box::new(Screen::Notifications(Box::new(notif_screen))),
                            ctx.with_settings(settings),
                        );
                        return task.map(Message::Notifications);
                    }
                }
            }
            return Task::none();
        }

        screen.update(rule_msg).map(Message::RuleEngine)
    }

    // ========================================================================
    // Platform/System Event Handlers
    // ========================================================================

    fn handle_tick(&mut self) -> Task<Message> {
        let App::Authenticated(boxed_screen, _) = self else {
            return Task::none();
        };
        let Screen::Notifications(screen) = &mut **boxed_screen else {
            return Task::none();
        };
        if screen.is_loading {
            return Task::none();
        }
        screen
            .update(NotificationMessage::Refresh)
            .map(Message::Notifications)
    }

    fn handle_tray_poll(&mut self) -> Task<Message> {
        let Some(cmd) = TrayManager::poll_global_events() else {
            return Task::none();
        };

        match cmd {
            TrayCommand::ShowWindow => {
                let was_hidden = window_state::restore_from_hidden();

                let window_task = window_state::get_window_id()
                    .map(|id| {
                        Task::batch([
                            window::set_mode(id, window::Mode::Windowed),
                            window::gain_focus(id),
                        ])
                    })
                    .unwrap_or_else(Task::none);

                let refresh_task = was_hidden
                    .then(|| self.notification_screen_mut())
                    .flatten()
                    .map(|screen| {
                        screen
                            .update(NotificationMessage::Refresh)
                            .map(Message::Notifications)
                    })
                    .unwrap_or_else(Task::none);

                Task::batch([window_task, refresh_task])
            }
            TrayCommand::Quit => exit(),
        }
    }

    fn handle_window_event(&mut self, id: WindowId, event: window::Event) -> Task<Message> {
        window_state::set_window_id(id);

        match event {
            window::Event::CloseRequested => {
                let minimize_to_tray = self
                    .current_settings()
                    .map(|s| s.minimize_to_tray)
                    .unwrap_or(false);

                if minimize_to_tray {
                    self.enter_tray_mode(id)
                } else {
                    exit()
                }
            }

            window::Event::Moved(position) => {
                let valid = position.x > MINIMIZED_POSITION_THRESHOLD
                    && position.y > MINIMIZED_POSITION_THRESHOLD;

                if let Some(s) = valid.then(|| self.settings_mut()).flatten() {
                    s.window_x = Some(position.x as i32);
                    s.window_y = Some(position.y as i32);
                    s.save_silent();
                }
                Task::none()
            }

            window::Event::Resized(size) => {
                let valid =
                    size.width > MINIMIZED_SIZE_THRESHOLD && size.height > MINIMIZED_SIZE_THRESHOLD;

                if let Some(s) = valid.then(|| self.settings_mut()).flatten() {
                    s.window_width = size.width;
                    s.window_height = size.height;
                    s.save_silent();
                }
                Task::none()
            }

            _ => Task::none(),
        }
    }

    // ========================================================================
    // Navigation Helpers
    // ========================================================================

    /// Get current settings, preferring SettingsScreen's copy if active.
    fn current_settings(&self) -> Option<&AppSettings> {
        let App::Authenticated(screen, ctx) = self else {
            return None;
        };
        Some(match &**screen {
            Screen::Settings(s) => &s.settings,
            _ => &ctx.settings,
        })
    }

    /// Navigate to the notifications screen.
    fn go_to_notifications(&mut self) -> Task<Message> {
        let settings = self
            .current_settings()
            .cloned()
            .unwrap_or_else(AppSettings::load);

        let App::Authenticated(_, ctx) = self else {
            return Task::none();
        };

        let Some(session) = ctx.sessions.primary() else {
            return Task::none();
        };

        let (notif_screen, task) =
            NotificationsScreen::new(session.client.clone(), session.user.clone());
        *self = App::Authenticated(
            Box::new(Screen::Notifications(Box::new(notif_screen))),
            ctx.with_settings(settings),
        );
        task.map(Message::Notifications)
    }

    /// Navigate to the settings screen.
    fn go_to_settings(&mut self) -> Task<Message> {
        let App::Authenticated(_, ctx) = self else {
            return Task::none();
        };

        let settings = ctx.settings.clone();
        let settings_screen = SettingsScreen::new(settings.clone());
        *self = App::Authenticated(
            Box::new(Screen::Settings(settings_screen)),
            ctx.with_settings(settings),
        );
        Task::none()
    }

    /// Navigate to the rule engine screen.
    fn go_to_rule_engine(&mut self, origin: RuleEngineOrigin) -> Task<Message> {
        let settings = self
            .current_settings()
            .cloned()
            .unwrap_or_else(AppSettings::load);

        let App::Authenticated(_, ctx) = self else {
            return Task::none();
        };

        let rules = NotificationRuleSet::load();
        let rule_engine_screen = RuleEngineScreen::new(rules, settings.clone());
        *self = App::Authenticated(
            Box::new(Screen::RuleEngine(Box::new(rule_engine_screen), origin)),
            ctx.with_settings(settings),
        );
        Task::none()
    }

    // ========================================================================
    // Tray/Memory Management
    // ========================================================================

    /// Enter tray mode: hide window and free memory.
    fn enter_tray_mode(&mut self, window_id: WindowId) -> Task<Message> {
        window_state::set_hidden(true);

        if let Some(screen) = self.notification_screen_mut() {
            screen.enter_low_memory_mode();
        }

        crate::platform::trim_memory();
        window::set_mode(window_id, window::Mode::Hidden)
    }

    /// Get mutable reference to settings.
    fn settings_mut(&mut self) -> Option<&mut AppSettings> {
        let App::Authenticated(boxed_screen, ctx) = self else {
            return None;
        };
        Some(match &mut **boxed_screen {
            Screen::Settings(screen) => &mut screen.settings,
            _ => &mut ctx.settings,
        })
    }

    /// Get mutable reference to notifications screen if active.
    fn notification_screen_mut(&mut self) -> Option<&mut NotificationsScreen> {
        let App::Authenticated(boxed, _) = self else {
            return None;
        };
        let Screen::Notifications(s) = &mut **boxed else {
            return None;
        };
        Some(s)
    }

    // ========================================================================
    // View Rendering
    // ========================================================================

    /// Render the current view.
    pub fn view(&self) -> Element<'_, Message> {
        match self {
            App::Loading => self.view_loading(),
            App::Login(screen) => screen.view().map(Message::Login),
            App::Authenticated(boxed_screen, ctx) => match &**boxed_screen {
                Screen::Notifications(notif_screen) => {
                    let accounts = ctx.account_names();

                    if ctx.settings.power_mode {
                        self.view_power_mode(notif_screen, &ctx.settings, accounts)
                    } else {
                        notif_screen
                            .view(
                                accounts,
                                ctx.settings.icon_theme,
                                ctx.settings.sidebar_width,
                                false,
                            )
                            .map(Message::Notifications)
                    }
                }
                Screen::Settings(settings_screen) => settings_screen.view().map(Message::Settings),
                Screen::RuleEngine(rule_screen, _) => rule_screen.view().map(Message::RuleEngine),
            },
        }
    }

    /// Render the loading screen.
    fn view_loading(&self) -> Element<'_, Message> {
        use crate::ui::theme;
        use iced::widget::{container, text};

        container(text("Loading...").size(14))
            .width(iced::Fill)
            .height(iced::Fill)
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center)
            .style(theme::app_container)
            .into()
    }

    /// Render the power mode layout: TopBar + [Sidebar | Content | Details] + StatusBar.
    fn view_power_mode<'a>(
        &self,
        screen: &'a NotificationsScreen,
        settings: &AppSettings,
        accounts: Vec<String>,
    ) -> Element<'a, Message> {
        use iced::widget::{column, row};

        let content = screen.view(
            accounts.clone(),
            settings.icon_theme,
            settings.sidebar_width,
            true,
        );

        let main_area: iced::Element<NotificationMessage> = if settings.show_details_panel {
            row![
                content,
                details_panel::view_details_panel(
                    screen.selected_notification(),
                    screen.selected_details(),
                    screen.is_loading_details,
                    settings.icon_theme
                )
            ]
            .height(iced::Fill)
            .into()
        } else {
            row![content].height(iced::Fill).into()
        };

        // Build account info for top bar
        let account_infos: Vec<top_bar::AccountInfo> = accounts
            .iter()
            .map(|username| top_bar::AccountInfo {
                username: username.clone(),
            })
            .collect();

        let unread_count = screen
            .filtered_notifications
            .iter()
            .filter(|n| n.unread)
            .count();

        let power_layout: iced::Element<NotificationMessage> = column![
            top_bar::view_top_bar(
                &screen.user,
                account_infos,
                screen.is_loading,
                unread_count,
                screen.filters.show_all,
                screen.bulk_mode,
                settings.icon_theme
            ),
            main_area,
            status_bar::view_status_bar(settings.icon_theme)
        ]
        .into();

        power_layout.map(Message::Notifications)
    }

    /// Window title.
    pub fn title(&self) -> String {
        match self {
            App::Loading => "GitTop".into(),
            App::Login(_) => "GitTop - Sign In".into(),
            App::Authenticated(screen, _) => (**screen).title(),
        }
    }

    /// Application theme - using a dark theme.
    pub fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }

    /// Subscriptions - periodic refresh, tray events, and window events.
    pub fn subscription(&self) -> Subscription<Message> {
        let is_hidden = window_state::is_hidden();

        let tray_interval = if is_hidden {
            TRAY_POLL_INTERVAL_HIDDEN_MS
        } else {
            TRAY_POLL_INTERVAL_ACTIVE_MS
        };

        let tray_sub = time::every(Duration::from_millis(tray_interval)).map(|_| Message::TrayPoll);

        let window_sub = event::listen_with(|event, _status, id| match event {
            Event::Window(e) => Some(Message::WindowEvent(id, e)),
            _ => None,
        });

        let on_notifications = matches!(
            self,
            App::Authenticated(screen, _) if matches!(&**screen, Screen::Notifications(_))
        );

        let tick_sub = on_notifications.then(|| {
            time::every(Duration::from_secs(REFRESH_INTERVAL_SECS)).map(|_| Message::Tick)
        });

        let subs: Vec<_> = tick_sub.into_iter().chain([tray_sub, window_sub]).collect();
        Subscription::batch(subs)
    }
}
