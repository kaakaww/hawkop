# HawkOp

**A CLI companion for the StackHawk DAST platform**

HawkOp brings StackHawk's application security intelligence to your terminal. Explore scan results and integrate with your workflows directly from the command line.

### Why HawkOp?

- **Explore interactively**: Quickly check scan status, browse findings, and drill into vulnerability details without context-switching to a browser
- **Automate with ease**: JSON output makes it simple to build scripts, dashboards, and CI/CD integrations—no custom API code required
- **Work faster**: Smart caching and parallel fetching keep large datasets snappy; shell completions speed up daily use

### Features

- **Full platform access** — Organizations, applications, scans, findings, users, teams, policies, and more
- **Powerful filtering** — Filter scans by status, environment, or application; audit logs by date, type, or user
- **Flexible output** — Human-readable tables for exploration, JSON for automation and scripting
- **Built for speed** — Local response caching, parallel pagination, automatic rate limit handling
- **Cross-platform** — Native binaries for Linux, macOS, and Windows (x86_64 and ARM64)

## Installation

### From GitHub Releases (Recommended)

Download the latest release for your platform from [GitHub Releases](https://github.com/kaakaww/hawkop/releases):

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | `hawkop-linux-x86_64.tar.gz` |
| Linux | ARM64 | `hawkop-linux-aarch64.tar.gz` |
| macOS | Intel | `hawkop-darwin-x86_64.tar.gz` |
| macOS | Apple Silicon | `hawkop-darwin-aarch64.tar.gz` |
| Windows | x86_64 | `hawkop-windows-x86_64.zip` |
| Windows | ARM64 | `hawkop-windows-aarch64.zip` |

Extract and move to your PATH:

```bash
# Linux/macOS example
tar -xzf hawkop-*.tar.gz
sudo mv hawkop /usr/local/bin/
```

### From Source

For the latest development version:

```bash
git clone https://github.com/kaakaww/hawkop.git
cd hawkop
cargo build --release
sudo cp target/release/hawkop /usr/local/bin/
```

## Quick Start

### 1. Initialize

Set up your API key and default organization:

```bash
hawkop init
```

This prompts for your [StackHawk API key](https://app.stackhawk.com/settings/apikeys), authenticates, and saves your configuration.

### 2. Verify Setup

```bash
hawkop status
```

### 3. Explore Your Data

```bash
hawkop org list          # List your organizations
hawkop app list          # List applications
hawkop scan list         # View recent scans
hawkop scan get <ID>     # Drill into scan details
```

## Commands

HawkOp provides comprehensive access to StackHawk data:

| Command | Description |
|---------|-------------|
| `hawkop init` | Interactive setup |
| `hawkop status` | Show configuration status |
| `hawkop org list\|set\|get` | Manage organizations |
| `hawkop app list` | List applications |
| `hawkop scan list` | List scans with filtering |
| `hawkop scan get <ID>` | Scan details and findings |
| `hawkop user list` | List organization members |
| `hawkop team list` | List teams |
| `hawkop policy list` | List scan policies |
| `hawkop repo list` | List attack surface repos |
| `hawkop audit list` | View audit log |
| `hawkop oas list` | List OpenAPI specs |
| `hawkop config list` | List scan configurations |
| `hawkop secret list` | List user secrets |
| `hawkop cache status\|clear\|path` | Manage local cache |
| `hawkop completion <shell>` | Shell completions |

**Use `--help` for detailed options:**

```bash
hawkop --help              # Overview
hawkop scan --help         # Scan command options
hawkop scan list --help    # Specific subcommand options
```

## Common Usage Examples

### Filtering Scans

```bash
hawkop scan list --status completed      # By status
hawkop scan list --env production        # By environment
hawkop scan list --app <APP_ID>          # By application
hawkop scan list --limit 50              # Limit results
```

### Audit Log Queries

```bash
hawkop audit list --since 7d             # Last 7 days
hawkop audit list --type SCAN_STARTED    # By activity type
hawkop audit list --user "Jane"          # By user
```

### JSON Output for Scripts

```bash
hawkop app list --format json | jq '.data[].name'
hawkop scan list --format json > scans.json
```

## Configuration

### Config File

HawkOp stores configuration at `~/.hawkop/config.yaml`:

```yaml
api_key: hawk_abc123...
org_id: org_abc123
jwt:
  token: eyJhbGci...
  expires_at: 2026-01-15T12:00:00Z
preferences:
  page_size: 1000
```

### Configuration Precedence

1. **Command-line flags** (highest priority)
2. **Environment variables**
3. **Config file**
4. **Built-in defaults**

### Environment Variables

| Variable | Description |
|----------|-------------|
| `HAWKOP_API_KEY` | API key (useful for CI/CD) |
| `HAWKOP_ORG_ID` | Default organization |
| `HAWKOP_FORMAT` | Output format (`table` or `json`) |
| `HAWKOP_CONFIG` | Config file path |
| `HAWKOP_DEBUG` | Enable debug logging |

### Global Flags

Available on all commands:

- `--format <table|json>` - Output format
- `--org <ORG_ID>` - Override organization
- `--config <PATH>` - Override config file
- `--no-cache` - Bypass local cache
- `--debug` - Enable debug output

## Caching

HawkOp caches API responses locally for faster repeat queries:

```bash
hawkop cache status    # View cache statistics
hawkop cache clear     # Clear all cached data
hawkop cache path      # Show cache location
```

Use `--no-cache` to bypass the cache and fetch fresh data.

## Shell Completions

Generate completions for your shell:

```bash
# Bash
hawkop completion bash > ~/.local/share/bash-completion/completions/hawkop

# Zsh
hawkop completion zsh > ~/.zfunc/_hawkop

# Fish
hawkop completion fish > ~/.config/fish/completions/hawkop.fish

# PowerShell
hawkop completion powershell >> $PROFILE
```

## Feedback & Issues

- **Report bugs**: [GitHub Issues](https://github.com/kaakaww/hawkop/issues)
- **Request features**: [GitHub Issues](https://github.com/kaakaww/hawkop/issues)
- **View roadmap**: [PLANNING.md](PLANNING.md)

## Contributing

Interested in contributing? See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, testing, and release processes.

## Resources

- [StackHawk Documentation](https://docs.stackhawk.com/)
- [StackHawk API Docs](https://apidocs.stackhawk.com/)
- [Technical Planning](PLANNING.md)

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**Made with care by the StackHawk team**
