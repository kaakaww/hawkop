//! Application API trait for CRUD operations
//!
//! This trait covers all application lifecycle operations including:
//! - Create new applications
//! - Get application details
//! - Update application properties
//! - Delete applications

use async_trait::async_trait;

use crate::client::models::{Application, CreateApplicationRequest};
use crate::error::Result;

/// Application management operations for the StackHawk API
///
/// This trait covers CRUD operations for applications, the core resource
/// in StackHawk that groups scan results and coordinates scan settings.
#[async_trait]
#[allow(dead_code)]
pub trait AppApi: Send + Sync {
    // ========================================================================
    // Read Operations
    // ========================================================================

    /// Get application details by ID.
    ///
    /// Returns the full application record including environment info.
    async fn get_app(&self, app_id: &str) -> Result<Application>;

    // ========================================================================
    // Write Operations
    // ========================================================================

    /// Create a new application in the organization.
    ///
    /// Requires at minimum a name and environment. Returns the created
    /// application with its generated ID.
    async fn create_app(
        &self,
        org_id: &str,
        request: CreateApplicationRequest,
    ) -> Result<Application>;

    /// Update an existing application.
    ///
    /// Currently only the application name can be updated. Returns the
    /// updated application.
    async fn update_app(&self, app_id: &str, name: &str) -> Result<Application>;

    /// Delete an application.
    ///
    /// **Destructive**: permanently removes the application and all its
    /// environments and scan results.
    async fn delete_app(&self, app_id: &str) -> Result<()>;
}
