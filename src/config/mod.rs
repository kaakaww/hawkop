//! Configuration management for HawkOp
//!
//! Supports a profile-based configuration system (v2) with backward-compatible
//! migration from the legacy single-profile format (v1).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::{ConfigError, Result};

/// Current config format version
pub const CONFIG_VERSION: u32 = 2;

fn default_version() -> u32 {
    CONFIG_VERSION
}

fn default_profile_name() -> String {
    "default".to_string()
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// StackHawk API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Default organization ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,

    /// Custom API host for development/testing (e.g., "http://localhost:8080")
    ///
    /// When set, overrides the default StackHawk API host. The v1 and v2 API
    /// paths are computed automatically from this host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_host: Option<String>,

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

#[allow(dead_code)]
impl Config {
    /// Get the default config file path
    pub fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(ConfigError::Invalid(
            "Could not determine home directory".to_string(),
        ))?;

        Ok(home.join(".hawkop").join("config.yaml"))
    }

    /// Resolve a config path, falling back to the default location
    pub fn resolve_path(path: Option<&str>) -> Result<PathBuf> {
        match path {
            Some(p) => Ok(PathBuf::from(p)),
            None => Self::default_path(),
        }
    }

    /// Load configuration from an optional path (or default)
    pub fn load_at(path: Option<&str>) -> Result<Self> {
        let path = Self::resolve_path(path)?;
        Self::load_from(path)
    }

    /// Load configuration from the default path
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        Self::load_from(Self::default_path()?)
    }

    /// Load configuration from a specific path
    pub fn load_from(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(ConfigError::NotFound.into());
        }

        let contents = std::fs::read_to_string(&path)?;
        let config: Config = serde_yaml::from_str(&contents).map_err(ConfigError::from)?;

        Ok(config)
    }

    /// Save configuration to the default path
    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        self.save_to(Self::default_path()?)
    }

    /// Save configuration to an optional path (or default)
    pub fn save_at(&self, path: Option<&str>) -> Result<()> {
        let path = Self::resolve_path(path)?;
        self.save_to(path)
    }

    /// Save configuration to a specific path
    pub fn save_to(&self, path: PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize config
        let contents =
            serde_yaml::to_string(self).map_err(|e| ConfigError::SaveError(e.to_string()))?;

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

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            org_id: None,
            api_host: None,
            jwt: None,
            preferences: Preferences::default(),
        }
    }
}

// ============================================================================
// Profile-based Configuration (v2)
// ============================================================================

/// Single profile's configuration
///
/// Each profile stores its own API key, organization, API host, JWT token,
/// and preferences, allowing users to switch between different StackHawk
/// environments easily.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// StackHawk API key for this profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Default organization ID for this profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_id: Option<String>,

    /// Custom API host (e.g., "http://localhost:8080" for local dev)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_host: Option<String>,

    /// Cached JWT token for this profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtToken>,

    /// User preferences for this profile
    #[serde(default)]
    pub preferences: Preferences,
}

impl ProfileConfig {
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

/// Convert legacy Config to ProfileConfig for migration
impl From<Config> for ProfileConfig {
    fn from(config: Config) -> Self {
        Self {
            api_key: config.api_key,
            org_id: config.org_id,
            api_host: config.api_host,
            jwt: config.jwt,
            preferences: config.preferences,
        }
    }
}

/// Top-level configuration with multiple named profiles (v2 format)
///
/// This structure supports:
/// - Multiple named profiles for different environments
/// - An active profile that's used by default
/// - Automatic migration from v1 (legacy) format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfiledConfig {
    /// Config format version (always 2 for this format)
    #[serde(default = "default_version")]
    pub version: u32,

    /// Name of the currently active profile
    #[serde(default = "default_profile_name")]
    pub active_profile: String,

    /// Map of profile name to profile configuration
    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,
}

impl ProfiledConfig {
    /// Get the default config file path
    #[allow(dead_code)]
    pub fn default_path() -> Result<PathBuf> {
        Config::default_path()
    }

    /// Resolve a config path, falling back to the default location
    pub fn resolve_path(path: Option<&str>) -> Result<PathBuf> {
        Config::resolve_path(path)
    }

    /// Load configuration from an optional path (or default), with auto-migration
    pub fn load_at(path: Option<&str>) -> Result<Self> {
        let path = Self::resolve_path(path)?;
        Self::load_from(path)
    }

