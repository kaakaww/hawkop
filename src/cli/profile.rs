//! Profile command implementations
//!
//! Manages configuration profiles for switching between different StackHawk
//! environments (production, test, localhost, etc.).

use colored::Colorize;
use dialoguer::{Confirm, Password, Select, theme::ColorfulTheme};
use serde::Serialize;

use crate::cli::OutputFormat;
use crate::cli::args::GlobalOptions;
use crate::client::{AuthApi, ListingApi, StackHawkClient};
use crate::config::{ProfileConfig, ProfiledConfig};
use crate::error::Result;
use crate::output::table::format_table;

/// Display model for profile list output
#[derive(Debug, Clone, Serialize, tabled::Tabled)]
pub struct ProfileListItem {
    #[tabled(rename = "")]
    pub active: String,
    #[tabled(rename = "PROFILE")]
    pub name: String,
    #[tabled(rename = "API HOST")]
    pub api_host: String,
    #[tabled(rename = "ORG ID")]
    pub org_id: String,
    #[tabled(rename = "API KEY")]
    pub has_api_key: String,
}

/// List all profiles
pub fn list(opts: &GlobalOptions) -> Result<()> {
    let config = ProfiledConfig::load_at(opts.config_ref())?;
    let active = &config.active_profile;

    let items: Vec<ProfileListItem> = config
        .list_profiles()
        .into_iter()
        .map(|name| {
            let profile = config.profiles.get(name).unwrap();
            ProfileListItem {
                active: if name == active {
                    "*".to_string()
                } else {
                    "".to_string()
                },
                name: name.to_string(),
                api_host: profile
                    .api_host
                    .clone()
                    .unwrap_or_else(|| "api.stackhawk.com".to_string()),
                org_id: profile.org_id.clone().unwrap_or_else(|| "-".to_string()),
                has_api_key: if profile.api_key.is_some() {
                    "yes".to_string()
                } else {
                    "no".to_string()
                },
            }
        })
        .collect();

    match opts.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "active_profile": active,
                "profiles": items,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!("{}", "Configuration Profiles".bold());
            println!();
            if items.is_empty() {
                println!(
                    "No profiles configured. Run {} to create one.",
                    "hawkop init".cyan()
                );
            } else {
                println!("{}", format_table(&items));
                println!();
                println!("Active profile: {}", active.bold());
                println!(
                    "\n{} Switch profiles with: {}",
                    "→".cyan(),
                    "hawkop profile use <name>".cyan()
                );
            }
        }
    }

    Ok(())
}

/// Switch to a different profile
pub fn use_profile(name: &str, opts: &GlobalOptions) -> Result<()> {
    let mut config = ProfiledConfig::load_at(opts.config_ref())?;

    // Check if already active
    if config.active_profile == name {
        println!(
            "{} Profile '{}' is already active.",
            "✓".green(),
            name.bold()
        );
        return Ok(());
    }

    // Set the new active profile (validates existence)
    config.set_active_profile(name)?;
    config.save_at(opts.config_ref())?;

    println!("{} Switched to profile: {}", "✓".green(), name.bold());

    // Show some info about the new profile
    if let Ok(profile) = config.get_profile(name) {
        if let Some(ref host) = profile.api_host {
            println!("  API host: {}", host);
        }
        if let Some(ref org) = profile.org_id {
            println!("  Organization: {}", org);
        }
    }

    Ok(())
}

