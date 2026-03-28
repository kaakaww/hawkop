# StackHawk API Reference for HawkOp

> **Last Updated**: 2026-03-28
> **OpenAPI Spec Version**: 3.0.1
> **HawkOp Version**: 0.4.0
> **Total Endpoints**: 55 | **Implemented**: 33 (~60%)

---

## Overview

### API Versions
- **v1**: `https://api.stackhawk.com/api/v1` - Most endpoints
- **v2**: `https://api.stackhawk.com/api/v2` - Newer list endpoints with better pagination

### Authentication
- Exchange API key for JWT via `GET /api/v1/auth/login`
- JWT expires after ~1 hour; refresh with `/api/v1/auth/refresh-token`
- Include JWT in `Authorization: Bearer <token>` header

### Rate Limiting
HawkOp uses reactive per-endpoint rate limiting (activates after 429):
| Category | Rate | Endpoints |
|----------|------|-----------|
| Scan | 80/sec | `/scan/*` |
| User | 80/sec | `/user`, `/members` |
| AppList | 80/sec | `/apps`, `/app/list` |
| Default | 6/sec | Everything else |

### Pagination
- `pageSize`: Max 1000 items per page
- `pageToken`: Cursor for next page
- `sortField` / `sortDir`: Sorting options
- Response includes `totalCount` (sometimes as string!)

---

## Data Model Relationships

```
Organization (org)
├── Applications (app)
│   ├── Environments (env)
│   │   └── Scan Configurations
│   ├── Scans (scan)
│   │   ├── Alerts (plugin-level findings)
│   │   │   └── Alert URIs (specific vulnerable paths)
│   │   │       └── Messages (HTTP request/response)
│   │   │           └── Finding Hash (stable cross-scan identifier)
│   │   └── Alert Stats (H/M/L counts)
│   ├── Profile Scans (testability analysis)
│   │   ├── Classification (SPA, API, website, static)
│   │   ├── Auth Markers (detected auth signals)
│   │   ├── Path Discovery (static/dynamic/auth-protected)
│   │   ├── Asset Inventory (scripts, media, dynamic content)
│   │   └── Recommendations
│   ├── Findings Triage (bulk status updates by hash)
│   ├── Scan Policies (which checks to run)
│   └── Hosted OAS (app-scoped OpenAPI specs)
├── Teams
│   ├── Members (users)
│   └── Applications (team ownership)
├── Repositories (attack surface)
│   ├── Sensitive Data Tags
│   └── Application Mappings
├── Scan Configurations (YAML files)
├── Global Configurations (shared HawkScan configs)
└── Audit Log
```

---

## Endpoints by Category

### Legend
- ✅ = Implemented in HawkOp
- 🔲 = Available but not implemented
- 🔸 = Partially implemented

---

### Authentication (2 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/auth/login` | (internal) | Exchange API key for JWT |
| 🔲 | GET | `/api/v1/auth/refresh-token` | - | Refresh JWT token |

---

### User (1 endpoint)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/user` | `org list` | Get current user + orgs |

**Key fields**: `id`, `name`, `email`, `organizations[]`

---

### Organizations (2 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/org/{orgId}/members` | `user list` | List org members |
| ✅ | GET | `/api/v1/org/{orgId}/audit` | `audit list` | Audit log history |

**Member fields**: `id`, `name`, `email`, `role`, `status`
**Audit fields**: `id`, `type`, `user`, `timestamp`, `details`

---

### Applications (8 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/app/{orgId}/list` | `app list` (v1) | List applications |
| ✅ | GET | `/api/v2/org/{orgId}/apps` | `app list` (v2) | List applications (better) |
| 🔲 | GET | `/api/v1/app/{appId}` | - | Get single application |
| 🔲 | POST | `/api/v1/org/{orgId}/app` | - | Create application |
| 🔲 | POST | `/api/v1/app/{appId}` | - | Update application |
| 🔲 | DELETE | `/api/v1/app/{appId}` | - | Delete application |
| ✅ | GET | `/api/v1/app/{appId}/env/list` | `env list` | List environments |
| 🔲 | GET | `/api/v2/org/{orgId}/envs` | - | List environments (v2) |

**App fields**: `applicationId`, `name`, `cloudScanTargets[]`, `env`, `riskGrade`

**Potential commands**:
- `app get <ID>` - View single app details
- `app create` - Create new application

---

