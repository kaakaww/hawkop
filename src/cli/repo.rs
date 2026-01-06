//! Repository management commands

use log::debug;

use crate::cli::{CommandContext, OutputFormat, PaginationArgs};
use crate::client::StackHawkApi;
use crate::error::Result;
use crate::models::RepoDisplay;
use crate::output::Formattable;

/// Run the repo list command
///
/// Fetches repositories from the organization's attack surface mapping.
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    pagination: &PaginationArgs,
    no_cache: bool,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path, no_cache).await?;
    let org_id = ctx.require_org_id()?;

    debug!("Fetching repositories for org {}", org_id);

    let params = pagination.to_params();
    let repos = ctx.client.list_repos(org_id, Some(&params)).await?;

    debug!("Fetched {} repositories", repos.len());

    // Convert to display format
    let display_repos: Vec<RepoDisplay> = repos.into_iter().map(RepoDisplay::from).collect();

    // Apply limit if specified
    let limited_repos = if let Some(limit) = pagination.limit {
        display_repos.into_iter().take(limit).collect()
    } else {
        display_repos
    };

    limited_repos.print(ctx.format)?;

    Ok(())
}
