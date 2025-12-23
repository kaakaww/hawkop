//! Mock StackHawk API client for testing
//!
//! Provides a mock implementation of the StackHawkApi trait for unit testing
//! without making real API calls.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::{Application, JwtToken, Organization, StackHawkApi};
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

#[async_trait]
impl StackHawkApi for MockStackHawkClient {
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

    async fn list_orgs(&self) -> Result<Vec<Organization>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_orgs += 1;

        Ok(self.orgs.lock().await.clone())
    }

    async fn list_apps(
        &self,
        _org_id: &str,
        _pagination: Option<&super::PaginationParams>,
    ) -> Result<Vec<Application>> {
        self.check_error().await?;

        let mut counts = self.call_count.lock().await;
        counts.list_apps += 1;

        Ok(self.apps.lock().await.clone())
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
