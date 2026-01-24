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

use cli::args::GlobalOptions;
use cli::{
    AppCommands, AuditCommands, CacheCommands, Cli, Commands, ConfigCommands, OasCommands,
    OrgCommands, PolicyCommands, ProfileCommands, RepoCommands, ScanCommands, SecretCommands,
    TeamCommands, UserCommands,
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
        log::debug!("Profile: {:?}", cli.profile);
        log::debug!("Org override: {:?}", cli.org);
        log::debug!("API host: {:?}", cli.api_host);
    }

    // Create GlobalOptions once and pass to all handlers
    let opts = GlobalOptions::from_cli(&cli);

    let result = match cli.command {
        Commands::Init => cli::init::run(&opts).await,
        Commands::Status => cli::status::run(&opts),
        Commands::Version => {
            println!("hawkop version {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Commands::Profile(profile_cmd) => match profile_cmd {
            ProfileCommands::List => cli::profile::list(&opts),
            ProfileCommands::Use { name } => cli::profile::use_profile(&name, &opts),
            ProfileCommands::Create { name, from } => {
                cli::profile::create(&name, from.as_deref(), &opts).await
            }
            ProfileCommands::Delete { name, yes } => cli::profile::delete(&name, yes, &opts),
            ProfileCommands::Show { name } => cli::profile::show(name.as_deref(), &opts),
        },
        Commands::Org(org_cmd) => match org_cmd {
            OrgCommands::List => cli::org::list(&opts).await,
            OrgCommands::Set { org_id } => cli::org::set(&opts, org_id).await,
            OrgCommands::Get => cli::org::get(&opts).await,
        },
        Commands::App(app_cmd) => match app_cmd {
            AppCommands::List {
                app_type,
                pagination,
            } => cli::app::list(&opts, app_type.as_deref(), &pagination).await,
        },
        Commands::Scan(scan_cmd) => match scan_cmd {
            ScanCommands::List {
                filters,
                pagination,
            } => cli::scan::list(&opts, &filters, &pagination).await,
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
                // scan get has its own format override (defaults to pretty)
                cli::scan::get(
                    &opts,
                    format,
                    &scan_id,
                    app.as_deref(),
                    app_id.as_deref(),
                    env.as_deref(),
                    plugin_id.as_deref(),
                    uri_id.as_deref(),
                    message,
                )
                .await
            }
        },
        Commands::User(user_cmd) => match user_cmd {
            UserCommands::List { pagination } => cli::user::list(&opts, &pagination).await,
        },
        Commands::Team(team_cmd) => match team_cmd {
            TeamCommands::List {
                pagination,
                filters,
            } => cli::team::list(&opts, &pagination, &filters).await,
            TeamCommands::Get { team } => cli::team::get(&opts, &team).await,
            TeamCommands::Create {
                name,
                users,
                apps,
                dry_run,
                force,
            } => cli::team::create(&opts, &name, users, apps, dry_run, force).await,
            TeamCommands::Delete { team, yes, dry_run } => {
                cli::team::delete(&opts, &team, yes, dry_run).await
            }
            TeamCommands::Rename {
                current,
                new_name,
                dry_run,
            } => cli::team::rename(&opts, &current, &new_name, dry_run).await,
            TeamCommands::AddUser {
                team,
                users,
                stdin,
                dry_run,
            } => cli::team::add_user(&opts, &team, users, stdin, dry_run).await,
            TeamCommands::RemoveUser {
                team,
                users,
                stdin,
                dry_run,
            } => cli::team::remove_user(&opts, &team, users, stdin, dry_run).await,
            TeamCommands::SetUsers {
                team,
                users,
                stdin,
                dry_run,
                yes,
            } => cli::team::set_users(&opts, &team, users, stdin, dry_run, yes).await,
            TeamCommands::AddApp {
                team,
                apps,
                stdin,
                dry_run,
                force,
            } => cli::team::add_app(&opts, &team, apps, stdin, dry_run, force).await,
            TeamCommands::RemoveApp {
                team,
                apps,
                stdin,
                dry_run,
            } => cli::team::remove_app(&opts, &team, apps, stdin, dry_run).await,
            TeamCommands::SetApps {
                team,
                apps,
                stdin,
                dry_run,
                yes,
                force,
            } => cli::team::set_apps(&opts, &team, apps, stdin, dry_run, yes, force).await,
        },
        Commands::Policy(policy_cmd) => match policy_cmd {
            PolicyCommands::List { pagination } => cli::policy::list(&opts, &pagination).await,
        },
        Commands::Repo(repo_cmd) => match repo_cmd {
            RepoCommands::List { pagination } => cli::repo::list(&opts, &pagination).await,
        },
        Commands::Oas(oas_cmd) => match oas_cmd {
            OasCommands::List { pagination } => cli::oas::list(&opts, &pagination).await,
        },
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::List { pagination } => cli::config::list(&opts, &pagination).await,
        },
        Commands::Secret(secret_cmd) => match secret_cmd {
            SecretCommands::List => cli::secret::list(&opts).await,
        },
        Commands::Audit(audit_cmd) => match audit_cmd {
            AuditCommands::List { filters } => cli::audit::list(&opts, &filters).await,
        },
        Commands::Cache(cache_cmd) => match cache_cmd {
            CacheCommands::Status => cli::cache::status(opts.format),
            CacheCommands::Clear => cli::cache::clear(opts.format),
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
