# HawkOp Planning & Technical Reference

This document covers the technical architecture, development history, and future roadmap for HawkOp. For user documentation, see [README.md](README.md). For contribution guidelines, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Project Overview

HawkOp is a professional-grade Rust CLI companion for the StackHawk platform. It provides developers and security teams with terminal access to StackHawk's dynamic application security testing capabilities.

The CLI follows GitHub's `gh` CLI design patterns with a `hawkop <command> <subcommand>` structure.

## Architecture

### Project Structure

```
src/
├── main.rs              # CLI entrypoint, command routing
├── error.rs             # Error types using thiserror
├── cli/                 # Command definitions and handlers
│   ├── mod.rs           # Clap command enums
│   ├── context.rs       # CommandContext for shared state
│   ├── args/            # Shared argument types
│   │   ├── common.rs    # OutputFormat, SortDir
│   │   ├── filters.rs   # ScanFilterArgs, AuditFilterArgs
│   │   └── pagination.rs # PaginationArgs
│   ├── handlers/        # Generic command handlers
│   │   └── list.rs      # Generic list handler
│   └── [command].rs     # Individual command handlers
├── client/              # StackHawk API client
│   ├── mod.rs           # Re-exports
│   ├── api/             # API trait definitions
│   │   ├── auth.rs      # AuthApi trait
│   │   ├── listing.rs   # ListingApi trait
│   │   └── scan_detail.rs # ScanDetailApi trait
│   ├── models/          # API data models
│   │   ├── org.rs       # Organization
│   │   ├── app.rs       # Application
│   │   ├── scan.rs      # ScanResult, ScanFilterParams
│   │   └── ...          # Other model files
│   ├── stackhawk.rs     # HTTP client implementation
│   ├── pagination.rs    # PaginationParams, PagedResponse
│   ├── parallel.rs      # Parallel page fetching
│   ├── rate_limit.rs    # Per-endpoint rate limiting
│   └── mock.rs          # Mock client for testing
├── cache/               # Response caching
│   ├── client.rs        # CachedStackHawkClient wrapper
│   └── storage.rs       # SQLite + blob storage
├── config/              # YAML config management
├── models/              # Display models
│   └── display/         # Formatted output types
│       ├── org.rs       # OrgDisplay
│       ├── app.rs       # AppDisplay
│       ├── scan.rs      # ScanDisplay
│       └── ...
└── output/              # Output formatters
    ├── table.rs         # Table formatting
    └── json.rs          # JSON with metadata wrapper
```

### Key Technologies

