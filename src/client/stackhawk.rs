//! StackHawk API client implementation

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Maximum number of retries for rate-limited requests
const MAX_RATE_LIMIT_RETRIES: u32 = 3;

use async_trait::async_trait;
use chrono::Utc;
use log::debug;
use reqwest::{Client as HttpClient, StatusCode};
use serde::Deserialize;
use tokio::sync::RwLock;

use serde::de::{self, Deserializer};

use super::api::{AuthApi, ListingApi, ScanDetailApi};
use super::models::{
    AlertMsgResponse, AlertResponse, Application, ApplicationAlert, AuditFilterParams, AuditRecord,
    JwtToken, OASAsset, OrgPolicy, Organization, Repository, ScanAlertsResponse, ScanConfig,
    ScanResult, Secret, StackHawkPolicy, Team, User,
};
use super::pagination::PagedResponse;
use super::rate_limit::{EndpointCategory, RateLimiterSet};
use crate::error::{ApiError, Result};

/// Deserialize a string to usize.
///
/// The StackHawk API inconsistently returns some numeric fields as JSON strings
/// (e.g., `"totalCount": "2666"` instead of `"totalCount": 2666`). This helper
/// handles both formats transparently using serde's `untagged` enum.
fn deserialize_string_to_usize<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(usize),
    }

    match Option::<StringOrNumber>::deserialize(deserializer)? {
        Some(StringOrNumber::String(s)) => s.parse::<usize>().map(Some).map_err(de::Error::custom),
        Some(StringOrNumber::Number(n)) => Ok(Some(n)),
        None => Ok(None),
    }
}

/// Decode base64url (URL-safe base64 without padding)
fn base64_decode_url(input: &str) -> std::result::Result<Vec<u8>, String> {
    use base64::{Engine as _, engine::general_purpose};

    // Base64url uses - instead of + and _ instead of /
    let standard_b64 = input.replace('-', "+").replace('_', "/");

    // Add padding if needed
    let padding = match standard_b64.len() % 4 {
        0 => "",
        2 => "==",
        3 => "=",
        _ => return Err("Invalid base64url length".to_string()),
    };

    let padded = format!("{}{}", standard_b64, padding);

    general_purpose::STANDARD
        .decode(&padded)
        .map_err(|e| e.to_string())
}

/// StackHawk API base URLs
const API_BASE_URL_V1: &str = "https://api.stackhawk.com/api/v1";
const API_BASE_URL_V2: &str = "https://api.stackhawk.com/api/v2";

/// StackHawk API client
pub struct StackHawkClient {
    http: HttpClient,
    base_url_v1: String,
    base_url_v2: String,
    /// Per-endpoint rate limiters (only active after 429 for each category)
    rate_limiters: Arc<RateLimiterSet>,
    auth_state: Arc<RwLock<AuthState>>,
}

/// Internal authentication state
#[derive(Debug, Clone)]
struct AuthState {
    api_key: Option<String>,
    jwt: Option<String>,
    jwt_expires_at: Option<chrono::DateTime<Utc>>,
}

