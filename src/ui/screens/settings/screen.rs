//! Settings screen - icon theme and account management.

use iced::widget::{button, column, container, row, text, toggler, Space};
use iced::{Alignment, Element, Fill, Task};

use crate::settings::{AppSettings, IconTheme, StoredAccount};
use crate::ui::{icons, theme};

/// Settings screen state.
#[derive(Debug, Clone)]
pub struct SettingsScreen {
    pub settings: AppSettings,
}

/// Settings screen messages.
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    /// Go back to notifications.
    Back,
    /// Toggle icon theme.
    ToggleIconTheme(bool),
    /// Toggle minimize to tray.
    ToggleMinimizeToTray(bool),
    /// Remove an account.
    RemoveAccount(String),
}

impl SettingsScreen {
    pub fn new(settings: AppSettings) -> Self {
        Self { settings }
    }

    pub fn update(&mut self, message: SettingsMessage) -> Task<SettingsMessage> {
        match message {
            SettingsMessage::Back => {
                // Handled by parent
                Task::none()
            }
            SettingsMessage::ToggleIconTheme(use_svg) => {
                self.settings.icon_theme = if use_svg {
                    IconTheme::Svg
                } else {
                    IconTheme::Emoji
                };
                let _ = self.settings.save();
                Task::none()
            }
            SettingsMessage::ToggleMinimizeToTray(enabled) => {
                self.settings.minimize_to_tray = enabled;
                let _ = self.settings.save();
                Task::none()
            }
            SettingsMessage::RemoveAccount(username) => {
                self.settings.remove_account(&username);
                let _ = self.settings.save();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, SettingsMessage> {
        let header = self.view_header();
        let content = self.view_content();

        column![header, content]
            .spacing(0)
            .width(Fill)
            .height(Fill)
            .into()
    }

    fn view_header(&self) -> Element<'_, SettingsMessage> {
        let icon_theme = self.settings.icon_theme;

        let back_btn = button(
            row![
                icons::icon_chevron_left(16.0, theme::TEXT_SECONDARY, icon_theme),
                Space::new().width(4),
                text("Back").size(12).color(theme::TEXT_SECONDARY),
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::ghost_button)
        .padding([6, 8])
        .on_press(SettingsMessage::Back);

        let title = text("Settings").size(18).color(theme::TEXT_PRIMARY);

        let header_row = row![
            back_btn,
            Space::new().width(Fill),
            title,
            Space::new().width(Fill),
        ]
        .align_y(Alignment::Center)
        .padding([12, 16]);

        container(header_row)
            .width(Fill)
            .style(theme::header)
            .into()
    }

    fn view_content(&self) -> Element<'_, SettingsMessage> {
        let content = column![
            // Appearance Section
            self.view_section_header("Appearance"),
            self.view_icon_theme_setting(),
            Space::new().height(24),
            // Behavior Section
            self.view_section_header("Behavior"),
            self.view_minimize_to_tray_setting(),
            Space::new().height(24),
            // Accounts Section
            self.view_section_header("Accounts"),
            self.view_accounts_section(),
        ]
        .spacing(8)
        .padding(16);

        container(content)
            .width(Fill)
            .height(Fill)
            .style(theme::app_container)
            .into()
    }

    fn view_section_header(&self, title: &'static str) -> Element<'static, SettingsMessage> {
        text(title).size(11).color(theme::TEXT_MUTED).into()
    }

    fn view_icon_theme_setting(&self) -> Element<'_, SettingsMessage> {
        let use_svg = self.settings.icon_theme == IconTheme::Svg;

        let description = if use_svg {
            "SVG icons (better quality, slightly higher memory usage +4MB~ extra )"
        } else {
            "Emoji icons (minimal memory usage)"
        };

        container(
            row![
                column![
                    text("Icon Style").size(14).color(theme::TEXT_PRIMARY),
                    Space::new().height(4),
                    text(description).size(11).color(theme::TEXT_SECONDARY),
                ]
                .width(Fill),
                toggler(use_svg)
                    .on_toggle(SettingsMessage::ToggleIconTheme)
                    .size(20),
            ]
            .align_y(Alignment::Center)
            .padding(12),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(theme::BG_CARD)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn view_minimize_to_tray_setting(&self) -> Element<'_, SettingsMessage> {
        let enabled = self.settings.minimize_to_tray;

        let description = if enabled {
            "App stays in system tray when you close the window"
        } else {
            "App exits completely when you close the window"
        };

        container(
            row![
                column![
                    text("Minimize to Tray").size(14).color(theme::TEXT_PRIMARY),
                    Space::new().height(4),
                    text(description).size(11).color(theme::TEXT_SECONDARY),
                ]
                .width(Fill),
                toggler(enabled)
                    .on_toggle(SettingsMessage::ToggleMinimizeToTray)
                    .size(20),
            ]
            .align_y(Alignment::Center)
            .padding(12),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(theme::BG_CARD)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn view_accounts_section(&self) -> Element<'_, SettingsMessage> {
        if self.settings.accounts.is_empty() {
            return container(
                text("No accounts added yet")
                    .size(12)
                    .color(theme::TEXT_MUTED),
            )
            .padding(12)
            .into();
        }

        let mut col = column![].spacing(8);

        for account in &self.settings.accounts {
            col = col.push(self.view_account_item(account));
        }

        col.into()
    }

    fn view_account_item(&self, account: &StoredAccount) -> Element<'static, SettingsMessage> {
        let icon_theme = self.settings.icon_theme;
        let status_color = if account.is_active {
            theme::ACCENT_GREEN
        } else {
            theme::TEXT_MUTED
        };

        let status_text = if account.is_active { "Active" } else { "" };
        let username = account.username.clone();
        let username_for_button = account.username.clone();

        container(
            row![
                icons::icon_user(14.0, theme::TEXT_SECONDARY, icon_theme),
                Space::new().width(8),
                text(username).size(13).color(theme::TEXT_PRIMARY),
                Space::new().width(8),
                text(status_text).size(10).color(status_color),
                Space::new().width(Fill),
                button(icons::icon_trash(14.0, theme::TEXT_MUTED, icon_theme))
                    .style(theme::ghost_button)
                    .padding(4)
                    .on_press(SettingsMessage::RemoveAccount(username_for_button)),
            ]
            .align_y(Alignment::Center)
            .padding(12),
        )
        .style(|_| container::Style {
            background: Some(iced::Background::Color(theme::BG_CARD)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }
}
