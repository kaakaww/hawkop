//! Status command implementation

use colored::Colorize;

use crate::config::Config;
use crate::error::{ConfigError, Result};

/// Run the status command
pub fn run(config_path: Option<&str>) -> Result<()> {
    println!("{}", "HawkOp Configuration Status".bold());
    println!();

    // Try to load config
    let config_path = Config::resolve_path(config_path)?;
    println!("Config file: {}", config_path.display());

    match Config::load_from(config_path.clone()) {
        Ok(config) => {
            // Authentication status
            if config.api_key.is_some() {
                println!("{} API key configured", "✓".green());

                // JWT status
                if config.is_token_expired() {
                    println!(
                        "  {} JWT token expired (will be refreshed automatically)",
                        "⚠".yellow()
                    );
                } else {
                    println!("  {} JWT token valid", "✓".green());
                }
            } else {
                println!("{} API key not configured", "✗".red());
            }

            // Organization status
            if let Some(org_id) = &config.org_id {
                println!("{} Default organization: {}", "✓".green(), org_id);
            } else {
                println!("{} No default organization set", "⚠".yellow());
            }

            // Preferences
            println!("\n{}", "Preferences:".bold());
            println!("  Page size: {}", config.preferences.page_size);
            if let Some(format) = &config.preferences.format {
                println!("  Default format: {}", format);
            }
        }
        Err(e) => {
            // Check if it's a ConfigError::NotFound
            match &e {
                crate::error::Error::Config(ConfigError::NotFound) => {
                    println!("{} Configuration not found", "✗".red());
                    println!("\nRun {} to set up HawkOp.", "hawkop init".cyan());
                }
                _ => {
                    println!("{} Error loading configuration: {}", "✗".red(), e);
                }
            }
        }
    }

    Ok(())
}
