//! OAS (OpenAPI specification) management commands

use crate::cli::handlers::run_list_command;
use crate::cli::{OutputFormat, PaginationArgs};
use crate::client::ListingApi;
use crate::client::models::OASAsset;
use crate::error::Result;
use crate::models::OASDisplay;

/// Run the oas list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    pagination: &PaginationArgs,
    no_cache: bool,
) -> Result<()> {
    run_list_command::<OASAsset, OASDisplay, _, _>(
        format,
        org_override,
        config_path,
        pagination,
        no_cache,
        "OAS assets",
        |client, org_id, params| async move { client.list_oas(&org_id, Some(&params)).await },
    )
    .await
}
