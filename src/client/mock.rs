//! Mock StackHawk API client for testing
//!
//! Provides a mock implementation of the API traits for unit testing
//! without making real API calls.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::api::{AuthApi, ListingApi, ScanDetailApi, TeamApi};
use super::models::{
    AlertMsgResponse, AlertResponse, Application, ApplicationAlert, AuditFilterParams, AuditRecord,
    CreateTeamRequest, JwtToken, OASAsset, OrgPolicy, Organization, Repository, ScanConfig,
    ScanMessage, ScanResult, Secret, StackHawkPolicy, Team, TeamApplication, TeamDetail, TeamUser,
    UpdateApplicationTeamRequest, UpdateTeamRequest, User,
};
use super::pagination::{PagedResponse, PaginationParams, ScanFilterParams};
use crate::error::{ApiError, Result};

/// Mock API client for testing.
///
/// Configure expected responses via builder methods, then use in tests.
///
/// # Example
/// ```ignore
/// let mock = MockStackHawkClient::new()
///     .with_orgs(vec![Organization { id: "test".into(), name: "Test Org".into(), ... }]);
///
/// let orgs = mock.list_orgs().await?;
/// assert_eq!(orgs.len(), 1);
/// ```
pub struct MockStackHawkClient {
    /// Organizations to return from list_orgs
    orgs: Arc<Mutex<Vec<Organization>>>,
    /// Applications to return from list_apps
    apps: Arc<Mutex<Vec<Application>>>,
    /// Scans to return from list_scans
    scans: Arc<Mutex<Vec<ScanResult>>>,
    /// Users to return from list_users
    users: Arc<Mutex<Vec<User>>>,
    /// Teams to return from list_teams
    teams: Arc<Mutex<Vec<Team>>>,
    /// Team details for get_team/CRUD operations
    team_details: Arc<Mutex<Vec<TeamDetail>>>,
    /// StackHawk policies to return from list_stackhawk_policies
    stackhawk_policies: Arc<Mutex<Vec<StackHawkPolicy>>>,
    /// Org policies to return from list_org_policies
    org_policies: Arc<Mutex<Vec<OrgPolicy>>>,
    /// Repositories to return from list_repos
    repos: Arc<Mutex<Vec<Repository>>>,
    /// OAS assets to return from list_oas
    oas_assets: Arc<Mutex<Vec<OASAsset>>>,
    /// Scan configurations to return from list_scan_configs
    scan_configs: Arc<Mutex<Vec<ScanConfig>>>,
    /// Secrets to return from list_secrets
    secrets: Arc<Mutex<Vec<Secret>>>,
    /// Audit records to return from list_audit
    audit_records: Arc<Mutex<Vec<AuditRecord>>>,
    /// JWT to return from authenticate
    jwt: Arc<Mutex<Option<JwtToken>>>,
    /// Error to return (if any) - consumed on first use
    error: Arc<Mutex<Option<ApiError>>>,
    /// Track number of calls for verification
    call_count: Arc<Mutex<CallCounts>>,
    /// Captured requests for test assertions
    captured_requests: Arc<Mutex<Vec<CapturedRequest>>>,
    /// Rate limit after N total calls (simulates 429 response)
    rate_limit_after: Arc<Mutex<Option<usize>>>,
    /// Paginated app responses (page index -> apps for that page)
    app_pages: Arc<Mutex<Option<Vec<Vec<Application>>>>>,
}

impl Default for MockStackHawkClient {
    fn default() -> Self {
        Self {
            orgs: Arc::new(Mutex::new(Vec::new())),
            apps: Arc::new(Mutex::new(Vec::new())),
            scans: Arc::new(Mutex::new(Vec::new())),
            users: Arc::new(Mutex::new(Vec::new())),
            teams: Arc::new(Mutex::new(Vec::new())),
            team_details: Arc::new(Mutex::new(Vec::new())),
            stackhawk_policies: Arc::new(Mutex::new(Vec::new())),
            org_policies: Arc::new(Mutex::new(Vec::new())),
            repos: Arc::new(Mutex::new(Vec::new())),
            oas_assets: Arc::new(Mutex::new(Vec::new())),
            scan_configs: Arc::new(Mutex::new(Vec::new())),
            secrets: Arc::new(Mutex::new(Vec::new())),
            audit_records: Arc::new(Mutex::new(Vec::new())),
            jwt: Arc::new(Mutex::new(None)),
            error: Arc::new(Mutex::new(None)),
            call_count: Arc::new(Mutex::new(CallCounts::default())),
            captured_requests: Arc::new(Mutex::new(Vec::new())),
            rate_limit_after: Arc::new(Mutex::new(None)),
            app_pages: Arc::new(Mutex::new(None)),
        }
    }
}

