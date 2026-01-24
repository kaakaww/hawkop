//! Team management commands

use std::collections::HashSet;
use std::io::{self, BufRead};
use std::sync::Arc;

use colored::Colorize;
use dialoguer::Confirm;
use log::debug;

use crate::cache::CachedStackHawkClient;
use crate::cli::args::GlobalOptions;
use crate::cli::{CommandContext, OutputFormat, PaginationArgs, TeamFilterArgs};
use crate::client::models::{
    Application, CreateTeamRequest, Team, TeamDetail, UpdateTeamRequest, User,
};
use crate::client::pagination::PaginationParams;
use crate::client::{ListingApi, StackHawkClient, TeamApi, fetch_remaining_pages};
use crate::error::Result;

/// Type alias for the Arc-wrapped cached client used throughout this module
type Client = Arc<CachedStackHawkClient<StackHawkClient>>;

/// Page size for parallel fetching (API max is 1000)
const RESOLUTION_PAGE_SIZE: usize = 1000;

/// Max concurrent requests for parallel fetching
const PARALLEL_FETCH_LIMIT: usize = 32;

// ============================================================================
// Identifier Resolution Helpers
// ============================================================================

/// Simple UUID format check (8-4-4-4-12 hex pattern)
fn looks_like_uuid(s: &str) -> bool {
    let parts: Vec<&str> = s.split('-').collect();
    parts.len() == 5
        && parts
            .iter()
            .all(|p| p.chars().all(|c| c.is_ascii_hexdigit()))
}

// ============================================================================
// Parallel Fetching Helpers for Enterprise-Scale Data
// ============================================================================

/// Fetch all teams for an organization using parallel pagination.
///
/// Uses the paged API variant to get total_count, then fetches remaining
/// pages in parallel for optimal performance with large organizations.
async fn fetch_all_teams(client: Client, org_id: &str) -> Result<Vec<Team>> {
    let first_params = PaginationParams::new()
        .page_size(RESOLUTION_PAGE_SIZE)
        .page(0);

    debug!(
        "Fetching first page of teams (pageSize={})",
        RESOLUTION_PAGE_SIZE
    );
    let first_response = client.list_teams_paged(org_id, Some(&first_params)).await?;

    let mut all_teams = first_response.items;
    debug!(
        "First page returned {} teams, totalCount={:?}",
        all_teams.len(),
        first_response.total_count
    );

    // Fetch remaining pages if totalCount indicates more
    if let Some(total_count) = first_response.total_count {
        let total_pages = total_count.div_ceil(RESOLUTION_PAGE_SIZE);

        if total_pages > 1 {
            let remaining_pages: Vec<usize> = (1..total_pages).collect();

            if !remaining_pages.is_empty() {
                debug!(
                    "Fetching {} remaining team pages in parallel",
                    remaining_pages.len()
                );

                let org = org_id.to_string();

                let remaining_teams = fetch_remaining_pages(
                    remaining_pages,
                    move |page| {
                        let c = client.clone();
                        let o = org.clone();
                        async move {
                            let params = PaginationParams::new()
                                .page_size(RESOLUTION_PAGE_SIZE)
                                .page(page);
                            c.list_teams(&o, Some(&params)).await
                        }
                    },
                    PARALLEL_FETCH_LIMIT,
                )
                .await?;

                all_teams.extend(remaining_teams);
            }
        }
    }

    debug!("Total teams fetched: {}", all_teams.len());
    Ok(all_teams)
}

/// Fetch all users for an organization using parallel pagination.
///
/// Uses the paged API variant to get total_count, then fetches remaining
/// pages in parallel for optimal performance with large organizations.
async fn fetch_all_users(client: Client, org_id: &str) -> Result<Vec<User>> {
    let first_params = PaginationParams::new()
        .page_size(RESOLUTION_PAGE_SIZE)
        .page(0);

    debug!(
        "Fetching first page of users (pageSize={})",
        RESOLUTION_PAGE_SIZE
    );
    let first_response = client.list_users_paged(org_id, Some(&first_params)).await?;

    let mut all_users = first_response.items;
    debug!(
        "First page returned {} users, totalCount={:?}",
        all_users.len(),
        first_response.total_count
    );

    // Fetch remaining pages if totalCount indicates more
    if let Some(total_count) = first_response.total_count {
        let total_pages = total_count.div_ceil(RESOLUTION_PAGE_SIZE);

        if total_pages > 1 {
            let remaining_pages: Vec<usize> = (1..total_pages).collect();

            if !remaining_pages.is_empty() {
                debug!(
                    "Fetching {} remaining user pages in parallel",
                    remaining_pages.len()
                );

                let org = org_id.to_string();

                let remaining_users = fetch_remaining_pages(
                    remaining_pages,
                    move |page| {
                        let c = client.clone();
                        let o = org.clone();
                        async move {
                            let params = PaginationParams::new()
                                .page_size(RESOLUTION_PAGE_SIZE)
                                .page(page);
                            c.list_users(&o, Some(&params)).await
                        }
                    },
                    PARALLEL_FETCH_LIMIT,
                )
                .await?;

                all_users.extend(remaining_users);
            }
        }
    }

    debug!("Total users fetched: {}", all_users.len());
    Ok(all_users)
}

/// Fetch all applications for an organization using parallel pagination.
///
/// Uses the paged API variant to get total_count, then fetches remaining
/// pages in parallel for optimal performance with large organizations.
async fn fetch_all_apps(client: Client, org_id: &str) -> Result<Vec<Application>> {
    let first_params = PaginationParams::new()
        .page_size(RESOLUTION_PAGE_SIZE)
        .page(0);

    debug!(
        "Fetching first page of apps (pageSize={})",
        RESOLUTION_PAGE_SIZE
    );
    let first_response = client.list_apps_paged(org_id, Some(&first_params)).await?;

    let mut all_apps = first_response.items;
    debug!(
        "First page returned {} apps, totalCount={:?}",
        all_apps.len(),
        first_response.total_count
    );

    // Fetch remaining pages if totalCount indicates more
    if let Some(total_count) = first_response.total_count {
        let total_pages = total_count.div_ceil(RESOLUTION_PAGE_SIZE);

        if total_pages > 1 {
            let remaining_pages: Vec<usize> = (1..total_pages).collect();

            if !remaining_pages.is_empty() {
                debug!(
                    "Fetching {} remaining app pages in parallel",
                    remaining_pages.len()
                );

                let org = org_id.to_string();

                let remaining_apps = fetch_remaining_pages(
                    remaining_pages,
                    move |page| {
                        let c = client.clone();
                        let o = org.clone();
                        async move {
                            let params = PaginationParams::new()
                                .page_size(RESOLUTION_PAGE_SIZE)
                                .page(page);
                            c.list_apps(&o, Some(&params)).await
                        }
                    },
                    PARALLEL_FETCH_LIMIT,
                )
                .await?;

                all_apps.extend(remaining_apps);
            }
        }
    }

    debug!("Total apps fetched: {}", all_apps.len());
    Ok(all_apps)
}

