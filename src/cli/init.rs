//! Init command implementation

use colored::Colorize;
use dialoguer::{Confirm, Password, Select, theme::ColorfulTheme};

use crate::cli::args::GlobalOptions;
use crate::client::{AppApi, AuthApi, ListingApi, StackHawkClient};
use crate::config::{ProfileConfig, ProfiledConfig};
use crate::error::Result;
use crate::git;

/// Run the init command
///
/// During interactive setup, the default production API is used. Custom API
/// hosts can be configured manually in the config file or via environment
/// variables after initialization.
pub async fn run(opts: &GlobalOptions) -> Result<()> {
    // Determine which profile to initialize
    let profile_name = opts.profile.as_deref().unwrap_or("default");

    println!("{}", "Welcome to HawkOp!".bold().green());
    if profile_name != "default" {
        println!("Setting up profile: {}\n", profile_name.bold());
    } else {
        println!("Let's set up your StackHawk configuration.\n");
    }

    // Prompt for API key
    let api_key: String = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter your StackHawk API key")
        .interact()?;

    // Authenticate and get JWT (uses custom API host if provided)
    println!("\n{}", "Authenticating...".cyan());
    let client = StackHawkClient::with_host(Some(api_key.clone()), opts.api_host.clone())?;
    let jwt_token = client.authenticate(&api_key).await?;

    println!("{}", "✓ Authentication successful!".green());

    // Get organizations
    println!("\n{}", "Fetching your organizations...".cyan());
    client.set_jwt(jwt_token.clone()).await;
    let orgs = client.list_orgs().await?;

    // Prompt for default organization
    let org_id = if orgs.is_empty() {
        println!("{}", "⚠ No organizations found.".yellow());
        None
    } else if orgs.len() == 1 {
        let org = &orgs[0];
        println!("Found organization: {}", org.name.bold());
        let use_org = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Set this as your default organization?")
            .default(true)
            .interact()?;

        if use_org { Some(org.id.clone()) } else { None }
    } else {
        let org_names: Vec<String> = orgs.iter().map(|o| o.name.clone()).collect();

        println!("Found {} organizations.", orgs.len());
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select your default organization")
            .items(&org_names)
            .default(0)
            .interact_opt()?;

        selection.map(|idx| orgs[idx].id.clone())
    };

    // Create the profile config
    let profile = ProfileConfig {
        api_key: Some(api_key),
        org_id,
        api_host: opts.api_host.clone(),
        jwt: Some(crate::config::JwtToken {
            token: jwt_token.token,
            expires_at: jwt_token.expires_at,
        }),
        preferences: Default::default(),
    };

    // Load or create profiled config
    let mut profiled_config = ProfiledConfig::load_at(opts.config_ref()).unwrap_or_default();

    // Add/update the profile
    if profiled_config.profiles.contains_key(profile_name) {
        // Update existing profile
        *profiled_config.get_profile_mut(profile_name)? = profile;
    } else {
        // Create new profile
        profiled_config.create_profile(profile_name, profile)?;
    }

    // Set as active if it's the only profile or if explicitly specified
    if profiled_config.profiles.len() == 1 || opts.profile.is_some() {
        profiled_config.set_active_profile(profile_name)?;
    }

    profiled_config.save_at(opts.config_ref())?;

    let config_path = ProfiledConfig::resolve_path(opts.config_ref())?;
    println!(
        "\n{} Configuration saved to: {}",
        "✓".green(),
        config_path.display()
    );

    if profile_name != "default" {
        println!("  Profile: {}", profile_name.bold());
    }

    if let Some(org_id) = &profiled_config.get_profile(profile_name)?.org_id {
        println!("  Default organization: {}", org_id.bold());
    }

    println!("\n{}", "You're all set! Try running:".bold());
    println!("  {} - Show configuration status", "hawkop status".cyan());
    println!("  {} - List organizations", "hawkop org list".cyan());

    // ── Post-setup: detect git repo and offer to link ──────────────────
    if let Some(org_id) = &profiled_config.get_profile(profile_name)?.org_id {
        post_setup_repo_detection(&client, org_id).await;
    }

    Ok(())
}

