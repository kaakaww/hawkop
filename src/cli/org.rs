//! Organization command implementations

use colored::Colorize;
use tabled::Tabled;

use crate::cli::OutputFormat;
use crate::client::{Organization, StackHawkApi, StackHawkClient};
use crate::config::Config;
use crate::error::Result;
use crate::output::{json, table};

/// Organization for table display
#[derive(Tabled)]
struct OrgDisplay {
    #[tabled(rename = "ORG ID")]
    id: String,
    #[tabled(rename = "NAME")]
    name: String,
}

impl From<Organization> for OrgDisplay {
    fn from(org: Organization) -> Self {
        Self {
            id: org.id,
            name: org.name,
        }
    }
}

/// Run the org list command
pub async fn list(format: OutputFormat, config_path: Option<&str>) -> Result<()> {
    let config = Config::load_at(config_path)?;
    config.validate_auth()?;

    let client = StackHawkClient::new(config.api_key.clone())?;

    // Set JWT if available
    if let Some(jwt) = config.jwt {
        client
            .set_jwt(crate::client::JwtToken {
                token: jwt.token,
                expires_at: jwt.expires_at,
            })
            .await;
    }

    let orgs = client.list_orgs().await?;

    match format {
        OutputFormat::Table => {
            let display_orgs: Vec<OrgDisplay> = orgs.into_iter().map(OrgDisplay::from).collect();
            let output = table::format_table(&display_orgs);
            println!("{}", output);
        }
        OutputFormat::Json => {
            let output = json::format_json(&orgs)?;
            println!("{}", output);
        }
    }

    Ok(())
}

/// Run the org set command
pub async fn set(org_id: String, config_path: Option<&str>) -> Result<()> {
    let resolved_path = Config::resolve_path(config_path)?;
    let mut config = Config::load_from(resolved_path.clone())?;
    config.validate_auth()?;

    // Verify org exists
    let client = StackHawkClient::new(config.api_key.clone())?;

    if let Some(jwt) = &config.jwt {
        client
            .set_jwt(crate::client::JwtToken {
                token: jwt.token.clone(),
                expires_at: jwt.expires_at,
            })
            .await;
    }

    println!("Verifying organization...");

    // Get all orgs and verify the provided org_id exists
    let orgs = client.list_orgs().await?;
    let org = orgs
        .iter()
        .find(|o| o.id == org_id)
        .ok_or_else(|| {
            crate::error::ApiError::NotFound(format!(
                "Organization {} not found or you don't have access to it",
                org_id
            ))
        })?;

    // Update config
    config.org_id = Some(org_id.clone());
    config.save_to(resolved_path)?;

    println!(
        "{} Set default organization to: {} ({})",
        "✓".green(),
        org.name.bold(),
        org_id
    );

    Ok(())
}

/// Run the org get command
pub async fn get(
    format: OutputFormat,
    org_override: Option<&str>,
    config_path: Option<&str>,
) -> Result<()> {
    let mut config = Config::load_at(config_path)?;
    config.validate_auth()?;

    if let Some(org) = org_override {
        config.org_id = Some(org.to_string());
    }

    let org_id = config
        .org_id
        .as_ref()
        .ok_or(crate::error::ConfigError::MissingOrgId)?;

    let client = StackHawkClient::new(config.api_key.clone())?;

    if let Some(jwt) = config.jwt {
        client
            .set_jwt(crate::client::JwtToken {
                token: jwt.token,
                expires_at: jwt.expires_at,
            })
            .await;
    }

    // Get all orgs and find the one matching our configured org_id
    let orgs = client.list_orgs().await?;
    let org = orgs
        .iter()
        .find(|o| &o.id == org_id)
        .ok_or_else(|| {
            crate::error::ApiError::NotFound(format!(
                "Organization {} not found or you don't have access to it",
                org_id
            ))
        })?;

    match format {
        OutputFormat::Table => {
            println!("{}", "Current Default Organization".bold());
            println!();
            println!("  ID:   {}", org.id);
            println!("  Name: {}", org.name);
        }
        OutputFormat::Json => {
            let output = json::format_json(&org)?;
            println!("{}", output);
        }
    }

    Ok(())
}
