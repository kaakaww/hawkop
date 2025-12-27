# Claude Code Guidelines for HawkOp

## Project Overview

HawkOp is a professional CLI companion for the StackHawk DAST platform, built in Rust. It provides developers and security teams with terminal access to StackHawk's application security intelligence.

## Project Structure

```
src/
├── main.rs              # CLI entrypoint, command routing
├── error.rs             # Error types using thiserror
├── cli/                 # Command definitions and handlers
│   ├── mod.rs           # Clap command enums (Commands, OrgCommands, etc.)
│   ├── context.rs       # CommandContext for shared state
│   ├── init.rs          # Interactive setup
│   ├── status.rs        # Config status display
│   ├── org.rs           # Organization commands
│   ├── app.rs           # Application commands
│   ├── scan.rs          # Scan commands
│   ├── user.rs          # User commands
│   └── team.rs          # Team commands
├── client/              # StackHawk API client
│   ├── mod.rs           # API trait + data models (Organization, Application, etc.)
│   ├── stackhawk.rs     # HTTP client implementation
│   ├── pagination.rs    # PaginationParams, PagedResponse, filters
│   ├── parallel.rs      # fetch_remaining_pages() for parallel API calls
│   ├── rate_limit.rs    # Per-endpoint reactive rate limiting
│   └── mock.rs          # Mock client for testing
├── config/              # YAML config management (~/.hawkop/config.yaml)
├── models/              # Display models for CLI output
│   ├── mod.rs
│   └── display.rs       # OrgDisplay, AppDisplay, ScanDisplay, etc.
└── output/              # Output formatters
    ├── mod.rs           # Formattable trait
    ├── table.rs         # tabled formatting
    └── json.rs          # JSON with metadata wrapper
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

## Current CLI Commands

```
hawkop init              # Interactive setup
hawkop status            # Show config status
hawkop version           # Version info
hawkop org list|set|get  # Organization management
hawkop app list          # List applications (supports pagination)
hawkop scan list         # List scans (supports pagination + filters)
hawkop user list         # List organization members
hawkop team list         # List organization teams
```

Global flags: `--format table|json`, `--org <ID>`, `--config <PATH>`, `--debug`

## Adding New Commands - Pattern to Follow

### 1. Add API Models (`src/client/mod.rs`)

```rust
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

### 3. Implement in Client (`src/client/stackhawk.rs`)

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

### 4. Add Display Model (`src/models/display.rs`)

```rust
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

## Coding Conventions

- Rust 2021 edition, idiomatic ownership, `?` for error propagation
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
