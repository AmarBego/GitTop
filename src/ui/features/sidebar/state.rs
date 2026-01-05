//! Sidebar state structure for view rendering.

use crate::github::{SubjectType, UserInfo};
use crate::settings::IconTheme;

/// Persistent state for the sidebar (filters, selections).
#[derive(Debug, Clone, Default)]
pub struct SidebarState {
    pub show_all: bool,
    /// None means "All Types"
    pub selected_type: Option<SubjectType>,
    /// None means "All Repos"
    pub selected_repo: Option<String>,
}

/// View arguments for rendering the sidebar.
pub struct SidebarViewArgs<'a> {
    pub user: &'a UserInfo,
    pub accounts: Vec<String>,
    pub type_counts: &'a [(SubjectType, usize)],
    pub repo_counts: &'a [(String, usize)],
    pub selected_type: Option<SubjectType>,
    pub selected_repo: Option<&'a str>,
    pub total_count: usize,
    pub total_repo_count: usize,
    pub icon_theme: IconTheme,
    pub width: f32,
    pub power_mode: bool,
}
