//! Application settings with persistence.
//!
//! Stores user preferences like icon theme, app theme, and account list.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Icon rendering theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IconTheme {
    /// SVG icons from Lucide (better quality, ~4MB extra).
    #[default]
    Svg,
    /// Emoji/Unicode icons (minimal memory).
    Emoji,
}

/// Visual theme preset.
/// Platform-aware defaults: Linux uses GTK, Windows uses Windows11, macOS uses native.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    /// Clean light theme
    Light,
    /// Dark theme with blue-grey tones (inspired by Steam)
    Steam,
    /// GTK Adwaita-inspired dark theme (best for Linux)
    GtkDark,
    /// Windows 11 Fluent dark theme
    Windows11,
    /// macOS-inspired dark theme
    MacOS,
    /// High contrast for accessibility
    HighContrast,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::platform_default()
    }
}

impl AppTheme {
    /// Returns the best theme for the current platform.
    pub fn platform_default() -> Self {
        #[cfg(target_os = "linux")]
        {
            Self::GtkDark
        }

        #[cfg(target_os = "windows")]
        {
            Self::Windows11
        }

        #[cfg(target_os = "macos")]
        {
            Self::MacOS
        }

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            Self::Steam
        }
    }

    /// Convert to u8 for atomic storage.
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Light => 0,
            Self::Steam => 1,
            Self::GtkDark => 2,
            Self::Windows11 => 3,
            Self::MacOS => 4,
            Self::HighContrast => 5,
        }
    }

    /// Convert from u8 (from atomic storage).
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Light,
            1 => Self::Steam,
            2 => Self::GtkDark,
            3 => Self::Windows11,
            4 => Self::MacOS,
            5 => Self::HighContrast,
            _ => Self::platform_default(),
        }
    }
}

/// Stored account information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAccount {
    pub username: String,
    pub is_active: bool,
}

/// Application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub icon_theme: IconTheme,
    /// Visual theme preset.
    #[serde(default)]
    pub theme: AppTheme,
    pub accounts: Vec<StoredAccount>,
    /// Whether closing the window minimizes to tray instead of quitting.
    #[serde(default = "default_minimize_to_tray")]
    pub minimize_to_tray: bool,
    /// Font scale for notifications and sidebar (1.0 = default, range 0.8-1.5)
    #[serde(default = "default_font_scale")]
    pub font_scale: f32,
}

fn default_minimize_to_tray() -> bool {
    true
}

fn default_font_scale() -> f32 {
    1.0
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            icon_theme: IconTheme::Svg,
            theme: AppTheme::default(),
            accounts: Vec::new(),
            minimize_to_tray: true,
            font_scale: 1.0,
        }
    }
}

impl AppSettings {
    /// Get the settings file path.
    fn settings_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("gittop").join("settings.json"))
    }

    /// Load settings from disk, or return defaults.
    pub fn load() -> Self {
        Self::settings_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Save settings to disk.
    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::settings_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }

    /// Add or update an account.
    pub fn set_active_account(&mut self, username: &str) {
        // Deactivate all accounts first
        for acc in &mut self.accounts {
            acc.is_active = false;
        }

        // Find or add the account
        if let Some(acc) = self.accounts.iter_mut().find(|a| a.username == username) {
            acc.is_active = true;
        } else {
            self.accounts.push(StoredAccount {
                username: username.to_string(),
                is_active: true,
            });
        }
    }

    /// Remove an account by username.
    pub fn remove_account(&mut self, username: &str) {
        self.accounts.retain(|a| a.username != username);
    }

    /// Get the active account username.
    #[allow(dead_code)] // Reserved for multi-account feature
    pub fn active_account(&self) -> Option<&str> {
        self.accounts
            .iter()
            .find(|a| a.is_active)
            .map(|a| a.username.as_str())
    }
}
