//! View modules for the notifications screen.
//!
//! Contains all view/rendering logic for the notifications screen.
//! - `bulk`: Bulk action bar for Power Mode
//! - `content`: Main content area with notification list
//! - `group`: Collapsible notification group headers
//! - `header`: Content header with title, sync status, filters
//! - `sidebar`: Sidebar navigation and filtering
//! - `sidebar_state`: State structure for sidebar rendering
//! - `states`: Loading, error, and empty state views

mod bulk;
mod content;
pub mod group;
mod header;
pub mod sidebar;
pub mod sidebar_state;
pub mod states;

// Re-export commonly used view functions for use by screen.rs
pub use sidebar::view_sidebar;
pub use sidebar_state::SidebarState;

// impl blocks in bulk, content, and header extend NotificationsScreen
// and are automatically available when this module is imported.
