//! StackHawk API client

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[cfg(test)]
pub mod mock;
pub mod pagination;
pub mod parallel;
pub mod rate_limit;
pub mod stackhawk;

#[cfg(test)]
#[allow(unused_imports)]
pub use mock::MockStackHawkClient;
#[allow(unused_imports)]
pub use pagination::{
    MAX_PAGE_SIZE, PagedResponse, PaginatedResponse, PaginationMeta, PaginationParams,
    ScanFilterParams, SortOrder,
};
#[allow(unused_imports)]
pub use parallel::fetch_remaining_pages;
pub use stackhawk::StackHawkClient;

/// StackHawk API client trait
#[async_trait]
pub trait StackHawkApi: Send + Sync {
    /// Authenticate with API key and get JWT token
    async fn authenticate(&self, api_key: &str) -> Result<JwtToken>;

    /// List all accessible organizations
    async fn list_orgs(&self) -> Result<Vec<Organization>>;

    /// List all applications for an organization with optional pagination
    async fn list_apps(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Application>>;

    /// List applications with pagination metadata for parallel fetching.
    ///
    /// Returns `PagedResponse` with `total_count` for calculating remaining pages.
    async fn list_apps_paged(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<PagedResponse<Application>>;

    /// List scans for an organization with optional pagination and filters
    async fn list_scans(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
        filters: Option<&ScanFilterParams>,
    ) -> Result<Vec<ScanResult>>;

    /// List scans with pagination metadata for parallel fetching.
    ///
    /// Returns `PagedResponse` with `total_count` for calculating remaining pages.
    async fn list_scans_paged(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
        filters: Option<&ScanFilterParams>,
    ) -> Result<PagedResponse<ScanResult>>;

    /// List users (members) for an organization with optional pagination
    async fn list_users(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<User>>;

    /// List teams for an organization with optional pagination
    async fn list_teams(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Team>>;

    /// List all StackHawk preset policies (read-only)
    async fn list_stackhawk_policies(&self) -> Result<Vec<StackHawkPolicy>>;

    /// List organization custom policies with optional pagination
    async fn list_org_policies(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<OrgPolicy>>;

    /// List repositories for an organization with optional pagination
    async fn list_repos(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Repository>>;

    /// List OpenAPI specification assets for an organization
    async fn list_oas(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<OASAsset>>;

    /// List scan configurations for an organization
    async fn list_scan_configs(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<ScanConfig>>;

    /// List user secrets (user-scoped, not org-scoped)
    async fn list_secrets(&self) -> Result<Vec<Secret>>;

    /// List audit log records for an organization
    async fn list_audit(
        &self,
        org_id: &str,
        filters: Option<&AuditFilterParams>,
    ) -> Result<Vec<AuditRecord>>;

    // ========================================================================
    // Scan Drill-Down Methods (for exploring scan results)
    // ========================================================================

    /// Get a single scan by ID
    ///
    /// Fetches detailed scan information including alert stats.
    async fn get_scan(&self, org_id: &str, scan_id: &str) -> Result<ScanResult>;

    /// List all alerts (plugins) for a scan
    ///
    /// Returns plugin-level finding summaries with counts and triage stats.
    async fn list_scan_alerts(
        &self,
        scan_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<ApplicationAlert>>;

    /// Get alert details with affected paths for a specific plugin
    ///
    /// Returns the alert info plus paginated list of vulnerable endpoints.
    async fn get_alert_with_paths(
        &self,
        scan_id: &str,
        plugin_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<AlertResponse>;

    /// Get HTTP request/response details for a specific finding
    ///
    /// Returns full message details including optional curl validation command.
    async fn get_alert_message(
        &self,
        scan_id: &str,
        alert_uri_id: &str,
        message_id: &str,
        include_curl: bool,
    ) -> Result<AlertMsgResponse>;
}

/// JWT authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtToken {
    /// The JWT token string
    pub token: String,

    /// Token expiration time
    #[serde(rename = "expiresAt")]
    pub expires_at: DateTime<Utc>,
}

/// Organization resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// Organization ID
    pub id: String,

    /// Organization name
    pub name: String,

    /// Number of users (optional, may not be in all responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_count: Option<usize>,

    /// Number of applications (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_count: Option<usize>,
}

/// Application resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    /// Application ID
    #[serde(rename = "applicationId")]
    pub id: String,

    /// Application name
    pub name: String,

    /// Environment name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,

    /// Risk level (optional)
    #[serde(skip_serializing_if = "Option::is_none", rename = "riskLevel")]
    pub risk_level: Option<String>,

    /// Application status (optional)
    #[serde(skip_serializing_if = "Option::is_none", rename = "applicationStatus")]
    pub status: Option<String>,

    /// Organization ID (optional)
    #[serde(skip_serializing_if = "Option::is_none", rename = "organizationId")]
    pub organization_id: Option<String>,

    /// Application type: "STANDARD" or "CLOUD"
    #[serde(skip_serializing_if = "Option::is_none", rename = "applicationType")]
    pub application_type: Option<String>,

    /// Cloud scan target (only for CLOUD apps)
    #[serde(skip_serializing_if = "Option::is_none", rename = "cloudScanTarget")]
    pub cloud_scan_target: Option<CloudScanTarget>,
}

/// Cloud scan target for hosted/cloud applications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudScanTarget {
    /// Target URL to scan
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_url: Option<String>,

    /// Whether the domain has been verified
    #[serde(default)]
    pub is_domain_verified: bool,
}

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
}

/// Custom deserializer for timestamp that handles both int64 and string
fn deserialize_timestamp<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::Deserialize;

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
    pub severity_stats: Option<std::collections::HashMap<String, u32>>,

    /// Application host URL
    #[serde(default)]
    pub app_host: Option<String>,
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
    pub severity_stats: std::collections::HashMap<String, u32>,
}

// ============================================================================
// Scan Alert / Finding Models (for drill-down exploration)
// ============================================================================

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
    pub alert_status_stats: Vec<AlertStatusStats>,
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
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_i64_or_string")]
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
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_i64_or_string")]
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
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_i64_or_string")]
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
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_int_or_string")]
    pub url_count: Option<u32>,

    /// Application host URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_host: Option<String>,
}

/// Custom deserializer for fields that may be int or string (u32)
fn deserialize_optional_int_or_string<'de, D>(deserializer: D) -> std::result::Result<Option<u32>, D::Error>
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
fn deserialize_optional_i64_or_string<'de, D>(deserializer: D) -> std::result::Result<Option<i64>, D::Error>
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

/// Organization member/user (wrapper for API response)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// User details from external field
    pub external: UserExternal,
}

/// User details from the external field in API response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserExternal {
    /// User ID
    pub id: String,

    /// User email address
    pub email: String,

    /// User's first name (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,

    /// User's last name (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// User's full name (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
}

/// Organization team
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    /// Team ID
    pub id: String,

