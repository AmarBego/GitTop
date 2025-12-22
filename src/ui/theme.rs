//! Theme system for GitTop - platform-aware color palettes with good contrast.
//!
//! Design principles:
//! - Strong contrast for readability (no grey-on-grey)
//! - Platform-aware defaults
//! - Clean, professional aesthetic with subtle depth

use iced::widget::{button, container, pick_list, scrollable, text, text_input};
use iced::{Background, Border, Color, Theme};
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};

use crate::settings::AppTheme;

// ============================================================================
// HELPERS - boilerplate reduction
// ============================================================================

/// Access palette with less boilerplate
#[inline]
fn with_palette<T>(f: impl FnOnce(ThemePalette) -> T) -> T {
    f(palette())
}

/// Standard button status color helper
#[inline]
fn hover_active(status: button::Status, normal: Color, hover: Color, active: Color) -> Color {
    match status {
        button::Status::Hovered => hover,
        button::Status::Pressed => active,
        _ => normal,
    }
}

/// Standard card border helper
#[inline]
fn card_border(radius: f32) -> Border {
    Border {
        color: Color::TRANSPARENT,
        width: 0.0,
        radius: radius.into(),
    }
}

// ============================================================================
// THEME PALETTE - Dynamic colors based on selected theme
// ============================================================================

/// Complete color palette for a theme
#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    /// Main background
    pub bg_base: Color,
    /// Card/panel background
    pub bg_card: Color,
    /// Control/input background
    pub bg_control: Color,
    /// Hover state background
    pub bg_hover: Color,
    /// Active/pressed state
    pub bg_active: Color,
    /// Sidebar background
    pub bg_sidebar: Color,

    /// Primary text (high contrast)
    pub text_primary: Color,
    /// Secondary text (medium contrast)
    pub text_secondary: Color,
    /// Muted text (still readable!)
    pub text_muted: Color,

    /// Primary accent color
    pub accent: Color,
    /// Success/green accent
    pub accent_success: Color,
    /// Warning/orange accent
    pub accent_warning: Color,
    /// Danger/red accent
    pub accent_danger: Color,
    /// Purple accent
    pub accent_purple: Color,

    /// Border color
    pub border: Color,
    /// Subtle border/divider
    pub border_subtle: Color,
}

// Static palette definitions
pub const LIGHT: ThemePalette = ThemePalette {
    // Backgrounds - white/light grey
    bg_base: Color::from_rgb(0.98, 0.98, 0.98), // #fafafa
    bg_card: Color::WHITE,                      // #ffffff
    bg_control: Color::from_rgb(0.94, 0.94, 0.94), // #f0f0f0
    bg_hover: Color::from_rgb(0.90, 0.92, 0.95), // #e6ebf2 light blue tint
    bg_active: Color::from_rgb(0.85, 0.88, 0.92), // #d9e0eb
    bg_sidebar: Color::from_rgb(0.96, 0.96, 0.97), // #f5f5f7

    // Text - dark for contrast
    text_primary: Color::from_rgb(0.10, 0.10, 0.12), // #1a1a1e nearly black
    text_secondary: Color::from_rgb(0.35, 0.35, 0.40), // #595966
    text_muted: Color::from_rgb(0.55, 0.55, 0.60),   // #8c8c99

    // Accents - vibrant blue
    accent: Color::from_rgb(0.10, 0.46, 0.82), // #1a75d1
    accent_success: Color::from_rgb(0.15, 0.65, 0.30), // #26a64d
    accent_warning: Color::from_rgb(0.90, 0.60, 0.05), // #e6990d
    accent_danger: Color::from_rgb(0.85, 0.20, 0.20), // #d93333
    accent_purple: Color::from_rgb(0.55, 0.35, 0.75), // #8c59bf

    // Borders - light grey
    border: Color::from_rgb(0.82, 0.82, 0.85), // #d1d1d9
    border_subtle: Color::from_rgb(0.90, 0.90, 0.92), // #e6e6eb
};

