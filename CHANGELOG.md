# Changelog

All notable changes to HawkOp will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-01-12

### Added

- **Dynamic shell completions** for scan IDs, app names, plugin IDs, and URI IDs
  - Completions query StackHawk API in real-time when you press TAB
  - Rich metadata shown in completion hints (app, env, status, date)
  - Context-aware: plugin completions show severity, URI completions show method/path
  - Supports bash, zsh, and fish via `source <(COMPLETE=<shell> hawkop)`
- **SQLite caching** for API responses and completions
  - Configurable TTLs per endpoint type
  - Automatic cache invalidation and schema versioning
  - Blob storage for large responses (>10KB)
- **Exponential backoff** for rate limit retries with jitter
- **`hawkop scan get`** command for detailed scan exploration
  - Rich pretty format with color-coded severity
  - User and policy name lookup
  - `--plugin-id` and `--uri-id` filters for drill-down
- Test fixture builders for easy test data creation
- Enhanced `MockStackHawkClient` with configurable simulation capabilities
- Integration tests for CLI error scenarios

### Changed

- **Major codebase refactoring** for improved maintainability:
  - Split `client/mod.rs` into `client/api/` and `client/models/` sub-modules
  - Split `models/display.rs` (1900 lines) into domain-specific modules
  - Extracted CLI args into `cli/args/` module (common, pagination, filters)
  - Added generic list command handler to reduce duplication
- Moved SQLite cache I/O to blocking thread pool for better async performance
- Reduced default scan list limit from 25 to 10 for faster response
- Restructured documentation (README, CONTRIBUTING, PLANNING) for clarity

### Fixed

- Deduplicate tags in scan output
- Filter unexpanded environment variables from scan metadata

## [0.3.0] - 2025-12-28

### Added

- `hawkop scan view` command for drill-down exploration of scan results
  - `scan view <id>` - scan overview with findings summary
  - `scan view <id> alerts` - list all alerts (plugins)
  - `scan view <id> alert <plugin>` - alert detail with affected paths
  - `scan view <id> alert <plugin> uri <uri-id>` - URI detail with evidence
  - `scan view <id> alert <plugin> uri <uri-id> message` - HTTP request/response with curl command
- Scan context banner showing app/environment, host, HawkScan version, date, and duration
- Timezone abbreviation mapping for common zones (MST, EST, PST, etc.)

### Changed

- Table output now uses clean minimal style (vertical separators, header underline only)
- Navigation hints guide users through drill-down exploration

## [0.2.2] - 2025-12-27

### Added

- `hawkop audit list` command with filtering by activity type, user, email, and date ranges
- `hawkop oas list` command to list hosted OpenAPI specifications
- `hawkop config list` command to list scan configurations
- `hawkop secret list` command to list user secret names
- `hawkop completion` command for shell completions (bash, zsh, fish, powershell)
- `--type` filter for `hawkop app list` to filter by cloud/standard applications
- Comprehensive unit tests for error types, display models, and output formatters

### Changed

- Refactored date parsing to use let-chains for cleaner conditionals

## [0.1.0] - 2025-12-27

### Added

- Initial release
- `hawkop init` for interactive setup
- `hawkop status` to display configuration status
- `hawkop org list|set|get` for organization management
- `hawkop app list` with pagination and parallel fetching
- `hawkop scan list` with filtering by status, environment, and application
- `hawkop user list` to list organization members
- `hawkop team list` to list organization teams
- `hawkop policy list` to list scan policies
- `hawkop repo list` to list repositories in attack surface
- Table and JSON output formats (`--format table|json`)
- Global `--org` flag for organization override
- Global `--debug` flag for verbose logging
- Reactive per-endpoint rate limiting
- Parallel pagination for large datasets

[Unreleased]: https://github.com/kaakaww/hawkop/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.4.0
[0.3.0]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.3.0
[0.2.2]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.2.2
[0.2.1]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.2.1
[0.2.0]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kaakaww/hawkop/releases/tag/v0.1.0
