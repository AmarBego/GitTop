//! Settings screen - theme, icon style, and account management.

use iced::widget::{button, column, container, pick_list, row, text, toggler, Space};
use iced::{Alignment, Element, Fill, Task};

use crate::settings::{AppSettings, AppTheme, IconTheme, StoredAccount};
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
    /// Change app theme.
    ChangeTheme(AppTheme),
    /// Toggle icon theme.
    ToggleIconTheme(bool),
    /// Toggle minimize to tray.
    ToggleMinimizeToTray(bool),
    /// Set font scale (0.8 - 1.5).
    SetFontScale(f32),
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
            SettingsMessage::ChangeTheme(new_theme) => {
                self.settings.theme = new_theme;
                // Update global theme for immediate effect
                theme::set_theme(new_theme);
                let _ = self.settings.save();
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
            SettingsMessage::SetFontScale(scale) => {
                // Clamp to valid range
                let clamped = scale.clamp(0.8, 1.5);
                self.settings.font_scale = clamped;
                // Update global font scale for immediate effect
                theme::set_font_scale(clamped);
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
        let p = theme::palette();
        let icon_theme = self.settings.icon_theme;

        let back_btn = button(
            row![
                icons::icon_chevron_left(16.0, p.text_secondary, icon_theme),
                Space::new().width(4),
                text("Back").size(13).color(p.text_secondary),
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::ghost_button)
        .padding([6, 10])
        .on_press(SettingsMessage::Back);

        let title = text("Settings").size(18).color(p.text_primary);

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
        let p = theme::palette();

        let content = column![
            // Appearance Section
            self.view_section_header("Appearance"),
            self.view_theme_setting(),
            Space::new().height(8),
            self.view_icon_theme_setting(),
            Space::new().height(8),
            self.view_font_scale_setting(),
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
        .padding(20);

        container(content)
            .width(Fill)
            .height(Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_base)),
                ..Default::default()
            })
            .into()
    }

    fn view_section_header(&self, title: &'static str) -> Element<'static, SettingsMessage> {
        let p = theme::palette();
        text(title).size(11).color(p.text_muted).into()
    }

    fn view_theme_setting(&self) -> Element<'_, SettingsMessage> {
        let p = theme::palette();
        let current_theme = self.settings.theme;

        let themes = vec![
            AppTheme::Light,
            AppTheme::Steam,
            AppTheme::GtkDark,
            AppTheme::Windows11,
            AppTheme::MacOS,
            AppTheme::HighContrast,
        ];

        container(
            row![
                column![
                    text("Theme").size(14).color(p.text_primary),
                    Space::new().height(4),
                    text("Visual style and color palette")
                        .size(11)
                        .color(p.text_secondary),
                ]
                .width(Fill),
                pick_list(themes, Some(current_theme), SettingsMessage::ChangeTheme)
                    .text_size(13)
                    .padding([8, 12]),
            ]
            .align_y(Alignment::Center)
            .padding(14),
        )
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_card)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn view_icon_theme_setting(&self) -> Element<'_, SettingsMessage> {
        let p = theme::palette();
        let use_svg = self.settings.icon_theme == IconTheme::Svg;

        let description = if use_svg {
            "High quality SVG icons"
        } else {
            "Emoji icons (minimal memory)"
        };

        container(
            row![
                column![
                    text("Icon Style").size(14).color(p.text_primary),
                    Space::new().height(4),
                    text(description).size(11).color(p.text_secondary),
                ]
                .width(Fill),
                toggler(use_svg)
                    .on_toggle(SettingsMessage::ToggleIconTheme)
                    .size(20),
            ]
            .align_y(Alignment::Center)
            .padding(14),
        )
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_card)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn view_font_scale_setting(&self) -> Element<'_, SettingsMessage> {
        let p = theme::palette();
        let scale = self.settings.font_scale;

        // Format scale as percentage
        let scale_text = format!("{}%", (scale * 100.0) as i32);

        container(
            column![
                row![
                    text("Text Size").size(14).color(p.text_primary),
                    Space::new().width(Fill),
                    text(scale_text).size(12).color(p.text_secondary),
                ]
                .align_y(Alignment::Center),
                Space::new().height(8),
                text("Affects notifications and sidebar")
                    .size(11)
                    .color(p.text_muted),
                Space::new().height(12),
                iced::widget::slider(0.8..=1.5, scale, SettingsMessage::SetFontScale).step(0.05),
            ]
            .padding(14),
        )
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_card)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn view_minimize_to_tray_setting(&self) -> Element<'_, SettingsMessage> {
        let p = theme::palette();
        let enabled = self.settings.minimize_to_tray;

        let description = if enabled {
            "App stays in system tray when closed"
        } else {
            "App exits when closed"
        };

        container(
            row![
                column![
                    text("Minimize to Tray").size(14).color(p.text_primary),
                    Space::new().height(4),
                    text(description).size(11).color(p.text_secondary),
                ]
                .width(Fill),
                toggler(enabled)
                    .on_toggle(SettingsMessage::ToggleMinimizeToTray)
                    .size(20),
            ]
            .align_y(Alignment::Center)
            .padding(14),
        )
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_card)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn view_accounts_section(&self) -> Element<'_, SettingsMessage> {
        let p = theme::palette();

        if self.settings.accounts.is_empty() {
            return container(text("No accounts added yet").size(12).color(p.text_muted))
                .padding(14)
                .into();
        }

        let mut col = column![].spacing(8);

        for account in &self.settings.accounts {
            col = col.push(self.view_account_item(account));
        }

        col.into()
    }

    fn view_account_item(&self, account: &StoredAccount) -> Element<'static, SettingsMessage> {
        let p = theme::palette();
        let icon_theme = self.settings.icon_theme;
        let status_color = if account.is_active {
            p.accent_success
        } else {
            p.text_muted
        };

        let status_text = if account.is_active { "Active" } else { "" };
        let username = account.username.clone();
        let username_for_button = account.username.clone();

        container(
            row![
                icons::icon_user(14.0, p.text_secondary, icon_theme),
                Space::new().width(8),
                text(username).size(13).color(p.text_primary),
                Space::new().width(8),
                text(status_text).size(10).color(status_color),
                Space::new().width(Fill),
                button(icons::icon_trash(14.0, p.text_muted, icon_theme))
                    .style(theme::ghost_button)
                    .padding(6)
                    .on_press(SettingsMessage::RemoveAccount(username_for_button)),
            ]
            .align_y(Alignment::Center)
            .padding(14),
        )
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_card)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }
}

// Display impl for pick_list
impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Light => "Light",
                Self::Steam => "Steam Dark",
                Self::GtkDark => "GTK Adwaita",
                Self::Windows11 => "Windows 11",
                Self::MacOS => "macOS",
                Self::HighContrast => "High Contrast",
            }
        )
    }
}
