//! CLI command definitions and handlers

use clap::{Args, Parser, Subcommand};

use crate::client::PaginationParams;

pub mod app;
pub mod context;
pub mod init;
pub mod org;
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

    /// Output format
    #[arg(long, global = true, env = "HAWKOP_FORMAT", default_value = "table")]
    pub format: OutputFormat,

    /// Override default organization
    #[arg(long, global = true, env = "HAWKOP_ORG_ID")]
    pub org: Option<String>,

    /// Override config file location
    #[arg(long, global = true, env = "HAWKOP_CONFIG")]
    pub config: Option<String>,

    /// Enable debug logging
    #[arg(long, global = true, env = "HAWKOP_DEBUG")]
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
    /// Maximum number of results to return (max 1000)
    #[arg(long, short = 'n')]
    pub limit: Option<usize>,

    /// Page number (0-indexed)
    #[arg(long, short = 'p')]
    pub page: Option<usize>,

    /// Field to sort by
    #[arg(long)]
    pub sort_by: Option<String>,

    /// Sort direction
    #[arg(long, value_enum)]
    pub sort_dir: Option<SortDir>,
}

/// Sort direction for list commands
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SortDir {
    /// Ascending order (A-Z, 0-9, oldest first)
    Asc,
    /// Descending order (Z-A, 9-0, newest first)
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
    /// Human-readable table format
    Table,
    /// JSON format for scripting
    Json,
}
