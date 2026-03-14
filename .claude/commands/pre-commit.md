---
description: Run comprehensive pre-commit checklist (quality + docs + design review)
---

Run the full HawkOp pre-commit checklist. This goes beyond `make pre-commit` by also
checking documentation currency and CLI design compliance.

## Step 1: Code Quality Gate (fast, parallel where possible)

Run these checks and report results:

1. `cargo fmt -- --check` — verify formatting
2. `cargo clippy -- -D warnings` — lint for bugs and anti-patterns
3. `cargo test` — run all unit and integration tests

If any fail, stop here and report the failures with suggested fixes.

## Step 2: Test Coverage Check

Run `./scripts/check-test-coverage.sh` to verify that all implemented CLI commands
have corresponding functional tests. Report any gaps.

If new commands were added in this branch, verify they have:
- At least one happy-path functional test
- At least one error-path test (invalid args, not-found, etc.)
- JSON format test (`--format json` output check)

## Step 3: Documentation Currency

Check if any of these docs need updating based on the changes in this branch.
Use `git diff main...HEAD -- src/cli/` to see what CLI code changed.

For each doc, state whether it needs updating and why:

1. **docs/CLI_REFERENCE.md** — Does it reflect any new/changed commands, flags, or arguments?
2. **docs/ROADMAP.md** — Should any planned items be marked as complete or in-progress?
3. **CHANGELOG.md** — Is there an entry in `[Unreleased]` for the changes in this branch?
4. **CONTRIBUTING.md** — Do any workflow changes need documenting?

Do NOT auto-update docs. Instead, list what needs attention so the developer can review.

## Step 4: CLI Design Principles Review

Review any new or changed CLI commands against the design principles in
`docs/CLI_DESIGN_PRINCIPLES.md`. Check for:

- stdout for data, stderr for messages/progress
- `--format json` support for all data-producing commands
- Actionable error messages (what went wrong + how to fix)
- Confirmation required for destructive operations (`--yes` to skip)
- Suggested next commands in output
- No required interactive prompts (flag overrides for everything)

## Step 5: Summary

Output a summary table:

```
Pre-commit Checklist
====================
Format check:     PASS/FAIL
Clippy:           PASS/FAIL
Tests:            PASS/FAIL
Test coverage:    PASS/FAIL (N gaps)
Docs currency:    OK / NEEDS UPDATE (list which)
Design review:    OK / ISSUES (list which)

Recommendation:   READY TO COMMIT / NEEDS WORK
```