/// Tracks API call counts for test verification
#[derive(Default, Debug, Clone)]
pub struct CallCounts {
    pub authenticate: usize,
    pub list_orgs: usize,
    pub list_apps: usize,
    pub list_scans: usize,
    pub list_users: usize,
    pub list_teams: usize,
    pub list_stackhawk_policies: usize,
    pub list_org_policies: usize,
    pub list_repos: usize,
    pub list_oas: usize,
    pub list_scan_configs: usize,
    pub list_secrets: usize,
    pub list_audit: usize,
    // Team CRUD operations
    pub get_team: usize,
    pub create_team: usize,
    pub update_team: usize,
    pub delete_team: usize,
}

impl CallCounts {
    /// Get total number of API calls made.
    pub fn total(&self) -> usize {
        self.authenticate
            + self.list_orgs
            + self.list_apps
            + self.list_scans
            + self.list_users
            + self.list_teams
            + self.list_stackhawk_policies
            + self.list_org_policies
            + self.list_repos
            + self.list_oas
            + self.list_scan_configs
            + self.list_secrets
            + self.list_audit
            + self.get_team
            + self.create_team
            + self.update_team
            + self.delete_team
    }
}

/// A captured API request for test assertions.
#[derive(Debug, Clone)]
pub struct CapturedRequest {
    /// The API method called (e.g., "list_apps", "list_scans")
    pub method: String,
    /// Organization ID if provided
    pub org_id: Option<String>,
    /// Page number if pagination was requested
    pub page: Option<usize>,
    /// Page size if pagination was requested
    pub page_size: Option<usize>,
}

impl MockStackHawkClient {
    /// Create a new mock client with default (empty) responses.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure organizations to return from list_orgs.
    pub async fn with_orgs(self, orgs: Vec<Organization>) -> Self {
        *self.orgs.lock().await = orgs;
        self
    }

    /// Configure applications to return from list_apps.
    pub async fn with_apps(self, apps: Vec<Application>) -> Self {
        *self.apps.lock().await = apps;
        self
    }

    /// Configure scans to return from list_scans.
    #[allow(dead_code)]
    pub async fn with_scans(self, scans: Vec<ScanResult>) -> Self {
        *self.scans.lock().await = scans;
        self
    }

    /// Configure users to return from list_users.
    #[allow(dead_code)]
    pub async fn with_users(self, users: Vec<User>) -> Self {
        *self.users.lock().await = users;
        self
    }

    /// Configure teams to return from list_teams.
    #[allow(dead_code)]
    pub async fn with_teams(self, teams: Vec<Team>) -> Self {
        *self.teams.lock().await = teams;
        self
    }

    /// Configure team details for get_team and CRUD operations.
    #[allow(dead_code)]
    pub async fn with_team_details(self, details: Vec<TeamDetail>) -> Self {
        *self.team_details.lock().await = details;
        self
    }

    /// Configure JWT token to return from authenticate.
    pub async fn with_jwt(self, jwt: JwtToken) -> Self {
        *self.jwt.lock().await = Some(jwt);
        self
    }

    /// Configure an error to return on the next API call.
    /// The error is consumed after one use.
    pub async fn with_error(self, error: ApiError) -> Self {
        *self.error.lock().await = Some(error);
        self
    }

    /// Get the call counts for verification in tests.
    pub async fn call_counts(&self) -> CallCounts {
        self.call_count.lock().await.clone()
    }

