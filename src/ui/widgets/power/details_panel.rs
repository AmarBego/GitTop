//! Details Panel widget - Contextual information for the selected notification.
//!
//! Displays fetched Issue/PR/Comment content inline when a notification is
//! clicked in power mode.

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Color, Element, Fill, Length};

use crate::github::subject_details::{
    CommentDetails, DiscussionDetails, IssueDetails, NotificationSubjectDetail, PullRequestDetails,
};
use crate::github::NotificationView;
use crate::settings::IconTheme;
use crate::ui::screens::notifications::NotificationMessage;
use crate::ui::{icons, theme};

/// View the details panel for a selected notification.
pub fn view_details_panel<'a>(
    notification: Option<&'a NotificationView>,
    details: Option<&'a NotificationSubjectDetail>,
    is_loading: bool,
    icon_theme: IconTheme,
) -> Element<'a, NotificationMessage> {
    let p = theme::palette();

    let content: Element<'a, NotificationMessage> = if is_loading {
        view_loading(&p)
    } else if let Some(notif) = notification {
        if let Some(detail) = details {
            view_details(notif, detail, icon_theme, &p)
        } else {
            view_notification_header(notif, &p, icon_theme)
        }
    } else {
        view_empty_state(&p)
    };

    container(content)
        .width(Length::Fixed(380.0))
        .height(Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_base)),
            border: iced::Border {
                width: 1.0,
                color: p.border_subtle,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
}

fn view_loading<'a>(p: &theme::ThemePalette) -> Element<'a, NotificationMessage> {
    column![
        Space::new().height(Fill),
        text("Loading...").size(14).color(p.text_muted),
        Space::new().height(Fill),
    ]
    .align_x(Alignment::Center)
    .width(Fill)
    .height(Fill)
    .into()
}

fn view_empty_state<'a>(p: &theme::ThemePalette) -> Element<'a, NotificationMessage> {
    column![
        Space::new().height(Fill),
        text("Select a notification").size(14).color(p.text_muted),
        Space::new().height(8),
        text("Click a notification to view details")
            .size(12)
            .color(p.text_muted),
        Space::new().height(Fill),
    ]
    .align_x(Alignment::Center)
    .width(Fill)
    .height(Fill)
    .into()
}

fn view_notification_header<'a>(
    notif: &'a NotificationView,
    p: &theme::ThemePalette,
    icon_theme: IconTheme,
) -> Element<'a, NotificationMessage> {
    column![
        text(&notif.repo_full_name).size(12).color(p.text_secondary),
        Space::new().height(4),
        text(&notif.title).size(18).color(p.text_primary),
        Space::new().height(16),
        text(format!("Reason: {}", notif.reason.label()))
            .size(12)
            .color(p.text_muted),
        Space::new().height(16),
        view_open_button(icon_theme),
    ]
    .padding(24)
    .width(Fill)
    .into()
}

fn view_details<'a>(
    notif: &'a NotificationView,
    detail: &'a NotificationSubjectDetail,
    icon_theme: IconTheme,
    p: &theme::ThemePalette,
) -> Element<'a, NotificationMessage> {
    let content: Element<'a, NotificationMessage> = match detail {
        NotificationSubjectDetail::Issue(issue) => view_issue(issue, notif, icon_theme, p),
        NotificationSubjectDetail::PullRequest(pr) => view_pull_request(pr, notif, icon_theme, p),
        NotificationSubjectDetail::Comment {
            comment,
            context_title,
        } => view_comment(comment, context_title, notif, icon_theme, p),
        NotificationSubjectDetail::Discussion(discussion) => {
            view_discussion(discussion, notif, icon_theme, p)
        }
        NotificationSubjectDetail::SecurityAlert { title, severity } => {
            view_security_alert(title, severity.as_deref(), notif, icon_theme, p)
        }
        NotificationSubjectDetail::Unsupported { subject_type } => {
            view_unsupported(subject_type, notif, icon_theme, p)
        }
    };

    scrollable(content)
        .height(Fill)
        .width(Fill)
        .style(theme::scrollbar)
        .into()
}

