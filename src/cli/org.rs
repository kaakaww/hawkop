//! Organization command implementations

use colored::Colorize;

use crate::cli::args::GlobalOptions;
use crate::cli::{CommandContext, OutputFormat};
use crate::client::ListingApi;
use crate::error::Result;
use crate::models::OrgDisplay;
use crate::output::{Formattable, json};

/// Run the org list command
pub async fn list(opts: &GlobalOptions) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let orgs = ctx.client.list_orgs().await?;

    let display_orgs: Vec<OrgDisplay> = orgs.into_iter().map(OrgDisplay::from).collect();
    display_orgs.print(ctx.format)?;

    Ok(())
}

/// Run the org set command
pub async fn set(opts: &GlobalOptions, org_id: String) -> Result<()> {
    // Use CommandContext for client initialization
    let mut ctx = CommandContext::new(opts).await?;

    println!("Verifying organization...");

    // Get all orgs and verify the provided org_id exists
    let orgs = ctx.client.list_orgs().await?;
    let org = orgs.iter().find(|o| o.id == org_id).ok_or_else(|| {
        crate::error::ApiError::NotFound(format!(
            "Organization {} not found or you don't have access to it",
            org_id
        ))
    })?;

    // Update the profile's org_id and save
    if let Ok(profile) = ctx.profiled_config.get_profile_mut(&ctx.profile_name) {
        profile.org_id = Some(org_id.clone());
    }
    ctx.save_config()?;

    println!(
        "{} Set default organization to: {} ({})",
        "✓".green(),
        org.name.bold(),
        org_id
    );

    Ok(())
}

/// Run the org get command
pub async fn get(opts: &GlobalOptions) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;

    // Get all orgs and find the one matching our configured org_id
    let orgs = ctx.client.list_orgs().await?;
    let org = orgs.iter().find(|o| o.id == org_id).ok_or_else(|| {
        crate::error::ApiError::NotFound(format!(
            "Organization {} not found or you don't have access to it",
            org_id
        ))
    })?;

    match ctx.format {
        OutputFormat::Pretty | OutputFormat::Table => {
            println!("{}", "Current Default Organization".bold());
            println!();
            println!("  ID:   {}", org.id);
            println!("  Name: {}", org.name);
        }
        OutputFormat::Json => {
            let output = json::format_json(&org)?;
            println!("{}", output);
        }
    }

    Ok(())
}
