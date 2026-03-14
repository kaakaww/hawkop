//! Run command handlers for hosted scan control
//!
//! Provides start, stop, and status operations for cloud-based scans.

use std::sync::Arc;
use std::time::Duration;

use colored::Colorize;
use dialoguer::Confirm;
use log::debug;

use crate::cache::CachedStackHawkClient;
use crate::cli::args::GlobalOptions;
use crate::cli::{CommandContext, OutputFormat};
use crate::client::models::Application;
use crate::client::{ListingApi, PerchApi, StackHawkClient};
use crate::error::Result;
use crate::models::display::{PrettyRunStatus, RunStatusDisplay};
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

// ============================================================================
// Start Command
// ============================================================================

/// Start a hosted scan for an application
pub async fn start(
    opts: &GlobalOptions,
    app: &str,
    env: Option<&str>,
    config: Option<&str>,
    watch: bool,
) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;
    let client = ctx.client.clone();

    // Resolve app name/ID to full application
    let application = resolve_app(&client, org_id, app).await?;
    let app_id = &application.id;

    debug!("Starting scan for app {} ({})", application.name, app_id);

    // Check if there's already a running scan
    let status = client.get_scan_status(app_id).await?;
    if status.is_running() {
        eprintln!(
            "{} A scan is already running for '{}'. Use `hawkop run stop` to stop it first.",
            "Warning:".yellow().bold(),
            application.name
        );
        return Ok(());
    }

    // Start the scan
    eprintln!("{} Starting scan for '{}'...", "→".blue(), application.name);

    let response = client.start_scan(app_id, env, config).await?;

    if let Some(id) = &response.id {
        debug!("Scan started with command ID: {}", id);
    }

    eprintln!("{} Scan started for '{}'", "✓".green(), application.name);

    // If watch mode, poll for status
    if watch {
        eprintln!();
        eprintln!(
            "{} Watching scan progress (Ctrl+C to stop watching)...",
            "→".blue()
        );
        eprintln!();

        watch_status(&client, app_id, Some(&application.name), opts.format, 5).await?;
    } else {
        eprintln!(
            "{}",
            format!("→ Use `hawkop run status --app {}` to check progress", app).dimmed()
        );
        eprintln!(
            "{}",
            format!(
                "→ Use `hawkop run start --app {} --watch` to watch progress",
                app
            )
            .dimmed()
        );
    }

    Ok(())
}

// ============================================================================
// Stop Command
// ============================================================================

/// Stop a running hosted scan
pub async fn stop(opts: &GlobalOptions, app: &str, yes: bool) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;
    let client = ctx.client.clone();

    // Resolve app name/ID
    let application = resolve_app(&client, org_id, app).await?;
    let app_id = &application.id;

    // Check if there's actually a running scan
    let status = client.get_scan_status(app_id).await?;
    if !status.is_running() {
        eprintln!(
            "{} No scan is currently running for '{}'.",
            "Info:".blue(),
            application.name
        );
        return Ok(());
    }

    // Confirm unless --yes
    if !yes {
        let confirmed = Confirm::new()
            .with_prompt(format!("Stop the running scan for '{}'?", application.name))
            .default(false)
            .interact()?;

        if !confirmed {
            eprintln!("{}", "Cancelled".yellow());
            return Ok(());
        }
    }

    debug!("Stopping scan for app {} ({})", application.name, app_id);

    // Stop the scan
    eprintln!("{} Stopping scan for '{}'...", "→".blue(), application.name);

    client.stop_scan(app_id).await?;

    eprintln!(
        "{} Stop command sent. The scan will stop after completing its current operation.",
        "✓".green()
    );
    eprintln!(
        "{}",
        format!("→ Use `hawkop run status --app {}` to verify", app).dimmed()
    );

    Ok(())
}

// ============================================================================
// Status Command
// ============================================================================

/// Get the status of a hosted scan
pub async fn status(opts: &GlobalOptions, app: &str, watch: bool, interval: u64) -> Result<()> {
    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;
    let client = ctx.client.clone();

    // Resolve app name/ID
    let application = resolve_app(&client, org_id, app).await?;
    let app_id = &application.id;

    if watch {
        watch_status(
            &client,
            app_id,
            Some(&application.name),
            opts.format,
            interval,
        )
        .await
    } else {
        // Single status check
        let device = client.get_scan_status(app_id).await?;

        match opts.format {
            OutputFormat::Json => {
                let display = RunStatusDisplay::from(device);
                let json = format_json(&display)?;
                println!("{}", json);
            }
            OutputFormat::Table => {
                let display = RunStatusDisplay::from(device);
                let table = format_table(&[display]);
                println!("{}", table);
            }
            OutputFormat::Pretty => {
                PrettyRunStatus::new(&device, Some(&application.name)).print();
            }
        }

        Ok(())
    }
}

/// Watch status with periodic polling
async fn watch_status(
    client: &Client,
    app_id: &str,
    app_name: Option<&str>,
    format: OutputFormat,
    interval_secs: u64,
) -> Result<()> {
    let interval = Duration::from_secs(interval_secs);

    loop {
        // Clear screen for clean update (only in pretty mode)
        if format == OutputFormat::Pretty {
            // Use ANSI escape to move cursor up and clear (works in most terminals)
            print!("\x1B[2J\x1B[1;1H");
        }

        let device = client.get_scan_status(app_id).await?;
        let is_running = device.is_running();

        match format {
            OutputFormat::Json => {
                let display = RunStatusDisplay::from(device);
                let json = format_json(&display)?;
                println!("{}", json);
            }
            OutputFormat::Table => {
                let display = RunStatusDisplay::from(device);
                let table = format_table(&[display]);
                println!("{}", table);
            }
            OutputFormat::Pretty => {
                PrettyRunStatus::new(&device, app_name).print();
                println!();
                println!(
                    "{}",
                    format!("Refreshing every {}s... (Ctrl+C to stop)", interval_secs).dimmed()
                );
            }
        }

        // If scan completed, stop watching
        if !is_running {
            if format == OutputFormat::Pretty {
                eprintln!();
                eprintln!("{} Scan completed or stopped.", "✓".green());
            }
            break;
        }

        tokio::time::sleep(interval).await;
    }

    Ok(())
}
