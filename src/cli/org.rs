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
pub async fn list(format: OutputFormat) -> Result<()> {
    let config = Config::load()?;
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
pub async fn set(org_id: String) -> Result<()> {
    let mut config = Config::load()?;
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
    let org = client.get_org(&org_id).await?;

    // Update config
    config.org_id = Some(org_id.clone());
    config.save()?;

    println!(
        "{} Set default organization to: {} ({})",
        "✓".green(),
        org.name.bold(),
        org_id
    );

    Ok(())
}

/// Run the org get command
pub async fn get(format: OutputFormat) -> Result<()> {
    let config = Config::load()?;
    config.validate_auth()?;

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

    let org = client.get_org(org_id).await?;

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
