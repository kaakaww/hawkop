//! HawkOp CLI - Professional companion for the StackHawk DAST platform

use clap::{CommandFactory, Parser};
use clap_complete::env::CompleteEnv;
use clap_complete::generate;

mod cache;
mod cli;
mod client;
mod config;
mod error;
mod models;
mod output;

use cli::{
    AppCommands, AuditCommands, CacheCommands, Cli, Commands, ConfigCommands, OasCommands,
    OrgCommands, PolicyCommands, RepoCommands, ScanCommands, SecretCommands, TeamCommands,
    UserCommands,
};
use error::Result;

fn main() {
    // Handle shell completion requests BEFORE starting tokio runtime.
    // CompleteEnv::complete() will exit if handling a completion request.
    // Our completion functions need their own runtime, which conflicts with tokio::main.
    CompleteEnv::with_factory(Cli::command).complete();

    // Now start the async runtime for normal command execution
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    if let Err(err) = runtime.block_on(run()) {
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
            OrgCommands::List => {
                cli::org::list(cli.format, cli.config.as_deref(), cli.no_cache).await
            }
            OrgCommands::Set { org_id } => {
                cli::org::set(org_id, cli.config.as_deref(), cli.no_cache).await
            }
            OrgCommands::Get => {
                cli::org::get(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    cli.no_cache,
                )
                .await
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
                    cli.no_cache,
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
                    cli.no_cache,
                )
                .await
            }
            ScanCommands::Get {
                scan_id,
                app,
                app_id,
                env,
                plugin_id,
                uri_id,
                message,
                format,
            } => {
                cli::scan::get(
                    format, // Use command-level format (defaults to pretty)
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &scan_id,
                    app.as_deref(),
                    app_id.as_deref(),
                    env.as_deref(),
                    plugin_id.as_deref(),
                    uri_id.as_deref(),
                    message,
                    cli.no_cache,
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
                    cli.no_cache,
                )
                .await
            }
        },
        Commands::Team(team_cmd) => match team_cmd {
            TeamCommands::List {
                pagination,
                filters,
            } => {
                cli::team::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &pagination,
                    &filters,
                    cli.no_cache,
                )
                .await
            }
            TeamCommands::Get { team } => {
                cli::team::get(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &team,
                    cli.no_cache,
                )
                .await
            }
            TeamCommands::Create {
                name,
                users,
                apps,
                dry_run,
                force,
            } => {
                cli::team::create(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &name,
                    users,
                    apps,
                    dry_run,
                    force,
                    cli.no_cache,
                )
                .await
            }
            TeamCommands::Delete { team, yes, dry_run } => {
                cli::team::delete(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &team,
                    yes,
                    dry_run,
                    cli.no_cache,
                )
                .await
            }
            TeamCommands::Rename {
                current,
                new_name,
                dry_run,
            } => {
                cli::team::rename(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &current,
                    &new_name,
                    dry_run,
                    cli.no_cache,
                )
                .await
            }
            TeamCommands::AddUser {
                team,
                users,
                stdin,
                dry_run,
            } => {
                cli::team::add_user(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &team,
                    users,
                    stdin,
                    dry_run,
                    cli.no_cache,
                )
                .await
            }
            TeamCommands::RemoveUser {
                team,
                users,
                stdin,
                dry_run,
            } => {
                cli::team::remove_user(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &team,
                    users,
                    stdin,
                    dry_run,
                    cli.no_cache,
                )
                .await
            }
            TeamCommands::SetUsers {
                team,
                users,
                stdin,
                dry_run,
                yes,
            } => {
                cli::team::set_users(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &team,
                    users,
                    stdin,
                    dry_run,
                    yes,
                    cli.no_cache,
                )
                .await
            }
            TeamCommands::AddApp {
                team,
                apps,
                stdin,
                dry_run,
                force,
            } => {
                cli::team::add_app(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &team,
                    apps,
                    stdin,
                    dry_run,
                    force,
                    cli.no_cache,
                )
                .await
            }
            TeamCommands::RemoveApp {
                team,
                apps,
                stdin,
                dry_run,
            } => {
                cli::team::remove_app(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &team,
                    apps,
                    stdin,
                    dry_run,
                    cli.no_cache,
                )
                .await
            }
            TeamCommands::SetApps {
                team,
                apps,
                stdin,
                dry_run,
                yes,
                force,
            } => {
                cli::team::set_apps(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &team,
                    apps,
                    stdin,
                    dry_run,
                    yes,
                    force,
                    cli.no_cache,
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
                    cli.no_cache,
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
                    cli.no_cache,
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
                    cli.no_cache,
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
                    cli.no_cache,
                )
                .await
            }
        },
        Commands::Secret(secret_cmd) => match secret_cmd {
            SecretCommands::List => {
                cli::secret::list(cli.format, cli.config.as_deref(), cli.no_cache).await
            }
        },
        Commands::Audit(audit_cmd) => match audit_cmd {
            AuditCommands::List { filters } => {
                cli::audit::list(
                    cli.format,
                    cli.org.as_deref(),
                    cli.config.as_deref(),
                    &filters,
                    cli.no_cache,
                )
                .await
            }
        },
        Commands::Cache(cache_cmd) => match cache_cmd {
            CacheCommands::Status => cli::cache::status(cli.format),
            CacheCommands::Clear => cli::cache::clear(cli.format),
            CacheCommands::Path => cli::cache::path(),
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
