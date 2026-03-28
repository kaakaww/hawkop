//! CLI command definitions and handlers

use clap::{Parser, Subcommand};
pub use clap_complete::Shell;

use completions::{
    app_name_candidates, plugin_id_candidates, scan_id_candidates, team_name_candidates,
    uri_id_candidates, user_email_candidates,
};

pub mod app;
pub mod args;
pub mod audit;
pub mod cache;
pub mod completions;
pub mod config;
pub mod context;
pub mod env;
pub mod handlers;
pub mod init;
pub mod oas;
pub mod org;
pub mod policy;
pub mod profile;
pub mod repo;
pub mod run;
pub mod scan;
pub mod secret;
pub mod status;
pub mod team;
pub mod user;

pub use args::{AuditFilterArgs, OutputFormat, PaginationArgs, ScanFilterArgs, SortDir};
use clap::Args;

/// Team list filters for narrowing down results
#[derive(Debug, Clone, Args, Default)]
pub struct TeamFilterArgs {
    /// Filter by team name (substring match, case-insensitive)
    #[arg(long)]
    pub name: Option<String>,

    /// Filter by member email (teams containing this user)
    #[arg(long)]
    pub member: Option<String>,

    /// Filter by app name (teams assigned to this app)
    #[arg(long)]
    pub app: Option<String>,
}

pub use context::CommandContext;

/// HawkOp CLI - Professional companion for the StackHawk DAST platform
#[derive(Parser, Debug)]
#[command(name = "hawkop")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Output format (pretty, table, json)
    #[arg(
        long,
        global = true,
        env = "HAWKOP_FORMAT",
        default_value = "pretty",
        hide_env = true,
        hide_possible_values = true
    )]
    pub format: OutputFormat,

    /// Override default organization
    #[arg(long, global = true, env = "HAWKOP_ORG_ID", hide_env = true)]
    pub org: Option<String>,

    /// Override config file location
    #[arg(long, global = true, env = "HAWKOP_CONFIG", hide_env = true)]
    pub config: Option<String>,

    /// Configuration profile to use (for switching orgs, users, or API keys)
    #[arg(
        long,
        short = 'P',
        global = true,
        env = "HAWKOP_PROFILE",
        hide_env = true
    )]
    pub profile: Option<String>,

    /// Enable debug logging
    #[arg(long, global = true, env = "HAWKOP_DEBUG", hide_env = true)]
    pub debug: bool,

    /// Bypass cache, fetch fresh data from API
    #[arg(long, global = true, env = "HAWKOP_NO_CACHE", hide_env = true)]
    pub no_cache: bool,

    /// Custom API host for development/testing (hidden developer option)
    ///
    /// Overrides the default StackHawk API host. The v1 and v2 paths are
    /// computed automatically (e.g., "http://localhost:8080" becomes
    /// "http://localhost:8080/api/v1" and "http://localhost:8080/api/v2").
    #[arg(long = "api-host", global = true, env = "HAWKOP_API_HOST", hide = true)]
    pub api_host: Option<String>,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize HawkOp configuration
    Init,

    /// Show authentication and configuration status
    Status,

    /// Display version information
    Version,

    /// Manage organizations
    #[command(subcommand)]
    Org(OrgCommands),

    /// Manage applications
    #[command(subcommand)]
    App(AppCommands),

    /// View and manage scans
    #[command(subcommand)]
    Scan(ScanCommands),

    /// Run hosted scans (start, stop, status)
    #[command(subcommand)]
    Run(RunCommands),

    /// List organization users/members
    #[command(subcommand)]
    User(UserCommands),

    /// List organization teams
    #[command(subcommand)]
    Team(TeamCommands),

    /// Manage scan policies
    #[command(subcommand)]
    Policy(PolicyCommands),

    /// List repositories in attack surface
    #[command(subcommand)]
    Repo(RepoCommands),

    /// List hosted OpenAPI specifications
    #[command(subcommand)]
    Oas(OasCommands),

    /// List scan configurations
    #[command(subcommand)]
    Config(ConfigCommands),

    /// List user secrets
    #[command(subcommand)]
    Secret(SecretCommands),

    /// View organization audit log
    #[command(subcommand)]
    Audit(AuditCommands),

    /// Manage application environments
    #[command(subcommand)]
    Env(EnvCommands),

    /// Manage local response cache
    #[command(subcommand)]
    Cache(CacheCommands),

    /// Manage configuration profiles (for different orgs, users, or API keys)
    #[command(subcommand, visible_alias = "profiles")]
    Profile(ProfileCommands),

    /// Generate shell completions (static)
    #[command(after_help = "\
