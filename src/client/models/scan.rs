//! Scan models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Scan result from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scan {
    /// Scan ID
    #[serde(rename = "id")]
    pub id: String,

    /// Application ID
    #[serde(default)]
    pub application_id: String,

    /// Application name
    #[serde(default)]
    pub application_name: String,

    /// Environment name
    #[serde(default)]
    pub env: String,

    /// Scan status (STARTED, COMPLETED, ERROR, UNKNOWN)
    #[serde(default)]
    pub status: String,

    /// Timestamp when scan started (Unix epoch as integer or string)
    #[serde(default, deserialize_with = "deserialize_timestamp")]
    pub timestamp: String,

    /// HawkScan version used for this scan
    #[serde(default)]
    pub version: String,

    /// User ID who initiated the scan
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_user_id: Option<String>,
}

/// Custom deserializer for timestamp that handles both int64 and string
fn deserialize_timestamp<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TimestampValue {
        Integer(i64),
        String(String),
    }

    match TimestampValue::deserialize(deserializer)? {
        TimestampValue::Integer(i) => Ok(i.to_string()),
        TimestampValue::String(s) => Ok(s),
    }
}

/// Full scan result with duration and stats from applicationScanResults
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    /// Core scan data
    pub scan: Scan,

    /// Scan duration in seconds (as string from API)
    #[serde(default)]
    pub scan_duration: Option<String>,

    /// Number of URLs scanned
    #[serde(default)]
    pub url_count: Option<u32>,

    /// Alert statistics
    #[serde(default)]
    pub alert_stats: Option<AlertStats>,

    /// Severity statistics - map of severity name to count
    #[serde(default)]
    pub severity_stats: Option<HashMap<String, u32>>,

    /// Application host URL
    #[serde(default)]
    pub app_host: Option<String>,

    /// Policy name used for this scan (may be empty, prefer metadata.tags.policyDisplayName)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_name: Option<String>,

    /// Scan tags (name-value pairs)
    #[serde(default)]
    pub tags: Vec<ScanTag>,

    /// Scan metadata with extended context (userId, policyName, etc.)
    #[serde(default)]
    pub metadata: Option<ScanMetadata>,
}

/// Scan tag (name-value metadata pair)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanTag {
    /// Tag name
    #[serde(default)]
    pub name: String,

    /// Tag value
    #[serde(default)]
    pub value: String,
}

/// Alert statistics from scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertStats {
    /// Total number of alerts
    #[serde(default)]
    pub total_alerts: u32,

    /// Number of unique alerts
    #[serde(default)]
    pub unique_alerts: u32,

    /// Stats broken down by alert status
    #[serde(default)]
    pub alert_status_stats: Vec<AlertStatusStats>,
}

/// Alert statistics by status (UNKNOWN = new, PROMOTED = triaged)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertStatusStats {
    /// Alert status: UNKNOWN (new), PROMOTED (triaged), etc.
    #[serde(default)]
    pub alert_status: String,

    /// Total count for this status
    #[serde(default)]
    pub total_count: u32,

    /// Breakdown by severity
    #[serde(default)]
    pub severity_stats: HashMap<String, u32>,
}

/// Scan metadata containing tags as key-value pairs
///
/// The API returns `metadata.tags` as a HashMap with various scan context:
/// - `userId`: The UUID of the user who initiated the scan
/// - `policyName`: The policy code name (e.g., "DEFAULT_API")
/// - `policyDisplayName`: The human-friendly policy name (e.g., "OpenAPI/REST API")
/// - `isCustomPolicy`: Whether the policy is customized
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanMetadata {
    /// Key-value tags containing scan context (userId, policyName, etc.)
    #[serde(default)]
    pub tags: HashMap<String, String>,
}