    /// Load configuration from a specific path, with auto-migration from v1
    pub fn load_from(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(ConfigError::NotFound.into());
        }

        let contents = std::fs::read_to_string(&path)?;

        // Try v2 format first (has version and profiles fields)
        #[allow(clippy::collapsible_if)]
        if let Ok(config) = serde_yaml::from_str::<ProfiledConfig>(&contents) {
            if config.version >= 2 && !config.profiles.is_empty() {
                return Ok(config);
            }
        }

        // Fall back to v1 format and migrate
        let legacy: Config = serde_yaml::from_str(&contents).map_err(ConfigError::from)?;
        Ok(Self::migrate_from_v1(legacy))
    }

    /// Migrate from v1 (legacy) config format
    fn migrate_from_v1(legacy: Config) -> Self {
        let profile = ProfileConfig::from(legacy);
        let mut profiles = HashMap::new();
        profiles.insert("default".to_string(), profile);

        Self {
            version: CONFIG_VERSION,
            active_profile: "default".to_string(),
            profiles,
        }
    }

    /// Get a profile by name
    pub fn get_profile(&self, name: &str) -> Result<&ProfileConfig> {
        self.profiles
            .get(name)
            .ok_or_else(|| ConfigError::ProfileNotFound(name.to_string()).into())
    }

    /// Get a mutable reference to a profile by name
    pub fn get_profile_mut(&mut self, name: &str) -> Result<&mut ProfileConfig> {
        self.profiles
            .get_mut(name)
            .ok_or_else(|| ConfigError::ProfileNotFound(name.to_string()).into())
    }

    /// Resolve which profile to use based on override or active profile
    ///
    /// Precedence: override_name > active_profile > "default"
    ///
    /// Returns the resolved profile name (owned String) and a reference to the profile.
    pub fn resolve_profile(&self, override_name: Option<&str>) -> Result<(String, &ProfileConfig)> {
        let profile_name = override_name
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.active_profile.clone());

        // If the requested profile doesn't exist, return error
        if !self.profiles.contains_key(&profile_name) {
            return Err(ConfigError::ProfileNotFound(profile_name).into());
        }

        let profile = self.profiles.get(&profile_name).unwrap();
        Ok((profile_name, profile))
    }

    /// List all profile names
    pub fn list_profiles(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.profiles.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Create a new profile
    pub fn create_profile(&mut self, name: &str, config: ProfileConfig) -> Result<()> {
        if self.profiles.contains_key(name) {
            return Err(ConfigError::ProfileExists(name.to_string()).into());
        }
        self.profiles.insert(name.to_string(), config);
        Ok(())
    }

    /// Delete a profile
    pub fn delete_profile(&mut self, name: &str) -> Result<()> {
        if name == "default" {
            return Err(ConfigError::CannotDeleteDefault.into());
        }
        if name == self.active_profile {
            return Err(ConfigError::CannotDeleteActive(name.to_string()).into());
        }
        if !self.profiles.contains_key(name) {
            return Err(ConfigError::ProfileNotFound(name.to_string()).into());
        }
        self.profiles.remove(name);
        Ok(())
    }

    /// Set the active profile
    pub fn set_active_profile(&mut self, name: &str) -> Result<()> {
        if !self.profiles.contains_key(name) {
            return Err(ConfigError::ProfileNotFound(name.to_string()).into());
        }
        self.active_profile = name.to_string();
        Ok(())
    }

    /// Save configuration to an optional path (or default)
    pub fn save_at(&self, path: Option<&str>) -> Result<()> {
        let path = Self::resolve_path(path)?;
        self.save_to(path)
    }

    /// Save configuration to a specific path
    pub fn save_to(&self, path: PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize config
        let contents =
            serde_yaml::to_string(self).map_err(|e| ConfigError::SaveError(e.to_string()))?;

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
}

impl Default for ProfiledConfig {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert("default".to_string(), ProfileConfig::default());

        Self {
            version: CONFIG_VERSION,
            active_profile: "default".to_string(),
            profiles,
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
        assert!(config.api_host.is_none());
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

    // Profile-based config tests

    #[test]
    fn test_default_profiled_config() {
        let config = ProfiledConfig::default();
        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.active_profile, "default");
        assert!(config.profiles.contains_key("default"));
    }

    #[test]
    fn test_migrate_from_v1() {
        let legacy = Config {
            api_key: Some("sk_test_123".to_string()),
            org_id: Some("org_456".to_string()),
            api_host: Some("http://localhost:8080".to_string()),
            jwt: None,
            preferences: Preferences::default(),
        };

        let migrated = ProfiledConfig::migrate_from_v1(legacy);

        assert_eq!(migrated.version, CONFIG_VERSION);
        assert_eq!(migrated.active_profile, "default");
        assert!(migrated.profiles.contains_key("default"));

        let profile = migrated.profiles.get("default").unwrap();
        assert_eq!(profile.api_key, Some("sk_test_123".to_string()));
        assert_eq!(profile.org_id, Some("org_456".to_string()));
        assert_eq!(profile.api_host, Some("http://localhost:8080".to_string()));
    }

    #[test]
    fn test_resolve_profile_with_override() {
        let mut config = ProfiledConfig::default();
        config.profiles.insert(
            "prod".to_string(),
            ProfileConfig {
                api_key: Some("sk_prod".to_string()),
                ..Default::default()
            },
        );
        config.active_profile = "default".to_string();

        // Override should take precedence
        let (name, profile) = config.resolve_profile(Some("prod")).unwrap();
        assert_eq!(name, "prod");
        assert_eq!(profile.api_key, Some("sk_prod".to_string()));
    }

    #[test]
    fn test_resolve_profile_uses_active() {
        let mut config = ProfiledConfig::default();
        config.profiles.insert(
            "test".to_string(),
            ProfileConfig {
                api_key: Some("sk_test".to_string()),
                ..Default::default()
            },
        );
        config.active_profile = "test".to_string();

        // Should use active profile when no override
        let (name, profile) = config.resolve_profile(None).unwrap();
        assert_eq!(name, "test");
        assert_eq!(profile.api_key, Some("sk_test".to_string()));
    }

    #[test]
    fn test_resolve_profile_not_found() {
        let config = ProfiledConfig::default();
        let result = config.resolve_profile(Some("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_profile() {
        let mut config = ProfiledConfig::default();
        let new_profile = ProfileConfig {
            api_key: Some("sk_new".to_string()),
            ..Default::default()
        };

        config.create_profile("new", new_profile).unwrap();
        assert!(config.profiles.contains_key("new"));
        assert_eq!(
            config.profiles.get("new").unwrap().api_key,
            Some("sk_new".to_string())
        );
    }

    #[test]
    fn test_create_profile_exists() {
        let mut config = ProfiledConfig::default();
        let result = config.create_profile("default", ProfileConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_profile() {
        let mut config = ProfiledConfig::default();
        config
            .profiles
            .insert("test".to_string(), ProfileConfig::default());
        config.active_profile = "default".to_string();

        config.delete_profile("test").unwrap();
        assert!(!config.profiles.contains_key("test"));
    }

    #[test]
    fn test_delete_profile_cannot_delete_default() {
        let mut config = ProfiledConfig::default();
        let result = config.delete_profile("default");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_profile_cannot_delete_active() {
        let mut config = ProfiledConfig::default();
        config
            .profiles
            .insert("test".to_string(), ProfileConfig::default());
        config.active_profile = "test".to_string();

        let result = config.delete_profile("test");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_active_profile() {
        let mut config = ProfiledConfig::default();
        config
            .profiles
            .insert("prod".to_string(), ProfileConfig::default());

        config.set_active_profile("prod").unwrap();
        assert_eq!(config.active_profile, "prod");
    }

    #[test]
    fn test_set_active_profile_not_found() {
        let mut config = ProfiledConfig::default();
        let result = config.set_active_profile("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_profiles() {
        let mut config = ProfiledConfig::default();
        config
            .profiles
            .insert("alpha".to_string(), ProfileConfig::default());
        config
            .profiles
            .insert("beta".to_string(), ProfileConfig::default());

        let profiles = config.list_profiles();
        // Should be sorted alphabetically
        assert_eq!(profiles, vec!["alpha", "beta", "default"]);
    }

    #[test]
    fn test_profile_config_token_expiry() {
        let mut profile = ProfileConfig::default();

        // No token should be expired
        assert!(profile.is_token_expired());

        // Token expires in the future
        profile.jwt = Some(JwtToken {
            token: "test".to_string(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        });
        assert!(!profile.is_token_expired());
    }
}