Static completions (subcommands/flags only):
  bash:   hawkop completion bash > /etc/bash_completion.d/hawkop
  zsh:    hawkop completion zsh > \"${fpath[1]}/_hawkop\"
  fish:   hawkop completion fish > ~/.config/fish/completions/hawkop.fish

Dynamic completions (includes scan IDs, app names via API):
  bash:   echo 'source <(COMPLETE=bash hawkop)' >> ~/.bashrc
  zsh:    echo 'source <(COMPLETE=zsh hawkop)' >> ~/.zshrc
  fish:   echo 'COMPLETE=fish hawkop | source' >> ~/.config/fish/config.fish

Note: Dynamic completions query the StackHawk API when you press TAB.
Re-source completions after upgrading hawkop.")]
    Completion {
        /// Shell to generate completions for (static only)
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// Organization management subcommands
#[derive(Subcommand, Debug)]
pub enum OrgCommands {
    /// List all accessible organizations
    List,

    /// Set default organization
    Set {
        /// Organization ID to set as default
        org_id: String,
    },

    /// Show current default organization
    Get,
}

/// Application management subcommands
#[derive(Subcommand, Debug)]
pub enum AppCommands {
    /// List all applications in the current organization
    List {
        /// Filter by application type (cloud, standard)
        #[arg(long = "type", short = 't')]
        app_type: Option<String>,

        #[command(flatten)]
        pagination: PaginationArgs,
    },

    /// Create a new application in the current organization
    #[command(after_help = "EXAMPLES:\n  \
            hawkop app create --name my-api --env Development\n  \
            hawkop app create --name my-api --env prod --type cloud --host https://api.example.com\n  \
            hawkop app create --name my-api --env dev --team-id <uuid>\n  \
            hawkop app create --name my-api --format json | jq '.data.applicationId'\n\n\
        OUTPUT:\n  \
            Table/pretty: prints application ID to stdout (pipeable).\n  \
            JSON: prints full application object wrapped in {data, meta}.")]
    Create {
        /// Application name (required)
        #[arg(long, short = 'n')]
        name: String,

        /// Initial environment name
        #[arg(long, short = 'e', default_value = "Development")]
        env: String,

        /// Application type: standard (default) or cloud
        #[arg(long = "type", short = 't', default_value = "standard")]
        app_type: String,

        /// Application host URL (e.g., http://localhost:8080)
        #[arg(long)]
        host: Option<String>,

        /// Cloud scan target URL (required for cloud type apps)
        #[arg(long = "cloud-url")]
        cloud_scan_target_url: Option<String>,

        /// Team ID to assign the new application to
        #[arg(long)]
        team_id: Option<String>,

        /// Preview without creating
        #[arg(long, short = 'N')]
        dry_run: bool,
    },

    /// Get application details by ID or name
    #[command(after_help = "EXAMPLES:\n  \
            hawkop app get <app-id>\n  \
            hawkop app get --name my-api\n  \
            hawkop app get <app-id> --format json | jq '.data'")]
    Get {
        /// Application ID (UUID)
        #[arg(group = "app_selector")]
        app_id: Option<String>,

        /// Application name (resolved via API)
        #[arg(long, short = 'n', group = "app_selector")]
        name: Option<String>,
    },

    /// Rename an existing application
    #[command(after_help = "EXAMPLES:\n  \
            hawkop app update <app-id> --name new-name\n  \
            hawkop app update <app-id> --name new-name --dry-run")]
    Update {
        /// Application ID (UUID)
        app_id: String,

        /// New application name
        #[arg(long, short = 'n')]
        name: String,

        /// Preview without making changes
        #[arg(long, short = 'N')]
        dry_run: bool,
    },

    /// Delete an application (destructive)
    #[command(after_help = "EXAMPLES:\n  \
            hawkop app delete <app-id>\n  \
            hawkop app delete <app-id> --yes")]
    Delete {
        /// Application ID (UUID)
        app_id: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

/// Scan management subcommands
#[derive(Subcommand, Debug)]
pub enum ScanCommands {
    /// List recent scans across all applications
    List {
        #[command(flatten)]
        filters: ScanFilterArgs,

        #[command(flatten)]
        pagination: PaginationArgs,
    },

    /// Get scan details with optional drill-down
    #[command(
        visible_alias = "g",
        after_help = "EXAMPLES:\n  \
            hawkop scan get                          # Latest scan (overview + alerts)\n  \
            hawkop scan get --app myapp              # Latest for app (by name)\n  \
            hawkop scan get --app-id <uuid>          # Latest for app (by ID)\n  \
            hawkop scan get abc123                   # Specific scan\n  \
            hawkop scan get abc123 --plugin-id 40012 # Plugin detail\n  \
            hawkop scan get abc123 --uri-id xyz -m   # Finding with HTTP message\n  \
            hawkop scan get --detail full --format json    # Full detail for AI agents\n  \
            hawkop scan get --app myapp --detail full --max-findings 10\n\n\
        DETAIL LEVELS:\n  \
            (default)  Overview with alerts table\n  \
            full       Complete findings with HTTP messages, evidence,\n  \
                       remediation advice, and validation commands.\n  \
                       Best with --format json for AI agent consumption."
    )]
    Get {
        /// Scan ID (UUID) or "latest" - defaults to latest if omitted
        #[arg(default_value = "latest", add = scan_id_candidates())]
        scan_id: String,

        /// Filter by application name (only with "latest")
        #[arg(long, short = 'a', conflicts_with = "app_id", add = app_name_candidates())]
        app: Option<String>,

        /// Filter by application ID (only with "latest")
        #[arg(long = "app-id")]
        app_id: Option<String>,

        /// Filter by environment (only with "latest")
        #[arg(long, short = 'e')]
        env: Option<String>,

        /// Detail level: "full" fetches all findings with HTTP messages and remediation
        #[arg(long, short = 'd')]
        detail: Option<String>,

        /// Maximum number of findings to include (sorted by severity, highest first)
        #[arg(long, default_value = "100")]
        max_findings: usize,

        /// Maximum response body size in bytes before truncation (default: 10KB)
        #[arg(long, default_value = "10240")]
        max_body_size: usize,

        /// Show detail for specific plugin/vulnerability type
        #[arg(long = "plugin-id", short = 'p', add = plugin_id_candidates())]
        plugin_id: Option<String>,

        /// Show detail for specific URI/finding (unique within scan)
        #[arg(long = "uri-id", short = 'u', add = uri_id_candidates())]
        uri_id: Option<String>,

        /// Include HTTP request/response (requires --uri-id)
        #[arg(long, short = 'm', requires = "uri_id")]
        message: bool,

        /// Output format: pretty (default), table, json
        #[arg(long, short = 'o', default_value = "pretty")]
        format: OutputFormat,
    },
}

/// Run (hosted scan control) subcommands
#[derive(Subcommand, Debug)]
pub enum RunCommands {
    /// Start a hosted scan for an application
    #[command(after_help = "EXAMPLES:\n  \
            hawkop run start --app myapp              # Start scan for app by name\n  \
            hawkop run start --app <uuid>             # Start scan for app by ID\n  \
            hawkop run start --app myapp --watch      # Start and watch progress\n  \
            hawkop run start --app myapp --env prod   # Scan specific environment\n  \
            hawkop run start --app myapp --config ci  # Use specific scan config")]
    Start {
        /// Application name or ID
        #[arg(long, short = 'a', required = true, add = app_name_candidates())]
        app: String,

        /// Environment to scan (optional)
        #[arg(long, short = 'e')]
        env: Option<String>,

        /// Scan configuration name to use (optional)
        #[arg(long, short = 'c')]
        config: Option<String>,

        /// Watch scan progress after starting
        #[arg(long, short = 'w')]
        watch: bool,
    },

    /// Stop a running hosted scan
    #[command(after_help = "EXAMPLES:\n  \
            hawkop run stop --app myapp       # Stop with confirmation\n  \
            hawkop run stop --app myapp --yes # Skip confirmation")]
    Stop {
        /// Application name or ID
        #[arg(long, short = 'a', required = true, add = app_name_candidates())]
        app: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Get the status of a hosted scan
    #[command(after_help = "EXAMPLES:\n  \
            hawkop run status --app myapp           # Check current status\n  \
            hawkop run status --app myapp --watch   # Auto-refresh status\n  \
            hawkop run status --app myapp -w -i 10  # Refresh every 10s")]
    Status {
        /// Application name or ID
        #[arg(long, short = 'a', required = true, add = app_name_candidates())]
        app: String,

        /// Watch status with auto-refresh
        #[arg(long, short = 'w')]
        watch: bool,

        /// Refresh interval in seconds (default: 5)
        #[arg(long, short = 'i', default_value = "5")]
        interval: u64,
    },
}

/// User management subcommands
#[derive(Subcommand, Debug)]
pub enum UserCommands {
    /// List organization members
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
}

/// Team management subcommands
#[derive(Subcommand, Debug)]
pub enum TeamCommands {
    /// List organization teams
    #[command(
        visible_alias = "ls",
        after_help = "EXAMPLES:\n  \
            hawkop team list                         # List all teams\n  \
            hawkop team list --format json           # JSON for scripting\n  \
            hawkop team list --name Security         # Filter by name substring\n  \
            hawkop team list --member alice@ex.com   # Teams with this member\n  \
            hawkop team list --app \"Web App\"         # Teams assigned to app"
    )]
    List {
        #[command(flatten)]
        pagination: PaginationArgs,

        #[command(flatten)]
        filters: TeamFilterArgs,
    },

    /// Get team details with members and applications
    #[command(
        visible_alias = "g",
        after_help = "EXAMPLES:\n  \
            hawkop team get \"Security Team\"   # By name\n  \
            hawkop team get abc123              # By ID\n  \
            hawkop team get abc123 --format json | jq '.data.users'"
    )]
    Get {
        /// Team ID or name
        #[arg(add = team_name_candidates())]
        team: String,
    },

    /// Create a new team
    #[command(after_help = "EXAMPLES:\n  \
            hawkop team create \"New Team\"                               # Empty team\n  \
            hawkop team create \"Dev Team\" --users alice@ex.com          # With initial member\n  \
            hawkop team create \"Team\" -u alice@ex.com -a \"Web App\"      # With member and app\n  \
            hawkop team create \"Test\" --dry-run                         # Preview only\n\n\
        SAFETY:\n  \
            Apps can only belong to one team at a time. Attempting to assign\n  \
            an app that's already in another team will fail unless --force is used.")]
    Create {
        /// Team name
        name: String,
        /// Initial members (email or user ID), comma-separated or repeated
        #[arg(long, short = 'u', value_delimiter = ',', add = user_email_candidates())]
        users: Option<Vec<String>>,
        /// Initial applications (name or ID), comma-separated or repeated
        #[arg(long, short = 'a', value_delimiter = ',', add = app_name_candidates())]
        apps: Option<Vec<String>>,
        /// Preview without creating
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Allow duplicate app assignments (not recommended - can cause API issues)
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Delete a team
    #[command(after_help = "EXAMPLES:\n  \
            hawkop team delete \"Old Team\"     # With confirmation\n  \
            hawkop team delete abc123 --yes    # Skip confirmation\n  \
            hawkop team delete abc123 --dry-run # Preview what would happen")]
    Delete {
        /// Team ID or name
        #[arg(add = team_name_candidates())]
        team: String,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
        /// Preview without deleting
        #[arg(long, short = 'n')]
        dry_run: bool,
    },

    /// Rename a team
    #[command(after_help = "EXAMPLES:\n  \
            hawkop team rename \"Old Name\" \"New Name\"\n  \
            hawkop team rename abc123 \"Renamed\" --dry-run")]
    Rename {
        /// Team ID or current name
        #[arg(add = team_name_candidates())]
        current: String,
        /// New team name
        new_name: String,
        /// Preview without renaming
        #[arg(long, short = 'n')]
        dry_run: bool,
    },

    /// Add users to a team
    #[command(
        visible_alias = "add-member",
        after_help = "EXAMPLES:\n  \
            hawkop team add-user \"Security\" alice@ex.com\n  \
            hawkop team add-user abc123 user1@ex.com,user2@ex.com\n  \
            cat users.txt | hawkop team add-user \"Team\" --stdin\n  \
            hawkop team add-user \"Team\" alice@ex.com --dry-run"
    )]
    AddUser {
        /// Team ID or name
        #[arg(add = team_name_candidates())]
        team: String,
        /// Users to add (email or user ID), comma-separated or repeated
        #[arg(required_unless_present = "stdin", value_delimiter = ',', add = user_email_candidates())]
        users: Vec<String>,
        /// Read users from stdin (one per line)
        #[arg(long)]
        stdin: bool,
        /// Preview without adding
        #[arg(long, short = 'n')]
        dry_run: bool,
    },

    /// Remove users from a team
    #[command(
        visible_alias = "remove-member",
        after_help = "EXAMPLES:\n  \
            hawkop team remove-user \"Security\" alice@ex.com\n  \
            hawkop team remove-user abc123 user1@ex.com,user2@ex.com\n  \
            cat users.txt | hawkop team remove-user \"Team\" --stdin\n  \
            hawkop team remove-user \"Team\" alice@ex.com --dry-run"
    )]
    RemoveUser {
        /// Team ID or name
        #[arg(add = team_name_candidates())]
        team: String,
        /// Users to remove (email or user ID), comma-separated or repeated
        #[arg(required_unless_present = "stdin", value_delimiter = ',', add = user_email_candidates())]
        users: Vec<String>,
        /// Read users from stdin (one per line)
        #[arg(long)]
        stdin: bool,
        /// Preview without removing
        #[arg(long, short = 'n')]
        dry_run: bool,
    },

    /// Replace all team members (for SCIM sync)
    #[command(
        visible_alias = "sync-users",
        after_help = "EXAMPLES:\n  \
            hawkop team set-users \"Team\" alice@ex.com,bob@ex.com\n  \
            cat members.txt | hawkop team set-users \"Team\" --stdin\n  \
            hawkop team set-users \"Team\" --stdin --dry-run < idp-export.txt\n  \
            hawkop team set-users \"Team\" a@ex.com,b@ex.com --yes  # No confirm"
    )]
    SetUsers {
        /// Team ID or name
        #[arg(add = team_name_candidates())]
        team: String,
        /// Complete list of users (replaces existing), comma-separated or repeated
        #[arg(required_unless_present = "stdin", value_delimiter = ',', add = user_email_candidates())]
        users: Vec<String>,
        /// Read users from stdin (one per line)
        #[arg(long)]
        stdin: bool,
        /// Preview changes without applying
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Assign applications to a team
    #[command(after_help = "EXAMPLES:\n  \
            hawkop team add-app \"Security\" \"Web App\"\n  \
            hawkop team add-app abc123 app1,app2\n  \
            cat apps.txt | hawkop team add-app \"Team\" --stdin\n  \
            hawkop team add-app \"Team\" \"Web App\" --dry-run\n\n\
SAFETY:\n  \
            Apps can only belong to one team at a time. Attempting to assign\n  \
            an app that's already in another team will fail unless --force is used.")]
    AddApp {
        /// Team ID or name
        #[arg(add = team_name_candidates())]
        team: String,
        /// Applications to assign (name or ID), comma-separated or repeated
        #[arg(required_unless_present = "stdin", value_delimiter = ',', add = app_name_candidates())]
        apps: Vec<String>,
        /// Read apps from stdin (one per line)
        #[arg(long)]
        stdin: bool,
        /// Preview without assigning
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Allow duplicate app assignments (not recommended - can cause API issues)
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Remove applications from a team
    #[command(after_help = "EXAMPLES:\n  \
            hawkop team remove-app \"Security\" \"Old App\"\n  \
            hawkop team remove-app abc123 app1,app2\n  \
            cat apps.txt | hawkop team remove-app \"Team\" --stdin\n  \
            hawkop team remove-app \"Team\" \"Old App\" --dry-run")]
    RemoveApp {
        /// Team ID or name
        #[arg(add = team_name_candidates())]
        team: String,
        /// Applications to unassign (name or ID), comma-separated or repeated
        #[arg(required_unless_present = "stdin", value_delimiter = ',', add = app_name_candidates())]
        apps: Vec<String>,
        /// Read apps from stdin (one per line)
        #[arg(long)]
        stdin: bool,
        /// Preview without removing
        #[arg(long, short = 'n')]
        dry_run: bool,
    },

    /// Replace all team application assignments
    #[command(
        visible_alias = "sync-apps",
        after_help = "EXAMPLES:\n  \
            hawkop team set-apps \"Team\" \"App1\",\"App2\"\n  \
            cat apps.txt | hawkop team set-apps \"Team\" --stdin\n  \
            hawkop team set-apps \"Team\" app1,app2 --dry-run\n  \
            hawkop team set-apps \"Team\" app1,app2 --yes\n\n\
SAFETY:\n  \
            Apps can only belong to one team at a time. Attempting to assign\n  \
            an app that's already in another team will fail unless --force is used."
    )]
    SetApps {
        /// Team ID or name
        #[arg(add = team_name_candidates())]
        team: String,
        /// Complete list of applications (replaces existing), comma-separated or repeated
        #[arg(required_unless_present = "stdin", value_delimiter = ',', add = app_name_candidates())]
        apps: Vec<String>,
        /// Read apps from stdin (one per line)
        #[arg(long)]
        stdin: bool,
        /// Preview changes without applying
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
        /// Allow duplicate app assignments (not recommended - can cause API issues)
        #[arg(long, short = 'f')]
        force: bool,
    },
}

