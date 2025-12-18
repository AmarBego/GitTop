//! Sidebar component - navigation and filtering.

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Fill, Length, Padding};

use super::screen::NotificationMessage;
use crate::github::{SubjectType, UserInfo};
use crate::ui::{icons, theme};

/// Sidebar width constant.
pub const SIDEBAR_WIDTH: u16 = 220;

/// Render the sidebar.
pub fn view_sidebar<'a>(
    user: &'a UserInfo,
    type_counts: &[(SubjectType, usize)],
    repo_counts: &[(String, usize)],
    selected_type: Option<SubjectType>,
    selected_repo: Option<&str>,
    total_count: usize,
) -> Element<'a, NotificationMessage> {
    // Scrollable content (branding, types, repos)
    let scrollable_content = column![
        // App branding
        view_branding(),
        Space::new().height(16),
        // Types section
        view_types_section(type_counts, selected_type, total_count),
        Space::new().height(16),
        // Repositories section
        view_repos_section(repo_counts, selected_repo),
    ]
    .spacing(0)
    .padding([16, 12]);

    // Main layout: scrollable area + user section pinned at bottom
    let content = column![
        scrollable(scrollable_content)
            .height(Fill)
            .style(theme::scrollbar),
        // User section pinned at the bottom
        container(view_user_section(user)).padding(Padding {
            top: 0.0,
            right: 12.0,
            bottom: 16.0,
            left: 12.0,
        }),
    ]
    .height(Fill);

    container(content)
        .width(Length::Fixed(SIDEBAR_WIDTH as f32))
        .height(Fill)
        .style(sidebar_style)
        .into()
}

fn view_branding<'a>() -> Element<'a, NotificationMessage> {
    row![
        icons::icon_brand(20.0, theme::ACCENT_BLUE),
        Space::new().width(8),
        text("GitTop").size(18).color(theme::TEXT_PRIMARY),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn view_types_section(
    type_counts: &[(SubjectType, usize)],
    selected_type: Option<SubjectType>,
    total_count: usize,
) -> Element<'static, NotificationMessage> {
    let mut col = column![
        text("Types").size(11).color(theme::TEXT_MUTED),
        Space::new().height(8),
        // "All" option
        sidebar_item_svg(
            icons::icon_inbox(14.0, theme::TEXT_SECONDARY),
            "All".to_owned(),
            total_count,
            selected_type.is_none(),
            NotificationMessage::SelectType(None),
        ),
    ]
    .spacing(2);

    // Add type items
    for (subject_type, count) in type_counts {
        let is_selected = selected_type == Some(*subject_type);
        let icon_color = if is_selected {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_SECONDARY
        };
        col = col.push(sidebar_item_svg(
            subject_type_icon(*subject_type, icon_color),
            subject_type_label(*subject_type).to_owned(),
            *count,
            is_selected,
            NotificationMessage::SelectType(Some(*subject_type)),
        ));
    }

    col.into()
}

fn view_repos_section(
    repo_counts: &[(String, usize)],
    selected_repo: Option<&str>,
) -> Element<'static, NotificationMessage> {
    let mut col = column![
        text("Repositories").size(11).color(theme::TEXT_MUTED),
        Space::new().height(8),
    ]
    .spacing(2);

    // Limit to top 10 repos
    for (repo, count) in repo_counts.iter().take(10) {
        let is_selected = selected_repo == Some(repo.as_str());
        let icon_color = if is_selected {
            theme::TEXT_PRIMARY
        } else {
            theme::TEXT_SECONDARY
        };
        // Extract just the repo name (after the /)
        let short_name = repo.split('/').next_back().unwrap_or(repo).to_owned();
        col = col.push(sidebar_item_svg(
            icons::icon_folder(14.0, icon_color),
            short_name,
            *count,
            is_selected,
            NotificationMessage::SelectRepo(Some(repo.clone())),
        ));
    }

    if repo_counts.is_empty() {
        col = col.push(text("No repositories").size(11).color(theme::TEXT_MUTED));
    }

    col.into()
}

fn view_user_section<'a>(user: &'a UserInfo) -> Element<'a, NotificationMessage> {
    column![
        container(Space::new().height(1))
            .width(Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(theme::BORDER)),
                ..Default::default()
            }),
        Space::new().height(12),
        row![
            icons::icon_user(14.0, theme::TEXT_SECONDARY),
            Space::new().width(8),
            text(&user.login).size(12).color(theme::TEXT_SECONDARY),
            Space::new().width(Fill),
            button(icons::icon_power(12.0, theme::TEXT_MUTED))
                .style(theme::ghost_button)
                .padding([4, 8])
                .on_press(NotificationMessage::Logout),
        ]
        .align_y(Alignment::Center),
    ]
    .into()
}

/// Get the SVG icon for a subject type.
fn subject_type_icon(t: SubjectType, color: iced::Color) -> iced::widget::Svg<'static> {
    match t {
        SubjectType::Issue => icons::icon_issue(14.0, color),
        SubjectType::PullRequest => icons::icon_pull_request(14.0, color),
        SubjectType::Release => icons::icon_release(14.0, color),
        SubjectType::Discussion => icons::icon_discussion(14.0, color),
        SubjectType::CheckSuite => icons::icon_check_suite(14.0, color),
        SubjectType::Commit => icons::icon_commit(14.0, color),
        SubjectType::RepositoryVulnerabilityAlert => icons::icon_security(14.0, color),
        SubjectType::Unknown => icons::icon_unknown(14.0, color),
    }
}

/// Sidebar item with SVG icon, label, count, and selection state.
fn sidebar_item_svg<'a>(
    icon: iced::widget::Svg<'a>,
    label: String,
    count: usize,
    is_selected: bool,
    on_press: NotificationMessage,
) -> Element<'a, NotificationMessage> {
    let text_color = if is_selected {
        theme::TEXT_PRIMARY
    } else {
        theme::TEXT_SECONDARY
    };

    let content = row![
        icon,
        Space::new().width(8),
        text(label).size(12).color(text_color),
        Space::new().width(Fill),
        text(format!("{}", count)).size(11).color(theme::TEXT_MUTED),
    ]
    .align_y(Alignment::Center)
    .padding([6, 8]);

    button(content)
        .style(move |theme, status| sidebar_button_style(theme, status, is_selected))
        .on_press(on_press)
        .width(Fill)
        .into()
}

fn subject_type_label(t: SubjectType) -> &'static str {
    match t {
        SubjectType::PullRequest => "Pull requests",
        SubjectType::Issue => "Issues",
        SubjectType::Commit => "Commits",
        SubjectType::CheckSuite => "Workflows",
        SubjectType::Discussion => "Discussions",
        SubjectType::Release => "Releases",
        SubjectType::RepositoryVulnerabilityAlert => "Security",
        SubjectType::Unknown => "Other",
    }
}

fn sidebar_style(_: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(theme::BG_CARD)),
        border: iced::Border {
            color: theme::BORDER,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn sidebar_button_style(
    _: &iced::Theme,
    status: button::Status,
    is_selected: bool,
) -> button::Style {
    let bg = if is_selected {
        theme::BG_CONTROL
    } else {
        match status {
            button::Status::Hovered => theme::BG_HOVER,
            button::Status::Pressed => theme::BG_CONTROL,
            _ => iced::Color::TRANSPARENT,
        }
    };

    button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: theme::TEXT_PRIMARY,
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}
