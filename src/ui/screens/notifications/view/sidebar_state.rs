//! Sidebar state structure for view rendering.

use crate::github::{SubjectType, UserInfo};
use crate::settings::IconTheme;

pub struct SidebarState<'a> {
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