/// Policy management subcommands
#[derive(Subcommand, Debug)]
pub enum PolicyCommands {
    /// List scan policies (StackHawk preset and organization custom)
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
}

/// Repository management subcommands
#[derive(Subcommand, Debug)]
pub enum RepoCommands {
    /// List repositories in the organization's attack surface
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },

    /// Link an application to a repository (additive, preserves existing mappings)
    #[command(after_help = "EXAMPLES:\n  \
            hawkop repo link --repo-id <uuid> --app-id <uuid>         # Link existing app\n  \
            hawkop repo link --repo-id <uuid> --app-name \"my-api\"     # Create new app + link\n  \
            hawkop repo link --repo <name> --app-id <uuid>            # Find repo by name\n\n\
        The API replaces all app mappings for a repo, so this command reads\n\
        existing mappings first, merges in the new app, then writes the full list.")]
    Link {
        /// Repository ID (UUID)
        #[arg(long, group = "repo_selector")]
        repo_id: Option<String>,

        /// Repository name (resolved via API)
        #[arg(long = "repo", group = "repo_selector")]
        repo_name: Option<String>,

        /// Existing application ID to link
        #[arg(long, group = "app_selector")]
        app_id: Option<String>,

        /// New application name (creates app + links)
        #[arg(long, group = "app_selector")]
        app_name: Option<String>,

        /// Environment for new app (only with --app-name)
        #[arg(long, short = 'e', default_value = "Development")]
        env: String,

        /// Preview without making changes
        #[arg(long, short = 'N')]
        dry_run: bool,
    },

    /// Replace all application mappings for a repository (full replacement)
    #[command(after_help = "EXAMPLES:\n  \
            hawkop repo set-apps --repo-id <uuid> --app-ids <id1>,<id2> --yes\n\n\
        WARNING: This replaces ALL app mappings. Existing mappings not in the\n\
        list will be removed. Use 'repo link' for additive operations.")]
    SetApps {
        /// Repository ID (UUID)
        #[arg(long)]
        repo_id: String,

        /// Comma-separated application IDs
        #[arg(long, value_delimiter = ',')]
        app_ids: Vec<String>,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,

        /// Preview without making changes
        #[arg(long, short = 'N')]
        dry_run: bool,
    },
}

