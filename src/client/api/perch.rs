//! Perch API trait for hosted scan control
//!
//! This trait covers operations for controlling hosted (cloud) scans:
//! - Start a scan on a cloud application
//! - Stop a running scan
//! - Get scan status

use async_trait::async_trait;

use crate::client::models::{PerchCommandResponse, PerchDevice};
use crate::error::Result;

/// Hosted scan control operations for the StackHawk Perch service
///
/// This trait provides methods to start, stop, and monitor cloud-based
/// security scans on hosted applications.
#[async_trait]
#[allow(dead_code)]
pub trait PerchApi: Send + Sync {
    /// Start a hosted scan for an application.
    ///
    /// Initiates a cloud-based security scan on the specified application.
    /// The scan will use the application's configured environment and settings.
    ///
    /// # Arguments
    /// * `app_id` - The application UUID to scan
    /// * `env` - Optional environment name to scan (defaults to app's default env)
    /// * `config` - Optional scan configuration name to use
    ///
    /// # Returns
    /// A response containing the command ID for tracking
    async fn start_scan(
        &self,
        app_id: &str,
        env: Option<&str>,
        config: Option<&str>,
    ) -> Result<PerchCommandResponse>;

    /// Stop a running hosted scan.
    ///
    /// Signals the scan to stop gracefully. The scan may take a moment
    /// to complete its current operation before fully stopping.
    ///
    /// # Arguments
    /// * `app_id` - The application UUID whose scan should be stopped
    ///
    /// # Returns
    /// A response containing the command ID for tracking
    async fn stop_scan(&self, app_id: &str) -> Result<PerchCommandResponse>;

    /// Get the status of a hosted scan.
    ///
    /// Returns information about the current scan state, including
    /// whether a scan is running, its progress, and any errors.
    ///
    /// # Arguments
    /// * `app_id` - The application UUID to check
    ///
    /// # Returns
    /// Device information including status and any active command
    async fn get_scan_status(&self, app_id: &str) -> Result<PerchDevice>;
}
