//! OAS (OpenAPI specification) management commands

use crate::cli::{CommandContext, OutputFormat, PaginationArgs};
use crate::client::StackHawkApi;
use crate::error::Result;
use crate::models::OASDisplay;
use crate::output::Formattable;

/// Run the oas list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    pagination: &PaginationArgs,
    no_cache: bool,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path, no_cache).await?;
    let org_id = ctx.require_org_id()?;

    let params = pagination.to_params();
    let oas_assets = ctx.client.list_oas(org_id, Some(&params)).await?;

    // Apply limit if specified
    let limited_assets = if let Some(limit) = pagination.limit {
        oas_assets.into_iter().take(limit).collect()
    } else {
        oas_assets
    };

    let display_assets: Vec<OASDisplay> =
        limited_assets.into_iter().map(OASDisplay::from).collect();
    display_assets.print(ctx.format)?;

    Ok(())
}
