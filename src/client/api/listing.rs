//! Listing API trait for collection operations

use async_trait::async_trait;

use crate::client::models::{
    Application, AuditFilterParams, AuditRecord, OASAsset, OrgPolicy, Organization, Repository,
    ScanConfig, ScanResult, Secret, StackHawkPolicy, Team, User,
};
use crate::client::pagination::{PagedResponse, PaginationParams, ScanFilterParams};
use crate::error::Result;

/// Collection listing operations for the StackHawk API
///
/// This trait covers all `list_*` operations that return collections of resources.
/// Methods support optional pagination and filtering where applicable.
#[async_trait]
pub trait ListingApi: Send + Sync {
    // ========================================================================
    // Organizations
    // ========================================================================

    /// List all accessible organizations
    async fn list_orgs(&self) -> Result<Vec<Organization>>;

    /// List organization custom policies with optional pagination
    async fn list_org_policies(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<OrgPolicy>>;

    // ========================================================================
    // Applications
    // ========================================================================

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

    // ========================================================================
    // Scans
    // ========================================================================

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

    // ========================================================================
    // Users & Teams
    // ========================================================================

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

    // ========================================================================
    // Policies
    // ========================================================================

    /// List all StackHawk preset policies (read-only)
    async fn list_stackhawk_policies(&self) -> Result<Vec<StackHawkPolicy>>;

    // ========================================================================
    // Infrastructure & Assets
    // ========================================================================

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

    // ========================================================================
    // Secrets & Audit
    // ========================================================================

    /// List user secrets (user-scoped, not org-scoped)
    async fn list_secrets(&self) -> Result<Vec<Secret>>;

    /// List audit log records for an organization
    async fn list_audit(
        &self,
        org_id: &str,
        filters: Option<&AuditFilterParams>,
    ) -> Result<Vec<AuditRecord>>;
}
