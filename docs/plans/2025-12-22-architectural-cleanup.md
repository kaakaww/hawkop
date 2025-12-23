# HawkOp Architectural Cleanup Plan

**Date**: 2025-12-22
**Status**: Approved
**Goal**: Address technical debt before scaling to 10+ commands

## Summary

6-phase refactoring to eliminate boilerplate, complete unused abstractions, and establish patterns for scaling. Each phase is incremental and leaves codebase in working state.

---

## Phase 1: CommandContext Helper (Critical)

**Problem**: 10-15 lines of config/client initialization duplicated 5+ times
**Solution**: Single `CommandContext` struct

### Files to Create/Modify
- `src/cli/context.rs` (new) - CommandContext struct with `new()` async constructor
- `src/cli/mod.rs` - Export context module
- `src/cli/org.rs` - Replace boilerplate with `CommandContext::new()`
- `src/cli/app.rs` - Replace boilerplate with `CommandContext::new()`

### Pattern
```rust
// Before: 15 lines per command
let config = Config::load_at(config_path)?;
config.validate_auth()?;
let client = StackHawkClient::new(config.api_key.clone())?;
if let Some(jwt) = config.jwt { client.set_jwt(...).await; }

// After: 1 line
let ctx = CommandContext::new(format, org_override, config_path).await?;
```

---

## Phase 2: Display Models Module (Critical)

**Problem**: OrgDisplay, AppDisplay nearly identical, defined separately
**Solution**: Shared `models` module with `DisplayModel` trait

### Files to Create/Modify
- `src/models/mod.rs` (new)
- `src/models/display.rs` (new) - DisplayModel trait, OrgDisplay, AppDisplay
- `src/main.rs` - Add `mod models`
- `src/cli/org.rs` - Remove local OrgDisplay, import from models
- `src/cli/app.rs` - Remove local AppDisplay, import from models

---

## Phase 3: Complete Formattable Trait (Critical)

**Problem**: Formattable trait in output/mod.rs exists but unused
**Solution**: Implement trait properly, use in all commands

### Files to Modify
- `src/output/mod.rs` - Complete Formattable with blanket impl for Vec<T: Tabled + Serialize>
- `src/cli/org.rs` - Use `display_orgs.print(format)?`
- `src/cli/app.rs` - Use `display_apps.print(format)?`

### Pattern
```rust
// Before: 10 lines match statement per command
match format {
    OutputFormat::Table => { ... }
    OutputFormat::Json => { ... }
}

// After: 1 line
display_items.print(format)?;
```

---

## Phase 4: Unit Test Infrastructure (High)

**Problem**: Only 4 integration tests, no mocks, slow feedback
**Solution**: Mock client and test helpers

### Files to Create
- `src/client/mock.rs` - MockStackHawkClient implementing StackHawkApi trait
- `src/config/test_helpers.rs` - test_config(), write_test_config()

### Files to Modify
- `src/client/mod.rs` - Add `#[cfg(test)] pub mod mock`
- `src/models/display.rs` - Add unit tests for DisplayModel

---

## Phase 5: Error Context and Debug Logging (Medium)

**Problem**: --debug flag does nothing useful
**Solution**: Debug logging throughout request/response cycle

### Files to Modify
- `src/error.rs` - Add ErrorContext trait for chained errors
- `src/main.rs` - Debug output for config, org, format; error details on failure
- `src/client/stackhawk.rs` - Optional debug logging in requests

---

## Phase 6: Pagination Abstraction (High)

**Problem**: No pagination helpers for future scan/finding/report commands
**Solution**: PaginationParams and RequestBuilder

### Files to Create
- `src/client/pagination.rs` - PaginationParams, PaginatedResponse<T>
- `src/client/request.rs` - RequestBuilder with fluent API

### Files to Modify
- `src/client/mod.rs` - Export new modules

---

## Dependency Order

```
Phase 1 ──┬──> Phase 4
          └──> Phase 5

Phase 2 ────> Phase 3
          └──> Phase 4

Phases 1-4 ──> Phase 6
```

## Verification Per Phase

1. `cargo build` succeeds
2. `cargo test` passes
3. `cargo clippy` clean
4. Manual test: existing commands work identically

## Success Metrics

- Config/client boilerplate: 15 lines → 2 lines per command
- Display models: Single shared definition
- Adding new command: ~30-40 lines instead of ~80+
- All 4 existing tests pass throughout
