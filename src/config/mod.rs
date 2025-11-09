//! Configuration management for HawkOp

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{ConfigError, Result};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// StackHawk API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Default organization ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,

    /// JWT authentication state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtToken>,

    /// User preferences
    #[serde(default)]
    pub preferences: Preferences,
}

/// JWT token with expiry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtToken {
    /// The JWT token string
    pub token: String,

    /// Token expiration time
    pub expires_at: DateTime<Utc>,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    /// Default output format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Default page size for API requests
    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

fn default_page_size() -> usize {
    1000
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            format: None,
            page_size: default_page_size(),
        }
    }
}

impl Config {
    /// Get the default config file path
    pub fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(ConfigError::Invalid(
            "Could not determine home directory".to_string(),
        ))?;

        Ok(home.join(".hawkop").join("config.yaml"))
    }

    /// Load configuration from the default path
    pub fn load() -> Result<Self> {
        Self::load_from(Self::default_path()?)
    }

    /// Load configuration from a specific path
    pub fn load_from(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(ConfigError::NotFound.into());
        }

        let contents = std::fs::read_to_string(&path)?;
        let config: Config = serde_yaml::from_str(&contents)
            .map_err(ConfigError::from)?;

        Ok(config)
    }

    /// Save configuration to the default path
    pub fn save(&self) -> Result<()> {
        self.save_to(Self::default_path()?)
    }

    /// Save configuration to a specific path
    pub fn save_to(&self, path: PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize config
        let contents = serde_yaml::to_string(self)
            .map_err(|e| ConfigError::SaveError(e.to_string()))?;

        // Write to file
        std::fs::write(&path, contents)?;

        // Set file permissions to 600 on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    /// Check if the JWT token is expired or will expire soon (within 5 minutes)
    pub fn is_token_expired(&self) -> bool {
        match &self.jwt {
            None => true,
            Some(jwt) => {
                let now = Utc::now();
                let buffer = chrono::Duration::minutes(5);
                jwt.expires_at - buffer < now
            }
        }
    }

    /// Validate that required configuration is present
    pub fn validate_auth(&self) -> Result<()> {
        if self.api_key.is_none() {
            return Err(ConfigError::MissingApiKey.into());
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            org_id: None,
            jwt: None,
            preferences: Preferences::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.api_key.is_none());
        assert!(config.org_id.is_none());
        assert!(config.jwt.is_none());
        assert_eq!(config.preferences.page_size, 1000);
    }

    #[test]
    fn test_token_expiry() {
        let mut config = Config::default();

        // No token should be expired
        assert!(config.is_token_expired());

        // Token expired in the past
        config.jwt = Some(JwtToken {
            token: "test".to_string(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
        });
        assert!(config.is_token_expired());

        // Token expires in the future (more than 5 minutes)
        config.jwt = Some(JwtToken {
            token: "test".to_string(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        });
        assert!(!config.is_token_expired());

        // Token expires soon (less than 5 minutes)
        config.jwt = Some(JwtToken {
            token: "test".to_string(),
            expires_at: Utc::now() + chrono::Duration::minutes(2),
        });
        assert!(config.is_token_expired());
    }
}
