#!/usr/bin/env bash
# Stop hook: Reminds about pre-commit checklist when CLI code has been modified.
#
# Checks both staged and unstaged changes for src/cli/ modifications.
# Outputs a systemMessage so the user sees the reminder before the session ends.

set -euo pipefail

# Check for any uncommitted CLI changes (staged or unstaged)
cli_changed=false
if git diff --name-only HEAD 2>/dev/null | grep -qE 'src/cli/.*\.rs$'; then
  cli_changed=true
elif git diff --name-only 2>/dev/null | grep -qE 'src/cli/.*\.rs$'; then
  cli_changed=true
elif git diff --cached --name-only 2>/dev/null | grep -qE 'src/cli/.*\.rs$'; then
  cli_changed=true
fi

if [ "$cli_changed" = true ]; then
  echo '{"systemMessage":"CLI code was modified. Before committing, consider running /pre-commit to verify: test coverage, CLI design compliance, and documentation currency."}'
fi
