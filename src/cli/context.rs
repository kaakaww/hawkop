//! Command execution context
//!
//! Provides a unified context for command execution, eliminating boilerplate
//! for config loading, authentication validation, and client initialization.

use std::sync::Arc;

use crate::cache::CachedStackHawkClient;
use crate::cli::OutputFormat;
use crate::cli::args::GlobalOptions;
use crate::client::models::JwtToken;
use crate::client::{AuthApi, StackHawkClient};
use crate::config::{ProfileConfig, ProfiledConfig};
use crate::error::Result;

/// Context for command execution containing config, client, and runtime options.
///
/// This struct encapsulates all shared state needed by commands, providing:
/// - Loaded configuration with resolved profile
/// - Authenticated API client with JWT set (wrapped in Arc for parallel requests)
/// - Output format preference
/// - Current profile information
/// - Resolved API host (for display in status command)
pub struct CommandContext {
    /// Full profiled configuration (for saving updates)
    pub profiled_config: ProfiledConfig,
    /// Current profile settings (convenience accessor)
    pub profile: ProfileConfig,
    /// Name of the active profile
    pub profile_name: String,
    /// Authenticated API client with caching (Arc-wrapped for parallel request support)
    pub client: Arc<CachedStackHawkClient<StackHawkClient>>,
    /// Output format preference
    pub format: OutputFormat,
    /// Resolved API host (for display purposes, e.g., in status command)
    #[allow(dead_code)]
    pub api_host: Option<String>,
    /// Config file path (for saving updates)
    pub config_path: Option<String>,
}

impl CommandContext {
    /// Create a new command context with full initialization.
    ///
    /// This handles:
    /// - Loading config from path (or default location)
    /// - Resolving the active profile (from override or config)
    /// - Applying org_id override if provided
    /// - Resolving API host (CLI > profile)
    /// - Validating authentication (API key present)
    /// - Creating the API client with caching wrapper
    /// - Authenticating and caching JWT token
    ///
    /// # Arguments
    /// * `opts` - Global CLI options containing format, org override, config path, etc.
    ///
    /// # Errors
    /// Returns error if config cannot be loaded or authentication is invalid.
    pub async fn new(opts: &GlobalOptions) -> Result<Self> {
        let mut profiled_config = ProfiledConfig::load_at(opts.config_ref())?;

        // Resolve which profile to use
        let (profile_name, _profile_ref) = profiled_config.resolve_profile(opts.profile_ref())?;
        log::debug!("Using profile: {}", profile_name);

        // Get a mutable copy of the profile for modifications
        let mut profile = profiled_config.get_profile(&profile_name)?.clone();

        // Validate authentication
        profile.validate_auth()?;

        // Apply org override if provided
        if let Some(org) = opts.org_ref() {
            profile.org_id = Some(org.to_string());
        }

        // Resolve API host: CLI flag > profile setting (env var is handled in client)
        let resolved_api_host = opts
            .api_host_ref()
            .map(|s| s.to_string())
            .or_else(|| profile.api_host.clone());

        // Create the raw client first (need to set JWT before wrapping)
        let raw_client =
            StackHawkClient::with_host(profile.api_key.clone(), resolved_api_host.clone())?;

        // Use cached JWT if valid, otherwise authenticate and cache
        if !profile.is_token_expired() {
            // Use cached token
            if let Some(ref jwt) = profile.jwt {
                raw_client
                    .set_jwt(JwtToken {
                        token: jwt.token.clone(),
                        expires_at: jwt.expires_at,
                    })
                    .await;
            }
        } else {
            // Authenticate and cache the new token
            let api_key = profile.api_key.as_ref().expect("validated above");
            let jwt = raw_client.authenticate(api_key).await?;

            // Save to profile for future runs
            profile.jwt = Some(crate::config::JwtToken {
                token: jwt.token.clone(),
                expires_at: jwt.expires_at,
            });

            // Update the profile in the config and save
            if let Ok(stored_profile) = profiled_config.get_profile_mut(&profile_name) {
                stored_profile.jwt = profile.jwt.clone();
            }
            profiled_config.save_at(opts.config_ref())?;

            // Set on client
            raw_client.set_jwt(jwt).await;
        }

        // Wrap with caching layer (disabled if --no-cache)
        // Pass API host to cache layer to prevent cross-environment cache hits
        let client = Arc::new(CachedStackHawkClient::with_host(
            raw_client,
            !opts.no_cache,
            resolved_api_host.clone(),
        ));

        Ok(Self {
            profiled_config,
            profile,
            profile_name,
            client,
            format: opts.format,
            api_host: resolved_api_host,
            config_path: opts.config.clone(),
        })
    }

    /// Get the organization ID, returning an error if not set.
    ///
    /// Use this when a command requires an organization ID.
    pub fn require_org_id(&self) -> Result<&str> {
        self.profile
            .org_id
            .as_deref()
            .ok_or_else(|| crate::error::ConfigError::MissingOrgId.into())
    }

    /// Get the organization ID if set.
    #[allow(dead_code)]
    pub fn org_id(&self) -> Option<&str> {
        self.profile.org_id.as_deref()
    }

    /// Save the current configuration to disk
    pub fn save_config(&self) -> Result<()> {
        self.profiled_config.save_at(self.config_path.as_deref())
    }
}
