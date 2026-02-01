//! OAS (OpenAPI specification) management commands

use std::fs;
use std::sync::Arc;

use colored::Colorize;

use crate::cache::CachedStackHawkClient;
use crate::cli::PaginationArgs;
use crate::cli::args::GlobalOptions;
use crate::cli::handlers::run_list_command;
use crate::cli::{CommandContext, OutputFormat};
use crate::client::models::{Application, OASAsset};
use crate::client::{ListingApi, OASApi, StackHawkClient};
use crate::error::Result;
use crate::models::OASDisplay;
use crate::output::json::format_json;
use crate::output::table::format_table;

/// Type alias for the Arc-wrapped cached client
type Client = Arc<CachedStackHawkClient<StackHawkClient>>;

/// Simple UUID format check (8-4-4-4-12 hex pattern)
fn looks_like_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 5
        && parts
            .iter()
            .all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
}

/// Resolve an app identifier (name or UUID) to an Application
async fn resolve_app(client: &Client, org_id: &str, identifier: &str) -> Result<Application> {
    // If it looks like a UUID, try to find by ID
    if looks_like_uuid(identifier) {
        let apps = client.list_apps(org_id, None).await?;
        if let Some(app) = apps.into_iter().find(|a| a.id == identifier) {
            return Ok(app);
        }
        return Err(crate::error::ApiError::NotFound(format!(
            "Application not found: {}",
            identifier
        ))
        .into());
    }

    // Otherwise, search by name (case-insensitive)
    let apps = client.list_apps(org_id, None).await?;
    let matches: Vec<_> = apps
        .into_iter()
        .filter(|a| a.name.eq_ignore_ascii_case(identifier))
        .collect();

    match matches.len() {
        0 => Err(crate::error::ApiError::NotFound(format!(
            "Application not found: {}. Use `hawkop app list` to see available applications.",
            identifier
        ))
        .into()),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => {
            // Multiple matches - show them and ask user to be more specific
            let ids: Vec<String> = matches.iter().map(|a| a.id.clone()).collect();
            Err(crate::error::ApiError::BadRequest(format!(
                "Multiple applications match '{}'. Please use the application ID:\n{}",
                identifier,
                ids.join("\n")
            ))
            .into())
        }
    }
}

/// Run the oas list command
pub async fn list(opts: &GlobalOptions, pagination: &PaginationArgs) -> Result<()> {
    run_list_command::<OASAsset, OASDisplay, _, _>(
        opts,
        pagination,
        "OAS assets",
        |client, org_id, params| async move { client.list_oas(&org_id, Some(&params)).await },
    )
    .await
}

// ============================================================================
// Get Command
// ============================================================================

/// Get the content of an OpenAPI specification
pub async fn get(opts: &GlobalOptions, oas_id: &str, output: Option<&str>) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;
    let client = ctx.client.clone();

    eprintln!("{} Fetching OAS '{}'...", "→".blue(), oas_id);

    let content = client.get_oas(org_id, oas_id).await?;

    match output {
        Some(path) => {
            // Write to file
            fs::write(path, &content)?;
            eprintln!("{} OpenAPI specification saved to {}", "✓".green(), path);
        }
        None => {
            // Print to stdout
            match opts.format {
                OutputFormat::Json => {
                    // Content is already JSON, just print it
                    println!("{}", content);
                }
                _ => {
                    // Pretty-print the JSON
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                        println!("{}", serde_json::to_string_pretty(&parsed)?);
                    } else {
                        println!("{}", content);
                    }
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Mappings Command
// ============================================================================

/// List OpenAPI specs mapped to an application
pub async fn mappings(opts: &GlobalOptions, app: &str) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;
    let client = ctx.client.clone();

    // Resolve app name/ID to full application
    let application = resolve_app(&client, org_id, app).await?;

    let oas_assets = client.get_oas_mappings(&application.id).await?;

    if oas_assets.is_empty() {
        eprintln!(
            "{} No OpenAPI specs mapped to application '{}'",
            "Info:".blue(),
            application.name
        );
        eprintln!(
            "{}",
            "→ Map OAS specs to your app in the StackHawk UI".dimmed()
        );
        return Ok(());
    }

    match opts.format {
        OutputFormat::Json => {
            let displays: Vec<OASDisplay> = oas_assets.iter().map(OASDisplay::from).collect();
            let json = format_json(&displays)?;
            println!("{}", json);
        }
        OutputFormat::Table | OutputFormat::Pretty => {
            let displays: Vec<OASDisplay> = oas_assets.iter().map(OASDisplay::from).collect();
            let table = format_table(&displays);
            println!("{}", table);
        }
    }

    Ok(())
}
