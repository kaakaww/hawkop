//! HawkOp CLI - Professional companion for the StackHawk DAST platform

use clap::Parser;

mod cli;
mod client;
mod config;
mod error;
mod output;

use cli::{Cli, Commands, OrgCommands};
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
        println!("Debug mode enabled");
    }

    match cli.command {
        Commands::Init => {
            println!("Init command - to be implemented");
            Ok(())
        }
        Commands::Status => {
            println!("Status command - to be implemented");
            Ok(())
        }
        Commands::Version => {
            println!("hawkop version {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Org(org_cmd) => match org_cmd {
            OrgCommands::List => {
                println!("Org list command - to be implemented");
                Ok(())
            }
            OrgCommands::Set { org_id } => {
                println!("Org set command - to be implemented for: {}", org_id);
                Ok(())
            }
            OrgCommands::Get => {
                println!("Org get command - to be implemented");
                Ok(())
            }
        },
    }
}
