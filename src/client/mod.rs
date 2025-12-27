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
}

/// Scan result from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scan {
    /// Scan ID
    #[serde(rename = "id")]
    pub id: String,

    /// Application ID
    pub application_id: String,

    /// Application name
    pub application_name: String,

    /// Environment name
    pub env: String,

    /// Scan status (STARTED, COMPLETED, ERROR, UNKNOWN)
    pub status: String,

    /// Timestamp when scan started (Unix epoch milliseconds as string)
    pub timestamp: String,
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
    pub alert_status: String,

    /// Total count for this status
    #[serde(default)]
    pub total_count: u32,

    /// Breakdown by severity
    #[serde(default)]
    pub severity_stats: std::collections::HashMap<String, u32>,
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