pub const STEAM: ThemePalette = ThemePalette {
    // Backgrounds - rich blue-grey, not too dark
    bg_base: Color::from_rgb(0.12, 0.16, 0.22), // #1e2936
    bg_card: Color::from_rgb(0.15, 0.20, 0.27), // #263444
    bg_control: Color::from_rgb(0.18, 0.24, 0.32), // #2e3d52
    bg_hover: Color::from_rgb(0.22, 0.30, 0.40), // #384d66
    bg_active: Color::from_rgb(0.25, 0.35, 0.45), // #405973
    bg_sidebar: Color::from_rgb(0.10, 0.13, 0.18), // #1a212e

    // Text - high contrast, clearly readable
    text_primary: Color::from_rgb(0.95, 0.96, 0.98), // #f2f5fa almost white
    text_secondary: Color::from_rgb(0.75, 0.80, 0.85), // #bfccd9
    text_muted: Color::from_rgb(0.55, 0.62, 0.70),   // #8c9eb3 still visible!

    // Accents - vibrant and visible
    accent: Color::from_rgb(0.40, 0.75, 0.95), // #66bff2 steam blue
    accent_success: Color::from_rgb(0.40, 0.80, 0.45), // #66cc73
    accent_warning: Color::from_rgb(0.95, 0.65, 0.25), // #f2a640
    accent_danger: Color::from_rgb(0.90, 0.35, 0.35), // #e65959
    accent_purple: Color::from_rgb(0.70, 0.55, 0.90), // #b38ce6

    // Borders
    border: Color::from_rgb(0.30, 0.38, 0.48), // #4d6179
    border_subtle: Color::from_rgb(0.22, 0.28, 0.36), // #38475c
};

pub const GTK_DARK: ThemePalette = ThemePalette {
    // Backgrounds - Adwaita dark grey
    bg_base: Color::from_rgb(0.14, 0.14, 0.14), // #242424
    bg_card: Color::from_rgb(0.19, 0.19, 0.19), // #303030
    bg_control: Color::from_rgb(0.24, 0.24, 0.24), // #3d3d3d
    bg_hover: Color::from_rgb(0.30, 0.30, 0.30), // #4d4d4d
    bg_active: Color::from_rgb(0.35, 0.35, 0.35), // #595959
    bg_sidebar: Color::from_rgb(0.12, 0.12, 0.12), // #1e1e1e

    // Text - Adwaita uses warm whites
    text_primary: Color::from_rgb(0.96, 0.94, 0.92), // #f5f0eb
    text_secondary: Color::from_rgb(0.78, 0.76, 0.74), // #c7c2bc
    text_muted: Color::from_rgb(0.58, 0.56, 0.54),   // #948f8a

    // Accents - Adwaita blue
    accent: Color::from_rgb(0.21, 0.52, 0.89), // #3584e4
    accent_success: Color::from_rgb(0.30, 0.76, 0.35), // #4dc259
    accent_warning: Color::from_rgb(0.96, 0.76, 0.07), // #f5c211
    accent_danger: Color::from_rgb(0.90, 0.29, 0.24), // #e64a3d
    accent_purple: Color::from_rgb(0.61, 0.35, 0.71), // #9c59b5

    // Borders
    border: Color::from_rgb(0.35, 0.35, 0.35),
    border_subtle: Color::from_rgb(0.25, 0.25, 0.25),
};

