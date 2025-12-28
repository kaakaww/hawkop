# HawkOp

**Professional CLI companion for the StackHawk DAST platform**

HawkOp is a cross-platform command-line tool that provides developers and security teams with streamlined access to StackHawk's application security intelligence platform directly from the terminal.

Built with Rust for performance, reliability, and excellent cross-platform support.

## Features

- 🔐 **Secure Authentication** - API key-based authentication with automatic JWT token management
- 🌐 **Cross-Platform** - First-class support for macOS, Linux, and Windows
- 📊 **Multiple Output Formats** - Beautiful table output for humans, JSON for automation
- ⚡ **Smart Rate Limiting** - Automatic API rate limiting (360 requests/minute)
- 🔄 **Auto Token Refresh** - JWT tokens automatically refresh when expired
- 🎨 **Rich CLI Experience** - Interactive prompts, progress indicators, and colored output
- 🛡️ **Secure Config Storage** - Config files with proper permissions (600 on Unix)
- 🔧 **Flexible Configuration** - CLI flags, environment variables, and config file support

## Installation

### From Source

```bash
git clone https://github.com/kaakaww/hawkop.git
cd hawkop
cargo build --release
sudo cp target/release/hawkop /usr/local/bin/
```

### Pre-built Binaries

Coming soon! We'll provide pre-built binaries for:
- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)
- Windows (x86_64)

## Quick Start

### 1. Initialize HawkOp

Run the interactive setup to configure your API key and default organization:

```bash
hawkop init
```

This will:
- Prompt for your StackHawk API key
- Authenticate and retrieve a JWT token
- List your organizations and let you choose a default
- Save your configuration to `~/.hawkop/config.yaml`

### 2. Check Configuration Status

```bash
hawkop status
```

### 3. List Organizations

```bash
# Table output (default)
hawkop org list

# JSON output
hawkop org list --format json
```

### 4. Set Default Organization

```bash
hawkop org set <ORG_ID>
```

### 5. Get Current Organization

```bash
hawkop org get
```

## Commands

### Global Flags

These flags are available on all commands:

- `--format <FORMAT>` - Output format: `table` (default) or `json`
- `--org <ORG_ID>` - Override default organization
- `--config <PATH>` - Override config file location
- `--debug` - Enable debug logging

All global flags can also be set via environment variables:
- `HAWKOP_FORMAT`
- `HAWKOP_ORG_ID`
- `HAWKOP_CONFIG`
- `HAWKOP_DEBUG`

### Output Formats

- `table` is human-readable and suited for terminals.
- `json` wraps results as `{"data": ..., "meta": {"timestamp": "...", "version": "..."}}` for automation and scripting.

### `hawkop init`

Initialize HawkOp configuration with interactive prompts.

```bash
hawkop init
```

### `hawkop status`

Show authentication and configuration status.

```bash
hawkop status
```

### `hawkop version`

Display version information.

```bash
hawkop version
```

### `hawkop org`

Manage organizations.

```bash
hawkop org list              # List all accessible organizations
hawkop org set <ORG_ID>      # Set default organization
hawkop org get               # Show current default organization
```

### `hawkop app`

List applications.

```bash
hawkop app list                    # List all applications
hawkop app list --type cloud       # Filter by type (cloud, standard)
hawkop app list --limit 50         # Limit results
```

### `hawkop scan`

List scans with filtering.

```bash
hawkop scan list                          # List recent scans
hawkop scan list --status completed       # Filter by status
hawkop scan list --env production         # Filter by environment
hawkop scan list --app <APP_ID>           # Filter by application
```

### `hawkop user`

List organization members.

```bash
hawkop user list
```

### `hawkop team`

List organization teams.

```bash
hawkop team list
```

### `hawkop policy`

List scan policies.

```bash
hawkop policy list
```

### `hawkop repo`

List repositories in attack surface.

```bash
hawkop repo list
```

### `hawkop audit`

View organization audit log.

```bash
hawkop audit list                              # Recent audit entries
hawkop audit list --type SCAN_STARTED          # Filter by activity type
hawkop audit list --since 7d                   # Last 7 days
hawkop audit list --user "John"                # Filter by user
```

### `hawkop oas`

List hosted OpenAPI specifications.

```bash
hawkop oas list
```

### `hawkop config`

List scan configurations.

```bash
hawkop config list
```

### `hawkop secret`

List user secret names.

```bash
hawkop secret list
```

### `hawkop completion`

Generate shell completions.

```bash
hawkop completion bash       # Bash completions
hawkop completion zsh        # Zsh completions
hawkop completion fish       # Fish completions
hawkop completion powershell # PowerShell completions
```

## Configuration

### Configuration File

