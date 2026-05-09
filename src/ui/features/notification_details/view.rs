use iced::widget::{Space, button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Fill, Length};

use crate::github::NotificationView;
use crate::github::subject_details::{
    CheckRun, CommentDetails, DiscussionDetails, IssueDetails, Label, NotificationSubjectDetail,
    PullRequestDetails,
};
use crate::settings::IconTheme;
use crate::ui::features::notification_details::NotificationDetailsMessage;
use crate::ui::features::thread_actions::ThreadActionMessage;
use crate::ui::screens::notifications::messages::NotificationMessage;
use crate::ui::{icons, theme};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct LabelViewArgs<'a> {
    pub input: &'a str,
    pub available: &'a [Label],
    pub loading: bool,
    pub pending_ops: &'a HashSet<String>,
    pub error: Option<&'a str>,
    pub check_runs: &'a [CheckRun],
    pub checks_loading: bool,
    pub comments: Option<&'a [CommentDetails]>,
    pub comments_loading: bool,
    pub comments_error: Option<&'a str>,
}

pub fn view<'a>(
    notification: Option<&'a NotificationView>,
    details: Option<&'a NotificationSubjectDetail>,
    is_loading: bool,
    icon_theme: IconTheme,
    label_args: LabelViewArgs<'a>,
) -> Element<'a, NotificationMessage> {
    let p = theme::palette();

    let content: Element<'a, NotificationMessage> = if is_loading {
        view_loading(&p)
    } else if let Some(notif) = notification {
        if let Some(detail) = details {
            view_details(notif, detail, icon_theme, &p, label_args)
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
    label_args: LabelViewArgs<'a>,
) -> Element<'a, NotificationMessage> {
    let content: Element<'a, NotificationMessage> = match detail {
        NotificationSubjectDetail::Issue(issue) => {
            view_issue(issue, notif, icon_theme, p, label_args)
        }
        NotificationSubjectDetail::PullRequest(pr) => {
            view_pull_request(pr, notif, icon_theme, p, label_args)
        }
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
    label_args: LabelViewArgs<'a>,
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

    if let Some(body) = &issue.body
        && !body.is_empty()
    {
        let truncated = truncate_text(body, 1500);
        col = col.push(
            container(
                text(truncated)
                    .size(13)
                    .color(text_secondary)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            )
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

    if !issue.labels.is_empty() {
        let mut wrap = iced_aw::Wrap::new().spacing(4.0).line_spacing(4.0);
        for label in issue.labels.iter() {
            wrap = wrap.push(view_label(&label.name, &label.color));
        }
        col = col.push(wrap);
        col = col.push(Space::new().height(16));
    }

    col = col.push(
        text(format!("{} comments", issue.comments_count))
            .size(12)
            .color(text_muted),
    );
    col = col.push(Space::new().height(16));
    col = col.push(view_action_buttons(&notif.id, notif.unread, icon_theme));
    col = col.push(Space::new().height(24));
    col = col.push(view_comments_section(
        notif.url.as_deref().unwrap_or(""),
        label_args.comments,
        label_args.comments_loading,
        label_args.comments_error,
        *p,
        icon_theme,
    ));

    col.padding(24).into()
}

fn view_pull_request<'a>(
    pr: &'a PullRequestDetails,
    notif: &'a NotificationView,
    icon_theme: IconTheme,
    p: &theme::ThemePalette,
    label_args: LabelViewArgs<'a>,
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

    let (owner, repo) = split_repo_full_name(&notif.repo_full_name);
    let pr_number = pr.number;

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

    if let Some(body) = &pr.body
        && !body.is_empty()
    {
        let truncated = truncate_text(body, 1500);
        col = col.push(
            container(
                text(truncated)
                    .size(13)
                    .color(text_secondary)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            )
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

    if !pr.labels.is_empty() {
        let mut wrap = iced_aw::Wrap::new().spacing(4.0).line_spacing(4.0);

        for label in &pr.labels {
            let is_pending = label_args.pending_ops.contains(&label.name);
            let label_element = view_pr_label(
                &label.name,
                &label.color,
                is_pending,
                owner.to_string(),
                repo.to_string(),
                pr_number,
            );
            wrap = wrap.push(label_element);
        }
        col = col.push(wrap);
        col = col.push(Space::new().height(16));
    }

    // Add Label UI
    col = col.push(view_label_editor(
        label_args, &pr.labels, owner, repo, pr_number, p, icon_theme,
    ));
    col = col.push(Space::new().height(16));

    // Add Checks UI
    col = col.push(view_checks(
        label_args.check_runs,
        label_args.checks_loading,
        owner,
        repo,
        &pr.head.sha,
        p,
        icon_theme,
    ));
    col = col.push(Space::new().height(16));

    if let Some(err) = label_args.error {
        col = col.push(text(err).size(11).color(p.accent_danger));
        col = col.push(Space::new().height(8));
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
    col = col.push(Space::new().height(24));
    col = col.push(view_comments_section(
        notif.url.as_deref().unwrap_or(""),
        label_args.comments,
        label_args.comments_loading,
        label_args.comments_error,
        *p,
        icon_theme,
    ));

    col.padding(24).into()
}

fn view_checks<'a>(
    check_runs: &'a [CheckRun],
    loading: bool,
    owner: &str,
    repo: &str,
    sha: &str,
    p: &theme::ThemePalette,
    icon_theme: IconTheme,
) -> Element<'a, NotificationMessage> {
    let mut col = column![].spacing(8);

    let refresh_area: Element<'a, NotificationMessage> = if loading {
        text("Refreshing...").size(10).color(p.text_muted).into()
    } else {
        button(text("Refresh").size(10))
            .padding([2, 8])
            .style(theme::ghost_button)
            .on_press(NotificationMessage::Details(
                NotificationDetailsMessage::RefreshChecks(
                    owner.to_string(),
                    repo.to_string(),
                    sha.to_string(),
                ),
            ))
            .into()
    };

    let header = row![
        text("Checks").size(12).color(p.text_muted),
        Space::new().width(Fill),
        refresh_area,
    ]
    .align_y(Alignment::Center);

    col = col.push(header);

    if check_runs.is_empty() {
        if !loading {
            col = col.push(text("No checks found").size(11).color(p.text_muted));
        }
    } else {
        let mut runs_col = column![].spacing(4);
        for run in check_runs {
            let icon = match run.conclusion.as_deref() {
                Some("success") => icons::icon_check(12.0, p.accent_success, icon_theme),
                Some("failure") => icons::icon_x(12.0, p.accent_danger, icon_theme),
                Some("neutral") => icons::icon_info(12.0, p.text_muted, icon_theme),
                _ => {
                    if run.status == "in_progress" {
                        icons::icon_refresh(12.0, p.accent_warning, icon_theme)
                    } else {
                        icons::icon_unknown(12.0, p.text_muted, icon_theme)
                    }
                }
            };

            runs_col = runs_col.push(
                row![
                    icon,
                    Space::new().width(8),
                    container(
                        text(&run.name)
                            .size(11)
                            .color(p.text_secondary)
                            .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                    )
                    .width(Fill),
                    Space::new().width(8),
                    text(&run.status).size(10).color(p.text_muted),
                ]
                .align_y(Alignment::Center),
            );
        }
        col = col.push(runs_col);
    }

    col.into()
}

fn view_label_editor<'a>(
    label_args: LabelViewArgs<'a>,
    current_labels: &[Label],
    owner: &'a str,
    repo: &'a str,
    pr_number: u64,
    p: &theme::ThemePalette,
    _icon_theme: IconTheme,
) -> Element<'a, NotificationMessage> {
    let input = label_args.input;
    let available_labels = label_args.available;
    let loading = label_args.loading;

    let current_names: HashSet<&str> = current_labels.iter().map(|l| l.name.as_str()).collect();
    let suggestions: Vec<&Label> = available_labels
        .iter()
        .filter(|l| !current_names.contains(l.name.as_str()))
        .collect();

    let text_muted = p.text_muted;
    let bg_control = p.bg_control;
    let border_subtle = p.border_subtle;

    let mut col = column![].spacing(6);

    col = col.push(text("Labels").size(12).color(text_muted));

    let owner_string = owner.to_string();
    let repo_string = repo.to_string();

    col = col.push(
        text_input("Add a label...", input)
            .on_input(|text| {
                NotificationMessage::Details(NotificationDetailsMessage::SetLabelInput(text))
            })
            .on_submit({
                let owner = owner_string.clone();
                let repo = repo_string.clone();
                NotificationMessage::Details(NotificationDetailsMessage::SubmitLabelFromInput(
                    owner, repo, pr_number,
                ))
            })
            .padding([6, 10])
            .size(12),
    );

    if !input.trim().is_empty() || loading {
        if loading {
            col = col.push(text("Loading labels...").size(11).color(text_muted));
        } else if !suggestions.is_empty() {
            let filtered: Vec<&&Label> = if input.trim().is_empty() {
                suggestions.iter().take(8).collect()
            } else {
                let lower = input.trim().to_lowercase();
                suggestions
                    .iter()
                    .filter(|l| l.name.to_lowercase().contains(&lower))
                    .take(8)
                    .collect()
            };

            if !filtered.is_empty() {
                let mut sug_col = column![].spacing(2);
                for label in filtered {
                    let name = label.name.clone();
                    let owner = owner_string.clone();
                    let repo = repo_string.clone();
                    sug_col = sug_col.push(
                        button(
                            row![
                                view_label_swatch(&label.name, &label.color),
                                Space::new().width(6),
                                text(&label.name).size(11),
                            ]
                            .align_y(Alignment::Center),
                        )
                        .style(theme::ghost_button)
                        .padding([4, 8])
                        .on_press(NotificationMessage::Details(
                            NotificationDetailsMessage::SubmitAddLabel(
                                owner, repo, pr_number, name,
                            ),
                        )),
                    );
                }
                col = col.push(container(sug_col).padding(8).width(Fill).style(move |_| {
                    container::Style {
                        background: Some(iced::Background::Color(bg_control)),
                        border: iced::Border {
                            radius: 6.0.into(),
                            width: 1.0,
                            color: border_subtle,
                        },
                        ..Default::default()
                    }
                }));
            }
        }
    }

    col.into()
}

