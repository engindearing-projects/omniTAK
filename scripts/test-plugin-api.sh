#!/bin/bash
#
# Plugin API Manual Testing Script
#
# This script provides manual testing capabilities for the OmniTAK Plugin API.
# It includes curl commands for all plugin endpoints with proper authentication.
#
# Usage:
#   ./test-plugin-api.sh [command] [args]
#
# Commands:
#   setup           - Initial setup (login and get token)
#   list            - List all plugins
#   load            - Load a plugin (requires plugin path)
#   details <id>    - Get plugin details
#   metrics <id>    - Get plugin metrics
#   health <id>     - Get plugin health
#   config <id>     - Update plugin configuration
#   toggle <id>     - Toggle plugin enabled/disabled
#   reload <id>     - Reload a specific plugin
#   reload-all      - Reload all plugins
#   unload <id>     - Unload a plugin
#   all             - Run all test commands in sequence
#   help            - Show this help message

set -e

# Configuration
BASE_URL="${OMNITAK_API_URL:-http://localhost:8443}"
ADMIN_USER="${ADMIN_USER:-admin}"
ADMIN_PASS="${ADMIN_PASS:-admin_password_123}"
OPERATOR_USER="${OPERATOR_USER:-operator}"
OPERATOR_PASS="${OPERATOR_PASS:-operator_password_123}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Token storage
TOKEN_FILE="/tmp/omnitak-test-token.txt"
TEST_PLUGIN_ID="test-filter-plugin"

# ============================================================================
# Helper Functions
# ============================================================================

print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Check if server is reachable
check_server() {
    print_info "Checking if server is reachable at $BASE_URL..."
    if curl -s -f -m 5 "$BASE_URL/health" > /dev/null 2>&1; then
        print_success "Server is reachable"
        return 0
    else
        print_error "Server is not reachable at $BASE_URL"
        print_info "Make sure the OmniTAK server is running"
        print_info "You can set a custom URL with: export OMNITAK_API_URL=http://your-server:port"
        return 1
    fi
}

# Login and get token
login() {
    local username="$1"
    local password="$2"

    print_info "Logging in as $username..."

    local response=$(curl -s -X POST "$BASE_URL/api/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$username\",\"password\":\"$password\"}")

    local token=$(echo "$response" | jq -r '.access_token // empty')

    if [ -n "$token" ] && [ "$token" != "null" ]; then
        echo "$token" > "$TOKEN_FILE"
        print_success "Logged in successfully"
        print_info "Token saved to $TOKEN_FILE"
        return 0
    else
        print_error "Login failed"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        return 1
    fi
}

# Get stored token
get_token() {
    if [ ! -f "$TOKEN_FILE" ]; then
        print_warning "No token found. Running login first..."
        login "$ADMIN_USER" "$ADMIN_PASS" || return 1
    fi
    cat "$TOKEN_FILE"
}

# Pretty print JSON response
print_response() {
    local response="$1"
    local status="$2"

    if [ "$status" -ge 200 ] && [ "$status" -lt 300 ]; then
        print_success "Status: $status"
    else
        print_error "Status: $status"
    fi

    echo "$response" | jq '.' 2>/dev/null || echo "$response"
}

# Make authenticated API request
api_request() {
    local method="$1"
    local endpoint="$2"
    local data="$3"

    local token=$(get_token) || return 1
    local url="$BASE_URL$endpoint"

    print_info "Request: $method $endpoint"

    local response
    local status

    if [ -z "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$url" \
            -H "Authorization: Bearer $token" \
            -H "Content-Type: application/json")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$url" \
            -H "Authorization: Bearer $token" \
            -H "Content-Type: application/json" \
            -d "$data")
    fi

    status=$(echo "$response" | tail -n1)
    response=$(echo "$response" | sed '$d')

    print_response "$response" "$status"
    echo ""
}

# ============================================================================
# Test Commands
# ============================================================================

