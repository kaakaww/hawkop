---
description: Add a new StackHawk API integration (with pattern matching + quirks check)
---

Help me add a new StackHawk API endpoint integration.

## Step 1: Research the Endpoint

Before writing code, gather information:

1. **Read the API reference** — `.claude/skills/stackhawk-api-sherpa/stackhawk-api.md`
2. **Check the OpenAPI spec** — `stackhawk-openapi.json` for exact request/response schemas
3. **Check API quirks** — `.claude/skills/stackhawk-api-sherpa/api-quirks.md` for known issues
4. **Check the roadmap** — `docs/ROADMAP.md` for planned status

Ask the user:
1. What API endpoint should be integrated?
2. What HTTP method (GET, POST, PUT, DELETE)?
3. Should this support pagination?
4. Is this a read-only query or a state mutation?

## Step 2: Study Existing Patterns

Read the most similar existing integration to match patterns:

**For GET/list endpoints** — study:
- `src/client/api/listing.rs` for pagination and query params
- `src/client/pagination.rs` for `PaginationParams` and `PagedResponse`

**For POST/PUT mutations** — study:
- `src/client/stackhawk.rs` for `request_json_with_query()` and `request_with_body()`
- The team update pattern (fetch-modify-put) if it's a PUT endpoint

**For detail/drill-down** — study:
- `src/client/api/scan_detail.rs` for nested resource fetching

## Step 3: Implementation

Follow this exact pattern:

### 3.1 API Model (`src/client/models/<resource>.rs`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewResource {
    // Use #[serde(default)] for fields that may be missing
    // Use #[serde(alias = "...")] for API field name differences
    // Use Option<T> for truly optional fields
}
```

### 3.2 Response Wrapper (in implementation file)
```rust
#[derive(Deserialize)]
struct ListResponse {
    items: Vec<NewResource>,
    #[serde(default, deserialize_with = "deserialize_string_to_usize")]
    total_count: usize,  // Some endpoints return this as a string!
}
```

### 3.3 Trait Method (`src/client/mod.rs`)
```rust
async fn list_resource(&self, org_id: &str, pagination: Option<&PaginationParams>) -> Result<Vec<Resource>>;
```

### 3.4 Implementation
- Use `self.request_with_query()` for GET with query params
- Use `self.request_json_with_query()` for POST with body
- Apply rate limiting category (check `src/client/rate_limit.rs`)
- Add to `CachedStackHawkClient` in `src/cache/client.rs` with appropriate TTL

### 3.5 Mock Implementation (`src/client/mock.rs`)
Add the method to `MockStackHawkClient` returning fixture data.

## Step 4: Verify

```bash
cargo check     # Compiles?
cargo test      # Tests pass?
cargo clippy -- -D warnings  # Lint clean?
```

## Common Pitfalls

From `api-quirks.md`:
- **PUT = REPLACE-ALL** — Always fetch current state before PUT operations
- **readOnly fields required** — Don't trust OpenAPI `readOnly` annotations
- **String numbers** — Use `deserialize_string_to_usize` for `totalCount` fields
- **Removed endpoints** — Check if the endpoint was removed from recent spec updates
- **Rate limits** — Categorize correctly: Scan (80/sec), User (80/sec), AppList (80/sec), Default (6/sec)
