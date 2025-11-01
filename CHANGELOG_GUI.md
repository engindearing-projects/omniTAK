# OmniTAK GUI Changelog

## Version 0.2.1 (Latest) - Enhanced GUI Features

### New Features

#### Backend Service Integration
- **Async Backend Service**: Added fully integrated backend service for managing real TAK server connections
- **Command-Event Architecture**: Implemented command/event pattern for clean separation between UI and backend
- **Connection Management**: Backend handles connection lifecycle (connect, disconnect, reconnect)
- **Real-time Status Updates**: Connection status updates flow from backend to UI automatically
- **Metrics Aggregation**: Backend collects and aggregates metrics across all connections

#### User Interface Enhancements
- **Connect/Disconnect Buttons**: Added interactive buttons to start and stop connections from the Connections tab
- **Status Bar**: Bottom status bar shows real-time notifications and feedback
  - Info messages (blue)
  - Success messages (green)
  - Warning messages (yellow)
  - Error messages (red)
  - Auto-expiring messages (configurable duration)
- **Connection Controls**: Per-server connect/disconnect with visual state indicators
- **Improved Button Layout**: Better organization of action buttons in connection cards

#### Configuration Management
- **Export Configuration**:
  - Export to YAML format
  - Export to JSON format
  - Validation before export
  - One-click export from Settings tab
- **Import Configuration**:
  - Import from YAML files
  - Import from JSON files
  - Validation before import
  - Option to add or replace servers
  - Detailed import feedback
- **Config File Support**: Full serialization/deserialization of server configurations

#### Error Handling & Notifications
- **User-Friendly Notifications**: All operations provide clear feedback
- **Error Messages**: Detailed error information for troubleshooting
- **Success Confirmations**: Positive feedback for completed operations
- **Warning Alerts**: Caution messages for important operations

### Technical Improvements

#### Architecture
- **Backend Module** (`backend.rs`):
  - Async worker thread with Tokio runtime
  - Command channel for UI → Backend communication
  - Event channel for Backend → UI updates
  - Graceful shutdown handling
  - Thread-safe state management

- **Config I/O Module** (`config_io.rs`):
  - Format detection from file extension
  - YAML and JSON support
  - Configuration validation
  - Error handling with anyhow

#### Code Organization
- Separated backend logic from UI rendering
- Clean async/sync boundaries
- Proper state management with Arc<Mutex<>>
- Event-driven updates for better performance

#### Dependencies
- Added `async-channel` for async communication
- Added `serde_yaml` for YAML config support
- Leveraged `tokio` runtime for backend operations

### User Experience
- **Immediate Feedback**: All button clicks provide instant visual feedback
- **Non-Blocking UI**: Backend operations don't freeze the interface
- **Auto-Refresh**: UI updates every second to show latest status
- **Status Indicators**: Clear visual cues for connection states
- **Helpful Messages**: Informative status bar messages guide users

### Bug Fixes
- Fixed server removal to disconnect before deleting
- Improved state synchronization between UI and backend
- Added proper cleanup on application shutdown

### Documentation
- Added comprehensive inline documentation
- Created `CHANGELOG_GUI.md` for tracking changes
- Updated README with new features
- Enhanced code comments for maintainability

## Version 0.2.0 - Initial GUI Release

### Core Features
- Native desktop GUI using egui/eframe
- Dashboard with system overview
- Connection management (add/edit/delete servers)
- Message log viewer
- Settings panel
- TLS certificate configuration
- Cross-platform support (Ubuntu/macOS)

### Components
- Dashboard tab with metrics
- Connections tab for server management
- Messages tab for log viewing
- Settings tab for application configuration
- Server dialog for add/edit operations

---

## Upcoming Features (Roadmap)

### Next Release (0.3.0)
- [ ] File picker integration for certificate selection
- [ ] Connection testing before saving
- [ ] Live API integration with running OmniTAK server
- [ ] Enhanced metrics visualization (charts/graphs)
- [ ] Dark/light theme toggle
- [ ] Connection templates

### Future Releases
- [ ] System tray integration
- [ ] Auto-start connections on launch
- [ ] Advanced filtering for message log
- [ ] Export message logs
- [ ] Connection grouping
- [ ] Backup/restore functionality
- [ ] Windows platform support
- [ ] Connection health checks
- [ ] Network latency monitoring
- [ ] Message statistics dashboard

---

## Development Notes

### Backend Service Architecture

The backend service runs in a separate thread with its own Tokio runtime:

```
┌─────────────┐         Commands          ┌──────────────┐
│             │ ──────────────────────────> │              │
│  GUI Thread │                             │    Backend   │
│   (egui)    │ <────────────────────────── │    Worker    │
│             │         Events              │   (Tokio)    │
└─────────────┘                             └──────────────┘
```

**Commands** (UI → Backend):
- `Connect(ServerConfig)`
- `Disconnect(String)`
- `UpdateConfig(ServerConfig)`
- `Shutdown`

**Events** (Backend → UI):
- `StatusUpdate(String, ConnectionMetadata)`
- `MessageReceived(MessageLog)`
- `Error(String, String)`
- `MetricsUpdate(AppMetrics)`

### Configuration Format

YAML Example:
```yaml
servers:
  - name: "TAK Server 1"
    host: "192.168.1.100"
    port: 8089
    protocol: tls
    enabled: true
    tls:
      ca_cert_path: "/path/to/ca.pem"
      client_cert_path: "/path/to/client.pem"
      client_key_path: "/path/to/key.pem"
      verify_cert: true
```

JSON Example:
```json
{
  "servers": [
    {
      "name": "TAK Server 1",
      "host": "192.168.1.100",
      "port": 8089,
      "protocol": "tls",
      "enabled": true,
      "tls": {
        "ca_cert_path": "/path/to/ca.pem",
        "client_cert_path": "/path/to/client.pem",
        "client_key_path": "/path/to/key.pem",
        "verify_cert": true
      }
    }
  ]
}
```

---

## Contributing

We welcome contributions! Areas for improvement:
- File picker integration
- Enhanced metrics visualization
- Platform-specific optimizations
- Performance improvements
- Additional configuration options
- UI/UX enhancements

See the main [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT OR Apache-2.0