pub const WINDOWS11: ThemePalette = ThemePalette {
    // Backgrounds - Mica dark
    bg_base: Color::from_rgb(0.12, 0.12, 0.12), // #202020
    bg_card: Color::from_rgb(0.17, 0.17, 0.17), // #2b2b2b
    bg_control: Color::from_rgb(0.22, 0.22, 0.22), // #383838
    bg_hover: Color::from_rgb(0.28, 0.28, 0.28), // #474747
    bg_active: Color::from_rgb(0.33, 0.33, 0.33), // #545454
    bg_sidebar: Color::from_rgb(0.10, 0.10, 0.10), // #1a1a1a

    // Text - pure white hierarchy
    text_primary: Color::WHITE,
    text_secondary: Color::from_rgb(0.82, 0.82, 0.82), // #d1d1d1
    text_muted: Color::from_rgb(0.60, 0.60, 0.60),     // #999

    // Accents - Windows blue
    accent: Color::from_rgb(0.38, 0.80, 1.0), // #60cdff
    accent_success: Color::from_rgb(0.42, 0.80, 0.37), // #6bcc5e
    accent_warning: Color::from_rgb(0.99, 0.72, 0.11), // #fcb81c
    accent_danger: Color::from_rgb(0.95, 0.32, 0.32), // #f25252
    accent_purple: Color::from_rgb(0.78, 0.65, 0.95), // #c7a6f2

    // Borders
    border: Color::from_rgb(0.35, 0.35, 0.35),
    border_subtle: Color::from_rgb(0.25, 0.25, 0.25),
};

pub const MACOS: ThemePalette = ThemePalette {
    // Backgrounds - macOS dark grey
    bg_base: Color::from_rgb(0.11, 0.11, 0.12), // #1c1c1e
    bg_card: Color::from_rgb(0.17, 0.17, 0.18), // #2c2c2e
    bg_control: Color::from_rgb(0.22, 0.22, 0.24), // #38383c
    bg_hover: Color::from_rgb(0.28, 0.28, 0.30), // #48484c
    bg_active: Color::from_rgb(0.34, 0.34, 0.36), // #57575c
    bg_sidebar: Color::from_rgb(0.09, 0.09, 0.10), // #17171a

    // Text
    text_primary: Color::WHITE,
    text_secondary: Color::from_rgb(0.78, 0.78, 0.80), // #c7c7cc
    text_muted: Color::from_rgb(0.55, 0.55, 0.58),     // #8c8c94

    // Accents - macOS system blue
    accent: Color::from_rgb(0.04, 0.52, 1.0), // #0a84ff
    accent_success: Color::from_rgb(0.20, 0.78, 0.35), // #32c759
    accent_warning: Color::from_rgb(1.0, 0.62, 0.04), // #ff9f0a
    accent_danger: Color::from_rgb(1.0, 0.27, 0.23), // #ff453a
    accent_purple: Color::from_rgb(0.75, 0.35, 0.95), // #bf5af2

    // Borders
    border: Color::from_rgb(0.30, 0.30, 0.32),
    border_subtle: Color::from_rgb(0.22, 0.22, 0.24),
};

pub const HIGH_CONTRAST: ThemePalette = ThemePalette {
    // Backgrounds - true black
    bg_base: Color::BLACK,
    bg_card: Color::from_rgb(0.08, 0.08, 0.08),
    bg_control: Color::from_rgb(0.15, 0.15, 0.15),
    bg_hover: Color::from_rgb(0.25, 0.25, 0.25),
    bg_active: Color::from_rgb(0.35, 0.35, 0.35),
    bg_sidebar: Color::BLACK,

    // Text - pure white
    text_primary: Color::WHITE,
    text_secondary: Color::from_rgb(0.90, 0.90, 0.90),
    text_muted: Color::from_rgb(0.75, 0.75, 0.75),

    // Accents - bright, saturated
    accent: Color::from_rgb(0.0, 0.80, 1.0), // pure cyan
    accent_success: Color::from_rgb(0.0, 1.0, 0.40), // bright green
    accent_warning: Color::from_rgb(1.0, 0.85, 0.0), // yellow
    accent_danger: Color::from_rgb(1.0, 0.20, 0.20), // red
    accent_purple: Color::from_rgb(0.85, 0.45, 1.0), // bright purple

    // Borders - visible
    border: Color::from_rgb(0.50, 0.50, 0.50),
    border_subtle: Color::from_rgb(0.35, 0.35, 0.35),
};

