//! User management commands

use crate::cli::PaginationArgs;
use crate::cli::args::GlobalOptions;
use crate::cli::handlers::run_list_command;
use crate::client::ListingApi;
use crate::client::models::User;
use crate::error::Result;
use crate::models::UserDisplay;

/// Run the user list command
pub async fn list(opts: &GlobalOptions, pagination: &PaginationArgs) -> Result<()> {
    run_list_command::<User, UserDisplay, _, _>(
        opts,
        pagination,
        "users",
        |client, org_id, params| async move { client.list_users(&org_id, Some(&params)).await },
    )
    .await
}
