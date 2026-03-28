---
description: Add a new CLI command to HawkOp (with design review + test-first workflow)
---

Help me add a new command to the HawkOp CLI.

## Phase 1: Design (before writing any implementation code)

### 1.1 Gather Requirements

Answer these questions with the user:
1. What should the command do?
2. What command name and subcommands?
3. What flags and arguments? (prefer flags over multiple positional args)
4. What API endpoint(s) does it call? (check `stackhawk-api.md` and `api-quirks.md`)

### 1.2 Invoke CLI Designer

**Invoke the `cli-designer` skill** to design the interface. Verify:
- Command follows clig.dev and 12-Factor CLI principles
- Args vs flags decision is correct
- Help text includes examples in `after_help`
- Matches existing HawkOp patterns (`--org`, `--format`, `--limit`, etc.)

### 1.3 Check Existing Patterns

Before implementing, study the closest existing command:
- Read `src/cli/` files for a similar command (e.g., `scan.rs` for list+detail, `team.rs` for CRUD)
- Read `src/client/api/` for the API integration pattern
- Read `src/models/display/` for the display model pattern
- Check `src/client/mod.rs` for trait method conventions

### 1.4 Check API Quirks

Read `.claude/skills/stackhawk-api-sherpa/api-quirks.md` to check if the endpoint has known quirks (field naming, replace-all PUT behavior, removed endpoints, etc.).

## Phase 2: Test-First Contract

### 2.1 Write Test Stubs First

Before implementing the command, create test stubs that define the expected behavior:

**Unit test** in the relevant source file:
```rust
#[cfg(test)]
mod tests {
    // Test that the command handler produces expected output
    // Test error cases (not found, invalid args)
    // Test JSON format output
}
```

**Functional test** in `tests/functional/`:
- Add to the appropriate test file (read_tests.rs, mutation_tests.rs, etc.)
- At minimum, write stubs for:
  - Happy-path test (command runs successfully)
  - Error-path test (invalid input, not-found)
  - JSON output test (`--format json` produces valid JSON)

### 2.2 Update Coverage Script

Add the new command to `scripts/check-test-coverage.sh` so it's tracked:
```bash
check_coverage "hawkop <cmd> <subcmd>" "test_<cmd>_<subcmd>|\"<cmd>\".*\"<subcmd>\""
```

## Phase 3: Implementation

Follow this order (matches CLAUDE.md "Adding New Commands"):

1. **API models** — `src/client/models/<resource>.rs`
   - `#[serde(rename_all = "camelCase")]`
   - Use `#[serde(default)]` for optional fields
   - Check `stackhawk-openapi.json` for exact field names
2. **Trait method** — `src/client/mod.rs`
3. **API implementation** — `src/client/api/` (or `src/client/stackhawk.rs`)
4. **Mock implementation** — `src/client/mock.rs`
5. **Display model** — `src/models/display/<resource>.rs`
6. **CLI command enum** — `src/cli/mod.rs`
7. **Handler** — `src/cli/<command>.rs`
8. **Main routing** — `src/main.rs`

## Phase 4: Verification

### 4.1 Run Tests
```bash
cargo test                        # All unit/integration tests
cargo clippy -- -D warnings       # Lint check
```

### 4.2 Run CLI Design Checklist

Verify against `docs/CLI_DESIGN_PRINCIPLES.md`:
- [ ] `-h`/`--help` display useful help with examples
- [ ] `--format json` produces valid JSON parseable by `jq`
- [ ] Data on stdout, messages on stderr
- [ ] Non-zero exit on failure
- [ ] Navigation hints (`-> hawkop next-command`)
- [ ] Works when piped (`hawkop cmd | jq`)
- [ ] No ANSI codes when not TTY

### 4.3 Run Functional Tests
```bash
HAWKOP_PROFILE=test HAWKOP_FUNCTIONAL_TESTS_CONFIRM=yes \
  cargo test --features functional-tests --test functional -- --test-threads=1 --nocapture "<test_name>"
```

### 4.4 Check Test Coverage
```bash
./scripts/check-test-coverage.sh
```

## Phase 5: Documentation

- [ ] Update `docs/CLI_REFERENCE.md` with the new command
- [ ] Update `docs/ROADMAP.md` if this completes a planned endpoint
- [ ] Add entry to `CHANGELOG.md` under `[Unreleased]`
- [ ] Update `CLAUDE.md` command list if a new top-level command
