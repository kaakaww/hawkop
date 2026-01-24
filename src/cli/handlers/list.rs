//! Generic list command handler
//!
//! Provides a reusable pattern for list commands that follow the standard flow:
//! 1. Create command context
//! 2. Get org ID
//! 3. Fetch data with pagination
//! 4. Apply limit
//! 5. Convert to display type
//! 6. Print output

use std::future::Future;
use std::sync::Arc;

use log::debug;
use serde::Serialize;
use tabled::Tabled;

use crate::cache::CachedStackHawkClient;
use crate::cli::args::GlobalOptions;
use crate::cli::{CommandContext, PaginationArgs};
use crate::client::{PaginationParams, StackHawkClient};
use crate::error::Result;
use crate::output::Formattable;

/// Run a standard list command with the common fetch → limit → display → print pattern.
///
/// This eliminates boilerplate across list commands like user, team, repo, oas, and config.
///
/// # Type Parameters
///
/// * `T` - The API model type returned by the fetcher (e.g., `User`, `Team`)
/// * `D` - The display type that implements `From<T>`, `Tabled`, and `Serialize`
/// * `Fut` - The future type returned by the fetcher
///
/// # Arguments
///
/// * `opts` - Global CLI options (format, org override, config path, etc.)
/// * `pagination` - Pagination arguments from CLI
/// * `resource_name` - Name for debug logging (e.g., "users", "teams")
/// * `fetcher` - Async function that fetches the data given (client, org_id, params)
///
/// # Example
///
/// ```ignore
/// run_list_command::<User, UserDisplay, _, _>(
///     opts,
///     pagination,
///     "users",
///     |client, org_id, params| async move {
///         client.list_users(&org_id, Some(&params)).await
///     },
/// ).await
/// ```
pub async fn run_list_command<T, D, Fut, F>(
    opts: &GlobalOptions,
    pagination: &PaginationArgs,
    resource_name: &str,
    fetcher: F,
) -> Result<()>
where
    T: 'static,
    D: From<T> + Tabled + Serialize,
    Fut: Future<Output = Result<Vec<T>>>,
    F: FnOnce(Arc<CachedStackHawkClient<StackHawkClient>>, String, PaginationParams) -> Fut,
{
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;

    debug!("Fetching {} for org {}", resource_name, org_id);

    let params = pagination.to_params();
    let items = fetcher(ctx.client.clone(), org_id.to_string(), params).await?;

    debug!("Fetched {} {}", items.len(), resource_name);

    // Apply limit if specified
    let limited_items: Vec<T> = if let Some(limit) = pagination.limit {
        items.into_iter().take(limit).collect()
    } else {
        items
    };

    // Convert to display type and print
    let display_items: Vec<D> = limited_items.into_iter().map(D::from).collect();
    display_items.print(ctx.format)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // Tests would require mocking CommandContext which is complex.
    // The individual command handlers that use this are tested through integration tests.
}