    /// Team name
    pub name: String,

    /// Organization ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
}

/// Policy type (StackHawk preset or Organization custom)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyType {
    /// Preset policy created by StackHawk (read-only)
    StackHawk,
    /// Custom policy for an organization (editable)
    Organization,
}

impl std::fmt::Display for PolicyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyType::StackHawk => write!(f, "StackHawk"),
            PolicyType::Organization => write!(f, "Organization"),
        }
    }
}

/// StackHawk scan policy (preset, read-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackHawkPolicy {
    /// Policy ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Policy name (unique identifier)
    pub name: String,

    /// Human-readable display name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Policy description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Organization scan policy (custom, editable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgPolicy {
    /// Policy name (unique identifier)
    pub name: String,

    /// Human-readable display name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Policy description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Organization ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
}

/// Repository from attack surface mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    /// Repository ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Git provider (GITHUB, AZURE_DEVOPS, BITBUCKET, GITLAB)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_source: Option<String>,

    /// Provider organization name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_org_name: Option<String>,

    /// Repository name
    #[serde(default)]
    pub name: String,

    /// OpenAPI spec information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_api_spec_info: Option<OpenApiSpecInfo>,

    /// Whether StackHawk has generated an OpenAPI spec
    #[serde(default)]
    pub has_generated_open_api_spec: bool,

    /// Whether this repo is in the attack surface
    #[serde(default)]
    pub is_in_attack_surface: bool,

    /// Detected framework names
    #[serde(default)]
    pub framework_names: Vec<String>,

    /// Sensitive data tags detected
    #[serde(default)]
    pub sensitive_data_tags: Vec<SensitiveDataTag>,

    /// Timestamp of last commit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_commit_timestamp: Option<String>,

    /// Last contributor information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_contributor: Option<RepoContributor>,

    /// Commit count (30-day activity)
    #[serde(default)]
    pub commit_count: u32,

    /// Mapped application info
    #[serde(default)]
    pub app_infos: Vec<RepoAppInfo>,

    /// API Discovery insights
    #[serde(default)]
    pub insights: Vec<RepoInsight>,
}

