//! User secret management commands

use crate::cli::{CommandContext, OutputFormat};
use crate::client::StackHawkApi;
use crate::error::Result;
use crate::models::SecretDisplay;
use crate::output::Formattable;

/// Run the secret list command
pub async fn list(format: OutputFormat, config_path: Option<&str>, no_cache: bool) -> Result<()> {
    // Secrets are user-scoped, not org-scoped, so no org_override needed
    let ctx = CommandContext::new(format, None, config_path, no_cache).await?;

    let secrets = ctx.client.list_secrets().await?;

    let display_secrets: Vec<SecretDisplay> =
        secrets.into_iter().map(SecretDisplay::from).collect();
    display_secrets.print(ctx.format)?;

    Ok(())
}
