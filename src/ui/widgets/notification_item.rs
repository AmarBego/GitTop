//! Notification item widget - displays a single notification.
//!
//! Architecture:
//! - `NotificationVisualState`: Single source of truth for ALL visual decisions
//!   - Subject colors (Issue=green, PR=blue, etc.)
//!   - Card styling (background, border, accent bar)
//!   - State indicators (priority, silent)
//! - Widget builders: `account_badge()`, `priority_indicator()`, `silent_indicator()`
//! - `notification_item()`: Coordinates layout using the visual state

use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Color, Element, Fill};

use crate::github::types::{self, SubjectType};
use crate::settings::IconTheme;
use crate::ui::screens::notifications::helper::ProcessedNotification;
use crate::ui::screens::notifications::messages::{
    NotificationMessage, ThreadMessage, ViewMessage,
};
use crate::ui::screens::settings::rule_engine::RuleAction;
use crate::ui::{icons, theme};

// ============================================================================
// Visual State - Single Source of Truth
// ============================================================================

/// Centralized visual decisions for a notification.
///
/// This is the SINGLE SOURCE OF TRUTH for all visual properties:
/// - Subject type colors (consistent across all usages)
/// - Card styling (background, border, accent bar)
/// - State indicators (priority, silent, unread)
///
/// By computing everything in one place, we guarantee:
/// 1. Consistent colors between icon and accent bar
/// 2. No scattered color logic throughout the codebase
/// 3. Easy modification of visual rules in one location
pub struct NotificationVisualState {
    pub is_priority: bool,
    pub is_silent: bool,
    pub is_unread: bool,
    pub subject_color: Color,
    pub bar_color: Color,
    pub card_bg: Color,
    pub border_color: Color,
    pub show_border: bool,
}

impl NotificationVisualState {
    /// Compute complete visual state from notification data.
    ///
    /// This method encapsulates ALL visual logic, ensuring consistency
    /// between the subject icon, accent bar, card styling, and indicators.
    pub fn compute(
        is_unread: bool,
        subject_type: SubjectType,
        action: RuleAction,
        is_priority_group: bool,
    ) -> Self {
        let p = theme::palette();

        // Subject color is the foundation - used for icons, bars, and tints
        let subject_color = Self::color_for_subject_type(subject_type);

        // Priority styling only applies when in the priority group
        let is_priority = is_priority_group && action == RuleAction::Important;
        let is_silent = action == RuleAction::Silent;

        // Bar color: priority gets warning, unread gets subject color, read gets transparent
        let bar_color = if is_priority {
            p.accent_warning
        } else if is_unread {
            subject_color
        } else {
            Color::TRANSPARENT
        };

        // Card background: subtle tint for priority/unread
        let card_bg = if is_priority {
            Color::from_rgba(
                p.accent_warning.r,
                p.accent_warning.g,
                p.accent_warning.b,
                0.08,
            )
        } else if is_unread {
            Color::from_rgba(subject_color.r, subject_color.g, subject_color.b, 0.05)
        } else {
            Color::TRANSPARENT
        };

        // Border color: stronger tint for priority/unread
        let border_color = if is_priority {
            Color::from_rgba(
                p.accent_warning.r,
                p.accent_warning.g,
                p.accent_warning.b,
                0.2,
            )
        } else if is_unread {
            Color::from_rgba(subject_color.r, subject_color.g, subject_color.b, 0.12)
        } else {
            Color::TRANSPARENT
        };

        let show_border = is_priority || is_unread;

        Self {
            is_priority,
            is_silent,
            is_unread,
            subject_color,
            bar_color,
            card_bg,
            border_color,
            show_border,
        }
    }

    /// Get the canonical color for a subject type.
    ///
    /// This is the single definition of subject type colors, ensuring
    /// consistency between icons, accent bars, and any other usage.
    #[inline]
    pub fn color_for_subject_type(subject_type: SubjectType) -> Color {
        let p = theme::palette();
        match subject_type {
            SubjectType::Issue => p.accent_success,
            SubjectType::PullRequest => p.accent,
            SubjectType::Release => p.accent_purple,
            SubjectType::Discussion => p.accent,
            SubjectType::CheckSuite => p.accent_warning,
            SubjectType::RepositoryVulnerabilityAlert => p.accent_danger,
            SubjectType::Commit => p.text_secondary,
            SubjectType::Unknown => p.text_secondary,
        }
    }