fn view_pr_label<'a>(
    name: &'a str,
    color: &'a str,
    is_pending: bool,
    owner: String,
    repo: String,
    pr_number: u64,
) -> Element<'a, NotificationMessage> {
    let p = theme::palette();
    let parsed = parse_hex_color(color).unwrap_or(p.text_muted);
    let opacity = if is_pending { 0.5 } else { 1.0 };

    button(
        row![
            text(name)
                .size(10)
                .color(Color::from_rgba(parsed.r, parsed.g, parsed.b, opacity,)),
            Space::new().width(4),
            text("×").size(10).color(Color::from_rgba(
                parsed.r,
                parsed.g,
                parsed.b,
                opacity * 0.7,
            )),
        ]
        .align_y(Alignment::Center),
    )
    .style(move |_, _| button::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            parsed.r,
            parsed.g,
            parsed.b,
            0.15 * opacity,
        ))),
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .padding([2, 6])
    .on_press(NotificationMessage::Details(
        NotificationDetailsMessage::RemoveLabel(
            owner.clone(),
            repo.clone(),
            pr_number,
            name.to_string(),
        ),
    ))
    .into()
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

    let mut col = column![
        text(&notif.repo_full_name).size(11).color(text_muted),
        Space::new().height(6),
    ]
    .width(Fill);

    let mut header_row = row![
        icons::icon_discussion(14.0, accent, icon_theme),
        Space::new().width(8),
        text(category_label).size(12).color(text_secondary),
    ]
    .spacing(0)
    .align_y(Alignment::Center);

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
    col = col.push(text(&discussion.title).size(16).color(text_primary));
    col = col.push(Space::new().height(4));

    if let Some(author) = &discussion.author {
        col = col.push(
            text(format!("Started by @{}", author))
                .size(11)
                .color(text_muted),
        );
    }
    col = col.push(Space::new().height(16));

    if let Some(body) = &discussion.body
        && !body.is_empty()
    {
        let truncated = truncate_text(body, 1500);
        col = col.push(
            container(
                text(truncated)
                    .size(13)
                    .color(text_secondary)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            )
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

    if discussion.comments_count > 0 {
        col = col.push(
            text(format!("{} comments", discussion.comments_count))
                .size(12)
                .color(text_muted),
        );
        col = col.push(Space::new().height(16));
    }

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

fn view_action_buttons(
    notification_id: &str,
    is_unread: bool,
    icon_theme: IconTheme,
) -> Element<'static, NotificationMessage> {
    let p = theme::palette();
    let id = notification_id.to_string();
    let id_for_done = id.clone();

    let mut buttons_row = row![].spacing(8);

    if is_unread {
        buttons_row = buttons_row.push(view_action_button(
            "Mark as Read",
            p.accent_success,
            icons::icon_check(12.0, p.accent_success, icon_theme),
            NotificationMessage::Thread(ThreadActionMessage::MarkAsRead(id.clone())),
        ));
    }

    buttons_row = buttons_row.push(view_action_button(
        "Delete",
        p.accent_danger,
        icons::icon_trash(12.0, p.accent_danger, icon_theme),
        NotificationMessage::Thread(ThreadActionMessage::MarkAsDone(id_for_done)),
    ));

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
    .on_press(NotificationMessage::Details(
        NotificationDetailsMessage::OpenInBrowser,
    ))
    .into()
}

fn view_open_button(icon_theme: IconTheme) -> Element<'static, NotificationMessage> {
    view_open_in_github_button(icon_theme)
}

