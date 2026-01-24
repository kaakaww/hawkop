//! Init command implementation

use colored::Colorize;
use dialoguer::{Password, Select, theme::ColorfulTheme};

use crate::cli::args::GlobalOptions;
use crate::client::{AuthApi, ListingApi, StackHawkClient};
use crate::config::{ProfileConfig, ProfiledConfig};
use crate::error::Result;

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

    Ok(())
}
