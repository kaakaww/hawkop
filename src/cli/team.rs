//! Team management commands

use log::debug;

use crate::cli::{CommandContext, OutputFormat, PaginationArgs};
use crate::client::StackHawkApi;
use crate::error::Result;
use crate::models::TeamDisplay;
use crate::output::Formattable;

/// Run the team list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    pagination: &PaginationArgs,
    no_cache: bool,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path, no_cache).await?;
    let org_id = ctx.require_org_id()?;

    debug!("Fetching teams for org {}", org_id);

    let params = pagination.to_params();
    let teams = ctx.client.list_teams(org_id, Some(&params)).await?;

    debug!("Fetched {} teams", teams.len());

    // Apply limit if specified
    let limited_teams = if let Some(limit) = pagination.limit {
        teams.into_iter().take(limit).collect()
    } else {
        teams
    };

    let display_teams: Vec<TeamDisplay> =
        limited_teams.into_iter().map(TeamDisplay::from).collect();
    display_teams.print(ctx.format)?;

    Ok(())
}
