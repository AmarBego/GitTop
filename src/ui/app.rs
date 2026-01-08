//! Main application state and effect executor.
//!
//! This module implements the Effect Pattern: app.rs is a thin orchestrator
//! that delegates to screens and applies the effects they return.

use std::time::Duration;

use iced::window::Id as WindowId;
use iced::{Element, Event, Subscription, Task, Theme, event, time, window};

use crate::github::{SessionManager, auth};
use crate::settings::AppSettings;
use crate::ui::context::AppContext;
use crate::ui::effects::{AppEffect, NavigateTo, SessionEffect};
use crate::ui::features;
use crate::ui::handlers::platform;

use crate::ui::routing::{RuleEngineOrigin, Screen};
use crate::ui::screens::{
    login::{LoginMessage, LoginScreen},
    notifications::NotificationsScreen,
    notifications::messages::NotificationMessage,
    settings::messages::SettingsMessage,
    settings::rule_engine::messages::RuleEngineMessage,
};
use crate::ui::state;

/// Application state - which phase we're in.
pub enum App {
    /// Checking for existing auth on startup.
    Loading,
    /// Login screen - no auth.
    Login(LoginScreen),
    /// Authenticated state with screen and shared context.
    Authenticated(Box<Screen>, AppContext),
}

/// Top-level application messages.
#[derive(Debug, Clone)]
pub enum Message {
    // -- Lifecycle --
    RestoreComplete(SessionManager, Option<String>),
    /// Update check completed
    UpdateCheckResult(Option<crate::update_checker::UpdateInfo>),

    // -- UI Screens --
    Login(LoginMessage),
    Notifications(NotificationMessage),
    Settings(SettingsMessage),
    RuleEngine(RuleEngineMessage),

