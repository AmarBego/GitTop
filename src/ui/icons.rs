//! Centralized SVG icon helpers using Lucide icons.
//!
//! Uses icondata_lu for CPU-efficient SVG rendering with tiny-skia.

use iced::widget::{svg, Svg};
use iced::Color;
use icondata_core::IconData;

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

/// Create an SVG icon from IconData with the specified size.
pub fn icon(data: &IconData, size: f32) -> Svg<'static> {
    let bytes = icon_to_svg_bytes(data);
    svg(svg::Handle::from_memory(bytes))
        .width(size)
        .height(size)
}

/// Create a colored SVG icon.
pub fn icon_colored(data: &IconData, size: f32, color: Color) -> Svg<'static> {
    let bytes = icon_to_svg_bytes(data);
    svg(svg::Handle::from_memory(bytes))
        .width(size)
        .height(size)
        .style(move |_, _| svg::Style { color: Some(color) })
}

// =============================================================================
// APP ICONS
// =============================================================================

/// App branding icon (diamond).
pub fn icon_brand(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuDiamond, size, color)
}

/// User/profile icon.
pub fn icon_user(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuUser, size, color)
}

/// Power/logout icon.
pub fn icon_power(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuPower, size, color)
}

/// Refresh icon.
pub fn icon_refresh(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuRefreshCw, size, color)
}

/// Check/success icon.
pub fn icon_check(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuCheck, size, color)
}

/// Alert/warning icon.
pub fn icon_alert(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuTriangleAlert, size, color)
}

// =============================================================================
// SIDEBAR ICONS
// =============================================================================

/// Inbox/all icon.
pub fn icon_inbox(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuInbox, size, color)
}

/// Folder/repository icon.
pub fn icon_folder(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuFolder, size, color)
}

// =============================================================================
// SUBJECT TYPE ICONS
// =============================================================================

/// Issue icon (circle dot).
pub fn icon_issue(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuCircleDot, size, color)
}

/// Pull request icon.
pub fn icon_pull_request(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuGitPullRequest, size, color)
}

/// Release/tag icon.
pub fn icon_release(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuTag, size, color)
}

/// Discussion icon.
pub fn icon_discussion(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuMessageCircle, size, color)
}

/// CI/workflow check icon.
pub fn icon_check_suite(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuCircleCheck, size, color)
}

/// Commit icon.
pub fn icon_commit(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuGitCommitHorizontal, size, color)
}

/// Security/vulnerability icon.
pub fn icon_security(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuShieldAlert, size, color)
}

/// Unknown/generic icon.
pub fn icon_unknown(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuCircle, size, color)
}

/// Circle check/success icon with fill.
pub fn icon_circle_check(size: f32, color: Color) -> Svg<'static> {
    icon_colored(&icondata_lu::LuCircleCheck, size, color)
}
