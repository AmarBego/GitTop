//! Thread action state.
//!
//! This feature handles individual notification thread operations:
//! - Opening notifications in browser
//! - Marking individual threads as read
//! - Marking individual threads as done

use std::collections::HashSet;

/// State for pending thread operations.
///
/// Tracks in-flight operations to prevent duplicate requests
/// and enable optimistic UI updates.
#[derive(Debug, Clone, Default)]
pub struct ThreadActionState {
    pub pending_mark_read: HashSet<String>,
    pub pending_mark_done: HashSet<String>,
    pub pending_mark_all: bool,
}

impl ThreadActionState {
    pub fn new() -> Self {
        Self::default()
    }
}