    /// Get all captured requests for test assertions.
    #[allow(dead_code)]
    pub async fn captured_requests(&self) -> Vec<CapturedRequest> {
        self.captured_requests.lock().await.clone()
    }

    /// Configure rate limiting to trigger after N total API calls.
    /// After the threshold is reached, all subsequent calls return RateLimited error.
    #[allow(dead_code)]
    pub async fn rate_limit_after(self, calls: usize) -> Self {
        *self.rate_limit_after.lock().await = Some(calls);
        self
    }

    /// Configure paginated app responses by page.
    /// Page 0 returns pages[0], page 1 returns pages[1], etc.
    #[allow(dead_code)]
    pub async fn with_app_pages(self, pages: Vec<Vec<Application>>) -> Self {
        *self.app_pages.lock().await = Some(pages);
        self
    }

    /// Check if there's a pending error and consume it.
    /// Also checks rate limit threshold.
    async fn check_error(&self) -> Result<()> {
        // Check one-shot error first
        {
            let mut error = self.error.lock().await;
            if let Some(e) = error.take() {
                return Err(e.into());
            }
        }

        // Check rate limit threshold
        {
            let rate_limit = self.rate_limit_after.lock().await;
            if let Some(threshold) = *rate_limit {
                let counts = self.call_count.lock().await;
                if counts.total() >= threshold {
                    return Err(ApiError::RateLimited.into());
                }
            }
        }

        Ok(())
    }

    /// Record a captured request for test assertions.
    async fn capture_request(
        &self,
        method: &str,
        org_id: Option<&str>,
        pagination: Option<&PaginationParams>,
    ) {
        let mut requests = self.captured_requests.lock().await;
        requests.push(CapturedRequest {
            method: method.to_string(),
            org_id: org_id.map(|s| s.to_string()),
            page: pagination.and_then(|p| p.page),
            page_size: pagination.and_then(|p| p.page_size),
        });
    }
}

// ============================================================================
// AuthApi Implementation
// ============================================================================

#[async_trait]
impl AuthApi for MockStackHawkClient {
    async fn authenticate(&self, _api_key: &str) -> Result<JwtToken> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.authenticate += 1;

        let jwt = self.jwt.lock().await;
        Ok(jwt.clone().unwrap_or_else(|| JwtToken {
            token: "mock-jwt-token".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        }))
    }
}

// ============================================================================
// ListingApi Implementation
// ============================================================================

#[async_trait]
impl ListingApi for MockStackHawkClient {
    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_orgs += 1;

