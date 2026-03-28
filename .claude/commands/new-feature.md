---
description: "End-to-end feature workflow: API research -> CLI design -> test-first implementation -> docs"
arguments:
  - name: feature
    description: "Brief description of the feature to implement"
    required: true
---

Orchestrate a complete feature implementation for HawkOp. This command sequences
all the right skills and checks so nothing falls through the cracks.

**Feature requested**: $ARGUMENTS

---

## Stage 1: Research & Design

### 1.1 API Research

Invoke the **stackhawk-api-sherpa** skill to research the API endpoints needed:
1. Read `.claude/skills/stackhawk-api-sherpa/stackhawk-api.md` for endpoint details
2. Check `stackhawk-openapi.json` for request/response schemas
3. Read `.claude/skills/stackhawk-api-sherpa/api-quirks.md` for known API gotchas
4. Check `docs/ROADMAP.md` to see if this feature is planned and what phase it's in

Present findings to the user:
- Which endpoint(s) are needed
- Any API quirks or limitations
- Whether the endpoint is stable or under active development
- Recommended approach

**Wait for user approval before proceeding.**

### 1.2 CLI Design

Invoke the **cli-designer** skill to design the command interface:
1. Command name, subcommands, flags, arguments
2. Apply clig.dev and 12-Factor CLI principles
3. Match existing HawkOp conventions
4. Write help text with examples

Present the proposed CLI interface to the user.

**Wait for user approval before proceeding.**

## Stage 2: Test-First Contract

### 2.1 Study Existing Patterns

Before writing any code, study the closest existing implementation:
- Read the most similar command in `src/cli/` for handler patterns
- Read the corresponding API method in `src/client/api/`
- Read the display model in `src/models/display/`
- Note any reusable utilities or shared patterns

### 2.2 Write Test Stubs

Create test stubs BEFORE implementation:

1. **Unit tests** — Define expected behavior:
   - Mock client returns expected data
   - Error cases produce correct error types
   - Display model converts correctly

2. **Functional tests** — In `tests/functional/`:
   - Happy-path test (command succeeds with real API)
   - Error-path test (invalid input, not-found)
   - JSON output test (`--format json` produces valid, parseable JSON)

3. **Update coverage script** — Add to `scripts/check-test-coverage.sh`

### 2.3 Verify Stubs Compile

```bash
cargo test --no-run
```

Tests should compile but may not pass yet (that's expected at this stage).

## Stage 3: Implementation

Follow the implementation order from `/add-command`:

1. API models (`src/client/models/`)
2. Trait method (`src/client/mod.rs`)
3. API implementation (`src/client/api/` or `src/client/stackhawk.rs`)
4. Mock implementation (`src/client/mock.rs`)
5. Display model (`src/models/display/`)
6. CLI command enum (`src/cli/mod.rs`)
7. Handler (`src/cli/<command>.rs`)
8. Main routing (`src/main.rs`)

After each file, run `cargo check` to catch errors early.

## Stage 4: Verify

### 4.1 Quality Gate

Run the full quality suite:
```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

### 4.2 CLI Design Compliance

Run through the CLI design checklist (`.claude/skills/cli-designer/checklist.md`):
- Help text with examples
- JSON output valid
- stdout/stderr separation
- Exit codes correct
- Navigation hints present
- Non-TTY compatible

### 4.3 Functional Tests

```bash
HAWKOP_PROFILE=test HAWKOP_FUNCTIONAL_TESTS_CONFIRM=yes \
  cargo test --features functional-tests --test functional -- --test-threads=1 --nocapture "<test_pattern>"
```

### 4.4 Test Coverage

```bash
./scripts/check-test-coverage.sh
```

## Stage 5: Documentation

Update all relevant docs:
- [ ] `docs/CLI_REFERENCE.md` — New command, flags, examples
- [ ] `docs/ROADMAP.md` — Mark endpoint as complete, update coverage count
- [ ] `CHANGELOG.md` — Entry under `[Unreleased]`
- [ ] `CLAUDE.md` — Add to command list if new top-level command

## Stage 6: Pre-Commit

Run `/pre-commit` for the final check before the user commits.

Present a summary:
```
Feature Implementation Summary
==============================
Command:           hawkop <cmd> <subcmd>
API endpoint(s):   <method> <path>
Files created:     N new files
Files modified:    N existing files
Tests:             N unit, N functional
Docs updated:      CLI_REFERENCE, ROADMAP, CHANGELOG
Quality:           fmt OK, clippy OK, tests OK
Coverage:          N/N commands tested
```
