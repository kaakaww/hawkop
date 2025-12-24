//! CLI command definitions and handlers

use clap::{Args, Parser, Subcommand};

use crate::client::PaginationParams;

pub mod app;
pub mod context;
pub mod init;
pub mod org;
pub mod scan;
pub mod status;

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
