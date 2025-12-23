//! Sidebar component - navigation and filtering.

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Element, Fill, Length, Padding};

use crate::github::{SubjectType, UserInfo};
use crate::settings::IconTheme;
use crate::ui::screens::notifications::messages::NotificationMessage;
use crate::ui::{icons, theme};

use super::sidebar_state::SidebarState;

/// Render the sidebar.
pub fn view_sidebar<'a>(state: SidebarState<'a>) -> Element<'a, NotificationMessage> {
    // Scrollable content (branding, types, repos)
    let mut scrollable_content = column![];

    // Only show branding in sidebar if NOT in power mode (it's in top bar otherwise)
    if !state.power_mode {
        scrollable_content = scrollable_content
            .push(view_branding(state.icon_theme))
            .push(Space::new().height(16));
    }

    let scrollable_content = scrollable_content
        .push(view_types_section(
            state.type_counts,
            state.selected_type,
            state.total_count,
            state.icon_theme,
        ))
        .push(Space::new().height(16))
        .push(view_repos_section(
            state.repo_counts,
            state.selected_repo,
            state.icon_theme,
        ))
        .spacing(0)
        .padding([16, 12]);

    // Main layout: scrollable area + user section pinned at bottom
    let content: Element<'a, NotificationMessage> = if state.power_mode {
        // In power mode, user info is in top bar, so just show scrollable content
        // We still wrap in clear container for consistent background if needed
        container(
            scrollable(scrollable_content)
                .height(Fill)
                .style(theme::scrollbar),
        )
        .width(Fill)
        .height(Fill)
        .into()
    } else {
        column![
            scrollable(scrollable_content)
                .height(Fill)
                .style(theme::scrollbar),
            // User section pinned at the bottom
            container(view_user_section(
                state.user,
                &state.accounts,
                state.icon_theme,
            ))
            .padding(Padding {
                top: 0.0,
                right: 12.0,
                bottom: 16.0,
                left: 12.0,
            }),
        ]
        .height(Fill)
        .into()
    };

    container(content)
        .width(Length::Fixed(state.width.clamp(180.0, 400.0)))
        .height(Fill)
        .style(theme::sidebar)
        .into()
}

fn view_branding<'a>(icon_theme: IconTheme) -> Element<'a, NotificationMessage> {
    let p = theme::palette();
    row![
        icons::icon_brand(20.0, p.accent, icon_theme),
        Space::new().width(8),
        text("GitTop").size(18).color(p.text_primary),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn view_types_section(
    type_counts: &[(SubjectType, usize)],
    selected_type: Option<SubjectType>,
    total_count: usize,
    icon_theme: IconTheme,
) -> Element<'static, NotificationMessage> {
    let p = theme::palette();

    let mut col = column![
        text("Types")
            .size(theme::sidebar_scaled(11.0))
            .color(p.text_secondary),
        Space::new().height(8),
        // "All" option
        sidebar_item(
            icons::icon_inbox(14.0, p.text_primary, icon_theme),
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
            p.accent
        } else {
            p.text_primary
        };
        col = col.push(sidebar_item(
            subject_type_icon(*subject_type, icon_color, icon_theme),
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
    icon_theme: IconTheme,
) -> Element<'static, NotificationMessage> {
    let p = theme::palette();

    let mut col = column![
        text("Repositories")
            .size(theme::sidebar_scaled(11.0))
            .color(p.text_secondary),
        Space::new().height(8),
    ]
    .spacing(2);

    // Limit to top 10 repos
    for (repo, count) in repo_counts.iter().take(10) {
        let is_selected = selected_repo == Some(repo.as_str());
        let icon_color = if is_selected {
            p.accent
        } else {
            p.text_primary
        };
        // Extract just the repo name (after the /)
        let short_name = repo.split('/').next_back().unwrap_or(repo).to_owned();
        col = col.push(sidebar_item(
            icons::icon_folder(14.0, icon_color, icon_theme),
            short_name,
            *count,
            is_selected,
            NotificationMessage::SelectRepo(Some(repo.clone())),
        ));
    }

    if repo_counts.is_empty() {
        col = col.push(text("No repositories").size(11).color(p.text_muted));
    }

    col.into()
}

fn view_user_section<'a>(
    user: &'a UserInfo,
    accounts: &[String],
    icon_theme: IconTheme,
) -> Element<'a, NotificationMessage> {
    let p = theme::palette();

    // Account selector or just label
    let account_control: Element<'_, _, _, iced::Renderer> = if accounts.len() > 1 {
        // Dropdown for switching
        iced::widget::pick_list(
            accounts.to_vec(),
            Some(user.login.clone()),
            NotificationMessage::SwitchAccount,
        )
        .text_size(13)
        .padding([4, 8])
        .style(theme::pick_list_style)
        .into()
    } else {
        // Just the username
        text(&user.login).size(13).color(p.text_primary).into()
    };

    column![
        container(Space::new().height(1))
            .width(Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(p.border)),
                ..Default::default()
            }),
        Space::new().height(12),
        row![
            icons::icon_user(14.0, p.text_secondary, icon_theme),
            Space::new().width(8),
            account_control,
            Space::new().width(Fill), // Push buttons to the right
            button(icons::icon_settings(14.0, p.text_muted, icon_theme))
                .style(theme::ghost_button)
                .padding([6, 8])
                .on_press(NotificationMessage::OpenSettings),
            button(icons::icon_power(14.0, p.text_muted, icon_theme))
                .style(theme::ghost_button)
                .padding([6, 8])
                .on_press(NotificationMessage::Logout),
        ]
        .align_y(Alignment::Center),
    ]
    .into()
}

/// Get the icon for a subject type.
fn subject_type_icon(
    t: SubjectType,
    color: iced::Color,
    icon_theme: IconTheme,
) -> Element<'static, NotificationMessage> {
    match t {
        SubjectType::Issue => icons::icon_issue(14.0, color, icon_theme),
        SubjectType::PullRequest => icons::icon_pull_request(14.0, color, icon_theme),
        SubjectType::Release => icons::icon_release(14.0, color, icon_theme),
        SubjectType::Discussion => icons::icon_discussion(14.0, color, icon_theme),
        SubjectType::CheckSuite => icons::icon_check_suite(14.0, color, icon_theme),
        SubjectType::Commit => icons::icon_commit(14.0, color, icon_theme),
        SubjectType::RepositoryVulnerabilityAlert => icons::icon_security(14.0, color, icon_theme),
        SubjectType::Unknown => icons::icon_unknown(14.0, color, icon_theme),
    }
}

/// Sidebar item with icon element, label, count, and selection state.
fn sidebar_item<'a>(
    icon: Element<'a, NotificationMessage>,
    label: String,
    count: usize,
    is_selected: bool,
    on_press: NotificationMessage,
) -> Element<'a, NotificationMessage> {
    let p = theme::palette();
    // Use primary text for all items - much more readable
    let text_color = p.text_primary;

    // Use scaled font sizes (f32 for iced Pixels)
    let label_size = theme::sidebar_scaled(13.0);
    let count_size = theme::sidebar_scaled(11.0);

    let content = row![
        icon,
        Space::new().width(8),
        text(label).size(label_size).color(text_color),
        Space::new().width(Fill),
        text(format!("{}", count))
            .size(count_size)
            .color(p.text_secondary),
    ]
    .align_y(Alignment::Center)
    .padding([8, 10]);

    button(content)
        .style(move |theme, status| (theme::sidebar_button(is_selected))(theme, status))
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
