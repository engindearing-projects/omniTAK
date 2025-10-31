# OmniTAK GUI

Native desktop GUI for the OmniTAK TAK server aggregator, built with Rust and egui/eframe.

## Features

### Dashboard Tab
- System overview with key metrics
- Active/failed connection counts
- Total message throughput (sent/received)
- Data transfer statistics
- Connection status table with uptime tracking

### Connections Tab
- Add, edit, and remove TAK server connections
- Visual connection status indicators
- Per-connection metrics (messages, bytes, latency)
- TLS certificate management
- Connection health monitoring
- Enable/disable individual connections

### Messages Tab
- Real-time CoT message log viewer
- Message filtering by server/content/type
- Auto-scroll capability
- Message history with timestamps
- Clear log functionality

### Settings Tab
- Application information
- Configuration overview
- Future expansion for advanced settings

## Architecture

The GUI is built using:
- **eframe**: Cross-platform GUI framework
- **egui**: Immediate-mode GUI library
- Pure Rust with no web dependencies
- Integration with existing OmniTAK core libraries

## Building

```bash
# Build the GUI application
cargo build --bin omnitak-gui --release

# Run the GUI
cargo run --bin omnitak-gui
```

## Running

### Linux (Ubuntu)
```bash
./target/release/omnitak-gui
```

### macOS
```bash
./target/release/omnitak-gui
```

### Windows
```bash
.\target\release\omnitak-gui.exe
```

## Platform Support

- ✅ Ubuntu/Linux (Primary target)
- ✅ macOS (Primary target)
- ⏳ Windows (Future support)

## Server Connection Dialog

The server connection dialog allows configuring:
- **Basic Settings**:
  - Server name (identifier)
  - Host and port
  - Protocol (TCP/UDP/TLS/WebSocket)
  - Enabled/disabled state

- **TLS Configuration**:
  - CA certificate path
  - Client certificate path (optional)
  - Client key path (optional)
  - Server name for SNI
  - Certificate verification toggle

## Integration

The GUI is designed to work alongside the existing OmniTAK server components:
- Uses the same configuration types from `omnitak-core`
- Can display connection metadata from `omnitak-pool`
- Shows CoT messages from `omnitak-cot`
- Future integration with REST API from `omnitak-api`

## Future Enhancements

- [ ] File picker integration for certificate selection
- [ ] Real-time connection to running OmniTAK server via API
- [ ] Configuration import/export (YAML)
- [ ] Connection testing before saving
- [ ] Message filtering by affiliation type
- [ ] Graphical metrics (charts/graphs)
- [ ] Dark/light theme toggle
- [ ] System tray integration
- [ ] Auto-start connections on launch
- [ ] Connection templates
- [ ] Backup/restore functionality

## Development

The GUI crate is organized as follows:

```
crates/omnitak-gui/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs          # Main application logic and state
│   └── ui/
│       ├── mod.rs
│       ├── dashboard.rs    # Dashboard view
│       ├── connections.rs  # Connection management
│       ├── messages.rs     # Message log viewer
│       ├── settings.rs     # Settings view
│       └── server_dialog.rs # Server add/edit dialog
```

## License

MIT OR Apache-2.0