fn view_issue<'a>(
    issue: &'a IssueDetails,
    notif: &'a NotificationView,
    icon_theme: IconTheme,
    p: &theme::ThemePalette,
) -> Element<'a, NotificationMessage> {
    let state_color = if issue.state == "open" {
        p.accent_success
    } else {
        p.accent_danger
    };

    let bg_control = p.bg_control;
    let border_subtle = p.border_subtle;
    let text_secondary = p.text_secondary;
    let text_primary = p.text_primary;
    let text_muted = p.text_muted;

    let mut col = column![
        row![
            icons::icon_issue(14.0, state_color, icon_theme),
            Space::new().width(8),
            text(format!("#{}", issue.number))
                .size(14)
                .color(text_secondary),
            Space::new().width(8),
            text(&issue.state).size(12).color(state_color),
        ]
        .align_y(Alignment::Center),
        Space::new().height(8),
        text(&issue.title).size(16).color(text_primary),
        Space::new().height(4),
        text(format!("Opened by @{}", issue.user.login))
            .size(11)
            .color(text_muted),
        Space::new().height(16),
    ]
    .width(Fill);

    if let Some(body) = &issue.body {
        if !body.is_empty() {
            let truncated = truncate_text(body, 1500);
            col = col.push(
                container(text(truncated).size(13).color(text_secondary))
                    .padding(12)
                    .width(Fill)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(bg_control)),
                        border: iced::Border {
                            radius: 6.0.into(),
                            color: border_subtle,
                            width: 1.0,
                        },
                        ..Default::default()
                    }),
            );
            col = col.push(Space::new().height(16));
        }
    }

    if !issue.labels.is_empty() {
        let mut label_row = row![].spacing(4);
        for label in issue.labels.iter().take(5) {
            label_row = label_row.push(view_label(&label.name, &label.color));
        }
        col = col.push(label_row);
        col = col.push(Space::new().height(16));
    }

    col = col.push(
        text(format!("{} comments", issue.comments_count))
            .size(12)
            .color(text_muted),
    );
    col = col.push(Space::new().height(16));
    col = col.push(view_action_buttons(&notif.id, notif.unread, icon_theme));

    col.padding(24).into()
}

fn view_pull_request<'a>(
    pr: &'a PullRequestDetails,
    notif: &'a NotificationView,
    icon_theme: IconTheme,
    p: &theme::ThemePalette,
) -> Element<'a, NotificationMessage> {
    let state_color = if pr.merged {
        p.accent_purple
    } else if pr.state == "open" {
        p.accent_success
    } else {
        p.accent_danger
    };

    let state_text = if pr.merged {
        "merged"
    } else {
        pr.state.as_str()
    };

    let bg_control = p.bg_control;
    let border_subtle = p.border_subtle;
    let text_secondary = p.text_secondary;
    let text_primary = p.text_primary;
    let text_muted = p.text_muted;
    let accent_success = p.accent_success;
    let accent_danger = p.accent_danger;

    let mut col = column![
        row![
            icons::icon_pull_request(14.0, state_color, icon_theme),
            Space::new().width(8),
            text(format!("#{}", pr.number))
                .size(14)
                .color(text_secondary),
            Space::new().width(8),
            text(state_text).size(12).color(state_color),
        ]
        .align_y(Alignment::Center),
        Space::new().height(8),
        text(&pr.title).size(16).color(text_primary),
        Space::new().height(4),
        text(format!("Opened by @{}", pr.user.login))
            .size(11)
            .color(text_muted),
        Space::new().height(16),
    ]
    .width(Fill);

    if let Some(body) = &pr.body {
        if !body.is_empty() {
            let truncated = truncate_text(body, 1500);
            col = col.push(
                container(text(truncated).size(13).color(text_secondary))
                    .padding(12)
                    .width(Fill)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(bg_control)),
                        border: iced::Border {
                            radius: 6.0.into(),
                            color: border_subtle,
                            width: 1.0,
                        },
                        ..Default::default()
                    }),
            );
            col = col.push(Space::new().height(16));
        }
    }

    col = col.push(
        row![
            view_stat_badge(format!("+{}", pr.additions), accent_success),
            Space::new().width(8),
            view_stat_badge(format!("-{}", pr.deletions), accent_danger),
            Space::new().width(8),
            text(format!("{} files", pr.changed_files))
                .size(12)
                .color(text_muted),
            Space::new().width(8),
            text(format!("{} commits", pr.commits))
                .size(12)
                .color(text_muted),
        ]
        .align_y(Alignment::Center),
    );
    col = col.push(Space::new().height(16));
    col = col.push(view_action_buttons(&notif.id, notif.unread, icon_theme));

    col.padding(24).into()
}

fn view_comment<'a>(
    comment: &'a CommentDetails,
    context_title: &'a str,
    notif: &'a NotificationView,
    icon_theme: IconTheme,
    p: &theme::ThemePalette,
) -> Element<'a, NotificationMessage> {
    let bg_control = p.bg_control;
    let border_subtle = p.border_subtle;
    let text_primary = p.text_primary;
    let text_secondary = p.text_secondary;
    let accent = p.accent;

    column![
        row![
            icons::icon_at(14.0, accent, icon_theme),
            Space::new().width(8),
            text(format!("Mentioned by @{}", comment.user.login))
                .size(14)
                .color(text_primary),
        ]
        .align_y(Alignment::Center),
        Space::new().height(8),
        text(context_title).size(13).color(text_secondary),
        Space::new().height(16),
        container(text(&comment.body).size(13).color(text_primary))
            .padding(12)
            .width(Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg_control)),
                border: iced::Border {
                    radius: 6.0.into(),
                    color: border_subtle,
                    width: 1.0,
                },
                ..Default::default()
            }),
        Space::new().height(16),
        view_action_buttons(&notif.id, notif.unread, icon_theme),
    ]
    .padding(24)
    .width(Fill)
    .into()
}

