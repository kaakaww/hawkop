//! CLI command definitions and handlers

use clap::{Parser, Subcommand};
pub use clap_complete::Shell;

use completions::{
    app_name_candidates, plugin_id_candidates, scan_id_candidates, uri_id_candidates,
};

pub mod app;
pub mod args;
pub mod audit;
pub mod cache;
pub mod completions;
pub mod config;
pub mod context;
pub mod init;
pub mod oas;
pub mod org;
pub mod policy;
pub mod repo;
pub mod scan;
pub mod secret;
pub mod status;
pub mod team;
pub mod user;

pub use args::{AuditFilterArgs, OutputFormat, PaginationArgs, ScanFilterArgs, SortDir};
pub use context::CommandContext;

/// HawkOp CLI - Professional companion for the StackHawk DAST platform
#[derive(Parser, Debug)]
#[command(name = "hawkop")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Output format (table, json)
    #[arg(
        long,
        global = true,
        env = "HAWKOP_FORMAT",
        default_value = "table",
        hide_env = true,
        hide_possible_values = true
    )]
    pub format: OutputFormat,

    /// Override default organization
    #[arg(long, global = true, env = "HAWKOP_ORG_ID", hide_env = true)]
    pub org: Option<String>,

    /// Override config file location
    #[arg(long, global = true, env = "HAWKOP_CONFIG", hide_env = true)]
    pub config: Option<String>,

    /// Enable debug logging
    #[arg(long, global = true, env = "HAWKOP_DEBUG", hide_env = true)]
    pub debug: bool,

    /// Bypass cache, fetch fresh data from API
    #[arg(long, global = true, env = "HAWKOP_NO_CACHE", hide_env = true)]
    pub no_cache: bool,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize HawkOp configuration
    Init,

    /// Show authentication and configuration status
    Status,

    /// Display version information
    Version,

    /// Manage organizations
    #[command(subcommand)]
    Org(OrgCommands),

    /// Manage applications
    #[command(subcommand)]
    App(AppCommands),

    /// View and manage scans
    #[command(subcommand)]
    Scan(ScanCommands),

    /// List organization users/members
    #[command(subcommand)]
    User(UserCommands),

    /// List organization teams
    #[command(subcommand)]
    Team(TeamCommands),

    /// Manage scan policies
    #[command(subcommand)]
    Policy(PolicyCommands),

    /// List repositories in attack surface
    #[command(subcommand)]
    Repo(RepoCommands),

    /// List hosted OpenAPI specifications
    #[command(subcommand)]
    Oas(OasCommands),

    /// List scan configurations
    #[command(subcommand)]
    Config(ConfigCommands),

    /// List user secrets
    #[command(subcommand)]
    Secret(SecretCommands),

    /// View organization audit log
    #[command(subcommand)]
    Audit(AuditCommands),

    /// Manage local response cache
    #[command(subcommand)]
    Cache(CacheCommands),

    /// Generate shell completions (static)
    #[command(after_help = "\
Static completions (subcommands/flags only):
  bash:   hawkop completion bash > /etc/bash_completion.d/hawkop
  zsh:    hawkop completion zsh > \"${fpath[1]}/_hawkop\"
  fish:   hawkop completion fish > ~/.config/fish/completions/hawkop.fish

Dynamic completions (includes scan IDs, app names via API):
  bash:   echo 'source <(COMPLETE=bash hawkop)' >> ~/.bashrc
  zsh:    echo 'source <(COMPLETE=zsh hawkop)' >> ~/.zshrc
  fish:   echo 'COMPLETE=fish hawkop | source' >> ~/.config/fish/config.fish

