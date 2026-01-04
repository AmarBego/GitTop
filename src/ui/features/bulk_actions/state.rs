//! Bulk action state.
//!
//! This feature handles multi-select operations on notifications:
//! - Toggle bulk selection mode
//! - Select/deselect notifications
//! - Bulk mark as read
//! - Bulk mark as done

use std::collections::HashSet;

/// State for bulk selection and operations.
#[derive(Debug, Clone, Default)]
pub struct BulkActionState {
    pub selected_ids: HashSet<String>,
    pub bulk_mode: bool,
}

impl BulkActionState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a notification is selected.
    pub fn is_selected(&self, id: &str) -> bool {
        self.selected_ids.contains(id)
    }

    /// Get count of selected notifications.
    pub fn selection_count(&self) -> usize {
        self.selected_ids.len()
    }

    /// Clear all selections and exit bulk mode.
    pub fn clear(&mut self) {
        self.selected_ids.clear();
        self.selected_ids.shrink_to_fit();
        self.bulk_mode = false;
    }
}