/// OAS management subcommands
#[derive(Subcommand, Debug)]
pub enum OasCommands {
    /// List hosted OpenAPI specifications
    #[command(visible_alias = "ls")]
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },

    /// Get OpenAPI specification content
    #[command(after_help = "EXAMPLES:\n  \
            hawkop oas get <oas-id>              # Display OAS content (JSON)\n  \
            hawkop oas get <oas-id> -o spec.json # Save to file")]
    Get {
        /// OAS ID (UUID)
        oas_id: String,

        /// Output file path (optional, prints to stdout if not specified)
        #[arg(long, short = 'o')]
        output: Option<String>,
    },

    /// List OpenAPI specs mapped to an application
    #[command(after_help = "EXAMPLES:\n  \
            hawkop oas mappings --app myapp      # List by app name\n  \
            hawkop oas mappings --app <uuid>     # List by app ID")]
    Mappings {
        /// Application name or ID
        #[arg(long, short = 'a', required = true, add = app_name_candidates())]
        app: String,
    },
}

/// Configuration management subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// List scan configurations
    #[command(
        visible_alias = "ls",
        after_help = "EXAMPLES:\n  \
            hawkop config list                    # List all org configs"
    )]
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },

    /// Get a scan configuration's content
    #[command(after_help = "EXAMPLES:\n  \
            hawkop config get myconfig            # Display config content\n  \
            hawkop config get myconfig -o out.yml # Save to file")]
    Get {
        /// Configuration name
        name: String,

        /// Output file path (optional, prints to stdout if not specified)
        #[arg(long, short = 'o')]
        output: Option<String>,
    },

    /// Create or update a scan configuration
    #[command(after_help = "EXAMPLES:\n  \
            hawkop config set myconfig -f config.yml  # Create/update config")]
    Set {
        /// Configuration name
        name: String,

        /// YAML configuration file to upload
        #[arg(long, short = 'f', required = true)]
        file: String,
    },

    /// Delete a scan configuration
    #[command(after_help = "EXAMPLES:\n  \
            hawkop config delete myconfig         # Delete with confirmation\n  \
            hawkop config delete myconfig --yes   # Delete without confirmation")]
    Delete {
        /// Configuration name
        name: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Rename a scan configuration
    #[command(after_help = "EXAMPLES:\n  \
            hawkop config rename oldname newname  # Rename config")]
    Rename {
        /// Current configuration name
        old_name: String,

        /// New configuration name
        new_name: String,
    },

    /// Validate a scan configuration
    #[command(after_help = "EXAMPLES:\n  \
            hawkop config validate -f stackhawk.yml  # Validate local file\n  \
            hawkop config validate myconfig          # Validate stored config")]
    Validate {
        /// Configuration name (to validate stored config)
        #[arg(conflicts_with = "file")]
        name: Option<String>,

        /// YAML file to validate (local file)
        #[arg(long, short = 'f', conflicts_with = "name")]
        file: Option<String>,
    },
}