### Environments (3 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | POST | `/api/v1/app/{appId}/env` | `env create` | Create environment |
| 🔲 | POST | `/api/v1/app/{appId}/env/{envId}` | - | Update environment |
| ✅ | DELETE | `/api/v1/app/{appId}/env/{envId}` | `env delete` | Delete environment |

**Removed in spec (2026-03)**:
- ~~`GET /api/v1/app/{appId}/env/{envId}/config/default`~~ — Get default YAML config endpoint removed

---

### Scans & Results (7 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/scan/{orgId}` | `scan list` | List scan results |
| ✅ | GET | `/api/v1/scan/{scanId}/alerts` | `scan get` | List scan alerts (findings) |
| ✅ | GET | `/api/v1/scan/{scanId}/alert/{pluginId}` | `scan get --plugin` | Alert details + paths |
| ✅ | GET | `/api/v1/scan/{scanId}/uri/{alertUriId}/messages/{messageId}` | `scan get --message` | HTTP request/response |
| 🔲 | DELETE | `/api/v1/scan/{scanId}` | - | Delete a scan |
| 🔲 | GET | `/api/v1/reports/org/{orgId}/findings` | - | Organization findings report |
| 🔲 | POST | `/api/v1/org/{orgId}/app/{appId}/env/{envId}/findings/triage` | - | Bulk triage findings by hash |

**Scan fields**: `scanId`, `applicationId`, `applicationName`, `scanStatus`, `alertStats`, `startedTimestamp`, `completedTimestamp`
**Alert fields**: `pluginId`, `pluginName`, `severity`, `count`, `paths[]`
**Finding hash**: `findingHash` — SHA-256 stable identifier for a finding across scans (on `AlertMsgResponse` and `ApplicationAlertUri`)

**Implemented features**:
- Drill-down: scan → alerts → URIs → messages
- Filter by severity, plugin, path
- View full HTTP request/response with `--curl` flag

**New in spec (2026-03)**:
- `tag` query parameter on scan list: filter by `name:value`, supports OR (`|`), wildcards (`*`), and AND (repeat param). Example: `tag=branch:main|develop`
- `findingHash` field added to `AlertMsgResponse` and `ApplicationAlertUri` — enables cross-scan finding tracking
- Bulk triage endpoint accepts array of `FindingTriageAction` with `findingHash`, `status`, and optional `note`

**Potential commands**:
- `scan list --tag branch:main` - Filter scans by tag
- `findings triage --app <APP> --env <ENV>` - Bulk triage findings

---

### Scan Control - Perch (5 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | POST | `/api/v1/app/{appId}/perch/start` | `run start` | Start hosted scan |
| ✅ | GET | `/api/v1/app/{appId}/perch/status` | `run status` | Get scan status |
| ✅ | POST | `/api/v1/app/{appId}/perch/stop` | `run stop` | Stop hosted scan |
| 🔲 | POST | `/api/v1/app/{appId}/perch/profile-scan` | - | Launch profile scan (testability analysis) |
| 🔲 | POST | `/api/v1/org/{orgId}/perch/status` | - | Search devices for multiple apps (bulk status) |

**New in spec (2026-03)**:
- Profile scan launch via Perch — triggers automatic OpenAPI spec discovery and testability analysis
- Bulk device status — query scan device state across multiple apps in one call
- `PerchDevice` now includes `configValidationResult`, `scanId`, `updatedDate`

**Potential commands**:
- `run start --app <ID>` - Trigger hosted scan
- `run status --app <ID>` - Check running scan
- `run stop --app <ID>` - Cancel scan
- `run profile --app <ID>` - Launch profile scan

---

### Scan Policies (7 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/policy/all` | `policy list` | All StackHawk policies |
| ✅ | GET | `/api/v1/policy/{orgId}/list` | `policy list --org` | Org custom policies |
| 🔲 | GET | `/api/v1/policy` | - | Get single SH policy |
| 🔲 | GET | `/api/v1/policy/{orgId}/{policyName}` | - | Get org policy |
| 🔲 | POST | `/api/v1/policy/{orgId}/update` | - | Update org policy |
| 🔲 | PUT | `/api/v1/app/{appId}/policy/assign` | - | Assign policy to app |
| 🔲 | GET | `/api/v1/app/{appId}/policy/plugins/{pluginId}/{toggle}` | - | Toggle plugin |

**Removed in spec (2026-03)**:
- ~~`DELETE /api/v1/policy/{orgId}/{policyName}`~~ — Policy delete method removed
- ~~`GET /api/v1/app/{appId}/policy`~~ — App policy get endpoint removed

