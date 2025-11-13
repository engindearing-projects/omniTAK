# OmniTAK Testing Guide

Quick reference for running plugin system tests.

## Quick Start

```bash
# Run all unit/integration tests
cargo test

# Run plugin API tests
cargo test -p omnitak-api plugin_api_test

# Run manual test script
./scripts/test-plugin-api.sh all
```

## Test Types

### 1. API Integration Tests
Fast tests that verify API endpoint behavior without requiring a running server.

```bash
cd crates/omnitak-api
cargo test plugin_api_test
```

**Coverage**: 25+ tests covering all plugin endpoints, validation, permissions, and error cases.

### 2. End-to-End Tests
Full integration tests that require a running server instance.

```bash
# Terminal 1: Start server
cargo run --bin omnitak

# Terminal 2: Run E2E tests
cargo test --test plugin_integration_test -- --ignored
```

### 3. Manual Testing
Interactive script for manual API testing with real HTTP requests.

```bash
./scripts/test-plugin-api.sh help
./scripts/test-plugin-api.sh setup
./scripts/test-plugin-api.sh list
```

## Common Commands

### Run Specific Test
```bash
cargo test test_list_plugins_empty
cargo test test_load_plugin_requires_admin
```

### Run with Output
```bash
cargo test -- --nocapture
RUST_LOG=debug cargo test -- --nocapture
```

### Run Ignored Tests
```bash
cargo test -- --ignored
```

### Run All Tests
```bash
cargo test --workspace
```

## Manual Testing Examples

```bash
# Setup authentication
./scripts/test-plugin-api.sh setup

# List all plugins
./scripts/test-plugin-api.sh list

# Load a plugin
./scripts/test-plugin-api.sh load /path/to/plugin.wasm

# Get plugin details
./scripts/test-plugin-api.sh details my-plugin

# Get metrics
./scripts/test-plugin-api.sh metrics my-plugin

# Update config
./scripts/test-plugin-api.sh config my-plugin

# Toggle plugin
./scripts/test-plugin-api.sh toggle my-plugin false

# Reload plugin
./scripts/test-plugin-api.sh reload my-plugin

# Unload plugin
./scripts/test-plugin-api.sh unload my-plugin
```

## Configuration

Set environment variables to customize tests:

```bash
export OMNITAK_API_URL=http://localhost:8443
export ADMIN_USER=admin
export ADMIN_PASS=admin_password_123
```

## Test Coverage Summary

| Component | Tests | Status |
|-----------|-------|--------|
| List Plugins | 3 | ✓ |
| Load Plugin | 4 | ✓ |
| Plugin Details | 2 | ✓ |
| Update Config | 3 | ✓ |
| Toggle Plugin | 2 | ✓ |
| Plugin Metrics | 2 | ✓ |
| Plugin Health | 2 | ✓ |
| Reload Plugin | 3 | ✓ |
| Unload Plugin | 3 | ✓ |
| Permissions | 5 | ✓ |
| Error Handling | 5 | ✓ |
| **Total** | **34** | **✓** |

## Debugging

### Enable Debug Logging
```bash
RUST_LOG=debug cargo test -- --nocapture
```

### Check Server Health
```bash
curl http://localhost:8443/health
```

### View Auth Token
```bash
cat /tmp/omnitak-test-token.txt
```

### Manual curl Request
```bash
TOKEN=$(cat /tmp/omnitak-test-token.txt)
curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8443/api/v1/plugins
```

## CI/CD Integration

Add to your CI pipeline:

```yaml
- name: Run Tests
  run: |
    cargo test --workspace

- name: Run Integration Tests
  run: |
    cargo run --bin omnitak &
    sleep 5
    ./scripts/test-plugin-api.sh all
```

## More Information

See [tests/README.md](tests/README.md) for detailed documentation.