/// Secret management subcommands
#[derive(Subcommand, Debug)]
pub enum SecretCommands {
    /// List user secrets (names only)
    List,
}

/// Cache management subcommands
#[derive(Subcommand, Debug)]
pub enum CacheCommands {
    /// Show cache statistics
    Status,
    /// Clear all cached data
    Clear,
    /// Print cache directory path
    Path,
}

/// Audit log subcommands
#[derive(Subcommand, Debug)]
pub enum AuditCommands {
    /// List audit log entries
    #[command(after_help = "\
Examples:
  hawkop audit list --type SCAN_STARTED,SCAN_COMPLETED --since 7d
  hawkop audit list --since 2024-12-01 --until 2024-12-31
  hawkop audit list --org-type EXTERNAL_ALERTS_SENT,ORGANIZATION_CREATED")]
    List {
        #[command(flatten)]
        filters: AuditFilterArgs,
    },
}

/// Environment management subcommands
#[derive(Subcommand, Debug)]
pub enum EnvCommands {
    /// List environments for an application
    #[command(
        visible_alias = "ls",
        after_help = "EXAMPLES:\n  \
            hawkop env list --app myapp              # List by app name\n  \
            hawkop env list --app <uuid>             # List by app ID"
    )]
    List {
        /// Application name or ID
        #[arg(long, short = 'a', required = true, add = app_name_candidates())]
        app: String,

        #[command(flatten)]
        pagination: PaginationArgs,
    },

    /// Get default YAML configuration for an environment
    #[command(after_help = "EXAMPLES:\n  \
            hawkop env config --app myapp production           # Display config\n  \
            hawkop env config --app myapp production -o out.yml # Save to file")]
    Config {
        /// Application name or ID
        #[arg(long, short = 'a', required = true, add = app_name_candidates())]
        app: String,

        /// Environment name or ID
        env: String,

        /// Output file path (optional, prints to stdout if not specified)
        #[arg(long, short = 'o')]
        output: Option<String>,
    },

    /// Create a new environment for an application
    #[command(after_help = "EXAMPLES:\n  \
            hawkop env create --app myapp staging    # Create 'staging' environment")]
    Create {
        /// Application name or ID
        #[arg(long, short = 'a', required = true, add = app_name_candidates())]
        app: String,

        /// Environment name
        name: String,
    },

    /// Delete an environment
    #[command(after_help = "EXAMPLES:\n  \
            hawkop env delete --app myapp staging       # Delete with confirmation\n  \
            hawkop env delete --app myapp staging --yes # Skip confirmation")]
    Delete {
        /// Application name or ID
        #[arg(long, short = 'a', required = true, add = app_name_candidates())]
        app: String,

        /// Environment name or ID
        env: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

/// Profile management subcommands for switching between orgs, users, or API keys
#[derive(Subcommand, Debug)]
pub enum ProfileCommands {
    /// List all configuration profiles
    #[command(visible_alias = "ls")]
    List,

    /// Switch to a different profile
    #[command(after_help = "EXAMPLES:\n  \
            hawkop profile use work      # Switch to 'work' profile\n  \
            hawkop profile use personal  # Switch to 'personal' profile")]
    Use {
        /// Name of the profile to activate
        name: String,
    },

    /// Create a new profile
    #[command(after_help = "EXAMPLES:\n  \
            hawkop profile create work             # Interactive creation\n  \
            hawkop profile create backup --from work # Copy from existing")]
    Create {
        /// Name for the new profile
        name: String,

        /// Copy settings from an existing profile
        #[arg(long)]
        from: Option<String>,
    },

    /// Delete a profile
    #[command(after_help = "EXAMPLES:\n  \
            hawkop profile delete old-test       # With confirmation\n  \
            hawkop profile delete staging --yes  # Skip confirmation")]
    Delete {
        /// Name of the profile to delete
        name: String,

        /// Skip confirmation prompt
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Show profile details
    #[command(after_help = "EXAMPLES:\n  \
            hawkop profile show        # Show active profile\n  \
            hawkop profile show prod   # Show specific profile")]
    Show {
        /// Profile name (defaults to active profile)
        name: Option<String>,
    },
}