**Policy fields**: `name`, `type` (Stackhawk/Organization), `plugins[]`, `enabled`

---

### Scan Configurations (6 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/configuration/{orgId}/list` | `config list` | List YAML configs |
| ✅ | GET | `/api/v1/configuration/{orgId}/{configName}` | `config get` | Get config content |
| ✅ | POST | `/api/v1/configuration/{orgId}/update` | `config set` | Create/update config |
| ✅ | POST | `/api/v1/configuration/{orgId}/rename` | `config rename` | Rename config |
| ✅ | POST | `/api/v1/configuration/{orgId}/validate` | `config validate` | Validate YAML |
| ✅ | DELETE | `/api/v1/configuration/{orgId}/{configName}` | `config delete` | Delete config |

---

### Teams (7 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/org/{orgId}/teams` | `team list` | List all teams |
| ✅ | GET | `/api/v1/org/{orgId}/team/{teamId}` | `team get` | Get team details |
| ✅ | POST | `/api/v1/org/{orgId}/team` | `team create` | Create team |
| ✅ | PUT | `/api/v1/org/{orgId}/team/{teamId}` | `team update` | Update team |
| ✅ | DELETE | `/api/v1/org/{orgId}/team/{teamId}` | `team delete` | Delete team |
| ✅ | PUT | `/api/v1/org/{orgId}/team/{teamId}/application` | `team add-app/remove-app/set-apps` | Assign app to team |
| 🔲 | GET | `/api/v1/org/{orgId}/user/{userId}/teams` | - | User's teams |

**Team fields**: `teamId`, `name`, `applications[]`, `users[]`

**Implemented features**:
- Full CRUD: create, get, update, delete teams
- Member management: add-user, remove-user, set-users (SCIM sync)
- App assignment: add-app, remove-app, set-apps
- Flexible identifier resolution (name or UUID)
- Dry-run mode for all mutating operations
- Shell completions for team names, user emails, app names

---

### Repositories / Attack Surface (4 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/org/{orgId}/repos` | `repo list` | List repositories |
| 🔲 | PUT | `/api/v1/org/{orgId}/repos/apps` | - | Link app to repo |
| 🔲 | POST | `/api/v1/org/{orgId}/repo/{repoId}/applications` | - | Replace app mappings |
| 🔲 | GET | `/api/v1/org/{orgId}/repo/{repoId}/sensitive/list` | - | Sensitive data in repo |

**Repo fields**: `repoId`, `name`, `url`, `languages[]`, `contributors[]`, `apps[]`, `sensitiveDataTags[]`

---

### Hosted OAS (3 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/oas/{appId}/mapping` | `oas mappings` | App's mapped specs |
| 🔲 | POST | `/api/v1/oas/{appId}/mapping` | - | Map spec to app |
| 🔲 | POST | `/api/v1/oas/{appId}/upload` | - | Upload OpenAPI spec file |

**⚠️ BREAKING CHANGE (2026-03)**: OAS endpoints restructured from org-scoped to app-scoped:
- ~~`GET /api/v1/oas/{orgId}/list`~~ — **Removed** (was used by `oas list` command)
- ~~`GET /api/v1/oas/{orgId}/{oasId}`~~ — **Removed** (was used by `oas get` command)

Our `oas list` and `oas get` commands call removed endpoints. They may still work if the API hasn't been decommissioned yet, but should be migrated to use app-scoped `GET /api/v1/oas/{appId}/mapping` instead. See `api-quirks.md` for migration details.

**OAS fields**: `oasId`, `fileName`, `fileSize`, `uploadedAt`, `applications[]`

**Potential commands**:
- `oas upload --app <APP> <FILE>` - Upload an OpenAPI spec
- `oas map --app <APP>` - Map/unmap specs to an app

---

### Secrets (1 endpoint)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/user` (secrets) | `secret list` | List user secrets |

**Secret fields**: `name`, `createdAt` (values are never returned)

---

### Profile Scans (4 endpoints) — NEW

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| 🔲 | GET | `/api/v1/app/{appId}/profile/results` | - | Get latest profile scan result |
| 🔲 | GET | `/api/v1/app/{appId}/profile/results/list` | - | List profile scan results |
| 🔲 | GET | `/api/v1/app/{appId}/profile/results/{scanId}` | - | Get specific profile scan result |
| 🔲 | POST | `/api/v1/org/{orgId}/profile/results` | - | Bulk get latest results for multiple apps |