impl ThemePalette {
    /// Get palette for the specified theme
    pub fn for_theme(theme: AppTheme) -> Self {
        match theme {
            AppTheme::Light => LIGHT,
            AppTheme::Steam => STEAM,
            AppTheme::GtkDark => GTK_DARK,
            AppTheme::Windows11 => WINDOWS11,
            AppTheme::MacOS => MACOS,
            AppTheme::HighContrast => HIGH_CONTRAST,
        }
    }
}

// ============================================================================
// GLOBAL THEME STATE - Thread-safe runtime theme switching
// ============================================================================

mod theme_state {
    use super::*;

    /// Global theme storage (thread-safe)
    static CURRENT_THEME: AtomicU8 = AtomicU8::new(0);

    /// Global font scale for notifications
    static NOTIFICATION_FONT_SCALE: AtomicU32 = AtomicU32::new(1065353216); // 1.0f32 as u32 bits

    /// Global font scale for sidebar
    static SIDEBAR_FONT_SCALE: AtomicU32 = AtomicU32::new(1065353216); // 1.0f32 as u32 bits

    pub fn set_theme(theme: AppTheme) {
        CURRENT_THEME.store(theme.to_u8(), Ordering::Relaxed);
    }

    pub fn current_theme() -> AppTheme {
        AppTheme::from_u8(CURRENT_THEME.load(Ordering::Relaxed))
    }

    pub fn set_notification_font_scale(scale: f32) {
        NOTIFICATION_FONT_SCALE.store(scale.to_bits(), Ordering::Relaxed);
    }

    pub fn notification_font_scale() -> f32 {
        f32::from_bits(NOTIFICATION_FONT_SCALE.load(Ordering::Relaxed))
    }

    pub fn set_sidebar_font_scale(scale: f32) {
        SIDEBAR_FONT_SCALE.store(scale.to_bits(), Ordering::Relaxed);
    }

    pub fn sidebar_font_scale() -> f32 {
        f32::from_bits(SIDEBAR_FONT_SCALE.load(Ordering::Relaxed))
    }
}

/// Set the current theme (call this when user changes theme in settings)
pub fn set_theme(theme: AppTheme) {
    theme_state::set_theme(theme);
}

/// Get the current theme
pub fn current_theme() -> AppTheme {
    theme_state::current_theme()
}

/// Set the notification font scale (0.8 - 1.5)
pub fn set_notification_font_scale(scale: f32) {
    theme_state::set_notification_font_scale(scale);
}

/// Get the notification font scale
pub fn notification_font_scale() -> f32 {
    theme_state::notification_font_scale()
}

/// Get a scaled font size for notifications
#[inline]
pub fn notification_scaled(base_size: f32) -> f32 {
    base_size * notification_font_scale()
}

/// Set the sidebar font scale (0.8 - 1.5)
pub fn set_sidebar_font_scale(scale: f32) {
    theme_state::set_sidebar_font_scale(scale);
}

/// Get the sidebar font scale
pub fn sidebar_font_scale() -> f32 {
    theme_state::sidebar_font_scale()
}

/// Get a scaled font size for sidebar
#[inline]
pub fn sidebar_scaled(base_size: f32) -> f32 {
    base_size * sidebar_font_scale()
}

/// Get the current theme palette
#[inline]
pub fn palette() -> ThemePalette {
    ThemePalette::for_theme(current_theme())
}

// ============================================================================
// CONTAINER STYLES
// ============================================================================

pub fn app_container(_: &Theme) -> container::Style {
    with_palette(|p| container::Style {
        background: Some(Background::Color(p.bg_base)),
        ..Default::default()
    })
}

