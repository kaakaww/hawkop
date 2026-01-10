//! Team management commands

use crate::cli::handlers::run_list_command;
use crate::cli::{OutputFormat, PaginationArgs};
use crate::client::StackHawkApi;
use crate::client::models::Team;
use crate::error::Result;
use crate::models::TeamDisplay;

/// Run the team list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    pagination: &PaginationArgs,
    no_cache: bool,
) -> Result<()> {
    run_list_command::<Team, TeamDisplay, _, _>(
        format,
        org_override,
        config_path,
        pagination,
        no_cache,
        "teams",
        |client, org_id, params| async move { client.list_teams(&org_id, Some(&params)).await },
    )
    .await
}