cmd_setup() {
    print_header "Initial Setup"
    check_server || return 1
    login "$ADMIN_USER" "$ADMIN_PASS"
}

cmd_list() {
    print_header "List All Plugins"
    api_request "GET" "/api/v1/plugins"
}

cmd_list_filters() {
    print_header "List Filter Plugins"
    api_request "GET" "/api/v1/plugins?plugin_type=filter"
}

cmd_load() {
    print_header "Load Plugin"

    local plugin_path="${1:-/tmp/test-plugin.wasm}"
    print_info "Plugin path: $plugin_path"

    local payload=$(cat <<EOF
{
    "id": "$TEST_PLUGIN_ID",
    "path": "$plugin_path",
    "enabled": true,
    "pluginType": "filter",
    "config": {
        "id": "$TEST_PLUGIN_ID",
        "name": "Test Filter Plugin",
        "version": "0.1.0",
        "author": "Test Suite",
        "description": "A test filter plugin for API testing",
        "maxExecutionTimeUs": 1000
    }
}
EOF
)

    api_request "POST" "/api/v1/plugins" "$payload"
}

cmd_details() {
    local plugin_id="${1:-$TEST_PLUGIN_ID}"
    print_header "Get Plugin Details: $plugin_id"
    api_request "GET" "/api/v1/plugins/$plugin_id"
}

cmd_metrics() {
    local plugin_id="${1:-$TEST_PLUGIN_ID}"
    print_header "Get Plugin Metrics: $plugin_id"
    api_request "GET" "/api/v1/plugins/$plugin_id/metrics"
}

cmd_health() {
    local plugin_id="${1:-$TEST_PLUGIN_ID}"
    print_header "Get Plugin Health: $plugin_id"
    api_request "GET" "/api/v1/plugins/$plugin_id/health"
}

cmd_config() {
    local plugin_id="${1:-$TEST_PLUGIN_ID}"
    print_header "Update Plugin Config: $plugin_id"

    local payload=$(cat <<EOF
{
    "config": {
        "threshold": 100,
        "enabled": true,
        "filterRules": [
            {
                "pattern": "^a-",
                "action": "allow"
            }
        ]
    }
}
EOF
)

    api_request "PUT" "/api/v1/plugins/$plugin_id/config" "$payload"
}

cmd_toggle() {
    local plugin_id="${1:-$TEST_PLUGIN_ID}"
    local enabled="${2:-false}"
    print_header "Toggle Plugin: $plugin_id (enabled=$enabled)"

    local payload="{\"enabled\": $enabled}"
    api_request "POST" "/api/v1/plugins/$plugin_id/toggle" "$payload"
}

cmd_reload() {
    local plugin_id="${1:-$TEST_PLUGIN_ID}"
    print_header "Reload Plugin: $plugin_id"
    api_request "POST" "/api/v1/plugins/$plugin_id/reload"
}

cmd_reload_all() {
    print_header "Reload All Plugins"
    api_request "POST" "/api/v1/plugins/reload-all"
}

cmd_unload() {
    local plugin_id="${1:-$TEST_PLUGIN_ID}"
    print_header "Unload Plugin: $plugin_id"
    api_request "DELETE" "/api/v1/plugins/$plugin_id"
}

# ============================================================================
# Test Sequences
# ============================================================================

cmd_all() {
    print_header "Running All Plugin API Tests"

    # Setup
    cmd_setup || return 1
    sleep 1

    # List (should be empty or have existing plugins)
    cmd_list
    sleep 1

    # Try to load a plugin (will fail without actual WASM)
    print_warning "Load plugin will fail without actual WASM file"
    cmd_load
    sleep 1

    # Try to get details (will work if plugin exists)
    cmd_details
    sleep 1

    # Try to get metrics
    cmd_metrics
    sleep 1

    # Try to get health
    cmd_health
    sleep 1

    # Try to update config
    cmd_config
    sleep 1

    # Try to toggle
    cmd_toggle "$TEST_PLUGIN_ID" "false"
    sleep 1

    # Try to reload
    cmd_reload
    sleep 1

    # Reload all
    cmd_reload_all
    sleep 1

    # List again
    cmd_list
    sleep 1

    print_header "All Tests Completed"
}

