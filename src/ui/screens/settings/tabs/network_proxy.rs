//! Network proxy settings tab.

use iced::widget::{column, row, text, text_input, toggler, Space};
use iced::{Alignment, Element, Fill};

use crate::settings::AppSettings;
use crate::ui::{icons, theme};

use super::super::components::{setting_card, tab_title};
use super::super::messages::SettingsMessage;

/// View for network proxy settings
pub fn view(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();

    column![
        tab_title("Network Proxy"),
        text("Configure proxy settings for GitHub API requests.")
            .size(12)
            .color(p.text_secondary),
        Space::new().height(16),
        view_proxy_enabled(settings),
        Space::new().height(8),
        view_proxy_url(settings),
        Space::new().height(8),
        view_proxy_auth(settings),
    ]
    .spacing(4)
    .padding(24)
    .width(Fill)
    .into()
}

/// Proxy enabled toggle card
fn view_proxy_enabled(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let enabled = settings.proxy.enabled;
    let desc = if enabled {
        "Proxy will be used for all GitHub API requests"
    } else {
        "Direct connection to GitHub API"
    };

    setting_card(
        row![
            column![
                text("Enable Network Proxy").size(14).color(p.text_primary),
                Space::new().height(4),
                text(desc).size(11).color(p.text_secondary),
            ]
            .width(Fill),
            toggler(enabled)
                .on_toggle(SettingsMessage::ToggleProxyEnabled)
                .size(24),
        ]
        .align_y(Alignment::Center),
    )
}

/// Proxy URL input card
fn view_proxy_url(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();

    setting_card(column![
        row![
            text("Proxy URL").size(14).color(p.text_primary),
            Space::new().width(Fill),
            if settings.proxy.enabled {
                text("http://proxy.company.com:8080")
                    .size(11)
                    .color(p.text_muted)
            } else {
                text("Disabled").size(11).color(p.text_muted)
            },
        ]
        .align_y(Alignment::Center),
        Space::new().height(12),
        text_input("http://proxy.company.com:8080", &settings.proxy.url)
            .on_input(SettingsMessage::ProxyUrlChanged)
            .padding([8, 12])
            .size(13)
            .width(Fill)
            .style(theme::text_input_style),
    ])
}

/// Proxy authentication card (username and password)
fn view_proxy_auth(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();

    let has_auth = settings.proxy.username.is_some() || settings.proxy.password.is_some();

    setting_card(
        column![
            row![
                column![
                    text("Authentication").size(14).color(p.text_primary),
                    Space::new().height(4),
                    text(if has_auth {
                        "Proxy requires authentication"
                    } else {
                        "Optional: leave empty if not required"
                    })
                    .size(11)
                    .color(p.text_secondary),
                ]
                .width(Fill),
                if has_auth {
                    icons::icon_check(16.0, p.accent_success, settings.icon_theme)
                } else {
                    icons::icon_at(16.0, p.text_muted, settings.icon_theme)
                },
            ]
            .align_y(Alignment::Center),
            Space::new().height(16),
            row![
                text_input("Username", settings.proxy.username.as_deref().unwrap_or(""))
                    .on_input(SettingsMessage::ProxyUsernameChanged)
                    .padding([8, 12])
                    .size(13)
                    .width(Fill)
                    .style(theme::text_input_style),
                Space::new().width(8),
                text_input("Password", settings.proxy.password.as_deref().unwrap_or(""))
                    .secure(true)
                    .on_input(SettingsMessage::ProxyPasswordChanged)
                    .padding([8, 12])
                    .size(13)
                    .width(Fill)
                    .style(theme::text_input_style),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(4),
    )
}