//! HawkOp CLI - Professional companion for the StackHawk DAST platform

use clap::{CommandFactory, Parser};
use clap_complete::generate;

mod cli;
mod client;
mod config;
mod error;
mod models;
mod output;

use cli::{
    AppCommands, AuditCommands, Cli, Commands, ConfigCommands, OasCommands, OrgCommands,
    PolicyCommands, RepoCommands, ScanCommands, SecretCommands, TeamCommands, UserCommands,
};
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

    // Initialize logging if debug mode is enabled
    if debug {
        env_logger::Builder::new()
            .filter_level(log::LevelFilter::Debug)
            .format_timestamp_millis()
            .init();

        log::debug!("HawkOp v{}", env!("CARGO_PKG_VERSION"));
        log::debug!("Command: {:?}", cli.command);
        log::debug!("Format: {:?}", cli.format);
        log::debug!("Config path: {:?}", cli.config);
        log::debug!("Org override: {:?}", cli.org);
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
            AppCommands::List {
                app_type,
                pagination,
            } => {
                cli::app::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    app_type.as_deref(),
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
            ScanCommands::View { scan_id, args } => {
                cli::scan::view(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &scan_id,
                    &args,
                )
                .await
            }
        },
        Commands::User(user_cmd) => match user_cmd {
            UserCommands::List { pagination } => {
                cli::user::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &pagination,
                )
                .await
            }
        },
        Commands::Team(team_cmd) => match team_cmd {
            TeamCommands::List { pagination } => {
                cli::team::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &pagination,
                )
                .await
            }
        },
        Commands::Policy(policy_cmd) => match policy_cmd {
            PolicyCommands::List { pagination } => {
                cli::policy::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &pagination,
                )
                .await
            }
        },
        Commands::Repo(repo_cmd) => match repo_cmd {
            RepoCommands::List { pagination } => {
                cli::repo::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &pagination,
                )
                .await
            }
        },
        Commands::Oas(oas_cmd) => match oas_cmd {
            OasCommands::List { pagination } => {
                cli::oas::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &pagination,
                )
                .await
            }
        },
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::List { pagination } => {
                cli::config::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &pagination,
                )
                .await
            }
        },
        Commands::Secret(secret_cmd) => match secret_cmd {
            SecretCommands::List => cli::secret::list(cli.format, cli.config.as_deref()).await,
        },
        Commands::Audit(audit_cmd) => match audit_cmd {
            AuditCommands::List { filters } => {
                cli::audit::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &filters,
                )
                .await
            }
        },
        Commands::Completion { shell } => {
            generate(shell, &mut Cli::command(), "hawkop", &mut std::io::stdout());
            Ok(())
        }
    };

    // Log debug info on completion
    if debug {
        if let Err(ref e) = result {
            log::debug!("Error: {:?}", e);
        } else {
            log::debug!("Command completed successfully");
        }
    }

    result
}
