//! Repository API trait for write operations
//!
//! This trait covers repository-application mapping operations:
//! - Get a single repository's details (for read-merge-write)
//! - Replace all application mappings for a repository
//!
//! Note: Read-only listing is handled by [`ListingApi::list_repos`].
//! This trait adds the write side for linking apps to repos.

use async_trait::async_trait;

use crate::client::models::{
    ReplaceRepoAppMappingsRequest, ReplaceRepoAppMappingsResponse, Repository,
};
use crate::error::Result;

/// Repository write operations for the StackHawk API
///
/// Manages the association between repositories (from attack surface mapping)
/// and StackHawk applications. The primary use case is linking apps to repos
/// to enable API Discovery coverage tracking.
///
/// **Important**: The underlying API uses full-replacement semantics for app
/// mappings. The `replace_repo_app_mappings` method replaces ALL mappings,
/// so callers must implement a read-merge-write pattern to preserve existing
/// associations when adding a new link.
#[async_trait]
#[allow(dead_code)]
pub trait RepoApi: Send + Sync {
    // ========================================================================
    // Read Operations
    // ========================================================================

    /// Get a single repository by ID.
    ///
    /// Used before mutations (read-merge-write) to fetch existing app
    /// mappings that must be preserved.
    async fn get_repo(&self, org_id: &str, repo_id: &str) -> Result<Repository>;

    // ========================================================================
    // Write Operations
    // ========================================================================

    /// Replace all application mappings for a repository.
    ///
    /// **This is a full replacement** — any existing mappings not included
    /// in the request will be removed. Callers should:
    /// 1. Read current mappings via `get_repo()` or `list_repos()`
    /// 2. Merge desired changes into the existing list
    /// 3. POST the complete list
    ///
    /// Each app info can specify either:
    /// - `id` only — links an existing application
    /// - `name` only — creates a new application and links it
    async fn replace_repo_app_mappings(
        &self,
        request: ReplaceRepoAppMappingsRequest,
    ) -> Result<ReplaceRepoAppMappingsResponse>;
}
