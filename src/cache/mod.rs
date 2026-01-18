//! Local cache for API responses
//!
//! Provides SQLite-backed caching with file blob storage for large responses.
//! Designed to speed up shell completions and reduce API calls.

pub mod client;
pub mod key;
pub mod storage;

use std::time::Duration;

/// Cache TTL configuration per data type
///
/// These constants define caching duration for each type of API response.
/// Currently used for completions; will expand to full API caching later.
pub struct CacheTtl;

#[allow(dead_code)]
impl CacheTtl {
    // Scan data - new scans appear frequently
    pub const SCAN_LIST: Duration = Duration::from_secs(2 * 60); // 2 min
    pub const SCAN_DETAIL_COMPLETED: Duration = Duration::from_secs(24 * 60 * 60); // 24 hr
    pub const SCAN_DETAIL_RUNNING: Duration = Duration::from_secs(30); // 30 sec

    // Alert/finding data - triage can change
    pub const ALERTS: Duration = Duration::from_secs(10 * 60); // 10 min
    pub const ALERT_PATHS: Duration = Duration::from_secs(10 * 60); // 10 min

    // Completion caching - scan findings are stable once scan completes
    // Plugin IDs and URI IDs don't change, only triage state does
    pub const COMPLETION_ALERTS: Duration = Duration::from_secs(4 * 60 * 60); // 4 hr

    // Relatively stable data
    pub const APPS: Duration = Duration::from_secs(60 * 60); // 1 hr
    pub const ORGS: Duration = Duration::from_secs(60 * 60); // 1 hr
    pub const USERS: Duration = Duration::from_secs(60 * 60); // 1 hr
    pub const TEAMS: Duration = Duration::from_secs(60); // 1 min - teams change frequently via CRUD
    pub const POLICIES: Duration = Duration::from_secs(60 * 60); // 1 hr

    // Other data
    pub const REPOS: Duration = Duration::from_secs(60 * 60); // 1 hr
    pub const OAS: Duration = Duration::from_secs(60 * 60); // 1 hr
    pub const SCAN_CONFIGS: Duration = Duration::from_secs(60 * 60); // 1 hr
    pub const SECRETS: Duration = Duration::from_secs(60 * 60); // 1 hr
    pub const AUDIT: Duration = Duration::from_secs(5 * 60); // 5 min
}

// Re-export main types
pub use client::CachedStackHawkClient;
pub use key::cache_key;
pub use storage::CacheStorage;
