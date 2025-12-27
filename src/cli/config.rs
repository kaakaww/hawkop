//! Scan configuration management commands

use crate::cli::{CommandContext, OutputFormat, PaginationArgs};
use crate::client::StackHawkApi;
use crate::error::Result;
use crate::models::ConfigDisplay;
use crate::output::Formattable;

/// Run the config list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    pagination: &PaginationArgs,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path).await?;
    let org_id = ctx.require_org_id()?;

    let params = pagination.to_params();
    let configs = ctx.client.list_scan_configs(org_id, Some(&params)).await?;

    // Apply limit if specified
    let limited_configs = if let Some(limit) = pagination.limit {
        configs.into_iter().take(limit).collect()
    } else {
        configs
    };

    let display_configs: Vec<ConfigDisplay> =
        limited_configs.into_iter().map(ConfigDisplay::from).collect();
    display_configs.print(ctx.format)?;

    Ok(())
}