Note: Dynamic completions query the StackHawk API when you press TAB.
Re-source completions after upgrading hawkop.")]
    Completion {
        /// Shell to generate completions for (static only)
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Organization management subcommands
#[derive(Subcommand, Debug)]
pub enum OrgCommands {
    /// List all accessible organizations
    List,

    /// Set default organization
    Set {
        /// Organization ID to set as default
        org_id: String,
    },

    /// Show current default organization
    Get,
}

/// Application management subcommands
#[derive(Subcommand, Debug)]
pub enum AppCommands {
    /// List all applications in the current organization
    List {
        /// Filter by application type (cloud, standard)
        #[arg(long = "type", short = 't')]
        app_type: Option<String>,

        #[command(flatten)]
        pagination: PaginationArgs,
    },
}

/// Scan management subcommands
#[derive(Subcommand, Debug)]
pub enum ScanCommands {
    /// List recent scans across all applications
    List {
        #[command(flatten)]
        filters: ScanFilterArgs,

        #[command(flatten)]
        pagination: PaginationArgs,
    },

    /// Get scan details with optional drill-down
    #[command(
        visible_alias = "g",
        after_help = "EXAMPLES:\n  \
            hawkop scan get                          # Latest scan (overview + alerts)\n  \
            hawkop scan get --app myapp              # Latest for app (by name)\n  \
            hawkop scan get --app-id <uuid>          # Latest for app (by ID)\n  \
            hawkop scan get abc123                   # Specific scan\n  \
            hawkop scan get abc123 --plugin-id 40012 # Plugin detail\n  \
            hawkop scan get abc123 --uri-id xyz -m   # Finding with HTTP message"
    )]
    Get {
        /// Scan ID (UUID) or "latest" - defaults to latest if omitted
        #[arg(default_value = "latest", add = scan_id_candidates())]
        scan_id: String,

        /// Filter by application name (only with "latest")
        #[arg(long, short = 'a', conflicts_with = "app_id", add = app_name_candidates())]
        app: Option<String>,

        /// Filter by application ID (only with "latest")
        #[arg(long = "app-id")]
        app_id: Option<String>,

        /// Filter by environment (only with "latest")
        #[arg(long, short = 'e')]
        env: Option<String>,

        /// Show detail for specific plugin/vulnerability type
        #[arg(long = "plugin-id", short = 'p', add = plugin_id_candidates())]
        plugin_id: Option<String>,

        /// Show detail for specific URI/finding (unique within scan)
        #[arg(long = "uri-id", short = 'u', add = uri_id_candidates())]
        uri_id: Option<String>,

        /// Include HTTP request/response (requires --uri-id)
        #[arg(long, short = 'm', requires = "uri_id")]
        message: bool,

        /// Output format: pretty (default), table, json
        #[arg(long, short = 'o', default_value = "pretty")]
        format: OutputFormat,
    },
}

/// User management subcommands
#[derive(Subcommand, Debug)]
pub enum UserCommands {
    /// List organization members
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
}

/// Team management subcommands
#[derive(Subcommand, Debug)]
pub enum TeamCommands {
    /// List organization teams
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
}

/// Policy management subcommands
#[derive(Subcommand, Debug)]
pub enum PolicyCommands {
    /// List scan policies (StackHawk preset and organization custom)
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
}

/// Repository management subcommands
#[derive(Subcommand, Debug)]
pub enum RepoCommands {
    /// List repositories in the organization's attack surface
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
}

/// OAS management subcommands
#[derive(Subcommand, Debug)]
pub enum OasCommands {
    /// List hosted OpenAPI specifications
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
}

/// Configuration management subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// List scan configurations
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
}

/// Secret management subcommands
#[derive(Subcommand, Debug)]
pub enum SecretCommands {
    /// List user secrets (names only)
    List,
}

/// Cache management subcommands
#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// Show cache statistics
    Status,
    /// Clear all cached data
    Clear,
    /// Print cache directory path
    Path,
}

/// Audit log subcommands
#[derive(Subcommand, Debug)]
pub enum AuditCommands {
    /// List audit log entries
    #[command(after_help = "\
Examples:
  hawkop audit list --type SCAN_STARTED,SCAN_COMPLETED --since 7d
  hawkop audit list --since 2024-12-01 --until 2024-12-31
  hawkop audit list --org-type EXTERNAL_ALERTS_SENT,ORGANIZATION_CREATED")]
    List {
        #[command(flatten)]
        filters: AuditFilterArgs,
    },
}
