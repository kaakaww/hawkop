//! Scan configuration management commands
//!
//! Provides list, get, set, delete, rename, and validate operations for
//! organization scan configurations.

use std::fs;
use std::path::Path;

use colored::Colorize;
use dialoguer::Confirm;

use crate::cli::args::GlobalOptions;
use crate::cli::handlers::run_list_command;
use crate::cli::{CommandContext, OutputFormat, PaginationArgs};
use crate::client::models::{ConfigType, ScanConfig, ValidatedAssetResponse};
use crate::client::{ConfigApi, ListingApi};
use crate::error::Result;
use crate::models::ConfigDisplay;
use crate::output::json::format_json;

// ============================================================================
// List Command
// ============================================================================

/// Run the config list command
pub async fn list(opts: &GlobalOptions, pagination: &PaginationArgs) -> Result<()> {
    run_list_command::<ScanConfig, ConfigDisplay, _, _>(
        opts,
        pagination,
        "scan configs",
        |client, org_id, params| async move {
            client.list_scan_configs(&org_id, Some(&params)).await
        },
    )
    .await
}

// ============================================================================
// Get Command
// ============================================================================

/// Get and display a configuration's content
pub async fn get(opts: &GlobalOptions, name: &str, output: Option<&str>) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;

    eprintln!("{} Fetching configuration '{}'...", "→".blue(), name);

    let content = ctx.client.get_scan_config(org_id, name).await?;

    match output {
        Some(path) => {
            // Write to file
            fs::write(path, &content)?;
            eprintln!("{} Configuration '{}' saved to {}", "✓".green(), name, path);
        }
        None => {
            // Print to stdout
            match opts.format {
                OutputFormat::Json => {
                    // Wrap content in JSON structure
                    let wrapper = serde_json::json!({
                        "name": name,
                        "content": content
                    });
                    println!("{}", serde_json::to_string_pretty(&wrapper)?);
                }
                _ => {
                    // Print raw YAML content
                    println!("{}", content);
                }
            }
        }
    }

    Ok(())
}

// ============================================================================
// Set Command
// ============================================================================

/// Create or update a configuration from a file
pub async fn set(opts: &GlobalOptions, name: &str, file: &str) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;

    // Read the file content
    let path = Path::new(file);
    if !path.exists() {
        return Err(crate::error::Error::Other(format!(
            "File not found: {}",
            file
        )));
    }

    let content = fs::read_to_string(path)?;

    // Validate before uploading
    eprintln!(
        "{} Validating configuration '{}'...",
        "→".blue(),
        path.display()
    );

    let validation = ctx.client.validate_scan_config(org_id, &content).await?;

    if !validation.is_valid() {
        print_validation_results(&validation, Some(file));
        return Err(crate::error::Error::Other(
            "Configuration has validation errors. Fix errors before uploading.".to_string(),
        ));
    }

    // Show warnings if any
    if !validation.warnings().is_empty() {
        print_validation_results(&validation, Some(file));
        eprintln!();
    }

    // Upload the configuration
    eprintln!("{} Uploading configuration '{}'...", "→".blue(), name);

    ctx.client
        .set_scan_config(org_id, name, &content, ConfigType::Org)
        .await?;

    eprintln!(
        "{} Configuration '{}' created/updated successfully",
        "✓".green(),
        name
    );
    eprintln!(
        "{}",
        format!("→ Reference in stackhawk.yml with: hawk://{}", name).dimmed()
    );

    Ok(())
}

// ============================================================================
// Delete Command
// ============================================================================

/// Delete a configuration
pub async fn delete(opts: &GlobalOptions, name: &str, yes: bool) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;

    // Confirm unless --yes
    if !yes {
        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Delete configuration '{}'? This cannot be undone.",
                name
            ))
            .default(false)
            .interact()?;

        if !confirmed {
            eprintln!("{}", "Cancelled".yellow());
            return Ok(());
        }
    }

    eprintln!("{} Deleting configuration '{}'...", "→".blue(), name);

    ctx.client.delete_scan_config(org_id, name).await?;

    eprintln!("{} Configuration '{}' deleted", "✓".green(), name);

    Ok(())
}