    /// Get the icon for a subject type with the correct color.
    ///
    /// Uses `color_for_subject_type` internally to guarantee consistency.
    /// This is a static method for cases where you don't have a visual state instance.
    #[allow(dead_code)] // Public API for external use
    pub fn icon_for_subject_type(
        subject_type: SubjectType,
        icon_theme: IconTheme,
    ) -> Element<'static, NotificationMessage> {
        let color = Self::color_for_subject_type(subject_type);
        Self::build_icon(subject_type, color, icon_theme)
    }

    /// Get the icon for a subject type using the pre-computed subject_color.
    ///
    /// This uses the already-computed `subject_color` from the visual state,
    /// avoiding redundant color lookups and ensuring consistency.
    pub fn icon_for_subject_type_with_color(
        &self,
        subject_type: SubjectType,
        icon_theme: IconTheme,
    ) -> Element<'static, NotificationMessage> {
        Self::build_icon(subject_type, self.subject_color, icon_theme)
    }

    fn build_icon(
        subject_type: SubjectType,
        color: Color,
        icon_theme: IconTheme,
    ) -> Element<'static, NotificationMessage> {
        let icon_size = theme::notification_scaled(14.0);

        match subject_type {
            SubjectType::Issue => icons::icon_issue(icon_size, color, icon_theme),
            SubjectType::PullRequest => icons::icon_pull_request(icon_size, color, icon_theme),
            SubjectType::Release => icons::icon_release(icon_size, color, icon_theme),
            SubjectType::Discussion => icons::icon_discussion(icon_size, color, icon_theme),
            SubjectType::CheckSuite => icons::icon_check_suite(icon_size, color, icon_theme),
            SubjectType::Commit => icons::icon_commit(icon_size, color, icon_theme),
            SubjectType::RepositoryVulnerabilityAlert => {
                icons::icon_security(icon_size, color, icon_theme)
            }
            SubjectType::Unknown => icons::icon_unknown(icon_size, color, icon_theme),
        }
    }
}

// ============================================================================
// Reusable Widget Builders
// ============================================================================

fn account_badge(account: &str, size: f32) -> Element<'_, NotificationMessage> {
    let p = theme::palette();
    container(text(format!("@{}", account)).size(size).color(p.text_muted))
        .padding([2, 6])
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                p.text_muted.r,
                p.text_muted.g,
                p.text_muted.b,
                0.1,
            ))),
            border: iced::Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

fn priority_indicator(size: f32) -> Element<'static, NotificationMessage> {
    container(text("⚡").size(size)).padding([0, 4]).into()
}

fn silent_indicator(size: f32) -> Element<'static, NotificationMessage> {
    container(text("🔕").size(size)).padding([2, 4]).into()
}

// ============================================================================
// Main Widget
// ============================================================================

/// Uses `NotificationVisualState` as the single source of truth for all
/// visual decisions, ensuring consistency between icons, colors, and styling.
pub fn notification_item(
    processed: &ProcessedNotification,
    icon_theme: IconTheme,
    dense: bool,
    is_priority_group: bool,
    interactive: bool,
) -> Element<'_, NotificationMessage> {
    let notif = &processed.notification;
    let p = theme::palette();

    // Compute all visual decisions upfront (single source of truth)
    let visual = NotificationVisualState::compute(
        notif.unread,
        notif.subject_type,
        processed.action,
        is_priority_group,
    );

    // Get subject icon using the visual state's color (guaranteed consistency)
    let subject_icon = visual.icon_for_subject_type_with_color(notif.subject_type, icon_theme);

    // --- SIZING & SPACING ---
    let metrics = LayoutMetrics {
        title_size: theme::notification_scaled(if dense { 13.0 } else { 14.0 }),
        meta_size: theme::notification_scaled(12.0),
        reason_size: theme::notification_scaled(11.0),
        account_size: theme::notification_scaled(10.0),
        padding_x: if dense { 12.0 } else { 16.0 },
        padding_y: if dense { 8.0 } else { 14.0 },
        content_spacing: if dense { 2.0 } else { 6.0 },
        row_spacing: 8.0,
    };

    // --- BUILD CONTENT ---
    let content = if dense {
        build_dense_layout(notif, icon_theme, &visual, &metrics, &p)
    } else {
        build_standard_layout(notif, subject_icon, &visual, &metrics, &p)
    };

    let content_element: Element<'_, NotificationMessage> = if interactive {
        // Click behavior depends on mode:
        // - Dense (power mode): Select for details panel view
        // - Standard: Open in browser
        let click_message = if dense {
            NotificationMessage::View(ViewMessage::SelectNotification(notif.id.clone()))
        } else {
            NotificationMessage::Thread(ThreadMessage::Open(notif.id.clone()))
        };

        button(content)
            .style(theme::notification_button)
            .on_press(click_message)
            .width(Fill)
            .into()
    } else {
        // Just the content. We remove the container wrapper to avoid layout issues
        // when nested in other structures (like bulk selection button).
        // The content itself (Row) handles sizing via its children.
        content.into()
    };

    build_card(content_element, &visual, dense)
}

// ============================================================================
// Layout Builders
// ============================================================================

struct LayoutMetrics {
    title_size: f32,
    meta_size: f32,
    reason_size: f32,
    account_size: f32,
    padding_x: f32,
    padding_y: f32,
    content_spacing: f32,
    row_spacing: f32,
}