impl StackHawkClient {
    /// Create a new StackHawk API client
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let http = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ApiError::Network(e.to_string()))?;

        let base_url_v1 =
            std::env::var("HAWKOP_API_BASE_URL").unwrap_or_else(|_| API_BASE_URL_V1.to_string());
        let base_url_v2 =
            std::env::var("HAWKOP_API_BASE_URL_V2").unwrap_or_else(|_| API_BASE_URL_V2.to_string());

        Ok(Self {
            http,
            base_url_v1,
            base_url_v2,
            rate_limiters: Arc::new(RateLimiterSet::new()),
            auth_state: Arc::new(RwLock::new(AuthState {
                api_key,
                jwt: None,
                jwt_expires_at: None,
            })),
        })
    }

    /// Set the JWT token and expiry
    pub async fn set_jwt(&self, token: JwtToken) {
        let mut state = self.auth_state.write().await;
        state.jwt = Some(token.token);
        state.jwt_expires_at = Some(token.expires_at);
    }

    /// Check if JWT is expired or will expire soon (within 5 minutes)
    async fn is_jwt_expired(&self) -> bool {
        let state = self.auth_state.read().await;
        match state.jwt_expires_at {
            None => true,
            Some(expires_at) => {
                let now = Utc::now();
                let buffer = chrono::Duration::minutes(5);
                expires_at - buffer < now
            }
        }
    }

    /// Get the current JWT token, refreshing if necessary
    async fn get_valid_jwt(&self) -> Result<String> {
        // Check if we need to refresh
        if self.is_jwt_expired().await {
            // Get API key
            let api_key = {
                let state = self.auth_state.read().await;
                state.api_key.clone().ok_or(ApiError::Unauthorized)?
            };

            // Refresh JWT
            let jwt_token = self.authenticate(&api_key).await?;
            self.set_jwt(jwt_token).await;
        }

        // Return current JWT
        let state = self.auth_state.read().await;
        state.jwt.clone().ok_or(ApiError::Unauthorized.into())
    }

    /// Make an authenticated API request
    fn request<'a, T: for<'de> Deserialize<'de> + 'a>(
        &'a self,
        method: reqwest::Method,
        path: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>> {
        Box::pin(async move { self.request_inner(method, &self.base_url_v1, path).await })
    }

    /// Internal request implementation
    async fn request_inner<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        base_url: &str,
        path: &str,
    ) -> Result<T> {
        self.request_with_query(method, base_url, path, &[]).await
    }

    /// Internal request implementation with query parameters
    async fn request_with_query<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        base_url: &str,
        path: &str,
        query_params: &[(&str, String)],
    ) -> Result<T> {
        self.request_with_retry(method, base_url, path, query_params, 0)
            .await
    }

    /// Internal request implementation with retry support for rate limiting
    ///
    /// Uses exponential backoff with jitter for 429 responses:
    /// - Base wait from retry-after header (default 1s)
    /// - Exponential: base * 2^attempt
    /// - Jitter: 0-1000ms random offset to prevent thundering herd
    async fn request_with_retry<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        base_url: &str,
        path: &str,
        query_params: &[(&str, String)],
        attempt: u32,
    ) -> Result<T> {
        // Categorize this endpoint for rate limiting
        let category = EndpointCategory::from_request(path, &method);

        // Wait if rate limiting is active for this category
        self.rate_limiters.wait_for(category).await;

        // Get valid JWT
        let jwt = self.get_valid_jwt().await?;

        // Build request
        let url = format!("{}{}", base_url, path);
        debug!("API request: {} {} (category: {:?})", method, url, category);
        if !query_params.is_empty() {
            debug!("Query params: {:?}", query_params);
        }

        let mut request = self
            .http
            .request(method.clone(), &url)
            .header("Authorization", format!("Bearer {}", jwt));

        // Add query parameters
        if !query_params.is_empty() {
            request = request.query(query_params);
        }

        let response = request.send().await.map_err(ApiError::from)?;

        // Handle response status
        let status = response.status();
        debug!(
            "API response: {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        );

        match status {
            StatusCode::OK => {
                // Get response body as text first for better error messages
                let body = response.text().await.map_err(|e| {
                    ApiError::InvalidResponse(format!("Failed to read response body: {}", e))
                })?;

                // Parse JSON with detailed error reporting
                let data: T = serde_json::from_str(&body).map_err(|e| {
                    // Log part of the response body for debugging
                    let preview = if body.len() > 500 {
                        format!("{}...", &body[..500])
                    } else {
                        body.clone()
                    };
                    debug!("JSON parse error: {} in response: {}", e, preview);
                    ApiError::InvalidResponse(format!(
                        "Failed to parse response: {} (line {}, col {})",
                        e,
                        e.line(),
                        e.column()
                    ))
                })?;
                Ok(data)
            }
            StatusCode::UNAUTHORIZED => {
                debug!("Received 401, attempting token refresh");
                // Try to refresh token once
                let api_key = {
                    let state = self.auth_state.read().await;
                    state.api_key.clone()
                };

                if let Some(api_key) = api_key {
                    let jwt_token = self.authenticate(&api_key).await?;
                    self.set_jwt(jwt_token).await;
                    debug!("Token refreshed, retrying request");

                    // Retry request with same query params - box the recursive call
                    return Box::pin(self.request_with_retry(
                        method,
                        base_url,
                        path,
                        query_params,
                        attempt,
                    ))
                    .await;
                }
                Err(ApiError::Unauthorized.into())
            }
            StatusCode::FORBIDDEN => Err(ApiError::Forbidden.into()),
            StatusCode::NOT_FOUND => {
                let error_msg = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Resource not found".to_string());
                Err(ApiError::NotFound(error_msg).into())
            }
            StatusCode::TOO_MANY_REQUESTS => {
                // Activate rate limiting for THIS endpoint category only
                self.rate_limiters.activate(category).await;

                // Check if we've exceeded max retries
                if attempt >= MAX_RATE_LIMIT_RETRIES {
                    debug!(
                        "Rate limit retry exhausted after {} attempts for {:?}",
                        attempt, category
                    );
                    return Err(ApiError::RateLimited.into());
                }

                // Parse retry-after header (default 1 second for backoff calculation)
                let base_wait_secs = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(1);

                // Exponential backoff: base * 2^attempt
                let backoff_secs = base_wait_secs.saturating_mul(1 << attempt);

                // Jitter: 0-1000ms using nanosecond timestamp (avoids rand dependency)
                let jitter_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.subsec_nanos() % 1000)
                    .unwrap_or(0) as u64;

                let total_wait =
                    Duration::from_secs(backoff_secs) + Duration::from_millis(jitter_ms);

                debug!(
                    "Rate limited (429) for {:?}, attempt {}/{}, waiting {:?} before retry",
                    category,
                    attempt + 1,
                    MAX_RATE_LIMIT_RETRIES,
                    total_wait
                );

                tokio::time::sleep(total_wait).await;

                // Retry with incremented attempt counter
                Box::pin(self.request_with_retry(method, base_url, path, query_params, attempt + 1))
                    .await
            }
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                let error_msg = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Bad request".to_string());
                debug!("Bad request: {}", error_msg);
                Err(ApiError::BadRequest(error_msg).into())
            }
            status if status.is_server_error() => {
                let error_msg = response
                    .text()
                    .await
                    .unwrap_or_else(|_| format!("Server error: {}", status));
                debug!("Server error: {}", error_msg);
                Err(ApiError::ServerError(error_msg).into())
            }
            _ => {
                let error_msg = format!("Unexpected status code: {}", status);
                debug!("{}", error_msg);
                Err(ApiError::InvalidResponse(error_msg).into())
            }
        }
    }
}

