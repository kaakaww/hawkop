//! SQLite-based cache storage with file blob support
//!
//! Stores small responses inline in SQLite, large responses (>10KB) as files.

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::CacheError;

/// Schema version - increment to trigger nuke-and-rebuild
const SCHEMA_VERSION: i32 = 1;

/// Responses larger than this are stored as external blobs
const INLINE_THRESHOLD: usize = 10 * 1024; // 10KB

type Result<T> = std::result::Result<T, CacheError>;

/// SQLite-backed cache storage with file blob support
pub struct CacheStorage {
    conn: Connection,
    blobs_dir: PathBuf,
}

impl CacheStorage {
    /// Open or create cache storage at the default XDG cache location
    pub fn open() -> Result<Self> {
        let cache_dir = Self::cache_dir()?;
        Self::open_at(&cache_dir)
    }

    /// Get the cache directory path (~/.cache/hawkop on Linux/macOS)
    pub fn cache_dir() -> Result<PathBuf> {
        let cache_base = dirs::cache_dir().ok_or(CacheError::NoHome)?;
        Ok(cache_base.join("hawkop"))
    }

    /// Open cache storage at a specific directory (for testing)
    pub fn open_at(cache_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(cache_dir)
            .map_err(|e| CacheError::Io(format!("Failed to create cache dir: {}", e)))?;

        let db_path = cache_dir.join("cache.db");
        let blobs_dir = cache_dir.join("blobs");
        std::fs::create_dir_all(&blobs_dir)
            .map_err(|e| CacheError::Io(format!("Failed to create blobs dir: {}", e)))?;

        let conn = Connection::open(&db_path)?;

        // Check schema version - nuke if mismatched
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap_or(0);

        if version != 0 && version != SCHEMA_VERSION {
            log::info!(
                "Cache schema version mismatch ({} != {}), rebuilding",
                version,
                SCHEMA_VERSION
            );
            drop(conn);
            Self::nuke(&db_path, &blobs_dir)?;
            return Self::open_at(cache_dir);
        }

        // Initialize schema
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS cache_entries (
                cache_key TEXT PRIMARY KEY NOT NULL,
                org_id TEXT,
                endpoint TEXT NOT NULL,
                data TEXT,
                blob_path TEXT,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                size_bytes INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_expires_at ON cache_entries(expires_at);
            CREATE INDEX IF NOT EXISTS idx_org_id ON cache_entries(org_id);
            CREATE INDEX IF NOT EXISTS idx_endpoint ON cache_entries(endpoint);
            "#,
        )?;

        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;

        Ok(Self { conn, blobs_dir })
    }

    /// Get cached data if valid (not expired)
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let now = Utc::now().timestamp();

        let result: Option<(Option<String>, Option<String>)> = self
            .conn
            .query_row(
                "SELECT data, blob_path FROM cache_entries
                 WHERE cache_key = ?1 AND expires_at > ?2",
                params![key, now],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        match result {
            Some((Some(data), None)) => Ok(Some(data.into_bytes())),
            Some((None, Some(blob_path))) => {
                let full_path = self.blobs_dir.join(&blob_path);
                match std::fs::read(&full_path) {
                    Ok(data) => Ok(Some(data)),
                    Err(e) => {
                        log::warn!("Failed to read blob {}: {}", blob_path, e);
                        // Delete stale entry
                        let _ = self
                            .conn
                            .execute("DELETE FROM cache_entries WHERE cache_key = ?1", [key]);
                        Ok(None)
                    }
                }
            }
            _ => Ok(None),
        }
    }

