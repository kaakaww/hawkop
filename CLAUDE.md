# Claude Code Guidelines for HawkOp

## Project Overview

HawkOp is a professional CLI companion for the StackHawk platform, built in Rust. It provides developers and security teams with terminal access to StackHawk's application security intelligence.

## Tech Stack

- **Language**: Rust 2024 edition
- **CLI Framework**: clap v4 with derives
- **Async Runtime**: tokio
- **HTTP Client**: reqwest with rustls-tls
- **Serialization**: serde (JSON/YAML)
- **Error Handling**: anyhow + thiserror
- **Output**: tabled (tables), colored (terminal colors)
- **Interactive**: dialoguer, indicatif
- **Rate Limiting**: governor (reactive per-endpoint)
- **Caching**: rusqlite (SQLite), sha2 (cache key hashing)
- **Shell Completions**: clap_complete (static + dynamic)
- **Date/Time**: chrono

## Project Structure

```
src/
├── main.rs              # CLI entrypoint, command routing
├── error.rs             # Error types using thiserror
├── cache/               # SQLite-backed response caching
│   ├── mod.rs           # TTL configs, re-exports
│   ├── key.rs           # Cache key generation (SHA-256)
│   ├── storage.rs       # SQLite storage layer
│   └── client.rs        # CachedStackHawkClient wrapper
├── cli/                 # Command definitions and handlers
│   ├── mod.rs           # Clap command enums (Commands, OrgCommands, etc.)
│   ├── args/            # Shared CLI argument types
│   │   ├── mod.rs
│   │   ├── common.rs    # CommonArgs (org, format, debug)
│   │   ├── filters.rs   # ScanFilters, AuditFilters, etc.
│   │   └── pagination.rs # PaginationArgs
│   ├── handlers/        # Generic command handlers
│   │   ├── mod.rs
│   │   └── list.rs      # Generic list handler with pagination
│   ├── context.rs       # CommandContext for shared state
│   ├── completions.rs   # Dynamic shell completions (API-queried)
│   ├── cache.rs         # Cache management commands
│   ├── init.rs          # Interactive setup
│   ├── status.rs        # Config status display
│   ├── org.rs           # Organization commands
│   ├── app.rs           # Application commands
│   ├── scan.rs          # Scan commands (list + detail drill-down)
│   ├── user.rs          # User commands
│   ├── team.rs          # Team commands
│   ├── policy.rs        # Policy commands
│   ├── repo.rs          # Repository commands
│   ├── audit.rs         # Audit log commands
│   ├── oas.rs           # OpenAPI spec commands
│   ├── config.rs        # Scan config commands
│   └── secret.rs        # Secret commands
├── client/              # StackHawk API client
│   ├── mod.rs           # StackHawkApi trait definition
│   ├── stackhawk.rs     # HTTP client implementation
│   ├── api/             # API endpoint implementations
│   │   ├── mod.rs
│   │   ├── auth.rs      # Authentication/JWT handling
│   │   ├── listing.rs   # List endpoint implementations
│   │   └── scan_detail.rs # Scan detail with findings
│   ├── models/          # API data models
│   │   ├── mod.rs
│   │   ├── app.rs       # Application
│   │   ├── audit.rs     # AuditLogEntry
│   │   ├── auth.rs      # JwtPayload, TokenInfo
│   │   ├── config.rs    # ScanConfig
│   │   ├── finding.rs   # Finding, FindingDetail
│   │   ├── oas.rs       # OpenApiSpec
│   │   ├── org.rs       # Organization
│   │   ├── policy.rs    # ScanPolicy
│   │   ├── repo.rs      # Repository
│   │   ├── scan.rs      # Scan, ScanDetail
│   │   ├── secret.rs    # SecretInfo
│   │   └── user.rs      # User, Team
│   ├── pagination.rs    # PaginationParams, PagedResponse, filters
│   ├── parallel.rs      # fetch_remaining_pages() for parallel API calls
│   ├── rate_limit.rs    # Per-endpoint reactive rate limiting
│   ├── mock.rs          # Mock client for testing
│   └── fixtures.rs      # Test fixtures
├── config/              # Configuration management
│   └── mod.rs           # YAML config (~/.hawkop/config.yaml)
├── models/              # Display models for CLI output
│   ├── mod.rs
│   └── display/         # Individual display models
│       ├── mod.rs
│       ├── app.rs       # AppDisplay
│       ├── audit.rs     # AuditDisplay
│       ├── common.rs    # Shared display utilities
│       ├── config.rs    # ConfigDisplay
│       ├── finding.rs   # FindingDisplay
│       ├── oas.rs       # OasDisplay
│       ├── org.rs       # OrgDisplay
│       ├── policy.rs    # PolicyDisplay
│       ├── repo.rs      # RepoDisplay
│       ├── scan.rs      # ScanDisplay
│       ├── secret.rs    # SecretDisplay
│       └── user.rs      # UserDisplay, TeamDisplay
└── output/              # Output formatters
    ├── mod.rs           # Formattable trait
    ├── formatters.rs    # Format selection logic
    ├── table.rs         # tabled formatting
    └── json.rs          # JSON with metadata wrapper
scripts/
└── release.sh           # Interactive release wizard
```

