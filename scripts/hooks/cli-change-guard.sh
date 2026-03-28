#!/usr/bin/env bash
# PostToolUse hook: Injects CLI design reminders when src/cli/ files are modified.
#
# Reads tool input from stdin (JSON), checks if the modified file is in src/cli/,
# and if so, injects additionalContext back into the model's context window.
#
# This ensures Claude always considers CLI design principles, test requirements,
# and documentation updates when working on CLI handler code.

set -euo pipefail

input=$(cat)
file_path=$(echo "$input" | jq -r '.tool_input.file_path // .tool_response.filePath // empty')

# Only fire for Rust files in src/cli/
if echo "$file_path" | grep -qE 'src/cli/.*\.rs$'; then
  cat <<'EOF'
{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"CLI file modified. Before proceeding, verify:\n1. Help text includes examples (after_help)\n2. --format json produces valid, parseable JSON\n3. stdout for data, stderr for messages/progress\n4. Non-zero exit code on failure\n5. Navigation hints on stderr (e.g. -> hawkop next-command)\n6. Works when piped (no ANSI codes, no interactive prompts)\n7. Functional tests needed in tests/functional/\n8. Update docs/CLI_REFERENCE.md if commands, flags, or args changed\nRefer to docs/CLI_DESIGN_PRINCIPLES.md and the cli-designer skill checklist."}}
EOF
fi