fn view_label<'a>(name: &'a str, hex_color: &str) -> Element<'a, NotificationMessage> {
    let p = theme::palette();
    let color = parse_hex_color(hex_color).unwrap_or(p.text_muted);

    container(text(name).size(10).color(color))
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

fn view_label_swatch<'a>(name: &'a str, hex_color: &str) -> Element<'a, NotificationMessage> {
    let p = theme::palette();
    let color = parse_hex_color(hex_color).unwrap_or(p.text_muted);

    container(text(name).size(10).color(color))
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

fn truncate_text(text: &str, max_len: usize) -> std::borrow::Cow<'_, str> {
    if text.len() <= max_len {
        std::borrow::Cow::Borrowed(text)
    } else {
        let mut idx = max_len;
        while !text.is_char_boundary(idx) {
            idx -= 1;
        }
        std::borrow::Cow::Owned(format!("{}...", &text[..idx]))
    }
}

fn split_repo_full_name(full_name: &str) -> (&str, &str) {
    let mut parts = full_name.splitn(2, '/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(repo)) => (owner, repo),
        _ => (full_name, ""),
    }
}

fn view_comments_section<'a>(
    subject_url: &str,
    comments: Option<&'a [CommentDetails]>,
    loading: bool,
    error: Option<&'a str>,
    p: theme::ThemePalette,
    icon_theme: IconTheme,
) -> Element<'a, NotificationMessage> {
    let mut col = column![].spacing(8);

    col = col.push(text("Comments").size(14).color(p.text_primary));

    if let Some(err) = error {
        col = col.push(text(err).size(12).color(p.accent_danger));
    }

    match comments {
        Some(comment_list) => {
            if comment_list.is_empty() {
                col = col.push(text("No comments yet.").size(12).color(p.text_muted));
            } else {
                let mut comments_col = column![].spacing(16);
                for comment in comment_list {
                    let body = truncate_text(&comment.body, 1000);
                    comments_col = comments_col.push(
                        container(column![
                            row![
                                icons::icon_at(12.0, p.text_secondary, icon_theme),
                                Space::new().width(4),
                                text(&comment.user.login).size(12).color(p.text_secondary),
                            ]
                            .align_y(Alignment::Center),
                            Space::new().height(6),
                            text(body)
                                .size(13)
                                .color(p.text_primary)
                                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
                        ])
                        .padding(12)
                        .width(Fill)
                        .style(move |_| container::Style {
                            background: Some(iced::Background::Color(p.bg_control)),
                            border: iced::Border {
                                radius: 6.0.into(),
                                color: p.border_subtle,
                                width: 1.0,
                            },
                            ..Default::default()
                        }),
                    );
                }
                col = col.push(comments_col);
            }
        }
        None => {
            if loading {
                col = col.push(text("Loading comments...").size(12).color(p.text_muted));
            } else {
                col = col.push(
                    button(text("Load Comments").size(12))
                        .padding([6, 12])
                        .style(theme::ghost_button)
                        .on_press(NotificationMessage::Details(
                            NotificationDetailsMessage::LoadComments(subject_url.to_string()),
                        )),
                );
            }
        }
    }

    col.into()
}