        Ok(self.orgs.lock().await.clone())
    }

    async fn list_apps(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Application>> {
        self.capture_request("list_apps", Some(org_id), pagination)
            .await;
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_apps += 1;
        drop(counts); // Release lock before acquiring app_pages lock

        // Check for paginated responses first
        let app_pages = self.app_pages.lock().await;
        if let Some(ref pages) = *app_pages {
            let page_idx = pagination.and_then(|p| p.page).unwrap_or(0);
            return Ok(pages.get(page_idx).cloned().unwrap_or_default());
        }
        drop(app_pages);

        Ok(self.apps.lock().await.clone())
    }

    async fn list_scans(
        &self,
        _org_id: &str,
        _pagination: Option<&PaginationParams>,
        _filters: Option<&ScanFilterParams>,
    ) -> Result<Vec<ScanResult>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_scans += 1;

        Ok(self.scans.lock().await.clone())
    }

    async fn list_apps_paged(
        &self,
        org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<PagedResponse<Application>> {
        self.capture_request("list_apps_paged", Some(org_id), pagination)
            .await;
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_apps += 1;
        drop(counts);

        let page_size = pagination.and_then(|p| p.page_size).unwrap_or(100);
        let page_idx = pagination.and_then(|p| p.page).unwrap_or(0);

        // Check for paginated responses first
        let app_pages = self.app_pages.lock().await;
        if let Some(ref pages) = *app_pages {
            let total_count: usize = pages.iter().map(|p| p.len()).sum();
            let apps = pages.get(page_idx).cloned().unwrap_or_default();
            return Ok(PagedResponse::new(
                apps,
                Some(total_count),
                page_size,
                page_idx,
            ));
        }
        drop(app_pages);

        let apps = self.apps.lock().await.clone();
        let total_count = apps.len();

        Ok(PagedResponse::new(
            apps,
            Some(total_count),
            page_size,
            page_idx,
        ))
    }

    async fn list_scans_paged(
        &self,
        _org_id: &str,
        pagination: Option<&PaginationParams>,
        _filters: Option<&ScanFilterParams>,
    ) -> Result<PagedResponse<ScanResult>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_scans += 1;

        let scans = self.scans.lock().await.clone();
        let total_count = scans.len();
        let page_size = pagination.and_then(|p| p.page_size).unwrap_or(100);
        let page_token = pagination.and_then(|p| p.page).unwrap_or(0);

        Ok(PagedResponse::new(
            scans,
            Some(total_count),
            page_size,
            page_token,
        ))
    }

    async fn list_users(
        &self,
        _org_id: &str,
        _pagination: Option<&PaginationParams>,
    ) -> Result<Vec<User>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_users += 1;

        Ok(self.users.lock().await.clone())
    }

    async fn list_teams(
        &self,
        _org_id: &str,
        _pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Team>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_teams += 1;

        Ok(self.teams.lock().await.clone())
    }

    async fn list_users_paged(
        &self,
        _org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<PagedResponse<User>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_users += 1;

        let users = self.users.lock().await.clone();
        let total_count = users.len();
        let page_size = pagination.and_then(|p| p.page_size).unwrap_or(100);
        let page_idx = pagination.and_then(|p| p.page).unwrap_or(0);

        Ok(PagedResponse::new(
            users,
            Some(total_count),
            page_size,
            page_idx,
        ))
    }

    async fn list_teams_paged(
        &self,
        _org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<PagedResponse<Team>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_teams += 1;

        let teams = self.teams.lock().await.clone();
        let total_count = teams.len();
        let page_size = pagination.and_then(|p| p.page_size).unwrap_or(100);
        let page_idx = pagination.and_then(|p| p.page).unwrap_or(0);

        Ok(PagedResponse::new(
            teams,
            Some(total_count),
            page_size,
            page_idx,
        ))
    }

    async fn list_stackhawk_policies(&self) -> Result<Vec<StackHawkPolicy>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_stackhawk_policies += 1;

        Ok(self.stackhawk_policies.lock().await.clone())
    }

    async fn list_org_policies(
        &self,
        _org_id: &str,
        _pagination: Option<&PaginationParams>,
    ) -> Result<Vec<OrgPolicy>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_org_policies += 1;

        Ok(self.org_policies.lock().await.clone())
    }

    async fn list_repos(
        &self,
        _org_id: &str,
        _pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Repository>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_repos += 1;

        Ok(self.repos.lock().await.clone())
    }

    async fn list_oas(
        &self,
        _org_id: &str,
        _pagination: Option<&PaginationParams>,
    ) -> Result<Vec<OASAsset>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_oas += 1;

        Ok(self.oas_assets.lock().await.clone())
    }

    async fn list_scan_configs(
        &self,
        _org_id: &str,
        _pagination: Option<&PaginationParams>,
    ) -> Result<Vec<ScanConfig>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_scan_configs += 1;

        Ok(self.scan_configs.lock().await.clone())
    }

    async fn list_secrets(&self) -> Result<Vec<Secret>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_secrets += 1;

        Ok(self.secrets.lock().await.clone())
    }

    async fn list_audit(
        &self,
        _org_id: &str,
        _filters: Option<&AuditFilterParams>,
    ) -> Result<Vec<AuditRecord>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_audit += 1;

        Ok(self.audit_records.lock().await.clone())
    }
}

// ============================================================================
// ScanDetailApi Implementation
// ============================================================================

#[async_trait]
impl ScanDetailApi for MockStackHawkClient {
    async fn get_scan(&self, _org_id: &str, _scan_id: &str) -> Result<ScanResult> {
        self.check_error().await?;
        // Return first scan or error if none
        let scans = self.scans.lock().await;
        scans
            .first()
            .cloned()
            .ok_or_else(|| ApiError::NotFound("Scan not found".to_string()).into())
    }

