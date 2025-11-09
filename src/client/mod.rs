//! StackHawk API client

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Result;

pub mod stackhawk;

pub use stackhawk::StackHawkClient;

/// StackHawk API client trait
#[async_trait]
pub trait StackHawkApi: Send + Sync {
    /// Authenticate with API key and get JWT token
    async fn authenticate(&self, api_key: &str) -> Result<JwtToken>;

    /// List all accessible organizations
    async fn list_orgs(&self) -> Result<Vec<Organization>>;

    /// Get organization details by ID
    async fn get_org(&self, org_id: &str) -> Result<Organization>;
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
