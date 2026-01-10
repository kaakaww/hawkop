//! Pagination argument types for CLI commands

use clap::Args;

use crate::client::PaginationParams;
use crate::client::pagination::SortOrder;

use super::SortDir;

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
            let order = match dir {
                SortDir::Asc => SortOrder::Asc,
                SortDir::Desc => SortOrder::Desc,
            };
            params = params.sort_order(order);
        }

        params
    }
}
