//! Perch (hosted scanning) models
//!
//! Models for controlling hosted scans via the StackHawk Perch service.
//! These endpoints allow starting, stopping, and monitoring cloud-based scans.

use serde::{Deserialize, Serialize};

/// Status of a hosted scan device/runner
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerchDevice {
    /// Application ID this device is scanning
    pub application_id: Option<String>,

    /// Organization ID
    pub org_id: Option<String>,

    /// Device/runner ID
    pub id: Option<String>,

    /// Device name
    pub name: Option<String>,

    /// Device network address
    pub device_address: Option<String>,

    /// Current status (e.g., "RUNNING", "STOPPED", "IDLE")
    pub status: Option<String>,

    /// User who initiated the scan
    pub user_id: Option<String>,

    /// When the device was created (Unix timestamp)
    pub created_date: Option<i64>,

    /// Current command being executed
    pub command: Option<PerchCommand>,
}

/// A command issued to the Perch service
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerchCommand {
    /// Command type (e.g., "START", "STOP")
    pub command: Option<String>,

    /// Command ID
    pub id: Option<String>,

    /// Target URL for the scan
    pub target_url: Option<String>,

    /// Error information if command failed
    pub error: Option<PerchError>,
}

/// Error information from a Perch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerchError {
    /// Error type/code
    pub error_type: Option<String>,

    /// Human-readable error message
    pub error_message: Option<String>,
}

/// Request to start or stop a hosted scan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerchCommandRequest {
    /// The command to execute
    pub command: Option<PerchCommand>,

    /// Request ID (optional, for tracking)
    pub id: Option<String>,

    /// Scan configuration name to use (optional)
    pub scan_config: Option<String>,

    /// Environment to scan (optional)
    pub env: Option<String>,
}

/// Response from a Perch command (start/stop)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerchCommandResponse {
    /// Response ID
    pub id: Option<String>,
}

/// Response from getting device status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPerchDeviceResponse {
    /// The device/runner information
    pub device: Option<PerchDevice>,
}

impl PerchDevice {
    /// Check if the device is currently running a scan
    pub fn is_running(&self) -> bool {
        self.status
            .as_ref()
            .map(|s| s.eq_ignore_ascii_case("RUNNING") || s.eq_ignore_ascii_case("SCANNING"))
            .unwrap_or(false)
    }

    /// Check if the device is idle/stopped
    #[allow(dead_code)] // Kept for future use
    pub fn is_idle(&self) -> bool {
        self.status
            .as_ref()
            .map(|s| {
                s.eq_ignore_ascii_case("IDLE")
                    || s.eq_ignore_ascii_case("STOPPED")
                    || s.eq_ignore_ascii_case("COMPLETE")
            })
            .unwrap_or(true) // Default to idle if no status
    }

    /// Get a human-readable status string
    #[allow(dead_code)] // Kept for future use
    pub fn status_display(&self) -> &str {
        self.status.as_deref().unwrap_or("Unknown")
    }
}