// ============================================================================
// Resolution Functions (use parallel fetching for enterprise scale)
// ============================================================================

/// Resolve team identifier (name or UUID) to UUID.
/// Errors if name matches multiple teams (safety for mutations).
async fn resolve_team(client: Client, org_id: &str, identifier: &str) -> Result<String> {
    // If it's already a valid UUID, return it
    if looks_like_uuid(identifier) {
        return Ok(identifier.to_string());
    }

    // Fetch all teams with parallel pagination
    let teams = fetch_all_teams(client, org_id).await?;

    // Find ALL teams with matching name (case-insensitive)
    let matches: Vec<_> = teams
        .iter()
        .filter(|t| t.name.eq_ignore_ascii_case(identifier))
        .collect();

    match matches.len() {
        0 => Err(crate::error::Error::Other(format!(
            "Team not found: {}\n\n→ Use 'hawkop team list' to see available teams.",
            identifier
        ))),
        1 => Ok(matches[0].id.clone()),
        _ => {
            // Multiple teams with same name - error with IDs for disambiguation
            let team_list = matches
                .iter()
                .map(|t| format!("  • {} (ID: {})", t.name, t.id))
                .collect::<Vec<_>>()
                .join("\n");

            Err(crate::error::Error::Other(format!(
                "Multiple teams found with name \"{}\". Use team ID instead:\n\n{}\n\n→ Example: hawkop team get {}",
                identifier, team_list, matches[0].id
            )))
        }
    }
}

/// Resolve user identifiers (email or UUID) to UUIDs
async fn resolve_users(
    client: Client,
    org_id: &str,
    identifiers: &[String],
) -> Result<Vec<String>> {
    if identifiers.is_empty() {
        return Ok(vec![]);
    }

    // Fetch all users with parallel pagination
    let members = fetch_all_users(client, org_id).await?;

    identifiers
        .iter()
        .map(|id| {
            // If it's a UUID, use it directly
            if looks_like_uuid(id) {
                return Ok(id.clone());
            }
            // Otherwise look up by email
            members
                .iter()
                .find(|u| u.external.email.eq_ignore_ascii_case(id))
                .map(|u| u.external.id.clone())
                .ok_or_else(|| {
                    crate::error::Error::Other(format!(
                        "User not found: {}\n\nUse 'hawkop user list' to see available users.",
                        id
                    ))
                })
        })
        .collect()
}

/// Resolve app identifiers (name or UUID) to UUIDs.
/// Errors if name matches multiple apps (safety for mutations).
async fn resolve_apps(client: Client, org_id: &str, identifiers: &[String]) -> Result<Vec<String>> {
    if identifiers.is_empty() {
        return Ok(vec![]);
    }

    // Fetch all applications with parallel pagination
    let apps = fetch_all_apps(client, org_id).await?;

    identifiers
        .iter()
        .map(|id| {
            // If it's a UUID, use it directly
            if looks_like_uuid(id) {
                return Ok(id.clone());
            }

            // Find ALL apps with matching name (case-insensitive)
            let matches: Vec<_> = apps
                .iter()
                .filter(|a| a.name.eq_ignore_ascii_case(id))
                .collect();

            match matches.len() {
                0 => Err(crate::error::Error::Other(format!(
                    "Application not found: {}\n\n→ Use 'hawkop app list' to see available applications.",
                    id
                ))),
                1 => Ok(matches[0].id.clone()),
                _ => {
                    // Multiple apps with same name - error with IDs for disambiguation
                    let app_list = matches
                        .iter()
                        .map(|a| format!("  • {} (ID: {})", a.name, a.id))
                        .collect::<Vec<_>>()
                        .join("\n");

                    Err(crate::error::Error::Other(format!(
                        "Multiple applications found with name \"{}\". Use app ID instead:\n\n{}\n\n→ Example: hawkop team add-app \"Team\" {}",
                        id, app_list, matches[0].id
                    )))
                }
            }
        })
        .collect()
}

/// Read identifiers from stdin (one per line)
#[allow(clippy::lines_filter_map_ok)]
fn read_stdin_lines() -> Result<Vec<String>> {
    let stdin = io::stdin();
    let lines: Vec<String> = stdin
        .lock()
        .lines()
        .filter_map(|l| l.ok())
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(lines)
}

// ============================================================================
// Duplicate App Detection (API Safety)
// ============================================================================

