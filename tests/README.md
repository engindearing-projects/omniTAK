# OmniTAK Plugin System Integration Tests

This directory contains comprehensive integration tests for the OmniTAK plugin system, including API endpoint tests, end-to-end workflow tests, and manual testing utilities.

## Test Files

### 1. API Integration Tests
**File**: `crates/omnitak-api/tests/plugin_api_test.rs`

Unit and integration tests for all plugin API endpoints:
- ✓ List plugins (with filters)
- ✓ Load plugins (with validation)
- ✓ Get plugin details
- ✓ Update plugin configuration
- ✓ Toggle plugin enable/disable
- ✓ Get plugin metrics
- ✓ Get plugin health
- ✓ Reload plugins
- ✓ Unload plugins
- ✓ Permission validation (admin vs operator)
- ✓ Error handling and edge cases

**Run with**:
```bash
cd crates/omnitak-api
cargo test plugin_api_test
```

### 2. End-to-End Integration Tests
**File**: `tests/plugin_integration_test.rs`

Full end-to-end tests that simulate real-world usage:
- Server startup and authentication
- Complete plugin lifecycle (load → configure → use → unload)
- API endpoint integration
- Metrics collection
- Permission levels
- Performance testing

**Run with**:
```bash
# Run all E2E tests (requires running server)
cargo test --test plugin_integration_test -- --ignored

# Run documentation test (no server required)
cargo test --test plugin_integration_test test_plugin_system_e2e_without_server
```

### 3. Manual Testing Script
**File**: `scripts/test-plugin-api.sh`

Interactive bash script for manual API testing with curl commands.

**Usage**:
```bash
# Show help
./scripts/test-plugin-api.sh help

# Initial setup and login
./scripts/test-plugin-api.sh setup

# List all plugins
./scripts/test-plugin-api.sh list

# Load a plugin
./scripts/test-plugin-api.sh load /path/to/plugin.wasm

# Get plugin details
./scripts/test-plugin-api.sh details test-filter-plugin

# Get metrics
./scripts/test-plugin-api.sh metrics test-filter-plugin

# Update configuration
./scripts/test-plugin-api.sh config test-filter-plugin

# Toggle plugin
./scripts/test-plugin-api.sh toggle test-filter-plugin false

# Unload plugin
./scripts/test-plugin-api.sh unload test-filter-plugin

# Run all tests
./scripts/test-plugin-api.sh all

# Test permissions
./scripts/test-plugin-api.sh permissions
```

## Test Fixtures

### Plugin Metadata Files
Located in `tests/fixtures/plugins/`:
- `test-filter-metadata.json` - Sample filter plugin configuration
- `test-transformer-metadata.json` - Sample transformer plugin configuration

### Common Test Utilities
Located in `tests/common/mod.rs`:
- Fixture directory helpers
- Mock WASM file generation
- Metadata loading utilities
- Test cleanup functions

## Test Coverage

### API Endpoints Tested

| Endpoint | Method | Tests |
|----------|--------|-------|
| `/api/v1/plugins` | GET | List plugins, filters, permissions |
| `/api/v1/plugins` | POST | Load plugin, validation, errors |
| `/api/v1/plugins/:id` | GET | Get details, not found |
| `/api/v1/plugins/:id` | DELETE | Unload, permissions, not found |
| `/api/v1/plugins/:id/config` | PUT | Update config, permissions |
| `/api/v1/plugins/:id/toggle` | POST | Enable/disable, permissions |
| `/api/v1/plugins/:id/metrics` | GET | Get metrics, not found |
| `/api/v1/plugins/:id/health` | GET | Get health, not found |
| `/api/v1/plugins/:id/reload` | POST | Reload, permissions |
| `/api/v1/plugins/reload-all` | POST | Reload all, permissions |

### Test Scenarios

#### ✓ Happy Path
- Successfully list plugins
- Load and unload plugins
- Update configurations
- Get metrics and health status

#### ✓ Error Cases
- Invalid plugin ID (validation)
- Non-existent plugin (404)
- Plugin file not found (500)
- Invalid JSON payload (400)
- Missing authentication (401)
- Insufficient permissions (403)

#### ✓ Permissions
- Admin can: load, unload, reload
- Operator can: list, configure, toggle
- Read-only can: list, view details

#### ✓ Edge Cases
- Empty plugin list
- Multiple plugins with filters
- Concurrent operations
- Cleanup and resource management

## Running Tests

### Prerequisites

1. **Install dependencies**:
   ```bash
   cargo build
   ```

2. **For E2E tests, start the server**:
   ```bash
   cargo run --bin omnitak
   ```
   Or set custom URL:
   ```bash
   export OMNITAK_API_URL=http://localhost:8443
   ```

3. **For manual testing, install jq**:
   ```bash
   # macOS
   brew install jq

   # Linux
   apt-get install jq
   ```

### Run All Tests

