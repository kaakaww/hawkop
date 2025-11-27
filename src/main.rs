//! HawkOp CLI - Professional companion for the StackHawk DAST platform

use clap::Parser;

mod cli;
mod client;
mod config;
mod error;
mod output;

use cli::{AppCommands, Cli, Commands, OrgCommands};
use error::Result;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Enable debug logging if requested
    if cli.debug {
        eprintln!("Debug mode enabled");
    }

    match cli.command {
        Commands::Init => cli::init::run(cli.config.as_deref()).await,
        Commands::Status => cli::status::run(cli.config.as_deref()),
        Commands::Version => {
            println!("hawkop version {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Org(org_cmd) => match org_cmd {
            OrgCommands::List => cli::org::list(cli.format, cli.config.as_deref()).await,
            OrgCommands::Set { org_id } => cli::org::set(org_id, cli.config.as_deref()).await,
            OrgCommands::Get => {
                cli::org::get(cli.format, cli.org.as_deref(), cli.config.as_deref()).await
            }
        },
        Commands::App(app_cmd) => match app_cmd {
            AppCommands::List => {
                cli::app::list(cli.format, cli.org.as_deref(), cli.config.as_deref()).await
            }
        },
    }
}
