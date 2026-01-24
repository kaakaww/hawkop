//! Status command implementation

use colored::Colorize;

use crate::cli::args::GlobalOptions;
use crate::config::ProfiledConfig;
use crate::error::Result;

/// Run the status command to display configuration status
pub fn run(opts: &GlobalOptions) -> Result<()> {
    println!("{}\n", "HawkOp Configuration Status".bold());

    // Load profiled config
    let config_result = ProfiledConfig::load_at(opts.config_ref());

    match config_result {
        Ok(profiled_config) => {
            // Show config file location
            let config_path = ProfiledConfig::resolve_path(opts.config_ref())?;
            println!("Config file: {}", config_path.display().to_string().cyan());

            // Resolve which profile to show
            let (profile_name, profile) = profiled_config.resolve_profile(opts.profile_ref())?;

            // Show profile info
            let is_active = profile_name == profiled_config.active_profile;
            if is_active {
                println!("Profile: {} {}", profile_name.bold(), "(active)".green());
            } else {
                println!(
                    "Profile: {} {}",
                    profile_name.bold(),
                    "(via --profile flag)".dimmed()
                );
            }

            println!();

            // API key status
            if profile.api_key.is_some() {
                println!("{} API key configured", "✓".green());
            } else {
                println!("{} API key not configured", "✗".red());
                println!("  → Run 'hawkop init' to configure");
            }

            // JWT token status
            if let Some(ref jwt) = profile.jwt {
                if profile.is_token_expired() {
                    println!(
                        "{} JWT token expired (will refresh on next command)",
                        "⚠".yellow()
                    );
                } else {
                    // Calculate remaining time
                    let now = chrono::Utc::now();
                    let expires = jwt.expires_at;
                    let remaining = expires.signed_duration_since(now);
                    let hours = remaining.num_hours();
                    let mins = remaining.num_minutes() % 60;

                    println!(
                        "{} JWT token valid (expires in {}h {}m)",
                        "✓".green(),
                        hours,
                        mins
                    );
                }
            } else {
                println!(
                    "{} JWT token not cached (will authenticate on next command)",
                    "○".dimmed()
                );
            }

            // Organization status
            if let Some(ref org_id) = profile.org_id {
                println!("{} Default organization: {}", "✓".green(), org_id);
            } else {
                println!("{} No default organization set", "○".dimmed());
                println!("  → Run 'hawkop org set <ID>' to set one");
            }

            // API host status (only show if custom)
            if let Some(ref host) = profile.api_host {
                println!("{} Custom API host: {}", "○".dimmed(), host.cyan());
            }

            // Show other profiles
            let other_profiles: Vec<_> = profiled_config
                .list_profiles()
                .into_iter()
                .filter(|p| *p != profile_name)
                .collect();

            if !other_profiles.is_empty() {
                println!();
                println!("Other profiles: {}", other_profiles.join(", ").dimmed());
            }

            println!();
        }
        Err(_) => {
            println!("{} Configuration not found", "✗".red());
            println!();
            println!(
                "Run {} to create a configuration file.",
                "hawkop init".cyan()
            );
            println!();
        }
    }

    Ok(())
}