/// Check if any of the given app IDs are already assigned to other teams.
///
/// Returns a list of (app_name, app_id, other_team_name) tuples for any duplicates found.
///
/// The StackHawk platform enforces that each app may only belong to one team at a time.
/// This check provides a clear error message before attempting an invalid assignment.
async fn check_duplicate_app_assignments(
    client: Client,
    org_id: &str,
    target_team_id: &str,
    app_ids: &[String],
) -> Result<Vec<(String, String, String)>> {
    use futures::stream::{FuturesUnordered, StreamExt};

    if app_ids.is_empty() {
        return Ok(vec![]);
    }

    // Fetch all teams to check their app assignments
    let all_teams = fetch_all_teams(client.clone(), org_id).await?;

    // Fetch team details in parallel
    debug!(
        "Checking {} teams for duplicate app assignments",
        all_teams.len()
    );
    let mut futures: FuturesUnordered<_> = all_teams
        .iter()
        .filter(|t| t.id != target_team_id) // Skip the target team
        .map(|team| {
            let c = client.clone();
            let org = org_id.to_string();
            let team_id = team.id.clone();
            async move { c.get_team(&org, &team_id).await }
        })
        .collect();

    // Build a map of app_id -> team_name for apps in other teams
    let mut app_to_team: std::collections::HashMap<String, (String, String)> =
        std::collections::HashMap::new();

    while let Some(result) = futures.next().await {
        if let Ok(team_detail) = result {
            for app in &team_detail.applications {
                let app_name = app
                    .application_name
                    .clone()
                    .unwrap_or_else(|| app.application_id.clone());
                app_to_team.insert(
                    app.application_id.clone(),
                    (app_name, team_detail.name.clone()),
                );
            }
        }
    }

    // Check which requested apps are already assigned elsewhere
    let duplicates: Vec<(String, String, String)> = app_ids
        .iter()
        .filter_map(|app_id| {
            app_to_team
                .get(app_id)
                .map(|(app_name, team_name)| (app_name.clone(), app_id.clone(), team_name.clone()))
        })
        .collect();

    Ok(duplicates)
}

// ============================================================================
// Command Context Setup
// ============================================================================

/// Common setup for team commands that need API access.
///
/// Returns an Arc-wrapped cached client suitable for parallel requests.
async fn setup_team_context(opts: &GlobalOptions) -> Result<(String, Client)> {
    let ctx = CommandContext::new(opts).await?;

    let org_id = ctx.require_org_id()?.to_string();
    Ok((org_id, ctx.client))
}

// ============================================================================
// Display Helpers
// ============================================================================