fn view_discussion<'a>(
    discussion: &'a DiscussionDetails,
    notif: &'a NotificationView,
    icon_theme: IconTheme,
    p: &theme::ThemePalette,
) -> Element<'a, NotificationMessage> {
    let text_primary = p.text_primary;
    let text_secondary = p.text_secondary;
    let text_muted = p.text_muted;
    let accent = p.accent;
    let accent_success = p.accent_success;
    let bg_control = p.bg_control;
    let border_subtle = p.border_subtle;

    // Build category display with emoji
    let category_label = discussion
        .category
        .as_ref()
        .map(|c| {
            if let Some(emoji) = &c.emoji {
                format!("{} {}", emoji, c.name)
            } else {
                c.name.clone()
            }
        })
        .unwrap_or_else(|| "Discussion".to_string());

    // Start building the column
    let mut col = column![
        // Repo name
        text(&notif.repo_full_name).size(11).color(text_muted),
        Space::new().height(6),
    ]
    .width(Fill);

    // Header row with icon and category
    let mut header_row = row![
        icons::icon_discussion(14.0, accent, icon_theme),
        Space::new().width(8),
        text(category_label).size(12).color(text_secondary),
    ]
    .spacing(0)
    .align_y(Alignment::Center);

    // Add answered badge if applicable
    if discussion.answer_chosen {
        header_row = header_row.push(Space::new().width(8));
        header_row = header_row.push(
            container(text("✓ Answered").size(10).color(accent_success))
                .padding([2, 6])
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        accent_success.r,
                        accent_success.g,
                        accent_success.b,
                        0.15,
                    ))),
                    border: iced::Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        );
    }

    col = col.push(header_row);
    col = col.push(Space::new().height(8));

    // Title
    col = col.push(text(&discussion.title).size(16).color(text_primary));
    col = col.push(Space::new().height(4));

    // Author
    if let Some(author) = &discussion.author {
        col = col.push(
            text(format!("Started by @{}", author))
                .size(11)
                .color(text_muted),
        );
    }
    col = col.push(Space::new().height(16));

    // Body
    if let Some(body) = &discussion.body {
        if !body.is_empty() {
            let truncated = truncate_text(body, 1500);
            col = col.push(
                container(text(truncated).size(13).color(text_secondary))
                    .padding(12)
                    .width(Fill)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(bg_control)),
                        border: iced::Border {
                            radius: 6.0.into(),
                            color: border_subtle,
                            width: 1.0,
                        },
                        ..Default::default()
                    }),
            );
            col = col.push(Space::new().height(16));
        }
    }

    // Comments count
    if discussion.comments_count > 0 {
        col = col.push(
            text(format!("{} comments", discussion.comments_count))
                .size(12)
                .color(text_muted),
        );
        col = col.push(Space::new().height(16));
    }

    // Action buttons
    col = col.push(view_action_buttons(&notif.id, notif.unread, icon_theme));
    col.padding(24).into()
}

fn view_security_alert<'a>(
    title: &'a str,
    severity: Option<&'a str>,
    notif: &'a NotificationView,
    icon_theme: IconTheme,
    p: &theme::ThemePalette,
) -> Element<'a, NotificationMessage> {
    let severity_color = match severity {
        Some("critical" | "high") => p.accent_danger,
        Some("moderate" | "medium") => p.accent_warning,
        _ => p.text_muted,
    };
    let text_primary = p.text_primary;
    let text_muted = p.text_muted;
    let accent_danger = p.accent_danger;

    column![
        row![
            icons::icon_security(14.0, accent_danger, icon_theme),
            Space::new().width(8),
            text("Security Alert").size(14).color(accent_danger),
        ]
        .align_y(Alignment::Center),
        Space::new().height(8),
        text(title).size(16).color(text_primary),
        Space::new().height(8),
        if let Some(sev) = severity {
            text(format!("Severity: {}", sev))
                .size(12)
                .color(severity_color)
        } else {
            text("").size(12)
        },
        Space::new().height(16),
        text("Security alert details are not available via the API.")
            .size(11)
            .color(text_muted),
        text("Click below to view on GitHub.")
            .size(11)
            .color(text_muted),
        Space::new().height(16),
        view_action_buttons(&notif.id, notif.unread, icon_theme),
    ]
    .padding(24)
    .width(Fill)
    .into()
}

