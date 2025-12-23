//! Command execution context
//!
//! Provides a unified context for command execution, eliminating boilerplate
//! for config loading, authentication validation, and client initialization.

use crate::cli::OutputFormat;
use crate::client::{JwtToken, StackHawkApi, StackHawkClient};
use crate::config::Config;
use crate::error::Result;

/// Context for command execution containing config, client, and runtime options.
///
/// This struct encapsulates all shared state needed by commands, providing:
/// - Loaded and validated configuration
/// - Authenticated API client with JWT set
/// - Output format preference
/// - Resolved organization ID (from config or override)
pub struct CommandContext {
    /// Loaded and validated configuration
    pub config: Config,
    /// Authenticated API client
    pub client: StackHawkClient,
    /// Output format preference
    pub format: OutputFormat,
}

impl CommandContext {
    /// Create a new command context with full initialization.
    ///
    /// This handles:
    /// - Loading config from path (or default location)
    /// - Applying org_id override if provided
    /// - Validating authentication (API key present)
    /// - Creating the API client
    /// - Authenticating and caching JWT token
    ///
    /// # Arguments
    /// * `format` - Output format (table/json)
    /// * `org_override` - Optional organization ID to override config
    /// * `config_path` - Optional path to config file (defaults to ~/.hawkop/config.yaml)
    ///
    /// # Errors
    /// Returns error if config cannot be loaded or authentication is invalid.
    pub async fn new(
        format: OutputFormat,
        org_override: Option<&str>,
        config_path: Option<&str>,
    ) -> Result<Self> {
        let mut config = Config::load_at(config_path)?;
        config.validate_auth()?;

        // Apply org override if provided
        if let Some(org) = org_override {
            config.org_id = Some(org.to_string());
        }

        let client = StackHawkClient::new(config.api_key.clone())?;

        // Use cached JWT if valid, otherwise authenticate and cache
        if !config.is_token_expired() {
            // Use cached token
            if let Some(ref jwt) = config.jwt {
                client
                    .set_jwt(JwtToken {
                        token: jwt.token.clone(),
                        expires_at: jwt.expires_at,
                    })
                    .await;
            }
        } else {
            // Authenticate and cache the new token
            let api_key = config.api_key.as_ref().expect("validated above");
            let jwt = client.authenticate(api_key).await?;

            // Save to config for future runs
            config.jwt = Some(crate::config::JwtToken {
                token: jwt.token.clone(),
                expires_at: jwt.expires_at,
            });
            config.save_at(config_path)?;

            // Set on client
            client.set_jwt(jwt).await;
        }

        Ok(Self {
            config,
            client,
            format,
        })
    }

    /// Get the organization ID, returning an error if not set.
    ///
    /// Use this when a command requires an organization ID.
    pub fn require_org_id(&self) -> Result<&str> {
        self.config
            .org_id
            .as_deref()
            .ok_or_else(|| crate::error::ConfigError::MissingOrgId.into())
    }

    /// Get the organization ID if set.
    #[allow(dead_code)]
    pub fn org_id(&self) -> Option<&str> {
        self.config.org_id.as_deref()
    }
}
