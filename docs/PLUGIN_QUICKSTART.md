# OmniTAK Plugin System - Quick Start

**Get up and running with plugins in 5 minutes!**

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-wasip1

# Install cargo-component
cargo install cargo-component
```

## Create Your First Plugin

### 1. Create Project

```bash
cargo new --lib my-filter
cd my-filter
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-filter"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.36"

[profile.release]
opt-level = "z"
lto = true
strip = true

[package.metadata.component]
package = "omnitak:plugin"
```

### 3. Write Plugin Code

Create `src/lib.rs`:

```rust
wit_bindgen::generate!({
    path: "path/to/omnitak/crates/omnitak-plugin-api/wit",
    world: "filter-plugin",
});

use exports::omnitak::plugin::filter::{CotMessage, FilterResult, FilterMetadata, Guest};

struct MyFilter;

impl Guest for MyFilter {
    fn evaluate(msg: CotMessage) -> FilterResult {
        // Your logic here
        if msg.lat > 40.0 {
            FilterResult::Pass
        } else {
            FilterResult::Block
        }
    }

    fn describe() -> String {
        "Filters messages above 40°N latitude".to_string()
    }

    fn get_metadata() -> FilterMetadata {
        FilterMetadata {
            id: "north-filter".to_string(),
            name: "Northern Hemisphere Filter".to_string(),
            version: "0.1.0".to_string(),
            author: "Your Name".to_string(),
            description: "Only allows messages from northern latitudes".to_string(),
            max_execution_time_us: 1000,
        }
    }
}

export!(MyFilter);
```

### 4. Build

```bash
cargo component build --release
```

Output: `target/wasm32-wasip1/release/my_filter.wasm`

### 5. Deploy

Copy to OmniTAK plugins directory:
```bash
cp target/wasm32-wasip1/release/my_filter.wasm /path/to/omnitak/plugins/
```

### 6. Configure

Add to `config.yaml`:
```yaml
plugins:
  filters:
    - id: north-filter
      path: plugins/my_filter.wasm
      enabled: true
```

### 7. Run

```bash
cd /path/to/omnitak
cargo run -- --config config.yaml
```

## Common Plugin Types

### Geofence Filter

```rust
fn evaluate(msg: CotMessage) -> FilterResult {
    let in_zone = msg.lat >= MIN_LAT && msg.lat <= MAX_LAT
               && msg.lon >= MIN_LON && msg.lon <= MAX_LON;
    if in_zone {
        FilterResult::Pass
    } else {
        FilterResult::Block
    }
}
```

### Callsign Filter

```rust
fn evaluate(msg: CotMessage) -> FilterResult {
    match msg.callsign {
        Some(cs) if cs.starts_with("ALPHA") => FilterResult::Pass,
        _ => FilterResult::Block
    }
}
```

### Team Filter

```rust
fn evaluate(msg: CotMessage) -> FilterResult {
    match msg.team.as_deref() {
        Some("cyan") | Some("blue") => FilterResult::Pass,
        _ => FilterResult::Block
    }
}
```

## Testing

### Unit Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter() {
        let msg = CotMessage {
            cot_type: "a-f-G",
            uid: "test-123",
            callsign: Some("ALPHA-1"),
            lat: 45.0,
            lon: -120.0,
            // ...
        };
        assert_eq!(MyFilter::evaluate(msg), FilterResult::Pass);
    }
}
```

## Debugging

### Enable Logging

```rust
omnitak::plugin::host::log(
    omnitak::plugin::host::LogLevel::Info,
    &format!("Processing: {}", msg.uid)
);
```

### View Logs

```bash
RUST_LOG=debug cargo run
```

## Examples

See `examples/plugins/` for complete examples:
- **geofence-filter** - Geographic boundary filtering
- More coming soon!

## Next Steps

- Read full guide: [`docs/PLUGIN_DEVELOPMENT.md`](PLUGIN_DEVELOPMENT.md)
- View WIT interface: [`crates/omnitak-plugin-api/wit/plugin.wit`](../crates/omnitak-plugin-api/wit/plugin.wit)
- Browse examples: [`examples/plugins/`](../examples/plugins/)

## Common Issues

**Build fails:** Make sure you have WASM target installed
```bash
rustup target add wasm32-wasip1
```

**Plugin won't load:** Check WIT path in `wit_bindgen::generate!()`

**Timeout errors:** Reduce execution time or optimize code

## Help

- GitHub Issues: https://github.com/engindearing-projects/omniTAK/issues
- Documentation: `docs/PLUGIN_DEVELOPMENT.md`

---

**Happy plugin building! 🚀**
