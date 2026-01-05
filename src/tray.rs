//! System tray management for GitTop.
//!
//! This module provides a platform-agnostic interface to the system tray.
//! The actual implementations live in the platform modules:
//! - Linux/FreeBSD: Uses `ksni` (pure-Rust StatusNotifierItem)
//! - Windows/macOS: Uses `tray-icon` (native platform APIs)

#[derive(Debug, Clone)]
pub enum TrayCommand {
    ShowWindow,
    Quit,
}

// Re-export the platform-specific TrayManager
pub use crate::platform::tray::TrayManager;
