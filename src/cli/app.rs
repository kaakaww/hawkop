//! Application management commands

use crate::cli::{CommandContext, OutputFormat, PaginationArgs};
use crate::client::StackHawkApi;
use crate::error::Result;
use crate::models::AppDisplay;
use crate::output::Formattable;

/// Run the app list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    pagination: &PaginationArgs,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path).await?;
    let org_id = ctx.require_org_id()?;

    let pagination_params = pagination.to_params();
    let apps = ctx
        .client
        .list_apps(org_id, Some(&pagination_params))
        .await?;

    let display_apps: Vec<AppDisplay> = apps.into_iter().map(AppDisplay::from).collect();
    display_apps.print(ctx.format)?;

    Ok(())
}
