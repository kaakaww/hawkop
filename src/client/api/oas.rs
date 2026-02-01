//! OAS (OpenAPI Specification) API trait
//!
//! Provides endpoints for managing hosted OpenAPI specifications.

use async_trait::async_trait;

use crate::client::models::OASAsset;
use crate::error::Result;

/// API for OAS (OpenAPI Specification) operations
#[async_trait]
pub trait OASApi: Send + Sync {
    /// Get the content of an OpenAPI specification by ID
    ///
    /// Returns the OAS content as a JSON string.
    async fn get_oas(&self, org_id: &str, oas_id: &str) -> Result<String>;

    /// Get the OAS specs mapped to an application
    ///
    /// Returns the list of OAS assets that are mapped to the specified application.
    async fn get_oas_mappings(&self, app_id: &str) -> Result<Vec<OASAsset>>;
}
