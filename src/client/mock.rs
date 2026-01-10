//! Mock StackHawk API client for testing
//!
//! Provides a mock implementation of the API traits for unit testing
//! without making real API calls.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::api::{AuthApi, ListingApi, ScanDetailApi};
use super::models::{
    AlertMsgResponse, AlertResponse, Application, ApplicationAlert, AuditFilterParams, AuditRecord,
    JwtToken, OASAsset, OrgPolicy, Organization, Repository, ScanConfig, ScanMessage, ScanResult,
    Secret, StackHawkPolicy, Team, User,
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
}

impl Default for MockStackHawkClient {
    fn default() -> Self {
        Self {
            orgs: Arc::new(Mutex::new(Vec::new())),
            apps: Arc::new(Mutex::new(Vec::new())),
            scans: Arc::new(Mutex::new(Vec::new())),
            users: Arc::new(Mutex::new(Vec::new())),
            teams: Arc::new(Mutex::new(Vec::new())),
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

    /// Check if there's a pending error and consume it.
    async fn check_error(&self) -> Result<()> {
        let mut error = self.error.lock().await;
        if let Some(e) = error.take() {
            return Err(e.into());
        }
        Ok(())
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
        _org_id: &str,
        _pagination: Option<&PaginationParams>,
    ) -> Result<Vec<Application>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_apps += 1;

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
        _org_id: &str,
        pagination: Option<&PaginationParams>,
    ) -> Result<PagedResponse<Application>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_apps += 1;

        let apps = self.apps.lock().await.clone();
        let total_count = apps.len();
        let page_size = pagination.and_then(|p| p.page_size).unwrap_or(100);
        let page_token = pagination.and_then(|p| p.page).unwrap_or(0);

        Ok(PagedResponse::new(
            apps,
            Some(total_count),
            page_size,
            page_token,
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
}
