#!/usr/bin/env bash
# Check that all CLI commands have corresponding functional tests.
#
# Uses two strategies to detect coverage:
# 1. Test function names (e.g., test_team_add_user_*)
# 2. Command string patterns in test bodies (e.g., "add-user")
#
# Usage: ./scripts/check-test-coverage.sh
# Exit code: 0 if all covered, 1 if gaps found

set -e

# Colors (respect NO_COLOR)
if [ -z "${NO_COLOR:-}" ] && [ "${TERM:-dumb}" != "dumb" ]; then
    GREEN='\033[0;32m'
    RED='\033[0;31m'
    YELLOW='\033[0;33m'
    NC='\033[0m'
else
    GREEN='' RED='' YELLOW='' NC=''
fi

TESTS_DIR="tests/functional"

if [ ! -d "$TESTS_DIR" ]; then
    echo -e "${RED}Error: $TESTS_DIR not found. Run from project root.${NC}"
    exit 1
fi

echo "Checking functional test coverage for CLI commands..."
echo ""

gaps=0
covered=0

# Check for coverage using both function names and string patterns
# $1 = display name, $2 = pattern (grep -E across all test files)
check_coverage() {
    local display="$1"
    local pattern="$2"

    if grep -rEq "$pattern" "$TESTS_DIR/" 2>/dev/null; then
        covered=$((covered + 1))
        echo -e "  ${GREEN}OK${NC}       $display"
        return 0
    else
        gaps=$((gaps + 1))
        echo -e "  ${RED}MISSING${NC}  $display"
        return 0  # don't exit, just count
    fi
}

# Standalone commands
check_coverage "hawkop status"         "test_status|\"status\""
check_coverage "hawkop version"        "test_version|\"version\""

# Org commands
check_coverage "hawkop org list"       "test_org_list|\"org\".*\"list\""
check_coverage "hawkop org get"        "test_org_get|\"org\".*\"get\""
check_coverage "hawkop org set"        "test_org_set"

# App commands
check_coverage "hawkop app list"       "test_app_list|\"app\".*\"list\""

# Scan commands
check_coverage "hawkop scan list"      "test_scan_list|\"scan\".*\"list\""
check_coverage "hawkop scan get"       "test_scan.*get|scan.*get.*scan_id|\"scan\".*\"get\""

# Run commands
check_coverage "hawkop run start"      "test_run_start|\"run\".*\"start\""
check_coverage "hawkop run stop"       "test_run_stop|\"run\".*\"stop\""
check_coverage "hawkop run status"     "test_run_status|\"run\".*\"status\""

# User commands
check_coverage "hawkop user list"      "test_user_list|\"user\".*\"list\""

# Team commands
check_coverage "hawkop team list"      "test_team_list|\"team\".*\"list\""
check_coverage "hawkop team get"       "test_team_get|\"team\".*\"get\""
check_coverage "hawkop team create"    "test_team_create|\"team\".*\"create\""
check_coverage "hawkop team delete"    "test_team_delete|\"team\".*\"delete\""
check_coverage "hawkop team rename"    "test_team_rename|\"team\".*\"rename\""
check_coverage "hawkop team add-user"  "test_team_add_user|\"add-user\""
check_coverage "hawkop team remove-user" "test_team_remove_user|\"remove-user\""
check_coverage "hawkop team set-users" "test_team_set_users|\"set-users\""
check_coverage "hawkop team add-app"   "test_team_add_app|\"add-app\""
check_coverage "hawkop team remove-app" "test_team_remove_app|\"remove-app\""
check_coverage "hawkop team set-apps"  "test_team_set_apps|\"set-apps\""

# Policy commands
check_coverage "hawkop policy list"    "test_policy_list|\"policy\".*\"list\""

# Repo commands
check_coverage "hawkop repo list"      "test_repo_list|\"repo\".*\"list\""

# OAS commands
check_coverage "hawkop oas list"       "test_oas_list|\"oas\".*\"list\""
check_coverage "hawkop oas get"        "test_oas_get|\"oas\".*\"get\""
check_coverage "hawkop oas mappings"   "test_oas_mappings|\"oas\".*\"mappings\""

# Config commands
check_coverage "hawkop config list"    "test_config_list|\"config\".*\"list\""
check_coverage "hawkop config get"     "test_config_get|\"config\".*\"get\""
check_coverage "hawkop config set"     "test_config_set|\"config\".*\"set\""
check_coverage "hawkop config delete"  "test_config_delete|\"config\".*\"delete\""
check_coverage "hawkop config rename"  "test_config_rename|\"config\".*\"rename\""
check_coverage "hawkop config validate" "test_config_validate|\"config\".*\"validate\""

# Secret commands
check_coverage "hawkop secret list"    "test_secret_list|\"secret\".*\"list\""

# Audit commands
check_coverage "hawkop audit list"     "test_audit_list|\"audit\".*\"list\""

# Env commands
check_coverage "hawkop env list"       "test_env_list|\"env\".*\"list\""
check_coverage "hawkop env config"     "test_env_config|\"env\".*\"config\""
check_coverage "hawkop env create"     "test_env_create|\"env\".*\"create\""
check_coverage "hawkop env delete"     "test_env_delete|\"env\".*\"delete\""

# Cache commands
check_coverage "hawkop cache status"   "test_cache_status|\"cache\".*\"status\""
check_coverage "hawkop cache path"     "test_cache_path|\"cache\".*\"path\""
check_coverage "hawkop cache clear"    "test_cache_clear|\"cache\".*\"clear\""

# Profile commands
check_coverage "hawkop profile list"   "test_profile_list|\"profile\".*\"list\""
check_coverage "hawkop profile use"    "test_profile_use|\"profile\".*\"use\""
check_coverage "hawkop profile create" "test_profile_create|\"profile\".*\"create\""
check_coverage "hawkop profile delete" "test_profile_delete|\"profile\".*\"delete\""
check_coverage "hawkop profile show"   "test_profile_show|\"profile\".*\"show\""

# Completion
check_coverage "hawkop completion bash" "test_completion_bash|\"completion\".*\"bash\""
check_coverage "hawkop completion zsh"  "test_completion_zsh|\"completion\".*\"zsh\""

echo ""
echo "Coverage: $covered commands tested, $gaps commands missing tests"

if [ "$gaps" -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}Tip: Add tests in tests/functional/ for missing commands.${NC}"
    echo -e "${YELLOW}See docs/CLI_REFERENCE.md for the full test coverage map.${NC}"
    exit 1
fi

echo -e "${GREEN}All commands have functional test coverage.${NC}"
