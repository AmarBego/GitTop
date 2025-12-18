//! Main application state and logic.

use std::time::Duration;

use iced::{time, Element, Subscription, Task, Theme};

use crate::github::{AuthManager, GitHubClient, UserInfo};
use crate::ui::screens::{
    login::{LoginMessage, LoginScreen},
    notifications::{NotificationMessage, NotificationsScreen},
};

/// Application state - which screen we're on.
pub enum App {
    /// Checking for existing auth on startup.
    Loading,
    /// Login screen - no auth.
    Login(LoginScreen),
    /// Main notifications screen - authenticated.
    Notifications(NotificationsScreen),
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
    /// Periodic refresh tick.
    Tick,
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
                    let (screen, task) = NotificationsScreen::new(client, user);
                    *self = App::Notifications(screen);
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
                        let (notif_screen, task) = NotificationsScreen::new(client, user);
                        *self = App::Notifications(notif_screen);
                        return task.map(Message::Notifications);
                    }

                    screen.update(login_msg).map(Message::Login)
                } else {
                    Task::none()
                }
            }

            Message::Notifications(notif_msg) => {
                if let App::Notifications(screen) = self {
                    // Check for logout
                    if let NotificationMessage::Logout = notif_msg {
                        let _ = AuthManager::delete_token();
                        *self = App::Login(LoginScreen::new());
                        return Task::none();
                    }

                    screen.update(notif_msg).map(Message::Notifications)
                } else {
                    Task::none()
                }
            }

            Message::Tick => {
                // Auto-refresh notifications every tick
                if let App::Notifications(screen) = self {
                    if !screen.is_loading {
                        return screen
                            .update(NotificationMessage::Refresh)
                            .map(Message::Notifications);
                    }
                }
                Task::none()
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
            App::Notifications(screen) => screen.view().map(Message::Notifications),
        }
    }

    /// Window title.
    pub fn title(&self) -> String {
        match self {
            App::Loading => "GitTop".to_string(),
            App::Login(_) => "GitTop - Sign In".to_string(),
            App::Notifications(screen) => {
                let unread = screen.all_notifications.iter().filter(|n| n.unread).count();
                if unread > 0 {
                    format!("GitTop ({} unread)", unread)
                } else {
                    "GitTop".to_string()
                }
            }
        }
    }

    /// Application theme - using a dark theme.
    pub fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }

    /// Subscriptions - periodic refresh.
    pub fn subscription(&self) -> Subscription<Message> {
        match self {
            App::Notifications(_) => {
                // Refresh every 60 seconds
                time::every(Duration::from_secs(60)).map(|_| Message::Tick)
            }
            _ => Subscription::none(),
        }
    }
}
