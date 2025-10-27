# OmniTAK API

REST API and WebSocket interface for the TAK (Team Awareness Kit) aggregator with military-grade security.

## Features

- **REST API**: Complete endpoint coverage for system management
- **WebSocket Streaming**: Real-time CoT message distribution
- **Authentication**: JWT tokens and API keys with RBAC
- **Security**: TLS-only, rate limiting, audit logging, input validation
- **Observability**: Prometheus metrics, structured logging
- **Documentation**: OpenAPI/Swagger UI

## Quick Start

```rust
use omnitak_api::{ServerBuilder, ServerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::default();
    let server = ServerBuilder::new(config)
        .with_default_user("admin", "secure_password")
        .build()?;

    server.run().await?;
    Ok(())
}
```

## REST API Endpoints

### System Management

- `GET /api/v1/status` - Overall system status
- `GET /health` - Health check (no auth)
- `GET /ready` - Readiness check (no auth)

### Connection Management

- `GET /api/v1/connections` - List all connections
- `GET /api/v1/connections/:id` - Get connection details
- `POST /api/v1/connections` - Add new connection (operator+)
- `DELETE /api/v1/connections/:id` - Remove connection (operator+)

### Filter Management

- `GET /api/v1/filters` - List all filters
- `GET /api/v1/filters/:id` - Get filter details
- `POST /api/v1/filters` - Add/update filter (operator+)
- `DELETE /api/v1/filters/:id` - Remove filter (operator+)

### Metrics & Monitoring

- `GET /api/v1/metrics` - Prometheus metrics

### Authentication

- `POST /api/v1/auth/login` - User login
- `POST /api/v1/auth/api-keys` - Create API key (admin only)

### Audit Logs

- `GET /api/v1/audit` - Get audit logs (admin only)

## WebSocket API

### CoT Message Stream

**Endpoint**: `WS /api/v1/stream`

Subscribe to real-time CoT messages with filtering.

**Client Messages**:

```json
{
  "type": "subscribe",
  "event_types": ["a-f-G", "a-h-G"],
  "uids": ["pattern-*"],
  "geo_bounds": {
    "min_lat": 34.0,
    "max_lat": 35.0,
    "min_lon": -119.0,
    "max_lon": -118.0
  },
  "binary": false
}
```

```json
{
  "type": "unsubscribe"
}
```

```json
{
  "type": "ping"
}
```

**Server Messages**:

```json
{
  "type": "cot_message",
  "id": "uuid",
  "source_connection": "uuid",
  "data": "<CoT XML>",
  "event_type": "a-f-G",
  "uid": "device-123",
  "timestamp": "2025-10-27T12:00:00Z"
}
```

```json
{
  "type": "ack",
  "message_type": "subscribe"
}
```

```json
{
  "type": "pong"
}
```

### System Events Stream

**Endpoint**: `WS /api/v1/events`

Subscribe to system events (connections, disconnections, errors).

**Server Messages**:

```json
{
  "type": "system_event",
  "event": "connection_added",
  "details": {
    "connection_id": "uuid",
    "address": "192.168.1.100:4242"
  },
  "timestamp": "2025-10-27T12:00:00Z"
}
```

## Authentication

### JWT Token Authentication

1. Login to get a token:

```bash
curl -X POST https://api.example.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password"}'
```

Response:
```json
{
  "access_token": "eyJhbGc...",
  "expires_at": "2025-10-28T12:00:00Z",
  "role": "admin"
}
```

2. Use token in requests:

```bash
curl https://api.example.com/api/v1/status \
  -H "Authorization: Bearer eyJhbGc..."
```

### API Key Authentication

1. Create API key (admin only):

```bash
curl -X POST https://api.example.com/api/v1/auth/api-keys \
  -H "Authorization: Bearer <admin-token>" \
  -H "Content-Type: application/json" \
  -d '{"name": "My API Key", "role": "readonly"}'
```

2. Use API key in requests:

```bash
curl https://api.example.com/api/v1/status \
  -H "X-API-Key: omni_abc123..."
```

