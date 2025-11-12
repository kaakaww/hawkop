//! StackHawk API client implementation

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::{Client as HttpClient, StatusCode};
use serde::Deserialize;
use tokio::sync::RwLock;

use super::{Application, JwtToken, Organization, StackHawkApi};
use crate::error::{ApiError, Result};

/// Decode base64url (URL-safe base64 without padding)
fn base64_decode_url(input: &str) -> std::result::Result<Vec<u8>, String> {
    use base64::{engine::general_purpose, Engine as _};

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

/// StackHawk API base URL
const API_BASE_URL: &str = "https://api.stackhawk.com/api/v1";

/// Rate limit: 360 requests per minute (6 per second)
const RATE_LIMIT_PER_SECOND: u32 = 6;

/// StackHawk API client
pub struct StackHawkClient {
    http: HttpClient,
    base_url: String,
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
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

        // Rate limiter: 6 requests per second = 360 per minute
        let quota = Quota::per_second(std::num::NonZeroU32::new(RATE_LIMIT_PER_SECOND).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Ok(Self {
            http,
            base_url: API_BASE_URL.to_string(),
            rate_limiter,
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
        Box::pin(async move { self.request_inner(method, path).await })
    }

    /// Internal request implementation
    async fn request_inner<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
    ) -> Result<T> {
        // Apply rate limiting
        self.rate_limiter.until_ready().await;

        // Get valid JWT
        let jwt = self.get_valid_jwt().await?;

        // Build request
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .request(method.clone(), &url)
            .header("Authorization", format!("Bearer {}", jwt))
            .send()
            .await
            .map_err(ApiError::from)?;

        // Handle response status
        let status = response.status();
        match status {
            StatusCode::OK => {
                let data = response.json::<T>().await.map_err(|e| {
                    ApiError::InvalidResponse(format!("Failed to parse response: {}", e))
                })?;
                Ok(data)
            }
            StatusCode::UNAUTHORIZED => {
                // Try to refresh token once
                let api_key = {
                    let state = self.auth_state.read().await;
                    state.api_key.clone()
                };

                if let Some(api_key) = api_key {
                    let jwt_token = self.authenticate(&api_key).await?;
                    self.set_jwt(jwt_token).await;

                    // Retry request - box the recursive call
                    return Box::pin(self.request_inner(method, path)).await;
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
                let retry_after = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(60);
                Err(ApiError::RateLimit(Duration::from_secs(retry_after)).into())
            }
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                let error_msg = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Bad request".to_string());
                Err(ApiError::BadRequest(error_msg).into())
            }
            status if status.is_server_error() => {
                let error_msg = response
                    .text()
                    .await
                    .unwrap_or_else(|_| format!("Server error: {}", status));
                Err(ApiError::ServerError(error_msg).into())
            }
            _ => {
                let error_msg = format!("Unexpected status code: {}", status);
                Err(ApiError::InvalidResponse(error_msg).into())
            }
        }
    }
}

#[async_trait]
impl StackHawkApi for StackHawkClient {
    async fn authenticate(&self, api_key: &str) -> Result<JwtToken> {
        // Apply rate limiting
        self.rate_limiter.until_ready().await;

        #[derive(Deserialize)]
        struct LoginResponse {
            token: String,
        }

        #[derive(Deserialize)]
        struct JwtPayload {
            exp: i64, // Unix timestamp
        }

        let url = format!("{}/auth/login", self.base_url);

        // Use GET with X-ApiKey header
        let response = self
            .http
            .get(&url)
            .header("X-ApiKey", api_key)
            .send()
            .await
            .map_err(ApiError::from)?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
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

        Ok(JwtToken {
            token: login_response.token,
            expires_at,
        })
    }

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

    async fn list_apps(&self, org_id: &str) -> Result<Vec<Application>> {
        #[derive(Deserialize)]
        struct AppsResponse {
            applications: Vec<Application>,
        }

        // Note: This endpoint is v2, while the base URL is v1, so we use ../v2
        let path = format!("/../v2/org/{}/apps", org_id);
        let response: AppsResponse = self.request(reqwest::Method::GET, &path).await?;
        Ok(response.applications)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = StackHawkClient::new(Some("test_key".to_string()));
        assert!(client.is_ok());
    }

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
