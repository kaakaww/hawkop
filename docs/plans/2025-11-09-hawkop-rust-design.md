# HawkOp Rust CLI - Design Document

**Date**: 2025-11-09
**Status**: Approved
**Author**: Design collaboration session

## Overview

HawkOp is a professional-grade CLI companion utility for the StackHawk DAST platform. This document describes the Rust implementation, replacing the original Go version to achieve better performance, smaller binaries, stronger type safety, and excellent cross-platform support.

The CLI follows GitHub's `gh` CLI design patterns and provides developers and security teams with streamlined terminal access to StackHawk's dynamic application security testing capabilities.

## Goals

### Primary Goals
- **Cross-platform first-class support**: macOS, Linux, Windows (PowerShell)
- **Performance**: Fast startup, efficient API usage, minimal resource consumption
- **User experience**: Intuitive commands, helpful error messages, beautiful output
- **Reliability**: Robust error handling, automatic auth refresh, graceful degradation
- **Maintainability**: Clean architecture, comprehensive tests, clear documentation

### Non-Goals (for MVP)
- Interactive TUI/dashboard interfaces
- Real-time scan monitoring
- Local vulnerability database
- Plugin/extension system

## Architecture

### Technology Stack

**Core Dependencies**:
- **CLI Framework**: `clap` v4 with derive API - type-safe argument parsing, automatic help generation
- **Async Runtime**: `tokio` v1 with full features - efficient async I/O for API calls
- **HTTP Client**: `reqwest` v0.12 with JSON - robust HTTP with connection pooling
- **Serialization**: `serde`, `serde_json`, `serde_yaml` - data (de)serialization
- **Error Handling**: `thiserror` + `anyhow` - ergonomic error types and context
- **Configuration**: `dirs` - cross-platform path resolution
- **Output**: `tabled` - professional table formatting
- **UX**: `dialoguer` (prompts), `indicatif` (progress), `colored` (colors)
- **Rate Limiting**: `governor` - token bucket rate limiter
- **Time**: `chrono` - date/time with serde support

**Development Dependencies**:
- `assert_cmd` - CLI integration testing
- `mockito` - HTTP mocking for tests
- `tempfile` - temporary directories for config tests

### Project Structure

```
hawkop/
├── src/
│   ├── main.rs              # Entry point, CLI initialization
│   ├── cli/
│   │   ├── mod.rs           # Clap command definitions
│   │   ├── init.rs          # init command handler
│   │   ├── status.rs        # status command handler
│   │   ├── version.rs       # version command handler
│   │   └── org.rs           # org subcommands (list/set/get)
│   ├── client/
│   │   ├── mod.rs           # API client trait definition
│   │   ├── stackhawk.rs     # StackHawk API implementation
│   │   ├── auth.rs          # Authentication logic
│   │   └── rate_limit.rs    # Rate limiting implementation
│   ├── config/
│   │   ├── mod.rs           # Configuration management
│   │   └── file.rs          # File I/O and permissions
│   ├── output/
│   │   ├── mod.rs           # Output formatter trait
│   │   ├── table.rs         # Table formatter implementation
│   │   └── json.rs          # JSON formatter implementation
│   └── error.rs             # Application error types
├── tests/
│   ├── integration/         # CLI integration tests
│   └── fixtures/            # Sample API responses
├── docs/
│   └── plans/               # Design documents
├── Cargo.toml
└── README.md
```

### Module Responsibilities

#### `main.rs`
- Initialize tokio runtime
- Parse CLI arguments with clap
- Route to appropriate command handler
- Handle top-level errors

#### `cli/` - Command Handlers
- Define command structure with clap derives
- Validate arguments and flags
- Load configuration
- Call API client methods
- Format and display output
- Handle command-specific errors

#### `client/` - API Client
- Trait-based design for testability
- HTTP request/response handling
- Authentication flow (API key → JWT)
- Automatic JWT refresh
- Rate limiting (360 req/min)
- Retry logic for transient failures
- Pagination handling

#### `config/` - Configuration Management
- Load/save YAML config file
- Merge CLI flags, env vars, and file config
- Cross-platform path resolution
- File permission enforcement (600 on Unix)
- Config validation

#### `output/` - Output Formatting
- Trait for multiple output formats
- Table rendering with width detection
- JSON serialization
- Color support with TTY detection
- Consistent styling across commands