/// Display team detail with members and apps
fn display_team_detail(team: &TeamDetail, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Pretty => {
            println!();
            println!("{}: {}", "Team".bold(), team.name);
            println!("{}: {}", "ID".dimmed(), team.id);

            println!();
            println!(
                "{} ({}):",
                "Members".bold(),
                team.users.len().to_string().cyan()
            );
            if team.users.is_empty() {
                println!("  {}", "(none)".dimmed());
            } else {
                for user in &team.users {
                    let email = user.email.as_deref().unwrap_or("--");
                    let name = user.user_name.as_deref().unwrap_or("--");
                    let role = user.role.as_deref().unwrap_or("--");
                    println!("  • {} ({}) [{}]", email, name, role.dimmed());
                }
            }

            println!();
            println!(
                "{} ({}):",
                "Applications".bold(),
                team.applications.len().to_string().cyan()
            );
            if team.applications.is_empty() {
                println!("  {}", "(none)".dimmed());
            } else {
                for app in &team.applications {
                    let name = app.application_name.as_deref().unwrap_or("--");
                    let envs = if app.environments.is_empty() {
                        "--".to_string()
                    } else {
                        app.environments.join(", ")
                    };
                    println!("  • {} ({})", name, envs.dimmed());
                }
            }
            println!();
        }
        OutputFormat::Table | OutputFormat::Json => {
            // For table/json, output as JSON
            let output = serde_json::json!({
                "data": team,
                "meta": {
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    Ok(())
}

// ============================================================================
// List Command
// ============================================================================

/// Run the team list command with user/app counts.
///
/// Unlike other list commands, this fetches team details in parallel to get
/// accurate member and application counts for each team.
pub async fn list(
    opts: &GlobalOptions,
    pagination: &PaginationArgs,
    filters: &TeamFilterArgs,
) -> Result<()> {
    use crate::models::TeamListDisplay;
    use crate::output::Formattable;
    use futures::stream::{FuturesUnordered, StreamExt};

    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Fetch teams - use pagination if limit specified, otherwise fetch all
    let teams = if let Some(limit) = pagination.limit {
        // Fetch single page with specified limit
        let params = PaginationParams::new()
            .page_size(limit)
            .page(pagination.page.unwrap_or(0));
        client.list_teams(&org_id, Some(&params)).await?
    } else {
        // Fetch all teams using parallel pagination (no limit means get everything)
        fetch_all_teams(client.clone(), &org_id).await?
    };

    if teams.is_empty() {
        // For JSON format, return empty array with metadata
        if matches!(format, OutputFormat::Json) {
            let empty: Vec<TeamListDisplay> = vec![];
            empty.print(format)?;
        } else {
            eprintln!("No teams found.");
            eprintln!("→ Create a team: hawkop team create <NAME>");
        }
        return Ok(());
    }

    // Fetch team details in parallel to get user/app counts
    debug!("Fetching details for {} teams in parallel", teams.len());
    let mut futures: FuturesUnordered<_> = teams
        .iter()
        .map(|team| {
            let c = client.clone();
            let org = org_id.clone();
            let team_id = team.id.clone();
            async move { c.get_team(&org, &team_id).await }
        })
        .collect();

    let mut team_details: Vec<TeamDetail> = Vec::with_capacity(teams.len());
    while let Some(result) = futures.next().await {
        match result {
            Ok(detail) => team_details.push(detail),
            Err(e) => {
                // Log error but continue with other teams
                debug!("Failed to fetch team detail: {}", e);
            }
        }
    }

    // Apply filters (client-side since API doesn't support them)
    let filtered_details: Vec<TeamDetail> = team_details
        .into_iter()
        .filter(|team| {
            // Name filter (substring match, case-insensitive)
            if let Some(ref name_filter) = filters.name
                && !team
                    .name
                    .to_lowercase()
                    .contains(&name_filter.to_lowercase())
            {
                return false;
            }

            // Member filter (email match, case-insensitive)
            if let Some(ref member_filter) = filters.member {
                let has_member = team.users.iter().any(|u| {
                    u.email
                        .as_ref()
                        .is_some_and(|e| e.eq_ignore_ascii_case(member_filter))
                });
                if !has_member {
                    return false;
                }
            }

            // App filter (app name match, case-insensitive)
            if let Some(ref app_filter) = filters.app {
                let has_app = team.applications.iter().any(|a| {
                    a.application_name
                        .as_ref()
                        .is_some_and(|n| n.eq_ignore_ascii_case(app_filter))
                });
                if !has_app {
                    return false;
                }
            }

            true
        })
        .collect();

    if filtered_details.is_empty() {
        // For JSON format, return empty array with metadata
        if matches!(format, OutputFormat::Json) {
            let empty: Vec<TeamListDisplay> = vec![];
            empty.print(format)?;
        } else if filters.name.is_some() || filters.member.is_some() || filters.app.is_some() {
            eprintln!("No teams match the specified filters.");
        } else {
            eprintln!("No teams found.");
            eprintln!("→ Create a team: hawkop team create <NAME>");
        }
        return Ok(());
    }

    // Sort by name for consistent output
    let mut sorted_details = filtered_details;
    sorted_details.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Convert to display format
    let display_items: Vec<TeamListDisplay> = sorted_details
        .into_iter()
        .map(TeamListDisplay::from)
        .collect();

    // Output using Formattable trait
    display_items.print(format)?;

    Ok(())
}

// ============================================================================
// Get Command
// ============================================================================

/// Get team details
pub async fn get(opts: &GlobalOptions, team_identifier: &str) -> Result<()> {
    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Resolve team ID
    let team_id = resolve_team(client.clone(), &org_id, team_identifier).await?;

    // Fetch team details
    let team = client.get_team(&org_id, &team_id).await?;

    display_team_detail(&team, format)?;

    Ok(())
}

// ============================================================================
// Create Command
// ============================================================================

/// Create a new team
pub async fn create(
    opts: &GlobalOptions,
    name: &str,
    users: Option<Vec<String>>,
    apps: Option<Vec<String>>,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Validate non-empty name
    let name = name.trim();
    if name.is_empty() {
        return Err(crate::error::Error::Other(
            "Team name cannot be empty.".to_string(),
        ));
    }

    // Check for existing team with same name (before any other operations)
    let existing_teams = fetch_all_teams(client.clone(), &org_id).await?;
    let duplicates: Vec<_> = existing_teams
        .iter()
        .filter(|t| t.name.eq_ignore_ascii_case(name))
        .collect();

    if !duplicates.is_empty() {
        let existing_list = duplicates
            .iter()
            .map(|t| format!("  • {} (ID: {})", t.name, t.id))
            .collect::<Vec<_>>()
            .join("\n");

        return Err(crate::error::Error::Other(format!(
            "A team named \"{}\" already exists:\n\n{}\n\n→ Choose a different name or rename the existing team.",
            name, existing_list
        )));
    }

    // Resolve user IDs if provided
    let user_ids = if let Some(ref user_list) = users {
        Some(resolve_users(client.clone(), &org_id, user_list).await?)
    } else {
        None
    };

    // Resolve app IDs if provided
    let app_ids = if let Some(ref app_list) = apps {
        Some(resolve_apps(client.clone(), &org_id, app_list).await?)
    } else {
        None
    };

    // Safety check: each app may only belong to one team at a time
    // For create, there's no "target team" yet, so we pass empty string to check all teams
    if !force && let Some(ref ids) = app_ids {
        let duplicates = check_duplicate_app_assignments(client.clone(), &org_id, "", ids).await?;

        if !duplicates.is_empty() {
            let dup_list = duplicates
                .iter()
                .map(|(app_name, app_id, other_team)| {
                    format!(
                        "  • \"{}\" (ID: {}) → already in team \"{}\"",
                        app_name, app_id, other_team
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let first_dup = &duplicates[0];
            return Err(crate::error::Error::Other(format!(
                "Cannot assign app(s) already in other teams:\n\n{}\n\n\
                 Each app may only be assigned to one team at a time.\n\n\
                 → Remove first: hawkop team remove-app \"{}\" \"{}\"\n\
                 → Or use --force to override (not recommended)",
                dup_list, first_dup.2, first_dup.0
            )));
        }
    }

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!("Would create team: {}", name.bold());
        if let Some(ref ids) = user_ids
            && !ids.is_empty()
        {
            eprintln!("Initial members: {}", ids.len());
        }
        if let Some(ref ids) = app_ids
            && !ids.is_empty()
        {
            eprintln!("Initial applications: {}", ids.len());
        }
        return Ok(());
    }

    // Warn if using --force
    if force && app_ids.is_some() {
        eprintln!(
            "{} Using --force: bypassing duplicate app safety check",
            "⚠".yellow()
        );
    }

    // Create team with optional users and apps in a single request
    let request = CreateTeamRequest {
        name: name.to_string(),
        organization_id: org_id.clone(),
        user_ids,
        application_ids: app_ids,
    };

    let team = client.create_team(&org_id, request).await?;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": team,
                "meta": {
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!(
                "{} Team \"{}\" created (ID: {})",
                "✓".green(),
                team.name,
                team.id
            );
            eprintln!("→ View team: hawkop team get {}", team.id);
        }
    }

    Ok(())
}

// ============================================================================
// Rename Command
// ============================================================================

/// Rename a team
pub async fn rename(
    opts: &GlobalOptions,
    team_identifier: &str,
    new_name: &str,
    dry_run: bool,
) -> Result<()> {
    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Validate non-empty name
    let new_name = new_name.trim();
    if new_name.is_empty() {
        return Err(crate::error::Error::Other(
            "Team name cannot be empty.".to_string(),
        ));
    }

    // Resolve team ID
    let team_id = resolve_team(client.clone(), &org_id, team_identifier).await?;

    // Get current team for display - use fresh read to ensure we have latest state before mutation
    let current_team = client.get_team_fresh(&org_id, &team_id).await?;

    // Check if new name already exists (skip if renaming to same name with different case)
    if !current_team.name.eq_ignore_ascii_case(new_name) {
        let existing_teams = fetch_all_teams(client.clone(), &org_id).await?;
        let duplicates: Vec<_> = existing_teams
            .iter()
            .filter(|t| t.name.eq_ignore_ascii_case(new_name) && t.id != team_id)
            .collect();

        if !duplicates.is_empty() {
            let existing_list = duplicates
                .iter()
                .map(|t| format!("  • {} (ID: {})", t.name, t.id))
                .collect::<Vec<_>>()
                .join("\n");

            return Err(crate::error::Error::Other(format!(
                "A team named \"{}\" already exists:\n\n{}\n\n→ Choose a different name.",
                new_name, existing_list
            )));
        }
    }

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!(
            "Would rename team: \"{}\" → \"{}\"",
            current_team.name,
            new_name.bold()
        );
        return Ok(());
    }

    // Preserve existing users and apps - API defaults to empty if not provided
    let current_user_ids: Vec<String> = current_team
        .users
        .iter()
        .map(|u| u.user_id.clone())
        .collect();
    let current_app_ids: Vec<String> = current_team
        .applications
        .iter()
        .map(|a| a.application_id.clone())
        .collect();

    log::debug!(
        "Building update request: name={}, user_ids={:?}, app_ids={:?}",
        new_name,
        current_user_ids,
        current_app_ids
    );

    let request = UpdateTeamRequest {
        team_id: team_id.clone(),
        organization_id: org_id.clone(),
        name: Some(new_name.to_string()),
        user_ids: Some(current_user_ids),
        application_ids: Some(current_app_ids),
    };

    let team = client.update_team(&org_id, &team_id, request).await?;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": team,
                "meta": {
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!(
                "{} Team renamed: \"{}\" → \"{}\"",
                "✓".green(),
                current_team.name,
                team.name
            );
            eprintln!("→ View team: hawkop team get {}", team.id);
        }
    }

    Ok(())
}

// ============================================================================
// Delete Command
// ============================================================================

/// Delete a team
pub async fn delete(
    opts: &GlobalOptions,
    team_identifier: &str,
    yes: bool,
    dry_run: bool,
) -> Result<()> {
    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Resolve team ID and get details - use fresh read to ensure we have latest state before deletion
    let team_id = resolve_team(client.clone(), &org_id, team_identifier).await?;
    let team = client.get_team_fresh(&org_id, &team_id).await?;

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!("Would delete team: \"{}\" (ID: {})", team.name, team.id);
        eprintln!(
            "  Members: {} users would be removed from this team",
            team.users.len()
        );
        eprintln!(
            "  Apps: {} applications would be unassigned",
            team.applications.len()
        );
        return Ok(());
    }

    // Confirmation prompt unless --yes
    if !yes {
        eprintln!(
            "{} Delete team \"{}\"? This cannot be undone.",
            "⚠".yellow(),
            team.name
        );
        eprintln!(
            "  Members: {} users will be removed from this team",
            team.users.len()
        );
        eprintln!(
            "  Apps: {} applications will be unassigned",
            team.applications.len()
        );
        eprintln!();

        let confirm = Confirm::new()
            .with_prompt("Confirm deletion?")
            .default(false)
            .interact()?;

        if !confirm {
            eprintln!("Cancelled.");
            return Ok(());
        }
    }

    client.delete_team(&org_id, &team_id).await?;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": { "deleted": true, "team_id": team_id, "team_name": team.name },
                "meta": {
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!("{} Team \"{}\" deleted", "✓".green(), team.name);
            eprintln!("→ List teams: hawkop team list");
        }
    }

    Ok(())
}

// ============================================================================
// Member Management Commands
// ============================================================================

/// Add users to a team
pub async fn add_user(
    opts: &GlobalOptions,
    team_identifier: &str,
    mut users: Vec<String>,
    stdin: bool,
    dry_run: bool,
) -> Result<()> {
    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Read from stdin if requested
    if stdin {
        users.extend(read_stdin_lines()?);
    }

    if users.is_empty() {
        return Err(crate::error::Error::Other(
            "No users specified. Provide user emails or IDs as arguments or use --stdin."
                .to_string(),
        ));
    }

    // Resolve team ID and get current state - use fresh read to ensure we have latest state before mutation
    let team_id = resolve_team(client.clone(), &org_id, team_identifier).await?;
    let team = client.get_team_fresh(&org_id, &team_id).await?;

    // Resolve user IDs
    let new_user_ids = resolve_users(client.clone(), &org_id, &users).await?;

    // Get current member IDs
    let current_ids: HashSet<_> = team.users.iter().map(|u| u.user_id.clone()).collect();

    // Filter to only new users
    let users_to_add: Vec<_> = new_user_ids
        .iter()
        .filter(|id| !current_ids.contains(*id))
        .cloned()
        .collect();

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!(
            "Would add {} user(s) to team \"{}\":",
            users_to_add.len(),
            team.name
        );
        for id in &users_to_add {
            eprintln!("  • {}", id);
        }
        if users_to_add.len() < new_user_ids.len() {
            eprintln!(
                "\n{} {} user(s) already in team (skipped)",
                "ℹ".blue(),
                new_user_ids.len() - users_to_add.len()
            );
        }
        return Ok(());
    }

    if users_to_add.is_empty() {
        eprintln!(
            "{} All specified users are already team members",
            "ℹ".blue()
        );
        return Ok(());
    }

    // Build new complete member list
    let mut all_user_ids: Vec<_> = current_ids.into_iter().collect();
    all_user_ids.extend(users_to_add.clone());

    // Preserve existing apps
    let current_app_ids: Vec<_> = team
        .applications
        .iter()
        .map(|a| a.application_id.clone())
        .collect();

    let request = UpdateTeamRequest {
        team_id: team_id.clone(),
        organization_id: org_id.clone(),
        name: Some(team.name.clone()),
        user_ids: Some(all_user_ids),
        application_ids: Some(current_app_ids),
    };

    let updated = client.update_team(&org_id, &team_id, request).await?;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": updated,
                "meta": {
                    "added": users_to_add.len(),
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!(
                "{} Added {} user(s) to team \"{}\"",
                "✓".green(),
                users_to_add.len(),
                updated.name
            );
            eprintln!("→ View team: hawkop team get {}", updated.id);
        }
    }

    Ok(())
}

