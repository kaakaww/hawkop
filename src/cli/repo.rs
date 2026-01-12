//! Repository management commands

use crate::cli::handlers::run_list_command;
use crate::cli::{OutputFormat, PaginationArgs};
use crate::client::ListingApi;
use crate::client::models::Repository;
use crate::error::Result;
use crate::models::RepoDisplay;

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
    run_list_command::<Repository, RepoDisplay, _, _>(
        format,
        org_override,
        config_path,
        pagination,
        no_cache,
        "repositories",
        |client, org_id, params| async move { client.list_repos(&org_id, Some(&params)).await },
    )
    .await
}
