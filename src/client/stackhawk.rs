//! StackHawk API client implementation

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use governor::{Quota, RateLimiter as GovernorRateLimiter};
use reqwest::{Client as HttpClient, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::{JwtToken, Organization, StackHawkApi};
use crate::error::{ApiError, Result};

/// StackHawk API base URL
const API_BASE_URL: &str = "https://api.stackhawk.com/api/v1";

/// Rate limit: 360 requests per minute (6 per second)
const RATE_LIMIT_PER_SECOND: u32 = 6;

/// StackHawk API client
pub struct StackHawkClient {
    http: HttpClient,
    base_url: String,
    rate_limiter: Arc<GovernorRateLimiter<governor::state::direct::NotKeyed, governor::clock::DefaultClock>>,
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
        let rate_limiter = Arc::new(GovernorRateLimiter::direct(quota));

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
        state.jwt.clone().ok_or(ApiError::Unauthorized)
    }

    /// Make an authenticated API request
    async fn request<T: for<'de> Deserialize<'de>>(
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
                let state = self.auth_state.read().await;
                if let Some(api_key) = &state.api_key {
                    drop(state); // Release lock before recursive call
                    let jwt_token = self.authenticate(api_key).await?;
                    self.set_jwt(jwt_token).await;

                    // Retry request
                    return self.request(method, path).await;
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

        #[derive(Serialize)]
        struct LoginRequest {
            #[serde(rename = "apiKey")]
            api_key: String,
        }

        #[derive(Deserialize)]
        struct LoginResponse {
            token: String,
            #[serde(rename = "expiresAt")]
            expires_at: chrono::DateTime<Utc>,
        }

        let url = format!("{}/auth/login", self.base_url);
        let request_body = LoginRequest {
            api_key: api_key.to_string(),
        };

        let response = self
            .http
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(ApiError::from)?;

        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(ApiError::Unauthorized.into());
        }

        let login_response = response.json::<LoginResponse>().await.map_err(|e| {
            ApiError::InvalidResponse(format!("Failed to parse login response: {}", e))
        })?;

        Ok(JwtToken {
            token: login_response.token,
            expires_at: login_response.expires_at,
        })
    }

    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        #[derive(Deserialize)]
        struct OrgsResponse {
            organizations: Vec<Organization>,
        }

        let response: OrgsResponse = self.request(reqwest::Method::GET, "/orgs").await?;
        Ok(response.organizations)
    }

    async fn get_org(&self, org_id: &str) -> Result<Organization> {
        let path = format!("/orgs/{}", org_id);
        self.request(reqwest::Method::GET, &path).await
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
        client.set_jwt(JwtToken {
            token: "test".to_string(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
        }).await;
        assert!(client.is_jwt_expired().await);

        // Set valid JWT (expires in 1 hour)
        client.set_jwt(JwtToken {
            token: "test".to_string(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        }).await;
        assert!(!client.is_jwt_expired().await);

        // Set JWT expiring soon (2 minutes)
        client.set_jwt(JwtToken {
            token: "test".to_string(),
            expires_at: Utc::now() + chrono::Duration::minutes(2),
        }).await;
        assert!(client.is_jwt_expired().await);
    }
}