// ============================================================================
// AuthApi Implementation
// ============================================================================

#[async_trait]
impl AuthApi for StackHawkClient {
    async fn authenticate(&self, api_key: &str) -> Result<JwtToken> {
        // Wait if rate limiting is active for the default category (auth endpoint)
        let category = EndpointCategory::Default;
        self.rate_limiters.wait_for(category).await;

        #[derive(Deserialize)]
        struct LoginResponse {
            token: String,
        }

        #[derive(Deserialize)]
        struct JwtPayload {
            exp: i64, // Unix timestamp
        }

        let url = format!("{}/auth/login", self.base_url_v1);
        debug!("Authenticating with API key");

        // Use GET with X-ApiKey header
        let response = self
            .http
            .get(&url)
            .header("X-ApiKey", api_key)
            .send()
            .await
            .map_err(ApiError::from)?;

        let status = response.status();
        debug!("Auth response: {}", status);
        if status == StatusCode::UNAUTHORIZED {
            debug!("Authentication failed: unauthorized");
            return Err(ApiError::Unauthorized.into());
        }

        // Get response text for debugging
        let response_text = response
            .text()
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to read response: {}", e)))?;

        let login_response: LoginResponse = serde_json::from_str(&response_text).map_err(|e| {
            ApiError::InvalidResponse(format!(
                "Failed to parse login response: {}. Body was: {}",
                e, response_text
            ))
        })?;

        // Decode JWT to extract expiration time
        // JWT format: header.payload.signature
        let parts: Vec<&str> = login_response.token.split('.').collect();
        if parts.len() != 3 {
            return Err(ApiError::InvalidToken.into());
        }

        // Decode the payload (base64url without padding)
        let payload_b64 = parts[1];
        let payload_bytes = base64_decode_url(payload_b64).map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to decode JWT payload: {}", e))
        })?;

        let payload: JwtPayload = serde_json::from_slice(&payload_bytes).map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to parse JWT payload: {}", e))
        })?;

        let expires_at = chrono::DateTime::from_timestamp(payload.exp, 0).ok_or_else(|| {
            ApiError::InvalidResponse("Invalid JWT expiration timestamp".to_string())
        })?;

        debug!("Authentication successful, token expires at {}", expires_at);
        Ok(JwtToken {
            token: login_response.token,
            expires_at,
        })
    }
}

