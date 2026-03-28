//! Repository management commands

use colored::Colorize;
use log::debug;

use crate::cli::OutputFormat;
use crate::cli::PaginationArgs;
use crate::cli::args::GlobalOptions;
use crate::cli::handlers::run_list_command;
use crate::client::models::{ReplaceRepoAppMappingsRequest, RepoAppInfoWrite, Repository};
use crate::client::{ListingApi, RepoApi};
use crate::error::Result;
use crate::models::RepoDisplay;

/// Run the repo list command
///
/// Fetches repositories from the organization's attack surface mapping.
pub async fn list(opts: &GlobalOptions, pagination: &PaginationArgs) -> Result<()> {
    run_list_command::<Repository, RepoDisplay, _, _>(
        opts,
        pagination,
        "repositories",
        |client, org_id, params| async move { client.list_repos(&org_id, Some(&params)).await },
    )
    .await
}

/// Run the repo link command (additive: read-merge-write)
///
/// Links an application to a repository while preserving existing mappings.
/// The API uses full-replacement semantics, so we must:
/// 1. Read existing mappings
/// 2. Merge in the new app
/// 3. POST the complete list
#[allow(clippy::too_many_arguments)]
pub async fn link(
    opts: &GlobalOptions,
    repo_id: Option<&str>,
    repo_name: Option<&str>,
    app_id: Option<&str>,
    app_name: Option<&str>,
    env: &str,
    dry_run: bool,
) -> Result<()> {
    use crate::cli::CommandContext;

    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;

    // Resolve repo: by ID or by name
    let resolved_repo = resolve_repo(&*ctx.client, org_id, repo_id, repo_name).await?;
    let resolved_repo_id = resolved_repo.id.clone().ok_or_else(|| {
        crate::error::Error::Other("Repository has no ID (unexpected API response).".to_string())
    })?;

    // Build the app info to add
    let new_app_info = match (app_id, app_name) {
        (Some(id), None) => RepoAppInfoWrite {
            id: Some(id.to_string()),
            name: None,
        },
        (None, Some(name)) => RepoAppInfoWrite {
            id: None,
            name: Some(name.to_string()),
        },
        _ => {
            return Err(crate::error::Error::Other(
                "Specify exactly one of --app-id or --app-name.\n\
                 → --app-id <uuid>   links an existing application\n\
                 → --app-name <name> creates a new application and links it"
                    .to_string(),
            ));
        }
    };

    // Read existing mappings
    let existing_apps: Vec<RepoAppInfoWrite> = resolved_repo
        .app_infos
        .iter()
        .map(|ai| RepoAppInfoWrite {
            id: ai.app_id.clone(),
            name: ai.app_name.clone(),
        })
        .collect();

    // Check if already linked (by ID)
    if let Some(ref link_id) = new_app_info.id
        && existing_apps
            .iter()
            .any(|a| a.id.as_deref() == Some(link_id))
    {
        match ctx.format {
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "data": {
                        "alreadyLinked": true,
                        "repoId": resolved_repo_id,
                        "appId": link_id
                    },
                    "meta": {
                        "version": env!("CARGO_PKG_VERSION"),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
            _ => {
                eprintln!(
                    "{} Application {} is already linked to repository \"{}\".",
                    "ℹ".blue(),
                    link_id,
                    resolved_repo.name
                );
            }
        }
        return Ok(());
    }

    // Merge: existing + new
    let mut merged_apps = existing_apps;
    merged_apps.push(new_app_info.clone());

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!(
            "Would link to repository: \"{}\" (ID: {})",
            resolved_repo.name, resolved_repo_id
        );
        if let Some(ref id) = new_app_info.id {
            eprintln!("  App ID: {}", id);
        }
        if let Some(ref name) = new_app_info.name {
            eprintln!("  New app name: {} (env: {})", name, env);
        }
        eprintln!(
            "  Existing mappings: {} (will be preserved)",
            merged_apps.len() - 1
        );
        eprintln!("  Total mappings after: {}", merged_apps.len());
        return Ok(());
    }

    debug!(
        "Linking app to repo {}: {:?} (total mappings: {})",
        resolved_repo_id,
        new_app_info,
        merged_apps.len()
    );

    let request = ReplaceRepoAppMappingsRequest {
        org_id: org_id.to_string(),
        repo_id: resolved_repo_id.clone(),
        app_infos: merged_apps,
    };

    let response = ctx.client.replace_repo_app_mappings(request).await?;

    match ctx.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": response,
                "meta": {
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!(
                "{} Linked to repository \"{}\" ({})",
                "✓".green(),
                resolved_repo.name,
                resolved_repo_id
            );
            if let Some(ref id) = new_app_info.id {
                eprintln!("  App ID: {}", id);
            }
            if let Some(ref name) = new_app_info.name {
                eprintln!("  Created + linked app: \"{}\"", name);
            }
            eprintln!("  Total app mappings: {}", response.app_infos.len());
            eprintln!();
            eprintln!("→ hawkop repo list");
        }
    }

    Ok(())
}

