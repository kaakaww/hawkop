//! Repository management commands

use crate::cli::PaginationArgs;
use crate::cli::args::GlobalOptions;
use crate::cli::handlers::run_list_command;
use crate::client::ListingApi;
use crate::client::models::Repository;
use crate::error::Result;
use crate::models::RepoDisplay;

/// Run the repo list command
///
/// Fetches repositories from the organization's attack surface mapping.
pub async fn list(opts: &GlobalOptions, pagination: &PaginationArgs) -> Result<()> {
    run_list_command::<Repository, RepoDisplay, _, _>(
        opts,
        pagination,
        "repositories",
        |client, org_id, params| async move { client.list_repos(&org_id, Some(&params)).await },
    )
    .await
}
