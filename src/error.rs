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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_unauthorized_message() {
        let err = ApiError::Unauthorized;
        assert!(err.to_string().contains("hawkop init"));
    }

    #[test]
    fn test_api_error_forbidden_message() {
        let err = ApiError::Forbidden;
        assert!(err.to_string().contains("permission"));
    }

    #[test]
    fn test_api_error_not_found() {
        let err = ApiError::NotFound("Application abc-123".to_string());
        assert!(err.to_string().contains("abc-123"));
    }

    #[test]
    fn test_api_error_rate_limit() {
        let err = ApiError::RateLimit(Duration::from_secs(30));
        let msg = err.to_string();
        assert!(msg.contains("Rate limit"));
        assert!(msg.contains("30"));
    }

    #[test]
    fn test_api_error_bad_request() {
        let err = ApiError::BadRequest("Invalid filter".to_string());
        assert!(err.to_string().contains("Invalid filter"));
    }

    #[test]
    fn test_api_error_server_error() {
        let err = ApiError::ServerError("Internal error".to_string());
        assert!(err.to_string().contains("Internal error"));
    }

    #[test]
    fn test_api_error_network() {
        let err = ApiError::Network("Connection refused".to_string());
        assert!(err.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_api_error_invalid_response() {
        let err = ApiError::InvalidResponse("Missing field 'id'".to_string());
        assert!(err.to_string().contains("Missing field"));
    }

    #[test]
    fn test_api_error_invalid_token() {
        let err = ApiError::InvalidToken;
        assert!(err.to_string().contains("JWT"));
    }

    #[test]
    fn test_config_error_not_found() {
        let err = ConfigError::NotFound;
        assert!(err.to_string().contains("hawkop init"));
    }

    #[test]
    fn test_config_error_parse() {
        let err = ConfigError::ParseError("unexpected key".to_string());
        assert!(err.to_string().contains("unexpected key"));
    }

    #[test]
    fn test_config_error_invalid() {
        let err = ConfigError::Invalid("bad format".to_string());
        assert!(err.to_string().contains("bad format"));
    }

    #[test]
    fn test_config_error_save() {
        let err = ConfigError::SaveError("disk full".to_string());
        assert!(err.to_string().contains("disk full"));
    }

    #[test]
    fn test_config_error_missing_api_key() {
        let err = ConfigError::MissingApiKey;
        assert!(err.to_string().contains("hawkop init"));
    }

    #[test]
    fn test_config_error_missing_org() {
        let err = ConfigError::MissingOrgId;
        assert!(err.to_string().contains("hawkop org set"));
    }

    #[test]
    fn test_error_from_api_error() {
        let api_err = ApiError::Unauthorized;
        let err: Error = api_err.into();

        match err {
            Error::Api(ApiError::Unauthorized) => (),
            _ => panic!("Expected Error::Api(ApiError::Unauthorized)"),
        }
    }

    #[test]
    fn test_error_from_config_error() {
        let cfg_err = ConfigError::NotFound;
        let err: Error = cfg_err.into();

        match err {
            Error::Config(ConfigError::NotFound) => (),
            _ => panic!("Expected Error::Config(ConfigError::NotFound)"),
        }
    }

    #[test]
    fn test_error_other() {
        let err = Error::Other("Custom error".to_string());
        assert!(err.to_string().contains("Custom error"));
    }

    #[test]
    fn test_config_error_from_yaml_error() {
        let yaml_str = "invalid: [yaml: content";
        let yaml_err = serde_yaml::from_str::<serde_yaml::Value>(yaml_str).unwrap_err();
        let config_err: ConfigError = yaml_err.into();

        match config_err {
            ConfigError::ParseError(_) => (),
            _ => panic!("Expected ConfigError::ParseError"),
        }
    }
}
