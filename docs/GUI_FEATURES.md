# OmniTAK GUI Feature Guide

Complete guide to using the OmniTAK desktop GUI application.

## Table of Contents
1. [Getting Started](#getting-started)
2. [Dashboard](#dashboard)
3. [Connection Management](#connection-management)
4. [Message Viewer](#message-viewer)
5. [Settings & Configuration](#settings--configuration)
6. [Status Bar & Notifications](#status-bar--notifications)
7. [Keyboard Shortcuts](#keyboard-shortcuts)
8. [Tips & Tricks](#tips--tricks)

---

## Getting Started

### Launching the GUI

```bash
# From the project root
cargo run --bin omnitak-gui --release

# Or run the compiled binary
./target/release/omnitak-gui
```

The GUI will remember your settings and server configurations between sessions.

### First Time Setup

1. Launch the application
2. Navigate to the **Connections** tab
3. Click **"➕ Add Server"**
4. Configure your first TAK server (see [Adding a Server](#adding-a-server))
5. Click **"▶ Connect"** to establish connection

---

## Dashboard

The Dashboard provides a real-time overview of your OmniTAK system.

### Metrics Cards

**Active Connections** (Green)
- Number of currently connected TAK servers
- Updates in real-time

**Failed Connections** (Red)
- Number of servers that failed to connect
- Indicates issues requiring attention

**Total Servers** (Blue)
- Total number of configured servers
- Includes enabled and disabled servers

**Messages Received/Sent** (Light Blue/Green)
- Total count of CoT messages
- Tracks overall message throughput

**Data Transfer** (Blue/Green)
- Bytes received and sent
- Automatically formatted (B, KB, MB, GB, TB)

### Connection Status Table

Displays all configured servers with:
- **Server Name**: Identifier for the connection
- **Status**: Current connection state (color-coded)
  - 🟢 Connected
  - ⚪ Disconnected
  -🟡 Reconnecting
  - 🔴 Failed
- **Messages**: Sent (↑) and received (↓) counts
- **Uptime**: How long the connection has been active

---

## Connection Management

The Connections tab is where you manage all TAK server configurations.

### Adding a Server

1. Click **"➕ Add Server"** button
2. Fill in the server details:

#### Basic Configuration
- **Name**: Unique identifier (e.g., "Production TAK Server")
- **Host**: IP address or hostname (e.g., "192.168.1.100")
- **Port**: Server port number (default: 8089 for TLS)
- **Protocol**: Select from:
  - TCP - Unencrypted TCP connection
  - UDP - Unreliable datagram transport
  - TLS - Encrypted TCP (recommended)
  - WebSocket - HTTP-upgraded connection
- **Enabled**: Check to enable the connection

#### TLS Configuration
If using TLS protocol:

1. Check **"Enable TLS"**
2. Configure certificates:
   - **CA Certificate**: Path to certificate authority file
     - Example: `/home/user/.omnitak/certs/ca.pem`
   - **Client Certificate**: Path to client cert (optional for mutual TLS)
     - Example: `/home/user/.omnitak/certs/client.pem`
   - **Client Key**: Path to client private key
     - Example: `/home/user/.omnitak/certs/client-key.pem`
   - **Verify Certificate**: Enable to verify server's certificate
   - **Server Name (SNI)**: Server name for SNI (optional)

3. Click **"Save"**

### Editing a Server

1. Find the server in the list
2. Click **"✏ Edit"**
3. Modify the configuration
4. Click **"Save"**

Note: Editing an enabled server will disconnect and reconnect with new settings.

### Connecting/Disconnecting

Each server card has connection controls:

**▶ Connect Button** (Appears when disconnected)
- Initiates connection to the TAK server
- Button becomes **⏸ Disconnect** when connected

**⏸ Disconnect Button** (Appears when connected)
- Gracefully disconnects from the server
- Returns to **▶ Connect** state

**Status Indicators**:
- **● Connected** (Green): Successfully connected
- **● Disconnected** (Gray): Not connected
- **● Reconnecting** (Yellow): Attempting to reconnect
- **● Failed** (Red): Connection failed

### Connection Metrics

Each server card displays:
- **Messages**: ↓ [received count] / ↑ [sent count]
- **Data Transfer**: Formatted byte counts
- **Reconnect Attempts**: Number of retry attempts
- **Last Error**: Error message if connection failed
- **🔒 TLS Enabled**: Indicates secure connection

### Deleting a Server

1. Click **"🗑 Delete"** on the server card
2. Server will be disconnected (if connected) and removed
3. Configuration is immediately deleted

⚠️ **Warning**: Deletion is permanent. Export configuration first if you want to keep a backup.

---

## Message Viewer

The Messages tab displays real-time CoT message logs.

### Features

**Message Table**
- **Timestamp**: When the message was received (HH:MM:SS format)
- **Server**: Which server received the message
- **Type**: Message type/category (color-coded tag)
- **Content**: Message content (truncated if long)

**Filter Box**
- Type keywords to filter messages
- Searches across server name, message type, and content
- Updates results in real-time

**Controls**
- **Auto-scroll**: Automatically scroll to newest messages
- **🗑 Clear Log**: Remove all messages from the log

**Message Retention**
- Keeps last 1,000 messages automatically
- Older messages are automatically removed
- Message count displayed above table

### Usage Tips

**Finding Specific Messages**
```
Filter by server: "TAK-1"
Filter by type: "position"
Filter by content: "friendly"
```

**Managing Log Size**
- Click "Clear Log" to remove all messages
- Export important logs before clearing

---

## Settings & Configuration

The Settings tab provides application configuration and info.

### Application Information

- **Version**: Current OmniTAK version
- **License**: Project license information
- **Description**: Brief about OmniTAK

### Configuration Summary

- **Total Servers**: Number of configured servers
- **Active Connections**: Currently connected servers
- **Message Log Size**: Current message log entry count

### Import/Export Configuration

#### Exporting Configuration

**Export to YAML**
1. Click **"📤 Export to YAML"**
2. Configuration saved to `omnitak_config.yaml` in current directory
3. Success message appears in status bar

**Export to JSON**
1. Click **"📤 Export to JSON"**
2. Configuration saved to `omnitak_config.json` in current directory
3. Success message appears in status bar

**What Gets Exported**:
- All server configurations
- Server names, hosts, ports, protocols
- TLS settings (certificate paths)
- Enable/disable states
- Connection timeouts and retry settings

#### Importing Configuration

**Import from File**
1. Place your config file in the current directory as `import_config.yaml` (or `.json`)
2. Click **"🔄 Import from import_config.yaml"**
3. Servers are added to existing configuration
4. Success message shows number of imported servers

**Import Behavior**:
- Imported servers are **added** to existing servers
- Does not remove or replace current servers
- Validates configuration before importing
- Shows errors if validation fails

**Supported Formats**:
- YAML (`.yaml`, `.yml`)
- JSON (`.json`)

#### Configuration File Format

**YAML Example**:
```yaml
servers:
  - name: "Production TAK"
    host: "tak.example.com"
    port: 8089
    protocol: tls
    enabled: true
    tls:
      ca_cert_path: "/path/to/ca.pem"
      client_cert_path: "/path/to/client.pem"
      client_key_path: "/path/to/key.pem"
      verify_cert: true
      server_name: "tak.example.com"
```

**JSON Example**:
```json
{
  "servers": [
    {
      "name": "Production TAK",
      "host": "tak.example.com",
      "port": 8089,
      "protocol": "tls",
      "enabled": true,
      "tls": {
        "ca_cert_path": "/path/to/ca.pem",
        "client_cert_path": "/path/to/client.pem",
        "client_key_path": "/path/to/key.pem",
        "verify_cert": true,
        "server_name": "tak.example.com"
      }
    }
  ]
}
```

---

## Status Bar & Notifications

The status bar appears at the bottom of the window and shows real-time notifications.

### Notification Types

**ℹ Info (Blue)**
- General information
- Operation in progress
- Example: "Connecting to TAK Server 1..."

**✓ Success (Green)**
- Operation completed successfully
- Example: "Configuration exported to omnitak_config.yaml"

**⚠ Warning (Yellow)**
- Caution or important notice
- Non-critical issues
- Example: "Importing will add to existing servers"

**✗ Error (Red)**
- Operation failed
- Critical issues requiring attention
- Example: "Export failed: Permission denied"

### Message Duration

- Info: 3-5 seconds
- Success: 3-5 seconds
- Warning: 5-10 seconds
- Error: 10 seconds

Messages automatically disappear after the duration expires.

---

## Keyboard Shortcuts

### Navigation
- **Tab**: Navigate between UI elements
- **Enter**: Activate focused button
- **Escape**: Close dialogs

### Dialog Controls
- **Escape**: Cancel and close dialog
- **Enter**: Save (when in text fields)

---

## Tips & Tricks

### Connection Management

**Testing New Servers**
1. Add server with "Enabled" unchecked
2. Click "Edit" to review settings
3. Check "Enabled" and save
4. Click "▶ Connect" to test

**Organizing Servers**
- Use descriptive names (e.g., "Prod TAK - HQ", "Test TAK - Lab")
- Use consistent naming conventions
- Export configurations regularly as backups

**TLS Certificate Management**
- Store certificates in a dedicated directory (e.g., `~/.omnitak/certs/`)
- Use absolute paths for certificate files
- Keep backups of certificates
- Test certificate paths in the dialog before saving

### Performance

**Managing Message Log**
- Clear log periodically to improve performance
- Disable auto-scroll when searching old messages
- Use filters to reduce displayed messages

**Connection Optimization**
- Disable unused servers instead of deleting
- Monitor "Failed" connections and investigate errors
- Check reconnect attempt counts for unstable connections

### Configuration Backups

**Regular Exports**
1. Export configuration weekly (or before major changes)
2. Name files with dates: `omnitak_config_2024-11-01.yaml`
3. Store backups in a safe location
4. Test imports periodically

**Configuration Management**
```bash
# Backup current configuration
# (Click "Export to YAML" in GUI)

# Keep versioned backups
cp omnitak_config.yaml backups/config_$(date +%Y%m%d).yaml

# Restore from backup
cp backups/config_20241101.yaml import_config.yaml
# (Click "Import" in GUI)
```

### Troubleshooting

**Connection Fails**
1. Check status bar for error message
2. Verify host and port are correct
3. Ensure TAK server is running
4. Check firewall rules
5. Review certificate paths if using TLS

**No Messages Appearing**
1. Verify connection status is "Connected"
2. Check that server is sending CoT messages
3. Ensure message log hasn't reached limit
4. Clear filters in message viewer

**Import Fails**
1. Check file format (YAML or JSON)
2. Validate file syntax
3. Ensure all required fields are present
4. Check certificate paths exist

**UI Performance Issues**
1. Clear message log
2. Disable unused connections
3. Reduce auto-refresh frequency (in future releases)

---

## Best Practices

### Security
- Always use TLS protocol for production servers
- Store certificates in protected directories
- Use strong certificate verification
- Don't share private keys

### Reliability
- Monitor "Reconnect attempts" counter
- Investigate recurring connection failures
- Keep backup configurations
- Test configuration changes on disabled servers first

### Maintenance
- Export configuration before major changes
- Clear message logs periodically
- Review connection metrics regularly
- Update server configurations when certificates rotate

---

## Getting Help

If you encounter issues:

1. Check the status bar for error messages
2. Review this feature guide
3. Consult [GUI_SETUP.md](GUI_SETUP.md) for installation issues
4. Check application logs in the terminal
5. Open an issue on GitHub with:
   - OmniTAK version
   - Operating system
   - Error messages
   - Steps to reproduce

## Next Steps

- Explore the [GUI Setup Guide](GUI_SETUP.md) for installation
- Review the [Changelog](../CHANGELOG_GUI.md) for latest features
- Check the main [README](../README.md) for project overview

---

**Happy TAK Aggregating! 🎯**
