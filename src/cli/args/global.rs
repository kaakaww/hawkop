//! Global CLI options shared across all commands
//!
//! This module provides a centralized struct for global CLI options, eliminating
//! the need to thread 6+ parameters through every command handler.

use crate::cli::{Cli, OutputFormat};

/// Global CLI options passed to all command handlers.
///
/// This struct consolidates all global flags from the CLI into a single unit,
/// making handler signatures cleaner and more maintainable. When new global
/// options are added, only this struct and `main.rs` need to change.
///
/// # Precedence
///
/// For most options, the precedence is: CLI flag > environment variable > config file > default.
/// This struct captures the CLI/env layer; config file defaults are resolved later in
/// `CommandContext`.
#[derive(Debug, Clone)]
pub struct GlobalOptions {
    /// Output format (pretty, table, json)
    pub format: OutputFormat,

    /// Organization ID override (bypasses config file)
    pub org: Option<String>,

    /// Custom config file path (defaults to ~/.hawkop/config.yaml)
    pub config: Option<String>,

    /// Profile name override (bypasses active_profile in config)
    pub profile: Option<String>,

    /// Bypass cache and fetch fresh data from API
    pub no_cache: bool,

    /// Custom API host for development/testing
    pub api_host: Option<String>,
}

impl GlobalOptions {
    /// Create GlobalOptions from a parsed CLI struct.
    ///
    /// This is the primary constructor, called once in main.rs after parsing.
    pub fn from_cli(cli: &Cli) -> Self {
        Self {
            format: cli.format,
            org: cli.org.clone(),
            config: cli.config.clone(),
            profile: cli.profile.clone(),
            no_cache: cli.no_cache,
            api_host: cli.api_host.clone(),
        }
    }

    /// Get organization override as `Option<&str>`.
    pub fn org_ref(&self) -> Option<&str> {
        self.org.as_deref()
    }

    /// Get config path as `Option<&str>`.
    pub fn config_ref(&self) -> Option<&str> {
        self.config.as_deref()
    }

    /// Get profile override as `Option<&str>`.
    pub fn profile_ref(&self) -> Option<&str> {
        self.profile.as_deref()
    }

    /// Get API host override as `Option<&str>`.
    pub fn api_host_ref(&self) -> Option<&str> {
        self.api_host.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_options_accessors() {
        let opts = GlobalOptions {
            format: OutputFormat::Json,
            org: Some("org-123".to_string()),
            config: Some("/custom/path".to_string()),
            profile: Some("prod".to_string()),
            no_cache: true,
            api_host: Some("http://localhost:8080".to_string()),
        };

        assert_eq!(opts.org_ref(), Some("org-123"));
        assert_eq!(opts.config_ref(), Some("/custom/path"));
        assert_eq!(opts.profile_ref(), Some("prod"));
        assert_eq!(opts.api_host_ref(), Some("http://localhost:8080"));
        assert!(opts.no_cache);
    }

    #[test]
    fn test_global_options_none_accessors() {
        let opts = GlobalOptions {
            format: OutputFormat::Pretty,
            org: None,
            config: None,
            profile: None,
            no_cache: false,
            api_host: None,
        };

        assert_eq!(opts.org_ref(), None);
        assert_eq!(opts.config_ref(), None);
        assert_eq!(opts.profile_ref(), None);
        assert_eq!(opts.api_host_ref(), None);
        assert!(!opts.no_cache);
    }
}