Profile scans perform **testability analysis** on applications. Results include:
- **Classification**: `APP_SPA_WITH_AUTH`, `APP_API_WITH_AUTH`, `APP_PUBLIC_WEBSITE`, `APP_STATIC_SITE`, `APP_CLASSIFICATION_UNKNOWN`
- **Testability score**: `TESTABILITY_HIGH`, `TESTABILITY_MEDIUM`, `TESTABILITY_LOW` with score reasons
- **Auth markers**: Detected authentication signals with confidence and evidence
- **Path discovery**: Static, dynamic, and auth-protected path counts with samples
- **Asset inventory**: Scripts, static media, dynamic content with sizes
- **Recommendations**: Actionable strings for improving scan coverage
- **Discovered OpenAPI spec path**: Auto-detected spec location (e.g., `/api/v3/api-docs`)

**Potential commands**:
- `profile get --app <APP>` - View latest profile scan result
- `profile list --app <APP>` - List profile scan history
- `profile get --app <APP> --scan <ID>` - View specific result

---

### Global Configuration (1 endpoint) — NEW

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| 🔲 | GET | `/api/v1/global-configuration/{configName}` | - | Get global HawkScan config (redirects to S3) |

Returns a redirect to S3 presigned URL for shared HawkScan configuration files. Requires `read:scan_config` permission.

---

### Alert Rules — REMOVED

~~`POST /api/v1/app/{appId}/alerts/rules/{integrationId}`~~ — Upsert alert rule endpoint removed from spec in 2026-03 update.

---

## API Quirks & Gotchas

### 1. String-Encoded Numbers
Some fields return numbers as strings:
```json
{"totalCount": "2666"}  // Not: {"totalCount": 2666}
```
HawkOp uses `deserialize_string_to_usize` helper to handle this.

### 2. Field Name Differences
API uses camelCase; use `#[serde(rename_all = "camelCase")]` or aliases:
```rust
#[serde(alias = "applicationId")]
pub app_id: String,
```

### 3. Pagination Tokens
- `pageToken` is opaque; don't parse it
- First request omits `pageToken`; subsequent requests include it from response

### 4. v1 vs v2 Endpoints
- v2 endpoints have better pagination and response structure
- Prefer v2 when available (currently: apps, envs)

### 5. Empty vs Null
- Empty arrays are `[]`, not null
- Missing optional fields are omitted, not null

### 6. Date Formats
- ISO 8601: `2026-01-17T03:37:43.580308+00:00`
- Some timestamps are Unix epoch milliseconds

---

## Implementation Checklist

When adding a new endpoint to HawkOp:

1. **Add model** in `src/client/models/<resource>.rs`
2. **Add trait method** in `src/client/mod.rs`
3. **Implement** in `src/client/api/listing.rs` (or new file)
4. **Add display model** in `src/models/display/<resource>.rs`
5. **Add CLI handler** in `src/cli/<command>.rs`
6. **Wire up** in `src/main.rs`
7. **Update this reference** - mark status as ✅

See `CLAUDE.md` → "Adding New Commands" for detailed patterns.

---

## Feature Ideas (Unimplemented Endpoints)

### High Value
- **`scan start/stop`** - Trigger scans from CLI (Perch endpoints)
- **`app create/delete`** - Manage applications
- **`config get/upload`** - Manage scan configurations
- **`findings`** - Organization-wide findings report
- **`profile get/list`** - Profile scan testability analysis (NEW in 2026-03)
- **`findings triage`** - Bulk triage findings by hash (NEW in 2026-03)
- **`scan list --tag`** - Filter scans by tag (NEW parameter in 2026-03)

### Medium Value
- **`policy assign`** - Assign policies to apps
- **`env create/delete`** - Environment management
- **`user teams`** - List teams for a specific user
- **`oas upload`** - Upload OpenAPI spec to an app (NEW in 2026-03)

### Lower Priority
- **`repo sensitive`** - View sensitive data in repos
- **`oas map`** - Map/unmap specs to apps
- **`global-config get`** - Fetch shared HawkScan configs (NEW in 2026-03)

### Removed from API (no longer available)
- ~~`alert rules`~~ - Alert rule upsert endpoint removed
- ~~`policy delete`~~ - Policy delete method removed
- ~~`app policy get`~~ - App policy get endpoint removed
