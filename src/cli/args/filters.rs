//! Filter argument types for CLI commands

use clap::Args;

use super::SortDir;

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
