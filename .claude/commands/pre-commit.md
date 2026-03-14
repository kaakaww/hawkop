---
description: "Run pre-commit checklist. Use '/pre-commit full' to include functional tests."
arguments:
  - name: scope
    description: "'full' to include functional tests against real API, omit for fast checks only"
    required: false
---

Run the HawkOp pre-commit checklist. This goes beyond `make pre-commit` by also
checking documentation currency and CLI design compliance.

**Scope**: If the argument is "full", also run functional tests against the real API
(requires HAWKOP_PROFILE to be set). Otherwise, run fast checks only.

## Step 1: Code Quality Gate (fast, parallel where possible)

Run these checks and report results:

1. `cargo fmt -- --check` — verify formatting
2. `cargo clippy -- -D warnings` — lint for bugs and anti-patterns
3. `cargo test` — run all unit and integration tests

If any fail, stop here and report the failures with suggested fixes.

## Step 2: Functional Tests (only if scope is "full")

If the argument is "full":

1. Check if `HAWKOP_PROFILE` is set. If not, warn and skip this step.
2. Run: `HAWKOP_FUNCTIONAL_TESTS_CONFIRM=yes cargo test --features functional-tests --test functional -- --test-threads=1 --nocapture`
3. Report the results. Watch for any `⚠️ SKIPPED` warnings indicating feature flags not enabled.
4. If any tests fail, report them with the stderr output.

If the argument is NOT "full", skip this step and note:
"Functional tests skipped. Run `/pre-commit full` to include them."

## Step 3: Test Coverage Check

Run `./scripts/check-test-coverage.sh` to verify that all implemented CLI commands
have corresponding functional tests. Report any gaps.

If new commands were added in this branch, verify they have:
- At least one happy-path functional test
- At least one error-path test (invalid args, not-found, etc.)
- JSON format test (`--format json` output check)

If there ARE gaps, suggest specific test functions to add, following the patterns
in the existing test files (read_tests.rs, mutation_tests.rs, hosted_tests.rs, local_tests.rs).

## Step 4: Documentation Currency

Check if any of these docs need updating based on the changes in this branch.
Use `git diff main...HEAD -- src/cli/` to see what CLI code changed.

For each doc, state whether it needs updating and why:

1. **docs/CLI_REFERENCE.md** — Does it reflect any new/changed commands, flags, or arguments?
2. **docs/ROADMAP.md** — Should any planned items be marked as complete or in-progress?
3. **CHANGELOG.md** — Is there an entry in `[Unreleased]` for the changes in this branch?
4. **CONTRIBUTING.md** — Do any workflow changes need documenting?

Do NOT auto-update docs. Instead, list what needs attention so the developer can review.

## Step 5: CLI Design Principles Review

Review any new or changed CLI commands against the design principles in
`docs/CLI_DESIGN_PRINCIPLES.md`. Check for:

- stdout for data, stderr for messages/progress
- `--format json` support for all data-producing commands
- Actionable error messages (what went wrong + how to fix)
- Confirmation required for destructive operations (`--yes` to skip)
- Suggested next commands in output
- No required interactive prompts (flag overrides for everything)

## Step 6: Summary

Output a summary table:

```
Pre-commit Checklist
====================
Format check:       PASS/FAIL
Clippy:             PASS/FAIL
Unit tests:         PASS/FAIL
Functional tests:   PASS/FAIL/SKIPPED (N passed, N failed, N skipped)
Test coverage:      PASS/FAIL (N/N commands, N gaps)
Docs currency:      OK / NEEDS UPDATE (list which)
Design review:      OK / ISSUES (list which)

Recommendation:     READY TO COMMIT / NEEDS WORK
```
