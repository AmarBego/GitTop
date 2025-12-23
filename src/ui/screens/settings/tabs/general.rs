//! General tab - consolidated appearance and behavior settings.

use iced::widget::{column, pick_list, row, slider, text, toggler, Space};
use iced::{Alignment, Element, Fill};

use crate::settings::{AppSettings, AppTheme, IconTheme};
use crate::ui::theme;

use super::super::components::{setting_card, tab_title};
use super::super::messages::SettingsMessage;

/// Render the general tab content.
pub fn view(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();

    column![
        tab_title("General"),
        text("Appearance and behavior preferences.")
            .size(12)
            .color(p.text_secondary),
        Space::new().height(16),
        // Theme
        view_theme(settings),
        Space::new().height(8),
        // Icon Style
        view_icons(settings),
        Space::new().height(8),
        // Minimize to Tray
        view_minimize_to_tray(settings),
        Space::new().height(24),
        // Section: Display
        text("Display").size(13).color(p.text_muted),
        Space::new().height(8),
        view_notification_scale(settings),
        Space::new().height(8),
        view_sidebar_scale(settings),
        Space::new().height(8),
        view_sidebar_width(settings),
    ]
    .spacing(4)
    .padding(24)
    .width(Fill)
    .into()
}

fn view_theme(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let themes = [
        AppTheme::Light,
        AppTheme::Steam,
        AppTheme::GtkDark,
        AppTheme::Windows11,
        AppTheme::MacOS,
        AppTheme::HighContrast,
    ];

    setting_card(
        row![
            column![
                text("Theme").size(14).color(p.text_primary),
                Space::new().height(4),
                text("Choose your preferred color scheme")
                    .size(11)
                    .color(p.text_secondary),
            ]
            .width(Fill),
            pick_list(themes, Some(settings.theme), SettingsMessage::ChangeTheme)
                .text_size(13)
                .padding([8, 12])
                .style(theme::pick_list_style)
                .menu_style(theme::menu_style),
        ]
        .align_y(Alignment::Center),
    )
}

fn view_icons(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let use_svg = settings.icon_theme == IconTheme::Svg;
    let desc = if use_svg {
        "High quality SVG icons"
    } else {
        "Emoji icons (minimal memory)"
    };

    setting_card(
        row![
            column![
                text("Icon Style").size(14).color(p.text_primary),
                Space::new().height(4),
                text(desc).size(11).color(p.text_secondary),
            ]
            .width(Fill),
            toggler(use_svg)
                .on_toggle(SettingsMessage::ToggleIconTheme)
                .size(20),
        ]
        .align_y(Alignment::Center),
    )
}

fn view_minimize_to_tray(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let enabled = settings.minimize_to_tray;
    let desc = if enabled {
        "App stays in system tray when closed"
    } else {
        "App exits when closed"
    };

    setting_card(
        row![
            column![
                text("Minimize to Tray").size(14).color(p.text_primary),
                Space::new().height(4),
                text(desc).size(11).color(p.text_secondary),
            ]
            .width(Fill),
            toggler(enabled)
                .on_toggle(SettingsMessage::ToggleMinimizeToTray)
                .size(20),
        ]
        .align_y(Alignment::Center),
    )
}

fn view_notification_scale(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let scale = settings.notification_font_scale;

    setting_card(column![
        row![
            text("Notification Text Size")
                .size(14)
                .color(p.text_primary),
            Space::new().width(Fill),
            text(format!("{}%", (scale * 100.0) as i32))
                .size(12)
                .color(p.text_secondary),
        ]
        .align_y(Alignment::Center),
        Space::new().height(12),
        slider(0.8..=1.5, scale, SettingsMessage::SetNotificationFontScale).step(0.05),
    ])
}

fn view_sidebar_scale(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let scale = settings.sidebar_font_scale;

    setting_card(column![
        row![
            text("Sidebar Text Size").size(14).color(p.text_primary),
            Space::new().width(Fill),
            text(format!("{}%", (scale * 100.0) as i32))
                .size(12)
                .color(p.text_secondary),
        ]
        .align_y(Alignment::Center),
        Space::new().height(12),
        slider(0.8..=1.5, scale, SettingsMessage::SetSidebarFontScale).step(0.05),
    ])
}

fn view_sidebar_width(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let width = settings.sidebar_width;

    setting_card(column![
        row![
            text("Sidebar Width").size(14).color(p.text_primary),
            Space::new().width(Fill),
            text(format!("{}px", width as i32))
                .size(12)
                .color(p.text_secondary),
        ]
        .align_y(Alignment::Center),
        Space::new().height(12),
        slider(180.0..=400.0, width, SettingsMessage::SetSidebarWidth).step(10.0),
    ])
}
