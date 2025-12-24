#![allow(dead_code)] // Infrastructure prepared for Phase 1, will be used when integrated
//! Disk Cache - Sled-backed persistent storage.
//!
//! Stores notification read status, sync timestamps, and cached responses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Cache-related errors.
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Cache directory not found")]
    NoCacheDir,
}

/// Cached notification status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedNotification {
    pub id: String,
    pub is_read: bool,
    pub last_seen: DateTime<Utc>,
}

/// Account sync metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    pub last_sync: DateTime<Utc>,
    pub etag: Option<String>,
    pub notification_count: usize,
}

/// Sled-backed persistent cache.
pub struct DiskCache {
    db: sled::Db,
}

impl DiskCache {
    /// Opens the cache at the default location.
    pub fn open() -> Result<Self, CacheError> {
        let path = Self::cache_path()?;
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    /// Gets the cache directory path.
    fn cache_path() -> Result<PathBuf, CacheError> {
        dirs::cache_dir()
            .map(|p| p.join("GitTop").join("cache.sled"))
            .ok_or(CacheError::NoCacheDir)
    }

    // =========================================================================
    // Notification Read Status
    // =========================================================================

    /// Save read status for a notification.
    pub fn save_read_status(&self, notification_id: &str, is_read: bool) -> Result<(), CacheError> {
        let tree = self.db.open_tree("read_status")?;
        let value = if is_read { b"1" } else { b"0" };
        tree.insert(notification_id.as_bytes(), value)?;
        Ok(())
    }

    /// Load read status for a notification.
    pub fn load_read_status(&self, notification_id: &str) -> Result<Option<bool>, CacheError> {
        let tree = self.db.open_tree("read_status")?;
        match tree.get(notification_id.as_bytes())? {
            Some(v) => Ok(Some(v.as_ref() == b"1")),
            None => Ok(None),
        }
    }

    // =========================================================================
    // Sync Metadata (per-account)
    // =========================================================================

    /// Save sync metadata for an account.
    pub fn save_sync_metadata(
        &self,
        account: &str,
        metadata: &SyncMetadata,
    ) -> Result<(), CacheError> {
        let tree = self.db.open_tree("sync_meta")?;
        let json =
            serde_json::to_vec(metadata).map_err(|e| CacheError::Serialization(e.to_string()))?;
        tree.insert(account.as_bytes(), json)?;
        Ok(())
    }

    /// Load sync metadata for an account.
    pub fn load_sync_metadata(&self, account: &str) -> Result<Option<SyncMetadata>, CacheError> {
        let tree = self.db.open_tree("sync_meta")?;
        match tree.get(account.as_bytes())? {
            Some(bytes) => {
                let meta: SyncMetadata = serde_json::from_slice(&bytes)
                    .map_err(|e| CacheError::Serialization(e.to_string()))?;
                Ok(Some(meta))
            }
            None => Ok(None),
        }
    }

    // =========================================================================
    // ETag Cache
    // =========================================================================

    /// Store an ETag and cached response body for a URL.
    pub fn save_etag_response(&self, url: &str, etag: &str, body: &[u8]) -> Result<(), CacheError> {
        // Store ETag separately for fast lookup (HEAD-like checks)
        let etags = self.db.open_tree("etags")?;
        etags.insert(url.as_bytes(), etag.as_bytes())?;

        // Store body separately
        let bodies = self.db.open_tree("bodies")?;
        bodies.insert(url.as_bytes(), body)?;

        Ok(())
    }

    /// Get cached ETag for a URL.
    pub fn get_etag(&self, url: &str) -> Result<Option<String>, CacheError> {
        let etags = self.db.open_tree("etags")?;
        match etags.get(url.as_bytes())? {
            Some(bytes) => {
                let etag = String::from_utf8_lossy(&bytes).to_string();
                Ok(Some(etag))
            }
            None => Ok(None),
        }
    }

    /// Get cached response body for a URL.
    pub fn get_cached_body(&self, url: &str) -> Result<Option<Vec<u8>>, CacheError> {
        let bodies = self.db.open_tree("bodies")?;
        match bodies.get(url.as_bytes())? {
            Some(bytes) => Ok(Some(bytes.to_vec())),
            None => Ok(None),
        }
    }

    /// Flush changes to disk.
    pub fn flush(&self) -> Result<(), CacheError> {
        self.db.flush()?;
        Ok(())
    }
}