// ============================================================================
// Rename Command
// ============================================================================

/// Rename a configuration
pub async fn rename(opts: &GlobalOptions, old_name: &str, new_name: &str) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;

    eprintln!(
        "{} Renaming configuration '{}' to '{}'...",
        "→".blue(),
        old_name,
        new_name
    );

    ctx.client
        .rename_scan_config(org_id, old_name, new_name)
        .await?;

    eprintln!(
        "{} Configuration renamed from '{}' to '{}'",
        "✓".green(),
        old_name,
        new_name
    );
    eprintln!(
        "{}",
        format!(
            "→ Update references from hawk://{} to hawk://{}",
            old_name, new_name
        )
        .dimmed()
    );

    Ok(())
}

// ============================================================================
// Validate Command
// ============================================================================

/// Validate a configuration (either stored or local file)
pub async fn validate(opts: &GlobalOptions, name: Option<&str>, file: Option<&str>) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;

    // Get content from either stored config or local file
    let (content, source) = match (name, file) {
        (Some(config_name), None) => {
            eprintln!("{} Fetching configuration '{}'...", "→".blue(), config_name);
            let content = ctx.client.get_scan_config(org_id, config_name).await?;
            (content, config_name.to_string())
        }
        (None, Some(file_path)) => {
            let path = Path::new(file_path);
            if !path.exists() {
                return Err(crate::error::Error::Other(format!(
                    "File not found: {}",
                    file_path
                )));
            }
            let content = fs::read_to_string(path)?;
            (content, file_path.to_string())
        }
        _ => {
            return Err(crate::error::Error::Other(
                "Specify either a configuration name or --file".to_string(),
            ));
        }
    };

    eprintln!("{} Validating '{}'...", "→".blue(), source);

    let validation = ctx.client.validate_scan_config(org_id, &content).await?;

    match opts.format {
        OutputFormat::Json => {
            let json = format_json(&validation)?;
            println!("{}", json);
        }
        _ => {
            print_validation_results(&validation, Some(&source));
        }
    }

    // Return error if validation failed
    if !validation.is_valid() {
        return Err(crate::error::Error::Other(
            "Configuration validation failed".to_string(),
        ));
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Print validation results in a human-readable format
fn print_validation_results(validation: &ValidatedAssetResponse, source: Option<&str>) {
    let errors = validation.errors();
    let warnings = validation.warnings();

    if errors.is_empty() && warnings.is_empty() {
        eprintln!("{} Configuration is valid", "✓".green());
        return;
    }

    let source_prefix = source.map(|s| format!("{}:", s)).unwrap_or_default();

    // Print errors
    for marker in &errors {
        let location = marker.location();
        let message = marker.message.as_deref().unwrap_or("Unknown error");

        if location.is_empty() {
            eprintln!("{} {}", "error:".red().bold(), message);
        } else {
            eprintln!(
                "{}{} {} {}",
                source_prefix,
                location,
                "error:".red().bold(),
                message
            );
        }

        // Show code context if available
        if let Some(code) = &marker.code {
            eprintln!("  {}", code.dimmed());
        }
    }

    // Print warnings
    for marker in &warnings {
        let location = marker.location();
        let message = marker.message.as_deref().unwrap_or("Unknown warning");

        if location.is_empty() {
            eprintln!("{} {}", "warning:".yellow().bold(), message);
        } else {
            eprintln!(
                "{}{} {} {}",
                source_prefix,
                location,
                "warning:".yellow().bold(),
                message
            );
        }

        // Show code context if available
        if let Some(code) = &marker.code {
            eprintln!("  {}", code.dimmed());
        }
    }

    // Summary
    eprintln!();
    if !errors.is_empty() {
        eprintln!(
            "{} {} error(s), {} warning(s)",
            "✗".red(),
            errors.len(),
            warnings.len()
        );
    } else {
        eprintln!("{} {} warning(s)", "⚠".yellow(), warnings.len());
    }
}