HawkOp stores its configuration at `~/.hawkop/config.yaml`:

```yaml
api_key: hawk_abc123...
org_id: org_abc123
jwt:
  token: eyJhbGci...
  expires_at: 2026-01-15T12:00:00Z
preferences:
  page_size: 1000
```

**File Permissions**: On Unix systems, the config file is automatically created with `600` permissions (read/write for owner only) to protect your API key.

### Configuration Precedence

Configuration values are resolved in the following order (highest to lowest priority):

1. **Command-line flags**: `--org <ORG_ID>`
2. **Environment variables**: `HAWKOP_ORG_ID`
3. **Configuration file**: `~/.hawkop/config.yaml`
4. **Defaults**: Built-in default values

### Environment Variables

- `HAWKOP_API_KEY` - StackHawk API key (useful for CI/CD)
- `HAWKOP_ORG_ID` - Default organization ID
- `HAWKOP_FORMAT` - Default output format (`table` or `json`)
- `HAWKOP_CONFIG` - Override config file path
- `HAWKOP_DEBUG` - Enable debug logging
- `HAWKOP_API_BASE_URL`, `HAWKOP_API_BASE_URL_V2` - Override API base URLs (useful for testing/mocking)

## Architecture

HawkOp is built with a modular architecture:

- **CLI Layer** (`src/cli/`) - Command definitions and handlers using `clap`
- **API Client** (`src/client/`) - StackHawk API integration with automatic auth refresh and rate limiting
- **Configuration** (`src/config/`) - YAML-based config management with cross-platform support
- **Output** (`src/output/`) - Formatters for table and JSON output
- **Error Handling** (`src/error.rs`) - Comprehensive error types with user-friendly messages

### Key Technologies

- **[clap](https://docs.rs/clap/)** - CLI framework with derives
- **[tokio](https://tokio.rs/)** - Async runtime
- **[reqwest](https://docs.rs/reqwest/)** - HTTP client
- **[serde](https://serde.rs/)** - Serialization (JSON/YAML)
- **[governor](https://docs.rs/governor/)** - Rate limiting
- **[tabled](https://docs.rs/tabled/)** - Table formatting
- **[dialoguer](https://docs.rs/dialoguer/)** - Interactive prompts
- **[colored](https://docs.rs/colored/)** - Terminal colors

## Development

### Prerequisites

- Rust 1.70+ (2021 edition)
- Cargo

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

### Running from Source

```bash
cargo run -- --help
cargo run -- init
cargo run -- org list
```

### Code Quality

```bash
# Format code
cargo fmt

# Run lints
cargo clippy

# Run tests
cargo test
```

## Roadmap

### Implemented ✅

- ✅ Authentication and configuration management
- ✅ Organization management (`org list`, `org set`, `org get`)
- ✅ Application management (`app list` with pagination and type filtering)
- ✅ Scan management (`scan list` with filtering by status, env, app)
- ✅ User management (`user list`)
- ✅ Team management (`team list`)
- ✅ Policy management (`policy list`)
- ✅ Repository management (`repo list` - attack surface)
- ✅ Audit log (`audit list` with filtering)
- ✅ OpenAPI specs (`oas list`)
- ✅ Scan configs (`config list`)
- ✅ Secrets (`secret list`)
- ✅ Shell completions (`completion bash|zsh|fish|powershell`)
- ✅ Interactive init command
- ✅ Status and version commands
- ✅ Table and JSON output formats
- ✅ Cross-platform support
- ✅ Parallel pagination for large datasets
- ✅ Per-endpoint reactive rate limiting

### Planned

- [ ] `hawkop app get <APP_ID>` - Application details
- [ ] `hawkop scan get <SCAN_ID>` - Scan details
- [ ] `hawkop scan alerts <SCAN_ID>` - Security alerts
- [ ] Scan finding details with request/response data
- [ ] Cross-application security dashboards
- [ ] Historical trending and metrics
- [ ] Export capabilities (CSV, PDF)
- [ ] Homebrew tap for macOS
- [ ] APT/YUM repositories for Linux
- [ ] Publish to crates.io
- [ ] Docker image

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and lints (`cargo test && cargo clippy`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Resources

- [StackHawk Documentation](https://docs.stackhawk.com/)
- [StackHawk API Documentation](https://apidocs.stackhawk.com/)
- [Design Document](docs/plans/2025-11-09-hawkop-rust-design.md)

## Support

- 📖 [Documentation](https://docs.stackhawk.com/)
- 💬 [GitHub Issues](https://github.com/kaakaww/hawkop/issues)
- 🌐 [StackHawk Website](https://www.stackhawk.com/)

---

**Made with ❤️ by the StackHawk team**
