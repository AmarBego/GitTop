//! Cache module - Persistent storage and smart caching.
//!
//! Uses sled for disk persistence and in-memory caching for hot data.

mod disk;

#[allow(unused_imports)]
pub use disk::{CacheError, DiskCache};