/// After init completes, check if the user is in a git repo and offer to
/// create an app + link it. This drives API Discovery adoption by reducing
/// the gap between "set up auth" and "first scan."
async fn post_setup_repo_detection(client: &StackHawkClient, org_id: &str) {
    let local_repo = match git::detect_local_repo() {
        Some(info) => info,
        None => return, // Not in a git repo — skip silently
    };

    println!();
    println!("📂 Detected git repo: {}", local_repo.full_name().bold());

    // Try to match against platform repos
    let platform_match = match git::match_platform_repo(client, org_id, &local_repo).await {
        Ok(Some(repo)) => Some(repo),
        Ok(None) => None,
        Err(e) => {
            log::debug!("Failed to match platform repo: {}", e);
            None
        }
    };

    if let Some(ref repo) = platform_match {
        // Repo exists in the platform
        let app_count = repo.app_infos.len();
        if app_count > 0 {
            let app_names: Vec<String> = repo
                .app_infos
                .iter()
                .filter_map(|ai| ai.app_name.clone())
                .collect();
            println!(
                "  {} This repo is tracked in your attack surface with {} linked app(s): {}",
                "✓".green(),
                app_count,
                if app_names.is_empty() {
                    "(unnamed)".to_string()
                } else {
                    app_names.join(", ")
                }
            );
        } else {
            println!(
                "  {} This repo is in your attack surface but has no linked apps.",
                "ℹ".blue()
            );
            offer_create_and_link(client, org_id, &local_repo, repo).await;
        }
    } else {
        // Repo not found in platform
        println!(
            "  {} This repo isn't in your attack surface yet.",
            "ℹ".blue()
        );
        println!("  To start scanning, create an app and link it to this repo:");
        println!(
            "  → {} --name {} --env Development --repo {}",
            "hawkop app create".cyan(),
            local_repo.name,
            local_repo.full_name()
        );
    }
}

/// Offer to create an app and link it to a platform repo that has no apps.
async fn offer_create_and_link(
    client: &StackHawkClient,
    org_id: &str,
    local_repo: &git::LocalRepoInfo,
    platform_repo: &crate::client::models::Repository,
) {
    let create = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Create an app and link it to this repo?")
        .default(true)
        .interact_opt();

    let confirmed = match create {
        Ok(Some(true)) => true,
        _ => return,
    };

    if !confirmed {
        return;
    }

    // Use repo name as default app name
    let app_name = &local_repo.name;
    let request = crate::client::models::CreateApplicationRequest {
        name: app_name.to_string(),
        env: "Development".to_string(),
        application_type: Some("STANDARD".to_string()),
        host: None,
        cloud_scan_target_url: None,
        team_id: None,
    };

    println!("\n{}", "Creating application...".cyan());

    let app = match client.create_app(org_id, request).await {
        Ok(app) => app,
        Err(e) => {
            eprintln!("  {} Could not create application: {}", "⚠".yellow(), e);
            return;
        }
    };

    println!(
        "  {} Application \"{}\" created (ID: {})",
        "✓".green(),
        app.name,
        app.id
    );

    // Link to repo
    let app_info = crate::client::models::RepoAppInfoWrite {
        id: Some(app.id.clone()),
        name: None,
    };
    match crate::cli::repo::link_app_to_repo(client, org_id, platform_repo, &app_info).await {
        Ok(crate::cli::repo::LinkResult::Linked { repo_name, .. }) => {
            println!("  {} Linked to repository \"{}\"", "✓".green(), repo_name);
        }
        Ok(crate::cli::repo::LinkResult::AlreadyLinked { .. }) => {
            println!("  {} Already linked.", "ℹ".blue());
        }
        Err(e) => {
            eprintln!("  {} Could not link to repository: {}", "⚠".yellow(), e);
            eprintln!(
                "  → hawkop repo link --app-id {} --repo {}",
                app.id,
                local_repo.full_name()
            );
            return;
        }
    }

    println!();
    println!("  App ID for stackhawk.yml: {}", app.id.bold());
    println!("  → {} to run your first scan", "hawk scan".cyan());
}
