# Example Filter Plugin

A simple example message filter plugin for omniTAK that demonstrates how to create WASM plugins for filtering Cursor-on-Target (CoT) XML messages.

## Overview

This plugin demonstrates the basic structure and functionality of an omniTAK message filter plugin. It:

- Accepts CoT XML messages as strings
- Scans for hostile keywords (hostile, enemy, threat, attack, danger)
- Tags messages containing hostile keywords with warning attributes
- Passes clean messages through unchanged
- Provides plugin metadata (name, version, description)

## Structure

```
example-filter/
├── Cargo.toml              # Package manifest with WASM configuration
├── wit/
│   └── message-filter.wit  # WIT interface definition
├── src/
│   ├── lib.rs             # Main plugin implementation
│   ├── bindings.rs        # WIT bindings generation
│   └── host.rs            # Host function wrappers
└── README.md              # This file
```

## Building

### Prerequisites

- Rust toolchain (1.91 or later recommended)
- `wasm32-wasip2` target installed

Install the WASM target if you haven't already:

```bash
rustup target add wasm32-wasip2
```

### Build Command

Build the plugin for WASM:

```bash
cd /Users/iesouskurios/omniTAK/plugins/example-filter
cargo build --target wasm32-wasip2 --release
```

The compiled WASM module will be located at:
```
target/wasm32-wasip2/release/example_filter.wasm
```

## Functionality

### Filter Logic

1. **Empty Check**: Rejects empty messages with an error
2. **Keyword Scanning**: Scans for hostile keywords (case-insensitive)
3. **Tagging**: If hostile keywords are found:
   - Adds attributes to the `<event>` tag indicating detection
   - Logs the detected keywords using structured logging
   - Returns the modified message
4. **Pass-through**: If no keywords found, returns original message unchanged

### Example Input/Output

**Input (clean message):**
```xml
<event version="2.0" uid="test-001" type="a-f-G" how="m-g">
```

**Output:**
```xml
<event version="2.0" uid="test-001" type="a-f-G" how="m-g">
```

**Input (hostile message):**
```xml
<event version="2.0" uid="test-002" type="a-h-G" how="m-g">hostile contact detected</event>
```

**Output:**
```xml
<event hostile_detected="true" keywords="hostile" version="2.0" uid="test-002" type="a-h-G" how="m-g">hostile contact detected</event>
```

## Plugin Interface

The plugin implements the `message-filter` interface defined in WIT:

### Exported Functions

- `filter-message(cot-xml: string) -> result<string, string>`
  - Main filtering function
  - Returns modified message or error

- `get-name() -> string`
  - Returns: "Example Filter Plugin"

- `get-version() -> string`
  - Returns current version from Cargo.toml

- `get-description() -> string`
  - Returns plugin description

### Host Functions Available

- `host::log(message: string)`
  - Log simple messages to host

- `host::log-structured(message: string, properties: list<tuple<string, string>>)`
  - Log structured messages with metadata

## Extending This Plugin

To create your own filter plugin:

1. Copy this directory structure
2. Modify the filter logic in `src/lib.rs`
3. Update the keyword list or implement custom filtering logic
4. Adjust metadata (name, version, description)
5. Build with the same commands

## Testing

Unit tests are not included in this example because they require a WASM runtime with host function implementations. Testing should be performed through integration tests with the actual omniTAK host runtime that loads and executes the plugin.

## License

MIT OR Apache-2.0
