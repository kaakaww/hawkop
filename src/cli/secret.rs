//! User secret management commands

use crate::cli::CommandContext;
use crate::cli::args::GlobalOptions;
use crate::client::ListingApi;
use crate::error::Result;
use crate::models::SecretDisplay;
use crate::output::Formattable;

/// Run the secret list command
pub async fn list(opts: &GlobalOptions) -> Result<()> {
    // Secrets are user-scoped, not org-scoped
    let ctx = CommandContext::new(opts).await?;

    let secrets = ctx.client.list_secrets().await?;

    let display_secrets: Vec<SecretDisplay> =
        secrets.into_iter().map(SecretDisplay::from).collect();
    display_secrets.print(ctx.format)?;

    Ok(())
}
