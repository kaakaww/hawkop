//! User management commands

use crate::cli::handlers::run_list_command;
use crate::cli::{OutputFormat, PaginationArgs};
use crate::client::StackHawkApi;
use crate::client::models::User;
use crate::error::Result;
use crate::models::UserDisplay;

/// Run the user list command
pub async fn list(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    pagination: &PaginationArgs,
    no_cache: bool,
) -> Result<()> {
    run_list_command::<User, UserDisplay, _, _>(
        format,
        org_override,
        config_path,
        pagination,
        no_cache,
        "users",
        |client, org_id, params| async move { client.list_users(&org_id, Some(&params)).await },
    )
    .await
}
