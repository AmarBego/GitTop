//! General tab - consolidated appearance and behavior settings.

use iced::widget::{Space, column, pick_list, row, slider, text, toggler};
use iced::{Alignment, Element, Fill};

use crate::settings::{AppSettings, AppTheme, IconTheme};
use crate::ui::theme;

use super::super::components::{setting_card, tab_title};
use super::super::messages::SettingsMessage;

pub fn view(settings: &AppSettings, start_on_boot_enabled: bool) -> Element<'_, SettingsMessage> {
    let p = theme::palette();

    column![
        tab_title("General"),
        text("Appearance and behavior preferences.")
            .size(12)
            .color(p.text_secondary),
        Space::new().height(16),
        view_theme(settings),
        Space::new().height(8),
        view_icons(settings),
        Space::new().height(8),
        view_minimize_to_tray(settings),
        Space::new().height(8),
        view_start_on_boot(start_on_boot_enabled),
        Space::new().height(24),
        text("Display").size(13).color(p.text_muted),
        Space::new().height(8),
        view_notification_scale(settings),
        Space::new().height(8),
        view_sidebar_scale(settings),
        Space::new().height(8),
        view_sidebar_width(settings),
        Space::new().height(24),
        text("Behavior").size(13).color(p.text_muted),
        Space::new().height(8),
        view_refresh_interval(settings),
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
                row![
                    text("Theme").size(14).color(p.text_primary),
                    text(format!(" (Default: {})", AppTheme::platform_default()))
                        .size(11)
                        .color(p.text_muted),
                ],
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
    let use_svg = settings.icon_theme == IconTheme::Svg;
    let desc = if use_svg {
        "High quality SVG icons"
    } else {
        "Emoji icons (minimal memory)"
    };

    toggle_card(
        "Icon Style",
        true, // default: SVG icons enabled
        desc,
        use_svg,
        SettingsMessage::ToggleIconTheme,
    )
}

fn view_minimize_to_tray(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let enabled = settings.minimize_to_tray;
    let desc = if enabled {
        "App stays in system tray when closed"
    } else {
        "App exits when closed"
    };

    toggle_card(
        "Minimize to Tray",
        true, // default: enabled
        desc,
        enabled,
        SettingsMessage::ToggleMinimizeToTray,
    )
}

fn view_start_on_boot(start_on_boot_enabled: bool) -> Element<'static, SettingsMessage> {
    let desc = if start_on_boot_enabled {
        "GitTop starts when you log in"
    } else {
        "GitTop does not start automatically"
    };

    toggle_card(
        "Start on Boot",
        false, // default: disabled
        desc,
        start_on_boot_enabled,
        SettingsMessage::ToggleStartOnBoot,
    )
}

fn view_notification_scale(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let scale = settings.notification_font_scale;
    slider_card(
        "Notification Text Size",
        "100%",
        format!("{}%", (scale * 100.0) as i32),
        0.8..=1.5,
        scale,
        0.05,
        SettingsMessage::SetNotificationFontScale,
    )
}

fn view_sidebar_scale(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let scale = settings.sidebar_font_scale;
    slider_card(
        "Sidebar Text Size",
        "100%",
        format!("{}%", (scale * 100.0) as i32),
        0.8..=1.5,
        scale,
        0.05,
        SettingsMessage::SetSidebarFontScale,
    )
}

fn view_sidebar_width(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let width = settings.sidebar_width;
    slider_card(
        "Sidebar Width",
        "220px",
        format!("{}px", width as i32),
        180.0..=400.0,
        width,
        10.0,
        SettingsMessage::SetSidebarWidth,
    )
}

// ============================================================================
// Helpers
// ============================================================================

fn toggle_card<'a>(
    title: &'static str,
    default_on: bool,
    description: &'a str,
    is_toggled: bool,
    on_toggle: impl Fn(bool) -> SettingsMessage + 'a,
) -> Element<'a, SettingsMessage> {
    let p = theme::palette();
    let default_text = if default_on { "On" } else { "Off" };

    setting_card(
        row![
            column![
                row![
                    text(title).size(14).color(p.text_primary),
                    text(format!(" (Default: {})", default_text))
                        .size(11)
                        .color(p.text_muted),
                ],
                Space::new().height(4),
                text(description).size(11).color(p.text_secondary),
            ]
            .width(Fill),
            toggler(is_toggled).on_toggle(on_toggle).size(20),
        ]
        .align_y(Alignment::Center),
    )
}

fn slider_card<'a>(
    title: &'static str,
    default_text: &'static str,
    value_text: String,
    range: std::ops::RangeInclusive<f32>,
    value: f32,
    step: f32,
    on_change: impl Fn(f32) -> SettingsMessage + 'a,
) -> Element<'a, SettingsMessage> {
    let p = theme::palette();

    setting_card(column![
        row![
            row![
                text(title).size(14).color(p.text_primary),
                text(format!(" (Default: {})", default_text))
                    .size(11)
                    .color(p.text_muted),
            ],
            Space::new().width(Fill),
            text(value_text).size(12).color(p.text_secondary),
        ]
        .align_y(Alignment::Center),
        Space::new().height(12),
        slider(range, value, on_change).step(step),
    ])
}

fn view_refresh_interval(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let secs = settings.refresh_interval_secs;
    let p = theme::palette();

    setting_card(column![
        row![
            column![
                row![
                    text("Refresh Interval").size(14).color(p.text_primary),
                    text(" (Default: 60s)").size(11).color(p.text_muted),
                ],
                Space::new().height(4),
                text("How often to check for new notifications")
                    .size(11)
                    .color(p.text_secondary),
            ]
            .width(Fill),
            text(format!("{}s", secs)).size(12).color(p.text_secondary),
        ]
        .align_y(Alignment::Center),
        Space::new().height(12),
        slider(15.0..=300.0, secs as f32, |v| {
            SettingsMessage::SetRefreshInterval(v as u64)
        })
        .step(15.0),
    ])
}