/// Run the repo set-apps command (full replacement)
///
/// Replaces ALL application mappings for a repository. This is destructive —
/// any existing mappings not in the list are removed.
pub async fn set_apps(
    opts: &GlobalOptions,
    repo_id: &str,
    app_ids: &[String],
    yes: bool,
    dry_run: bool,
) -> Result<()> {
    use crate::cli::CommandContext;

    let ctx = CommandContext::new(opts).await?;
    let org_id = ctx.require_org_id()?;

    // Fetch current repo for confirmation display
    let repo = ctx.client.get_repo(org_id, repo_id).await?;

    let app_infos: Vec<RepoAppInfoWrite> = app_ids
        .iter()
        .map(|id| RepoAppInfoWrite {
            id: Some(id.clone()),
            name: None,
        })
        .collect();

    if dry_run {
        eprintln!("{}", "DRY RUN - no changes will be made".yellow());
        eprintln!();
        eprintln!(
            "Would replace all app mappings for repository: \"{}\" (ID: {})",
            repo.name, repo_id
        );
        eprintln!("  Current mappings: {} apps", repo.app_infos.len());
        eprintln!("  New mappings: {} apps", app_ids.len());
        for id in app_ids {
            eprintln!("    - {}", id);
        }
        return Ok(());
    }

    // Confirmation prompt (unless --yes)
    if !yes {
        eprintln!(
            "This will replace ALL app mappings for repository \"{}\".",
            repo.name.bold()
        );
        eprintln!(
            "  Current: {} app(s)  →  New: {} app(s)",
            repo.app_infos.len(),
            app_ids.len()
        );

        let removed_count = repo
            .app_infos
            .iter()
            .filter(|a| a.app_id.as_ref().is_none_or(|id| !app_ids.contains(id)))
            .count();

        if removed_count > 0 {
            eprintln!(
                "  {} {} existing mapping(s) will be removed.",
                "⚠".yellow(),
                removed_count
            );
        }

        use dialoguer::Confirm;
        let confirmed = Confirm::new()
            .with_prompt("Continue?")
            .default(false)
            .interact()?;

        if !confirmed {
            eprintln!("Cancelled.");
            return Ok(());
        }
    }

    debug!(
        "Replacing app mappings for repo {}: {} apps",
        repo_id,
        app_ids.len()
    );

    let request = ReplaceRepoAppMappingsRequest {
        org_id: org_id.to_string(),
        repo_id: repo_id.to_string(),
        app_infos,
    };

    let response = ctx.client.replace_repo_app_mappings(request).await?;

    match ctx.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "data": response,
                "meta": {
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            eprintln!(
                "{} Replaced app mappings for repository \"{}\"",
                "✓".green(),
                repo.name
            );
            eprintln!("  Total app mappings: {}", response.app_infos.len());
            eprintln!();
            eprintln!("→ hawkop repo list");
        }
    }

    Ok(())
}

/// Resolve a repository by ID or name.
///
/// When using `--repo <name>`, fetches the repo list and finds a match.
/// Errors if both or neither selector is provided.
async fn resolve_repo(
    client: &(impl ListingApi + RepoApi),
    org_id: &str,
    repo_id: Option<&str>,
    repo_name: Option<&str>,
) -> Result<Repository> {
    match (repo_id, repo_name) {
        (Some(id), None) => client.get_repo(org_id, id).await,
        (None, Some(name)) => {
            let repos = client.list_repos(org_id, None).await?;
            let matches: Vec<_> = repos
                .into_iter()
                .filter(|r| r.name.eq_ignore_ascii_case(name))
                .collect();

            match matches.len() {
                0 => Err(crate::error::Error::Other(format!(
                    "No repository found matching \"{}\".\n→ hawkop repo list",
                    name
                ))),
                1 => Ok(matches.into_iter().next().unwrap()),
                n => Err(crate::error::Error::Other(format!(
                    "Ambiguous: {} repositories match \"{}\". Use --repo-id instead.\n→ hawkop repo list -o json | jq '.data[] | select(.name==\"{}\") | .id'",
                    n, name, name
                ))),
            }
        }
        _ => Err(crate::error::Error::Other(
            "Specify exactly one of --repo-id or --repo.\n\
             → --repo-id <uuid>  target repository by ID\n\
             → --repo <name>     find repository by name"
                .to_string(),
        )),
    }
}
