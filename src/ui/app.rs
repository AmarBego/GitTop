//! Main application state and logic.

use std::sync::OnceLock;
use std::time::Duration;

use iced::window::Id as WindowId;
use iced::{event, exit, time, window, Element, Event, Subscription, Task, Theme};

use crate::github::{AuthManager, GitHubClient, UserInfo};
use crate::settings::AppSettings;
use crate::tray::{TrayCommand, TrayManager};
use crate::ui::screens::{
    login::{LoginMessage, LoginScreen},
    notifications::{NotificationMessage, NotificationsScreen},
    settings::{SettingsMessage, SettingsScreen},
};

/// Global storage for the main window ID
static MAIN_WINDOW_ID: OnceLock<WindowId> = OnceLock::new();

/// Track if window is currently hidden (minimized to tray)
static IS_WINDOW_HIDDEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Application state - which screen we're on.
pub enum App {
    /// Checking for existing auth on startup.
    Loading,
    /// Login screen - no auth.
    Login(LoginScreen),
    /// Main notifications screen - authenticated.
    Notifications(NotificationsScreen, AppSettings),
    /// Settings screen.
    Settings(SettingsScreen, GitHubClient, UserInfo),
}

/// Top-level application messages.
#[derive(Debug, Clone)]
pub enum Message {
    /// Restore result.
    RestoreComplete(Option<(GitHubClient, UserInfo)>),
    /// Login screen messages.
    Login(LoginMessage),
    /// Notifications screen messages.
    Notifications(NotificationMessage),
    /// Settings screen messages.
    Settings(SettingsMessage),
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
                async { AuthManager::try_restore().await.ok().flatten() },
                Message::RestoreComplete,
            ),
        )
    }

    /// Update application state.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RestoreComplete(result) => match result {
                Some((client, user)) => {
                    let mut settings = AppSettings::load();
                    settings.set_active_account(&user.login);
                    let _ = settings.save();

                    let (screen, task) = NotificationsScreen::new(client, user);
                    *self = App::Notifications(screen, settings);
                    task.map(Message::Notifications)
                }
                None => {
                    *self = App::Login(LoginScreen::new());
                    Task::none()
                }
            },

            Message::Login(login_msg) => {
                if let App::Login(screen) = self {
                    // Check for successful login
                    if let LoginMessage::LoginSuccess(client, user) = login_msg.clone() {
                        let mut settings = AppSettings::load();
                        settings.set_active_account(&user.login);
                        let _ = settings.save();

                        let (notif_screen, task) = NotificationsScreen::new(client, user);
                        *self = App::Notifications(notif_screen, settings);
                        return task.map(Message::Notifications);
                    }

                    screen.update(login_msg).map(Message::Login)
                } else {
                    Task::none()
                }
            }

            Message::Notifications(notif_msg) => {
                if let App::Notifications(screen, settings) = self {
                    // Check for logout
                    if let NotificationMessage::Logout = notif_msg {
                        let _ = AuthManager::delete_token();
                        *self = App::Login(LoginScreen::new());
                        return Task::none();
                    }

                    // Check for open settings
                    if let NotificationMessage::OpenSettings = notif_msg {
                        let settings_screen = SettingsScreen::new(settings.clone());
                        let client = screen.client.clone();
                        let user = screen.user.clone();
                        *self = App::Settings(settings_screen, client, user);
                        return Task::none();
                    }

                    screen.update(notif_msg).map(Message::Notifications)
                } else {
                    Task::none()
                }
            }

            Message::Settings(settings_msg) => {
                if let App::Settings(screen, client, user) = self {
                    // Check for back navigation
                    if let SettingsMessage::Back = settings_msg {
                        let (notif_screen, task) =
                            NotificationsScreen::new(client.clone(), user.clone());
                        *self = App::Notifications(notif_screen, screen.settings.clone());
                        return task.map(Message::Notifications);
                    }

                    screen.update(settings_msg).map(Message::Settings)
                } else {
                    Task::none()
                }
            }

            Message::Tick => {
                // Auto-refresh notifications every tick
                if let App::Notifications(screen, _) = self {
                    if !screen.is_loading {
                        return screen
                            .update(NotificationMessage::Refresh)
                            .map(Message::Notifications);
                    }
                }
                Task::none()
            }

            Message::TrayPoll => {
                // Poll tray events
                if let Some(cmd) = TrayManager::poll_global_events() {
                    match cmd {
                        TrayCommand::ShowWindow => {
                            // Show and focus the window using stored ID
                            let was_hidden = IS_WINDOW_HIDDEN.swap(false, std::sync::atomic::Ordering::Relaxed);
                            
                            let window_task = if let Some(&id) = MAIN_WINDOW_ID.get() {
                                Task::batch([
                                    window::set_mode(id, window::Mode::Windowed),
                                    window::gain_focus(id),
                                ])
                            } else {
                                Task::none()
                            };
                            
                            // If coming back from hidden, trigger a refresh
                            if was_hidden {
                                if let App::Notifications(screen, _) = self {
                                    return Task::batch([
                                        window_task,
                                        screen.update(NotificationMessage::Refresh).map(Message::Notifications),
                                    ]);
                                }
                            }
                            
                            window_task
                        }
                        TrayCommand::Quit => {
                            // Exit the application
                            exit()
                        }
                    }
                } else {
                    Task::none()
                }
            }

            Message::WindowEvent(id, event) => {
                // Store the main window ID on first event
                let _ = MAIN_WINDOW_ID.set(id);
                
                if let window::Event::CloseRequested = event {
                    // Check if minimize to tray is enabled
                    let minimize_to_tray = match self {
                        App::Notifications(_, settings) => settings.minimize_to_tray,
                        App::Settings(screen, _, _) => screen.settings.minimize_to_tray,
                        _ => AppSettings::load().minimize_to_tray,
                    };

                    if minimize_to_tray {
                        // Hide the window to tray instead of quitting
                        IS_WINDOW_HIDDEN.store(true, std::sync::atomic::Ordering::Relaxed);
                        window::set_mode(id, window::Mode::Hidden)
                    } else {
                        // Exit the application
                        exit()
                    }
                } else {
                    Task::none()
                }
            }
        }
    }

    /// Render the current view.
    pub fn view(&self) -> Element<'_, Message> {
        match self {
            App::Loading => {
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
            App::Login(screen) => screen.view().map(Message::Login),
            App::Notifications(screen, settings) => {
                screen.view(settings.icon_theme).map(Message::Notifications)
            }
            App::Settings(screen, _, _) => screen.view().map(Message::Settings),
        }
    }

    /// Window title.
    pub fn title(&self) -> String {
        match self {
            App::Loading => "GitTop".to_string(),
            App::Login(_) => "GitTop - Sign In".to_string(),
            App::Notifications(screen, _) => {
                let unread = screen.all_notifications.iter().filter(|n| n.unread).count();
                if unread > 0 {
                    format!("GitTop ({} unread)", unread)
                } else {
                    "GitTop".to_string()
                }
            }
            App::Settings(_, _, _) => "GitTop - Settings".to_string(),
        }
    }

    /// Application theme - using a dark theme.
    pub fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }

    /// Subscriptions - periodic refresh, tray events, and window events.
    pub fn subscription(&self) -> Subscription<Message> {
        let is_hidden = IS_WINDOW_HIDDEN.load(std::sync::atomic::Ordering::Relaxed);
        
        // Poll tray events - slower when hidden to save CPU
        let tray_poll_interval = if is_hidden { 500 } else { 100 };
        let tray_sub = time::every(Duration::from_millis(tray_poll_interval)).map(|_| Message::TrayPoll);

        // Subscribe to window events
        let window_sub = event::listen_with(|event, _status, id| {
            if let Event::Window(window_event) = event {
                Some(Message::WindowEvent(id, window_event))
            } else {
                None
            }
        });

        match self {
            App::Notifications(_, _) => {
                if is_hidden {
                    // When hidden, only poll tray and listen for window events
                    // Skip auto-refresh to save memory and CPU
                    Subscription::batch([tray_sub, window_sub])
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
