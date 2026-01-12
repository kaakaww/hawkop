//! Authentication API trait

use async_trait::async_trait;

use crate::client::models::JwtToken;
use crate::error::Result;

/// Authentication operations for the StackHawk API
#[async_trait]
pub trait AuthApi: Send + Sync {
    /// Authenticate with API key and get JWT token
    async fn authenticate(&self, api_key: &str) -> Result<JwtToken>;
}
