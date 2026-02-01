//! Environment API trait
//!
//! Provides methods for managing application environments.

use async_trait::async_trait;

use crate::client::PaginationParams;
use crate::client::models::{Application, Environment};
use crate::error::Result;

/// Environment management API
///
/// Provides operations for managing application environments.
#[async_trait]
pub trait EnvironmentApi: Send + Sync {
    /// List environments for an application
    ///
    /// Returns all environments belonging to the specified application.
    async fn list_environments(
        &self,
        app_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Environment>>;

    /// Get the default YAML configuration for an environment
    ///
    /// Returns a starter HawkScan configuration pre-populated with
    /// the app ID and environment settings.
    async fn get_environment_default_config(&self, app_id: &str, env_id: &str) -> Result<String>;

    /// Create a new environment for an application
    ///
    /// Returns the created application/environment details.
    async fn create_environment(&self, app_id: &str, env_name: &str) -> Result<Application>;

    /// Delete an environment
    ///
    /// **Warning**: This also permanently deletes all scan results
    /// collected for this environment.
    async fn delete_environment(&self, app_id: &str, env_id: &str) -> Result<()>;
}