// ============================================================================
// ListingApi Implementation
// ============================================================================

#[async_trait]
impl ListingApi for StackHawkClient {
    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        #[derive(Deserialize)]
        struct UserOrganization {
            organization: Organization,
        }

        #[derive(Deserialize)]
        struct UserExternal {
            organizations: Vec<UserOrganization>,
        }

        #[derive(Deserialize)]
        struct User {
            external: UserExternal,
        }

        #[derive(Deserialize)]
        struct UserResponse {
            user: User,
        }

        let response: UserResponse = self.request(reqwest::Method::GET, "/user").await?;
        Ok(response
            .user
            .external
            .organizations
            .into_iter()
            .map(|uo| uo.organization)
            .collect())
    }

    async fn list_apps(
        &self,
        org_id: &str,
        pagination: Option<&super::PaginationParams>,
    ) -> Result<Vec<Application>> {
        #[derive(Deserialize)]
        struct AppsResponse {
            applications: Vec<Application>,
        }

        let path = format!("/org/{}/apps", org_id);

        // Build query params from pagination
        let mut query_params: Vec<(&str, String)> =
            pagination.map(|p| p.to_query_params()).unwrap_or_default();

        // Include all application types (API defaults to STANDARD only)
        query_params.push(("applicationTypes", "STANDARD,CLOUD".to_string()));

        let response: AppsResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v2,
                &path,
                &query_params,
            )
            .await?;
        Ok(response.applications)
    }

    async fn list_scans(
        &self,
        org_id: &str,
        pagination: Option<&super::PaginationParams>,
        filters: Option<&super::ScanFilterParams>,
    ) -> Result<Vec<ScanResult>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ScansResponse {
            application_scan_results: Vec<ScanResult>,
        }

        let path = format!("/scan/{}", org_id);

        // Build query params from pagination and filters
        let mut query_params: Vec<(&str, String)> =
            pagination.map(|p| p.to_query_params()).unwrap_or_default();

        // Add filter params (appIds, envs, teamIds, start, end)
        if let Some(f) = filters {
            query_params.extend(f.to_query_params());
        }

        let response: ScansResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;
        Ok(response.application_scan_results)
    }

    async fn list_apps_paged(
        &self,
        org_id: &str,
        pagination: Option<&super::PaginationParams>,
    ) -> Result<PagedResponse<Application>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AppsPagedResponse {
            applications: Vec<Application>,
            /// totalCount may come as a string from the API
            #[serde(default, deserialize_with = "deserialize_string_to_usize")]
            total_count: Option<usize>,
        }

        let path = format!("/org/{}/apps", org_id);

        // Use provided pagination or default
        let default_params = super::PaginationParams::new().page_size(100);
        let params = pagination.unwrap_or(&default_params);
        let mut query_params: Vec<(&str, String)> = params.to_query_params();

        // Include all application types (API defaults to STANDARD only)
        query_params.push(("applicationTypes", "STANDARD,CLOUD".to_string()));

        let response: AppsPagedResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v2,
                &path,
                &query_params,
            )
            .await?;

        Ok(PagedResponse::new(
            response.applications,
            response.total_count,
            params.page_size.unwrap_or(100),
            params.page.unwrap_or(0),
        ))
    }

    async fn list_scans_paged(
        &self,
        org_id: &str,
        pagination: Option<&super::PaginationParams>,
        filters: Option<&super::ScanFilterParams>,
    ) -> Result<PagedResponse<ScanResult>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ScansPagedResponse {
            application_scan_results: Vec<ScanResult>,
            /// totalCount comes as a string from the API
            #[serde(default, deserialize_with = "deserialize_string_to_usize")]
            total_count: Option<usize>,
        }

        let path = format!("/scan/{}", org_id);

        // Use provided pagination or default
        let default_params = super::PaginationParams::new().page_size(100);
        let params = pagination.unwrap_or(&default_params);
        let mut query_params: Vec<(&str, String)> = params.to_query_params();

        // Add filter params (appIds, envs, teamIds, start, end)
        if let Some(f) = filters {
            query_params.extend(f.to_query_params());
        }

        let response: ScansPagedResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;

        Ok(PagedResponse::new(
            response.application_scan_results,
            response.total_count,
            params.page_size.unwrap_or(100),
            params.page.unwrap_or(0),
        ))
    }

    async fn list_users(
        &self,
        org_id: &str,
        pagination: Option<&super::PaginationParams>,
    ) -> Result<Vec<User>> {
        #[derive(Deserialize)]
        struct UsersResponse {
            users: Vec<User>,
        }

        let path = format!("/org/{}/members", org_id);

        // Build query params from pagination
        let query_params: Vec<(&str, String)> =
            pagination.map(|p| p.to_query_params()).unwrap_or_default();

        let response: UsersResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;
        Ok(response.users)
    }

    async fn list_teams(
        &self,
        org_id: &str,
        pagination: Option<&super::PaginationParams>,
    ) -> Result<Vec<Team>> {
        #[derive(Deserialize)]
        struct TeamsResponse {
            teams: Vec<Team>,
        }

        let path = format!("/org/{}/teams", org_id);

        // Build query params from pagination
        let query_params: Vec<(&str, String)> =
            pagination.map(|p| p.to_query_params()).unwrap_or_default();

        let response: TeamsResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;
        Ok(response.teams)
    }

    async fn list_stackhawk_policies(&self) -> Result<Vec<StackHawkPolicy>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PoliciesResponse {
            scan_policies: Vec<StackHawkPolicy>,
        }

        let response: PoliciesResponse = self
            .request_inner(reqwest::Method::GET, &self.base_url_v1, "/policy/all")
            .await?;
        Ok(response.scan_policies)
    }

    async fn list_org_policies(
        &self,
        org_id: &str,
        pagination: Option<&super::PaginationParams>,
    ) -> Result<Vec<OrgPolicy>> {
        #[derive(Deserialize)]
        struct PoliciesResponse {
            policies: Vec<OrgPolicy>,
        }

        let path = format!("/policy/{}/list", org_id);

        // Build query params from pagination
        let query_params: Vec<(&str, String)> =
            pagination.map(|p| p.to_query_params()).unwrap_or_default();

        let response: PoliciesResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;
        Ok(response.policies)
    }

    async fn list_repos(
        &self,
        org_id: &str,
        pagination: Option<&super::PaginationParams>,
    ) -> Result<Vec<Repository>> {
        #[derive(Deserialize)]
        struct ReposResponse {
            repositories: Vec<Repository>,
        }

        let path = format!("/org/{}/repos", org_id);

        // Build query params from pagination
        let query_params: Vec<(&str, String)> =
            pagination.map(|p| p.to_query_params()).unwrap_or_default();

        let response: ReposResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;
        Ok(response.repositories)
    }

    async fn list_oas(
        &self,
        org_id: &str,
        pagination: Option<&super::PaginationParams>,
    ) -> Result<Vec<OASAsset>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct OASResponse {
            oas_files: Vec<OASAsset>,
        }

        let path = format!("/oas/{}/list", org_id);

        // Build query params from pagination
        let query_params: Vec<(&str, String)> =
            pagination.map(|p| p.to_query_params()).unwrap_or_default();

        let response: OASResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;
        Ok(response.oas_files)
    }

    async fn list_scan_configs(
        &self,
        org_id: &str,
        pagination: Option<&super::PaginationParams>,
    ) -> Result<Vec<ScanConfig>> {
        #[derive(Deserialize)]
        struct ConfigResponse {
            configs: Vec<ScanConfig>,
        }

        let path = format!("/configuration/{}/list", org_id);

        // Build query params from pagination
        let query_params: Vec<(&str, String)> =
            pagination.map(|p| p.to_query_params()).unwrap_or_default();

        let response: ConfigResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;
        Ok(response.configs)
    }

    async fn list_secrets(&self) -> Result<Vec<Secret>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct SecretsResponse {
            user_secrets: Vec<Secret>,
        }

        // Note: This is a user-scoped endpoint, not org-scoped
        let response: SecretsResponse = self
            .request_inner(reqwest::Method::GET, &self.base_url_v1, "/user/secret/list")
            .await?;
        Ok(response.user_secrets)
    }

    async fn list_audit(
        &self,
        org_id: &str,
        filters: Option<&AuditFilterParams>,
    ) -> Result<Vec<AuditRecord>> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AuditResponse {
            audit_records: Vec<AuditRecord>,
        }

        let path = format!("/org/{}/audit", org_id);

        // Build query params from filters
        let query_params: Vec<(&str, String)> =
            filters.map(|f| f.to_query_params()).unwrap_or_default();

        let response: AuditResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;
        Ok(response.audit_records)
    }
}

