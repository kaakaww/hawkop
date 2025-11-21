# HawkOp Project Context

## Project Overview
HawkOp is a professional command-line interface (CLI) tool for the StackHawk DAST (Dynamic Application Security Testing) platform. Built with Rust for performance, reliability, and cross-platform support.

## Tech Stack
- **Language**: Rust (2021 edition, 1.70+)
- **CLI Framework**: clap v4 with derives
- **Async Runtime**: tokio
- **HTTP Client**: reqwest
- **Serialization**: serde (JSON/YAML)
- **Error Handling**: anyhow + thiserror
- **Output**: tabled (tables), colored (terminal colors)
- **Interactive**: dialoguer, indicatif
- **Rate Limiting**: governor (360 req/min)

## Project Structure
```
src/
├── cli/           # Command definitions and handlers
│   ├── app.rs     # Main CLI app structure
│   ├── init.rs    # Interactive initialization
│   ├── org.rs     # Organization commands
│   └── status.rs  # Status command
├── client/        # StackHawk API client
│   └── stackhawk.rs  # API integration with JWT auth
├── config/        # Configuration management
│   └── mod.rs     # YAML config (~/.hawkop/config.yaml)
├── output/        # Output formatters
│   ├── table.rs   # Table formatting
│   └── json.rs    # JSON formatting
├── error.rs       # Error types and handling
└── main.rs        # Entry point
```

## Key Features
- JWT-based authentication with automatic token refresh
- Cross-platform config storage (~/.hawkop/config.yaml with 600 permissions)
- Multiple output formats (table/JSON)
- Rate limiting (360 requests/minute)
- Interactive setup with `hawkop init`

## Configuration
- **File**: `~/.hawkop/config.yaml`
- **Env Vars**: HAWKOP_API_KEY, HAWKOP_ORG_ID, HAWKOP_FORMAT, HAWKOP_DEBUG
- **Precedence**: CLI flags > env vars > config file > defaults

## Development Commands
```bash
# Build & run
cargo build              # Debug build
cargo build --release    # Release build
cargo run -- [ARGS]      # Run with arguments

# Quality
cargo fmt                # Format code
cargo clippy             # Lints
cargo test               # Run tests

# Example commands
cargo run -- init
cargo run -- org list
cargo run -- status
```

## Current State
**MVP Complete**:
- Authentication and configuration management
- Organization management (list, get, set)
- Interactive init command
- Status and version commands
- Table and JSON output

**Upcoming**: Application management, scan management, user/team features

## Important Notes
- Config files must have 600 permissions on Unix for security
- All API operations require valid JWT tokens
- Rate limiting is enforced at 360 requests/minute
- Error messages should be user-friendly (not debug dumps)
- Support for --format json flag on all data commands
