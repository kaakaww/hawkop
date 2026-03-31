# HawkOp API Coverage Roadmap

**Last updated**: 2026-03-30
**Current version**: v0.5.1+ (feature/refinements)
**OAS reference**: `stackhawk-openapi.json` (root of repo, refreshed 2026-03-28)
**Current coverage**: 38/55 endpoints (69%)

This document tracks HawkOp's progress toward 100% coverage of the [StackHawk Public API](https://apidocs.stackhawk.com/docs). Each phase groups related endpoints by user value and implementation dependencies.

---

## Coverage Summary

| Phase | Area | Endpoints | Status |
|-------|------|-----------|--------|
| Done | Auth, List/Get, Scan drill-down, Teams CRUD, Configs CRUD, Envs, Hosted scans, OAS mappings, Audit, Secrets, App CRUD, Repo link | 38 | Complete |
| 1 | App CRUD + Org Findings | 5 | Partial (4/5 — findings list pending) |
| 2 | Policy Management | 7 | Not started |
| 3 | OAS + Env Completion | 3 | Not started |
| 4 | Repo Management + Misc | 3 | Partial (2/3 — repo set-apps + repo link done) |
| 5 | Profile Scans + Triage (deferred — under active development) | 6 | Not started |

---

## Phase 1 — App CRUD + Org Findings

**Goal**: Complete the application lifecycle and add org-wide vulnerability reporting.
**Why first**: Applications are the core resource. Users can list apps but can't create, get, update, or delete them. Org findings provide the most-requested security overview.

| Endpoint | Method | operationId | CLI Command | Status |
|----------|--------|-------------|-------------|--------|
| `/api/v1/org/{orgId}/app` | POST | `createApplication` | `app create` | Complete |
| `/api/v1/app/{appId}` | GET | `getApplication` | `app get` | Complete |
| `/api/v1/app/{appId}` | POST | `updateApplication` | `app update` | Complete |
| `/api/v1/app/{appId}` | DELETE | `deleteApplication` | `app delete` | Complete |
| `/api/v1/reports/org/{orgId}/findings` | GET | `listOrganizationFindings` | `findings list` | Not started |

### Notes
- `app create` supports `--name`, `--type`, `--env`, `--host`, `--cloud-url`, `--team-id`, `--repo`/`--repo-id` ✅
- `app delete` requires `--yes` or interactive confirmation ✅
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
| `/api/v1/app/{appId}/policy/assign` | PUT | `assignAppPlugins` | `app policy assign` | Not started |
| `/api/v1/app/{appId}/policy/flags` | GET | `getAppTechFlags` | `app policy flags` | Not started |
| `/api/v1/app/{appId}/policy/flags` | PUT | `updateAppTechFlags` | `app policy flags --set` | Not started |
| `/api/v1/app/{appId}/policy/plugins/{pluginId}/{toggle}` | GET | `toggleAppPlugin` | `app policy toggle` | Not started |

### Notes
- Org policy CRUD follows the same pattern as config CRUD (list/get/set)
- `DELETE /api/v1/policy/{orgId}/{policyName}` and `GET /api/v1/app/{appId}/policy` were **removed from the API** in 2026-03 spec update
- App-level policy commands may be nested: `hawkop app policy ...`
- Tech flags and plugin toggles are specialized — design CLI carefully
- `toggleAppPlugin` uses GET with a path param toggle (unusual) — verify behavior

---

## Phase 3 — OAS + Env Completion

**Goal**: Migrate OAS commands to new app-scoped API and round out environment management.

| Endpoint | Method | operationId | CLI Command | Status |
|----------|--------|-------------|-------------|--------|
| `/api/v1/oas/{appId}/upload` | POST | `uploadOAS` | `oas upload` | Not started |
| `/api/v1/oas/{appId}/mapping` | POST | `toggleApplicationOASMapping` | `oas map` / `oas unmap` | Not started |
| `/api/v1/app/{appId}/env/{envId}` | POST | `updateEnvironment` | `env update` | Not started |

### Notes
- **⚠️ OAS endpoint migration needed**: `oas list` and `oas get` use removed org-scoped endpoints (`/api/v1/oas/{orgId}/list` and `/api/v1/oas/{orgId}/{oasId}`). These need to be migrated to app-scoped `GET /api/v1/oas/{appId}/mapping` or removed if no replacement exists.
- `GET /api/v1/app/{appId}/env/{envId}/config/default` was removed from the spec — `env config set` is no longer viable
- `GET /api/v2/org/{orgId}/envs` (list all envs across org) is still available but lower priority
- `oas upload` is a new endpoint — enables uploading specs from the CLI