cmd_permissions() {
    print_header "Testing Permission Levels"

    # Test with operator account
    print_info "Testing with operator account..."
    login "$OPERATOR_USER" "$OPERATOR_PASS" || return 1

    print_info "Operator attempting to load plugin (should fail)..."
    cmd_load
    sleep 1

    print_info "Operator attempting to update config (should succeed)..."
    cmd_config
    sleep 1

    # Switch back to admin
    print_info "Switching back to admin account..."
    login "$ADMIN_USER" "$ADMIN_PASS"
}

# ============================================================================
# Help and Main
# ============================================================================

cmd_help() {
    cat <<EOF
Plugin API Manual Testing Script

Usage: $0 [command] [args]

Configuration (via environment variables):
  OMNITAK_API_URL    - Base URL of the API (default: http://localhost:8443)
  ADMIN_USER         - Admin username (default: admin)
  ADMIN_PASS         - Admin password (default: admin_password_123)
  OPERATOR_USER      - Operator username (default: operator)
  OPERATOR_PASS      - Operator password (default: operator_password_123)

Commands:
  setup              - Initial setup (login and get token)
  list               - List all plugins
  list-filters       - List only filter plugins
  load [path]        - Load a plugin (default path: /tmp/test-plugin.wasm)
  details [id]       - Get plugin details (default: $TEST_PLUGIN_ID)
  metrics [id]       - Get plugin metrics
  health [id]        - Get plugin health status
  config [id]        - Update plugin configuration
  toggle [id] [bool] - Toggle plugin (enabled true/false)
  reload [id]        - Reload a specific plugin
  reload-all         - Reload all plugins
  unload [id]        - Unload a plugin
  all                - Run all test commands in sequence
  permissions        - Test permission levels (admin vs operator)
  help               - Show this help message

Examples:
  $0 setup
  $0 list
  $0 load /path/to/plugin.wasm
  $0 details my-plugin-id
  $0 toggle my-plugin-id false
  $0 all

Notes:
  - The server must be running before executing tests
  - Authentication token is stored in $TOKEN_FILE
  - Most commands require admin privileges
  - Some operations (config, toggle) can be done by operators

EOF
}

# ============================================================================
# Main Script Logic
# ============================================================================

main() {
    # Check for jq
    if ! command -v jq &> /dev/null; then
        print_error "jq is required but not installed"
        print_info "Install with: brew install jq (macOS) or apt-get install jq (Linux)"
        exit 1
    fi

    # Check for curl
    if ! command -v curl &> /dev/null; then
        print_error "curl is required but not installed"
        exit 1
    fi

    local command="${1:-help}"
    shift || true

    case "$command" in
        setup)
            cmd_setup
            ;;
        list)
            cmd_list
            ;;
        list-filters)
            cmd_list_filters
            ;;
        load)
            cmd_load "$@"
            ;;
        details)
            cmd_details "$@"
            ;;
        metrics)
            cmd_metrics "$@"
            ;;
        health)
            cmd_health "$@"
            ;;
        config)
            cmd_config "$@"
            ;;
        toggle)
            cmd_toggle "$@"
            ;;
        reload)
            cmd_reload "$@"
            ;;
        reload-all)
            cmd_reload_all
            ;;
        unload)
            cmd_unload "$@"
            ;;
        all)
            cmd_all
            ;;
        permissions)
            cmd_permissions
            ;;
        help|--help|-h)
            cmd_help
            ;;
        *)
            print_error "Unknown command: $command"
            echo ""
            cmd_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