    async fn list_scan_alerts(
        &self,
        _scan_id: &str,
        _pagination: Option<&PaginationParams>,
    ) -> Result<Vec<ApplicationAlert>> {
        self.check_error().await?;
        Ok(vec![])
    }

    async fn get_alert_with_paths(
        &self,
        _scan_id: &str,
        _plugin_id: &str,
        _pagination: Option<&PaginationParams>,
    ) -> Result<AlertResponse> {
        self.check_error().await?;
        Ok(AlertResponse {
            alert: ApplicationAlert {
                plugin_id: "test".to_string(),
                name: "Test Alert".to_string(),
                description: "Test description".to_string(),
                severity: "Medium".to_string(),
                cwe_id: None,
                references: vec![],
                uri_count: 0,
                alert_status_stats: vec![],
            },
            application_scan_alert_uris: vec![],
            app_host: None,
            category: None,
            cheatsheet: None,
            next_page_token: None,
            total_count: Some(0),
        })
    }

    async fn get_alert_message(
        &self,
        _scan_id: &str,
        _alert_uri_id: &str,
        _message_id: &str,
        _include_curl: bool,
    ) -> Result<AlertMsgResponse> {
        self.check_error().await?;
        Ok(AlertMsgResponse {
            scan_message: ScanMessage {
                id: "msg-1".to_string(),
                request_header: Some("GET / HTTP/1.1".to_string()),
                request_body: None,
                response_header: Some("HTTP/1.1 200 OK".to_string()),
                response_body: Some("<html></html>".to_string()),
                cookie_params: None,
            },
            uri: "/test".to_string(),
            evidence: None,
            param: None,
            other_info: None,
            description: None,
            validation_command: None,
        })
    }
}

// ============================================================================
// TeamApi Implementation
// ============================================================================

#[async_trait]
impl TeamApi for MockStackHawkClient {
    async fn get_team(&self, _org_id: &str, team_id: &str) -> Result<TeamDetail> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.get_team += 1;
        drop(counts);