## Role-Based Access Control

### Roles

- **Admin**: Full system access, user management
- **Operator**: Manage connections and filters, read all data
- **ReadOnly**: Read-only access to system status and data

### Permissions Matrix

| Endpoint | ReadOnly | Operator | Admin |
|----------|----------|----------|-------|
| GET /api/v1/status | ✓ | ✓ | ✓ |
| GET /api/v1/connections | ✓ | ✓ | ✓ |
| POST /api/v1/connections | ✗ | ✓ | ✓ |
| DELETE /api/v1/connections | ✗ | ✓ | ✓ |
| GET /api/v1/filters | ✓ | ✓ | ✓ |
| POST /api/v1/filters | ✗ | ✓ | ✓ |
| POST /api/v1/auth/api-keys | ✗ | ✗ | ✓ |
| GET /api/v1/audit | ✗ | ✗ | ✓ |

## Security Features

### TLS Configuration

TLS is required for production use:

```rust
let config = ServerConfig {
    enable_tls: true,
    tls_cert_path: Some("/path/to/cert.pem".to_string()),
    tls_key_path: Some("/path/to/key.pem".to_string()),
    ..Default::default()
};
```

### Rate Limiting

Configurable per-client rate limiting:

```rust
let config = ServerConfig {
    rate_limit_rps: 100, // 100 requests per second
    ..Default::default()
};
```

### Audit Logging

All API operations are logged with:
- User identity
- Action performed
- Resource affected
- Timestamp
- Success/failure status
- Source IP address

### Security Headers

Automatically applied:
- `Strict-Transport-Security` (HSTS)
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Content-Security-Policy`
- `X-XSS-Protection`

## Metrics

Prometheus-compatible metrics available at `/api/v1/metrics`:

```
# Connections
omnitak_connections_total
omnitak_connections_active

# Messages
omnitak_messages_processed_total
omnitak_messages_per_second

# Filters
omnitak_filters_active
omnitak_filter_matches_total

# API
omnitak_api_requests_total
omnitak_api_request_duration_seconds
```

## WebSocket Protocol Specification

### Connection Flow

1. **Establish WebSocket connection** to `/api/v1/stream` or `/api/v1/events`
2. **Authenticate** (if required) - TBD based on auth middleware
3. **Subscribe** to desired message types/filters
4. **Receive messages** from server
5. **Send ping** periodically to keep connection alive
6. **Close connection** gracefully

### Backpressure Handling

The server implements backpressure management:
- Internal message queue per client (default 1000 messages)
- When queue full, oldest messages are dropped
- Client should monitor message timestamps to detect gaps

### Binary Encoding

Clients can request binary encoding (protobuf/msgpack) for efficiency:

```json
{
  "type": "subscribe",
  "binary": true
}
```

Binary format TBD - will use protobuf or msgpack for CoT messages.

## Configuration

```rust
pub struct ServerConfig {
    /// Server bind address (default: 0.0.0.0:8443)
    pub bind_addr: SocketAddr,

    /// Enable TLS (default: true)
    pub enable_tls: bool,

    /// TLS certificate path
    pub tls_cert_path: Option<String>,

    /// TLS key path
    pub tls_key_path: Option<String>,

    /// Authentication configuration
    pub auth_config: AuthConfig,

    /// Rate limit (requests per second, default: 100)
    pub rate_limit_rps: u32,

    /// Enable Swagger UI (default: true)
    pub enable_swagger: bool,

    /// Enable static file serving (default: true)
    pub enable_static_files: bool,
}
```

## Development

### Build

```bash
cargo build --package omnitak-api
```

### Test

```bash
cargo test --package omnitak-api
```

### Run

```bash
cargo run --package omnitak-api --example server
```

### API Documentation

Access Swagger UI at: `https://localhost:8443/swagger-ui`

## Examples

See `examples/` directory for:
- Basic server setup
- Custom authentication
- WebSocket client
- Metrics integration

## License

MIT OR Apache-2.0
