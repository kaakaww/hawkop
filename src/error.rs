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

    #[error("{0}")]
    UnauthorizedFeature(String),

    #[error("Access denied. You don't have permission to access this resource.")]
    Forbidden,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded. Retry after {0:?}")]
    RateLimit(Duration),

    #[error("Rate limited by API after multiple retries. Try again later.")]
    RateLimited,

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

impl ApiError {
    /// Create an UnauthorizedFeature error with a formatted message
    ///
    /// This is used when a 401 is received after a successful token refresh,
    /// indicating the issue is authorization (feature/role) not authentication.
    pub fn unauthorized_feature(endpoint: Option<&str>, role: Option<&str>) -> Self {
        let mut msg = String::from("Access denied. ");

        if let Some(ep) = endpoint {
            msg.push_str(&format!("The endpoint '{}' ", ep));
        } else {
            msg.push_str("This feature ");
        }

        msg.push_str(
            "may require a feature flag not enabled for your organization, \
             or elevated privileges.\n\n",
        );

        let role_display = role.unwrap_or("Unknown");
        msg.push_str(&format!("Your current role: {}\n\n", role_display));

        msg.push_str("Possible causes:\n");
        msg.push_str("  • This feature requires a plan upgrade or feature flag\n");
        msg.push_str("  • Your role may not have access (Owner/Admin/Member)");

        ApiError::UnauthorizedFeature(msg)
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

    #[error("Profile '{0}' not found. Run `hawkop profile list` to see available profiles.")]
    ProfileNotFound(String),

    #[error("Profile '{0}' already exists. Use a different name or delete the existing profile.")]
    ProfileExists(String),

    #[error(
        "Cannot delete active profile '{0}'. Switch to another profile first with `hawkop profile use <other>`."
    )]
    CannotDeleteActive(String),

    #[error("Cannot delete the 'default' profile. It serves as a fallback.")]
    CannotDeleteDefault,
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        ConfigError::ParseError(err.to_string())
    }
}

/// Cache-related errors
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache database error: {0}")]
    Database(String),

    #[error("Cache I/O error: {0}")]
    Io(String),

    #[error("Could not determine cache directory")]
    NoHome,
}

impl From<rusqlite::Error> for CacheError {
    fn from(err: rusqlite::Error) -> Self {
        CacheError::Database(err.to_string())
    }
}

impl From<CacheError> for Error {
    fn from(err: CacheError) -> Self {
        Error::Other(err.to_string())
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
    fn test_api_error_unauthorized_feature_with_role() {
        let err = ApiError::unauthorized_feature(
            Some("https://api.stackhawk.com/api/v1/configuration/org-id/list"),
            Some("MEMBER"),
        );
        let msg = err.to_string();
        assert!(msg.contains("Access denied."));
        assert!(msg.contains("/api/v1/configuration"));
        assert!(msg.contains("Your current role: MEMBER"));
        assert!(msg.contains("feature flag"));
    }

    #[test]
    fn test_api_error_unauthorized_feature_without_role() {
        let err = ApiError::unauthorized_feature(
            Some("https://api.stackhawk.com/api/v1/some/endpoint"),
            None,
        );
        let msg = err.to_string();
        assert!(msg.contains("Access denied."));
        assert!(msg.contains("/api/v1/some/endpoint"));
        // Role should always be shown now, defaulting to "Unknown"
        assert!(msg.contains("Your current role: Unknown"));
    }

    #[test]
    fn test_api_error_unauthorized_feature_no_details() {
        let err = ApiError::unauthorized_feature(None, None);
        let msg = err.to_string();
        assert!(msg.contains("This feature"));
        assert!(msg.contains("feature flag"));
        assert!(msg.contains("Your current role: Unknown"));
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
