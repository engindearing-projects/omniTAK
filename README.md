# OmniTAK

**Military-Grade TAK Server Aggregator** | **Status: Beta (v0.2.0)**

OmniTAK is a high-performance, memory-safe TAK (Team Awareness Kit) server aggregator written in Rust. It acts as a multi-protocol client that connects to multiple TAK servers simultaneously, aggregates CoT (Cursor on Target) messages, and intelligently routes them based on configurable filters.

## Status

**Beta Release (v0.2.0)** - Core functionality complete and tested with:
- ✅ TAK Server (official) - TLS 1.2 with client certificates
- ✅ TAKy - Basic TCP connections
- 🚧 FreeTAKServer - Testing in progress
- 🚧 OpenTAKServer - Testing in progress

## Features

### Core Capabilities
- **Multi-Protocol Support**: TCP, UDP, TLS, WebSocket, and UDP Multicast
- **High Performance**: Handle 10,000+ concurrent connections with <1ms latency
- **Military-Grade Security**: TLS 1.3, client certificates, memory-safe implementation
- **MIL-STD-2525 Support**: Full affiliation and symbology parsing
- **Intelligent Filtering**: Route messages by affiliation, team, group, or geographic bounds
- **Message Deduplication**: Automatic duplicate detection across sources
- **Real-Time API**: REST API with WebSocket streaming
- **Production Ready**: Comprehensive metrics, health checks, audit logging

### Performance Targets
- **Throughput**: 100,000+ messages/second
- **Latency**: <1ms routing latency (p99)
- **Connections**: 10,000+ concurrent TAK servers
- **Memory**: <50MB per 1,000 connections
- **Parsing**: <2μs per CoT message

## Architecture

OmniTAK is built as a Rust workspace with modular crates:

```
omnitak/
├── omnitak-core        # Core types, config, errors
├── omnitak-cot         # CoT parser (XML & Protobuf)
├── omnitak-client      # Protocol clients (TCP/UDP/TLS/WS)
├── omnitak-filter      # MIL-STD-2525 filtering & routing
├── omnitak-pool        # Connection pool manager
├── omnitak-cert        # Certificate management
└── omnitak-api         # REST API & WebSocket server
```

### Technology Stack
- **Language**: Rust 1.90+ (2021 edition)
- **Async Runtime**: Tokio
- **TLS**: Rustls (memory-safe, no OpenSSL) - TLS 1.2/1.3 compatible
- **Web Framework**: Axum
- **Serialization**: Serde, quick-xml, Protobuf (prost)
- **Metrics**: Prometheus-compatible
- **Logging**: Tracing with structured logs

## Easy Setup with ADB (Recommended)

**NEW in v0.2.0**: Automatically pull TAK certificates and configuration from your Android device!

If you already have ATAK/WinTAK configured on an Android device, you can use the **ADB Setup Tool** to automatically extract certificates and generate your OmniTAK configuration.

### Quick Setup with ADB

```bash
# 1. Build OmniTAK with the ADB setup tool
cargo build --release

# 2. Connect your Android device via USB and enable USB debugging

# 3. Run the ADB setup tool
./target/release/omnitak-adb-setup --output config/config.yaml --cert-dir certs

# 4. Convert certificates to PEM format
./scripts/convert-p12-to-pem.sh certs/*.p12 certs

# 5. Start OmniTAK
cargo run --release -- --config config/config.yaml
```

**That's it!** The tool will:
- ✅ Detect your Android device
- ✅ Extract TAK certificates from the device
- ✅ Pull TAK server configuration (addresses, ports)
- ✅ Generate a ready-to-use `config.yaml` file

See the **[ADB Setup Guide](docs/ADB_SETUP.md)** for detailed instructions, troubleshooting, and manual setup options.

---

## TAK Server Certificate Setup (Manual)

**IMPORTANT**: Official TAK Server uses TLS 1.2 with client certificates. You must properly format your certificates for compatibility.

### Generating Client Certificates

On your TAK Server, generate a client certificate:

```bash
cd /opt/tak/certs
sudo STATE="YourState" CITY="YourCity" ORGANIZATIONAL_UNIT="TAKServer" \
  bash -c './makeCert.sh client omnitak'
```

This creates:
- `omnitak.p12` - PKCS12 bundle (password: atakatak by default)
- `omnitak.pem` - Client certificate
- `omnitak.key` - Encrypted private key
- `ca.pem` - Certificate Authority

### Converting Certificates for Rustls

**Critical**: Rustls requires traditional RSA format, not encrypted PKCS8. Convert using:

