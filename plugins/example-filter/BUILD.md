# Build Instructions for Example Filter Plugin

## Quick Start

```bash
cd /Users/iesouskurios/omniTAK/plugins/example-filter
cargo build --target wasm32-wasip2 --release
```

The compiled WASM module will be at:
```
target/wasm32-wasip2/release/example_filter.wasm
```

## Prerequisites

### Rust Toolchain

Ensure you have Rust installed (1.91 or later recommended):
```bash
rustc --version
```

If Rust is not installed, get it from https://rustup.rs/

### WASM Target

Install the `wasm32-wasip2` target:
```bash
rustup target add wasm32-wasip2
```

Verify installation:
```bash
rustup target list | grep wasm32-wasip2
```

## Build Commands

### Release Build (Optimized)

```bash
cargo build --target wasm32-wasip2 --release
```

Output: `target/wasm32-wasip2/release/example_filter.wasm` (~107KB)

### Debug Build

```bash
cargo build --target wasm32-wasip2
```

Output: `target/wasm32-wasip2/debug/example_filter.wasm` (larger, includes debug symbols)

### Clean Build

```bash
cargo clean
cargo build --target wasm32-wasip2 --release
```

## Verifying the Build

Check that the WASM file was created:
```bash
ls -lh target/wasm32-wasip2/release/example_filter.wasm
```

Expected output shows a file around 107KB in size.

## Troubleshooting

### Error: "can't find crate for `core`"

This means the wasm32-wasip2 target is not installed:
```bash
rustup target add wasm32-wasip2
```

### Error: "workspace.members" or "workspace.exclude"

If the plugin is in a directory that's part of a Rust workspace, the Cargo.toml already includes a `[workspace]` section to make it standalone.

### Error: "failed to read path for WIT"

Verify the WIT file exists at the correct path:
```bash
ls -l wit/message-filter.wit
```

The path in `src/bindings.rs` should be `wit/message-filter.wit` (relative to the crate root).

## Advanced Options

### Optimizing for Size

Add to Cargo.toml:
```toml
[profile.release]
opt-level = "z"
lto = true
strip = true
```

Then rebuild:
```bash
cargo build --target wasm32-wasip2 --release
```

### Checking Dependencies

View the dependency tree:
```bash
cargo tree --target wasm32-wasip2
```

### Building with Verbose Output

```bash
cargo build --target wasm32-wasip2 --release --verbose
```

## Deployment

Copy the compiled WASM file to your omniTAK plugins directory:
```bash
cp target/wasm32-wasip2/release/example_filter.wasm /path/to/omnitak/plugins/
```

## Development Workflow

1. Make changes to `src/lib.rs`
2. Build: `cargo build --target wasm32-wasip2 --release`
3. Deploy the updated WASM file
4. Test with the omniTAK host runtime
5. Iterate

## CI/CD Integration

Example GitHub Actions workflow snippet:

```yaml
- name: Install Rust
  uses: actions-rs/toolchain@v1
  with:
    toolchain: stable
    target: wasm32-wasip2

- name: Build Plugin
  run: |
    cd plugins/example-filter
    cargo build --target wasm32-wasip2 --release

- name: Upload Artifact
  uses: actions/upload-artifact@v2
  with:
    name: example-filter-wasm
    path: plugins/example-filter/target/wasm32-wasip2/release/example_filter.wasm
```
