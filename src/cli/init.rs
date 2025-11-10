//! Init command implementation

use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Password, Select};

use crate::client::{StackHawkApi, StackHawkClient};
use crate::config::Config;
use crate::error::Result;

/// Run the init command
pub async fn run() -> Result<()> {
    println!("{}", "Welcome to HawkOp!".bold().green());
    println!("Let's set up your StackHawk configuration.\n");

    // Prompt for API key
    let api_key: String = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter your StackHawk API key")
        .interact()?;

    // Authenticate and get JWT
    println!("\n{}", "Authenticating...".cyan());
    let client = StackHawkClient::new(Some(api_key.clone()))?;
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

        if use_org {
            Some(org.id.clone())
        } else {
            None
        }
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

    // Create and save config
    let config = Config {
        api_key: Some(api_key),
        org_id,
        jwt: Some(crate::config::JwtToken {
            token: jwt_token.token,
            expires_at: jwt_token.expires_at,
        }),
        preferences: Default::default(),
    };

    config.save()?;

    let config_path = Config::default_path()?;
    println!(
        "\n{} Configuration saved to: {}",
        "✓".green(),
        config_path.display()
    );

    if let Some(org_id) = &config.org_id {
        println!("  Default organization: {}", org_id.bold());
    }

    println!("\n{}", "You're all set! Try running:".bold());
    println!("  {} - Show configuration status", "hawkop status".cyan());
    println!("  {} - List organizations", "hawkop org list".cyan());

    Ok(())
}
