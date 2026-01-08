//! Application settings with persistence.
//!
//! Stores user preferences like icon theme, app theme, and account list.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IconTheme {
    #[default]
    Svg,
    Emoji,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum AppTheme {
    Light = 0,
    Steam = 1,
    GtkDark = 2,
    Windows11 = 3,
    MacOS = 4,
    HighContrast = 5,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::platform_default()
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "Light"),
            Self::Steam => write!(f, "Steam"),
            Self::GtkDark => write!(f, "GTK Dark"),
            Self::Windows11 => write!(f, "Windows 11"),
            Self::MacOS => write!(f, "macOS"),
            Self::HighContrast => write!(f, "High Contrast"),
        }
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

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for AppTheme {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Light),
            1 => Ok(Self::Steam),
            2 => Ok(Self::GtkDark),
            3 => Ok(Self::Windows11),
            4 => Ok(Self::MacOS),
            5 => Ok(Self::HighContrast),
            v => Err(v),
        }
    }
}

/// Stored account information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAccount {
    pub username: String,
    pub is_active: bool,
}

/// Proxy settings (credentials stored securely in keyring)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxySettings {
    pub enabled: bool,
    pub url: String,
    /// Flag indicating if credentials are stored in keyring
    #[serde(default)]
    pub has_credentials: bool,
}

/// Application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub icon_theme: IconTheme,
    #[serde(default)]
    pub theme: AppTheme,
    pub accounts: Vec<StoredAccount>,
    #[serde(default = "default_minimize_to_tray")]
    pub minimize_to_tray: bool,
    #[serde(default = "default_font_scale")]
    pub notification_font_scale: f32,
    #[serde(default = "default_font_scale")]
    pub sidebar_font_scale: f32,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: f32,
    #[serde(default)]
    pub window_x: Option<i32>,
    #[serde(default)]
    pub window_y: Option<i32>,
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    #[serde(default = "default_power_mode")]
    pub power_mode: bool,
    #[serde(default = "default_show_details_panel")]
    pub show_details_panel: bool,
    #[serde(default)]
    pub proxy: ProxySettings,
    /// Check for updates on startup (opt-in, default: false)
    #[serde(default)]
    pub check_for_updates: bool,
}

fn default_minimize_to_tray() -> bool {
    true
}

fn default_font_scale() -> f32 {
    1.0
}

fn default_sidebar_width() -> f32 {
    220.0
}

fn default_window_width() -> f32 {
    800.0
}

fn default_window_height() -> f32 {
    640.0
}

fn default_power_mode() -> bool {
    false
}

fn default_show_details_panel() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            icon_theme: IconTheme::Svg,
            theme: AppTheme::default(),
            accounts: Vec::new(),
            minimize_to_tray: true,
            notification_font_scale: 1.0,
            sidebar_font_scale: 1.0,
            sidebar_width: 220.0,
            window_x: None,
            window_y: None,
            window_width: 800.0,
            window_height: 640.0,
            power_mode: false,
            show_details_panel: true,
            proxy: ProxySettings::default(),
            check_for_updates: false,
        }
    }
}

impl AppSettings {
    /// Get the settings file path.
    fn settings_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("GitTop").join("settings.json"))
    }

    /// Load settings from disk, or return defaults.
    pub fn load() -> Self {
        let Some(path) = Self::settings_path() else {
            tracing::warn!("Settings directory not available; using defaults");
            return Self::default();
        };

        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(settings) => settings,
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to parse settings; using defaults"
                    );
                    Self::default()
                }
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                tracing::debug!(path = %path.display(), "Settings file not found; using defaults");
                Self::default()
            }
            Err(e) => {
                tracing::error!(
                    path = %path.display(),
                    error = %e,
                    "Failed to read settings; using defaults"
                );
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::settings_path().ok_or_else(|| {
            let err = std::io::Error::new(std::io::ErrorKind::NotFound, "No config directory");
            tracing::error!(error = %err, "Unable to resolve settings path");
            err
        })?;

        if let Some(parent) = path.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            tracing::error!(
                path = %parent.display(),
                error = %e,
                "Failed to create settings directory"
            );
            return Err(e);
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            tracing::error!(error = %e, "Failed to serialize settings");
            std::io::Error::other(e)
        })?;

        if let Err(e) = fs::write(&path, content) {
            tracing::error!(path = %path.display(), error = %e, "Failed to save settings");
            return Err(e);
        }

        Ok(())
    }

    pub fn set_active_account(&mut self, username: &str) {
        let mut found = false;
        for acc in &mut self.accounts {
            acc.is_active = acc.username == username;
            found |= acc.is_active;
        }

        if !found {
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

    /// Apply theme and font scale settings globally.
    /// Call this after loading settings to initialize the UI theme.
    pub fn apply_theme(&self) {
        crate::ui::theme::set_theme(self.theme);
        crate::ui::theme::set_notification_font_scale(self.notification_font_scale);
        crate::ui::theme::set_sidebar_font_scale(self.sidebar_font_scale);
    }

    /// Save settings to disk, ignoring any errors.
    pub fn save_silent(&self) {
        let _ = self.save();
    }
}
