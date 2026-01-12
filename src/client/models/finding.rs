//! Finding and alert models for scan drill-down

use serde::{Deserialize, Serialize};

use super::scan::{AlertStats, Scan, ScanMetadata, ScanTag};

/// Application alert (plugin-level finding summary)
///
/// Represents a vulnerability type detected by a specific scanner plugin.
/// Contains aggregate stats across all affected paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationAlert {
    /// Plugin identifier (e.g., "40012" for SQL Injection)
    #[serde(default)]
    pub plugin_id: String,

    /// Plugin/vulnerability name
    #[serde(default)]
    pub name: String,

    /// Detailed description of the vulnerability (markdown)
    #[serde(default)]
    pub description: String,

    /// Severity level: "High", "Medium", "Low"
    #[serde(default)]
    pub severity: String,

    /// CWE identifier (e.g., "89" for SQL Injection)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwe_id: Option<String>,

    /// Reference URLs for remediation
    #[serde(default)]
    pub references: Vec<String>,

    /// Number of affected paths/URIs
    #[serde(default)]
    pub uri_count: u32,

    /// Triage status breakdown (new vs triaged counts by severity)
    #[serde(default)]
    pub alert_status_stats: Vec<super::scan::AlertStatusStats>,
}

/// Application alert URI (path-level finding)
///
/// Represents a specific vulnerable endpoint discovered by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationAlertUri {
    /// Unique identifier for this alert+path combination
    pub alert_uri_id: String,

    /// The affected URI path
    pub uri: String,

    /// HTTP method (GET, POST, etc.)
    pub request_method: String,

    /// Message ID for retrieving request/response details
    pub msg_id: String,

    /// Triage status: UNKNOWN (new), PROMOTED, FALSE_POSITIVE, RISK_ACCEPTED
    #[serde(default)]
    pub status: String,

    /// Plugin ID that detected this finding
    pub plugin_id: String,

    /// Triage note/comment if present
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_rule_note: Option<String>,

    /// Timestamp of last triage update
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_i64_or_string"
    )]
    pub matched_rule_last_updated: Option<i64>,
}

/// Alert response containing alert details and affected paths
///
/// Returned by the alert findings endpoint, includes pagination for paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertResponse {
    /// The alert/plugin details
    pub alert: ApplicationAlert,

    /// List of affected paths (paginated)
    #[serde(default)]
    pub application_scan_alert_uris: Vec<ApplicationAlertUri>,

    /// Application host URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_host: Option<String>,

    /// Vulnerability category
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// OWASP cheatsheet reference URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cheatsheet: Option<String>,

    /// Next page token for pagination
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,

    /// Total count of paths for this alert
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_i64_or_string"
    )]
    pub total_count: Option<i64>,
}

/// HTTP scan message containing request and response details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanMessage {
    /// Message ID
    pub id: String,

    /// HTTP request headers
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_header: Option<String>,

    /// HTTP request body
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_body: Option<String>,

    /// HTTP response headers
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_header: Option<String>,

    /// HTTP response body
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,

    /// Cookie parameters
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cookie_params: Option<String>,
}

/// Alert message response with full finding details
///
/// Includes the HTTP request/response and optional curl validation command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertMsgResponse {
    /// HTTP request/response details
    pub scan_message: ScanMessage,

    /// Affected URI path
    #[serde(default)]
    pub uri: String,

    /// Evidence found (e.g., error message snippet)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,

    /// Additional information about the finding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_info: Option<String>,

    /// Vulnerability description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Vulnerable parameter name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,

    /// Curl command to reproduce the finding
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_command: Option<String>,
}

/// Wrapper for scan alerts list response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanAlertsResponse {
    /// Scan result with populated application_alerts
    #[serde(default)]
    pub application_scan_results: Vec<ScanResultWithAlerts>,

    /// Next page token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,

    /// Total alert count
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_i64_or_string"
    )]
    pub total_count: Option<i64>,
}

/// Scan result containing populated alerts list
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResultWithAlerts {
    /// Core scan data (may not be present in some API responses)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan: Option<Scan>,

    /// List of alerts/plugins detected in this scan
    #[serde(default)]
    pub application_alerts: Vec<ApplicationAlert>,

    /// Alert statistics
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alert_stats: Option<AlertStats>,

    /// Scan duration in seconds (API returns as string)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_duration: Option<String>,

    /// URL count (API may return as integer)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_int_or_string"
    )]
    pub url_count: Option<u32>,

    /// Application host URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

/// Custom deserializer for fields that may be int or string (u32)
fn deserialize_optional_int_or_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<u32>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntOrString {
        Int(u32),
        String(String),
    }

    match Option::<IntOrString>::deserialize(deserializer)? {
        Some(IntOrString::Int(i)) => Ok(Some(i)),
        Some(IntOrString::String(s)) => s.parse().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

/// Custom deserializer for i64 fields that may come as strings
fn deserialize_optional_i64_or_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<i64>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IntOrString {
        Int(i64),
        String(String),
    }

    match Option::<IntOrString>::deserialize(deserializer)? {
        Some(IntOrString::Int(i)) => Ok(Some(i)),
        Some(IntOrString::String(s)) => s.parse().map(Some).map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}
