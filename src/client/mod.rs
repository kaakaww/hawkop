//! StackHawk API client

use async_trait::async_trait;

use crate::error::Result;

#[cfg(test)]
pub mod mock;
pub mod models;
pub mod pagination;
pub mod parallel;
pub mod rate_limit;
pub mod stackhawk;

// Import model types used in the StackHawkApi trait
use models::{
    AlertMsgResponse, AlertResponse, Application, ApplicationAlert, AuditFilterParams, AuditRecord,
    JwtToken, OASAsset, OrgPolicy, Organization, Repository, ScanConfig, ScanResult, Secret,
    StackHawkPolicy, Team, User,
};

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
