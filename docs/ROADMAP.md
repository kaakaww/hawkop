# HawkOp API Coverage Roadmap

**Last updated**: 2026-03-14
**Current version**: v0.4.0
**OAS reference**: `stackhawk-openapi.json` (root of repo)
**Current coverage**: 36/60 endpoints (60%)

This document tracks HawkOp's progress toward 100% coverage of the [StackHawk Public API](https://apidocs.stackhawk.com/docs). Each phase groups related endpoints by user value and implementation dependencies.

---

## Coverage Summary

| Phase | Area | Endpoints | Status |
|-------|------|-----------|--------|
| Done | Auth, List/Get, Scan drill-down, Teams CRUD, Configs CRUD, Envs, Hosted scans, Audit, Secrets | 36 | Complete |
| 1 | App CRUD + Org Findings | 5 | Not started |
| 2 | Policy Management | 9 | Not started |
| 3 | Environment + OAS Completion | 4 | Not started |
| 4 | Repo Management + Misc | 6 | Not started |

---

## Phase 1 — App CRUD + Org Findings

**Goal**: Complete the application lifecycle and add org-wide vulnerability reporting.
**Why first**: Applications are the core resource. Users can list apps but can't create, get, update, or delete them. Org findings provide the most-requested security overview.

| Endpoint | Method | operationId | CLI Command | Status |
|----------|--------|-------------|-------------|--------|
| `/api/v1/org/{orgId}/app` | POST | `createApplication` | `app create` | Not started |
| `/api/v1/app/{appId}` | GET | `getApplication` | `app get` | Not started |
| `/api/v1/app/{appId}` | POST | `updateApplication` | `app update` | Not started |
| `/api/v1/app/{appId}` | DELETE | `deleteApplication` | `app delete` | Not started |
| `/api/v1/reports/org/{orgId}/findings` | GET | `listOrganizationFindings` | `findings list` | Not started |

### Notes
- `app create` should support `--name`, `--type`, `--env` flags
- `app delete` must require `--yes` or interactive confirmation
- `findings list` should support filtering by severity, status, app, and date range
- Consider whether `findings` is a top-level command or nested under `scan`

---

## Phase 2 — Policy Management

**Goal**: Enable policy-as-code workflows from the CLI.
**Why second**: Policy management is the next highest-value gap — security teams want to automate policy configuration.

| Endpoint | Method | operationId | CLI Command | Status |
|----------|--------|-------------|-------------|--------|
| `/api/v1/policy` | GET | `getStackHawkScanPolicy` | `policy get --stackhawk` | Not started |
| `/api/v1/policy/{orgId}/{policyName}` | GET | `getScanPolicyForOrg` | `policy get NAME` | Not started |
| `/api/v1/policy/{orgId}/update` | POST | `setScanPolicyForOrg` | `policy set` | Not started |
| `/api/v1/policy/{orgId}/{policyName}` | DELETE | `deleteScanPolicyForOrg` | `policy delete NAME` | Not started |
| `/api/v1/app/{appId}/policy` | GET | `getApplicationScanPolicy` | `app policy get` | Not started |
| `/api/v1/app/{appId}/policy/assign` | PUT | `assignAppPlugins` | `app policy assign` | Not started |
| `/api/v1/app/{appId}/policy/flags` | GET | `getAppTechFlags` | `app policy flags` | Not started |
| `/api/v1/app/{appId}/policy/flags` | PUT | `updateAppTechFlags` | `app policy flags --set` | Not started |
| `/api/v1/app/{appId}/policy/plugins/{pluginId}/{toggle}` | GET | `toggleAppPlugin` | `app policy toggle` | Not started |

### Notes
- Org policy CRUD follows the same pattern as config CRUD (list/get/set/delete)
- App-level policy commands may be nested: `hawkop app policy ...`
- Tech flags and plugin toggles are specialized — design CLI carefully
- `toggleAppPlugin` uses GET with a path param toggle (unusual) — verify behavior

---

## Phase 3 — Environment + OAS Completion

**Goal**: Round out environment management and OAS mapping capabilities.

| Endpoint | Method | operationId | CLI Command | Status |
|----------|--------|-------------|-------------|--------|
| `/api/v1/app/{appId}/env/{envId}` | POST | `updateEnvironment` | `env update` | Not started |
| `/api/v1/app/{appId}/env/{envId}/config/default` | POST | `createEnvironmentDefaultConfig` | `env config set` | Not started |
| `/api/v2/org/{orgId}/envs` | GET | `listEnvsV2` | `env list --all` | Not started |
| `/api/v1/oas/{appId}/mapping` | POST | `toggleApplicationOASMapping` | `oas map` / `oas unmap` | Not started |

