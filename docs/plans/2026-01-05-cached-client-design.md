# CachedStackHawkClient Design

## Summary

Add a `CachedStackHawkClient` wrapper that caches all API responses to reduce latency and API calls across the entire CLI.

| Decision | Choice |
|----------|--------|
| Pattern | Decorator wrapping `StackHawkApi` trait |
| Pagination | Cache individual pages (key includes pagination params) |
| Bypass | `--no-cache` global flag |
| Error handling | Silent fallback on cache errors |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    CommandContext                        │
│  ┌─────────────────────────────────────────────────┐    │
│  │         CachedStackHawkClient<C>                │    │
│  │  ┌─────────────────┐  ┌──────────────────┐     │    │
│  │  │  StackHawkClient │  │  CacheStorage    │     │    │
│  │  │  (inner)         │  │  (SQLite+blobs)  │     │    │
│  │  └─────────────────┘  └──────────────────┘     │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**Flow for each API call:**
1. Generate cache key from endpoint + org_id + pagination params
2. Check cache → if hit and valid, deserialize and return
3. If miss → call `inner.method()`
4. Serialize response and store in cache with appropriate TTL
5. Return response

## Files

| File | Action |
|------|--------|
| `src/cache/client.rs` | New: `CachedStackHawkClient` implementation |
| `src/cache/mod.rs` | Export new client |
| `src/cli/mod.rs` | Add `--no-cache` global flag |
| `src/cli/context.rs` | Wrap client with `CachedStackHawkClient` |

## Implementation

### CachedStackHawkClient Structure

```rust
pub struct CachedStackHawkClient<C: StackHawkApi> {
    inner: Arc<C>,
    cache: Option<CacheStorage>,  // None when --no-cache
}

impl<C: StackHawkApi> CachedStackHawkClient<C> {
    pub fn new(inner: C, enabled: bool) -> Self {
        let cache = if enabled {
            CacheStorage::open().ok()
        } else {
            None
        };
        Self { inner: Arc::new(inner), cache }
    }
}
```

### Method Pattern

```rust
async fn list_apps(&self, org_id: &str, pagination: Option<&PaginationParams>)
    -> Result<Vec<Application>>
{
    let key = cache_key("list_apps", Some(org_id), &pagination_to_params(pagination));

    // Cache errors don't fail the request
    if let Some(cached) = self.get_cached(&key) {
        return Ok(cached);
    }

    // API errors propagate normally
    let result = self.inner.list_apps(org_id, pagination).await?;

    // Cache write errors are silent
    self.set_cached(&key, &result, "list_apps", Some(org_id), CacheTtl::APPS);

    Ok(result)
}
```

### CLI Integration

```rust
#[derive(Parser, Debug)]
pub struct Cli {
    // ... existing fields ...

    #[arg(long, global = true, help = "Bypass cache, fetch fresh data")]
    pub no_cache: bool,
}
```

## TTL Mapping

| Method | TTL | Rationale |
|--------|-----|-----------|
| `authenticate` | **NEVER** | Security-sensitive |
| `list_orgs` | 1 hr | Orgs rarely change |
| `list_apps` / `list_apps_paged` | 1 hr | Apps rarely change |
| `list_scans` / `list_scans_paged` | 2 min | New scans appear frequently |
| `get_scan` | 24 hr OR 30 sec | Completed=immutable, Running=changes |
| `list_users` | 1 hr | Team membership stable |
| `list_teams` | 1 hr | Teams rarely change |
| `list_stackhawk_policies` | 1 hr | Read-only presets |
| `list_org_policies` | 1 hr | Policies rarely change |
| `list_repos` | 1 hr | Repos rarely change |
| `list_oas` | 1 hr | OAS specs rarely change |
| `list_scan_configs` | 1 hr | Configs rarely change |
| `list_secrets` | 1 hr | Secret names rarely change |
| `list_audit` | 5 min | Audit logs grow frequently |
| `list_scan_alerts` | 10 min | Triage can change |
| `get_alert_with_paths` | 10 min | Triage can change |
| `get_alert_message` | 24 hr | Alert messages are immutable |

## Error Handling

- Cache read errors → silently fall back to API
- Cache write errors → silently ignore (log at debug level)
- API errors → propagate normally (never cached)
- Never cache error responses

## Testing

Unit tests verify:
- Cache hit returns cached data without calling inner client
- Cache miss calls inner client and caches result
- `authenticate()` is never cached
- `--no-cache` mode bypasses cache entirely
- API errors are not cached
- Cache errors fall back to API gracefully