/// Remove users from a team
pub async fn remove_user(
    opts: &GlobalOptions,
    team_identifier: &str,
    mut users: Vec<String>,
    stdin: bool,
    dry_run: bool,
) -> Result<()> {
    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Read from stdin if requested
    if stdin {
        users.extend(read_stdin_lines()?);
    }

    if users.is_empty() {
        return Err(crate::error::Error::Other(
            "No users specified. Provide user emails or IDs as arguments or use --stdin."
                .to_string(),
        ));
    }

    // Resolve team ID and get current state - use fresh read to ensure we have latest state before mutation
    let team_id = resolve_team(client.clone(), &org_id, team_identifier).await?;
    let team = client.get_team_fresh(&org_id, &team_id).await?;

    // Resolve user IDs to remove
    let remove_ids: HashSet<_> = resolve_users(client.clone(), &org_id, &users)
        .await?
        .into_iter()
        .collect();

    // Get current member IDs and filter out those to remove
    let current_ids: Vec<_> = team.users.iter().map(|u| u.user_id.clone()).collect();
    let remaining_ids: Vec<_> = current_ids
        .iter()
        .filter(|id| !remove_ids.contains(*id))
        .cloned()
        .collect();

    let actually_removing = current_ids.len() - remaining_ids.len();

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!(
            "Would remove {} user(s) from team \"{}\":",
            actually_removing, team.name
        );
        for id in &remove_ids {
            if current_ids.contains(id) {
                eprintln!("  • {}", id);
            }
        }
        if actually_removing < remove_ids.len() {
            eprintln!(
                "\n{} {} user(s) not in team (ignored)",
                "ℹ".blue(),
                remove_ids.len() - actually_removing
            );
        }
        return Ok(());
    }

    if actually_removing == 0 {
        eprintln!(
            "{} None of the specified users are team members",
            "ℹ".blue()
        );
        return Ok(());
    }

    // Preserve existing apps
    let current_app_ids: Vec<_> = team
        .applications
        .iter()
        .map(|a| a.application_id.clone())
        .collect();

    let request = UpdateTeamRequest {
        team_id: team_id.clone(),
        organization_id: org_id.clone(),
        name: Some(team.name.clone()),
        user_ids: Some(remaining_ids),
        application_ids: Some(current_app_ids),
    };

    let updated = client.update_team(&org_id, &team_id, request).await?;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": updated,
                "meta": {
                    "removed": actually_removing,
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!(
                "{} Removed {} user(s) from team \"{}\"",
                "✓".green(),
                actually_removing,
                updated.name
            );
            eprintln!("→ View team: hawkop team get {}", updated.id);
        }
    }

    Ok(())
}

