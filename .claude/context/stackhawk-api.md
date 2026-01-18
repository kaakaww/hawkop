# StackHawk API Reference for HawkOp

> **Last Updated**: 2026-01-17
> **OpenAPI Spec Version**: 3.0.1
> **HawkOp Version**: 0.4.0
> **Total Endpoints**: 51 | **Implemented**: 14 (~27%)

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
│   │   └── Alert Stats (H/M/L counts)
│   ├── Scan Policies (which checks to run)
│   └── Alert Rules (notifications)
├── Teams
│   ├── Members (users)
│   └── Applications (team ownership)
├── Repositories (attack surface)
│   ├── Sensitive Data Tags
│   └── Application Mappings
├── Hosted OAS Files (OpenAPI specs)
├── Scan Configurations (YAML files)
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
| 🔲 | GET | `/api/v1/app/{appId}/env/list` | - | List environments |
| 🔲 | GET | `/api/v2/org/{orgId}/envs` | - | List environments (v2) |

**App fields**: `applicationId`, `name`, `cloudScanTargets[]`, `env`, `riskGrade`

**Potential commands**:
- `app get <ID>` - View single app details
- `app create` - Create new application
- `env list --app <ID>` - List app environments

---

### Environments (4 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| 🔲 | POST | `/api/v1/app/{appId}/env` | - | Create environment |
| 🔲 | POST | `/api/v1/app/{appId}/env/{envId}` | - | Update environment |
| 🔲 | DELETE | `/api/v1/app/{appId}/env/{envId}` | - | Delete environment |
| 🔲 | GET | `/api/v1/app/{appId}/env/{envId}/config/default` | - | Get default YAML config |

---

### Scans & Results (6 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/scan/{orgId}` | `scan list` | List scan results |
| ✅ | GET | `/api/v1/scan/{scanId}/alerts` | `scan get` | List scan alerts (findings) |
| ✅ | GET | `/api/v1/scan/{scanId}/alert/{pluginId}` | `scan get --plugin` | Alert details + paths |
| ✅ | GET | `/api/v1/scan/{scanId}/uri/{alertUriId}/messages/{messageId}` | `scan get --message` | HTTP request/response |
| 🔲 | DELETE | `/api/v1/scan/{scanId}` | - | Delete a scan |
| 🔲 | GET | `/api/v1/reports/org/{orgId}/findings` | - | Organization findings report |

**Scan fields**: `scanId`, `applicationId`, `applicationName`, `scanStatus`, `alertStats`, `startedTimestamp`, `completedTimestamp`
**Alert fields**: `pluginId`, `pluginName`, `severity`, `count`, `paths[]`

**Implemented features**:
- Drill-down: scan → alerts → URIs → messages
- Filter by severity, plugin, path
- View full HTTP request/response with `--curl` flag

---

### Scan Control - Perch (3 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| 🔲 | POST | `/api/v1/app/{appId}/perch/start` | - | Start hosted scan |
| 🔲 | GET | `/api/v1/app/{appId}/perch/status` | - | Get scan status |
| 🔲 | POST | `/api/v1/app/{appId}/perch/stop` | - | Stop hosted scan |

**Potential commands**:
- `scan start --app <ID>` - Trigger hosted scan
- `scan status --app <ID>` - Check running scan
- `scan stop --app <ID>` - Cancel scan

---

### Scan Policies (9 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/policy/all` | `policy list` | All StackHawk policies |
| ✅ | GET | `/api/v1/policy/{orgId}/list` | `policy list --org` | Org custom policies |
| 🔲 | GET | `/api/v1/policy` | - | Get single SH policy |
| 🔲 | GET | `/api/v1/policy/{orgId}/{policyName}` | - | Get org policy |
| 🔲 | POST | `/api/v1/policy/{orgId}/update` | - | Update org policy |
| 🔲 | DELETE | `/api/v1/policy/{orgId}/{policyName}` | - | Delete org policy |
| 🔲 | GET | `/api/v1/app/{appId}/policy` | - | Get app's assigned policy |
| 🔲 | PUT | `/api/v1/app/{appId}/policy/assign` | - | Assign policy to app |
| 🔲 | GET | `/api/v1/app/{appId}/policy/plugins/{pluginId}/{toggle}` | - | Toggle plugin |

**Policy fields**: `name`, `type` (Stackhawk/Organization), `plugins[]`, `enabled`

---

### Scan Configurations (6 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/configuration/{orgId}/list` | `config list` | List YAML configs |
| 🔲 | GET | `/api/v1/configuration/{orgId}/{configName}` | - | Get config content |
| 🔲 | POST | `/api/v1/configuration/{orgId}/update` | - | Create/update config |
| 🔲 | POST | `/api/v1/configuration/{orgId}/rename` | - | Rename config |
| 🔲 | POST | `/api/v1/configuration/{orgId}/validate` | - | Validate YAML |
| 🔲 | DELETE | `/api/v1/configuration/{orgId}/{configName}` | - | Delete config |

**Potential commands**:
- `config get <NAME>` - View config YAML
- `config validate <FILE>` - Validate local YAML
- `config upload <FILE>` - Upload config

---

### Teams (7 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/org/{orgId}/teams` | `team list` | List all teams |
| 🔲 | GET | `/api/v1/org/{orgId}/team/{teamId}` | - | Get team details |
| 🔲 | POST | `/api/v1/org/{orgId}/team` | - | Create team |
| 🔲 | PUT | `/api/v1/org/{orgId}/team/{teamId}` | - | Update team |
| 🔲 | DELETE | `/api/v1/org/{orgId}/team/{teamId}` | - | Delete team |
| 🔲 | PUT | `/api/v1/org/{orgId}/team/{teamId}/application` | - | Assign app to team |
| 🔲 | GET | `/api/v1/org/{orgId}/user/{userId}/teams` | - | User's teams |

**Team fields**: `teamId`, `name`, `applications[]`, `users[]`

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

### Hosted OAS (4 endpoints)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/oas/{orgId}/list` | `oas list` | List OpenAPI specs |
| 🔲 | GET | `/api/v1/oas/{orgId}/{oasId}` | - | Get spec content |
| 🔲 | GET | `/api/v1/oas/{appId}/mapping` | - | App's mapped specs |
| 🔲 | POST | `/api/v1/oas/{appId}/mapping` | - | Map spec to app |

**OAS fields**: `oasId`, `fileName`, `fileSize`, `uploadedAt`, `applications[]`

---

### Secrets (1 endpoint)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| ✅ | GET | `/api/v1/user` (secrets) | `secret list` | List user secrets |

**Secret fields**: `name`, `createdAt` (values are never returned)

---

### Alert Rules (1 endpoint)

| Status | Method | Endpoint | HawkOp | Description |
|--------|--------|----------|--------|-------------|
| 🔲 | POST | `/api/v1/app/{appId}/alerts/rules/{integrationId}` | - | Upsert alert rule |

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

### Medium Value
- **`team create/manage`** - Team administration
- **`policy assign`** - Assign policies to apps
- **`env create/delete`** - Environment management

### Lower Priority
- **`repo sensitive`** - View sensitive data in repos
- **`oas get/map`** - Manage OpenAPI specs
- **`alert rules`** - Configure notifications