```bash
# Extract certificate (already in correct format)
openssl pkcs12 -in omnitak.p12 -out omnitak.pem -clcerts -nokeys \
  -passin pass:atakatak -legacy

# Extract and convert key to traditional RSA format
openssl pkcs8 -in omnitak.key -out omnitak-rsa.key \
  -passin pass:atakatak -traditional

# Extract CA certificate
openssl pkcs12 -in omnitak.p12 -out ca.pem -cacerts -nokeys \
  -passin pass:atakatak -legacy
```

**Why `-traditional` is required**: TAK Server uses TLS 1.2 which requires traditional RSA key format. Modern PKCS8 format will cause handshake failures.

### Certificate File Locations

Place your certificates in a secure directory:

```bash
mkdir -p /path/to/certs
cp omnitak.pem /path/to/certs/
cp omnitak-rsa.key /path/to/certs/  # Use the -rsa version!
cp ca.pem /path/to/certs/
chmod 600 /path/to/certs/*.key
chmod 644 /path/to/certs/*.pem
```

### Configuration Example

```yaml
servers:
  - id: tak-server-1
    address: "takserver.example.com:8089"
    protocol: tls
    tls:
      cert_path: "/path/to/certs/omnitak.pem"
      key_path: "/path/to/certs/omnitak-rsa.key"  # Traditional RSA format
      ca_path: "/path/to/certs/ca.pem"
```

### Troubleshooting TLS Connections

**"TLS handshake failed"** - Wrong key format. Ensure you used `-traditional` flag.

**"No private key found"** - Key is still encrypted. Use `openssl pkcs8` to decrypt.

**"Certificate error: peer not verified"** - Certificate doesn't match TAK Server's CA. Regenerate using TAK Server's `makeCert.sh`.

**Connection timeout** - Check firewall rules and TAK Server's `CoreConfig.xml` allows your certificate's DN.

### Testing Your Connection

```bash
# Test TLS connectivity
openssl s_client -connect takserver.example.com:8089 \
  -cert omnitak.pem -key omnitak-rsa.key -CAfile ca.pem

# Should show "Verify return code: 0 (ok)"
```

## Installation

### Quick Install with Shell Script (Recommended)

#### Option 1: cargo-dist Shell Installer (Recommended)

The easiest way to install OmniTAK on macOS or Linux:

```bash
# Install latest version
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/engindearing-projects/omniTAK/releases/latest/download/omnitak-installer.sh | sh
```

This official installer is generated by cargo-dist and includes automatic updates and version management.

#### Option 2: Custom Install Script

Alternative installation script with more control:

```bash
# Install latest version
curl -fsSL https://raw.githubusercontent.com/engindearing-projects/omniTAK/main/scripts/install.sh | bash

# Or download and run
curl -fsSL https://raw.githubusercontent.com/engindearing-projects/omniTAK/main/scripts/install.sh -o install.sh
chmod +x install.sh
./install.sh

# Install specific version
./install.sh --version v0.2.0
```

The custom installer will:
- ✅ Detect your OS and architecture automatically
- ✅ Download the latest release from GitHub
- ✅ Install both `omnitak` and `omnitak-adb-setup` binaries
- ✅ Verify checksums (if available)
- ✅ Install to `/usr/local/bin` (or `~/.local/bin` without sudo)
- ✅ Set proper permissions

#### Windows PowerShell Installer

```powershell
# Install latest version
irm https://github.com/engindearing-projects/omniTAK/releases/latest/download/omnitak-installer.ps1 | iex
```

### Download Pre-built Binaries

