//! Top Bar widget - Global context and command center.

use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Element, Fill};

use crate::github::UserInfo;
use crate::settings::IconTheme;
use crate::ui::screens::notifications::NotificationMessage;
use crate::ui::{icons, theme};

/// Account info for the account switcher.
#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub username: String,
}

pub fn view_top_bar<'a>(
    user: &'a UserInfo,
    accounts: Vec<AccountInfo>,
    is_loading: bool,
    unread_count: usize,
    show_all_filters: bool,
    icon_theme: IconTheme,
) -> Element<'a, NotificationMessage> {
    let p = theme::palette();

    // Context Switcher (Account / Workspace selector)
    let context_switch: Element<'_, NotificationMessage> = if accounts.len() > 1 {
        // Dropdown for switching
        let account_names: Vec<String> = accounts.iter().map(|a| a.username.clone()).collect();

        iced::widget::pick_list(
            account_names,
            Some(user.login.clone()),
            NotificationMessage::SwitchAccount,
        )
        .text_size(13)
        .padding([4, 8])
        .style(theme::pick_list_style)
        .menu_style(theme::menu_style)
        .into()
    } else {
        // No switcher if only one account, just show text in profile area
        Space::new().width(0).into()
    };

    // Settings Button
    let settings_btn = button(icons::icon_settings(16.0, p.text_secondary, icon_theme))
        .on_press(NotificationMessage::OpenSettings)
        .style(theme::ghost_button)
        .padding(6);

    // Profile section (only show if single account, otherwise pick_list shows username)
    let profile_section: Element<'_, NotificationMessage> = if accounts.len() > 1 {
        Space::new().width(0).into()
    } else {
        row![
            // Vertical Divider
            container(Space::new().width(1).height(16)).style(move |_| container::Style {
                background: Some(iced::Background::Color(p.border_subtle)),
                ..Default::default()
            }),
            text(&user.login).size(13).color(p.text_secondary),
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .into()
    };

    // --- Middle Section: Notification Controls ---

    // 1. Sync Status / Refresh
    let sync_status: Element<'_, NotificationMessage> = if is_loading {
        row![
            icons::icon_refresh(14.0, p.text_muted, icon_theme),
            Space::new().width(6),
            text("Syncing...").size(12).color(p.text_muted),
        ]
        .align_y(Alignment::Center)
        .into()
    } else {
        button(
            row![
                icons::icon_refresh(14.0, p.text_secondary, icon_theme),
                Space::new().width(6),
                text("Refresh").size(12).color(p.text_secondary),
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::ghost_button)
        .padding([4, 8])
        .on_press(NotificationMessage::Refresh)
        .into()
    };

    // 2. Filter Toggle (Unread | All)
    let is_unread_filter = !show_all_filters;

    let unread_btn = button(text("Unread").size(12))
        .style(theme::segment_button(is_unread_filter))
        .padding([4, 10])
        .on_press(NotificationMessage::ToggleShowAll);

    let all_btn = button(text("All").size(12))
        .style(theme::segment_button(!is_unread_filter))
        .padding([4, 10])
        .on_press(NotificationMessage::ToggleShowAll);

    let filter_segment =
        container(row![unread_btn, all_btn].spacing(0)).style(theme::segment_container);

    // 3. Mark All Read
    let mark_read: Element<'_, NotificationMessage> = if unread_count > 0 {
        button(
            row![
                icons::icon_check(14.0, p.accent_success, icon_theme),
                Space::new().width(6),
                text("Mark all read").size(12).color(p.accent_success),
            ]
            .align_y(Alignment::Center),
        )
        .style(theme::ghost_button)
        .padding([4, 8])
        .on_press(NotificationMessage::MarkAllAsRead)
        .into()
    } else {
        Space::new().width(0).into()
    };

    // Middle container
    let middle_controls = row![
        filter_segment,
        Space::new().width(16),
        sync_status,
        Space::new().width(16),
        mark_read
    ]
    .align_y(Alignment::Center);

    container(
        row![
            // Left: Header Label (GitTop Logo in Power Mode)
            row![
                icons::icon_brand(18.0, p.accent, icon_theme),
                text("GitTop").size(16).color(p.text_primary)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
            Space::new().width(Fill),
            // Middle: Controls
            middle_controls,
            Space::new().width(Fill),
            // Right: Controls
            row![context_switch, settings_btn, profile_section,]
                .spacing(12)
                .align_y(Alignment::Center),
        ]
        .align_y(Alignment::Center)
        .padding([8, 16])
        .spacing(16),
    )
    .width(Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(p.bg_card)),
        border: iced::Border {
            width: 0.0,
            color: p.border_subtle,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}