| Technology | Purpose |
|------------|---------|
| [clap](https://docs.rs/clap/) | CLI framework with derive macros |
| [tokio](https://tokio.rs/) | Async runtime |
| [reqwest](https://docs.rs/reqwest/) | HTTP client |
| [serde](https://serde.rs/) | JSON/YAML serialization |
| [rusqlite](https://docs.rs/rusqlite/) | SQLite caching |
| [tabled](https://docs.rs/tabled/) | Table formatting |
| [dialoguer](https://docs.rs/dialoguer/) | Interactive prompts |
| [colored](https://docs.rs/colored/) | Terminal colors |

### API Client Design

The API client uses a trait-based architecture for testability:

```rust
// Split into focused sub-traits
pub trait AuthApi: Send + Sync {
    async fn authenticate(&self, api_key: &str) -> Result<JwtToken>;
}

pub trait ListingApi: Send + Sync {
    async fn list_orgs(&self) -> Result<Vec<Organization>>;
    async fn list_apps(&self, org_id: &str, ...) -> Result<Vec<Application>>;
    // ... other list methods
}

pub trait ScanDetailApi: Send + Sync {
    async fn get_scan(&self, org_id: &str, scan_id: &str) -> Result<ScanResult>;
    async fn list_scan_alerts(&self, scan_id: &str, ...) -> Result<Vec<Alert>>;
    // ... other detail methods
}

// Composed super-trait
pub trait StackHawkApi: AuthApi + ListingApi + ScanDetailApi {}
```

### Caching Architecture

HawkOp uses SQLite for metadata and blob storage for large responses:

```
~/.cache/hawkop/         # XDG cache directory
├── cache.db             # SQLite: keys, TTLs, metadata
└── blobs/               # Sharded blob storage
    ├── 00/              # First 2 hex chars of hash
    │   └── ab12...json  # Compressed response data
    └── ...
```

- **TTL-based invalidation**: 5-minute default, configurable per-resource
- **Bypass with `--no-cache`**: Always fetch fresh data
- **Manage with `hawkop cache`**: View stats, clear, find path

### Rate Limiting

Reactive per-endpoint rate limiting with exponential backoff:

| Category | Limit | Endpoints |
|----------|-------|-----------|
| Scan | 80/sec | `/scans/*` |
| User | 80/sec | `/user/*` |
| AppList | 80/sec | `/apps` |
| Default | 6/sec | Everything else |

Rate limits activate only after receiving a 429 response, then apply exponential backoff with jitter.

---

## Current State (v0.3.0)

### Implemented Features

**Core Commands:**
- `init` - Interactive setup with API key authentication
- `status` - Configuration and connection status
- `version` - Version information
- `org list|set|get` - Organization management

**Resource Listing:**
- `app list` - Applications with type filtering
- `scan list` - Scans with status/env/app filtering
- `scan get` - Scan details with drill-down to alerts
- `user list` - Organization members
- `team list` - Teams
- `policy list` - Scan policies
- `repo list` - Attack surface repositories
- `oas list` - Hosted OpenAPI specifications
- `config list` - Scan configurations
- `secret list` - User secrets
- `audit list` - Audit log with date/type/user filtering

**Infrastructure:**
- `cache status|clear|path` - Local cache management
- `completion <shell>` - Shell completions (bash/zsh/fish/powershell)

**Output & UX:**
- Table and JSON output formats
- Parallel pagination for large datasets
- Response caching with SQLite + blob storage
- JWT auto-refresh on expiry
- Cross-platform support (Linux, macOS, Windows)

### API Standards Compliance

- **Rate Limiting**: Reactive limiting with 429 detection and exponential backoff
- **Pagination**: Default pageSize=1000 to minimize API requests
- **Error Handling**: Comprehensive HTTP status handling (400, 401, 403, 404, 409, 422, 429)
- **Retry Logic**: Automatic JWT refresh, rate limit backoff

### Recent Improvements (Code Quality Plan)

A comprehensive code quality improvement plan was completed:

| Phase | Description | Status |
|-------|-------------|--------|
| 1.1 | CLI restructure - extracted `args/` and `handlers/` modules | ✅ |
| 1.2 | Model extraction - split API models into focused files | ✅ |
| 1.3 | Display split - organized display models by domain | ✅ |
| 1.4 | CLI tests - added unit tests for scan, app, audit handlers | ✅ |
| 2.1 | Generic list handler - reduced boilerplate in list commands | ✅ |
| 2.2 | Split API traits - AuthApi, ListingApi, ScanDetailApi | ✅ |
| 2.3 | Async cache - moved SQLite to spawn_blocking | ✅ |
| 2.4 | Exponential backoff - improved rate limit handling | ✅ |
| 3.x | Polish - documentation, formatting utilities | ✅ |
| 4.x | Testing infrastructure - fixtures, integration tests | ✅ |

---

## Future Roadmap

### Near-Term (Next Release)

1. **Scan Drill-Down Enhancements**
   - `scan alerts <SCAN_ID>` - List all alerts from a scan
   - `scan finding <SCAN_ID> <PLUGIN_ID>` - Individual finding details
   - Request/response data for security analysis

2. **Application Details**
   - `app get <APP_ID>` - Application metadata and configuration
   - Policy assignment visibility

### Medium-Term

3. **Enterprise Reporting**
   - `app summary` - Cross-application security posture dashboard
   - MTTR analysis and scan coverage metrics
   - Historical trending

4. **Export Capabilities**
   - CSV export for spreadsheet analysis
   - PDF reports for stakeholder communication

5. **Distribution**
   - Homebrew tap for macOS
   - APT/YUM repositories for Linux
   - Docker image
   - Publish to crates.io

### Long-Term Vision

6. **Repository Pattern**
   - Abstract cache vs API behind Repository trait
   - Full response payload caching with pagination metadata
   - Stale-while-revalidate for better UX

7. **Automation Helpers**
   - HawkScan installation automation
   - CRUD helpers for list endpoints requiring full updates

8. **Interactive Mode**
   - Guided workflows for common tasks
   - Scan result comparison and diff analysis

---

## Configuration Reference

### Config File Format

```yaml
# ~/.hawkop/config.yaml
api_key: hawk_xxx...
org_id: org_xxx
jwt:
  token: eyJhbGci...
  expires_at: 2026-01-15T12:00:00Z
preferences:
  page_size: 1000
```

File permissions: `600` (read/write for owner only)

### Environment Variables

| Variable | Description |
|----------|-------------|
| `HAWKOP_API_KEY` | API key |
| `HAWKOP_ORG_ID` | Default organization |
| `HAWKOP_FORMAT` | Output format |
| `HAWKOP_CONFIG` | Config file path |
| `HAWKOP_DEBUG` | Debug logging |
| `HAWKOP_API_BASE_URL` | V1 API base (testing) |
| `HAWKOP_API_BASE_URL_V2` | V2 API base (testing) |

---

## Reference Materials

### StackHawk Resources

- **OpenAPI Spec**: https://download.stackhawk.com/openapi/stackhawk-openapi.json
- **API Authentication**: https://apidocs.stackhawk.com/reference/login
- **API Documentation**: https://apidocs.stackhawk.com/
- **User Documentation**: https://docs.stackhawk.com/
- **HawkScan Config Schema**: https://download.stackhawk.com/hawk/jsonschema/hawkconfig.json

### CLI Design References

- [Command Line Interface Guidelines](https://clig.dev/)
- [12 Factor CLI Apps](https://medium.com/@jdxcode/12-factor-cli-apps-dd3c227a0e46)
- [GitHub CLI](https://cli.github.com/) - Design inspiration

### Internal Documentation

- `CLAUDE.md` - AI assistant guidelines for this project
- `docs/CLI_DESIGN_PRINCIPLES.md` - CLI UX standards
- `docs/plans/` - Historical planning documents