/// Replace all team members (SCIM sync)
pub async fn set_users(
    opts: &GlobalOptions,
    team_identifier: &str,
    mut users: Vec<String>,
    stdin: bool,
    dry_run: bool,
    yes: bool,
) -> Result<()> {
    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Read from stdin if requested
    if stdin {
        users.extend(read_stdin_lines()?);
    }

    // Resolve team ID and get current state - use fresh read to ensure we have latest state before mutation
    let team_id = resolve_team(client.clone(), &org_id, team_identifier).await?;
    let team = client.get_team_fresh(&org_id, &team_id).await?;

    // Resolve new user IDs
    let new_user_ids: HashSet<_> = resolve_users(client.clone(), &org_id, &users)
        .await?
        .into_iter()
        .collect();

    // Calculate diff
    let current_ids: HashSet<_> = team.users.iter().map(|u| u.user_id.clone()).collect();
    let to_add: Vec<_> = new_user_ids.difference(&current_ids).cloned().collect();
    let to_remove: Vec<_> = current_ids.difference(&new_user_ids).cloned().collect();
    let unchanged: Vec<_> = new_user_ids.intersection(&current_ids).cloned().collect();

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!("Team \"{}\" member changes:", team.name);
        if !to_add.is_empty() {
            eprintln!("  {} Add: {}", "+".green(), to_add.len());
            for id in &to_add {
                eprintln!("    • {}", id);
            }
        }
        if !to_remove.is_empty() {
            eprintln!("  {} Remove: {}", "-".red(), to_remove.len());
            for id in &to_remove {
                eprintln!("    • {}", id);
            }
        }
        if !unchanged.is_empty() {
            eprintln!("  {} Unchanged: {}", "=".dimmed(), unchanged.len());
        }
        return Ok(());
    }

    // Confirmation if removing users (unless --yes)
    if !yes && !to_remove.is_empty() {
        eprintln!(
            "{} This will replace team membership for \"{}\":",
            "⚠".yellow(),
            team.name
        );
        eprintln!("  Add: {} user(s)", to_add.len());
        eprintln!("  Remove: {} user(s)", to_remove.len());
        eprintln!("  Unchanged: {} user(s)", unchanged.len());
        eprintln!();

        let confirm = Confirm::new()
            .with_prompt("Proceed with membership sync?")
            .default(false)
            .interact()?;

        if !confirm {
            eprintln!("Cancelled.");
            return Ok(());
        }
    }

    // Preserve existing apps
    let current_app_ids: Vec<_> = team
        .applications
        .iter()
        .map(|a| a.application_id.clone())
        .collect();

    let request = UpdateTeamRequest {
        team_id: team_id.clone(),
        organization_id: org_id.clone(),
        name: Some(team.name.clone()),
        user_ids: Some(new_user_ids.into_iter().collect()),
        application_ids: Some(current_app_ids),
    };

    let updated = client.update_team(&org_id, &team_id, request).await?;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": updated,
                "meta": {
                    "added": to_add.len(),
                    "removed": to_remove.len(),
                    "unchanged": unchanged.len(),
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!(
                "{} Team \"{}\" membership updated (+{} -{} ={})",
                "✓".green(),
                updated.name,
                to_add.len(),
                to_remove.len(),
                unchanged.len()
            );
            eprintln!("→ View team: hawkop team get {}", updated.id);
        }
    }

    Ok(())
}

// ============================================================================
// Application Assignment Commands
// ============================================================================

