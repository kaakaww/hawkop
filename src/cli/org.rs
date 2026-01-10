//! Organization command implementations

use colored::Colorize;

use crate::cli::{CommandContext, OutputFormat};
use crate::client::ListingApi;
use crate::config::Config;
use crate::error::Result;
use crate::models::OrgDisplay;
use crate::output::{Formattable, json};

/// Run the org list command
pub async fn list(format: OutputFormat, config_path: Option<&str>, no_cache: bool) -> Result<()> {
    let ctx = CommandContext::new(format, None, config_path, no_cache).await?;
    let orgs = ctx.client.list_orgs().await?;

    let display_orgs: Vec<OrgDisplay> = orgs.into_iter().map(OrgDisplay::from).collect();
    display_orgs.print(ctx.format)?;

    Ok(())
}

/// Run the org set command
pub async fn set(org_id: String, config_path: Option<&str>, no_cache: bool) -> Result<()> {
    // We need the resolved path for saving, so get it first
    let resolved_path = Config::resolve_path(config_path)?;

    // Use CommandContext for client initialization
    let mut ctx = CommandContext::new(OutputFormat::Table, None, config_path, no_cache).await?;

    println!("Verifying organization...");

    // Get all orgs and verify the provided org_id exists
    let orgs = ctx.client.list_orgs().await?;
    let org = orgs.iter().find(|o| o.id == org_id).ok_or_else(|| {
        crate::error::ApiError::NotFound(format!(
            "Organization {} not found or you don't have access to it",
            org_id
        ))
    })?;

    // Update config and save
    ctx.config.org_id = Some(org_id.clone());
    ctx.config.save_to(resolved_path)?;

    println!(
        "{} Set default organization to: {} ({})",
        "✓".green(),
        org.name.bold(),
        org_id
    );

    Ok(())
}

/// Run the org get command
pub async fn get(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
    no_cache: bool,
) -> Result<()> {
    let ctx = CommandContext::new(format, org_override, config_path, no_cache).await?;
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