## Build & Development Commands

```bash
cargo build                      # Debug build
cargo build --release            # Release build (target/release/hawkop)
cargo run -- <command>           # Run with args (e.g., cargo run -- org list)
cargo fmt                        # Format code (run before committing)
cargo clippy -- -D warnings      # Lint with warnings as errors
cargo test                       # Run all tests
```

## Current CLI Commands (v0.4.0)

```
hawkop init                      # Interactive setup
hawkop status                    # Show config status
hawkop version                   # Version info

hawkop org list|set|get          # Organization management
hawkop app list                  # List applications (pagination + type filter)
hawkop scan list                 # List scans (pagination + filters)
hawkop scan get [ID]             # Scan detail with drill-down (plugin/URI filtering)
hawkop user list                 # List organization members
hawkop team list                 # List organization teams
hawkop policy list               # List scan policies
hawkop repo list                 # List repositories in attack surface
hawkop audit list                # View audit log (filters + date ranges)
hawkop oas list                  # List hosted OpenAPI specifications
hawkop config list               # List scan configurations
hawkop secret list               # List user secret names

hawkop cache status              # Show cache statistics
hawkop cache clear               # Clear cached responses
hawkop cache path                # Show cache database path

hawkop completion <shell>        # Generate shell completions (bash/zsh/fish/powershell)
```

**Global flags:** `--format table|json`, `--org <ID>`, `--config <PATH>`, `--debug`, `--no-cache`

## Configuration

### Files & Directories
- **Config file**: `~/.hawkop/config.yaml`
- **Cache database**: `~/.hawkop/cache/hawkop_cache.db`

### Environment Variables
- `HAWKOP_API_KEY` - API key for authentication
- `HAWKOP_ORG_ID` - Default organization ID
- `HAWKOP_FORMAT` - Output format (table/json)
- `HAWKOP_DEBUG` - Enable debug logging
- `HAWKOP_NO_CACHE` - Disable response caching

### Precedence
CLI flags > environment variables > config file > defaults

## Key Features

- **JWT-based authentication** with automatic token refresh
- **SQLite-backed response caching** with configurable TTLs
- **Dynamic shell completions** with API-queried data (org IDs, app IDs, etc.)
- **Scan detail drill-down** with plugin/URI filtering
- **Multiple output formats** (table/JSON/pretty)
- **Reactive per-endpoint rate limiting** (only activates after 429)
- **Parallel pagination** for large datasets
- **Cross-platform config** (~/.hawkop/)

## Adding New Commands - Pattern to Follow

### 1. Add API Models (`src/client/models/`)

```rust
// src/client/models/newresource.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewResource {
    pub id: String,
    pub name: String,
    // Use #[serde(alias = "fieldName")] for API field name differences
}
```

### 2. Add Trait Method (`src/client/mod.rs`)

```rust
#[async_trait]
pub trait StackHawkApi: Send + Sync {
    // ...existing methods...
    async fn list_new_resource(&self, org_id: &str, pagination: Option<&PaginationParams>) -> Result<Vec<NewResource>>;
}
```

### 3. Implement in Client (`src/client/api/listing.rs` or new file)

