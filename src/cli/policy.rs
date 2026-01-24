//! Policy management commands

use log::debug;

use crate::cli::args::GlobalOptions;
use crate::cli::{CommandContext, PaginationArgs};
use crate::client::ListingApi;
use crate::error::Result;
use crate::models::PolicyDisplay;
use crate::output::Formattable;

/// Run the policy list command
///
/// Fetches both StackHawk preset policies and organization custom policies,
/// combining them into a single list with type indicators.
pub async fn list(opts: &GlobalOptions, pagination: &PaginationArgs) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;

    // Fetch both policy types in parallel
    debug!(
        "Fetching StackHawk policies and org policies for {}",
        org_id
    );

    let params = pagination.to_params();
    let (stackhawk_result, org_result) = tokio::join!(
        ctx.client.list_stackhawk_policies(),
        ctx.client.list_org_policies(org_id, Some(&params))
    );

    let stackhawk_policies = stackhawk_result?;
    let org_policies = org_result?;

    debug!(
        "Fetched {} StackHawk policies and {} org policies",
        stackhawk_policies.len(),
        org_policies.len()
    );

    // Convert to display format
    let mut display_policies: Vec<PolicyDisplay> = Vec::new();

    // Add StackHawk policies first
    for policy in stackhawk_policies {
        display_policies.push(PolicyDisplay::from_stackhawk(policy));
    }

    // Add organization policies
    for policy in org_policies {
        display_policies.push(PolicyDisplay::from_org(policy));
    }

    // Apply limit if specified
    let limited_policies = if let Some(limit) = pagination.limit {
        display_policies.into_iter().take(limit).collect()
    } else {
        display_policies
    };

    limited_policies.print(ctx.format)?;

    Ok(())
}