```bash
# Run unit/integration tests (no server required)
cargo test

# Run API integration tests
cargo test -p omnitak-api plugin_api_test

# Run E2E tests (requires server)
cargo test --test plugin_integration_test -- --ignored

# Run specific test
cargo test test_list_plugins_empty

# Run with output
cargo test -- --nocapture
```

### Run Manual Tests

```bash
# Start server first
cargo run --bin omnitak &

# Run manual tests
./scripts/test-plugin-api.sh all

# Or test specific endpoints
./scripts/test-plugin-api.sh setup
./scripts/test-plugin-api.sh list
./scripts/test-plugin-api.sh load /path/to/plugin.wasm
```

## Configuration

### Environment Variables

- `OMNITAK_API_URL` - API base URL (default: `http://localhost:8443`)
- `ADMIN_USER` - Admin username (default: `admin`)
- `ADMIN_PASS` - Admin password (default: `admin_password_123`)
- `OPERATOR_USER` - Operator username (default: `operator`)
- `OPERATOR_PASS` - Operator password (default: `operator_password_123`)

### Test Configuration

Tests use these default values:
- Test server address: `127.0.0.1:18443`
- Test plugin directory: `/tmp/omnitak-test-plugins`
- Auth token storage: `/tmp/omnitak-test-token.txt`

## Creating Test Plugins

To create actual WASM plugins for testing:

```bash
# Build example plugin
cd examples/plugins/geofence-filter
cargo build --release --target wasm32-wasi

# Copy to test fixtures
cp target/wasm32-wasi/release/geofence_filter_plugin.wasm \
   ../../../tests/fixtures/plugins/

# Use in tests
./scripts/test-plugin-api.sh load \
   tests/fixtures/plugins/geofence_filter_plugin.wasm
```

## Test Results

Expected test output:
```
running 25 tests
test test_list_plugins_empty ... ok
test test_list_plugins_unauthorized ... ok
test test_load_plugin_validation_errors ... ok
test test_load_plugin_not_found ... ok
test test_load_plugin_requires_admin ... ok
test test_get_plugin_details_not_found ... ok
test test_update_plugin_config_not_found ... ok
test test_toggle_plugin_not_found ... ok
test test_unload_plugin_not_found ... ok
test test_unload_plugin_requires_admin ... ok
test test_get_plugin_metrics_not_found ... ok
test test_get_plugin_health_not_found ... ok
test test_reload_plugin_requires_admin ... ok
test test_reload_all_plugins_requires_admin ... ok
test test_reload_all_plugins_success ... ok
test test_plugin_lifecycle_without_actual_wasm ... ok
test test_error_responses_have_correct_format ... ok
...

test result: ok. 25 passed; 0 failed
```

## Continuous Integration

Add to your CI pipeline:

```yaml
# .github/workflows/test.yml
- name: Run Plugin API Tests
  run: |
    cargo test -p omnitak-api plugin_api_test

- name: Run Integration Tests
  run: |
    # Start server in background
    cargo run --bin omnitak &
    sleep 5

    # Run E2E tests
    cargo test --test plugin_integration_test -- --ignored

    # Cleanup
    pkill omnitak
```

## Debugging Tests

### Verbose Output
```bash
# Show test output
cargo test -- --nocapture

# Show HTTP requests/responses
RUST_LOG=debug cargo test -- --nocapture

# Run single test with backtrace
RUST_BACKTRACE=1 cargo test test_load_plugin_not_found -- --nocapture
```

### Manual Debugging
```bash
# Use the test script with verbose curl
./scripts/test-plugin-api.sh setup
curl -v -H "Authorization: Bearer $(cat /tmp/omnitak-test-token.txt)" \
  http://localhost:8443/api/v1/plugins
```

## Contributing

When adding new plugin features:

1. Add API endpoint tests to `plugin_api_test.rs`
2. Add E2E tests to `plugin_integration_test.rs`
3. Update the manual test script with new commands
4. Document the test cases in this README
5. Ensure all tests pass before submitting PR

## Troubleshooting

### Common Issues

**Server not starting**:
- Check if port 8443 is already in use
- Verify TLS certificates are configured
- Check logs: `RUST_LOG=debug cargo run --bin omnitak`

**Tests failing with 401 Unauthorized**:
- Token may have expired
- Run `./scripts/test-plugin-api.sh setup` to refresh
- Check username/password configuration

**Plugin load failures**:
- Ensure WASM file exists and is valid
- Check file permissions
- Verify plugin metadata is correct
- Check WASM runtime compatibility

**E2E tests timing out**:
- Increase timeout in test configuration
- Ensure server is fully started before tests run
- Check for resource exhaustion

## References

- [Plugin API Documentation](../crates/omnitak-api/src/rest/plugins.rs)
- [Plugin Manager Documentation](../crates/omnitak-plugin-api/src/manager.rs)
- [Example Plugins](../examples/plugins/)
- [API Types](../crates/omnitak-api/src/types.rs)