pub fn header(_: &Theme) -> container::Style {
    with_palette(|p| container::Style {
        background: Some(Background::Color(p.bg_card)),
        border: Border {
            color: p.border_subtle,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
}

pub fn sidebar(_: &Theme) -> container::Style {
    with_palette(|p| container::Style {
        background: Some(Background::Color(p.bg_sidebar)),
        border: Border {
            color: p.border_subtle,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
}

// ============================================================================
// BUTTON STYLES
// ============================================================================

pub fn primary_button(_: &Theme, status: button::Status) -> button::Style {
    with_palette(|p| {
        let bg = match status {
            button::Status::Hovered => {
                Color::from_rgba(p.accent.r * 0.9, p.accent.g * 0.9, p.accent.b * 0.9, 1.0)
            }
            button::Status::Pressed => {
                Color::from_rgba(p.accent.r * 0.8, p.accent.g * 0.8, p.accent.b * 0.8, 1.0)
            }
            _ => p.accent,
        };
        button::Style {
            background: Some(Background::Color(bg)),
            text_color: Color::BLACK,
            border: card_border(6.0),
            ..Default::default()
        }
    })
}

pub fn ghost_button(_: &Theme, status: button::Status) -> button::Style {
    with_palette(|p| button::Style {
        background: Some(Background::Color(hover_active(
            status,
            Color::TRANSPARENT,
            p.bg_hover,
            p.bg_active,
        ))),
        text_color: match status {
            button::Status::Pressed => p.text_secondary,
            _ => p.text_primary,
        },
        border: card_border(6.0),
        ..Default::default()
    })
}

/// Button style for priority group headers - uses warning/orange accent.
pub fn priority_header_button(_: &Theme, status: button::Status) -> button::Style {
    with_palette(|p| {
        let base_bg = Color::from_rgba(p.accent_warning.r, p.accent_warning.g, p.accent_warning.b, 0.05);
        let hover_bg = Color::from_rgba(p.accent_warning.r, p.accent_warning.g, p.accent_warning.b, 0.1);
        let active_bg = Color::from_rgba(p.accent_warning.r, p.accent_warning.g, p.accent_warning.b, 0.15);
        
        button::Style {
            background: Some(Background::Color(hover_active(
                status,
                base_bg,
                hover_bg,
                active_bg,
            ))),
            text_color: match status {
                button::Status::Pressed => p.accent_warning,
                _ => p.text_primary,
            },
            border: card_border(6.0),
            ..Default::default()
        }
    })
}

pub fn notification_button(_: &Theme, status: button::Status) -> button::Style {
    with_palette(|p| button::Style {
        background: Some(Background::Color(hover_active(
            status,
            Color::TRANSPARENT,
            p.bg_hover,
            p.bg_active,
        ))),
        text_color: p.text_primary,
        border: card_border(6.0),
        ..Default::default()
    })
}

// Sidebar filter button - shows selection state
pub fn sidebar_button(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        with_palette(|p| {
            let bg = if selected {
                p.bg_active
            } else {
                hover_active(status, Color::TRANSPARENT, p.bg_hover, p.bg_active)
            };
            let text = if selected { p.accent } else { p.text_primary };

            button::Style {
                background: Some(Background::Color(bg)),
                text_color: text,
                border: card_border(4.0),
                ..Default::default()
            }
        })
    }
}

/// Segment control button style (used for toggle groups like Unread/All, Dense/Normal)
/// - `is_selected`: whether this segment is currently active
pub fn segment_button(is_selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        with_palette(|p| {
            let base_bg = if is_selected {
                p.bg_active
            } else {
                Color::TRANSPARENT
            };
            let bg = match status {
                button::Status::Hovered if !is_selected => p.bg_hover,
                button::Status::Pressed => p.bg_active,
                _ => base_bg,
            };

            button::Style {
                background: Some(Background::Color(bg)),
                text_color: if is_selected {
                    p.text_primary
                } else {
                    p.text_secondary
                },
                border: Border {
                    radius: 0.0.into(), // No radius on individual buttons; container handles it
                    ..Default::default()
                },
                ..Default::default()
            }
        })
    }
}

/// Container style for segment control (wraps the segment buttons)
pub fn segment_container(_: &Theme) -> container::Style {
    with_palette(|p| container::Style {
        background: Some(Background::Color(p.bg_control)),
        border: Border {
            radius: 4.0.into(),
            color: p.border_subtle,
            width: 1.0,
        },
        ..Default::default()
    })
}

// ============================================================================
// TEXT STYLES
// ============================================================================

pub fn muted_text(_: &Theme) -> text::Style {
    text::Style {
        color: Some(palette().text_muted),
    }
}

pub fn secondary_text(_: &Theme) -> text::Style {
    text::Style {
        color: Some(palette().text_secondary),
    }
}

// ============================================================================
// TEXT INPUT STYLE
// ============================================================================

pub fn text_input_style(_: &Theme, status: text_input::Status) -> text_input::Style {
    with_palette(|p| {
        let (bg, border_color, border_width) = match status {
            text_input::Status::Focused { .. } => (p.bg_base, p.accent, 2.0),
            text_input::Status::Hovered => (p.bg_hover, p.border, 1.0),
            _ => (p.bg_control, p.border, 1.0),
        };
        text_input::Style {
            background: Background::Color(bg),
            border: Border {
                color: border_color,
                width: border_width,
                radius: 6.0.into(),
            },
            icon: p.text_muted,
            placeholder: p.text_muted,
            value: p.text_primary,
            selection: p.accent,
        }
    })
}

// ============================================================================
// SCROLLBAR STYLE
// ============================================================================

pub fn scrollbar(_: &Theme, _status: scrollable::Status) -> scrollable::Style {
    with_palette(|p| {
        let scroller = scrollable::Scroller {
            background: Background::Color(p.border),
            border: Border {
                width: 0.0,
                color: Color::TRANSPARENT,
                radius: 4.0.into(),
            },
        };

        let rail = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller,
        };

        scrollable::Style {
            vertical_rail: rail,
            horizontal_rail: rail,
            container: container::Style::default(),
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                shadow: iced::Shadow::default(),
                icon: Color::BLACK,
            },
        }
    })
}

// ============================================================================
// PICK LIST STYLE
// ============================================================================

pub fn menu_style(_: &Theme) -> iced::overlay::menu::Style {
    with_palette(|p| iced::overlay::menu::Style {
        text_color: p.text_secondary,
        background: Background::Color(p.bg_card),
        border: Border {
            width: 1.0,
            color: p.border,
            radius: 4.0.into(),
        },
        selected_text_color: p.text_primary,
        selected_background: Background::Color(p.bg_active),
        shadow: iced::Shadow::default(),
    })
}

/// Context menu container style (dropdown menus, right-click menus)
pub fn context_menu_container() -> container::Style {
    let p = palette();
    container::Style {
        background: Some(Background::Color(p.bg_control)),
        border: Border {
            radius: 6.0.into(),
            color: p.border_subtle,
            width: 1.0,
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}

/// Rule card container style
pub fn rule_card_container() -> container::Style {
    let p = palette();
    container::Style {
        background: Some(Background::Color(p.bg_card)),
        border: Border {
            radius: 8.0.into(),
            color: Color::TRANSPARENT,
            width: 0.0,
        },
        ..Default::default()
    }
}

pub fn pick_list_style(_: &Theme, status: pick_list::Status) -> pick_list::Style {
    with_palette(|p| {
        let (bg, border) = match status {
            pick_list::Status::Active => (p.bg_control, p.border),
            pick_list::Status::Hovered => (p.bg_hover, p.border),
            pick_list::Status::Opened { .. } => (p.bg_active, p.accent),
        };

        pick_list::Style {
            text_color: p.text_primary,
            placeholder_color: p.text_muted,
            background: Background::Color(bg),
            border: Border {
                radius: 6.0.into(),
                width: 1.0,
                color: border,
            },
            handle_color: p.text_secondary,
        }
    })
}