#### `error.rs` - Error Types
- Domain-specific error enums
- User-friendly error messages
- Error context and debugging info
- Conversion from underlying errors

## Configuration Management

### Configuration Sources (Priority Order)

1. **Command-line flags** (highest priority)
   - Format: `--org-id ORG_123`, `--format json`
   - Overrides all other sources
   - Validated by clap

2. **Environment variables**
   - Format: `HAWKOP_ORG_ID`, `HAWKOP_API_KEY`, `HAWKOP_FORMAT`
   - Automatically bound via clap's `env` attribute
   - Useful for CI/CD pipelines

3. **Configuration file**
   - Location: `~/.hawkop/config.yaml`
   - Permissions: 600 (Unix) or user-only ACL (Windows)
   - Contains persistent settings and secrets

4. **Defaults** (lowest priority)
   - Sensible fallbacks defined in code
   - Example: default format is "table"

### Configuration File Structure

```yaml
api_key: hawk_abc123def456...
org_id: org_default123
jwt:
  token: eyJhbGci...
  expires_at: 2025-11-10T15:30:45Z
preferences:
  format: table
  page_size: 1000
```

### Global Flags (Available on All Commands)

- `--org <ORG_ID>` / `HAWKOP_ORG_ID` - Override default organization
- `--format <FORMAT>` / `HAWKOP_FORMAT` - Output format (table/json)
- `--config <PATH>` / `HAWKOP_CONFIG` - Override config file location
- `--debug` / `HAWKOP_DEBUG` - Enable debug logging

### Cross-Platform Considerations

- **Config path resolution**: Use `dirs::home_dir()` + `.hawkop/config.yaml`
  - Unix: `~/.hawkop/config.yaml`
  - Windows: `%USERPROFILE%\.hawkop\config.yaml`
- **File permissions**:
  - Unix: Set mode 0600 with `std::fs::set_permissions`
  - Windows: Best effort or skip (document limitation)
- **Path handling**: Always use `std::path::PathBuf`, never string concatenation

## API Client Design

### Authentication Flow

```
User runs `hawkop init`
  ↓
Prompt for API key (hidden input)
  ↓
POST /api/v1/login with API key
  ↓
Receive JWT token + expiry
  ↓
Save API key + JWT to config file
  ↓
Future commands use JWT for auth
  ↓
If JWT expired or 401: auto-refresh from API key
```

### Client Architecture

```rust
pub struct StackHawkClient {
    http: reqwest::Client,
    config: Arc<Config>,
    rate_limiter: RateLimiter,
    auth: RwLock<AuthState>,
}

struct AuthState {
    jwt: Option<String>,
    expires_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait StackHawkApi {
    async fn authenticate(&self, api_key: &str) -> Result<JwtToken>;
    async fn list_orgs(&self) -> Result<Vec<Organization>>;
    async fn get_org(&self, org_id: &str) -> Result<Organization>;
}
```

### Rate Limiting Strategy

**Target**: 360 requests per minute (per StackHawk API limits)

