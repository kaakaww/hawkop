//! HawkOp CLI - Professional companion for the StackHawk DAST platform

use clap::Parser;

mod cli;
mod client;
mod config;
mod error;
mod models;
mod output;

use cli::{AppCommands, Cli, Commands, OrgCommands, ScanCommands};
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
    let debug = cli.debug;

    // Print debug info if requested
    if debug {
        eprintln!("[DEBUG] HawkOp v{}", env!("CARGO_PKG_VERSION"));
        eprintln!("[DEBUG] Command: {:?}", cli.command);
        eprintln!("[DEBUG] Format: {:?}", cli.format);
        eprintln!("[DEBUG] Config path: {:?}", cli.config);
        eprintln!("[DEBUG] Org override: {:?}", cli.org);
    }

    let result = match cli.command {
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
            AppCommands::List { pagination } => {
                cli::app::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &pagination,
                )
                .await
            }
        },
        Commands::Scan(scan_cmd) => match scan_cmd {
            ScanCommands::List {
                filters,
                pagination,
            } => {
                cli::scan::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &filters,
                    &pagination,
                )
                .await
            }
        },
    };

    // Print debug info on error
    if debug {
        if let Err(ref e) = result {
            eprintln!("[DEBUG] Error: {:?}", e);
        } else {
            eprintln!("[DEBUG] Command completed successfully");
        }
    }

    result
}