    /// Store data with TTL
    pub fn put(
        &self,
        key: &str,
        data: &[u8],
        endpoint: &str,
        org_id: Option<&str>,
        ttl: Duration,
    ) -> Result<()> {
        let now = Utc::now().timestamp();
        let expires = now + ttl.as_secs() as i64;

        if data.len() <= INLINE_THRESHOLD {
            // Store inline in SQLite
            self.conn.execute(
                "INSERT OR REPLACE INTO cache_entries
                 (cache_key, org_id, endpoint, data, blob_path, created_at, expires_at, size_bytes)
                 VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7)",
                params![
                    key,
                    org_id,
                    endpoint,
                    String::from_utf8_lossy(data).to_string(),
                    now,
                    expires,
                    data.len()
                ],
            )?;
        } else {
            // Store as external blob
            let blob_path = self.write_blob(key, data)?;
            self.conn.execute(
                "INSERT OR REPLACE INTO cache_entries
                 (cache_key, org_id, endpoint, data, blob_path, created_at, expires_at, size_bytes)
                 VALUES (?1, ?2, ?3, NULL, ?4, ?5, ?6, ?7)",
                params![key, org_id, endpoint, blob_path, now, expires, data.len()],
            )?;
        }
        Ok(())
    }

    /// Clear all cache entries
    pub fn clear_all(&self) -> Result<ClearStats> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM cache_entries", [], |r| r.get(0))?;

        self.conn.execute("DELETE FROM cache_entries", [])?;

        // Clear blobs directory
        if self.blobs_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&self.blobs_dir) {
                log::warn!("Failed to clear blobs directory: {}", e);
            }
            std::fs::create_dir_all(&self.blobs_dir)
                .map_err(|e| CacheError::Io(format!("Failed to recreate blobs dir: {}", e)))?;
        }

        Ok(ClearStats {
            entries_removed: count as usize,
        })
    }

    /// Delete a specific cache entry by key
    #[allow(dead_code)]
    pub fn delete_by_key(&self, key: &str) -> Result<bool> {
        let deleted = self
            .conn
            .execute("DELETE FROM cache_entries WHERE cache_key = ?1", [key])?;
        Ok(deleted > 0)
    }

    /// Delete cache entries by endpoint pattern and optional org_id
    ///
    /// Used to invalidate cache after mutations. For example:
    /// - `delete_by_endpoint("list_teams", Some("org-123"))` - clears team list cache
    /// - `delete_by_endpoint("get_team", Some("org-123"))` - clears all team detail caches
    pub fn delete_by_endpoint(&self, endpoint: &str, org_id: Option<&str>) -> Result<usize> {
        let deleted = match org_id {
            Some(org) => self.conn.execute(
                "DELETE FROM cache_entries WHERE endpoint = ?1 AND org_id = ?2",
                params![endpoint, org],
            )?,
            None => self.conn.execute(
                "DELETE FROM cache_entries WHERE endpoint = ?1",
                params![endpoint],
            )?,
        };
        Ok(deleted)
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let now = Utc::now().timestamp();

        let total_entries: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM cache_entries", [], |r| r.get(0))?;

        let valid_entries: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM cache_entries WHERE expires_at > ?1",
            [now],
            |r| r.get(0),
        )?;

        let total_size: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(size_bytes), 0) FROM cache_entries",
            [],
            |r| r.get(0),
        )?;

        let oldest: Option<i64> = self
            .conn
            .query_row(
                "SELECT MIN(created_at) FROM cache_entries WHERE expires_at > ?1",
                [now],
                |r| r.get(0),
            )
            .optional()?
            .flatten();

        let newest: Option<i64> = self
            .conn
            .query_row(
                "SELECT MAX(created_at) FROM cache_entries WHERE expires_at > ?1",
                [now],
                |r| r.get(0),
            )
            .optional()?
            .flatten();

        Ok(CacheStats {
            total_entries: total_entries as usize,
            valid_entries: valid_entries as usize,
            expired_entries: (total_entries - valid_entries) as usize,
            total_size_bytes: total_size as usize,
            oldest_entry: oldest,
            newest_entry: newest,
        })
    }

    /// Write a blob file, sharded by first 2 chars of key
    fn write_blob(&self, key: &str, data: &[u8]) -> Result<String> {
        let shard = &key[..2.min(key.len())];
        let shard_dir = self.blobs_dir.join(shard);
        std::fs::create_dir_all(&shard_dir)
            .map_err(|e| CacheError::Io(format!("Failed to create shard dir: {}", e)))?;

        let filename = format!("{}.json", key);
        let rel_path = format!("{}/{}", shard, filename);
        let full_path = shard_dir.join(&filename);

        std::fs::write(&full_path, data)
            .map_err(|e| CacheError::Io(format!("Failed to write blob: {}", e)))?;

        Ok(rel_path)
    }

    /// Nuke the cache (delete DB and all blobs)
    fn nuke(db_path: &Path, blobs_dir: &Path) -> Result<()> {
        if db_path.exists() {
            std::fs::remove_file(db_path)
                .map_err(|e| CacheError::Io(format!("Failed to remove cache DB: {}", e)))?;
        }
        if blobs_dir.exists() {
            std::fs::remove_dir_all(blobs_dir)
                .map_err(|e| CacheError::Io(format!("Failed to remove blobs dir: {}", e)))?;
        }
        Ok(())
    }
}

/// Statistics about cache clear operation
#[derive(Debug)]
pub struct ClearStats {
    pub entries_removed: usize,
}

/// Statistics about cache state
#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
    pub total_size_bytes: usize,
    pub oldest_entry: Option<i64>,
    pub newest_entry: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_storage() -> (CacheStorage, TempDir) {
        let dir = TempDir::new().unwrap();
        let storage = CacheStorage::open_at(dir.path()).unwrap();
        (storage, dir)
    }

    #[test]
    fn test_put_get_inline() {
        let (storage, _dir) = test_storage();
        let data = b"small data";

        storage
            .put("key1", data, "test", None, Duration::from_secs(60))
            .unwrap();

        let result = storage.get("key1").unwrap();
        assert_eq!(result, Some(data.to_vec()));
    }

    #[test]
    fn test_put_get_blob() {
        let (storage, _dir) = test_storage();
        let data = vec![b'x'; 20_000]; // 20KB - will use blob

        storage
            .put("key2", &data, "test", None, Duration::from_secs(60))
            .unwrap();

        let result = storage.get("key2").unwrap();
        assert_eq!(result, Some(data));
    }

    #[test]
    fn test_expiration() {
        let (storage, _dir) = test_storage();

        // Store with 0 TTL (immediately expired)
        storage
            .put("key3", b"data", "test", None, Duration::from_secs(0))
            .unwrap();

        let result = storage.get("key3").unwrap();
        assert_eq!(result, None); // Expired
    }

    #[test]
    fn test_clear_all() {
        let (storage, _dir) = test_storage();

        storage
            .put("k1", b"d1", "test", None, Duration::from_secs(60))
            .unwrap();
        storage
            .put("k2", b"d2", "test", None, Duration::from_secs(60))
            .unwrap();

        let stats = storage.clear_all().unwrap();
        assert_eq!(stats.entries_removed, 2);

        assert!(storage.get("k1").unwrap().is_none());
        assert!(storage.get("k2").unwrap().is_none());
    }

    #[test]
    fn test_stats() {
        let (storage, _dir) = test_storage();

        storage
            .put("k1", b"data1", "test", None, Duration::from_secs(60))
            .unwrap();
        storage
            .put("k2", b"data2", "test", None, Duration::from_secs(60))
            .unwrap();

        let stats = storage.stats().unwrap();
        assert_eq!(stats.valid_entries, 2);
        assert!(stats.total_size_bytes > 0);
    }
}
