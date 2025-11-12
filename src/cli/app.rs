//! Application management commands

use serde::Serialize;
use tabled::Tabled;

use crate::cli::OutputFormat;
use crate::client::{Application, StackHawkApi, StackHawkClient};
use crate::config::Config;
use crate::error::Result;
use crate::output::{json, table};

/// Display format for applications in table view
#[derive(Tabled, Serialize)]
struct AppDisplay {
    #[tabled(rename = "APP ID")]
    id: String,

    #[tabled(rename = "NAME")]
    name: String,
}

impl From<Application> for AppDisplay {
    fn from(app: Application) -> Self {
        Self {
            id: app.id,
            name: app.name,
        }
    }
}

/// Run the app list command
pub async fn list(format: OutputFormat) -> Result<()> {
    let config = Config::load()?;
    config.validate_auth()?;

    // Get org_id from config
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

    let apps = client.list_apps(org_id).await?;

    // Convert to display format
    let display_apps: Vec<AppDisplay> = apps.into_iter().map(|a| a.into()).collect();

    match format {
        OutputFormat::Table => {
            let table = table::format_table(&display_apps);
            println!("{}", table);
        }
        OutputFormat::Json => {
            let output = json::format_json(&display_apps)?;
            println!("{}", output);
        }
    }

    Ok(())
}
