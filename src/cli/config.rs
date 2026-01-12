//! Scan configuration management commands

use crate::cli::handlers::run_list_command;
use crate::cli::{OutputFormat, PaginationArgs};
use crate::client::ListingApi;
use crate::client::models::ScanConfig;
use crate::error::Result;
use crate::models::ConfigDisplay;

/// Run the config list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    pagination: &PaginationArgs,
    no_cache: bool,
) -> Result<()> {
    run_list_command::<ScanConfig, ConfigDisplay, _, _>(
        format,
        org_override,
        config_path,
        pagination,
        no_cache,
        "scan configs",
        |client, org_id, params| async move {
            client.list_scan_configs(&org_id, Some(&params)).await
        },
    )
    .await
}
