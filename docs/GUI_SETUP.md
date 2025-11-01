# OmniTAK GUI Setup Guide

This guide will help you build and run the OmniTAK GUI on Ubuntu and macOS.

## Prerequisites

### Ubuntu/Debian

```bash
# Install required system dependencies for GUI development
sudo apt-get update
sudo apt-get install -y \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libssl-dev \
    libfontconfig1-dev \
    pkg-config \
    protobuf-compiler \
    build-essential

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### macOS

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install required dependencies
brew install protobuf

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Building the GUI

```bash
# Clone the repository (if you haven't already)
git clone https://github.com/engindearing-projects/omniTAK.git
cd omniTAK

# Build the GUI in release mode (optimized)
cargo build --bin omnitak-gui --release

# Or build in debug mode (faster compilation, slower runtime)
cargo build --bin omnitak-gui
```

## Running the GUI

### From Source

```bash
# Run in release mode
cargo run --bin omnitak-gui --release

# Or run in debug mode
cargo run --bin omnitak-gui
```

### From Binary

After building, the binary will be located at:
- `target/release/omnitak-gui` (release build)
- `target/debug/omnitak-gui` (debug build)

```bash
# Run the release binary directly
./target/release/omnitak-gui
```

## GUI Features

### 1. Dashboard
- View system overview and key metrics
- Monitor active and failed connections
- Track message throughput and data transfer
- See connection uptime and status

### 2. Connections Management
- **Add new TAK servers**: Click "Add Server" button
- **Edit existing servers**: Click "Edit" on any server card
- **Remove servers**: Click "Delete" to remove a server
- **Configure TLS**: Enable TLS and provide certificate paths
- **View connection status**: Real-time status updates (Connected/Disconnected/Reconnecting/Failed)

### 3. Message Log
- View real-time CoT message stream
- Filter messages by server, type, or content
- Auto-scroll to latest messages
- Clear message log

### 4. Settings
- View application information
- Check configuration summary

## Adding Your First Server

1. Launch the GUI: `cargo run --bin omnitak-gui --release`
2. Navigate to the **Connections** tab
3. Click **"Add Server"**
4. Fill in the server details:
   - **Name**: Give your server a descriptive name (e.g., "TAK Server 1")
   - **Host**: IP address or hostname (e.g., "192.168.1.100")
   - **Port**: Server port (default 8089 for TLS)
   - **Protocol**: Select TLS (recommended for secure connections)
   - **Enabled**: Check to enable the connection

5. **Configure TLS** (if using TLS protocol):
   - Check "Enable TLS"
   - **CA Certificate**: Path to CA certificate (e.g., `/path/to/certs/ca.pem`)
   - **Client Certificate**: Path to client certificate (e.g., `/path/to/certs/client.pem`)
   - **Client Key**: Path to client private key (e.g., `/path/to/certs/client-key.pem`)
   - **Verify Certificate**: Check to verify server certificate
   - **Server Name (SNI)**: Optional server name for SNI

6. Click **Save**

## Certificate Management

### Using ADB to Extract Certificates

If you have TAK certificates on an Android device with ATAK installed:

```bash
# Build the ADB setup tool
cargo build --bin omnitak-adb-setup --release

# Run the tool to extract certificates
./target/release/omnitak-adb-setup --output config/config.yaml --cert-dir certs/

# The certificates will be extracted to the certs/ directory
# You can then use these paths in the GUI
```

### Manual Certificate Setup

1. Obtain your TAK certificates (usually provided by your TAK server administrator)
2. Place them in a secure directory (e.g., `~/.omnitak/certs/`)
3. When adding a server in the GUI, browse to these certificate files

## Troubleshooting

### GUI Won't Start

**Ubuntu**: Missing system libraries
```bash
# Install missing dependencies
sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

**macOS**: Permission issues
```bash
# Make sure the binary is executable
chmod +x target/release/omnitak-gui
```

### Build Errors

**Network issues with crates.io**:
```bash
# Clear cargo cache and retry
rm -rf ~/.cargo/registry
cargo build --bin omnitak-gui --release
```

**Rust version too old**:
```bash
# Update Rust to the latest version
rustup update
```

### Connection Issues

1. **TLS handshake failures**:
   - Verify certificate paths are correct
   - Ensure certificates are in PEM format
   - Check that the CA certificate matches the server's certificate

2. **Cannot connect to server**:
   - Verify host and port are correct
   - Check firewall rules
   - Ensure the server is running and accessible

3. **Status shows "Failed"**:
   - Check the error message in the connection card
   - Verify authentication credentials
   - Review the application logs

## Running Alongside OpenTAKServer

OmniTAK GUI can run on the same server as OpenTAKServer:

```bash
# On Ubuntu server with OpenTAKServer
# Install X server for GUI support (if headless)
sudo apt-get install -y xvfb

# Run GUI with virtual display
Xvfb :1 -screen 0 1024x768x24 &
export DISPLAY=:1
cargo run --bin omnitak-gui --release

# Or use X forwarding over SSH
ssh -X user@server
cargo run --bin omnitak-gui --release
```

For headless servers, consider using the existing web interface or REST API instead.

## Platform-Specific Notes

### Ubuntu
- Wayland users: egui works best with X11. Use `GDK_BACKEND=x11` if needed
- HiDPI displays: The GUI auto-scales based on system settings

### macOS
- The GUI supports both Intel and Apple Silicon (M1/M2/M3)
- Runs natively on both architectures
- Supports macOS 10.15 (Catalina) and later

## Next Steps

- Explore the **Dashboard** to monitor your connections
- Add multiple TAK servers in the **Connections** tab
- View real-time messages in the **Messages** tab
- Refer to the main [README](../README.md) for overall project documentation
- Check [ADB_SETUP.md](./ADB_SETUP.md) for certificate extraction guide

## Getting Help

If you encounter issues:
1. Check the application logs (displayed in terminal)
2. Review this troubleshooting guide
3. Open an issue on GitHub: https://github.com/engindearing-projects/omniTAK/issues
4. Refer to the existing documentation in the `docs/` directory

## Contributing

Contributions are welcome! Areas for improvement:
- File picker integration for certificate selection
- Real-time API integration with running OmniTAK server
- Additional metrics and visualization
- Platform-specific optimizations

See the [Contributing Guide](../CONTRIBUTING.md) for more information.
