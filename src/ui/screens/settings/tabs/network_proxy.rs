//! Network proxy settings tab.

use iced::widget::{Space, button, column, container, row, text, text_input, toggler};
use iced::{Alignment, Element, Fill, Length};

use crate::ui::{icons, theme};

use super::super::components::{setting_card, tab_title};
use super::super::messages::SettingsMessage;
use super::super::screen::SettingsScreen;

/// View for network proxy settings
pub fn view(screen: &SettingsScreen) -> Element<'_, SettingsMessage> {
    let p = theme::palette();

    column![
        tab_title("Network Proxy"),
        text("Configure proxy settings for GitHub API requests.")
            .size(12)
            .color(p.text_secondary),
        Space::new().height(16),
        view_proxy_enabled(screen),
        Space::new().height(8),
        view_proxy_configuration(screen),
    ]
    .spacing(4)
    .padding(24)
    .width(Fill)
    .into()
}

/// Proxy enabled toggle card
fn view_proxy_enabled(screen: &SettingsScreen) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let enabled = screen.proxy_enabled;
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

/// Proxy configuration card (URL and authentication combined)
fn view_proxy_configuration(screen: &SettingsScreen) -> Element<'_, SettingsMessage> {
    let p = theme::palette();

    let has_auth = screen.settings.proxy.has_credentials;
    let has_unsaved = screen.has_unsaved_proxy_changes();

    setting_card(
        column![
            // Proxy URL section
            row![
                text("Proxy URL").size(14).color(p.text_primary),
                Space::new().width(Fill),
            ]
            .align_y(Alignment::Center),
            Space::new().height(12),
            text_input("http://proxy.company.com:8080", &screen.proxy_url)
                .on_input(SettingsMessage::ProxyUrlChanged)
                .padding([8, 12])
                .size(13)
                .width(Fill)
                .style(theme::text_input_style),
            Space::new().height(12),
            // Separator
            container(Space::new().height(1))
                .width(Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(theme::palette().border_subtle,)),
                    ..Default::default()
                }),
            Space::new().height(10),
            // Authentication section
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
                    icons::icon_check(16.0, p.accent_success, screen.settings.icon_theme)
                } else {
                    icons::icon_at(16.0, p.text_muted, screen.settings.icon_theme)
                },
            ]
            .align_y(Alignment::Center),
            Space::new().height(16),
            row![
                text_input("Username", &screen.proxy_username)
                    .on_input(SettingsMessage::ProxyUsernameChanged)
                    .padding([8, 12])
                    .size(13)
                    .width(Fill)
                    .style(theme::text_input_style),
                Space::new().width(8),
                text_input("Password", &screen.proxy_password)
                    .secure(true)
                    .on_input(SettingsMessage::ProxyPasswordChanged)
                    .padding([8, 12])
                    .size(13)
                    .width(Fill)
                    .style(theme::text_input_style),
            ]
            .align_y(Alignment::Center),
            Space::new().height(10),
            // Save button
            row![
                Space::new().width(Fill),
                button(text("Save").size(13).width(Fill).align_x(Alignment::Center))
                    .style(if has_unsaved {
                        theme::primary_button
                    } else {
                        theme::ghost_button
                    })
                    .on_press(SettingsMessage::SaveProxySettings)
                    .width(Length::Fixed(60.0))
                    .padding(6),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(4),
    )
}
