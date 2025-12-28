//! CLI command definitions and handlers

use clap::{Args, Parser, Subcommand};
pub use clap_complete::Shell;

use crate::client::PaginationParams;

pub mod app;
pub mod audit;
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

    /// Generate shell completions
    #[command(after_help = "\
Setup:
  bash:
    hawkop completion bash > /etc/bash_completion.d/hawkop
    # Or for user install:
    hawkop completion bash >> ~/.bashrc

  zsh:
    hawkop completion zsh > \"${fpath[1]}/_hawkop\"
    # Or add to ~/.zshrc:
    eval \"$(hawkop completion zsh)\"

  fish:
    hawkop completion fish > ~/.config/fish/completions/hawkop.fish

  powershell:
    # Add to $PROFILE:
    hawkop completion powershell | Out-String | Invoke-Expression")]
    Completion {
        /// Shell to generate completions for
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

    /// View scan details and drill down into findings
    #[command(
        name = "view",
        visible_alias = "v",
        after_help = "DRILL-DOWN NAVIGATION:\n  \
            scan view <id>                                  Show scan overview\n  \
            scan view <id> alerts                           List all alerts\n  \
            scan view <id> alert <plugin>                   Alert detail with paths\n  \
            scan view <id> alert <plugin> uri <uri-id>      URI detail with evidence\n  \
            scan view <id> alert <plugin> uri <uri-id> message  HTTP request/response"
    )]
    View {
        /// Scan ID (UUID)
        scan_id: String,

        /// Drill-down: alerts | alert <plugin> [uri <uri-id>] [message]
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
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

/// Filter arguments for audit list command.
#[derive(Args, Debug, Clone)]
pub struct AuditFilterArgs {
    /// Filter by activity type
    #[arg(long = "type", short = 't', value_delimiter = ',')]
    pub activity_type: Vec<String>,

    /// Filter by org activity type
    #[arg(long = "org-type", value_delimiter = ',')]
    pub org_type: Vec<String>,

    /// Filter by user name
    #[arg(long, short = 'u')]
    pub user: Option<String>,

    /// Filter by user email
    #[arg(long)]
    pub email: Option<String>,

    /// Start date (ISO or relative: 7d, 30d)
    #[arg(long)]
    pub since: Option<String>,

    /// End date (ISO or relative: 7d, 30d)
    #[arg(long)]
    pub until: Option<String>,

    /// Sort direction (asc, desc)
    #[arg(
        long,
        value_enum,
        default_value = "desc",
        hide_possible_values = true,
        hide_default_value = true
    )]
    pub sort_dir: SortDir,

    /// Maximum results to return
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,
}

/// Filter arguments for scan list command.
///
/// Supports both repeated flags and comma-separated values:
/// - `--app app1 --app app2` (repeated)
/// - `--app app1,app2` (comma-separated)
/// - `--app app1 --app app2,app3` (mixed)
#[derive(Args, Debug, Default, Clone)]
pub struct ScanFilterArgs {
    /// Filter by application ID
    #[arg(long, short = 'a', value_delimiter = ',')]
    pub app: Vec<String>,

    /// Filter by environment
    #[arg(long, short = 'e', value_delimiter = ',')]
    pub env: Vec<String>,

    /// Filter by status (running, complete, failed)
    #[arg(long, short = 's')]
    pub status: Option<String>,
}

/// Shared pagination arguments for list commands.
///
/// Flatten this into any command that supports pagination:
/// ```ignore
/// List {
///     #[command(flatten)]
///     pagination: PaginationArgs,
/// }
/// ```
#[derive(Args, Debug, Default, Clone)]
pub struct PaginationArgs {
    /// Maximum results to return
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,

    /// Page number (0-indexed)
    #[arg(long, short = 'p')]
    pub page: Option<usize>,

    /// Field to sort by
    #[arg(long)]
    pub sort_by: Option<String>,

    /// Sort direction (asc, desc)
    #[arg(long, value_enum, hide_possible_values = true)]
    pub sort_dir: Option<SortDir>,
}

/// Sort direction for list commands
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SortDir {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

impl PaginationArgs {
    /// Convert CLI args to API pagination params.
    ///
    /// Always returns params with sensible defaults (max page size)
    /// to minimize API calls.
    pub fn to_params(&self) -> PaginationParams {
        let mut params = PaginationParams::new();

        if let Some(limit) = self.limit {
            params = params.page_size(limit);
        }
        // If no limit specified, PaginationParams defaults to MAX_PAGE_SIZE

        if let Some(page) = self.page {
            params = params.page(page);
        }
        if let Some(ref field) = self.sort_by {
            params = params.sort_by(field);
        }
        if let Some(dir) = self.sort_dir {
            use crate::client::pagination::SortOrder;
            let order = match dir {
                SortDir::Asc => SortOrder::Asc,
                SortDir::Desc => SortOrder::Desc,
            };
            params = params.sort_order(order);
        }

        params
    }
}

/// Output format options
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    /// Table format
    Table,
    /// JSON format
    Json,
}