**Implementation**:
- Use `governor` crate with token bucket algorithm
- Allow initial burst for responsive CLI feel
- Throttle to ~167ms between requests after burst
- Automatic backoff on 429 (Too Many Requests) responses
- Rate limiter shared across all API calls in a command

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Authentication failed. Run `hawkop init` to set up your API key.")]
    Unauthorized,

    #[error("Access denied. You don't have permission to access this resource.")]
    Forbidden,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded. Retry after {0:?}")]
    RateLimit(Duration),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Network error")]
    Network(#[from] reqwest::Error),
}
```

**Error handling strategy**:
- Parse API error responses for user-friendly messages
- Automatic retry on 401 (auth refresh)
- Automatic retry on 429 with exponential backoff
- Surface helpful guidance in error messages
- Include debug info when `--debug` flag is set

### Pagination

**Default behavior**: Fetch all results automatically
- Use `pageSize=1000` (API maximum) to minimize requests
- Internally handle pagination transparently
- Return complete `Vec<T>` to caller

**Optional `--limit` flag**: User-controlled result limiting
- Useful for quick checks or testing
- Stops fetching after limit reached

## Output and User Experience

### Output Formats

#### Table (Default)
- Professional table rendering with `tabled`
- Auto-detect terminal width
- Intelligent column truncation
- Header row with column names
- Aligned columns for readability

Example:
```
ORG ID              NAME                  USERS  APPS
org_abc123          Acme Security Team    24     12
org_def456          Dev Sandbox           5      3
```

#### JSON
- Pretty-printed JSON with `serde_json`
- Includes metadata (timestamp, CLI version)
- Enables scripting and `jq` integration

Example:
```json
{
  "data": [
    {
      "id": "org_abc123",
      "name": "Acme Security Team",
      "users": 24,
      "apps": 12
    }
  ],
  "meta": {
    "timestamp": "2025-11-09T10:30:45Z",
    "version": "0.1.0"
  }
}
```

### User Experience Enhancements

#### Interactive Prompts (`dialoguer`)
- `hawkop init`: Prompt for API key with hidden input
- Confirmation prompts before destructive operations
- Selection prompts for choosing from lists

#### Progress Indicators (`indicatif`)
- Long-running operations show progress bars
- Example: "Fetching applications... 75% (45/60)"
- Spinner for operations without progress tracking

#### Color Output (`colored`)
- Conditional on TTY detection (no colors when piped)
- Error messages: Red
- Warning messages: Yellow
- Success messages: Green
- Info messages: Blue/Cyan
- Use `console` crate for cross-platform support

#### Help Text
- Clap auto-generates command help
- Include usage examples in command descriptions
- Cross-reference related commands
- Link to docs in error messages

Example error:
```
Error: Authentication failed

Your API key may be invalid or expired.