/// Assign applications to a team
pub async fn add_app(
    opts: &GlobalOptions,
    team_identifier: &str,
    mut apps: Vec<String>,
    stdin: bool,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Read from stdin if requested
    if stdin {
        apps.extend(read_stdin_lines()?);
    }

    if apps.is_empty() {
        return Err(crate::error::Error::Other(
            "No applications specified. Provide app names or IDs as arguments or use --stdin."
                .to_string(),
        ));
    }

    // Resolve team ID and get current state - use fresh read to ensure we have latest state before mutation
    let team_id = resolve_team(client.clone(), &org_id, team_identifier).await?;
    let team = client.get_team_fresh(&org_id, &team_id).await?;

    // Resolve app IDs
    let new_app_ids = resolve_apps(client.clone(), &org_id, &apps).await?;

    // Get current app IDs
    let current_ids: HashSet<_> = team
        .applications
        .iter()
        .map(|a| a.application_id.clone())
        .collect();

    // Filter to only new apps
    let apps_to_add: Vec<_> = new_app_ids
        .iter()
        .filter(|id| !current_ids.contains(*id))
        .cloned()
        .collect();

    // Safety check: each app may only belong to one team at a time
    if !force && !apps_to_add.is_empty() {
        let duplicates =
            check_duplicate_app_assignments(client.clone(), &org_id, &team_id, &apps_to_add)
                .await?;

        if !duplicates.is_empty() {
            let dup_list = duplicates
                .iter()
                .map(|(app_name, app_id, other_team)| {
                    format!(
                        "  • \"{}\" (ID: {}) → already in team \"{}\"",
                        app_name, app_id, other_team
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let first_dup = &duplicates[0];
            return Err(crate::error::Error::Other(format!(
                "Cannot assign app(s) already in other teams:\n\n{}\n\n\
                 Each app may only be assigned to one team at a time.\n\n\
                 → Remove first: hawkop team remove-app \"{}\" \"{}\"\n\
                 → Or use --force to override (not recommended)",
                dup_list, first_dup.2, first_dup.0
            )));
        }
    }

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!(
            "Would assign {} application(s) to team \"{}\":",
            apps_to_add.len(),
            team.name
        );
        for id in &apps_to_add {
            eprintln!("  • {}", id);
        }
        if apps_to_add.len() < new_app_ids.len() {
            eprintln!(
                "\n{} {} application(s) already assigned (skipped)",
                "ℹ".blue(),
                new_app_ids.len() - apps_to_add.len()
            );
        }
        return Ok(());
    }

    if apps_to_add.is_empty() {
        eprintln!(
            "{} All specified applications are already assigned",
            "ℹ".blue()
        );
        return Ok(());
    }

    // Warn if using --force
    if force {
        eprintln!(
            "{} Using --force: bypassing duplicate app safety check",
            "⚠".yellow()
        );
    }

    // Build new complete app list
    let mut all_app_ids: Vec<_> = current_ids.into_iter().collect();
    all_app_ids.extend(apps_to_add.clone());

    // Preserve existing users
    let current_user_ids: Vec<_> = team.users.iter().map(|u| u.user_id.clone()).collect();

    let request = UpdateTeamRequest {
        team_id: team_id.clone(),
        organization_id: org_id.clone(),
        name: Some(team.name.clone()),
        user_ids: Some(current_user_ids),
        application_ids: Some(all_app_ids),
    };

    let updated = client.update_team(&org_id, &team_id, request).await?;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": updated,
                "meta": {
                    "added": apps_to_add.len(),
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!(
                "{} Assigned {} application(s) to team \"{}\"",
                "✓".green(),
                apps_to_add.len(),
                updated.name
            );
            eprintln!("→ View team: hawkop team get {}", updated.id);
        }
    }

    Ok(())
}

/// Remove applications from a team
pub async fn remove_app(
    opts: &GlobalOptions,
    team_identifier: &str,
    mut apps: Vec<String>,
    stdin: bool,
    dry_run: bool,
) -> Result<()> {
    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Read from stdin if requested
    if stdin {
        apps.extend(read_stdin_lines()?);
    }

    if apps.is_empty() {
        return Err(crate::error::Error::Other(
            "No applications specified. Provide app names or IDs as arguments or use --stdin."
                .to_string(),
        ));
    }

    // Resolve team ID and get current state - use fresh read to ensure we have latest state before mutation
    let team_id = resolve_team(client.clone(), &org_id, team_identifier).await?;
    let team = client.get_team_fresh(&org_id, &team_id).await?;

    // Resolve app IDs to remove
    let remove_ids: HashSet<_> = resolve_apps(client.clone(), &org_id, &apps)
        .await?
        .into_iter()
        .collect();

    // Get current app IDs and filter out those to remove
    let current_ids: Vec<_> = team
        .applications
        .iter()
        .map(|a| a.application_id.clone())
        .collect();
    let remaining_ids: Vec<_> = current_ids
        .iter()
        .filter(|id| !remove_ids.contains(*id))
        .cloned()
        .collect();

    let actually_removing = current_ids.len() - remaining_ids.len();

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!(
            "Would unassign {} application(s) from team \"{}\":",
            actually_removing, team.name
        );
        for id in &remove_ids {
            if current_ids.contains(id) {
                eprintln!("  • {}", id);
            }
        }
        if actually_removing < remove_ids.len() {
            eprintln!(
                "\n{} {} application(s) not assigned (ignored)",
                "ℹ".blue(),
                remove_ids.len() - actually_removing
            );
        }
        return Ok(());
    }

    if actually_removing == 0 {
        eprintln!(
            "{} None of the specified applications are assigned",
            "ℹ".blue()
        );
        return Ok(());
    }

    // Preserve existing users
    let current_user_ids: Vec<_> = team.users.iter().map(|u| u.user_id.clone()).collect();

    let request = UpdateTeamRequest {
        team_id: team_id.clone(),
        organization_id: org_id.clone(),
        name: Some(team.name.clone()),
        user_ids: Some(current_user_ids),
        application_ids: Some(remaining_ids),
    };

    let updated = client.update_team(&org_id, &team_id, request).await?;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": updated,
                "meta": {
                    "removed": actually_removing,
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!(
                "{} Unassigned {} application(s) from team \"{}\"",
                "✓".green(),
                actually_removing,
                updated.name
            );
            eprintln!("→ View team: hawkop team get {}", updated.id);
        }
    }

    Ok(())
}

