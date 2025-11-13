# Test Plugin Fixtures

This directory contains test fixtures for plugin integration tests.

## Mock Plugins

Since WASM plugins require compilation, the tests use mock plugins for verification.

### Creating Test WASM Files

To create actual test WASM plugins, you can:

1. Build the example plugins:
   ```bash
   cd examples/plugins/geofence-filter
   cargo build --release --target wasm32-wasi
   cp target/wasm32-wasi/release/geofence_filter_plugin.wasm ../../../tests/fixtures/plugins/
   ```

2. Create a minimal test plugin following the plugin API specification

## Mock Plugin Data

For API tests that don't require actual WASM execution, the tests use JSON
metadata files that describe plugin configurations.

## Files

- `test-filter-metadata.json` - Metadata for a test filter plugin
- `test-transformer-metadata.json` - Metadata for a test transformer plugin
- `mock-plugin.wasm` - Placeholder WASM file (if available)