// ============================================================================
// ScanDetailApi Implementation
// ============================================================================

#[async_trait]
impl ScanDetailApi for StackHawkClient {
    async fn get_scan(&self, _org_id: &str, scan_id: &str) -> Result<ScanResult> {
        // There's no direct "get single scan" API endpoint, so we use the alerts
        // endpoint which returns scan metadata along with alerts. We just need
        // the scan metadata from the response.
        let path = format!("/scan/{}/alerts", scan_id);

        let query_params: Vec<(&str, String)> = vec![("pageSize", "1".to_string())];

        let response: ScanAlertsResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;

        // Extract scan metadata from the alerts response
        let result = response
            .application_scan_results
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::NotFound(format!("Scan not found: {}", scan_id)))?;

        // Extract the Scan from the result
        let scan = result
            .scan
            .ok_or_else(|| ApiError::NotFound(format!("Scan metadata not found: {}", scan_id)))?;

        // Build a ScanResult from the alerts response data
        Ok(ScanResult {
            scan,
            scan_duration: result.scan_duration,
            url_count: result.url_count,
            alert_stats: result.alert_stats,
            severity_stats: None,
            app_host: result.app_host,
            policy_name: result.policy_name,
            tags: result.tags,
            metadata: result.metadata,
        })
    }

    async fn list_scan_alerts(
        &self,
        scan_id: &str,
        pagination: Option<&super::PaginationParams>,
    ) -> Result<Vec<ApplicationAlert>> {
        let path = format!("/scan/{}/alerts", scan_id);

        // Build query params from pagination
        let query_params: Vec<(&str, String)> =
            pagination.map(|p| p.to_query_params()).unwrap_or_default();

        let response: ScanAlertsResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;

        // API returns array with single element containing the alerts list
        Ok(response
            .application_scan_results
            .into_iter()
            .next()
            .map(|r| r.application_alerts)
            .unwrap_or_default())
    }

    async fn get_alert_with_paths(
        &self,
        scan_id: &str,
        plugin_id: &str,
        pagination: Option<&super::PaginationParams>,
    ) -> Result<AlertResponse> {
        let path = format!("/scan/{}/alert/{}", scan_id, plugin_id);

        // Build query params from pagination
        let query_params: Vec<(&str, String)> =
            pagination.map(|p| p.to_query_params()).unwrap_or_default();

        let response: AlertResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;

        Ok(response)
    }

    async fn get_alert_message(
        &self,
        scan_id: &str,
        alert_uri_id: &str,
        message_id: &str,
        include_curl: bool,
    ) -> Result<AlertMsgResponse> {
        let path = format!(
            "/scan/{}/uri/{}/messages/{}",
            scan_id, alert_uri_id, message_id
        );

        // Build query params
        let mut query_params: Vec<(&str, String)> = vec![];
        if include_curl {
            query_params.push(("includeValidationCommand", "true".to_string()));
        }

        let response: AlertMsgResponse = self
            .request_with_query(
                reqwest::Method::GET,
                &self.base_url_v1,
                &path,
                &query_params,
            )
            .await?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(target_os = "macos", ignore)]
    #[test]
    fn test_client_creation() {
        let client = StackHawkClient::new(Some("test_key".to_string()));
        assert!(client.is_ok());
    }

    #[cfg_attr(target_os = "macos", ignore)]
    #[tokio::test]
    async fn test_jwt_expiry_check() {
        let client = StackHawkClient::new(None).unwrap();

        // No JWT should be expired
        assert!(client.is_jwt_expired().await);

        // Set expired JWT
        client
            .set_jwt(JwtToken {
                token: "test".to_string(),
                expires_at: Utc::now() - chrono::Duration::hours(1),
            })
            .await;
        assert!(client.is_jwt_expired().await);

        // Set valid JWT (expires in 1 hour)
        client
            .set_jwt(JwtToken {
                token: "test".to_string(),
                expires_at: Utc::now() + chrono::Duration::hours(1),
            })
            .await;
        assert!(!client.is_jwt_expired().await);

        // Set JWT expiring soon (2 minutes)
        client
            .set_jwt(JwtToken {
                token: "test".to_string(),
                expires_at: Utc::now() + chrono::Duration::minutes(2),
            })
            .await;
        assert!(client.is_jwt_expired().await);
    }
}