```rust
async fn list_new_resource(&self, org_id: &str, pagination: Option<&PaginationParams>) -> Result<Vec<NewResource>> {
    #[derive(Deserialize)]
    struct Response { items: Vec<NewResource> }

    let path = format!("/endpoint/{}", org_id);
    let query_params = pagination.map(|p| p.to_query_params()).unwrap_or_default();

    let response: Response = self.request_with_query(
        reqwest::Method::GET,
        &self.base_url_v1,  // or base_url_v2
        &path,
        &query_params,
    ).await?;
    Ok(response.items)
}
```

### 4. Add Display Model (`src/models/display/`)

```rust
// src/models/display/newresource.rs
#[derive(Debug, Clone, Tabled, Serialize)]
pub struct NewResourceDisplay {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "NAME")]
    pub name: String,
}

impl From<NewResource> for NewResourceDisplay { ... }
```

### 5. Add CLI Command (`src/cli/mod.rs` + new handler file)

```rust
// In mod.rs - add to Commands enum and create XxxCommands enum
// Create src/cli/xxx.rs with list() function following existing patterns
```

### 6. Wire Up in `src/main.rs`

Add the match arm for the new command.

## API Integration Notes

### Rate Limiting

- Reactive per-endpoint rate limiting (only activates after 429)
- Categories: Scan (80/sec), User (80/sec), AppList (80/sec), Default (6/sec)
- See `src/client/rate_limit.rs` for EndpointCategory

### Response Caching

- SQLite-backed caching in `~/.hawkop/cache/hawkop_cache.db`
- TTLs configured per-endpoint type (see `src/cache/mod.rs`)
- Cache key generation uses SHA-256 hash of request parameters
- Bypass with `--no-cache` flag or `HAWKOP_NO_CACHE=1`

### Parallel Pagination

For large datasets, use `totalCount`-based parallel fetching:
1. First request gets `totalCount`
2. Calculate remaining pages
3. Fetch in parallel with `fetch_remaining_pages()`

See `src/cli/scan.rs` and `src/cli/app.rs` for examples.

### API Quirks

- Some endpoints return numbers as strings (e.g., `totalCount: "2666"`)
- Use `deserialize_string_to_usize` helper in stackhawk.rs
- API base URLs: v1 = `https://api.stackhawk.com/api/v1`, v2 = `https://api.stackhawk.com/api/v2`

## CLI Design Philosophy

HawkOp follows principles from [clig.dev](https://clig.dev) and [12-Factor CLI Apps](https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46). See `docs/CLI_DESIGN_PRINCIPLES.md` for the complete guide.

**Key rules:**
- stdout for data, stderr for messages/progress
- Prefer flags over multiple positional args
- Always support `--format json` for scripting
- Errors must be actionable (what went wrong + how to fix)
- Tables: no borders, one row per entry
- Respect `NO_COLOR`, `TERM=dumb`, `--no-color`
- Never require prompts (always allow flag override)
- Suggest next commands with `→ hawkop <next-command>`

**Before adding a new command, verify against the checklist in `docs/CLI_DESIGN_PRINCIPLES.md`.**

## Coding Conventions

- Rust 2024 edition, idiomatic ownership, `?` for error propagation
- `snake_case` for modules/variables, `PascalCase` for types
- CLI flags mirror existing patterns (`--org`, `--limit`, `--format`)
- Keep handlers small; business logic in `client` or `config` modules
- Use `log::debug!()` for diagnostics (enabled with `--debug`)

## Testing

- Unit tests alongside modules, integration tests in `tests/`
- Use `MockStackHawkClient` for API testing
- Use `tempfile` for config tests
- Name tests after behavior: `init_sets_default_org`, `org_list_prints_table`

## Security

- Never log API keys or JWTs
- Config file permissions: 600 on Unix
- Sensitive fields use `#[serde(skip_serializing)]` where appropriate

## Resources

- **OpenAPI Spec**: `stackhawk-openapi.json` (root of repo)
  - Source: https://download.stackhawk.com/openapi/stackhawk-openapi.json
  - Check periodically for updates
- **API Docs**: https://apidocs.stackhawk.com/docs
- **Design Docs**: `docs/plans/*.md`

## Commit Guidelines

- Concise imperative subjects: `add user list command`, `fix pagination for scans`
- Run `cargo fmt && cargo clippy -- -D warnings && cargo test` before committing
- Include sample CLI output in PRs when UI changes