fn view_unsupported<'a>(
    subject_type: &'a str,
    notif: &'a NotificationView,
    icon_theme: IconTheme,
    p: &theme::ThemePalette,
) -> Element<'a, NotificationMessage> {
    column![
        text(&notif.repo_full_name).size(12).color(p.text_secondary),
        Space::new().height(4),
        text(&notif.title).size(16).color(p.text_primary),
        Space::new().height(16),
        text(format!("Type: {}", subject_type))
            .size(12)
            .color(p.text_muted),
        Space::new().height(8),
        text("Detailed view not available for this notification type.")
            .size(11)
            .color(p.text_muted),
        Space::new().height(16),
        view_action_buttons(&notif.id, notif.unread, icon_theme),
    ]
    .padding(24)
    .width(Fill)
    .into()
}

// =============================================================================
// Helper widgets
// =============================================================================

/// Action buttons row with Mark as Read, Delete, and Open in GitHub
fn view_action_buttons(
    notification_id: &str,
    is_unread: bool,
    icon_theme: IconTheme,
) -> Element<'static, NotificationMessage> {
    let p = theme::palette();
    let id = notification_id.to_string();
    let id_for_done = id.clone();

    let mut buttons_row = row![].spacing(8);

    // Mark as Read button (only show if unread)
    if is_unread {
        buttons_row = buttons_row.push(view_action_button(
            "Mark as Read",
            p.accent_success,
            icons::icon_check(12.0, p.accent_success, icon_theme),
            NotificationMessage::MarkAsRead(id.clone()),
        ));
    }

    // Delete/Done button (removes from GitHub inbox)
    buttons_row = buttons_row.push(view_action_button(
        "Delete",
        p.accent_danger,
        icons::icon_trash(12.0, p.accent_danger, icon_theme),
        NotificationMessage::MarkAsDone(id_for_done),
    ));

    // Open in GitHub button
    buttons_row = buttons_row.push(view_open_in_github_button(icon_theme));

    buttons_row.into()
}

fn view_action_button(
    label: &'static str,
    color: Color,
    icon: Element<'static, NotificationMessage>,
    message: NotificationMessage,
) -> Element<'static, NotificationMessage> {
    let p = theme::palette();
    let bg_hover = p.bg_hover;
    let bg_active = p.bg_active;
    let border_subtle = p.border_subtle;

    button(
        row![
            icon,
            Space::new().width(6),
            text(label).size(12).color(color),
        ]
        .align_y(Alignment::Center),
    )
    .style(move |_theme, status| {
        let bg = match status {
            button::Status::Hovered => bg_hover,
            button::Status::Pressed => bg_active,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: color,
            border: iced::Border {
                radius: 6.0.into(),
                color: border_subtle,
                width: 1.0,
            },
            ..Default::default()
        }
    })
    .padding([8, 12])
    .on_press(message)
    .into()
}

fn view_open_in_github_button(icon_theme: IconTheme) -> Element<'static, NotificationMessage> {
    let p = theme::palette();
    let text_color = p.accent;
    let bg_hover = p.bg_hover;
    let bg_active = p.bg_active;
    let border_subtle = p.border_subtle;

    button(
        row![
            icons::icon_external_link(12.0, text_color, icon_theme),
            Space::new().width(6),
            text("Open in GitHub").size(12).color(text_color),
        ]
        .align_y(Alignment::Center),
    )
    .style(move |_theme, status| {
        let bg = match status {
            button::Status::Hovered => bg_hover,
            button::Status::Pressed => bg_active,
            _ => Color::TRANSPARENT,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color,
            border: iced::Border {
                radius: 6.0.into(),
                color: border_subtle,
                width: 1.0,
            },
            ..Default::default()
        }
    })
    .padding([8, 12])
    .on_press(NotificationMessage::OpenInBrowser)
    .into()
}

/// Simple open button for basic views (backward compat)
fn view_open_button(icon_theme: IconTheme) -> Element<'static, NotificationMessage> {
    view_open_in_github_button(icon_theme)
}

fn view_label(name: &str, hex_color: &str) -> Element<'static, NotificationMessage> {
    let p = theme::palette();
    let color = parse_hex_color(hex_color).unwrap_or(p.text_muted);
    let owned_name = name.to_string();
    container(text(owned_name).size(10).color(color))
        .padding([2, 6])
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                color.r, color.g, color.b, 0.15,
            ))),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn view_stat_badge(text_content: String, color: Color) -> Element<'static, NotificationMessage> {
    container(text(text_content).size(11).color(color))
        .padding([2, 6])
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                color.r, color.g, color.b, 0.15,
            ))),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::from_rgb8(r, g, b))
}

fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}
