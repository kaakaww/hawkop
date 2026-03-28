#!/usr/bin/env bash
# SessionStart hook: Checks project freshness at the start of each session.
#
# Currently checks:
# - stackhawk-openapi.json age (warns if >30 days old)
#
# Outputs a systemMessage so the user sees any warnings immediately.

set -euo pipefail

messages=()

# Check OAS spec age (macOS stat syntax; falls back to GNU stat)
if [ -f stackhawk-openapi.json ]; then
  mod_time=$(stat -f %m stackhawk-openapi.json 2>/dev/null || stat -c %Y stackhawk-openapi.json 2>/dev/null || echo "")
  if [ -n "$mod_time" ]; then
    now=$(date +%s)
    age=$(( (now - mod_time) / 86400 ))
    if [ "$age" -gt 30 ]; then
      messages+=("stackhawk-openapi.json is ${age} days old. Consider refreshing: curl -o stackhawk-openapi.json https://download.stackhawk.com/openapi/stackhawk-openapi.json")
    fi
  fi
fi

# Output combined message if any warnings
if [ ${#messages[@]} -gt 0 ]; then
  msg=$(printf '%s\n' "${messages[@]}")
  escaped=$(echo "$msg" | jq -Rs '.')
  echo "{\"systemMessage\":${escaped}}"
fi
