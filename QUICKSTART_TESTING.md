# Quick Start: Testing the Plugin System

## TL;DR

```bash
# Run automated tests
cargo test -p omnitak-api plugin_api_test

# Run manual tests (requires server)
./scripts/test-plugin-api.sh all
```

## 3-Minute Quick Start

### Option 1: Automated Tests (No Server Required)

```bash
cd crates/omnitak-api
cargo test plugin_api_test
```

Expected output:
```
running 26 tests
...
test result: ok. 26 passed; 0 failed
```

### Option 2: Manual Testing (Interactive)

**Step 1**: Start the server
```bash
cargo run --bin omnitak
```

**Step 2**: Run tests in another terminal
```bash
./scripts/test-plugin-api.sh setup    # Login
./scripts/test-plugin-api.sh list     # List plugins
./scripts/test-plugin-api.sh all      # Run all tests
```

## Common Commands

| Task | Command |
|------|---------|
| Run all API tests | `cargo test -p omnitak-api plugin_api_test` |
| Run specific test | `cargo test test_list_plugins_empty` |
| Show test output | `cargo test -- --nocapture` |
| Manual test - setup | `./scripts/test-plugin-api.sh setup` |
| Manual test - list | `./scripts/test-plugin-api.sh list` |
| Manual test - all | `./scripts/test-plugin-api.sh all` |
| Show manual help | `./scripts/test-plugin-api.sh help` |

## What's Tested?

✅ List plugins
✅ Load plugins
✅ Get plugin details
✅ Update configuration
✅ Toggle enable/disable
✅ Get metrics
✅ Get health
✅ Reload plugins
✅ Unload plugins
✅ Permissions (admin/operator)
✅ Error handling

## Need Help?

See full documentation:
- `tests/README.md` - Comprehensive guide
- `TESTING.md` - Quick reference
- `TEST_SUMMARY.md` - Detailed summary

## Troubleshooting

**Tests won't run**: Check if you're in the right directory
```bash
cd /path/to/omniTAK
cargo test -p omnitak-api
```

**Server not reachable**: Make sure it's running
```bash
cargo run --bin omnitak
```

**Permission errors**: Run setup first
```bash
./scripts/test-plugin-api.sh setup
```

That's it! You're ready to test the plugin system.