fn build_standard_layout<'a>(
    notif: &'a crate::github::types::NotificationView,
    subject_icon: Element<'static, NotificationMessage>,
    visual: &NotificationVisualState,
    metrics: &LayoutMetrics,
    p: &theme::ThemePalette,
) -> iced::widget::Row<'a, NotificationMessage> {
    let title_color = if visual.is_unread {
        p.text_primary
    } else {
        p.text_secondary
    };
    let title = text(&notif.title)
        .size(metrics.title_size)
        .color(title_color);

    let mut meta_row = row![
        subject_icon,
        Space::new().width(6),
        text(&notif.repo_full_name)
            .size(metrics.meta_size)
            .color(p.text_secondary),
        Space::new().width(8),
        text(notif.reason.label())
            .size(metrics.reason_size)
            .color(p.text_muted),
    ]
    .align_y(Alignment::Center);

    // Add account badge only for priority notifications (they can come from any account)
    if visual.is_priority && !notif.account.is_empty() {
        meta_row = meta_row.push(Space::new().width(8));
        meta_row = meta_row.push(account_badge(&notif.account, metrics.account_size));
    }

    if visual.is_silent {
        meta_row = meta_row.push(Space::new().width(4));
        meta_row = meta_row.push(silent_indicator(metrics.account_size));
    }

    let time_ago = types::format_time_ago(notif.updated_at);
    let time_row = build_time_row(visual, time_ago, metrics.meta_size, p);

    row![
        column![title, meta_row]
            .spacing(metrics.content_spacing)
            .width(Fill),
        container(time_row).padding([4, 8]),
    ]
    .spacing(metrics.row_spacing)
    .align_y(Alignment::Center)
    .padding([metrics.padding_y, metrics.padding_x])
    .width(Fill)
}

fn build_dense_layout<'a>(
    notif: &'a crate::github::types::NotificationView,
    icon_theme: IconTheme,
    visual: &NotificationVisualState,
    metrics: &LayoutMetrics,
    p: &theme::ThemePalette,
) -> iced::widget::Row<'a, NotificationMessage> {
    // Use visual state's pre-computed subject_color for the icon
    let subject_icon = visual.icon_for_subject_type_with_color(notif.subject_type, icon_theme);

    let title_color = if visual.is_unread {
        p.text_primary
    } else {
        p.text_secondary
    };

    let mut title_row = row![
        subject_icon,
        Space::new().width(6),
        text(&notif.title)
            .size(metrics.title_size)
            .color(title_color),
    ]
    .align_y(Alignment::Center);

    // Add account badge only for priority notifications (they can come from any account)
    if visual.is_priority && !notif.account.is_empty() {
        title_row = title_row.push(Space::new().width(8));
        title_row = title_row.push(account_badge(&notif.account, metrics.account_size));
    }

    let time_ago = types::format_time_ago(notif.updated_at);
    let time_row = build_time_row(visual, time_ago, metrics.meta_size, p);

    row![
        column![
            title_row,
            row![
                text(&notif.repo_full_name)
                    .size(metrics.meta_size)
                    .color(p.text_secondary),
                Space::new().width(8),
                text(notif.reason.label())
                    .size(metrics.reason_size)
                    .color(p.text_muted),
            ]
            .align_y(Alignment::Center)
            .padding([0, 20]) // Indent meta slightly
        ]
        .spacing(2)
        .width(Fill),
        container(time_row).padding([0, 8]),
    ]
    .align_y(Alignment::Center)
    .padding([metrics.padding_y, metrics.padding_x])
    .width(Fill)
}

fn build_time_row<'a>(
    visual: &NotificationVisualState,
    time_ago: String,
    meta_size: f32,
    p: &theme::ThemePalette,
) -> iced::widget::Row<'a, NotificationMessage> {
    let mut time_row = row![].align_y(Alignment::Center);
    if visual.is_priority {
        time_row = time_row.push(priority_indicator(meta_size));
    }
    time_row = time_row.push(text(time_ago).size(meta_size).color(p.text_muted));
    time_row
}

fn build_card<'a>(
    content_element: Element<'a, NotificationMessage>,
    visual: &NotificationVisualState,
    dense: bool,
) -> Element<'a, NotificationMessage> {
    let bar_color = visual.bar_color;
    let card_bg = visual.card_bg;
    let border_color = visual.border_color;
    let show_border = visual.show_border;

    // Use a fixed-size accent bar instead of Fill to avoid layout collapse
    // when nested in rows without explicit height
    let accent_bar = container(Space::new().width(3)).style(move |_| container::Style {
        background: Some(iced::Background::Color(bar_color)),
        ..Default::default()
    });

    container(
        row![accent_bar, content_element]
            .spacing(0)
            .align_y(Alignment::Center)
            .width(Fill),
    )
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(card_bg)),
        border: iced::Border {
            radius: if dense { 0.0.into() } else { 6.0.into() },
            color: border_color,
            width: if show_border { 1.0 } else { 0.0 },
        },
        ..Default::default()
    })
    .width(Fill)
    .into()
}
