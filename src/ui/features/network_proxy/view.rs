use iced::widget::{Space, button, column, container, row, text, text_input, toggler};
use iced::{Alignment, Element, Fill, Length};

use crate::settings::AppSettings;
use crate::ui::screens::settings::components::{setting_card, tab_title};
use crate::ui::{icons, theme};

use super::message::ProxyMessage;
use super::state::NetworkProxyState;

/// Check if proxy settings have unsaved changes
fn has_unsaved_changes(state: &NetworkProxyState, settings: &AppSettings) -> bool {
    let enabled_changed = state.enabled != settings.proxy.enabled;
    let url_changed = state.url != settings.proxy.url;
    let new_has_creds = !state.username.is_empty() || !state.password.is_empty();
    let creds_status_changed = new_has_creds != settings.proxy.has_credentials;

    enabled_changed || url_changed || creds_status_changed || state.creds_dirty
}

/// View for network proxy settings
pub fn view<'a>(
    state: &'a NetworkProxyState,
    settings: &'a AppSettings,
) -> Element<'a, ProxyMessage> {
    let p = theme::palette();

    column![
        tab_title("Network Proxy"),
        text("Configure proxy settings for GitHub API requests.")
            .size(12)
            .color(p.text_secondary),
        Space::new().height(16),
        view_proxy_enabled(state),
        Space::new().height(8),
        view_proxy_configuration(state, settings),
    ]
    .spacing(4)
    .padding(24)
    .width(Fill)
    .into()
}

/// Proxy enabled toggle card
fn view_proxy_enabled(state: &NetworkProxyState) -> Element<'_, ProxyMessage> {
    let p = theme::palette();
    let enabled = state.enabled;
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
                .on_toggle(ProxyMessage::ToggleEnabled)
                .size(24),
        ]
        .align_y(Alignment::Center),
    )
}

/// Proxy configuration card (URL and authentication combined)
fn view_proxy_configuration<'a>(
    state: &'a NetworkProxyState,
    settings: &'a AppSettings,
) -> Element<'a, ProxyMessage> {
    let p = theme::palette();

    let has_auth = settings.proxy.has_credentials;
    let has_unsaved = has_unsaved_changes(state, settings);

    setting_card(
        column![
            // Proxy URL section
            row![
                text("Proxy URL").size(14).color(p.text_primary),
                Space::new().width(Fill),
            ]
            .align_y(Alignment::Center),
            Space::new().height(12),
            text_input("http://proxy.company.com:8080", &state.url)
                .on_input(ProxyMessage::UrlChanged)
                .padding([8, 12])
                .size(13)
                .width(Fill)
                .style(theme::text_input_style),
            Space::new().height(12),
            // Separator
            container(Space::new().height(1))
                .width(Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(p.border_subtle)),
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
                    icons::icon_check(16.0, p.accent_success, settings.icon_theme)
                } else {
                    icons::icon_at(16.0, p.text_muted, settings.icon_theme)
                },
            ]
            .align_y(Alignment::Center),
            Space::new().height(16),
            row![
                text_input("Username", &state.username)
                    .on_input(ProxyMessage::UsernameChanged)
                    .padding([8, 12])
                    .size(13)
                    .width(Fill)
                    .style(theme::text_input_style),
                Space::new().width(8),
                text_input("Password", &state.password)
                    .secure(true)
                    .on_input(ProxyMessage::PasswordChanged)
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
                    .on_press(ProxyMessage::Save)
                    .width(Length::Fixed(60.0))
                    .padding(6),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(4),
    )
}