        let details = self.team_details.lock().await;
        details
            .iter()
            .find(|t| t.id == team_id)
            .cloned()
            .ok_or_else(|| ApiError::NotFound(format!("Team not found: {}", team_id)).into())
    }

    async fn get_team_fresh(&self, org_id: &str, team_id: &str) -> Result<TeamDetail> {
        // For mock, fresh and cached are the same
        self.get_team(org_id, team_id).await
    }

    async fn create_team(&self, _org_id: &str, request: CreateTeamRequest) -> Result<TeamDetail> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.create_team += 1;
        drop(counts);

        // Create a new team detail from the request
        // Use a simple incrementing ID for mock purposes
        let existing_count = self.team_details.lock().await.len();
        let new_team = TeamDetail {
            id: format!("mock-team-{}", existing_count + 1),
            name: request.name,
            organization_id: Some(request.organization_id),
            users: vec![],
            applications: vec![],
        };

        // Add to storage
        let mut details = self.team_details.lock().await;
        details.push(new_team.clone());

        Ok(new_team)
    }

    async fn update_team(
        &self,
        _org_id: &str,
        team_id: &str,
        request: UpdateTeamRequest,
    ) -> Result<TeamDetail> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.update_team += 1;
        drop(counts);

        let mut details = self.team_details.lock().await;
        let team = details
            .iter_mut()
            .find(|t| t.id == team_id)
            .ok_or_else(|| ApiError::NotFound(format!("Team not found: {}", team_id)))?;

        // Update the name if provided
        if let Some(name) = request.name {
            team.name = name;
        }

        // Update users if provided
        if let Some(user_ids) = request.user_ids {
            team.users = user_ids
                .into_iter()
                .map(|id| TeamUser {
                    user_id: id.clone(),
                    user_name: Some(format!("User {}", id)),
                    email: Some(format!("user-{}@example.com", id)),
                    role: Some("Member".to_string()),
                })
                .collect();
        }

        // Update applications if provided
        if let Some(app_ids) = request.application_ids {
            team.applications = app_ids
                .into_iter()
                .map(|id| TeamApplication {
                    application_id: id.clone(),
                    application_name: Some(format!("App {}", id)),
                    environments: vec!["Development".to_string()],
                })
                .collect();
        }

        Ok(team.clone())
    }

    async fn delete_team(&self, _org_id: &str, team_id: &str) -> Result<()> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.delete_team += 1;
        drop(counts);

        let mut details = self.team_details.lock().await;
        let initial_len = details.len();
        details.retain(|t| t.id != team_id);

        if details.len() == initial_len {
            return Err(ApiError::NotFound(format!("Team not found: {}", team_id)).into());
        }

        Ok(())
    }

    async fn assign_app_to_team(
        &self,
        _org_id: &str,
        _team_id: &str,
        _request: UpdateApplicationTeamRequest,
    ) -> Result<()> {
        self.check_error().await?;
        // For mock, just succeed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_client_default_empty() {
        let mock = MockStackHawkClient::new();

        let orgs = mock.list_orgs().await.unwrap();
        assert!(orgs.is_empty());

        let apps = mock.list_apps("test-org", None).await.unwrap();
        assert!(apps.is_empty());
    }

    #[tokio::test]
    async fn test_mock_client_with_orgs() {
        let mock = MockStackHawkClient::new()
            .with_orgs(vec![
                Organization {
                    id: "org-1".to_string(),
                    name: "Test Org 1".to_string(),
                    user_count: Some(5),
                    app_count: Some(3),
                },
                Organization {
                    id: "org-2".to_string(),
                    name: "Test Org 2".to_string(),
                    user_count: None,
                    app_count: None,
                },
            ])
            .await;

        let orgs = mock.list_orgs().await.unwrap();
        assert_eq!(orgs.len(), 2);
        assert_eq!(orgs[0].id, "org-1");
        assert_eq!(orgs[1].name, "Test Org 2");
    }

    #[tokio::test]
    async fn test_mock_client_with_apps() {
        let mock = MockStackHawkClient::new()
            .with_apps(vec![Application {
                id: "app-1".to_string(),
                name: "Test App".to_string(),
                env: Some("production".to_string()),
                risk_level: None,
                status: None,
                organization_id: Some("org-1".to_string()),
                application_type: None,
                cloud_scan_target: None,
            }])
            .await;

        let apps = mock.list_apps("org-1", None).await.unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].id, "app-1");
    }

    #[tokio::test]
    async fn test_mock_client_with_error() {
        let mock = MockStackHawkClient::new()
            .with_error(ApiError::Unauthorized)
            .await;

        let result = mock.list_orgs().await;
        assert!(result.is_err());

        // Error is consumed, next call succeeds
        let result = mock.list_orgs().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_client_call_counts() {
        let mock = MockStackHawkClient::new();

        mock.list_orgs().await.unwrap();
        mock.list_orgs().await.unwrap();
        mock.list_apps("org", None).await.unwrap();

        let counts = mock.call_counts().await;
        assert_eq!(counts.list_orgs, 2);
        assert_eq!(counts.list_apps, 1);
        assert_eq!(counts.authenticate, 0);
    }

    #[tokio::test]
    async fn test_mock_client_authenticate() {
        let jwt = JwtToken {
            token: "custom-token".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(2),
        };

        let mock = MockStackHawkClient::new().with_jwt(jwt.clone()).await;

        let result = mock.authenticate("api-key").await.unwrap();
        assert_eq!(result.token, "custom-token");
    }

    // ========================================================================
    // Rate limiting tests
    // ========================================================================

    #[tokio::test]
    async fn test_mock_client_rate_limit_after() {
        let mock = MockStackHawkClient::new().rate_limit_after(3).await;

        // First 3 calls succeed
        assert!(mock.list_orgs().await.is_ok());
        assert!(mock.list_orgs().await.is_ok());
        assert!(mock.list_orgs().await.is_ok());

        // 4th call fails with rate limit error
        let result = mock.list_orgs().await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Rate limited"));
    }

    #[tokio::test]
    async fn test_mock_client_rate_limit_across_methods() {
        let mock = MockStackHawkClient::new().rate_limit_after(2).await;

        // Mix of different methods counts toward limit
        assert!(mock.list_orgs().await.is_ok());
        assert!(mock.list_apps("org", None).await.is_ok());

        // 3rd call (any method) hits limit
        assert!(mock.list_orgs().await.is_err());
    }

    // ========================================================================
    // Captured requests tests
    // ========================================================================

    #[tokio::test]
    async fn test_mock_client_captured_requests() {
        let mock = MockStackHawkClient::new();

        mock.list_apps("org-123", None).await.unwrap();

        let captured = mock.captured_requests().await;
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].method, "list_apps");
        assert_eq!(captured[0].org_id, Some("org-123".to_string()));
        assert!(captured[0].page.is_none());
    }

    #[tokio::test]
    async fn test_mock_client_captured_requests_with_pagination() {
        let mock = MockStackHawkClient::new();

        let params = PaginationParams::new().page(2).page_size(50);
        mock.list_apps("org-456", Some(&params)).await.unwrap();

        let captured = mock.captured_requests().await;
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].method, "list_apps");
        assert_eq!(captured[0].org_id, Some("org-456".to_string()));
        assert_eq!(captured[0].page, Some(2));
        assert_eq!(captured[0].page_size, Some(50));
    }

    // ========================================================================
    // Paginated responses tests
    // ========================================================================

    #[tokio::test]
    async fn test_mock_client_with_app_pages() {
        let page0 = vec![Application {
            id: "app-1".to_string(),
            name: "App 1".to_string(),
            env: None,
            risk_level: None,
            status: None,
            organization_id: None,
            application_type: None,
            cloud_scan_target: None,
        }];
        let page1 = vec![Application {
            id: "app-2".to_string(),
            name: "App 2".to_string(),
            env: None,
            risk_level: None,
            status: None,
            organization_id: None,
            application_type: None,
            cloud_scan_target: None,
        }];

        let mock = MockStackHawkClient::new()
            .with_app_pages(vec![page0, page1])
            .await;

        // Page 0 returns app-1
        let apps_p0 = mock.list_apps("org", None).await.unwrap();
        assert_eq!(apps_p0.len(), 1);
        assert_eq!(apps_p0[0].id, "app-1");

        // Page 1 returns app-2
        let params = PaginationParams::new().page(1);
        let apps_p1 = mock.list_apps("org", Some(&params)).await.unwrap();
        assert_eq!(apps_p1.len(), 1);
        assert_eq!(apps_p1[0].id, "app-2");

        // Page 2 returns empty (out of range)
        let params = PaginationParams::new().page(2);
        let apps_p2 = mock.list_apps("org", Some(&params)).await.unwrap();
        assert!(apps_p2.is_empty());
    }

    #[tokio::test]
    async fn test_mock_client_with_app_pages_paged_response() {
        let page0 = vec![Application {
            id: "app-1".to_string(),
            name: "App 1".to_string(),
            env: None,
            risk_level: None,
            status: None,
            organization_id: None,
            application_type: None,
            cloud_scan_target: None,
        }];
        let page1 = vec![
            Application {
                id: "app-2".to_string(),
                name: "App 2".to_string(),
                env: None,
                risk_level: None,
                status: None,
                organization_id: None,
                application_type: None,
                cloud_scan_target: None,
            },
            Application {
                id: "app-3".to_string(),
                name: "App 3".to_string(),
                env: None,
                risk_level: None,
                status: None,
                organization_id: None,
                application_type: None,
                cloud_scan_target: None,
            },
        ];

        let mock = MockStackHawkClient::new()
            .with_app_pages(vec![page0, page1])
            .await;

        // Paged response includes total count across all pages
        let response = mock.list_apps_paged("org", None).await.unwrap();
        assert_eq!(response.total_count, Some(3)); // 1 + 2 apps total
        assert_eq!(response.items.len(), 1); // page 0 has 1 app
    }

    // ========================================================================
    // TeamApi tests
    // ========================================================================

    #[tokio::test]
    async fn test_mock_team_get_returns_configured_team() {
        let team = TeamDetail {
            id: "team-123".to_string(),
            name: "Security Team".to_string(),
            organization_id: Some("org-1".to_string()),
            users: vec![],
            applications: vec![],
        };

        let mock = MockStackHawkClient::new()
            .with_team_details(vec![team])
            .await;

        let result = mock.get_team("org-1", "team-123").await.unwrap();
        assert_eq!(result.id, "team-123");
        assert_eq!(result.name, "Security Team");

        // Verify call count
        let counts = mock.call_counts().await;
        assert_eq!(counts.get_team, 1);
    }

    #[tokio::test]
    async fn test_mock_team_get_not_found() {
        let mock = MockStackHawkClient::new();

        let result = mock.get_team("org-1", "nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_mock_team_create() {
        let mock = MockStackHawkClient::new();

        let request = CreateTeamRequest {
            name: "New Team".to_string(),
            organization_id: "org-1".to_string(),
            user_ids: None,
            application_ids: None,
        };

        let result = mock.create_team("org-1", request).await.unwrap();
        assert_eq!(result.name, "New Team");
        assert!(result.id.starts_with("mock-team-"));

        // Verify call count
        let counts = mock.call_counts().await;
        assert_eq!(counts.create_team, 1);

        // Verify team was added to storage and can be retrieved
        let retrieved = mock.get_team("org-1", &result.id).await.unwrap();
        assert_eq!(retrieved.name, "New Team");
    }

    #[tokio::test]
    async fn test_mock_team_update() {
        let team = TeamDetail {
            id: "team-456".to_string(),
            name: "Old Name".to_string(),
            organization_id: Some("org-1".to_string()),
            users: vec![],
            applications: vec![],
        };

        let mock = MockStackHawkClient::new()
            .with_team_details(vec![team])
            .await;

        let request = UpdateTeamRequest {
            team_id: "team-456".to_string(),
            name: Some("New Name".to_string()),
            user_ids: None,
            application_ids: None,
        };

        let result = mock
            .update_team("org-1", "team-456", request)
            .await
            .unwrap();
        assert_eq!(result.name, "New Name");

        // Verify call count
        let counts = mock.call_counts().await;
        assert_eq!(counts.update_team, 1);
    }

    #[tokio::test]
    async fn test_mock_team_delete() {
        let team = TeamDetail {
            id: "team-789".to_string(),
            name: "To Delete".to_string(),
            organization_id: Some("org-1".to_string()),
            users: vec![],
            applications: vec![],
        };

        let mock = MockStackHawkClient::new()
            .with_team_details(vec![team])
            .await;

        // Delete should succeed
        mock.delete_team("org-1", "team-789").await.unwrap();

        // Verify call count
        let counts = mock.call_counts().await;
        assert_eq!(counts.delete_team, 1);

        // Team should no longer exist
        let result = mock.get_team("org-1", "team-789").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_team_delete_not_found() {
        let mock = MockStackHawkClient::new();

        let result = mock.delete_team("org-1", "nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_team_call_counts_in_total() {
        let team = TeamDetail {
            id: "team-1".to_string(),
            name: "Test".to_string(),
            organization_id: Some("org-1".to_string()),
            users: vec![],
            applications: vec![],
        };

        let mock = MockStackHawkClient::new()
            .with_team_details(vec![team])
            .await;

        // Make some team API calls
        mock.get_team("org-1", "team-1").await.unwrap();
        mock.get_team("org-1", "team-1").await.unwrap();

        let request = UpdateTeamRequest {
            team_id: "team-1".to_string(),
            name: Some("Updated".to_string()),
            user_ids: None,
            application_ids: None,
        };
        mock.update_team("org-1", "team-1", request).await.unwrap();

        // Verify total includes team calls
        let counts = mock.call_counts().await;
        assert_eq!(counts.get_team, 2);
        assert_eq!(counts.update_team, 1);
        assert_eq!(counts.total(), 3); // 2 gets + 1 update
    }
}
