//! Application settings with persistence.
//!
//! Stores user preferences like icon theme and account list.

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
    pub accounts: Vec<StoredAccount>,
    /// Whether closing the window minimizes to tray instead of quitting.
    #[serde(default = "default_minimize_to_tray")]
    pub minimize_to_tray: bool,
}

fn default_minimize_to_tray() -> bool {
    true
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            icon_theme: IconTheme::Svg,
            accounts: Vec::new(),
            minimize_to_tray: true,
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
    pub fn active_account(&self) -> Option<&str> {
        self.accounts
            .iter()
            .find(|a| a.is_active)
            .map(|a| a.username.as_str())
    }
}
