# Changelog

All notable changes to HawkOp will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Application CRUD** — Full lifecycle management for applications
  - `app create` — Create with `--name`, `--env`, `--type`, `--host`, `--team-id`
  - `app create --repo` / `--repo-id` — Create and link to a repository in one step
  - `app get` — Get application details by ID or `--name`
  - `app update` — Rename an application
  - `app delete` — Delete with confirmation (`--yes` to skip)
- **AI-optimized scan output** — `scan get --detail full` produces a single self-contained JSON document with all findings, evidence, HTTP messages, and remediation advice for agentic workflows
- **Repository linking** — Connect applications to repositories for API Discovery
  - `repo link` — Additive link with read-merge-write (preserves existing mappings)
  - `repo set-apps` — Full replacement of app mappings for a repository
- **Git-aware app creation** — Detects local git repository and suggests a targeted `repo link` command when creating apps without `--repo`
- **Enhanced init flow** — After setup, detects git repo, matches against StackHawk platform, and offers to create app + link for API Discovery onboarding
- **Hosted scan control** — Start, stop, and monitor hosted scans from the CLI
  - `run start --app <APP>` — Start a hosted scan with optional `--env`, `--config`, `--watch`
  - `run stop --app <APP>` — Stop a running scan (with `--yes` to skip confirmation)
  - `run status --app <APP>` — Check scan status with optional `--watch` auto-refresh
- **Environment management** — Create, list, and manage application environments
  - `env list --app <APP>`, `env create`, `env delete`, `env config`
- **Scan configuration CRUD** — Full lifecycle management for scan configs
  - `config get`, `config set`, `config delete`, `config rename`, `config validate`
- **OAS extended commands** — `oas get` and `oas mappings --app`
- **Dynamic shell completions** for app IDs, repo IDs, and repo names
- **OpenAPI spec refresh** — Updated `stackhawk-openapi.json` for 2026-03 API changes
- **Claude Code development hooks** — Automated workflow enforcement and quality reminders
- **API coverage roadmap** and **CLI command reference** documentation

### Changed

- `repo link` refactored to use shared `link_app_to_repo()` helper — eliminates duplicated read-merge-write logic across `repo link`, `app create --repo`, and `init` post-setup

### Fixed

- **Functional test safety** — `HAWKOP_PROFILE` is now required to run functional tests, preventing accidental runs against the default (production) profile
- **Org config corruption** — Fixed `test_org_set_and_get_roundtrip` broken restore that silently left config pointing to wrong org (JSON path didn't account for `{data}` wrapper)
- **CLI test isolation** — Integration tests now remove `HAWKOP_PROFILE` from environment, preventing failures when the user's shell has it set

## [0.5.1] - 2026-01-23

### Added

- **Duplicate app safety check** in `team create` - Warns if specified apps are already assigned to other teams, preventing the cascading error bug

### Changed

- API quirks documentation now focuses on user-facing symptoms and workarounds

## [0.5.0] - 2026-01-19

### Added

- **Full team management commands** - First CRUD implementation in HawkOp
  - `team get <TEAM>` - View team details with members and applications
  - `team create <NAME>` - Create a new team with optional `--users` and `--apps`
  - `team rename <TEAM> --name <NEW>` - Rename an existing team
  - `team delete <TEAM>` - Delete a team (with confirmation prompt)
- **Team member management** for SCIM-style automation
  - `team add-user <TEAM> <USER>...` - Add users to a team
  - `team remove-user <TEAM> <USER>...` - Remove users from a team
  - `team set-users <TEAM> <USER>...` - Replace all team members (IdP sync)
- **Team application assignment**
  - `team add-app <TEAM> <APP>...` - Assign applications to a team
  - `team remove-app <TEAM> <APP>...` - Unassign applications from a team
  - `team set-apps <TEAM> <APP>...` - Replace all app assignments
- **Flexible identifier resolution** - All team commands accept names OR UUIDs
  - Teams: by name or team UUID
  - Users: by email or user UUID
  - Apps: by name or application UUID
- **Dry-run mode** (`--dry-run`) on all mutating team commands
- **Stdin support** (`--stdin`) for bulk operations in scripts
- **Team list filters** - `--name`, `--member`, `--app` for filtering teams
- **Shell completions** for team names, user emails, and app names
- **Duplicate detection** - Prevents creating teams with duplicate names
- **API documentation** for StackHawk API quirks and edge cases (`api-quirks.md`)

### Changed

- **Cache safety** - Fresh reads before mutations, cache invalidation after changes
- **Navigation hints** - Standardized to use `→` arrow across all commands
- Team data cached with 1-minute TTL for responsive completions

### Fixed

- Include `organizationId` in team update requests (API requires all 5 fields despite OpenAPI spec)

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

[Unreleased]: https://github.com/kaakaww/hawkop/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.5.1
[0.5.0]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.5.0
[0.4.0]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.4.0
[0.3.0]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.3.0
[0.2.2]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.2.2
[0.2.1]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.2.1
[0.2.0]: https://github.com/kaakaww/hawkop/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/kaakaww/hawkop/releases/tag/v0.1.0
