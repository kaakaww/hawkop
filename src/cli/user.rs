//! User management commands

use log::debug;

use crate::cli::{CommandContext, OutputFormat, PaginationArgs};
use crate::client::StackHawkApi;
use crate::error::Result;
use crate::models::UserDisplay;
use crate::output::Formattable;

/// Run the user list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    pagination: &PaginationArgs,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path).await?;
    let org_id = ctx.require_org_id()?;

    debug!("Fetching users for org {}", org_id);

    let params = pagination.to_params();
    let users = ctx.client.list_users(org_id, Some(&params)).await?;

    debug!("Fetched {} users", users.len());

    // Apply limit if specified
    let limited_users = if let Some(limit) = pagination.limit {
        users.into_iter().take(limit).collect()
    } else {
        users
    };

    let display_users: Vec<UserDisplay> = limited_users.into_iter().map(UserDisplay::from).collect();
    display_users.print(ctx.format)?;

    Ok(())
}
