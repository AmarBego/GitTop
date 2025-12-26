//! Power Mode tab - enterprise features and Rule Engine.

use iced::widget::{Space, button, column, container, row, text, toggler};
use iced::{Alignment, Element, Fill};

use crate::settings::AppSettings;
use crate::ui::{icons, theme};

use super::super::components::tab_title;
use super::super::messages::SettingsMessage;

pub fn view(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let enabled = settings.power_mode;

    let mut content = column![
        tab_title("Power Mode"),
        text("Unlock powerful notification management.")
            .size(12)
            .color(p.text_secondary),
        Space::new().height(20),
        view_hero(settings),
    ]
    .spacing(4);

    if enabled {
        content = content
            .push(Space::new().height(24))
            .push(view_features(settings))
            .push(Space::new().height(24))
            .push(view_rule_engine(settings));
    } else {
        content = content
            .push(Space::new().height(24))
            .push(view_disabled_features(settings));
    }

    content.padding(24).width(Fill).into()
}

/// Hero card with prominent Power Mode toggle.
fn view_hero(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let enabled = settings.power_mode;
    let icon_theme = settings.icon_theme;

    let (status_color, status_text) = if enabled {
        (p.accent_success, "ACTIVE")
    } else {
        (p.text_muted, "INACTIVE")
    };

    let icon_size = 32.0;
    let icon_color = if enabled { p.accent } else { p.text_muted };

    container(
        row![
            // Left: Large icon + text
            row![
                container(icons::icon_zap(icon_size, icon_color, icon_theme))
                    .padding(12)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(icon_color.scale_alpha(0.1))),
                        border: iced::Border {
                            radius: 12.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                Space::new().width(16),
                column![
                    text("Power Mode").size(18).color(p.text_primary),
                    Space::new().height(4),
                    row![
                        container(text(status_text).size(10).color(status_color))
                            .padding([2, 8])
                            .style(move |_| container::Style {
                                background: Some(iced::Background::Color(
                                    status_color.scale_alpha(0.15)
                                )),
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                        Space::new().width(8),
                        text(if enabled {
                            "Advanced layout enabled"
                        } else {
                            "Standard layout"
                        })
                        .size(12)
                        .color(p.text_secondary),
                    ]
                    .align_y(Alignment::Center),
                ],
            ]
            .align_y(Alignment::Center),
            Space::new().width(Fill),
            // Right: Toggle
            toggler(enabled)
                .on_toggle(SettingsMessage::TogglePowerMode)
                .size(24),
        ]
        .align_y(Alignment::Center)
        .padding(20),
    )
    .width(Fill)
    .style(move |_| {
        let (border_color, border_width) = if enabled {
            (p.accent.scale_alpha(0.5), 2.0)
        } else {
            (p.border_subtle, 1.0)
        };
        container::Style {
            background: Some(iced::Background::Color(p.bg_card)),
            border: iced::Border {
                radius: 12.0.into(),
                width: border_width,
                color: border_color,
            },
            ..Default::default()
        }
    })
    .into()
}

/// Feature cards shown when Power Mode is enabled.
fn view_features(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let icon_theme = settings.icon_theme;

    column![
        text("Enabled Features").size(13).color(p.text_muted),
        Space::new().height(12),
        feature_card(
            "Multi-Pane Layout",
            "Split view with sidebar, list, and detail panels",
            icons::icon_chart(18.0, p.accent, icon_theme),
            p
        ),
        Space::new().height(8),
        feature_card(
            "Rule Engine",
            "Automated actions based on notification rules",
            icons::icon_settings(18.0, p.accent_warning, icon_theme),
            p
        ),
        Space::new().height(8),
        feature_card(
            "Bulk Actions",
            "Select and manage multiple notifications at once",
            icons::icon_inbox(18.0, p.accent_success, icon_theme),
            p
        ),
    ]
    .into()
}

/// Single feature card.
fn feature_card<'a>(
    title: &'static str,
    description: &'static str,
    icon: Element<'a, SettingsMessage>,
    p: theme::ThemePalette,
) -> Element<'a, SettingsMessage> {
    container(
        row![
            container(icon).padding(8).style(move |_| container::Style {
                background: Some(iced::Background::Color(p.bg_base)),
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::new().width(12),
            column![
                text(title).size(13).color(p.text_primary),
                Space::new().height(2),
                text(description).size(11).color(p.text_secondary),
            ]
            .width(Fill),
        ]
        .align_y(Alignment::Center)
        .padding(12),
    )
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(p.bg_card)),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: p.border_subtle,
        },
        ..Default::default()
    })
    .into()
}

/// Disabled feature preview (grayed out).
fn view_disabled_features(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let icon_theme = settings.icon_theme;

    container(
        column![
            row![
                icons::icon_zap(16.0, p.text_muted, icon_theme),
                Space::new().width(8),
                text("Features available with Power Mode")
                    .size(13)
                    .color(p.text_muted),
            ]
            .align_y(Alignment::Center),
            Space::new().height(16),
            disabled_feature_row("Multi-Pane Layout", p),
            Space::new().height(8),
            disabled_feature_row("Rule Engine Automation", p),
            Space::new().height(8),
            disabled_feature_row("Bulk Actions", p),
        ]
        .padding(16),
    )
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(p.bg_card.scale_alpha(0.5))),
        border: iced::Border {
            radius: 8.0.into(),
            width: 1.0,
            color: p.border_subtle,
        },
        ..Default::default()
    })
    .into()
}

fn disabled_feature_row(
    label: &'static str,
    p: theme::ThemePalette,
) -> Element<'static, SettingsMessage> {
    row![
        container(Space::new().width(6).height(6)).style(move |_| container::Style {
            background: Some(iced::Background::Color(p.text_muted.scale_alpha(0.5))),
            border: iced::Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }),
        Space::new().width(12),
        text(label).size(12).color(p.text_muted),
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Rule Engine section with prominent action button.
fn view_rule_engine(settings: &AppSettings) -> Element<'_, SettingsMessage> {
    let p = theme::palette();
    let icon_theme = settings.icon_theme;

    container(
        column![
            row![
                container(icons::icon_filter(20.0, p.accent, icon_theme))
                    .padding(10)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(p.accent.scale_alpha(0.1))),
                        border: iced::Border {
                            radius: 10.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                Space::new().width(14),
                column![
                    text("Rule Engine").size(16).color(p.text_primary),
                    Space::new().height(4),
                    text("Create rules to automatically categorize, pin, snooze, or archive notifications.")
                        .size(12)
                        .color(p.text_secondary),
                ]
                .width(Fill),
            ]
            .align_y(Alignment::Center),
            Space::new().height(16),
            container(Space::new().height(1))
                .width(Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(p.border_subtle)),
                    ..Default::default()
                }),
            Space::new().height(16),
            button(
                row![
                    icons::icon_external_link(14.0, iced::Color::WHITE, icon_theme),
                    Space::new().width(10),
                    text("Configure Rules").size(14).color(iced::Color::WHITE),
                ]
                .align_y(Alignment::Center),
            )
            .style(theme::primary_button)
            .padding([12, 24])
            .on_press(SettingsMessage::OpenRuleEngine),
        ]
        .padding(20),
    )
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(p.bg_card)),
        border: iced::Border {
            radius: 12.0.into(),
            width: 1.0,
            color: p.accent.scale_alpha(0.3),
        },
        ..Default::default()
    })
    .into()
}