Pre-built binaries are available for Windows, macOS, and Linux on the [GitHub Releases](https://github.com/engindearing-projects/omniTAK/releases) page.

**Latest Release: v0.2.0**

Download the binary for your platform:
- **Windows (x64)**: `omnitak-windows-x86_64.exe`
- **macOS (Intel)**: `omnitak-macos-x86_64`
- **macOS (Apple Silicon)**: `omnitak-macos-aarch64`
- **Linux (x64)**: `omnitak-linux-x86_64`

### Manual Install

#### Windows (PowerShell)

```powershell
# Download the latest release
Invoke-WebRequest -Uri "https://github.com/engindearing-projects/omniTAK/releases/latest/download/omnitak-windows-x86_64.exe" -OutFile "omnitak.exe"

# Make executable and move to a directory in your PATH
# Example: Move to a local bin directory
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\bin"
Move-Item omnitak.exe "$env:USERPROFILE\bin\omnitak.exe"

# Add to PATH (if not already added)
$env:PATH += ";$env:USERPROFILE\bin"
```

#### macOS

```bash
# Download the latest release (Intel)
curl -L "https://github.com/engindearing-projects/omniTAK/releases/latest/download/omnitak-macos-x86_64" -o omnitak

# Or for Apple Silicon
curl -L "https://github.com/engindearing-projects/omniTAK/releases/latest/download/omnitak-macos-aarch64" -o omnitak

# Make executable
chmod +x omnitak

# Move to /usr/local/bin (optional, requires sudo)
sudo mv omnitak /usr/local/bin/
```

#### Linux

```bash
# Download the latest release
curl -L "https://github.com/engindearing-projects/omniTAK/releases/latest/download/omnitak-linux-x86_64" -o omnitak

# Make executable
chmod +x omnitak

# Move to /usr/local/bin (optional, requires sudo)
sudo mv omnitak /usr/local/bin/
```

### Detailed Installation Guide

For complete installation instructions, troubleshooting, and all installation methods, see the **[Installation Guide](docs/INSTALLATION.md)**.

### Platform-Specific Installation Notes

#### Windows
- Pre-built binaries are available for Windows 10/11 (x64)
- No additional dependencies required for the binary
- For running as a Windows Service, see the [Windows Setup Guide](SETUP_WINDOWS.md)

#### macOS
- Pre-built binaries are available for both Intel and Apple Silicon
- macOS may require you to allow the app in System Preferences > Security & Privacy
- To bypass Gatekeeper: `xattr -d com.apple.quarantine omnitak`

#### Linux
- Pre-built binaries are statically linked and should work on most distributions
- Tested on Ubuntu 20.04+, Debian 11+, Fedora 35+, and RHEL 8+
- For running as a systemd service, see the [Ubuntu/Linux Setup Guide](SETUP_UBUNTU.md)

### Verify Installation

After installation, verify that OmniTAK is installed correctly:

```bash
# Check version
omnitak --version

# Should output: omnitak 0.2.0

# Show help
omnitak --help
```

### Build from Source

If you prefer to build from source or need the latest development version, see the platform-specific setup guides:

- **[macOS Setup Guide](SETUP_MACOS.md)** - Complete build instructions for macOS (Intel & Apple Silicon)
- **[Ubuntu/Linux Setup Guide](SETUP_UBUNTU.md)** - Complete build instructions for Ubuntu, Debian, and derivatives
- **[Windows Setup Guide](SETUP_WINDOWS.md)** - Complete build instructions for Windows 10/11 (Native & WSL2)

Each guide includes:
- Step-by-step installation of all dependencies (Rust, protoc, etc.)
- Platform-specific troubleshooting
- Building and compiling from source
- Configuration examples
- Running as a service (where applicable)
- Performance tuning tips

---

## Getting Started

After installing OmniTAK (see [Installation](#installation) above), you need to configure it to connect to your TAK servers.

### First Run

1. **Create a configuration directory:**
   ```bash
   mkdir -p ~/omnitak/config
   mkdir -p ~/omnitak/certs
   ```

2. **Create a basic configuration file** (`~/omnitak/config/config.yaml`):
   ```yaml
   application:
     max_connections: 1000
     worker_threads: 8

   servers:
     - id: my-tak-server
       address: "takserver.example.com:8087"
       protocol: tcp

   api:
     bind_addr: "0.0.0.0:9443"
   ```

3. **Run OmniTAK:**
   ```bash
   omnitak --config ~/omnitak/config/config.yaml
   ```

For more detailed configuration examples, see the [Usage](#usage) section below.

### Alternative: Run with Docker

```bash
docker build -t omnitak:latest .
docker run -p 8443:8443 -v $(pwd)/config.yaml:/app/config.yaml omnitak:latest
```

## Usage

### Configuration

Create a `config.yaml` file:

```yaml
application:
  max_connections: 1000
  worker_threads: 8

servers:
  - id: tak-server-1
    address: "192.168.1.100:8087"
    protocol: tls
    tls:
      cert_path: "/path/to/client.pem"
      key_path: "/path/to/client.key"
      ca_path: "/path/to/ca.pem"

filters:
  mode: whitelist
  rules:
    - id: friendly-only
      type: affiliation
      allow: [friend, assumedfriend]
      destinations: [ground-forces-server]

api:
  bind_addr: "0.0.0.0:8443"
  enable_tls: true
  tls_cert_path: "/path/to/server.pem"
  tls_key_path: "/path/to/server.key"
```

**Tip**: Use the [ADB Setup Tool](docs/ADB_SETUP.md) to generate this configuration automatically from your Android device!

### Viewing Data Flow

OmniTAK provides multiple ways to view and filter data from connected TAK servers:

- **Web UI**: Real-time message feed and connection status at `http://localhost:9443`
- **WebSocket API**: Stream CoT messages with custom filters
- **REST API**: Query historical messages and statistics
- **Prometheus Metrics**: Monitor performance and message flow

See the **[Filtering Guide](docs/FILTERING.md)** for detailed information on:
- Setting up message filters
- Viewing data flow in real-time
- Filtering by affiliation, geography, team, or custom criteria
- Performance tuning and best practices

### REST API

```bash
# Get system status
curl https://localhost:8443/api/v1/status \
  -H "Authorization: Bearer $TOKEN"

# List connections
curl https://localhost:8443/api/v1/connections

# Add new TAK server connection
curl -X POST https://localhost:8443/api/v1/connections \
  -H "Content-Type: application/json" \
  -d '{
    "address": "192.168.1.50:8087",
    "protocol": "tcp"
  }'

# Stream CoT messages via WebSocket
wscat -c wss://localhost:8443/api/v1/stream \
  -H "Authorization: Bearer $TOKEN"
```

### WebSocket API

Connect to `wss://host:port/api/v1/stream` and send:

```json
{
  "type": "subscribe",
  "event_types": ["a-f-G"],
  "geo_bounds": {
    "min_lat": 34.0,
    "max_lat": 35.0,
    "min_lon": -119.0,
    "max_lon": -118.0
  }
}
```

## Monitoring

### Prometheus Metrics

OmniTAK exports Prometheus-compatible metrics on `/api/v1/metrics`:

- `omnitak_connections_active` - Active connections
- `omnitak_messages_total` - Total messages processed
- `omnitak_routing_latency_seconds` - Message routing latency (histogram)
- `omnitak_errors_total` - Error counter by type

### Health Checks

- **Liveness**: `GET /health` - Is the service running?
- **Readiness**: `GET /ready` - Is the service ready to handle requests?

## Security

### Authentication
- **JWT Tokens**: Bearer token authentication with configurable expiration
- **API Keys**: Long-lived keys for service-to-service auth
- **Argon2id**: Password hashing with memory-hard algorithm

### Authorization
- **RBAC**: Role-based access control (Admin, Operator, ReadOnly)
- **Audit Logging**: All API operations logged with user, action, and timestamp

### TLS
- **TLS 1.2/1.3**: Supports both modern TLS 1.3 and legacy TLS 1.2 (for TAK Server compatibility)
- **Client Certificates**: Mutual TLS for TAK server connections
- **Memory-Safe**: Rustls implementation prevents OpenSSL vulnerabilities

## Testing

```bash
# Run all tests
cargo test --workspace

# Run benchmarks
cargo bench --workspace

# Run specific crate tests
cargo test -p omnitak-cot

# Run with logging
RUST_LOG=debug cargo test
```

## Deployment

### Kubernetes

See `k8s/` directory for:
- Deployment manifests
- Service definitions
- ConfigMap templates
- Secret management
- Horizontal Pod Autoscaling

### Docker Compose

```bash
docker-compose up -d
```

## Documentation

### Setup Guides
- **[Installation Guide](docs/INSTALLATION.md)** - Complete installation instructions for all platforms
- **[ADB Setup Guide](docs/ADB_SETUP.md)** - Automatically pull certificates from Android devices
- **[Filtering Guide](docs/FILTERING.md)** - Configure message filtering and view data flow
- [macOS Setup](SETUP_MACOS.md) - Complete macOS build from source
- [Ubuntu Setup](SETUP_UBUNTU.md) - Complete Linux build from source
- [Windows Setup](SETUP_WINDOWS.md) - Complete Windows build from source

### Technical Documentation
- [API Documentation](./crates/omnitak-api/README.md)
- [CoT Parser](./crates/omnitak-cot/README.md)
- [Filter System](./crates/omnitak-filter/README.md)
- [Connection Pool](./crates/omnitak-pool/README.md)

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Standards
- Follow Rust style guidelines (`cargo fmt`)
- Pass all tests (`cargo test`)
- Pass clippy lints (`cargo clippy -- -D warnings`)
- Add tests for new features
- Update documentation

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Military Applications

OmniTAK is designed for tactical battlefield coordination:
- **Multi-Domain Operations**: Aggregate feeds from ground, air, and maritime units
- **Coalition Operations**: Bridge different TAK server implementations
- **Tactical Edge**: Run on resource-constrained devices (Raspberry Pi, etc.)
- **Denied/Degraded Networks**: Message deduplication and efficient routing
- **Operational Security**: Memory-safe, audit logging, RBAC

## Related Projects

- [TAK Server](https://tak.gov/) - Official Team Awareness Kit server
- [FreeTAKServer](https://github.com/FreeTAKTeam/FreeTakServer) - Python TAK server
- [TAKy](https://github.com/tkuester/taky) - Minimal TAK server
- [OpenTAKServer](https://github.com/brian7704/OpenTAKServer) - Flask-based TAK server

## Known Issues


## Support

- **Issues**: [GitHub Issues](https://github.com/engindearing-projects/omniTAK/issues)
- **Documentation**: [Wiki](https://github.com/engindearing-projects/omniTAK/wiki)

---

**Built with Rust for reliability and performance in tactical environments.**