/// Replace all team application assignments
pub async fn set_apps(
    opts: &GlobalOptions,
    team_identifier: &str,
    mut apps: Vec<String>,
    stdin: bool,
    dry_run: bool,
    yes: bool,
    force: bool,
) -> Result<()> {
    let (org_id, client) = setup_team_context(opts).await?;
    let format = opts.format;

    // Read from stdin if requested
    if stdin {
        apps.extend(read_stdin_lines()?);
    }

    // Resolve team ID and get current state - use fresh read to ensure we have latest state before mutation
    let team_id = resolve_team(client.clone(), &org_id, team_identifier).await?;
    let team = client.get_team_fresh(&org_id, &team_id).await?;

    // Resolve new app IDs
    let new_app_ids: HashSet<_> = resolve_apps(client.clone(), &org_id, &apps)
        .await?
        .into_iter()
        .collect();

    // Calculate diff
    let current_ids: HashSet<_> = team
        .applications
        .iter()
        .map(|a| a.application_id.clone())
        .collect();
    let to_add: Vec<_> = new_app_ids.difference(&current_ids).cloned().collect();
    let to_remove: Vec<_> = current_ids.difference(&new_app_ids).cloned().collect();
    let unchanged: Vec<_> = new_app_ids.intersection(&current_ids).cloned().collect();

    // Safety check: each app may only belong to one team at a time
    if !force && !to_add.is_empty() {
        let duplicates =
            check_duplicate_app_assignments(client.clone(), &org_id, &team_id, &to_add).await?;

        if !duplicates.is_empty() {
            let dup_list = duplicates
                .iter()
                .map(|(app_name, app_id, other_team)| {
                    format!(
                        "  • \"{}\" (ID: {}) → already in team \"{}\"",
                        app_name, app_id, other_team
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            let first_dup = &duplicates[0];
            return Err(crate::error::Error::Other(format!(
                "Cannot assign app(s) already in other teams:\n\n{}\n\n\
                 Each app may only be assigned to one team at a time.\n\n\
                 → Remove first: hawkop team remove-app \"{}\" \"{}\"\n\
                 → Or use --force to override (not recommended)",
                dup_list, first_dup.2, first_dup.0
            )));
        }
    }

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!("Team \"{}\" application changes:", team.name);
        if !to_add.is_empty() {
            eprintln!("  {} Add: {}", "+".green(), to_add.len());
            for id in &to_add {
                eprintln!("    • {}", id);
            }
        }
        if !to_remove.is_empty() {
            eprintln!("  {} Remove: {}", "-".red(), to_remove.len());
            for id in &to_remove {
                eprintln!("    • {}", id);
            }
        }
        if !unchanged.is_empty() {
            eprintln!("  {} Unchanged: {}", "=".dimmed(), unchanged.len());
        }
        return Ok(());
    }

    // Confirmation if removing apps (unless --yes)
    if !yes && !to_remove.is_empty() {
        eprintln!(
            "{} This will replace application assignments for \"{}\":",
            "⚠".yellow(),
            team.name
        );
        eprintln!("  Add: {} application(s)", to_add.len());
        eprintln!("  Remove: {} application(s)", to_remove.len());
        eprintln!("  Unchanged: {} application(s)", unchanged.len());
        eprintln!();

        let confirm = Confirm::new()
            .with_prompt("Proceed with assignment sync?")
            .default(false)
            .interact()?;

        if !confirm {
            eprintln!("Cancelled.");
            return Ok(());
        }
    }

    // Warn if using --force
    if force && !to_add.is_empty() {
        eprintln!(
            "{} Using --force: bypassing duplicate app safety check",
            "⚠".yellow()
        );
    }

    // Preserve existing users
    let current_user_ids: Vec<_> = team.users.iter().map(|u| u.user_id.clone()).collect();

    let request = UpdateTeamRequest {
        team_id: team_id.clone(),
        organization_id: org_id.clone(),
        name: Some(team.name.clone()),
        user_ids: Some(current_user_ids),
        application_ids: Some(new_app_ids.into_iter().collect()),
    };

    let updated = client.update_team(&org_id, &team_id, request).await?;

    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": updated,
                "meta": {
                    "added": to_add.len(),
                    "removed": to_remove.len(),
                    "unchanged": unchanged.len(),
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!(
                "{} Team \"{}\" applications updated (+{} -{} ={})",
                "✓".green(),
                updated.name,
                to_add.len(),
                to_remove.len(),
                unchanged.len()
            );
            eprintln!("→ View team: hawkop team get {}", updated.id);
        }
    }

    Ok(())
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // looks_like_uuid tests
    // ========================================================================

    #[test]
    fn test_looks_like_uuid_valid() {
        assert!(looks_like_uuid("550e8400-e29b-41d4-a716-446655440000"));
        assert!(looks_like_uuid("00000000-0000-0000-0000-000000000000"));
        assert!(looks_like_uuid("ABCDEF01-2345-6789-ABCD-EF0123456789"));
    }

    #[test]
    fn test_looks_like_uuid_invalid() {
        // Not enough parts
        assert!(!looks_like_uuid("550e8400-e29b-41d4-a716"));
        // Too many parts
        assert!(!looks_like_uuid(
            "550e8400-e29b-41d4-a716-446655440000-extra"
        ));
        // Not hex characters
        assert!(!looks_like_uuid("550e8400-e29b-41d4-a716-44665544000g"));
        // No dashes
        assert!(!looks_like_uuid("550e8400e29b41d4a716446655440000"));
        // Empty string
        assert!(!looks_like_uuid(""));
        // Team name
        assert!(!looks_like_uuid("Security Team"));
        // Email address
        assert!(!looks_like_uuid("user@example.com"));
    }

    #[test]
    fn test_looks_like_uuid_edge_cases() {
        // Single dash (not UUID format)
        assert!(!looks_like_uuid("team-name"));
        // Mixed case is fine for UUID
        assert!(looks_like_uuid("550E8400-e29B-41d4-A716-446655440000"));
    }

    // ========================================================================
    // read_stdin_lines tests
    // ========================================================================

    // Note: read_stdin_lines is difficult to unit test as it reads from actual stdin.
    // Integration testing would use echo piped to the CLI.

    // ========================================================================
    // Integration test notes
    // ========================================================================

    // The resolve_team, resolve_users, and resolve_apps functions require a full
    // CachedStackHawkClient<StackHawkClient> which makes unit testing complex.
    // These functions are best tested through:
    // 1. Integration tests with the actual CLI
    // 2. End-to-end tests against a test organization
    //
    // The functions are covered by the following manual test scenarios:
    // - `hawkop team get "Security Team"` (name resolution)
    // - `hawkop team get <UUID>` (UUID passthrough)
    // - `hawkop team add-user "Team" user@example.com` (email resolution)
}
