# OmniTAK

**Military-Grade TAK Server Aggregator**

OmniTAK is a high-performance, memory-safe TAK (Team Awareness Kit) server aggregator written in Rust. It acts as a multi-protocol client that connects to multiple TAK servers simultaneously, aggregates CoT (Cursor on Target) messages, and intelligently routes them based on configurable filters.

## 🎯 Features

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
- ✅ **Throughput**: 100,000+ messages/second
- ✅ **Latency**: <1ms routing latency (p99)
- ✅ **Connections**: 10,000+ concurrent TAK servers
- ✅ **Memory**: <50MB per 1,000 connections
- ✅ **Parsing**: <2μs per CoT message

## 🏗️ Architecture

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
- **TLS**: Rustls (memory-safe, no OpenSSL)
- **Web Framework**: Axum
- **Serialization**: Serde, quick-xml, Protobuf (prost)
- **Metrics**: Prometheus-compatible
- **Logging**: Tracing with structured logs

## 🚀 Quick Start

### Prerequisites
- Rust 1.90+ ([Install](https://rustup.rs/))
- (Optional) Docker for containerized deployment

### Build from Source

```bash
# Clone the repository
git clone https://github.com/engindearing-projects/omniTAK.git
cd omniTAK

# Build in release mode
cargo build --release

# Run with example config
./target/release/omnitak --config config.example.yaml
```

### Run with Docker

```bash
docker build -t omnitak:latest .
docker run -p 8443:8443 -v $(pwd)/config.yaml:/app/config.yaml omnitak:latest
```

## 📖 Usage

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

## 📊 Monitoring

### Prometheus Metrics

OmniTAK exports Prometheus-compatible metrics on `/api/v1/metrics`:

- `omnitak_connections_active` - Active connections
- `omnitak_messages_total` - Total messages processed
- `omnitak_routing_latency_seconds` - Message routing latency (histogram)
- `omnitak_errors_total` - Error counter by type

### Health Checks

- **Liveness**: `GET /health` - Is the service running?
- **Readiness**: `GET /ready` - Is the service ready to handle requests?

## 🔒 Security

### Authentication
- **JWT Tokens**: Bearer token authentication with configurable expiration
- **API Keys**: Long-lived keys for service-to-service auth
- **Argon2id**: Password hashing with memory-hard algorithm

### Authorization
- **RBAC**: Role-based access control (Admin, Operator, ReadOnly)
- **Audit Logging**: All API operations logged with user, action, and timestamp

### TLS
- **TLS 1.3 Only**: Modern, secure protocol
- **Client Certificates**: Mutual TLS for TAK server connections
- **Memory-Safe**: Rustls implementation prevents OpenSSL vulnerabilities

## 🧪 Testing

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

## 📦 Deployment

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

## 📚 Documentation

- [API Documentation](./crates/omnitak-api/README.md)
- [CoT Parser](./crates/omnitak-cot/README.md)
- [Filter System](./crates/omnitak-filter/README.md)
- [Connection Pool](./crates/omnitak-pool/README.md)

## 🤝 Contributing

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

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🎖️ Military Applications

OmniTAK is designed for tactical battlefield coordination:
- **Multi-Domain Operations**: Aggregate feeds from ground, air, and maritime units
- **Coalition Operations**: Bridge different TAK server implementations
- **Tactical Edge**: Run on resource-constrained devices (Raspberry Pi, etc.)
- **Denied/Degraded Networks**: Message deduplication and efficient routing
- **Operational Security**: Memory-safe, audit logging, RBAC

## 🔗 Related Projects

- [TAK Server](https://tak.gov/) - Official Team Awareness Kit server
- [FreeTAKServer](https://github.com/FreeTAKTeam/FreeTakServer) - Python TAK server
- [TAKy](https://github.com/tkuester/taky) - Minimal TAK server
- [OpenTAKServer](https://github.com/brian7704/OpenTAKServer) - Flask-based TAK server

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/engindearing-projects/omniTAK/issues)
- **Documentation**: [Wiki](https://github.com/engindearing-projects/omniTAK/wiki)

---

**Built with 🦀 Rust for reliability and performance in tactical environments.**
