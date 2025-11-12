//! CLI command definitions and handlers

use clap::{Parser, Subcommand};

pub mod app;
pub mod init;
pub mod org;
pub mod status;

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
    List,
}

/// Output format options
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table format
    Table,
    /// JSON format for scripting
    Json,
}
