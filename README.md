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

## Quick Start

### Platform-Specific Setup Guides

**Choose your operating system for detailed installation instructions:**

- **[macOS Setup Guide](SETUP_MACOS.md)** - Complete setup for macOS (Intel & Apple Silicon)
- **[Ubuntu/Linux Setup Guide](SETUP_UBUNTU.md)** - Complete setup for Ubuntu, Debian, and derivatives
- **[Windows Setup Guide](SETUP_WINDOWS.md)** - Complete setup for Windows 10/11 (Native & WSL2)

Each guide includes:
- Step-by-step installation of all dependencies
- Platform-specific troubleshooting
- Configuration examples
- Running as a service (where applicable)
- Performance tuning tips

### Quick Install (Summary)

For experienced users who prefer a quick reference:

#### Prerequisites
- Rust 1.90+ ([Install](https://rustup.rs/))
- Protocol Buffers compiler (protoc)
  - macOS: `brew install protobuf`
  - Ubuntu/Debian: `apt install protobuf-compiler`
  - Fedora/RHEL: `dnf install protobuf-compiler`
  - Windows: Download from [GitHub releases](https://github.com/protocolbuffers/protobuf/releases)

#### Build from Source

```bash
# Clone the repository
git clone https://github.com/engindearing-projects/omniTAK.git
cd omniTAK

# Build core crates (working)
cargo build --release -p omnitak-client -p omnitak-pool

# Create basic config
mkdir -p config
# (See platform guides for config.yaml template)

# Run
./target/release/omnitak --config config/config.yaml
```

### Run with Docker

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
- **[ADB Setup Guide](docs/ADB_SETUP.md)** - Automatically pull certificates from Android devices
- **[Filtering Guide](docs/FILTERING.md)** - Configure message filtering and view data flow
- [macOS Setup](SETUP_MACOS.md) - Complete macOS installation
- [Ubuntu Setup](SETUP_UBUNTU.md) - Complete Linux installation
- [Windows Setup](SETUP_WINDOWS.md) - Complete Windows installation

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
