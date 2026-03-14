//! Environment management commands
//!
//! Provides list, config, create, and delete operations for application environments.

use std::fs;
use std::sync::Arc;

use colored::Colorize;
use dialoguer::Confirm;

use crate::cache::CachedStackHawkClient;
use crate::cli::args::GlobalOptions;
use crate::cli::{CommandContext, OutputFormat, PaginationArgs};
use crate::client::models::{Application, Environment};
use crate::client::{EnvironmentApi, ListingApi, StackHawkClient};
use crate::error::Result;
use crate::models::EnvDisplay;
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

/// Resolve an environment identifier (name or UUID) to an Environment
async fn resolve_env(client: &Client, app_id: &str, identifier: &str) -> Result<Environment> {
    let envs = client.list_environments(app_id, None).await?;

    // If it looks like a UUID, try to find by ID
    if looks_like_uuid(identifier) {
        if let Some(env) = envs.into_iter().find(|e| e.environment_id == identifier) {
            return Ok(env);
        }
        return Err(crate::error::ApiError::NotFound(format!(
            "Environment not found: {}",
            identifier
        ))
        .into());
    }

    // Otherwise, search by name (case-insensitive)
    let matches: Vec<_> = envs
        .into_iter()
        .filter(|e| e.environment_name.eq_ignore_ascii_case(identifier))
        .collect();

    match matches.len() {
        0 => Err(crate::error::ApiError::NotFound(format!(
            "Environment not found: {}. Use `hawkop env list --app <app>` to see available environments.",
            identifier
        ))
        .into()),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => {
            // Multiple matches - show them and ask user to be more specific
            let ids: Vec<String> = matches.iter().map(|e| e.environment_id.clone()).collect();
            Err(crate::error::ApiError::BadRequest(format!(
                "Multiple environments match '{}'. Please use the environment ID:\n{}",
                identifier,
                ids.join("\n")
            ))
            .into())
        }
    }
}

// ============================================================================
// List Command
// ============================================================================

/// List environments for an application
pub async fn list(opts: &GlobalOptions, app: &str, pagination: &PaginationArgs) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;
    let client = ctx.client.clone();

    // Resolve app name/ID to full application
    let application = resolve_app(&client, org_id, app).await?;
    let app_id = &application.id;

    let params = pagination.to_params();
    let envs = client.list_environments(app_id, Some(&params)).await?;

    if envs.is_empty() {
        eprintln!(
            "{} No environments found for application '{}'",
            "Info:".blue(),
            application.name
        );
        eprintln!(
            "{}",
            format!("→ Use `hawkop env create --app {}` to create one", app).dimmed()
        );
        return Ok(());
    }

    match opts.format {
        OutputFormat::Json => {
            let displays: Vec<EnvDisplay> = envs.iter().map(EnvDisplay::from).collect();
            let json = format_json(&displays)?;
            println!("{}", json);
        }
        OutputFormat::Table | OutputFormat::Pretty => {
            let displays: Vec<EnvDisplay> = envs.iter().map(EnvDisplay::from).collect();
            let table = format_table(&displays);
            println!("{}", table);
        }
    }

    Ok(())
}

// ============================================================================
// Config Command
// ============================================================================

/// Get the default YAML configuration for an environment
pub async fn config(
    opts: &GlobalOptions,
    app: &str,
    env: &str,
    output: Option<&str>,
) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;
    let client = ctx.client.clone();

    // Resolve app and environment
    let application = resolve_app(&client, org_id, app).await?;
    let environment = resolve_env(&client, &application.id, env).await?;

    eprintln!(
        "{} Fetching default config for '{}/{}'...",
        "→".blue(),
        application.name,
        environment.environment_name
    );

    let content = client
        .get_environment_default_config(&application.id, &environment.environment_id)
        .await?;

    match output {
        Some(path) => {
            // Write to file
            fs::write(path, &content)?;
            eprintln!("{} Default configuration saved to {}", "✓".green(), path);
        }
        None => {
            // Print to stdout
            match opts.format {
                OutputFormat::Json => {
                    let wrapper = serde_json::json!({
                        "app": application.name,
                        "env": environment.environment_name,
                        "content": content
                    });
                    println!("{}", serde_json::to_string_pretty(&wrapper)?);
                }
                _ => {
                    println!("{}", content);
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Create Command
// ============================================================================

/// Create a new environment for an application
pub async fn create(opts: &GlobalOptions, app: &str, name: &str) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;
    let client = ctx.client.clone();

    // Resolve app name/ID
    let application = resolve_app(&client, org_id, app).await?;

    eprintln!(
        "{} Creating environment '{}' for '{}'...",
        "→".blue(),
        name,
        application.name
    );

    client.create_environment(&application.id, name).await?;

    eprintln!(
        "{} Environment '{}' created for '{}'",
        "✓".green(),
        name,
        application.name
    );
    eprintln!(
        "{}",
        format!(
            "→ Use `hawkop env config --app {} {}` to get a starter config",
            app, name
        )
        .dimmed()
    );

    Ok(())
}

// ============================================================================
// Delete Command
// ============================================================================

/// Delete an environment
pub async fn delete(opts: &GlobalOptions, app: &str, env: &str, yes: bool) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;
    let client = ctx.client.clone();

    // Resolve app and environment
    let application = resolve_app(&client, org_id, app).await?;
    let environment = resolve_env(&client, &application.id, env).await?;

    // Confirm unless --yes (extra warning about scan results)
    if !yes {
        eprintln!(
            "{} Deleting environment '{}' will also {}.",
            "Warning:".yellow().bold(),
            environment.environment_name,
            "permanently delete all scan results".red().bold()
        );
        eprintln!();

        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Delete environment '{}'? This cannot be undone.",
                environment.environment_name
            ))
            .default(false)
            .interact()?;

        if !confirmed {
            eprintln!("{}", "Cancelled".yellow());
            return Ok(());
        }
    }

    eprintln!(
        "{} Deleting environment '{}'...",
        "→".blue(),
        environment.environment_name
    );

    client
        .delete_environment(&application.id, &environment.environment_id)
        .await?;

    eprintln!(
        "{} Environment '{}' deleted",
        "✓".green(),
        environment.environment_name
    );

    Ok(())
}
