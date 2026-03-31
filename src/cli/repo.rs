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

    // Check if already linked (by ID only — name-based linking always proceeds)
    if let Some(ref link_id) = new_app_info.id {
        let existing = read_existing_mappings(&resolved_repo);
        if is_already_linked(&existing, link_id) {
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
    }

    if dry_run {
        let existing_count = resolved_repo.app_infos.len();
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
            existing_count
        );
        eprintln!("  Total mappings after: {}", existing_count + 1);
        return Ok(());
    }

    // Delegate to shared read-merge-write helper
    let result = link_app_to_repo(&*ctx.client, org_id, &resolved_repo, &new_app_info).await?;

    match result {
        LinkResult::Linked {
            ref repo_id,
            ref repo_name,
            total_mappings,
        } => match ctx.format {
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "data": {
                        "repoId": repo_id,
                        "repoName": repo_name,
                        "totalMappings": total_mappings,
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
                    "{} Linked to repository \"{}\" ({})",
                    "✓".green(),
                    repo_name,
                    repo_id
                );
                if let Some(ref id) = new_app_info.id {
                    eprintln!("  App ID: {}", id);
                }
                if let Some(ref name) = new_app_info.name {
                    eprintln!("  Created + linked app: \"{}\"", name);
                }
                eprintln!("  Total app mappings: {}", total_mappings);
                eprintln!();
                eprintln!("→ hawkop repo list");
            }
        },
        // AlreadyLinked is handled above before dry-run; this branch is unreachable
        // for the by-ID case but kept for completeness
        LinkResult::AlreadyLinked { ref app_id, .. } => {
            eprintln!(
                "{} Application {} is already linked to repository \"{}\".",
                "ℹ".blue(),
                app_id,
                resolved_repo.name
            );
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
pub(crate) async fn resolve_repo(
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

/// Result of linking an app to a repo.
#[derive(Debug)]
pub(crate) enum LinkResult {
    /// Successfully linked the app to the repo.
    Linked {
        repo_id: String,
        repo_name: String,
        total_mappings: usize,
    },
    /// The app was already linked to the repo.
    AlreadyLinked { repo_id: String, app_id: String },
}

/// Build the list of existing app mappings from a repository, as `RepoAppInfoWrite`.
pub(crate) fn read_existing_mappings(repo: &Repository) -> Vec<RepoAppInfoWrite> {
    repo.app_infos
        .iter()
        .map(|ai| RepoAppInfoWrite {
            id: ai.app_id.clone(),
            name: ai.app_name.clone(),
        })
        .collect()
}

/// Check if an app (by ID) is already linked to a repository.
pub(crate) fn is_already_linked(existing: &[RepoAppInfoWrite], app_id: &str) -> bool {
    existing.iter().any(|a| a.id.as_deref() == Some(app_id))
}

/// Link an application to a repository using read-merge-write.
///
/// Accepts a `RepoAppInfoWrite` so callers can link by ID (`--app-id`) or
/// by name (`--app-name`, which creates a new app during linking).
///
/// This is the shared core logic used by `repo link`, `app create --repo`,
/// and `init` post-setup.
pub(crate) async fn link_app_to_repo(
    client: &(impl ListingApi + RepoApi),
    org_id: &str,
    repo: &Repository,
    new_app: &RepoAppInfoWrite,
) -> Result<LinkResult> {
    let repo_id = repo.id.clone().ok_or_else(|| {
        crate::error::Error::Other("Repository has no ID (unexpected API response).".to_string())
    })?;

    let existing = read_existing_mappings(repo);

    // Check if already linked (only when linking by ID)
    if let Some(ref aid) = new_app.id
        && is_already_linked(&existing, aid)
    {
        return Ok(LinkResult::AlreadyLinked {
            repo_id,
            app_id: aid.clone(),
        });
    }

    // Merge: existing + new
    let mut merged = existing;
    merged.push(new_app.clone());

    debug!(
        "Linking app {:?} to repo {} (total mappings: {})",
        new_app,
        repo_id,
        merged.len()
    );

    let request = ReplaceRepoAppMappingsRequest {
        org_id: org_id.to_string(),
        repo_id: repo_id.clone(),
        app_infos: merged,
    };

    let response = client.replace_repo_app_mappings(request).await?;

    Ok(LinkResult::Linked {
        repo_id,
        repo_name: repo.name.clone(),
        total_mappings: response.app_infos.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::mock::MockStackHawkClient;
    use crate::client::models::RepoAppInfo;

    /// Create a minimal Repository for testing.
    fn make_repo(id: &str, name: &str, app_infos: Vec<RepoAppInfo>) -> Repository {
        Repository {
            id: Some(id.to_string()),
            repo_source: Some("GITHUB".to_string()),
            provider_org_name: Some("kaakaww".to_string()),
            name: name.to_string(),
            open_api_spec_info: None,
            has_generated_open_api_spec: false,
            is_in_attack_surface: true,
            framework_names: vec![],
            sensitive_data_tags: vec![],
            last_commit_timestamp: None,
            last_contributor: None,
            commit_count: 0,
            app_infos,
            insights: vec![],
        }
    }

    fn make_app_info(id: &str, name: &str) -> RepoAppInfo {
        RepoAppInfo {
            app_id: Some(id.to_string()),
            app_name: Some(name.to_string()),
        }
    }

    // ── read_existing_mappings ───────────────────────────────────────────

    #[test]
    fn read_existing_mappings_empty_repo() {
        let repo = make_repo("r1", "my-repo", vec![]);
        let mappings = read_existing_mappings(&repo);
        assert!(mappings.is_empty());
    }

    #[test]
    fn read_existing_mappings_preserves_all() {
        let repo = make_repo(
            "r1",
            "my-repo",
            vec![
                make_app_info("a1", "app-one"),
                make_app_info("a2", "app-two"),
            ],
        );
        let mappings = read_existing_mappings(&repo);
        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].id.as_deref(), Some("a1"));
        assert_eq!(mappings[1].name.as_deref(), Some("app-two"));
    }

    // ── is_already_linked ────────────────────────────────────────────────

    #[test]
    fn is_already_linked_true() {
        let existing = vec![RepoAppInfoWrite {
            id: Some("a1".to_string()),
            name: None,
        }];
        assert!(is_already_linked(&existing, "a1"));
    }

    #[test]
    fn is_already_linked_false() {
        let existing = vec![RepoAppInfoWrite {
            id: Some("a1".to_string()),
            name: None,
        }];
        assert!(!is_already_linked(&existing, "a2"));
    }

    #[test]
    fn is_already_linked_empty() {
        let existing: Vec<RepoAppInfoWrite> = vec![];
        assert!(!is_already_linked(&existing, "a1"));
    }

    // ── resolve_repo ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn resolve_repo_by_id() {
        let client = MockStackHawkClient::new()
            .with_repos(vec![make_repo("r1", "my-repo", vec![])])
            .await;
        let repo = resolve_repo(&client, "org1", Some("r1"), None)
            .await
            .unwrap();
        assert_eq!(repo.name, "my-repo");
    }

    #[tokio::test]
    async fn resolve_repo_by_name() {
        let client = MockStackHawkClient::new()
            .with_repos(vec![make_repo("r1", "my-repo", vec![])])
            .await;
        let repo = resolve_repo(&client, "org1", None, Some("my-repo"))
            .await
            .unwrap();
        assert_eq!(repo.id.as_deref(), Some("r1"));
    }

    #[tokio::test]
    async fn resolve_repo_by_name_case_insensitive() {
        let client = MockStackHawkClient::new()
            .with_repos(vec![make_repo("r1", "My-Repo", vec![])])
            .await;
        let repo = resolve_repo(&client, "org1", None, Some("my-repo"))
            .await
            .unwrap();
        assert_eq!(repo.name, "My-Repo");
    }

    #[tokio::test]
    async fn resolve_repo_not_found() {
        let client = MockStackHawkClient::new()
            .with_repos(vec![make_repo("r1", "other-repo", vec![])])
            .await;
        let err = resolve_repo(&client, "org1", None, Some("missing"))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("No repository found"));
    }

    #[tokio::test]
    async fn resolve_repo_ambiguous() {
        let client = MockStackHawkClient::new()
            .with_repos(vec![
                make_repo("r1", "my-repo", vec![]),
                make_repo("r2", "my-repo", vec![]),
            ])
            .await;
        let err = resolve_repo(&client, "org1", None, Some("my-repo"))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Ambiguous"));
    }

    #[tokio::test]
    async fn resolve_repo_neither_selector() {
        let client = MockStackHawkClient::new();
        let err = resolve_repo(&client, "org1", None, None).await.unwrap_err();
        assert!(err.to_string().contains("Specify exactly one"));
    }

    // ── link_app_to_repo ─────────────────────────────────────────────────

    #[tokio::test]
    async fn link_app_to_repo_success() {
        let repo = make_repo("r1", "my-repo", vec![]);
        let client = MockStackHawkClient::new()
            .with_repos(vec![repo.clone()])
            .await;
        let app_info = RepoAppInfoWrite {
            id: Some("a1".to_string()),
            name: None,
        };
        let result = link_app_to_repo(&client, "org1", &repo, &app_info)
            .await
            .unwrap();
        match result {
            LinkResult::Linked {
                repo_id,
                total_mappings,
                ..
            } => {
                assert_eq!(repo_id, "r1");
                assert_eq!(total_mappings, 1);
            }
            _ => panic!("Expected Linked, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn link_app_to_repo_already_linked() {
        let repo = make_repo("r1", "my-repo", vec![make_app_info("a1", "existing-app")]);
        let client = MockStackHawkClient::new()
            .with_repos(vec![repo.clone()])
            .await;
        let app_info = RepoAppInfoWrite {
            id: Some("a1".to_string()),
            name: None,
        };
        let result = link_app_to_repo(&client, "org1", &repo, &app_info)
            .await
            .unwrap();
        match result {
            LinkResult::AlreadyLinked { app_id, .. } => {
                assert_eq!(app_id, "a1");
            }
            _ => panic!("Expected AlreadyLinked, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn link_app_to_repo_preserves_existing() {
        let repo = make_repo(
            "r1",
            "my-repo",
            vec![
                make_app_info("a1", "app-one"),
                make_app_info("a2", "app-two"),
            ],
        );
        let client = MockStackHawkClient::new()
            .with_repos(vec![repo.clone()])
            .await;
        let app_info = RepoAppInfoWrite {
            id: Some("a3".to_string()),
            name: None,
        };
        let result = link_app_to_repo(&client, "org1", &repo, &app_info)
            .await
            .unwrap();
        match result {
            LinkResult::Linked { total_mappings, .. } => {
                assert_eq!(total_mappings, 3); // 2 existing + 1 new
            }
            _ => panic!("Expected Linked, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn link_app_to_repo_by_name() {
        let repo = make_repo("r1", "my-repo", vec![]);
        let client = MockStackHawkClient::new()
            .with_repos(vec![repo.clone()])
            .await;
        let app_info = RepoAppInfoWrite {
            id: None,
            name: Some("new-app".to_string()),
        };
        // Name-based linking doesn't check already-linked (no ID to compare)
        let result = link_app_to_repo(&client, "org1", &repo, &app_info)
            .await
            .unwrap();
        match result {
            LinkResult::Linked { total_mappings, .. } => {
                assert_eq!(total_mappings, 1);
            }
            _ => panic!("Expected Linked, got {:?}", result),
        }
    }
}