### Notes
- `env list --all` (v2) lists envs across all apps in the org — different from current per-app listing
- `oas map`/`unmap` toggle should confirm the action before proceeding

---

## Phase 4 — Repo Management + Misc

**Goal**: Complete remaining endpoints for full API parity.

| Endpoint | Method | operationId | CLI Command | Status |
|----------|--------|-------------|-------------|--------|
| `/api/v1/org/{orgId}/repos/apps` | PUT | `createAppsForRepos` | `repo associate` | Not started |
| `/api/v1/org/{orgId}/repo/{repoId}/applications` | POST | `replaceRepoAppMappings` | `repo set-apps` | Not started |
| `/api/v1/org/{orgId}/repo/{repoId}/sensitive/list` | GET | `listRepoSensitiveData` | `repo sensitive-data` | Not started |
| `/api/v1/scan/{scanId}` | DELETE | `deleteScan` | `scan delete` | Not started |
| `/api/v1/app/{appId}/alerts/rules/{integrationId}` | POST | `upsertAlertRule` | `app alerts set-rule` | Not started |
| `/api/v1/org/{orgId}/user/{userId}/teams` | GET | `listTeamsForOrgAndUser` | `team list --user` | Not started |

### Stretch / Low Priority
| Endpoint | Method | operationId | Notes |
|----------|--------|-------------|-------|
| `/api/v1/app/{appId}/config/{configHash}` | GET | `getConfig` | Historical config by hash — niche use case |

---

## Already Implemented (reference)

These endpoints are fully implemented and tested:

### Auth & User
- `GET /api/v1/auth/login` — JWT authentication
- `GET /api/v1/auth/refresh-token` — Token refresh (internal)
- `GET /api/v1/user` — Current user + org list

### Applications (read)
- `GET /api/v1/app/{orgId}/list` — List apps (v1)
- `GET /api/v2/org/{orgId}/apps` — List apps (v2, paginated)

### Scans
- `GET /api/v1/scan/{orgId}` — List scans
- `GET /api/v1/scan/{scanId}/alerts` — Scan alerts
- `GET /api/v1/scan/{scanId}/alert/{pluginId}` — Alert findings
- `GET /api/v1/scan/{scanId}/uri/{alertUriId}/messages/{messageId}` — Alert messages

### Teams (full CRUD)
- `GET /api/v1/org/{orgId}/teams` — List teams
- `GET /api/v1/org/{orgId}/team/{teamId}` — Get team
- `POST /api/v1/org/{orgId}/team` — Create team
- `PUT /api/v1/org/{orgId}/team/{teamId}` — Update team
- `DELETE /api/v1/org/{orgId}/team/{teamId}` — Delete team
- `PUT /api/v1/org/{orgId}/team/{teamId}/application` — Assign app

### Members
- `GET /api/v1/org/{orgId}/members` — List members

### Policies (read)
- `GET /api/v1/policy/all` — StackHawk preset policies
- `GET /api/v1/policy/{orgId}/list` — Org policies

### Scan Configurations (full CRUD)
- `GET /api/v1/configuration/{orgId}/list` — List configs
- `GET /api/v1/configuration/{orgId}/{configName}` — Get config
- `POST /api/v1/configuration/{orgId}/update` — Set config
- `DELETE /api/v1/configuration/{orgId}/{configName}` — Delete config
- `POST /api/v1/configuration/{orgId}/rename` — Rename config
- `POST /api/v1/configuration/{orgId}/validate` — Validate config

### OAS (read)
- `GET /api/v1/oas/{orgId}/list` — List OAS assets
- `GET /api/v1/oas/{orgId}/{oasId}` — Get OAS content
- `GET /api/v1/oas/{appId}/mapping` — Get OAS mappings

### Repositories (read)
- `GET /api/v1/org/{orgId}/repos` — List repos

### Audit
- `GET /api/v1/org/{orgId}/audit` — Audit log

### Environments
- `GET /api/v1/app/{appId}/env/list` — List environments
- `GET /api/v1/app/{appId}/env/{envId}/config/default` — Get default config
- `POST /api/v1/app/{appId}/env` — Create environment
- `DELETE /api/v1/app/{appId}/env/{envId}` — Delete environment

### Hosted Scans (Perch)
- `POST /api/v1/app/{appId}/perch/start` — Start scan
- `POST /api/v1/app/{appId}/perch/stop` — Stop scan
- `GET /api/v1/app/{appId}/perch/status` — Scan status

---

## How to Update This Document

When implementing a new endpoint:
1. Change its status from `Not started` to `In progress` or `Complete`
2. Update the coverage count in the summary table
3. Add any implementation notes or design decisions
4. After completing a phase, move its entries to the "Already Implemented" section
