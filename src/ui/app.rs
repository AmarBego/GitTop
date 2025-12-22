//! Main application state and logic.

use std::time::Duration;

use iced::window::Id as WindowId;
use iced::{event, exit, time, window, Element, Event, Subscription, Task, Theme};

use crate::github::{AuthManager, SessionManager};
use crate::settings::AppSettings;
use crate::tray::{TrayCommand, TrayManager};
use crate::ui::screens::settings::rule_engine::rules::NotificationRuleSet;
use crate::ui::screens::{
    login::{LoginMessage, LoginScreen},
    notifications::{NotificationMessage, NotificationsScreen},
    settings::{
        rule_engine::{RuleEngineMessage, RuleEngineScreen},
        SettingsMessage, SettingsScreen,
    },
};
use crate::ui::widgets::power::{details_panel, status_bar, top_bar};
use crate::ui::{theme, window_state};

// ============================================================================
// Helper Functions
// ============================================================================

/// Centralized settings save to avoid scattered save calls.
fn save_settings(settings: &AppSettings) {
    let _ = settings.save();
}

// ============================================================================
// Shared Application Context
// ============================================================================

/// Shared state across all authenticated screens.
pub struct AppContext {
    pub settings: AppSettings,
    pub sessions: SessionManager,
}

impl AppContext {
    /// Create a new context with the given sessions.
    pub fn new(settings: AppSettings, sessions: SessionManager) -> Self {
        Self { settings, sessions }
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
    Notifications(NotificationsScreen),
    /// Settings screen.
    Settings(SettingsScreen),
    /// Rule Engine screen.
    RuleEngine(RuleEngineScreen, RuleEngineOrigin),
}

/// Application state - which screen we're on.
pub enum App {
    /// Checking for existing auth on startup.
    Loading,
    /// Login screen - no auth.
    Login(LoginScreen),
    /// Authenticated state with shared context.
    Authenticated(Screen, AppContext),
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

impl App {
    /// Create the app and start the restore task.
    pub fn new() -> (Self, Task<Message>) {
        (
            App::Loading,
            Task::perform(
                async {
                    // Try to restore from old AuthManager first (migration)
                    let mut sessions = SessionManager::new();

                    // Load stored accounts from settings
                    let settings = AppSettings::load();
                    for account in &settings.accounts {
                        // Try to restore each account
                        let _ = sessions.restore_account(&account.username).await;
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
            App::Authenticated(screen, _) => match screen {
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
                save_settings(&settings);

                // Initialize global theme and font scales from settings
                theme::set_theme(settings.theme);
                theme::set_notification_font_scale(settings.notification_font_scale);
                theme::set_sidebar_font_scale(settings.sidebar_font_scale);

                let (notif_screen, task) =
                    NotificationsScreen::new(session.client.clone(), session.user.clone());
                let ctx = AppContext::new(settings, sessions);
                *self = App::Authenticated(Screen::Notifications(notif_screen), ctx);
                return task.map(Message::Notifications);
            } else {
                // Load settings to ensure correct theme on Login screen
                let settings = AppSettings::load();
                theme::set_theme(settings.theme);
                *self = App::Login(LoginScreen::new());
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

        // Check for successful login
        if let LoginMessage::LoginSuccess(client, user) = login_msg.clone() {
            let mut settings = AppSettings::load();
            settings.set_active_account(&user.login);
            save_settings(&settings);

            // Initialize global theme and font scales from settings
            theme::set_theme(settings.theme);
            theme::set_notification_font_scale(settings.notification_font_scale);
            theme::set_sidebar_font_scale(settings.sidebar_font_scale);

            // Save token to new keyring system
            let token = client.token().to_string();
            let username = user.login.clone();
            let _ = crate::github::AccountKeyring::save_token(&username, &token);

            // Create session manager with this account
            let sessions = SessionManager::new();
            // Note: Session will be restored on next app launch from keyring

            let (notif_screen, task) = NotificationsScreen::new(client, user);
            let ctx = AppContext::new(settings, sessions);
            *self = App::Authenticated(Screen::Notifications(notif_screen), ctx);
            return task.map(Message::Notifications);
        }

        screen.update(login_msg).map(Message::Login)
    }

    fn update_notifications(&mut self, message: Message) -> Task<Message> {
        let App::Authenticated(Screen::Notifications(screen), ctx) = self else {
            return Task::none();
        };

        let Message::Notifications(notif_msg) = message else {
            return Task::none();
        };

        match notif_msg {
            NotificationMessage::Logout => {
                // Collect usernames as owned strings first to avoid borrow issues
                let usernames: Vec<String> = ctx.sessions.usernames().map(String::from).collect();
                for username in usernames {
                    let _ = ctx.sessions.remove_account(&username);
                }
                // Also clean up old auth
                let _ = AuthManager::delete_token();
                *self = App::Login(LoginScreen::new());
                Task::none()
            }

            NotificationMessage::OpenSettings => self.go_to_settings(),

            NotificationMessage::OpenRuleEngine => {
                self.go_to_rule_engine(RuleEngineOrigin::Notifications)
            }

            NotificationMessage::SwitchAccount(username) => {
                // Check if we need to switch
                if let Some(current) = ctx.sessions.primary() {
                    if current.username == username {
                        return Task::none();
                    }
                }

                // Preserve cross-account priority notifications before switching
                let cross_account_priority = screen.get_cross_account_priority();

                // Set new active account
                ctx.sessions.set_primary(&username);

                // Navigate to notifications with new session
                if let Some(session) = ctx.sessions.primary() {
                    let (mut notif_screen, task) =
                        NotificationsScreen::new(session.client.clone(), session.user.clone());
                    
                    // Pass cross-account priority notifications to new screen
                    notif_screen.set_cross_account_priority(cross_account_priority);
                    
                    *self = App::Authenticated(Screen::Notifications(notif_screen), AppContext {
                        settings: ctx.settings.clone(),
                        sessions: ctx.sessions.clone(),
                    });
                    return task.map(Message::Notifications);
                }
                Task::none()
            }

            NotificationMessage::TogglePowerMode => {
                ctx.settings.power_mode = !ctx.settings.power_mode;
                save_settings(&ctx.settings);
                // reset view state
                screen.collapse_all_groups();
                Task::none()
            }

            other => screen.update(other).map(Message::Notifications),
        }
    }

    fn update_settings(&mut self, message: Message) -> Task<Message> {
        let App::Authenticated(Screen::Settings(screen), ctx) = self else {
            return Task::none();
        };

        // Handle session restoration
        if let Message::SessionRestored(result) = message {
            match result {
                Ok(session) => {
                    let username = session.username.clone();
                    ctx.sessions.add_session(session);
                    screen.settings.set_active_account(&username);
                    save_settings(&screen.settings);
                }
                Err(e) => {
                    eprintln!("Failed to restore session: {}", e);
                    screen.accounts_state.error_message =
                        Some(format!("Failed to activate account: {}", e));
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

            SettingsMessage::RemoveAccount(username) => {
                // Remove from active sessions
                let _ = ctx.sessions.remove_account(username);

                // If no accounts left, logout
                if ctx.sessions.primary().is_none() {
                    let _ = AuthManager::delete_token();
                    *self = App::Login(LoginScreen::new());
                    return Task::none();
                }

                // If we still have an account, ensure settings match the new primary
                if let Some(primary) = ctx.sessions.primary() {
                    screen.settings.set_active_account(&primary.username.clone());
                    save_settings(&screen.settings);
                }
            }

            SettingsMessage::TokenValidated(Ok(username)) => {
                let username = username.clone();
                return Task::perform(
                    async move {
                        let token = crate::github::AccountKeyring::load_token(&username)
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
            }

            _ => {}
        }

        screen.update(settings_msg).map(Message::Settings)
    }

    fn update_rule_engine(&mut self, message: Message) -> Task<Message> {
        let App::Authenticated(Screen::RuleEngine(screen, origin), ctx) = self else {
            return Task::none();
        };

        let Message::RuleEngine(rule_msg) = message else {
            return Task::none();
        };

        if let RuleEngineMessage::Back = rule_msg {
            match origin {
                RuleEngineOrigin::Settings => {
                    let settings_screen = SettingsScreen::new(ctx.settings.clone());
                    *self = App::Authenticated(
                        Screen::Settings(settings_screen),
                        AppContext {
                            settings: ctx.settings.clone(),
                            sessions: ctx.sessions.clone(),
                        },
                    );
                }
                RuleEngineOrigin::Notifications => {
                    if let Some(session) = ctx.sessions.primary() {
                        let (notif_screen, task) =
                            NotificationsScreen::new(session.client.clone(), session.user.clone());
                        *self = App::Authenticated(
                            Screen::Notifications(notif_screen),
                            AppContext {
                                settings: ctx.settings.clone(),
                                sessions: ctx.sessions.clone(),
                            },
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
        if let App::Authenticated(Screen::Notifications(screen), _) = self {
            if !screen.is_loading {
                return screen
                    .update(NotificationMessage::Refresh)
                    .map(Message::Notifications);
            }
        }
        Task::none()
    }

    fn handle_tray_poll(&mut self) -> Task<Message> {
        let Some(cmd) = TrayManager::poll_global_events() else {
            return Task::none();
        };

        match cmd {
            TrayCommand::ShowWindow => {
                let was_hidden = window_state::restore_from_hidden();

                let window_task = if let Some(id) = window_state::get_window_id() {
                    Task::batch([
                        window::set_mode(id, window::Mode::Windowed),
                        window::gain_focus(id),
                    ])
                } else {
                    Task::none()
                };

                // If coming back from hidden, trigger a refresh
                if was_hidden {
                    if let App::Authenticated(Screen::Notifications(screen), _) = self {
                        return Task::batch([
                            window_task,
                            screen
                                .update(NotificationMessage::Refresh)
                                .map(Message::Notifications),
                        ]);
                    }
                }

                window_task
            }
            TrayCommand::Quit => exit(),
        }
    }

    fn handle_window_event(&mut self, id: WindowId, event: window::Event) -> Task<Message> {
        // Store the main window ID on first event
        window_state::set_window_id(id);

        match event {
            window::Event::CloseRequested => {
                let minimize_to_tray = self.get_minimize_to_tray_setting();

                if minimize_to_tray {
                    self.enter_tray_mode(id)
                } else {
                    exit()
                }
            }

            window::Event::Moved(position) => {
                // Windows reports -32000, -32000 when window is minimized - ignore these
                if position.x > -10000.0 && position.y > -10000.0 {
                    if let Some(settings) = self.get_settings_mut() {
                        settings.window_x = Some(position.x as i32);
                        settings.window_y = Some(position.y as i32);
                        save_settings(settings);
                    }
                }
                Task::none()
            }

            window::Event::Resized(size) => {
                // Windows reports 0x0 when window is minimized - ignore these
                if size.width > 100.0 && size.height > 100.0 {
                    if let Some(settings) = self.get_settings_mut() {
                        settings.window_width = size.width;
                        settings.window_height = size.height;
                        save_settings(settings);
                    }
                }
                Task::none()
            }

            _ => Task::none(),
        }
    }

    // ========================================================================
    // Navigation Helpers
    // ========================================================================

    /// Navigate to the notifications screen using the current primary session.
    fn go_to_notifications(&mut self) -> Task<Message> {
        let App::Authenticated(current_screen, ctx) = self else {
            return Task::none();
        };

        // Sync settings from SettingsScreen if coming from there
        let settings = match current_screen {
            Screen::Settings(screen) => screen.settings.clone(),
            _ => ctx.settings.clone(),
        };

        if let Some(session) = ctx.sessions.primary() {
            let (notif_screen, task) =
                NotificationsScreen::new(session.client.clone(), session.user.clone());
            *self = App::Authenticated(
                Screen::Notifications(notif_screen),
                AppContext {
                    settings,
                    sessions: ctx.sessions.clone(),
                },
            );
            task.map(Message::Notifications)
        } else {
            Task::none()
        }
    }

    /// Navigate to the settings screen.
    fn go_to_settings(&mut self) -> Task<Message> {
        let App::Authenticated(_, ctx) = self else {
            return Task::none();
        };

        let settings_screen = SettingsScreen::new(ctx.settings.clone());
        *self = App::Authenticated(
            Screen::Settings(settings_screen),
            AppContext {
                settings: ctx.settings.clone(),
                sessions: ctx.sessions.clone(),
            },
        );
        Task::none()
    }

    /// Navigate to the rule engine screen.
    fn go_to_rule_engine(&mut self, origin: RuleEngineOrigin) -> Task<Message> {
        let App::Authenticated(current_screen, ctx) = self else {
            return Task::none();
        };

        // Sync settings from SettingsScreen if coming from there
        let settings = match current_screen {
            Screen::Settings(screen) => screen.settings.clone(),
            _ => ctx.settings.clone(),
        };

        let rules = NotificationRuleSet::load();
        let rule_engine_screen = RuleEngineScreen::new(rules, settings.clone());
        *self = App::Authenticated(
            Screen::RuleEngine(rule_engine_screen, origin),
            AppContext {
                settings,
                sessions: ctx.sessions.clone(),
            },
        );
        Task::none()
    }

    // ========================================================================
    // Tray/Memory Management
    // ========================================================================

    /// Enter tray mode: hide window and free memory.
    fn enter_tray_mode(&mut self, window_id: WindowId) -> Task<Message> {
        window_state::set_hidden(true);

        // Clear notification data to free memory
        if let App::Authenticated(Screen::Notifications(screen), _) = self {
            screen.all_notifications.clear();
            screen.all_notifications.shrink_to_fit();
            screen.filtered_notifications.clear();
            screen.filtered_notifications.shrink_to_fit();
            screen.groups.clear();
            screen.groups.shrink_to_fit();
            screen.type_counts.clear();
            screen.type_counts.shrink_to_fit();
            screen.repo_counts.clear();
            screen.repo_counts.shrink_to_fit();
        }

        // Aggressively trim memory
        crate::platform::trim_memory();

        window::set_mode(window_id, window::Mode::Hidden)
    }

    // ========================================================================
    // Settings Helpers
    // ========================================================================

    /// Get the minimize_to_tray setting from current state.
    fn get_minimize_to_tray_setting(&self) -> bool {
        match self {
            App::Authenticated(Screen::Settings(screen), _) => screen.settings.minimize_to_tray,
            App::Authenticated(_, ctx) => ctx.settings.minimize_to_tray,
            _ => AppSettings::load().minimize_to_tray,
        }
    }

    /// Get mutable reference to settings if available.
    fn get_settings_mut(&mut self) -> Option<&mut AppSettings> {
        match self {
            App::Authenticated(Screen::Settings(screen), _) => Some(&mut screen.settings),
            App::Authenticated(_, ctx) => Some(&mut ctx.settings),
            _ => None,
        }
    }

    // ========================================================================
    // View Rendering
    // ========================================================================

    /// Render the current view.
    pub fn view(&self) -> Element<'_, Message> {
        match self {
            App::Loading => self.view_loading(),
            App::Login(screen) => screen.view().map(Message::Login),
            App::Authenticated(screen, ctx) => match screen {
                Screen::Notifications(notif_screen) => {
                    let accounts = ctx.account_names();

                    if ctx.settings.power_mode {
                        self.view_power_mode(notif_screen, &ctx.settings, &accounts)
                    } else {
                        notif_screen
                            .view(
                                &accounts,
                                ctx.settings.icon_theme,
                                ctx.settings.sidebar_width,
                                false,
                            )
                            .map(Message::Notifications)
                    }
                }
                Screen::Settings(settings_screen) => {
                    settings_screen.view().map(Message::Settings)
                }
                Screen::RuleEngine(rule_screen, _) => {
                    rule_screen.view().map(Message::RuleEngine)
                }
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
        accounts: &[String],
    ) -> Element<'a, Message> {
        use iced::widget::{column, row};

        let content = screen.view(accounts, settings.icon_theme, settings.sidebar_width, true);

        let main_area: iced::Element<NotificationMessage> = if settings.show_details_panel {
            row![
                content,
                details_panel::view_details_panel(None, settings.icon_theme)
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
            App::Loading => "GitTop".to_string(),
            App::Login(_) => "GitTop - Sign In".to_string(),
            App::Authenticated(screen, _) => match screen {
                Screen::Notifications(notif_screen) => {
                    let unread = notif_screen
                        .all_notifications
                        .iter()
                        .filter(|n| n.unread)
                        .count();
                    if unread > 0 {
                        format!("GitTop ({} unread)", unread)
                    } else {
                        "GitTop".to_string()
                    }
                }
                Screen::Settings(_) => "GitTop - Settings".to_string(),
                Screen::RuleEngine(_, _) => "GitTop - Rule Engine".to_string(),
            },
        }
    }

    /// Application theme - using a dark theme.
    pub fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }

    /// Subscriptions - periodic refresh, tray events, and window events.
    pub fn subscription(&self) -> Subscription<Message> {
        let is_hidden = window_state::is_hidden();

        // Poll tray events - slower when hidden to save CPU
        let tray_poll_interval = if is_hidden { 500 } else { 100 };
        let tray_sub =
            time::every(Duration::from_millis(tray_poll_interval)).map(|_| Message::TrayPoll);

        // Subscribe to window events
        let window_sub = event::listen_with(|event, _status, id| {
            if let Event::Window(window_event) = event {
                Some(Message::WindowEvent(id, window_event))
            } else {
                None
            }
        });

        match self {
            App::Authenticated(Screen::Notifications(_), _) => {
                if is_hidden {
                    // When hidden, still refresh but less frequently (every 2 minutes)
                    // This allows desktop notifications to fire for new items
                    Subscription::batch([
                        time::every(Duration::from_secs(60)).map(|_| Message::Tick),
                        tray_sub,
                        window_sub,
                    ])
                } else {
                    // Refresh every 60 seconds + tray events + window events
                    Subscription::batch([
                        time::every(Duration::from_secs(60)).map(|_| Message::Tick),
                        tray_sub,
                        window_sub,
                    ])
                }
            }
            _ => Subscription::batch([tray_sub, window_sub]),
        }
    }
}
