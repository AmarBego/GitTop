//! Centralized icon helpers with theme support.
//!
//! Supports both SVG icons (Lucide) and emoji icons based on user preference.

use iced::widget::{svg, text, Svg, Text};
use iced::{Color, Element};
use icondata_core::IconData;

use crate::settings::IconTheme;

/// Convert IconData to SVG bytes by building the XML string.
fn icon_to_svg_bytes(data: &IconData) -> Vec<u8> {
    let mut svg_str = String::from(r#"<svg xmlns="http://www.w3.org/2000/svg""#);

    if let Some(w) = data.width {
        svg_str.push_str(&format!(r#" width="{}""#, w));
    }
    if let Some(h) = data.height {
        svg_str.push_str(&format!(r#" height="{}""#, h));
    }
    if let Some(vb) = data.view_box {
        svg_str.push_str(&format!(r#" viewBox="{}""#, vb));
    }
    if let Some(fill) = data.fill {
        svg_str.push_str(&format!(r#" fill="{}""#, fill));
    }
    if let Some(stroke) = data.stroke {
        svg_str.push_str(&format!(r#" stroke="{}""#, stroke));
    }
    if let Some(stroke_width) = data.stroke_width {
        svg_str.push_str(&format!(r#" stroke-width="{}""#, stroke_width));
    }
    if let Some(stroke_linecap) = data.stroke_linecap {
        svg_str.push_str(&format!(r#" stroke-linecap="{}""#, stroke_linecap));
    }
    if let Some(stroke_linejoin) = data.stroke_linejoin {
        svg_str.push_str(&format!(r#" stroke-linejoin="{}""#, stroke_linejoin));
    }
    if let Some(style) = data.style {
        svg_str.push_str(&format!(r#" style="{}""#, style));
    }

    svg_str.push('>');
    svg_str.push_str(data.data);
    svg_str.push_str("</svg>");

    svg_str.into_bytes()
}

/// Create a colored SVG icon.
pub fn icon_colored(data: &IconData, size: f32, color: Color) -> Svg<'static> {
    let bytes = icon_to_svg_bytes(data);
    svg(svg::Handle::from_memory(bytes))
        .width(size)
        .height(size)
        .style(move |_, _| svg::Style { color: Some(color) })
}

/// Create an emoji text icon.
fn emoji_icon(emoji: &'static str, size: u32, color: Color) -> Text<'static> {
    text(emoji).size(size).color(color)
}

/// Helper to switch between SVG and Emoji based on theme
fn themed_icon<M: 'static>(
    theme: IconTheme,
    svg_data: &'static IconData,
    emoji: &'static str,
    size: f32,
    color: Color,
) -> Element<'static, M> {
    match theme {
        IconTheme::Svg => icon_colored(svg_data, size, color).into(),
        IconTheme::Emoji => emoji_icon(emoji, size as u32, color).into(),
    }
}

// =============================================================================
// THEME-AWARE ICON FUNCTIONS
// These return Element<M> so they can be either SVG or Text based on theme.
// =============================================================================

/// App branding icon (diamond).
pub fn icon_brand<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuDiamond, "◆", size, color)
}

/// User/profile icon.
pub fn icon_user<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuUser, "👤", size, color)
}

/// Power/logout icon.
pub fn icon_power<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuPower, "⏻", size, color)
}

/// Refresh icon.
pub fn icon_refresh<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuRefreshCw, "↻", size, color)
}

/// Check/success icon.
pub fn icon_check<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuCheck, "✓", size, color)
}

/// Alert/warning icon.
pub fn icon_alert<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuTriangleAlert, "⚠", size, color)
}

/// Inbox/all icon.
pub fn icon_inbox<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuInbox, "📋", size, color)
}

/// Folder/repository icon.
pub fn icon_folder<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuFolder, "📁", size, color)
}

/// Issue icon (circle dot).
pub fn icon_issue<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuCircleDot, "●", size, color)
}

/// Pull request icon.
pub fn icon_pull_request<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuGitPullRequest, "⇄", size, color)
}

/// Release/tag icon.
pub fn icon_release<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuTag, "◆", size, color)
}

/// Discussion icon.
pub fn icon_discussion<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuMessageCircle, "💬", size, color)
}

/// CI/workflow check icon.
pub fn icon_check_suite<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuCircleCheck, "✓", size, color)
}

/// Commit icon.
pub fn icon_commit<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuGitCommitHorizontal, "◉", size, color)
}

/// Security/vulnerability icon.
pub fn icon_security<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuShieldAlert, "⚠", size, color)
}

/// Unknown/generic icon.
pub fn icon_unknown<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuCircle, "○", size, color)
}

/// Circle check/success icon with fill.
pub fn icon_circle_check<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuCircleCheck, "✓", size, color)
}

/// Settings gear icon.
pub fn icon_settings<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuSettings, "⚙", size, color)
}

/// Chevron down icon.
pub fn icon_chevron_down<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuChevronDown, "▼", size, color)
}

/// Chevron right icon.
pub fn icon_chevron_right<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuChevronRight, "▶", size, color)
}

/// Chevron left icon.
pub fn icon_chevron_left<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuChevronLeft, "◀", size, color)
}

/// Trash icon.
pub fn icon_trash<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuTrash2, "🗑", size, color)
}

/// Palette/appearance icon.
pub fn icon_palette<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuPalette, "🎨", size, color)
}

/// Bell/notification icon.
pub fn icon_notification<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuBell, "🔔", size, color)
}

/// Filter icon.
pub fn icon_filter<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuSlidersHorizontal, "⚙", size, color)
}

/// External link icon.
pub fn icon_external_link<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuExternalLink, "↗", size, color)
}

/// Clock icon.
pub fn icon_clock<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuClock, "🕐", size, color)
}

/// Calendar icon.
pub fn icon_calendar<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuCalendar, "📅", size, color)
}

/// Building icon.
pub fn icon_building<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuBuilding, "🏢", size, color)
}

/// Tag icon.
pub fn icon_tag<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuTag, "🏷", size, color)
}

/// Chart/dashboard icon.
pub fn icon_chart<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuLayoutDashboard, "📊", size, color)
}

/// Inbox empty icon.
pub fn icon_inbox_empty<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuArchive, "📭", size, color)
}

/// Plus/add icon.
pub fn icon_plus<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuPlus, "+", size, color)
}

/// X/close icon.
pub fn icon_x<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuX, "✕", size, color)
}

/// Zap/lightning icon.
pub fn icon_zap<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuZap, "⚡", size, color)
}

/// Eye off/hidden icon.
pub fn icon_eye_off<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuEyeOff, "👁‍🗨", size, color)
}

/// Briefcase icon.
pub fn icon_briefcase<M: 'static>(
    size: f32,
    color: Color,
    theme: IconTheme,
) -> Element<'static, M> {
    themed_icon(theme, &icondata_lu::LuBriefcase, "💼", size, color)
}
