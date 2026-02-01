//! Configuration API trait
//!
//! Provides methods for managing organization scan configurations.

use async_trait::async_trait;

use crate::client::models::{ConfigType, ValidatedAssetResponse};
use crate::error::Result;

/// Configuration management API
///
/// Provides CRUD operations for organization scan configurations.
#[async_trait]
pub trait ConfigApi: Send + Sync {
    /// Get a scan configuration's content by name
    ///
    /// This is a two-step process:
    /// 1. Get the presigned download URL from the API
    /// 2. Fetch the actual YAML content from that URL
    ///
    /// Returns the YAML configuration content as a string.
    async fn get_scan_config(&self, org_id: &str, config_name: &str) -> Result<String>;

    /// Create or update a scan configuration
    ///
    /// # Arguments
    /// * `org_id` - Organization ID
    /// * `name` - Configuration name
    /// * `content` - YAML configuration content
    /// * `config_type` - Configuration scope (ORG, APP, or TARGET)
    async fn set_scan_config(
        &self,
        org_id: &str,
        name: &str,
        content: &str,
        config_type: ConfigType,
    ) -> Result<()>;

    /// Delete a scan configuration
    async fn delete_scan_config(&self, org_id: &str, config_name: &str) -> Result<()>;

    /// Rename a scan configuration
    async fn rename_scan_config(&self, org_id: &str, old_name: &str, new_name: &str) -> Result<()>;

    /// Validate a scan configuration
    ///
    /// Returns validation markers (errors, warnings) for the configuration.
    async fn validate_scan_config(
        &self,
        org_id: &str,
        content: &str,
    ) -> Result<ValidatedAssetResponse>;
}