/// OpenAPI specification info for repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenApiSpecInfo {
    /// Number of generated OAS files
    #[serde(default)]
    pub generated_oas_count: u32,
}

/// Sensitive data tag detected in repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensitiveDataTag {
    /// Tag name (e.g., PII, PCI, PHI)
    #[serde(default)]
    pub name: String,
}

/// Repository contributor information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoContributor {
    /// Contributor name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Contributor email
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Application info linked to repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoAppInfo {
    /// Application ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,

    /// Application name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
}

/// API Discovery insight for repository
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInsight {
    /// Insight name (e.g., "apiStyle")
    #[serde(default)]
    pub name: String,

    /// Insight value
    #[serde(default)]
    pub value: String,
}

/// Hosted OpenAPI specification asset
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OASAsset {
    /// Unique OAS ID
    #[serde(default)]
    pub oas_id: String,

    /// Repository ID this OAS belongs to
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_id: Option<String>,

    /// Repository name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_name: Option<String>,

    /// Source root path in repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_root_path: Option<String>,
}

/// Organization scan configuration
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

/// User secret (name only - values are not returned by list API)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    /// Secret name
    #[serde(default)]
    pub name: String,
}

/// Audit log record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecord {
    /// Unique audit record ID
    #[serde(default)]
    pub id: String,

    /// User activity type (e.g., SCAN_STARTED, APPLICATION_ADDED)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_activity_type: Option<String>,

    /// Organization activity type (e.g., ORGANIZATION_CREATED)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_activity_type: Option<String>,

    /// Organization ID
    #[serde(default)]
    pub organization_id: String,

    /// User ID who performed the action
    #[serde(default)]
    pub user_id: String,

    /// User name
    #[serde(default)]
    pub user_name: String,

    /// User email
    #[serde(default)]
    pub user_email: String,

    /// Payload containing action-specific details (JSON string)
    #[serde(default)]
    pub payload: String,

    /// Timestamp in milliseconds (as string from API)
    #[serde(default)]
    pub timestamp: String,

    /// User IP address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_ip_addr: Option<String>,
}

/// Filter parameters for audit log queries
#[derive(Debug, Clone, Default)]
pub struct AuditFilterParams {
    /// Filter by user activity types
    pub types: Vec<String>,
    /// Filter by organization activity types
    pub org_types: Vec<String>,
    /// Filter by user name
    pub name: Option<String>,
    /// Filter by user email
    pub email: Option<String>,
    /// Start timestamp (milliseconds)
    pub start: Option<i64>,
    /// End timestamp (milliseconds)
    pub end: Option<i64>,
    /// Sort direction (asc/desc)
    pub sort_dir: Option<String>,
    /// Page size (max 1000)
    pub page_size: Option<usize>,
    /// Page token for pagination
    pub page_token: Option<String>,
}

impl AuditFilterParams {
    /// Create new empty filter params
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert to query parameters for the API
    pub fn to_query_params(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();

        // Add user activity types (comma-separated)
        if !self.types.is_empty() {
            params.push(("types", self.types.join(",")));
        }

        // Add org activity types (comma-separated)
        if !self.org_types.is_empty() {
            params.push(("orgTypes", self.org_types.join(",")));
        }

        if let Some(ref name) = self.name {
            params.push(("name", name.clone()));
        }

        if let Some(ref email) = self.email {
            params.push(("email", email.clone()));
        }

        if let Some(start) = self.start {
            params.push(("start", start.to_string()));
        }

        if let Some(end) = self.end {
            params.push(("end", end.to_string()));
        }

        // Only supported sort field is "createdDate"
        params.push(("sortField", "createdDate".to_string()));

        if let Some(ref dir) = self.sort_dir {
            params.push(("sortDir", dir.clone()));
        }

        if let Some(size) = self.page_size {
            params.push(("pageSize", size.to_string()));
        }

        if let Some(ref token) = self.page_token {
            params.push(("pageToken", token.clone()));
        }

        params
    }
}
