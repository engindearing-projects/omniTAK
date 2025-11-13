# OmniTAK Plugin Development Guide

**Version 0.1.0** | Last Updated: November 2025

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Getting Started](#getting-started)
4. [Plugin Types](#plugin-types)
5. [Development Workflow](#development-workflow)
6. [API Reference](#api-reference)
7. [Best Practices](#best-practices)
8. [Deployment](#deployment)
9. [Troubleshooting](#troubleshooting)

---

## Overview

OmniTAK supports a powerful WebAssembly (WASM) plugin system that allows you to extend TAK message processing without modifying the core codebase. Plugins run in a sandboxed environment with configurable resource limits and security policies.

### Why WASM Plugins?

- **Language Agnostic**: Write plugins in Rust, C++, Go, or any language that compiles to WASM
- **Sandboxed Execution**: Plugins run in isolated environments with no access to system resources by default
- **High Performance**: WASM provides near-native execution speed (85-90% of native)
- **Hot Reload**: Load, unload, and update plugins without restarting OmniTAK
- **Cross-Platform**: Same plugin binary works on Linux, macOS, and Windows

### Use Cases

- **Custom Filters**: Advanced geofencing, mission-specific routing, ML-based classification
- **Message Transformation**: Protocol translation, data enrichment, coordinate conversion
- **Protocol Handlers**: Support for custom TAK variants or non-TAK systems
- **Analytics**: Real-time message analysis and pattern detection

---

## Architecture

### Plugin System Components

```
┌─────────────────────────────────────────────────────┐
│                   OmniTAK Core                      │
├─────────────────────────────────────────────────────┤
│  ┌──────────────┐         ┌──────────────┐         │
│  │ Filter Chain │ ◄─────► │ Plugin API   │         │
│  └──────────────┘         └──────────────┘         │
│         │                        │                  │
│         │                 ┌──────▼──────┐           │
│         │                 │   Runtime   │           │
│         │                 │  (Wasmtime) │           │
│         │                 └──────┬──────┘           │
│         │                        │                  │
│         │        ┌───────────────┴────────────┐     │
│         │        │                            │     │
│         │   ┌────▼────┐               ┌───────▼──┐  │
│         └──►│ Filter  │               │Transform │  │
│             │ Plugin  │               │ Plugin   │  │
│             └─────────┘               └──────────┘  │
│               (WASM)                    (WASM)      │
└─────────────────────────────────────────────────────┘
```

### Component Interfaces (WIT)

Plugins communicate with OmniTAK using WebAssembly Interface Types (WIT). The interface definition is in `crates/omnitak-plugin-api/wit/plugin.wit`.

**Key Interfaces:**
- `filter` - Filter CoT messages
- `transformer` - Transform message payloads
- `metadata` - Plugin information
- `host` - Functions provided by OmniTAK to plugins

---

## Getting Started

### Prerequisites

1. **Rust toolchain** (1.90+)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **WASM target**
```bash
rustup target add wasm32-wasip1
```

3. **cargo-component** (for Component Model support)
```bash
cargo install cargo-component
```

### Creating Your First Plugin

#### 1. Create a new project

```bash
cargo new --lib my-filter-plugin
cd my-filter-plugin
```

#### 2. Configure Cargo.toml

```toml
[package]
name = "my-filter-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.36"

[profile.release]
opt-level = "z"  # Optimize for size
lto = true
strip = true

[package.metadata.component]
package = "omnitak:plugin"
```

#### 3. Implement the plugin

```rust
// src/lib.rs
wit_bindgen::generate!({
    path: "path/to/omnitak-plugin-api/wit",
    world: "filter-plugin",
});

use exports::omnitak::plugin::filter::{CotMessage, FilterResult, FilterMetadata, Guest};

struct MyFilterPlugin;

impl Guest for MyFilterPlugin {
    fn evaluate(msg: CotMessage) -> FilterResult {
        // Your filter logic here
        if msg.callsign.as_ref().map_or(false, |cs| cs.starts_with("ALPHA")) {
            FilterResult::Pass
        } else {
            FilterResult::Block
        }
    }

    fn describe() -> String {
        "Only allow callsigns starting with ALPHA".to_string()
    }

    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            id: "alpha-callsign-filter".to_string(),
            name: "ALPHA Callsign Filter".to_string(),
            version: "0.1.0".to_string(),
            author: "Your Name".to_string(),
            description: "Filters messages to only allow ALPHA callsigns".to_string(),
            max_execution_time_us: 1000,
        }
    }
}

export!(MyFilterPlugin);
```

#### 4. Build the plugin

```bash
cargo component build --release
```

The compiled plugin will be at:
```
target/wasm32-wasip1/release/my_filter_plugin.wasm
```

---

## Plugin Types

### 1. Filter Plugins

Filter plugins evaluate CoT messages and decide whether to pass or block them.

**Interface:**
```wit
interface filter {
    evaluate: func(msg: cot-message) -> filter-result;
    describe: func() -> string;
    get-metadata: func() -> filter-metadata;
}
```

**Example Use Cases:**
- Geofencing
- Affiliation-based filtering
- Time-based routing
- Custom access control

**Performance Target:** < 1μs per evaluation

### 2. Transformer Plugins

Transform plugins modify message payloads in-flight.

**Interface:**
```wit
interface transformer {
    transform: func(data: list<u8>) -> result<list<u8>, string>;
    can-transform: func(cot-type: string) -> bool;
    get-metadata: func() -> transformer-metadata;
}
```

**Example Use Cases:**
- Protocol translation
- Data enrichment (adding elevation, weather)
- Coordinate system conversion
- Message redaction/sanitization

**Performance Target:** < 10μs per transformation

---

## Development Workflow

### 1. Local Development

```bash
# Create plugin
cargo new --lib my-plugin
cd my-plugin

# Develop and test
cargo test

# Build WASM
cargo component build --release

# Test with OmniTAK
cp target/wasm32-wasip1/release/*.wasm /path/to/omnitak/plugins/
```

### 2. Testing

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_logic() {
        let msg = CotMessage {
            cot_type: "a-f-G",
            uid: "test-123",
            callsign: Some("ALPHA-1"),
            // ...
        };
        assert_eq!(MyFilterPlugin::evaluate(msg), FilterResult::Pass);
    }
}
```

#### Integration Tests

Create a test configuration:
```yaml
# test-config.yaml
plugins:
  filters:
    - id: my-filter
      path: target/wasm32-wasip1/release/my_filter_plugin.wasm
      enabled: true
```

Run OmniTAK with test config:
```bash
cargo run -- --config test-config.yaml
```

### 3. Debugging

#### Enable Plugin Logging

Plugins can log to the host:
```rust
omnitak::plugin::host::log(
    omnitak::plugin::host::LogLevel::Debug,
    &format!("Processing message: {}", msg.uid)
);
```

View logs:
```bash
RUST_LOG=debug cargo run
```

#### Performance Profiling

Check execution time in OmniTAK metrics:
```bash
curl http://localhost:9443/api/v1/metrics | grep plugin
```

---

## API Reference

### CotMessage Structure

```rust
record cot-message {
    cot-type: string,           // e.g., "a-f-G-E-V"
    uid: string,                // Unique identifier
    callsign: option<string>,   // Entity name
    group: option<string>,      // Group name
    team: option<string>,       // Team color
    lat: float64,               // Latitude (decimal degrees)
    lon: float64,               // Longitude (decimal degrees)
    hae: option<float64>,       // Height above ellipsoid (meters)
    time: string,               // ISO 8601 timestamp
    xml-payload: option<string> // Full XML (if needed)
}
```

### Host Functions

Functions provided by OmniTAK to plugins:

#### Logging
```rust
omnitak::plugin::host::log(level: LogLevel, message: string)
```

Levels: `Trace`, `Debug`, `Info`, `Warn`, `Error`

#### Time
```rust
omnitak::plugin::host::get_current_time_ms() -> u64
```

Returns milliseconds since Unix epoch.

#### Geospatial (if enabled)
```rust
omnitak::plugin::host::query_elevation(lat: f64, lon: f64) -> option<f64>
```

Query terrain elevation database.

---

## Best Practices

### Performance

1. **Minimize Allocations**
   - Avoid `Vec`, `String` allocations in hot path
   - Use stack variables when possible
   - Preallocate buffers

2. **Optimize for Size**
   ```toml
   [profile.release]
   opt-level = "z"  # or "s"
   lto = true
   codegen-units = 1
   strip = true
   ```

3. **Cache Expensive Operations**
   ```rust
   use once_cell::sync::Lazy;

   static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"pattern").unwrap());
   ```

### Security

1. **Validate All Inputs**
   ```rust
   fn evaluate(msg: CotMessage) -> FilterResult {
       // Validate coordinates
       if msg.lat < -90.0 || msg.lat > 90.0 {
           return FilterResult::Block;
       }
       // ... rest of logic
   }
   ```

2. **Handle Errors Gracefully**
   ```rust
   fn transform(data: list<u8>) -> result<list<u8>, string> {
       match parse_data(data) {
           Ok(parsed) => Ok(process(parsed)),
           Err(e) => Err(format!("Parse error: {}", e))
       }
   }
   ```

3. **Set Realistic Timeouts**
   ```rust
   get_metadata() -> FilterMetadata {
       FilterMetadata {
           max_execution_time_us: 500,  // Be conservative
           // ...
       }
   }
   ```

### Code Organization

```
my-plugin/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs          # Plugin entry point
│   ├── filter.rs       # Filter logic
│   └── utils.rs        # Helper functions
├── tests/
│   └── integration.rs
└── build.sh            # Build script
```

---

## Deployment

### Configuration

Add plugin to OmniTAK config:

```yaml
# config.yaml
plugins:
  # Plugin directory
  plugin_dir: "./plugins"

  # Hot reload (dev only)
  hot_reload: false

  # Resource limits
  resource_limits:
    max_execution_time_ms: 1
    max_memory_bytes: 10485760  # 10MB
    max_concurrent_executions: 100

  # Security policy
  sandbox_policy:
    allow_network: false
    allow_filesystem_read: false
    allow_filesystem_write: false
    allow_env_vars: false

  # Loaded plugins
  filters:
    - id: geofence-filter
      path: plugins/geofence_filter.wasm
      enabled: true
      priority: 100

    - id: custom-filter
      path: plugins/my_filter.wasm
      enabled: true
      priority: 50
```

### Loading Plugins

#### At Startup

Plugins are automatically loaded when OmniTAK starts.

#### Runtime Loading

Use the REST API:

```bash
curl -X POST http://localhost:9443/api/v1/plugins \
  -H "Content-Type: application/json" \
  -d '{
    "type": "filter",
    "path": "plugins/new_plugin.wasm",
    "enabled": true
  }'
```

### Monitoring

Check plugin status:
```bash
curl http://localhost:9443/api/v1/plugins
```

View metrics:
```bash
curl http://localhost:9443/api/v1/metrics | grep omnitak_plugin
```

Metrics available:
- `omnitak_plugin_executions_total` - Total plugin calls
- `omnitak_plugin_execution_duration_seconds` - Execution time histogram
- `omnitak_plugin_errors_total` - Error count
- `omnitak_plugin_timeouts_total` - Timeout count

---

## Troubleshooting

### Common Issues

#### 1. Plugin Won't Load

**Error:** `Failed to load plugin: invalid WASM module`

**Solutions:**
- Ensure you're targeting `wasm32-wasip1`
- Rebuild with `cargo component build`
- Check WIT path in `wit_bindgen::generate!()`

#### 2. Compilation Errors

**Error:** `error: linking with cc failed`

**Solution:** Add to `Cargo.toml`:
```toml
[lib]
crate-type = ["cdylib"]
```

#### 3. Performance Issues

**Error:** `Plugin execution timeout`

**Solutions:**
- Reduce `max_execution_time_us`
- Profile with `cargo bench`
- Optimize hot paths
- Check for excessive allocations

#### 4. Import/Export Mismatch

**Error:** `no function export found for 'evaluate'`

**Solution:** Ensure you call `export!(YourPlugin)` at the end of `lib.rs`

### Debug Mode

Build with debug symbols:
```bash
cargo component build --release
```

Enable verbose logging:
```bash
RUST_LOG=omnitak_plugin=trace cargo run
```

---

## Examples

See `examples/plugins/` directory for:
- **geofence-filter** - Geographic boundary filtering
- **affiliation-filter** - MIL-STD-2525 affiliation routing
- **callsign-transformer** - Callsign normalization
- **elevation-enricher** - Add terrain elevation data

---

## Additional Resources

- [OmniTAK GitHub](https://github.com/engindearing-projects/omniTAK)
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
- [Wasmtime Documentation](https://docs.wasmtime.dev/)
- [WIT Reference](https://component-model.bytecodealliance.org/design/wit.html)

---

## Support

- **Issues**: [GitHub Issues](https://github.com/engindearing-projects/omniTAK/issues)
- **Discussions**: [GitHub Discussions](https://github.com/engindearing-projects/omniTAK/discussions)

---

**Built with ❤️ for the TAK community**