---

## Phase 4 — Repo Management + Misc

**Goal**: Complete remaining endpoints for full API parity.

| Endpoint | Method | operationId | CLI Command | Status |
|----------|--------|-------------|-------------|--------|
| `/api/v1/org/{orgId}/repos/apps` | PUT | `createAppsForRepos` | `repo associate` | Not started |
| `/api/v1/org/{orgId}/repo/{repoId}/applications` | POST | `replaceRepoAppMappings` | `repo set-apps` / `repo link` | Complete |
| `/api/v1/org/{orgId}/repo/{repoId}/sensitive/list` | GET | `listRepoSensitiveData` | `repo sensitive-data` | Not started |

### Also available but lower priority
| Endpoint | Method | CLI Command | Notes |
|----------|--------|-------------|-------|
| `/api/v1/scan/{scanId}` | DELETE | `scan delete` | Destructive — needs confirmation |
| `/api/v1/org/{orgId}/user/{userId}/teams` | GET | `team list --user` | Convenience filter |
| `/api/v1/global-configuration/{configName}` | GET | `global-config get` | Returns S3 redirect to shared HawkScan configs |

### Removed from API (previously planned)
- ~~`POST /api/v1/app/{appId}/alerts/rules/{integrationId}`~~ — Alert rule upsert removed in 2026-03 spec update

---

## Phase 5 — Profile Scans + Triage (deferred)

**Goal**: Support StackHawk's new profile scanning and bulk triage capabilities.
**Why deferred**: These features are under active development upstream. Wait for API stability before building CLI support.

| Endpoint | Method | operationId | CLI Command | Status |
|----------|--------|-------------|-------------|--------|
| `POST /api/v1/app/{appId}/perch/profile-scan` | POST | `launchProfileScan` | `run profile` | Not started |
| `GET /api/v1/app/{appId}/profile/results` | GET | `getLatestProfileScanResult` | `profile get` | Not started |
| `GET /api/v1/app/{appId}/profile/results/list` | GET | `listProfileScanResults` | `profile list` | Not started |
| `GET /api/v1/app/{appId}/profile/results/{scanId}` | GET | `getProfileScanResult` | `profile get --scan` | Not started |
| `POST /api/v1/org/{orgId}/profile/results` | POST | `bulkGetProfileResults` | `profile list --org` | Not started |
| `POST /api/v1/org/{orgId}/app/{appId}/env/{envId}/findings/triage` | POST | `bulkTriageFindings` | `findings triage` | Not started |

### Notes
- Profile scans return testability analysis: app classification, auth markers, path discovery, asset inventory, recommendations
- Findings triage uses `findingHash` (SHA-256) for stable cross-scan identification — `findingHash` field was added to existing `AlertMsgResponse` and `ApplicationAlertUri` schemas
- `scan list` now supports `--tag` filtering (`tag=branch:main|develop`) — could be added independently
- New Perch bulk status endpoint (`POST /api/v1/org/{orgId}/perch/status`) enables multi-app device queries

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

### OAS
- `GET /api/v1/oas/{appId}/mapping` — Get OAS mappings
- ⚠️ `GET /api/v1/oas/{orgId}/list` — **Removed from spec** (still called by `oas list`)
- ⚠️ `GET /api/v1/oas/{orgId}/{oasId}` — **Removed from spec** (still called by `oas get`)

### Applications (write)
- `POST /api/v1/org/{orgId}/app` — Create application
- `GET /api/v1/app/{appId}` — Get application
- `POST /api/v1/app/{appId}` — Update application
- `DELETE /api/v1/app/{appId}` — Delete application

### Repositories (read + write)
- `GET /api/v1/org/{orgId}/repos` — List repos
- `POST /api/v1/org/{orgId}/repo/{repoId}/applications` — Replace repo app mappings (used by `repo link` and `repo set-apps`)

### Audit
- `GET /api/v1/org/{orgId}/audit` — Audit log

### Environments
- `GET /api/v1/app/{appId}/env/list` — List environments
- `POST /api/v1/app/{appId}/env` — Create environment
- `DELETE /api/v1/app/{appId}/env/{envId}` — Delete environment
- ⚠️ `GET /api/v1/app/{appId}/env/{envId}/config/default` — **Removed from spec** (still called by `env config`)

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
