//! Error types for the HawkOp CLI

use std::time::Duration;
use thiserror::Error;

/// Result type alias for HawkOp operations
pub type Result<T> = std::result::Result<T, Error>;

/// Top-level error type for the application
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum Error {
    #[error(transparent)]
    Api(#[from] ApiError),

    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Interactive prompt error: {0}")]
    Dialoguer(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Operation failed: {0}")]
    Other(String),
}

impl From<dialoguer::Error> for Error {
    fn from(err: dialoguer::Error) -> Self {
        Error::Dialoguer(err.to_string())
    }
}

/// API-related errors
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ApiError {
    #[error("Authentication failed. Run `hawkop init` to set up your API key.")]
    Unauthorized,

    #[error("Access denied. You don't have permission to access this resource.")]
    Forbidden,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded. Retry after {0:?}")]
    RateLimit(Duration),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("JWT token expired or invalid")]
    InvalidToken,
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApiError::Network("Request timed out".to_string())
        } else if err.is_connect() {
            ApiError::Network("Failed to connect to API".to_string())
        } else {
            ApiError::Network(err.to_string())
        }
    }
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found. Run `hawkop init` to set up.")]
    NotFound,

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Failed to save configuration: {0}")]
    SaveError(String),

    #[error("API key not configured. Run `hawkop init` to set up your API key.")]
    MissingApiKey,

    #[error(
        "Organization not configured. Run `hawkop org set <ORG_ID>` to set default organization."
    )]
    MissingOrgId,
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}