    // -- Platform/System --
    Tick,
    TrayPoll,
    WindowEvent(WindowId, window::Event),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            App::Loading,
            Task::perform(
                async {
                    use crate::github::session::SessionError;

                    let mut sessions = SessionManager::new();
                    let mut settings = AppSettings::load();
                    let mut failed_accounts = Vec::new();
                    let mut network_error: Option<String> = None;

                    for account in &settings.accounts {
                        match sessions.restore_account(&account.username).await {
                            Ok(()) => {}
                            Err(SessionError::AccountNotFound(_)) => {
                                failed_accounts.push(account.username.clone());
                            }
                            Err(SessionError::NetworkError(msg)) => {
                                network_error = Some(msg);
                            }
                            Err(e) => {
                                tracing::warn!(
                                    username = %account.username,
                                    error = %e,
                                    "Failed to restore saved session"
                                );
                                failed_accounts.push(account.username.clone());
                            }
                        }
                    }

                    if !failed_accounts.is_empty() {
                        for username in failed_accounts {
                            settings.remove_account(&username);
                        }
                        settings.save_silent();
                    }

                    let primary = settings
                        .accounts
                        .iter()
                        .find(|a| a.is_active)
                        .or_else(|| settings.accounts.first())
                        .map(|a| a.username.clone());

                    if let Some(username) = primary {
                        sessions.set_primary(&username);
                    }

                    (sessions, network_error)
                },
                |(sessions, network_error)| Message::RestoreComplete(sessions, network_error),
            ),
        )
    }

    /// Update application state.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        // Handle platform events first (tick, tray, window)
        match &message {
            Message::Tick => return self.handle_tick(),
            Message::TrayPoll => return self.handle_tray_poll(),
            Message::WindowEvent(id, event) => return self.handle_window_event(*id, event.clone()),
            Message::UpdateCheckResult(info) => {
                if let Some(screen) = self.notification_screen_mut() {
                    screen.update_info = info.clone();
                }
                return Task::none();
            }
            _ => {}
        }

        // Dispatch to state-specific handlers and apply effects
        let (task, effect) = match self {
            App::Loading => (self.update_loading(message), AppEffect::None),
            App::Login(_) => self.update_login(message),
            App::Authenticated(screen, _) => match &mut **screen {
                Screen::Notifications(_) => self.update_notifications(message),
                Screen::Settings(_) => self.update_settings(message),
                Screen::RuleEngine(_, _) => self.update_rule_engine(message),
            },
        };

        // Apply any effects and combine tasks
        let effect_task = self.apply_effect(effect);
        Task::batch([task, effect_task])
    }

    // ========================================================================
    // Effect Executor
    // ========================================================================

    fn apply_effect(&mut self, effect: AppEffect) -> Task<Message> {
        match effect {
            AppEffect::None => Task::none(),
            AppEffect::Navigate(to) => self.navigate(to),
            AppEffect::Session(s) => self.handle_session_effect(s),
        }
    }

    fn navigate(&mut self, to: NavigateTo) -> Task<Message> {
        use crate::ui::handlers::navigation;

        let App::Authenticated(current_screen, ctx) = self else {
            if matches!(to, NavigateTo::Login) {
                *self = App::Login(LoginScreen::new());
            }
            return Task::none();
        };

        match to {
            NavigateTo::Notifications => {
                match navigation::go_to_notifications(current_screen, ctx) {
                    Some(t) => {
                        *self = App::Authenticated(
                            Box::new(Screen::Notifications(t.screen)),
                            ctx.with_settings(t.updated_settings),
                        );
                        t.task
                    }
                    None => Task::none(),
                }
            }
            NavigateTo::Settings => {
                let t = navigation::go_to_settings(ctx);
                *self = App::Authenticated(
                    Box::new(Screen::Settings(t.screen)),
                    ctx.with_settings(t.updated_settings),
                );
                Task::none()
            }
            NavigateTo::RuleEngine { from_settings } => {
                let origin = RuleEngineOrigin::from_settings_flag(from_settings);
                let settings = match &**current_screen {
                    Screen::Settings(s) => Some(&s.settings),
                    _ => Some(&ctx.settings),
                };
                let t = navigation::go_to_rule_engine(settings, origin);
                *self = App::Authenticated(
                    Box::new(Screen::RuleEngine(t.screen, t.origin)),
                    ctx.with_settings(t.updated_settings),
                );
                Task::none()
            }
            NavigateTo::Login => {
                let _ = auth::delete_token();
                *self = App::Login(LoginScreen::new());
                Task::none()
            }
            NavigateTo::Back => match &**current_screen {
                Screen::Settings(_) => self.navigate(NavigateTo::Notifications),
                Screen::RuleEngine(_, origin) => {
                    let target = match origin {
                        RuleEngineOrigin::Settings => NavigateTo::Settings,
                        RuleEngineOrigin::Notifications => NavigateTo::Notifications,
                    };
                    self.navigate(target)
                }
                Screen::Notifications(_) => Task::none(),
            },
        }
    }

    fn handle_session_effect(&mut self, effect: SessionEffect) -> Task<Message> {
        use crate::ui::handlers::navigation;

        let App::Authenticated(screen, ctx) = self else {
            return Task::none();
        };

        match effect {
            SessionEffect::Logout => {
                match navigation::handle_logout(&mut ctx.sessions, &mut ctx.settings) {
                    Some((new_screen, task)) => {
                        *self = App::Authenticated(
                            Box::new(Screen::Notifications(new_screen)),
                            ctx.with_settings(ctx.settings.clone()),
                        );
                        task
                    }
                    None => self.navigate(NavigateTo::Login),
                }
            }
            SessionEffect::SwitchAccount(username) => {
                let current_screen = match &**screen {
                    Screen::Notifications(s) => s,
                    _ => return Task::none(),
                };
                match navigation::switch_account(
                    &username,
                    current_screen,
                    &mut ctx.sessions,
                    &mut ctx.settings,
                ) {
                    Some((new_screen, task)) => {
                        *self = App::Authenticated(
                            Box::new(Screen::Notifications(new_screen)),
                            ctx.with_settings(ctx.settings.clone()),
                        );
                        task
                    }
                    None => Task::none(),
                }
            }

            SessionEffect::RemoveAccount(username) => {
                let _ = ctx.sessions.remove_account(&username);
                ctx.settings.remove_account(&username);
                ctx.settings.save_silent();
                if ctx.sessions.primary().is_none() {
                    return self.navigate(NavigateTo::Login);
                }
                if let Some(primary) = ctx.sessions.primary() {
                    ctx.settings.set_active_account(&primary.username);
                    ctx.settings.save_silent();
                }
                if let Screen::Settings(s) = &mut **screen {
                    s.settings = ctx.settings.clone();
                }
                Task::none()
            }
        }
    }

    // ========================================================================
    // Screen Update Handlers
    // ========================================================================

    fn update_loading(&mut self, message: Message) -> Task<Message> {
        if let Message::RestoreComplete(sessions, network_error) = message {
            if let Some(session) = sessions.primary() {
                let mut settings = AppSettings::load();
                settings.set_active_account(&session.username);
                settings.save_silent();
                settings.apply_theme();

                let (mut notif_screen, task) =
                    NotificationsScreen::new(session.client.clone(), session.user.clone());

                if let Some(error) = network_error {
                    notif_screen.error_message = Some(format!("Network error: {}", error));
                }

                let ctx = AppContext::new(settings.clone(), sessions);
                *self = App::Authenticated(
                    Box::new(Screen::Notifications(Box::new(notif_screen))),
                    ctx,
                );

                // Spawn update check if enabled
                let update_task = if settings.check_for_updates {
                    Task::perform(
                        crate::update_checker::check_for_update(),
                        Message::UpdateCheckResult,
                    )
                } else {
                    Task::none()
                };

                return Task::batch([task.map(Message::Notifications), update_task]);
            }

            let settings = AppSettings::load();
            settings.apply_theme();

            let mut login_screen = LoginScreen::new();
            if let Some(error) = network_error {
                login_screen.error_message = Some(format!(
                    "Network error: {}. Your accounts are preserved - fix connection and restart.",
                    error
                ));
            }

            *self = App::Login(login_screen);
            crate::platform::trim_memory();
        }
        Task::none()
    }

    fn update_login(&mut self, message: Message) -> (Task<Message>, AppEffect) {
        let App::Login(screen) = self else {
            return (Task::none(), AppEffect::None);
        };

        let Message::Login(login_msg) = message else {
            return (Task::none(), AppEffect::None);
        };

        match login_msg {
            LoginMessage::LoginSuccess(client, user) => {
                let mut settings = AppSettings::load();
                settings.set_active_account(&user.login);
                settings.save_silent();
                settings.apply_theme();

                let token = client.token().to_string();
                let _ = crate::github::keyring::save_token(&user.login, &token);

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
                (task.map(Message::Notifications), AppEffect::None)
            }
            other => (screen.update(other).map(Message::Login), AppEffect::None),
        }
    }

    fn update_notifications(&mut self, message: Message) -> (Task<Message>, AppEffect) {
        let App::Authenticated(boxed_screen, ctx) = self else {
            return (Task::none(), AppEffect::None);
        };

        let Screen::Notifications(screen) = &mut **boxed_screen else {
            return (Task::none(), AppEffect::None);
        };

        let Message::Notifications(notif_msg) = message else {
            return (Task::none(), AppEffect::None);
        };

        // Let the screen handle the message and return effect
        let (task, effect) = screen.update_with_effect(notif_msg, ctx);
        (task.map(Message::Notifications), effect)
    }

    fn update_settings(&mut self, message: Message) -> (Task<Message>, AppEffect) {
        let App::Authenticated(boxed_screen, ctx) = self else {
            return (Task::none(), AppEffect::None);
        };

        let Screen::Settings(screen) = &mut **boxed_screen else {
            return (Task::none(), AppEffect::None);
        };

        // Handle session restoration (async result)

        let Message::Settings(settings_msg) = message else {
            return (Task::none(), AppEffect::None);
        };

        // Let the screen handle the message and return effect
        let (task, effect) = screen.update_with_effect(settings_msg, ctx);

        (task.map(Message::Settings), effect)
    }

    fn update_rule_engine(&mut self, message: Message) -> (Task<Message>, AppEffect) {
        let App::Authenticated(boxed_screen, _) = self else {
            return (Task::none(), AppEffect::None);
        };

        let Screen::RuleEngine(screen, _) = &mut **boxed_screen else {
            return (Task::none(), AppEffect::None);
        };

        let Message::RuleEngine(rule_msg) = message else {
            return (Task::none(), AppEffect::None);
        };

        // Let the screen handle the message and return effect
        let (task, effect) = screen.update_with_effect(rule_msg);
        (task.map(Message::RuleEngine), effect)
    }

    // ========================================================================
    // Platform Event Handlers
    // ========================================================================

    fn handle_tick(&mut self) -> Task<Message> {
        let App::Authenticated(boxed_screen, _) = self else {
            return Task::none();
        };
        let Screen::Notifications(screen) = &mut **boxed_screen else {
            return Task::none();
        };
        platform::handle_tick(screen)
    }

    fn handle_tray_poll(&mut self) -> Task<Message> {
        platform::handle_tray_poll(self.notification_screen_mut())
    }

    fn handle_window_event(&mut self, id: WindowId, event: window::Event) -> Task<Message> {
        let App::Authenticated(boxed_screen, ctx) = self else {
            state::set_window_id(id);
            return Task::none();
        };

        let minimize_to_tray = match &**boxed_screen {
            Screen::Settings(s) => s.settings.minimize_to_tray,
            _ => ctx.settings.minimize_to_tray,
        };

        let (settings, notification_screen) = match &mut **boxed_screen {
            Screen::Settings(s) => (Some(&mut s.settings), None),
            Screen::Notifications(s) => (Some(&mut ctx.settings), Some(&mut **s)),
            Screen::RuleEngine(_, _) => (Some(&mut ctx.settings), None),
        };

        platform::handle_window_event(
            id,
            event,
            platform::WindowEventContext {
                settings,
                minimize_to_tray,
                notification_screen,
            },
        )
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

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

    pub fn view(&self) -> Element<'_, Message> {
        match self {
            App::Loading => self.view_loading(),
            App::Login(screen) => screen.view().map(Message::Login),
            App::Authenticated(boxed_screen, ctx) => match &**boxed_screen {
                Screen::Notifications(notif_screen) => {
                    let accounts = ctx.account_names();

                    if ctx.settings.power_mode {
                        features::power_mode::view::app_layout(
                            notif_screen,
                            &ctx.settings,
                            accounts,
                        )
                        .map(Message::Notifications)
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

    pub fn title(&self) -> String {
        match self {
            App::Loading => "GitTop".into(),
            App::Login(_) => "GitTop - Sign In".into(),
            App::Authenticated(screen, _) => screen.title(),
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let is_hidden = state::is_hidden();

        let tray_interval = if is_hidden {
            platform::TRAY_POLL_INTERVAL_HIDDEN_MS
        } else {
            platform::TRAY_POLL_INTERVAL_ACTIVE_MS
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
            time::every(Duration::from_secs(platform::REFRESH_INTERVAL_SECS)).map(|_| Message::Tick)
        });

        let subs: Vec<_> = tick_sub.into_iter().chain([tray_sub, window_sub]).collect();
        Subscription::batch(subs)
    }

    // ========================================================================
    // Daemon Mode Support (Linux)
    // ========================================================================

    #[cfg(target_os = "linux")]
    pub fn new_for_daemon() -> (Self, Task<Message>) {
        let (app, restore_task) = Self::new();
        let (window_id, open_task) = crate::platform::linux::build_initial_window_settings();
        state::set_window_id(window_id);
        (app, Task::batch([restore_task, open_task.discard()]))
    }

    #[cfg(target_os = "linux")]
    pub fn view_for_daemon(&self, _window_id: window::Id) -> Element<'_, Message> {
        self.view()
    }

    #[cfg(target_os = "linux")]
    pub fn title_for_daemon(&self, _window_id: window::Id) -> String {
        self.title()
    }

    #[cfg(target_os = "linux")]
    pub fn theme_for_daemon(&self, _window_id: window::Id) -> Theme {
        self.theme()
    }
}
