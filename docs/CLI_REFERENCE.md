# HawkOp CLI Reference

**Last updated**: 2026-03-14
**Version**: v0.4.0
**Source of truth**: `src/cli/mod.rs` (clap derives)

Complete taxonomy of every command, subcommand, argument, option, and alias in the HawkOp CLI. Planned (not yet implemented) commands are marked with `[planned]`.

---

## Table of Contents

- [Global Options](#global-options)
- [Shared Argument Groups](#shared-argument-groups)
- [Command Tree](#command-tree)
  - [init](#hawkop-init)
  - [status](#hawkop-status)
  - [version](#hawkop-version)
  - [org](#hawkop-org)
  - [app](#hawkop-app)
  - [scan](#hawkop-scan)
  - [run](#hawkop-run)
  - [user](#hawkop-user)
  - [team](#hawkop-team)
  - [policy](#hawkop-policy)
  - [repo](#hawkop-repo)
  - [oas](#hawkop-oas)
  - [config](#hawkop-config)
  - [secret](#hawkop-secret)
  - [audit](#hawkop-audit)
  - [env](#hawkop-env)
  - [cache](#hawkop-cache)
  - [profile](#hawkop-profile)
  - [completion](#hawkop-completion)
  - [findings (planned)](#hawkop-findings-planned)
- [Test Coverage Map](#test-coverage-map)
- [Planned Commands Summary](#planned-commands-summary)

---

## Global Options

These flags apply to **all** commands (defined in `Cli` struct, `src/cli/mod.rs:58-107`):

| Flag | Short | Type | Default | Env Var | Description |
|------|-------|------|---------|---------|-------------|
| `--format` | | `pretty\|table\|json` | `pretty` | `HAWKOP_FORMAT` | Output format |
| `--org` | | `String` | from config | `HAWKOP_ORG_ID` | Override default organization |
| `--config` | | `String` | `~/.hawkop/config.yaml` | `HAWKOP_CONFIG` | Override config file location |
| `--profile` | `-P` | `String` | active profile | `HAWKOP_PROFILE` | Configuration profile to use |
| `--debug` | | `bool` | `false` | `HAWKOP_DEBUG` | Enable debug logging |
| `--no-cache` | | `bool` | `false` | `HAWKOP_NO_CACHE` | Bypass response cache |
| `--api-host` | | `String` | `https://api.stackhawk.com` | `HAWKOP_API_HOST` | Custom API host (hidden) |

**Precedence**: CLI flags > environment variables > config file > defaults

---

## Shared Argument Groups

These are flattened into commands via `#[command(flatten)]`.

### PaginationArgs

Source: `src/cli/args/pagination.rs:19-36`
Used by: `app list`, `scan list`, `user list`, `team list`, `policy list`, `repo list`, `oas list`, `config list`, `env list`

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--limit` | `-n` | `usize` | MAX_PAGE_SIZE | Maximum results to return |
| `--page` | `-p` | `usize` | `0` | Page number (0-indexed) |
| `--sort-by` | | `String` | (none) | Field to sort by |
| `--sort-dir` | | `asc\|desc` | (none) | Sort direction |

### ScanFilterArgs

Source: `src/cli/args/filters.rs:55-68`
Used by: `scan list`

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--app` | `-a` | `String[]` | (none) | Filter by app ID (comma-sep or repeated) |
| `--env` | `-e` | `String[]` | (none) | Filter by environment (comma-sep or repeated) |
| `--status` | `-s` | `String` | (none) | Filter by status (running, complete, failed) |

### AuditFilterArgs

Source: `src/cli/args/filters.rs:8-47`
Used by: `audit list`

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--type` | `-t` | `String[]` | (none) | Filter by activity type (comma-sep) |
| `--org-type` | | `String[]` | (none) | Filter by org activity type (comma-sep) |
| `--user` | `-u` | `String` | (none) | Filter by user name |
| `--email` | | `String` | (none) | Filter by user email |
| `--since` | | `String` | (none) | Start date (ISO or relative: 7d, 30d) |
| `--until` | | `String` | (none) | End date (ISO or relative: 7d, 30d) |
| `--sort-dir` | | `asc\|desc` | `desc` | Sort direction |
| `--limit` | `-n` | `usize` | (none) | Maximum results to return |

### TeamFilterArgs

Source: `src/cli/mod.rs:37-50`
Used by: `team list`

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--name` | | `String` | (none) | Filter by team name (substring, case-insensitive) |
| `--member` | | `String` | (none) | Filter by member email |
| `--app` | | `String` | (none) | Filter by app name |

---

## Command Tree

### `hawkop init`

Initialize HawkOp configuration (interactive setup).

| Component | Value |
|-----------|-------|
| Arguments | (none) |
| Options | (global only) |
| API calls | `GET /api/v1/auth/login`, `GET /api/v1/user` |
| Handler | `src/cli/init.rs` |

---

### `hawkop status`

Show authentication and configuration status.

| Component | Value |
|-----------|-------|
| Arguments | (none) |
| Options | (global only) |
| Notes | Does not support `--format json` — always human-readable |
| Handler | `src/cli/status.rs` |

---

### `hawkop version`

Display version information.

| Component | Value |
|-----------|-------|
| Arguments | (none) |
| Options | (global only) |
| Handler | `src/main.rs` (inline) |

---

### `hawkop org`

Manage organizations.

#### `org list`

List all accessible organizations.

| Component | Value |
|-----------|-------|
| Arguments | (none) |
| Options | (global only) |
| API call | `GET /api/v1/user` (orgs extracted from user profile) |
| Handler | `src/cli/org.rs` |

#### `org set`

Set default organization.

| Component | Value |
|-----------|-------|
| Arguments | `<ORG_ID>` — Organization ID (positional, required) |
| Options | (global only) |
| Handler | `src/cli/org.rs` |

#### `org get`

Show current default organization.

| Component | Value |
|-----------|-------|
| Arguments | (none) |
| Options | (global only) |
| Handler | `src/cli/org.rs` |

---

### `hawkop app`

Manage applications.

#### `app list`

List all applications in the current organization.

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `--type` | `-t` | `String` | (none) | Filter by app type (cloud, standard) |
| + PaginationArgs | | | | See [PaginationArgs](#paginationargs) |

| Component | Value |
|-----------|-------|
| API call | `GET /api/v2/org/{orgId}/apps` |
| Handler | `src/cli/app.rs` |

#### `app get` [planned]

Get a single application by ID.

| Flag | Type | Description |
|------|------|-------------|
| `<APP>` | String (positional) | Application name or ID |

| Component | Value |
|-----------|-------|
| API call | `GET /api/v1/app/{appId}` |
| Roadmap | Phase 1 |

#### `app create` [planned]

Create a new application.

| Component | Value |
|-----------|-------|
| API call | `POST /api/v1/org/{orgId}/app` |
| Roadmap | Phase 1 |

#### `app update` [planned]

Update an existing application.

| Component | Value |
|-----------|-------|
| API call | `POST /api/v1/app/{appId}` |
| Roadmap | Phase 1 |

#### `app delete` [planned]

Delete an application.

| Component | Value |
|-----------|-------|
| API call | `DELETE /api/v1/app/{appId}` |
| Roadmap | Phase 1 |

#### `app policy get` [planned]

Get the scan policy for an application.

| Component | Value |
|-----------|-------|
| API call | `GET /api/v1/app/{appId}/policy` |
| Roadmap | Phase 2 |

#### `app policy assign` [planned]

Assign scan policy plugins to an application.

| Component | Value |
|-----------|-------|
| API call | `PUT /api/v1/app/{appId}/policy/assign` |
| Roadmap | Phase 2 |

#### `app policy flags` [planned]

Get or update application tech flags.

| Component | Value |
|-----------|-------|
| API calls | `GET /api/v1/app/{appId}/policy/flags`, `PUT /api/v1/app/{appId}/policy/flags` |
| Roadmap | Phase 2 |

#### `app policy toggle` [planned]

Toggle a scan policy plugin on/off.

| Component | Value |
|-----------|-------|
| API call | `GET /api/v1/app/{appId}/policy/plugins/{pluginId}/{toggle}` |
| Roadmap | Phase 2 |
| Notes | Unusual: uses GET with path parameter for toggle action |

---

### `hawkop scan`

View and manage scans.

#### `scan list`

List recent scans across all applications.

| Component | Value |
|-----------|-------|
| Flattened | ScanFilterArgs + PaginationArgs |
| API call | `GET /api/v1/scan/{orgId}` |
| Handler | `src/cli/scan.rs` |

See [ScanFilterArgs](#scanfilterargs) and [PaginationArgs](#paginationargs).

#### `scan get`

Get scan details with optional drill-down.

| Aliases | `scan g` |
|---------|----------|

| Flag | Short | Type | Default | Description |
|------|-------|------|---------|-------------|
| `[SCAN_ID]` | | `String` (positional) | `latest` | Scan ID (UUID) or "latest" |
| `--app` | `-a` | `String` | (none) | Filter by app name (only with "latest") |
| `--app-id` | | `String` | (none) | Filter by app ID (only with "latest") |
| `--env` | `-e` | `String` | (none) | Filter by environment (only with "latest") |
| `--plugin-id` | `-p` | `String` | (none) | Show detail for specific plugin/vuln type |
| `--uri-id` | `-u` | `String` | (none) | Show detail for specific URI/finding |
| `--message` | `-m` | `bool` | `false` | Include HTTP message (requires `--uri-id`) |
| `--format` | `-o` | `pretty\|table\|json` | `pretty` | Output format (overrides global) |

| Component | Value |
|-----------|-------|
| Conflicts | `--app` conflicts with `--app-id` |
| Requires | `--message` requires `--uri-id` |
| Dynamic completions | scan_id, app_name, plugin_id, uri_id |
| API calls | `GET /api/v1/scan/{scanId}`, `GET /api/v1/scan/{scanId}/alerts`, `GET /api/v1/scan/{scanId}/alert/{pluginId}`, `GET /api/v1/scan/{scanId}/uri/{alertUriId}/messages/{messageId}` |
| Handler | `src/cli/scan.rs` |

#### `scan delete` [planned]

Delete a scan by ID.

| Component | Value |
|-----------|-------|
| API call | `DELETE /api/v1/scan/{scanId}` |
| Roadmap | Phase 4 |

---

### `hawkop run`

Run hosted scans (start, stop, status).

#### `run start`

Start a hosted scan for an application.

| Flag | Short | Type | Required | Default | Description |
|------|-------|------|----------|---------|-------------|
| `--app` | `-a` | `String` | Yes | | Application name or ID |
| `--env` | `-e` | `String` | No | (none) | Environment to scan |
| `--config` | `-c` | `String` | No | (none) | Scan configuration name |
| `--watch` | `-w` | `bool` | No | `false` | Watch scan progress after starting |

| Component | Value |
|-----------|-------|
| Dynamic completions | app_name |
| API call | `POST /api/v1/app/{appId}/perch/start` |
| Handler | `src/cli/run.rs` |

#### `run stop`

Stop a running hosted scan.

| Flag | Short | Type | Required | Default | Description |
|------|-------|------|----------|---------|-------------|
| `--app` | `-a` | `String` | Yes | | Application name or ID |
| `--yes` | `-y` | `bool` | No | `false` | Skip confirmation prompt |

| Component | Value |
|-----------|-------|
| Dynamic completions | app_name |
| API call | `POST /api/v1/app/{appId}/perch/stop` |
| Handler | `src/cli/run.rs` |

#### `run status`

Get the status of a hosted scan.

| Flag | Short | Type | Required | Default | Description |
|------|-------|------|----------|---------|-------------|
| `--app` | `-a` | `String` | Yes | | Application name or ID |
| `--watch` | `-w` | `bool` | No | `false` | Watch status with auto-refresh |
| `--interval` | `-i` | `u64` | No | `5` | Refresh interval in seconds |

| Component | Value |
|-----------|-------|
| Dynamic completions | app_name |
| API call | `GET /api/v1/app/{appId}/perch/status` |
| Handler | `src/cli/run.rs` |

---

### `hawkop user`

List organization users/members.

#### `user list`

List organization members.

| Component | Value |
|-----------|-------|
| Flattened | PaginationArgs |
| API call | `GET /api/v1/org/{orgId}/members` |
| Handler | `src/cli/user.rs` |

---

### `hawkop team`

Manage organization teams.

#### `team list`

List organization teams.

| Aliases | `team ls` |
|---------|-----------|

| Component | Value |
|-----------|-------|
| Flattened | PaginationArgs + TeamFilterArgs |
| API call | `GET /api/v1/org/{orgId}/teams` |
| Handler | `src/cli/team.rs` |

#### `team get`

Get team details with members and applications.

| Aliases | `team g` |
|---------|----------|

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `<TEAM>` | `String` (positional) | Yes | Team ID or name |

| Component | Value |
|-----------|-------|
| Dynamic completions | team_name |
| API call | `GET /api/v1/org/{orgId}/team/{teamId}` |
| Handler | `src/cli/team.rs` |

#### `team create`

Create a new team.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<NAME>` | | `String` (positional) | Yes | Team name |
| `--users` | `-u` | `String[]` | No | Initial members (email/ID, comma-sep or repeated) |
| `--apps` | `-a` | `String[]` | No | Initial apps (name/ID, comma-sep or repeated) |
| `--dry-run` | `-n` | `bool` | No | Preview without creating |
| `--force` | `-f` | `bool` | No | Allow duplicate app assignments |

| Component | Value |
|-----------|-------|
| Dynamic completions | user_email, app_name |
| API call | `POST /api/v1/org/{orgId}/team` |
| Handler | `src/cli/team.rs` |

#### `team delete`

Delete a team.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<TEAM>` | | `String` (positional) | Yes | Team ID or name |
| `--yes` | `-y` | `bool` | No | Skip confirmation prompt |
| `--dry-run` | `-n` | `bool` | No | Preview without deleting |

| Component | Value |
|-----------|-------|
| Dynamic completions | team_name |
| API call | `DELETE /api/v1/org/{orgId}/team/{teamId}` |
| Handler | `src/cli/team.rs` |

#### `team rename`

Rename a team.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<CURRENT>` | | `String` (positional) | Yes | Current team ID or name |
| `<NEW_NAME>` | | `String` (positional) | Yes | New team name |
| `--dry-run` | `-n` | `bool` | No | Preview without renaming |

| Component | Value |
|-----------|-------|
| Dynamic completions | team_name (current) |
| API call | `PUT /api/v1/org/{orgId}/team/{teamId}` |
| Handler | `src/cli/team.rs` |

#### `team add-user`

Add users to a team.

| Aliases | `team add-member` |
|---------|-------------------|

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<TEAM>` | | `String` (positional) | Yes | Team ID or name |
| `<USERS>` | | `String[]` (positional) | Unless `--stdin` | Users (email/ID, comma-sep) |
| `--stdin` | | `bool` | No | Read users from stdin (one per line) |
| `--dry-run` | `-n` | `bool` | No | Preview without adding |

| Component | Value |
|-----------|-------|
| Dynamic completions | team_name, user_email |
| API call | `PUT /api/v1/org/{orgId}/team/{teamId}` |
| Handler | `src/cli/team.rs` |

#### `team remove-user`

Remove users from a team.

| Aliases | `team remove-member` |
|---------|----------------------|

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<TEAM>` | | `String` (positional) | Yes | Team ID or name |
| `<USERS>` | | `String[]` (positional) | Unless `--stdin` | Users (email/ID, comma-sep) |
| `--stdin` | | `bool` | No | Read users from stdin (one per line) |
| `--dry-run` | `-n` | `bool` | No | Preview without removing |

| Component | Value |
|-----------|-------|
| Dynamic completions | team_name, user_email |
| API call | `PUT /api/v1/org/{orgId}/team/{teamId}` |
| Handler | `src/cli/team.rs` |

#### `team set-users`

Replace all team members (for SCIM sync).

| Aliases | `team sync-users` |
|---------|-------------------|

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<TEAM>` | | `String` (positional) | Yes | Team ID or name |
| `<USERS>` | | `String[]` (positional) | Unless `--stdin` | Complete user list (comma-sep) |
| `--stdin` | | `bool` | No | Read users from stdin (one per line) |
| `--dry-run` | `-n` | `bool` | No | Preview changes without applying |
| `--yes` | `-y` | `bool` | No | Skip confirmation prompt |

| Component | Value |
|-----------|-------|
| Dynamic completions | team_name, user_email |
| API call | `PUT /api/v1/org/{orgId}/team/{teamId}` |
| Handler | `src/cli/team.rs` |

#### `team add-app`

Assign applications to a team.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<TEAM>` | | `String` (positional) | Yes | Team ID or name |
| `<APPS>` | | `String[]` (positional) | Unless `--stdin` | Apps (name/ID, comma-sep) |
| `--stdin` | | `bool` | No | Read apps from stdin (one per line) |
| `--dry-run` | `-n` | `bool` | No | Preview without assigning |
| `--force` | `-f` | `bool` | No | Allow duplicate app assignments |

| Component | Value |
|-----------|-------|
| Dynamic completions | team_name, app_name |
| API call | `PUT /api/v1/org/{orgId}/team/{teamId}/application` |
| Safety | Apps can only belong to one team at a time |
| Handler | `src/cli/team.rs` |

#### `team remove-app`

Remove applications from a team.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<TEAM>` | | `String` (positional) | Yes | Team ID or name |
| `<APPS>` | | `String[]` (positional) | Unless `--stdin` | Apps (name/ID, comma-sep) |
| `--stdin` | | `bool` | No | Read apps from stdin (one per line) |
| `--dry-run` | `-n` | `bool` | No | Preview without removing |

| Component | Value |
|-----------|-------|
| Dynamic completions | team_name, app_name |
| API call | `PUT /api/v1/org/{orgId}/team/{teamId}` |
| Handler | `src/cli/team.rs` |

#### `team set-apps`

Replace all team application assignments.

| Aliases | `team sync-apps` |
|---------|------------------|

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<TEAM>` | | `String` (positional) | Yes | Team ID or name |
| `<APPS>` | | `String[]` (positional) | Unless `--stdin` | Complete app list (comma-sep) |
| `--stdin` | | `bool` | No | Read apps from stdin (one per line) |
| `--dry-run` | `-n` | `bool` | No | Preview changes without applying |
| `--yes` | `-y` | `bool` | No | Skip confirmation prompt |
| `--force` | `-f` | `bool` | No | Allow duplicate app assignments |

| Component | Value |
|-----------|-------|
| Dynamic completions | team_name, app_name |
| API call | `PUT /api/v1/org/{orgId}/team/{teamId}`, `PUT /api/v1/org/{orgId}/team/{teamId}/application` |
| Safety | Apps can only belong to one team at a time |
| Handler | `src/cli/team.rs` |

#### `team list --user` [planned]

List teams for a specific user.

| Component | Value |
|-----------|-------|
| API call | `GET /api/v1/org/{orgId}/user/{userId}/teams` |
| Roadmap | Phase 4 |

---

### `hawkop policy`

Manage scan policies.

#### `policy list`

List scan policies (StackHawk preset and organization custom).

| Component | Value |
|-----------|-------|
| Flattened | PaginationArgs |
| API calls | `GET /api/v1/policy/all`, `GET /api/v1/policy/{orgId}/list` |
| Handler | `src/cli/policy.rs` |

#### `policy get` [planned]

Get a specific scan policy (StackHawk preset or org custom).

| Component | Value |
|-----------|-------|
| API calls | `GET /api/v1/policy` (StackHawk default), `GET /api/v1/policy/{orgId}/{policyName}` (org) |
| Roadmap | Phase 2 |

#### `policy set` [planned]

Create or update an organization scan policy.

| Component | Value |
|-----------|-------|
| API call | `POST /api/v1/policy/{orgId}/update` |
| Roadmap | Phase 2 |

#### `policy delete` [planned]

Delete an organization scan policy.

| Component | Value |
|-----------|-------|
| API call | `DELETE /api/v1/policy/{orgId}/{policyName}` |
| Roadmap | Phase 2 |

---

### `hawkop repo`

Manage repositories in attack surface.

#### `repo list`

List repositories in the organization's attack surface.

| Component | Value |
|-----------|-------|
| Flattened | PaginationArgs |
| API call | `GET /api/v1/org/{orgId}/repos` |
| Handler | `src/cli/repo.rs` |

#### `repo associate` [planned]

Associate applications to repositories.

| Component | Value |
|-----------|-------|
| API call | `PUT /api/v1/org/{orgId}/repos/apps` |
| Roadmap | Phase 4 |

#### `repo set-apps` [planned]

Replace repository application mappings.

| Component | Value |
|-----------|-------|
| API call | `POST /api/v1/org/{orgId}/repo/{repoId}/applications` |
| Roadmap | Phase 4 |

#### `repo sensitive-data` [planned]

List sensitive data findings for a repository.

| Component | Value |
|-----------|-------|
| API call | `GET /api/v1/org/{orgId}/repo/{repoId}/sensitive/list` |
| Roadmap | Phase 4 |

---

### `hawkop oas`

Manage hosted OpenAPI specifications.

#### `oas list`

List hosted OpenAPI specifications.

| Aliases | `oas ls` |
|---------|----------|

| Component | Value |
|-----------|-------|
| Flattened | PaginationArgs |
| API call | `GET /api/v1/oas/{orgId}/list` |
| Handler | `src/cli/oas.rs` |

#### `oas get`

Get OpenAPI specification content.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<OAS_ID>` | | `String` (positional) | Yes | OAS ID (UUID) |
| `--output` | `-o` | `String` | No | Output file path (stdout if omitted) |

| Component | Value |
|-----------|-------|
| API call | `GET /api/v1/oas/{orgId}/{oasId}` |
| Handler | `src/cli/oas.rs` |

#### `oas mappings`

List OpenAPI specs mapped to an application.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `--app` | `-a` | `String` | Yes | Application name or ID |

| Component | Value |
|-----------|-------|
| Dynamic completions | app_name |
| API call | `GET /api/v1/oas/{appId}/mapping` |
| Handler | `src/cli/oas.rs` |

#### `oas map` / `oas unmap` [planned]

Toggle application OAS mapping.

| Component | Value |
|-----------|-------|
| API call | `POST /api/v1/oas/{appId}/mapping` |
| Roadmap | Phase 3 |

---

### `hawkop config`

Manage scan configurations.

#### `config list`

List scan configurations.

| Aliases | `config ls` |
|---------|-------------|

| Component | Value |
|-----------|-------|
| Flattened | PaginationArgs |
| API call | `GET /api/v1/configuration/{orgId}/list` |
| Feature flag | hosted-scan-configs |
| Handler | `src/cli/config.rs` |

#### `config get`

Get a scan configuration's content.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<NAME>` | | `String` (positional) | Yes | Configuration name |
| `--output` | `-o` | `String` | No | Output file path (stdout if omitted) |

| Component | Value |
|-----------|-------|
| API call | `GET /api/v1/configuration/{orgId}/{configName}` |
| Handler | `src/cli/config.rs` |

#### `config set`

Create or update a scan configuration.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<NAME>` | | `String` (positional) | Yes | Configuration name |
| `--file` | `-f` | `String` | Yes | YAML configuration file to upload |

| Component | Value |
|-----------|-------|
| API call | `POST /api/v1/configuration/{orgId}/update` |
| Handler | `src/cli/config.rs` |

#### `config delete`

Delete a scan configuration.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<NAME>` | | `String` (positional) | Yes | Configuration name |
| `--yes` | `-y` | `bool` | No | Skip confirmation prompt |

| Component | Value |
|-----------|-------|
| API call | `DELETE /api/v1/configuration/{orgId}/{configName}` |
| Notes | API implements idempotent DELETE (succeeds even if not found) |
| Handler | `src/cli/config.rs` |

#### `config rename`

Rename a scan configuration.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<OLD_NAME>` | | `String` (positional) | Yes | Current configuration name |
| `<NEW_NAME>` | | `String` (positional) | Yes | New configuration name |

| Component | Value |
|-----------|-------|
| API call | `POST /api/v1/configuration/{orgId}/rename` |
| Handler | `src/cli/config.rs` |

#### `config validate`

Validate a scan configuration.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `[NAME]` | | `String` (positional) | (1) | Config name (validate stored config) |
| `--file` | `-f` | `String` | (1) | YAML file to validate (local file) |

(1) Exactly one of `NAME` or `--file` required (they conflict with each other).

| Component | Value |
|-----------|-------|
| API call | `POST /api/v1/configuration/{orgId}/validate` |
| Handler | `src/cli/config.rs` |

---

### `hawkop secret`

Manage user secrets.

#### `secret list`

List user secrets (names only).

| Component | Value |
|-----------|-------|
| Arguments | (none) |
| API call | `GET /api/v1/user/secret/list` |
| Handler | `src/cli/secret.rs` |

---

### `hawkop audit`

View organization audit log.

#### `audit list`

List audit log entries.

| Component | Value |
|-----------|-------|
| Flattened | AuditFilterArgs |
| API call | `GET /api/v1/org/{orgId}/audit` |
| Cache | Disabled (TTL: none) |
| Handler | `src/cli/audit.rs` |

See [AuditFilterArgs](#auditfilterargs).

---

### `hawkop env`

Manage application environments.

#### `env list`

List environments for an application.

| Aliases | `env ls` |
|---------|----------|

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `--app` | `-a` | `String` | Yes | Application name or ID |
| + PaginationArgs | | | | See [PaginationArgs](#paginationargs) |

| Component | Value |
|-----------|-------|
| Dynamic completions | app_name |
| API call | `GET /api/v1/app/{appId}/env/list` |
| Handler | `src/cli/env.rs` |

#### `env config`

Get default YAML configuration for an environment.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `--app` | `-a` | `String` | Yes | Application name or ID |
| `<ENV>` | | `String` (positional) | Yes | Environment name or ID |
| `--output` | `-o` | `String` | No | Output file path (stdout if omitted) |

| Component | Value |
|-----------|-------|
| Dynamic completions | app_name |
| API call | `GET /api/v1/app/{appId}/env/{envId}/config/default` |
| Handler | `src/cli/env.rs` |

#### `env create`

Create a new environment for an application.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `--app` | `-a` | `String` | Yes | Application name or ID |
| `<NAME>` | | `String` (positional) | Yes | Environment name |

| Component | Value |
|-----------|-------|
| Dynamic completions | app_name |
| API call | `POST /api/v1/app/{appId}/env` |
| Handler | `src/cli/env.rs` |

#### `env delete`

Delete an environment.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `--app` | `-a` | `String` | Yes | Application name or ID |
| `<ENV>` | | `String` (positional) | Yes | Environment name or ID |
| `--yes` | `-y` | `bool` | No | Skip confirmation prompt |

| Component | Value |
|-----------|-------|
| Dynamic completions | app_name |
| API call | `DELETE /api/v1/app/{appId}/env/{envId}` |
| Handler | `src/cli/env.rs` |

#### `env update` [planned]

Update an environment.

| Component | Value |
|-----------|-------|
| API call | `POST /api/v1/app/{appId}/env/{envId}` |
| Roadmap | Phase 3 |

#### `env config set` [planned]

Create/set default YAML config for an environment.

| Component | Value |
|-----------|-------|
| API call | `POST /api/v1/app/{appId}/env/{envId}/config/default` |
| Roadmap | Phase 3 |

#### `env list --all` [planned]

List environments across all apps in the org (v2).

| Component | Value |
|-----------|-------|
| API call | `GET /api/v2/org/{orgId}/envs` |
| Roadmap | Phase 3 |

---

### `hawkop cache`

Manage local response cache.

#### `cache status`

Show cache statistics.

| Component | Value |
|-----------|-------|
| Arguments | (none) |
| Local only | Yes (no API calls) |
| Handler | `src/cli/cache.rs` |

#### `cache clear`

Clear all cached data.

| Component | Value |
|-----------|-------|
| Arguments | (none) |
| Local only | Yes |
| Handler | `src/cli/cache.rs` |

#### `cache path`

Print cache directory path.

| Component | Value |
|-----------|-------|
| Arguments | (none) |
| Local only | Yes |
| Handler | `src/cli/cache.rs` |

---

### `hawkop profile`

Manage configuration profiles (for different orgs, users, or API keys).

| Aliases | `hawkop profiles` |
|---------|-------------------|

#### `profile list`

List all configuration profiles.

| Aliases | `profile ls` |
|---------|--------------|

| Component | Value |
|-----------|-------|
| Arguments | (none) |
| Local only | Yes |
| Handler | `src/cli/profile.rs` |

#### `profile use`

Switch to a different profile.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `<NAME>` | `String` (positional) | Yes | Profile name to activate |

| Component | Value |
|-----------|-------|
| Local only | Yes |
| Handler | `src/cli/profile.rs` |

#### `profile create`

Create a new profile.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `<NAME>` | `String` (positional) | Yes | Name for the new profile |
| `--from` | `String` | No | Copy settings from existing profile |

| Component | Value |
|-----------|-------|
| Local only | Yes |
| Handler | `src/cli/profile.rs` |

#### `profile delete`

Delete a profile.

| Flag | Short | Type | Required | Description |
|------|-------|------|----------|-------------|
| `<NAME>` | | `String` (positional) | Yes | Profile name to delete |
| `--yes` | `-y` | `bool` | No | Skip confirmation prompt |

| Component | Value |
|-----------|-------|
| Local only | Yes |
| Handler | `src/cli/profile.rs` |

#### `profile show`

Show profile details.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `[NAME]` | `String` (positional) | No | Profile name (defaults to active profile) |

| Component | Value |
|-----------|-------|
| Local only | Yes |
| Handler | `src/cli/profile.rs` |

---

### `hawkop completion`

Generate shell completions.

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `<SHELL>` | `bash\|zsh\|fish\|powershell` | Yes | Shell to generate completions for |

Static completions (subcommands/flags). Dynamic completions (API-queried scan IDs, app names, etc.) are available separately via `COMPLETE=<shell> hawkop`.

| Component | Value |
|-----------|-------|
| Local only | Yes |
| Handler | `src/cli/completions.rs` |

---

### `hawkop findings` [planned]

Organization-wide vulnerability findings.

#### `findings list` [planned]

List findings across all scans in the organization.

| Component | Value |
|-----------|-------|
| API call | `GET /api/v1/reports/org/{orgId}/findings` |
| Roadmap | Phase 1 |
| Notes | Design decision needed: top-level `findings` vs nested `scan findings` |

---

## Test Coverage Map

### Functional Tests (require `--features functional-tests`)

| Command | Test File | Test Functions | Coverage |
|---------|-----------|----------------|----------|
| `status` | `read_tests.rs` | `test_status_shows_config` | Basic |
| `version` | `read_tests.rs` | `test_version_shows_version` | Basic |
| `--help` | `read_tests.rs` | `test_help_shows_commands` | Basic |
| `org list` | `read_tests.rs` | `test_org_list_returns_orgs`, `test_org_list_json_format` | Default + JSON |
| `org get` | `read_tests.rs` | `test_org_get_shows_current_org`, `test_org_get_json_format` | Default + JSON |
| `org set` | | | **None** |
| `app list` | `read_tests.rs` | `test_app_list_succeeds`, `_json_format`, `_with_limit`, `_with_type_filter` | Good |
| `scan list` | `read_tests.rs` | `test_scan_list_succeeds`, `_json_format`, `_with_limit`, `_with_status_filter` | Good |
| `scan get` | `error_tests.rs` | `test_nonexistent_scan_id_returns_error` | Error only |
| `run start` | `hosted_tests.rs` | `_without_app_fails`, `_help_shows_options` | Args + help |
| `run stop` | `hosted_tests.rs` | `_without_app_fails` | Args only |
| `run status` | `hosted_tests.rs` | `_without_app_fails`, `_with_nonexistent_app`, `_help_shows_options`, `_feature_flag_check` | Good |
| `user list` | `read_tests.rs` | `test_user_list_returns_users`, `_json_format` | Default + JSON |
| `team list` | `read_tests.rs` | `test_team_list_succeeds`, `_json_format`, `_with_filters` | Good |
| `team get` | `mutation_tests.rs` | `test_team_get_by_name`, `_json_format` | Good |
| `team create` | `mutation_tests.rs` | `_and_auto_cleanup`, `_json_output`, `_with_dry_run`, `_duplicate_fails`, `_empty_name_fails` | Excellent |
| `team delete` | `mutation_tests.rs` | `_with_yes_flag`, `_with_dry_run`, `_nonexistent_fails` | Excellent |
| `team rename` | `mutation_tests.rs` | `_succeeds`, `_with_dry_run`, `_to_empty_fails` | Excellent |
| `team add-user` | `mutation_tests.rs` | `_dry_run` | Dry-run only |
| `team remove-user` | `mutation_tests.rs` | `_dry_run` | Dry-run only |
| `team set-users` | | | **None** |
| `team add-app` | `mutation_tests.rs` | `_dry_run` | Dry-run only |
| `team remove-app` | `mutation_tests.rs` | `_dry_run` | Dry-run only |
| `team set-apps` | | | **None** |
| `policy list` | `read_tests.rs` | `test_policy_list_succeeds`, `_json_format` | Default + JSON |
| `repo list` | `read_tests.rs` | `test_repo_list_succeeds`, `_json_format` | Default + JSON |
| `oas list` | `read_tests.rs` | `test_oas_list_succeeds`, `_json_format` | Feature-flag aware |
| `oas get` | `hosted_tests.rs` | `_without_id_fails`, `_nonexistent` | Error only |
| `oas mappings` | `hosted_tests.rs` | `_without_app_fails`, `_with_nonexistent_app`, `_feature_flag_check` | Good |
| `config list` | `read_tests.rs` | `test_config_list_succeeds`, `_json_format` | Feature-flag aware |
| `config get` | `hosted_tests.rs` | `_nonexistent` | Error only |
| `config set` | `hosted_tests.rs` | `_missing_file_flag`, `_file_not_found` | Error only |
| `config delete` | `hosted_tests.rs` | `_nonexistent` | Error only |
| `config rename` | `hosted_tests.rs` | `_nonexistent` | Error only |
| `config validate` | `hosted_tests.rs` | `_missing_args`, `_file_not_found` | Error only |
| `secret list` | `read_tests.rs` | `test_secret_list_succeeds`, `_json_format` | Default + JSON |
| `audit list` | `read_tests.rs` | `test_audit_list_succeeds`, `_json_format` | Default + JSON |
| `env list` | `hosted_tests.rs` | `_without_app_fails`, `_with_nonexistent_app`, `_feature_flag_check` | Good |
| `env config` | `hosted_tests.rs` | `_without_app_fails` | Args only |
| `env create` | `hosted_tests.rs` | `_without_app_fails` | Args only |
| `env delete` | `hosted_tests.rs` | `_without_app_fails` | Args only |
| `cache status` | `read_tests.rs` | `test_cache_status_succeeds` | Basic |
| `cache path` | `read_tests.rs` | `test_cache_path_shows_path` | Basic |
| `cache clear` | | | **None** |
| `profile list` | | | **None** |
| `profile use` | | | **None** |
| `profile create` | | | **None** |
| `profile delete` | | | **None** |
| `profile show` | | | **None** |
| `completion bash` | `read_tests.rs` | `test_completion_bash` | Basic |
| `completion zsh` | `read_tests.rs` | `test_completion_zsh` | Basic |

### Error & Edge Case Tests (`error_tests.rs`)

| Scenario | Test Functions |
|----------|----------------|
| Invalid org ID | `test_invalid_org_id_returns_helpful_error`, `test_malformed_uuid_org_id` |
| Nonexistent team | `test_nonexistent_team_returns_not_found`, `_delete_returns_not_found` |
| Nonexistent scan | `test_nonexistent_scan_id_returns_error` |
| Missing arguments | `test_team_create_missing_name_shows_help`, `_get_missing_identifier`, `_rename_missing_args` |
| Invalid commands | `test_unknown_command_shows_suggestions`, `test_unknown_subcommand_shows_help` |
| Invalid flag values | `test_invalid_format_value`, `_limit_value_non_numeric`, `_negative_limit_value` |
| User resolution | `test_nonexistent_user_in_team_add` |
| App resolution | `test_nonexistent_app_in_team_add` |
| Error message quality | `test_error_includes_identifier`, `test_team_not_found_suggests_list` |

### Unit Tests (`tests/cli.rs`)

- Config file handling and precedence
- Org override resolution
- V2 API endpoint selection
- Error message formatting (auth, rate limit, server, network)

### Coverage Gaps

Commands with **no functional tests**:
- `org set`
- `team set-users`, `team set-apps`
- `cache clear`
- All `profile` commands (`list`, `use`, `create`, `delete`, `show`)
- `init` (interactive — difficult to test non-interactively)

Commands with **error-only tests** (no happy path):
- `scan get`, `oas get`, `config get/set/delete/rename/validate`

---

## Planned Commands Summary

See [ROADMAP.md](ROADMAP.md) for full details and prioritization.

| Phase | Command | API Endpoint |
|-------|---------|--------------|
| 1 | `app get` | `GET /api/v1/app/{appId}` |
| 1 | `app create` | `POST /api/v1/org/{orgId}/app` |
| 1 | `app update` | `POST /api/v1/app/{appId}` |
| 1 | `app delete` | `DELETE /api/v1/app/{appId}` |
| 1 | `findings list` | `GET /api/v1/reports/org/{orgId}/findings` |
| 2 | `policy get` | `GET /api/v1/policy/{orgId}/{policyName}` |
| 2 | `policy set` | `POST /api/v1/policy/{orgId}/update` |
| 2 | `policy delete` | `DELETE /api/v1/policy/{orgId}/{policyName}` |
| 2 | `app policy get` | `GET /api/v1/app/{appId}/policy` |
| 2 | `app policy assign` | `PUT /api/v1/app/{appId}/policy/assign` |
| 2 | `app policy flags` | `GET/PUT /api/v1/app/{appId}/policy/flags` |
| 2 | `app policy toggle` | `GET /api/v1/app/{appId}/policy/plugins/{pluginId}/{toggle}` |
| 3 | `env update` | `POST /api/v1/app/{appId}/env/{envId}` |
| 3 | `env config set` | `POST /api/v1/app/{appId}/env/{envId}/config/default` |
| 3 | `env list --all` | `GET /api/v2/org/{orgId}/envs` |
| 3 | `oas map` / `oas unmap` | `POST /api/v1/oas/{appId}/mapping` |
| 4 | `repo associate` | `PUT /api/v1/org/{orgId}/repos/apps` |
| 4 | `repo set-apps` | `POST /api/v1/org/{orgId}/repo/{repoId}/applications` |
| 4 | `repo sensitive-data` | `GET /api/v1/org/{orgId}/repo/{repoId}/sensitive/list` |
| 4 | `scan delete` | `DELETE /api/v1/scan/{scanId}` |
| 4 | `app alerts set-rule` | `POST /api/v1/app/{appId}/alerts/rules/{integrationId}` |
| 4 | `team list --user` | `GET /api/v1/org/{orgId}/user/{userId}/teams` |

---

## How to Update This Document

This document should be updated whenever:
1. A new command or subcommand is added
2. Flags/arguments change on an existing command
3. A planned command is implemented (remove `[planned]` marker)
4. New functional tests are added (update test coverage map)

The source of truth for implemented commands is always `src/cli/mod.rs`.
