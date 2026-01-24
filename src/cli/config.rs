//! Scan configuration management commands

use crate::cli::PaginationArgs;
use crate::cli::args::GlobalOptions;
use crate::cli::handlers::run_list_command;
use crate::client::ListingApi;
use crate::client::models::ScanConfig;
use crate::error::Result;
use crate::models::ConfigDisplay;

/// Run the config list command
pub async fn list(opts: &GlobalOptions, pagination: &PaginationArgs) -> Result<()> {
    run_list_command::<ScanConfig, ConfigDisplay, _, _>(
        opts,
        pagination,
        "scan configs",
        |client, org_id, params| async move {
            client.list_scan_configs(&org_id, Some(&params)).await
        },
    )
    .await
}
