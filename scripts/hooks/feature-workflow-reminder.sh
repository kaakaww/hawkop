#!/usr/bin/env bash
# UserPromptSubmit hook: Detects feature implementation requests and reminds
# Claude to use the /new-feature or /add-command workflow instead of jumping
# straight to coding.
#
# This closes the gap where the user describes a feature without explicitly
# invoking a slash command, and Claude skips the structured workflow.

set -euo pipefail

input=$(cat)
prompt=$(echo "$input" | jq -r '.user_prompt // empty' 2>/dev/null)

# If no prompt text available, exit silently
[ -z "$prompt" ] && exit 0

# Normalize to lowercase for matching
lower=$(echo "$prompt" | tr '[:upper:]' '[:lower:]')

# Don't fire if the user is already invoking a slash command
if echo "$lower" | grep -qE '^\s*/'; then
  exit 0
fi

# Detect feature implementation signals
# Match phrases like "add a command", "implement", "new endpoint", "build the X feature", etc.
if echo "$lower" | grep -qE '(add|implement|create|build|write|wire up).*(command|endpoint|feature|subcommand|cli|api integration)|(new|add).*(hawkop|cli) (command|feature)'; then
  cat <<'EOF'
{"hookSpecificOutput":{"hookEventName":"UserPromptSubmit","additionalContext":"Feature implementation detected. Before writing code, follow the structured workflow:\n\n1. For a full feature (API + CLI + tests + docs): use /new-feature\n2. For just a CLI command: use /add-command\n3. For just an API integration: use /add-api\n\nThese workflows enforce: test-first contract, existing pattern verification, API quirks check, CLI design review, and documentation updates.\n\nIf the user hasn't invoked a slash command, suggest the appropriate one before proceeding."}}
EOF
fi
