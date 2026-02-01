//! Scan configuration models
//!
//! Models for managing organization and application scan configurations.

use serde::{Deserialize, Serialize};

/// Organization scan configuration (from list endpoint)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanConfig {
    /// Configuration name
    #[serde(default)]
    pub name: String,

    /// Configuration description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Organization ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
}

// ============================================================================
// Configuration Get/Download
// ============================================================================

/// Response from GET /configuration/{orgId}/{configName}
///
/// Contains a pre-signed URL to download the actual YAML content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetHostedAssetResponse {
    /// HTTP headers (if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<String>,

    /// HTTP method
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// Pre-signed S3 URL to download the configuration content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presigned_download_url: Option<String>,
}

// ============================================================================
// Configuration Upsert (Create/Update)
// ============================================================================

/// Configuration scope/type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ConfigType {
    /// Organization-scoped configuration (usable via hawk://configName)
    #[default]
    Org,
    /// Application-scoped configuration
    App,
    /// Target-scoped configuration
    Target,
}

impl std::fmt::Display for ConfigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Org => write!(f, "ORG"),
            Self::App => write!(f, "APP"),
            Self::Target => write!(f, "TARGET"),
        }
    }
}

/// Request to create or update a scan configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertScanConfigurationRequest {
    /// The HawkScan YAML configuration content
    pub conf: String,

    /// Configuration type/scope
    #[serde(default)]
    pub config_type: ConfigType,

    /// Configuration name
    pub name: String,
}

// ============================================================================
// Configuration Rename
// ============================================================================

/// Request to rename a scan configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameConfigurationRequest {
    /// The original name of the configuration
    pub original_name: String,

    /// The new name for the configuration
    pub new_name: String,
}

// ============================================================================
// Configuration Validation
// ============================================================================

/// Response from configuration validation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatedAssetResponse {
    /// List of validation problems/markers
    #[serde(default)]
    pub markers: Vec<ValidationMarker>,
}

impl ValidatedAssetResponse {
    /// Check if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        !self.markers.iter().any(|m| m.is_error())
    }

    /// Get only error markers
    pub fn errors(&self) -> Vec<&ValidationMarker> {
        self.markers.iter().filter(|m| m.is_error()).collect()
    }

    /// Get only warning markers
    pub fn warnings(&self) -> Vec<&ValidationMarker> {
        self.markers.iter().filter(|m| m.is_warning()).collect()
    }
}

/// A validation problem marker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationMarker {
    /// Problem code line
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// End column of the problem
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_column: Option<i32>,

    /// End line number of the problem
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_line_number: Option<i32>,

    /// Problem message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Owner (typically "StackHawk")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    /// Name of validated file/resource
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,

    /// Severity level (e.g., "error", "warning", "info")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,

    /// Start column of the problem
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_column: Option<i32>,

    /// Start line number of the problem
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line_number: Option<i32>,
}

impl ValidationMarker {
    /// Check if this marker is an error
    pub fn is_error(&self) -> bool {
        self.severity
            .as_ref()
            .map(|s| s.eq_ignore_ascii_case("error"))
            .unwrap_or(false)
    }

    /// Check if this marker is a warning
    pub fn is_warning(&self) -> bool {
        self.severity
            .as_ref()
            .map(|s| s.eq_ignore_ascii_case("warning"))
            .unwrap_or(false)
    }

    /// Format the marker location as "line:column"
    pub fn location(&self) -> String {
        match (self.start_line_number, self.start_column) {
            (Some(line), Some(col)) => format!("{}:{}", line, col),
            (Some(line), None) => format!("{}", line),
            _ => String::new(),
        }
    }
}