Run `hawkop init` to set up a new API key.
For help, visit: https://docs.stackhawk.com/
```

## MVP Scope

### Phase 1: MVP Commands

**Essential commands to prove the architecture**:

1. **`hawkop init`**
   - Interactive API key setup
   - Exchange API key for JWT
   - Create config file with proper permissions
   - Optionally set default organization

2. **`hawkop status`**
   - Display authentication status
   - Show current default organization
   - Show config file location
   - Verify API connectivity

3. **`hawkop version`**
   - Display CLI version
   - Show config file location
   - Display build information

4. **`hawkop org list`**
   - List all accessible organizations
   - Table or JSON output
   - Demonstrates full API integration

5. **`hawkop org set <ORG_ID>`**
   - Set default organization in config
   - Validate organization exists
   - Update config file

6. **`hawkop org get`**
   - Display current default organization details
   - Show organization metadata

### What MVP Validates

- ✅ Cross-platform configuration management
- ✅ Authentication flow (API key → JWT → auto-refresh)
- ✅ API client with rate limiting and error handling
- ✅ Table and JSON output formatting
- ✅ Command structure and flag handling
- ✅ Environment variable and config file precedence
- ✅ User-friendly error messages and help text

### Deliberately Excluded (Post-MVP)

- User management commands
- Team management commands
- Application commands
- Scan management and analysis
- Finding and alert details
- Advanced filtering options
- Shell completion generation
- Interactive TUI features

## Testing Strategy

### Unit Tests
- Test individual modules in isolation
- Mock external dependencies (HTTP, filesystem)
- Test configuration parsing and merging
- Test error handling and edge cases
- Test output formatters with sample data

### Integration Tests
- Full CLI command execution with `assert_cmd`
- Mock API responses with `mockito`
- Test configuration file creation and updates
- Test cross-platform path handling
- Verify output formats (table/JSON)

### Test Fixtures
- Store sample API responses in `tests/fixtures/`
- Include success cases and error responses
- Test edge cases (empty lists, large datasets)

### Manual Testing Checklist
- [ ] Config file created with correct permissions
- [ ] Cross-platform path resolution works
- [ ] Color output displays correctly in various terminals
- [ ] JSON output is valid and pretty-printed
- [ ] Error messages are helpful and actionable
- [ ] Auth flow works (API key → JWT → refresh)
- [ ] Rate limiting prevents API rejections

### CI/CD Testing
- GitHub Actions for multi-platform builds
- Test on Linux, macOS, Windows
- Run clippy for lints
- Run rustfmt for formatting
- Consider `cargo-tarpaulin` for coverage (Linux only)

## Implementation Order

### Day 1-2: Foundation
- [ ] Update Cargo.toml with dependencies
- [ ] Create module structure
- [ ] Define error types with `thiserror`
- [ ] Implement config module (YAML parsing, paths, permissions)
- [ ] Write unit tests for config

### Day 2-3: API Client
- [ ] Implement StackHawk API client trait
- [ ] HTTP client setup with `reqwest`
- [ ] Authentication flow (API key → JWT)
- [ ] Rate limiter with `governor`
- [ ] Error handling and retries
- [ ] Mock-based unit tests

### Day 3-4: CLI Framework
- [ ] Define command structure with `clap`
- [ ] Common flags and env var binding
- [ ] Output module (table and JSON formatters)
- [ ] Argument parsing tests

### Day 4-5: Init & Status Commands
- [ ] `hawkop init` with interactive prompts
- [ ] `hawkop status` to display config/auth state
- [ ] `hawkop version` to show version info
- [ ] Integration tests with `assert_cmd`

### Day 5-6: Org Commands
- [ ] `hawkop org list` with API integration
- [ ] `hawkop org set` to update config
- [ ] `hawkop org get` to display current org
- [ ] Full E2E tests with mocked API

### Day 6-7: Polish & Documentation
- [ ] Refine error messages
- [ ] Improve help text and examples
- [ ] Write README with installation and usage
- [ ] Set up CI/CD for multi-platform builds
- [ ] Cross-platform testing

## Future Enhancements (Post-MVP)

### Phase 2: Application & Scan Management
- `hawkop app list` - List applications with filtering
- `hawkop app get <APP_ID>` - Application details
- `hawkop scan list` - List scans with filtering
- `hawkop scan get <SCAN_ID>` - Scan details
- `hawkop scan alerts <SCAN_ID>` - Security alerts

### Phase 3: User & Team Management
- `hawkop user list` - List users with role filtering
- `hawkop team list` - List teams

### Phase 4: Advanced Features
- Scan finding details with request/response data
- Cross-application security dashboards
- Historical trending and metrics
- Policy management
- Export capabilities (CSV, PDF)
- Scan result comparison

### Phase 5: Distribution & Tooling
- Shell completion (bash, zsh, fish, PowerShell)
- Homebrew tap for macOS
- APT/YUM repositories for Linux
- Publish to crates.io
- Docker image

## Build Configuration

### Release Profile
```toml
[profile.release]
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization, slower build
strip = true         # Remove debug symbols
opt-level = 3        # Maximum optimization
```

### Cross-Compilation Targets
- Linux: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
- macOS: `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`

### Distribution
- GitHub Releases with automated builds
- Future: Homebrew, package repositories, Cargo

## References

### StackHawk Resources
- API OpenAPI Spec: https://download.stackhawk.com/openapi/stackhawk-openapi.json
- API Documentation: https://apidocs.stackhawk.com/
- Authentication: https://apidocs.stackhawk.com/reference/login
- Documentation: https://docs.stackhawk.com/

### Rust Ecosystem
- Clap: https://docs.rs/clap/
- Tokio: https://tokio.rs/
- Reqwest: https://docs.rs/reqwest/
- Serde: https://serde.rs/
- Tabled: https://docs.rs/tabled/

## Success Criteria

The MVP is considered successful when:

1. ✅ CLI builds and runs on Linux, macOS, and Windows
2. ✅ `hawkop init` successfully authenticates and saves config
3. ✅ `hawkop status` shows correct auth and config state
4. ✅ `hawkop org list` displays organizations in table and JSON formats
5. ✅ `hawkop org set` updates default organization
6. ✅ Auth automatically refreshes JWT when expired
7. ✅ Rate limiting prevents API rejections
8. ✅ Config file created with correct permissions (600 on Unix)
9. ✅ Error messages are clear and actionable
10. ✅ All tests pass on all platforms

## Conclusion

This design provides a solid foundation for building a professional, cross-platform CLI tool in Rust. The MVP scope is intentionally limited to validate the architecture while delivering immediate value. The modular design enables incremental feature development in future phases.

The Rust implementation offers significant advantages over the original Go version:
- Smaller binaries with no runtime dependency
- Stronger type safety and compile-time guarantees
- Excellent error handling with Result types
- Zero-cost abstractions for performance
- Outstanding cross-platform support

Next steps: Begin implementation following the phased approach outlined above.