/// Create a new profile
pub async fn create(name: &str, from: Option<&str>, opts: &GlobalOptions) -> Result<()> {
    let mut config = ProfiledConfig::load_at(opts.config_ref()).unwrap_or_default();

    // Check if profile already exists
    if config.profiles.contains_key(name) {
        return Err(crate::error::ConfigError::ProfileExists(name.to_string()).into());
    }

    let new_profile = if let Some(source_name) = from {
        // Copy from existing profile
        let source = config.get_profile(source_name)?;
        let mut copied = source.clone();
        // Clear JWT since it's environment-specific
        copied.jwt = None;
        println!(
            "{} Copying settings from profile '{}'",
            "→".cyan(),
            source_name
        );
        copied
    } else {
        // Interactive creation
        println!("{}", format!("Creating new profile: {}", name).bold());
        println!();

        // Prompt for API key
        let api_key: String = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your StackHawk API key (or leave empty)")
            .allow_empty_password(true)
            .interact()?;

        let api_key = if api_key.is_empty() {
            None
        } else {
            Some(api_key)
        };

        // Prompt for custom API host
        let use_custom_host = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Use a custom API host? (for dev/test environments)")
            .default(false)
            .interact()?;

        let api_host = if use_custom_host {
            let host: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter API host (e.g., http://localhost:8080)")
                .interact_text()?;
            Some(host)
        } else {
            None
        };

        // Try to authenticate and get orgs if we have an API key
        let org_id = if let Some(ref key) = api_key {
            println!("\n{}", "Authenticating...".cyan());
            let client = StackHawkClient::with_host(Some(key.clone()), api_host.clone())?;

            match client.authenticate(key).await {
                Ok(jwt) => {
                    println!("{}", "✓ Authentication successful!".green());
                    client.set_jwt(jwt).await;

                    // Get organizations
                    println!("{}", "Fetching organizations...".cyan());
                    match client.list_orgs().await {
                        Ok(orgs) if !orgs.is_empty() => {
                            let org_names: Vec<String> = orgs
                                .iter()
                                .map(|o| format!("{} ({})", o.name, o.id))
                                .collect();

                            let selection = Select::with_theme(&ColorfulTheme::default())
                                .with_prompt("Select default organization for this profile")
                                .items(&org_names)
                                .default(0)
                                .interact_opt()?;

                            selection.map(|idx| orgs[idx].id.clone())
                        }
                        Ok(_) => {
                            println!("{}", "⚠ No organizations found.".yellow());
                            None
                        }
                        Err(e) => {
                            println!("{} Failed to fetch organizations: {}", "⚠".yellow(), e);
                            None
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "{} Authentication failed: {}. You can update the API key later.",
                        "⚠".yellow(),
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        ProfileConfig {
            api_key,
            org_id,
            api_host,
            jwt: None,
            preferences: Default::default(),
        }
    };

    config.create_profile(name, new_profile)?;
    config.save_at(opts.config_ref())?;

    println!("\n{} Created profile: {}", "✓".green(), name.bold());
    println!(
        "\n{} Activate with: {}",
        "→".cyan(),
        format!("hawkop profile use {}", name).cyan()
    );

    Ok(())
}

/// Delete a profile
pub fn delete(name: &str, yes: bool, opts: &GlobalOptions) -> Result<()> {
    let mut config = ProfiledConfig::load_at(opts.config_ref())?;

    // Validate the profile exists and can be deleted
    // (delete_profile checks for default and active constraints)
    if !config.profiles.contains_key(name) {
        return Err(crate::error::ConfigError::ProfileNotFound(name.to_string()).into());
    }

    // Confirm deletion
    if !yes {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Delete profile '{}'?", name))
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Cancelled.");
            return Ok(());
        }
    }

    config.delete_profile(name)?;
    config.save_at(opts.config_ref())?;

    println!("{} Deleted profile: {}", "✓".green(), name);

    Ok(())
}

/// Show profile details
pub fn show(name: Option<&str>, opts: &GlobalOptions) -> Result<()> {
    let config = ProfiledConfig::load_at(opts.config_ref())?;

    let profile_name = name.unwrap_or(&config.active_profile);
    let profile = config.get_profile(profile_name)?;

    let is_active = profile_name == config.active_profile;

    match opts.format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "name": profile_name,
                "active": is_active,
                "api_key_configured": profile.api_key.is_some(),
                "api_host": profile.api_host,
                "org_id": profile.org_id,
                "jwt_valid": !profile.is_token_expired(),
                "preferences": {
                    "format": profile.preferences.format,
                    "page_size": profile.preferences.page_size,
                }
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!(
                "{} {}{}",
                "Profile:".bold(),
                profile_name.bold(),
                if is_active {
                    " (active)".green().to_string()
                } else {
                    String::new()
                }
            );
            println!();

            // API key status
            if profile.api_key.is_some() {
                let masked = profile
                    .api_key
                    .as_ref()
                    .map(|k| {
                        if k.len() > 8 {
                            format!("{}...{}", &k[..4], &k[k.len() - 4..])
                        } else {
                            "****".to_string()
                        }
                    })
                    .unwrap_or_default();
                println!("{} API key: {}", "✓".green(), masked);
            } else {
                println!("{} API key not configured", "✗".red());
            }

            // API host
            if let Some(ref host) = profile.api_host {
                println!("{} API host: {} (custom)", "✓".green(), host);
            } else {
                println!("  API host: https://api.stackhawk.com (default)");
            }

            // Organization
            if let Some(ref org) = profile.org_id {
                println!("{} Organization: {}", "✓".green(), org);
            } else {
                println!("{} No organization set", "⚠".yellow());
            }

            // JWT status
            if profile.is_token_expired() {
                println!(
                    "  {} JWT token expired or missing (will refresh on next command)",
                    "⚠".yellow()
                );
            } else {
                println!("{} JWT token valid", "✓".green());
            }

            // Preferences
            println!("\n{}", "Preferences:".bold());
            println!("  Page size: {}", profile.preferences.page_size);
            if let Some(ref fmt) = profile.preferences.format {
                println!("  Default format: {}", fmt);
            }
        }
    }

    Ok(())
}
