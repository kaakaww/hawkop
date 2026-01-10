//! Common CLI types shared across commands

/// Sort direction for list commands
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SortDir {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

/// Output format options
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    /// Pretty format - human-optimized rich formatting
    Pretty,
    /// Table format - machine-parseable, one row per entry (global default)
    #[default]
    Table,
    /// JSON format - structured for scripts/APIs
    Json,
}
