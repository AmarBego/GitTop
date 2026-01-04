use std::fmt::Write;

use iced::widget::{Svg, Text, svg, text};
use iced::{Color, Element};
use icondata_core::IconData;

use crate::settings::IconTheme;

fn icon_to_svg_bytes(data: &IconData) -> Vec<u8> {
    let mut svg_str = String::from(r#"<svg xmlns="http://www.w3.org/2000/svg""#);

    let mut attr = |name: &str, value: &dyn std::fmt::Display| {
        let _ = write!(svg_str, r#" {}="{}""#, name, value);
    };

    if let Some(v) = data.width {
        attr("width", &v);
    }
    if let Some(v) = data.height {
        attr("height", &v);
    }
    if let Some(v) = data.view_box {
        attr("viewBox", &v);
    }
    if let Some(v) = data.fill {
        attr("fill", &v);
    }
    if let Some(v) = data.stroke {
        attr("stroke", &v);
    }
    if let Some(v) = data.stroke_width {
        attr("stroke-width", &v);
    }
    if let Some(v) = data.stroke_linecap {
        attr("stroke-linecap", &v);
    }
    if let Some(v) = data.stroke_linejoin {
        attr("stroke-linejoin", &v);
    }
    if let Some(v) = data.style {
        attr("style", &v);
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

fn emoji_icon(emoji: &'static str, size: f32, color: Color) -> Text<'static> {
    text(emoji).size(size.round() as u32).color(color)
}

fn themed_icon<M: 'static>(
    theme: IconTheme,
    svg_data: &'static IconData,
    emoji: &'static str,
    size: f32,
    color: Color,
) -> Element<'static, M> {
    match theme {
        IconTheme::Svg => icon_colored(svg_data, size, color).into(),
        IconTheme::Emoji => emoji_icon(emoji, size, color).into(),
    }
}

// =============================================================================
// THEME-AWARE ICON FUNCTIONS
// These return Element<M> so they can be either SVG or Text based on theme.
// =============================================================================

macro_rules! impl_icons {
    ($(fn $name:ident($icon:path, $emoji:literal);)+) => {
        $(
            pub fn $name<M: 'static>(size: f32, color: Color, theme: IconTheme) -> Element<'static, M> {
                themed_icon(theme, $icon, $emoji, size, color)
            }
        )+
    }
}

impl_icons! {
    fn icon_user(icondata_lu::LuUser, "👤");
    fn icon_power(icondata_lu::LuPower, "⏻");
    fn icon_settings(icondata_lu::LuSettings, "⚙");
    fn icon_refresh(icondata_lu::LuRefreshCw, "↻");
    fn icon_external_link(icondata_lu::LuExternalLink, "↗");
    fn icon_plus(icondata_lu::LuPlus, "+");
    fn icon_x(icondata_lu::LuX, "×");
    fn icon_check(icondata_lu::LuCheck, "✓");
    fn icon_circle_check(icondata_lu::LuCircleCheck, "✔");
    fn icon_check_suite(icondata_lu::LuCircleCheck, "☑");
    fn icon_alert(icondata_lu::LuTriangleAlert, "⚠");
    fn icon_security(icondata_lu::LuShieldAlert, "🔒");
    fn icon_inbox(icondata_lu::LuInbox, "📥");
    fn icon_inbox_empty(icondata_lu::LuArchive, "📭");
    fn icon_folder(icondata_lu::LuFolder, "📁");
    fn icon_building(icondata_lu::LuBuilding, "🏢");
    fn icon_issue(icondata_lu::LuCircleDot, "●");
    fn icon_pull_request(icondata_lu::LuGitPullRequest, "⇄");
    fn icon_commit(icondata_lu::LuGitCommitHorizontal, "◉");
    fn icon_release(icondata_lu::LuTag, "🚀");
    fn icon_tag(icondata_lu::LuTag, "🏷");
    fn icon_discussion(icondata_lu::LuMessageCircle, "💬");
    fn icon_at(icondata_lu::LuAtSign, "@");
    fn icon_info(icondata_lu::LuInfo, "ℹ");
    fn icon_eye_off(icondata_lu::LuEyeOff, "🙈");
    fn icon_chevron_down(icondata_lu::LuChevronDown, "v");
    fn icon_chevron_right(icondata_lu::LuChevronRight, ">");
    fn icon_chevron_left(icondata_lu::LuChevronLeft, "<");
    fn icon_trash(icondata_lu::LuTrash2, "🗑");
    fn icon_filter(icondata_lu::LuSlidersHorizontal, "⌕");
    fn icon_chart(icondata_lu::LuLayoutDashboard, "📊");
    fn icon_zap(icondata_lu::LuZap, "⚡");
    fn icon_wifi(icondata_lu::LuWifi, "📶");
    fn icon_unknown(icondata_lu::LuCircle, "?");
}
