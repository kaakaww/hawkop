//! Environment models
//!
//! Models for managing application environments.

use serde::de;
use serde::{Deserialize, Deserializer, Serialize};

// ============================================================================
// Deserialization Helpers
// ============================================================================

/// Deserialize a value that may be a string or number to i64.
///
/// The StackHawk API sometimes returns numeric fields as strings.
fn deserialize_string_to_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(i64),
    }

    match Option::<StringOrNumber>::deserialize(deserializer)? {
        Some(StringOrNumber::String(s)) => s.parse::<i64>().map(Some).map_err(de::Error::custom),
        Some(StringOrNumber::Number(n)) => Ok(Some(n)),
        None => Ok(None),
    }
}

/// Deserialize a value that may be a string or number to i32, defaulting to 0.
fn deserialize_string_to_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(i32),
    }

    match Option::<StringOrNumber>::deserialize(deserializer)? {
        Some(StringOrNumber::String(s)) => s.parse::<i32>().map_err(de::Error::custom),
        Some(StringOrNumber::Number(n)) => Ok(n),
        None => Ok(0),
    }
}

/// An application environment with scan statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    /// Environment UUID
    #[serde(default)]
    pub environment_id: String,

    /// Environment name (e.g., "development", "production")
    #[serde(default)]
    pub environment_name: String,

    /// Type of the latest scan (DEFAULT, REST, GRAPHQL, GRPC, SOAP)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_scan_type: Option<String>,

    /// Summary of current scan results
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_scan_summary: Option<EnvScanSummary>,
}

/// Summary of scan results for an environment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvScanSummary {
    /// Scan ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_id: Option<String>,

    /// Application ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,

    /// Timestamp when scan was started (seconds since epoch)
    /// API may return this as a string or number
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_string_to_i64"
    )]
    pub timestamp: Option<i64>,

    /// Hash of the HawkScan configuration used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_hash: Option<String>,

    /// HawkScan version used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Alert statistics
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alert_stats: Option<EnvAlertStats>,
}

/// Alert statistics for a scan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvAlertStats {
    /// High severity count (API may return as string)
    #[serde(default, deserialize_with = "deserialize_string_to_i32")]
    pub high: i32,

    /// Medium severity count (API may return as string)
    #[serde(default, deserialize_with = "deserialize_string_to_i32")]
    pub medium: i32,

    /// Low severity count (API may return as string)
    #[serde(default, deserialize_with = "deserialize_string_to_i32")]
    pub low: i32,
}

// ============================================================================
// List Environments Response
// ============================================================================

/// Response from listing environments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEnvironmentsResponse {
    /// List of environments
    #[serde(default)]
    pub environments: Vec<Environment>,

    /// Token for next page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,

    /// Total count of environments (API may return as string)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_string_to_i64"
    )]
    pub total_count: Option<i64>,
}

// ============================================================================
// Create Environment Request
// ============================================================================

/// Request to create a new environment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewEnvironmentRequest {
    /// Environment name
    pub env: String,
}

// ============================================================================
// Default Config Response
// ============================================================================

/// Response containing the default YAML configuration for an environment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentConfigResponse {
    /// The HawkScan configuration (as nested object or string)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conf: Option<serde_json::Value>,

    /// Hash of the configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_hash: Option<String>,
}

impl EnvironmentConfigResponse {
    /// Convert the configuration to YAML string
    pub fn to_yaml(&self) -> Option<String> {
        self.conf.as_ref().and_then(|conf| {
            // If conf is already a string, return it
            if let Some(s) = conf.as_str() {
                return Some(s.to_string());
            }
            // Otherwise try to serialize as YAML
            serde_yaml::to_string(conf).ok()
        })
    }
}
