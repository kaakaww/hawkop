//! OAS (OpenAPI specification) management commands

use crate::cli::PaginationArgs;
use crate::cli::args::GlobalOptions;
use crate::cli::handlers::run_list_command;
use crate::client::ListingApi;
use crate::client::models::OASAsset;
use crate::error::Result;
use crate::models::OASDisplay;

/// Run the oas list command
pub async fn list(opts: &GlobalOptions, pagination: &PaginationArgs) -> Result<()> {
    run_list_command::<OASAsset, OASDisplay, _, _>(
        opts,
        pagination,
        "OAS assets",
        |client, org_id, params| async move { client.list_oas(&org_id, Some(&params)).await },
    )
    .await
}
