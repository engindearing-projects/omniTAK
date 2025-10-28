# OmniTAK Web UI

A professional web interface for managing TAK server connections with certificate upload and password management capabilities.

## Features

### Certificate Management
- **File Upload**: Upload client certificates (.p12, .pem, .crt)
- **Private Keys**: Securely upload client private keys (.key, .pem)
- **CA Certificates**: Upload Certificate Authority certificates
- **Password Protection**: Support for password-encrypted certificates
- **TLS Configuration**: Configure TLS version and hostname verification

### Connection Management
- **Multiple Protocols**: Support for TCP, TLS, UDP, and WebSocket
- **Priority Settings**: Set connection priorities (1-10)
- **Auto-Reconnect**: Automatic reconnection on connection failure
- **Real-Time Status**: Live connection status monitoring
- **Advanced Options**: Timeout configuration, retry attempts, buffer size

### Monitoring
- **Real-Time Statistics**: Messages received, sent, filtered, and duplicated
- **Throughput Metrics**: Messages per second tracking
- **Connection Status**: Active/inactive connection visualization
- **Message Logging**: Real-time message stream with auto-scroll
- **Prometheus Integration**: Built-in metrics server on port 9090

## Quick Start

### 1. Start the Web Server

```bash
cd /Users/jfuginay/omniTAK
cargo run --example web_server -p omnitak-pool --release
```

### 2. Open in Browser

Navigate to: **http://localhost:8080**

### 3. Add a TAK Server Connection

#### Basic TCP Connection
1. Fill in the connection details:
   - **Connection Name**: "My TAK Server"
   - **Connection ID**: "my-tak-server"
   - **Server Address**: "192.168.1.100:8087"
   - **Protocol**: TCP
   - **Priority**: 5

2. Click "Add Connection"

#### TLS Connection with Certificates
1. Fill in basic details as above
2. Select **Protocol**: TLS (Secure)
3. The TLS Configuration section will appear
4. Upload certificates:
   - Click "Choose file" for **Client Certificate**
   - Click "Choose file" for **Client Private Key**
   - Click "Choose file" for **CA Certificate** (if required)
5. Enter **Certificate Password** if your certificate is encrypted
6. Configure TLS options:
   - Enable/disable **Verify Hostname**
   - Select **Minimum TLS Version** (1.2 or 1.3)
7. Click "Add Connection"

## File Structure

```
/Users/jfuginay/omniTAK/web/
├── index.html              # Main HTML interface
├── css/
│   └── styles.css         # Professional styling
├── js/
│   └── app.js             # Connection and certificate management
└── uploads/               # Certificate upload directory
```

## API Endpoints

The web server provides the following REST API endpoints:

### Status
- `GET /api/v1/status` - Get system status and statistics

### Connections
- `GET /api/v1/connections` - List all connections
- `POST /api/v1/connections` - Add new connection
- `DELETE /api/v1/connections/{id}` - Remove connection
- `POST /api/v1/connections/{id}/reconnect` - Reconnect to server

### Testing
- `POST /api/v1/test-connection` - Test connection before adding

### Metrics
- `GET /api/v1/metrics` - Prometheus metrics endpoint (redirects to :9090/metrics)

## Example Connection Payload

### TCP Connection
```json
{
  "id": "primary-tak",
  "name": "Primary TAK Server",
  "address": "public.opentakserver.io:8088",
  "protocol": "tcp",
  "priority": 10,
  "autoReconnect": true,
  "config": {
    "connectTimeout": 10,
    "readTimeout": 30,
    "retryAttempts": 3,
    "bufferSize": 65536
  }
}
```

### TLS Connection with Certificates
```json
{
  "id": "secure-tak",
  "name": "Secure TAK Server",
  "address": "192.168.1.100:8089",
  "protocol": "tls",
  "priority": 10,
  "autoReconnect": true,
  "tls": {
    "clientCert": {
      "name": "client.p12",
      "data": "<base64-encoded-certificate>",
      "size": 2048
    },
    "clientKey": {
      "name": "client.key",
      "data": "<base64-encoded-key>",
      "size": 1024
    },
    "caCert": {
      "name": "ca.pem",
      "data": "<base64-encoded-ca>",
      "size": 1536
    },
    "password": "certificate-password",
    "verifyHostname": true,
    "minTlsVersion": "1.3"
  }
}
```

## Certificate Formats Supported

- **PKCS#12 (.p12)**: Combined certificate and private key
- **PEM (.pem)**: Base64 encoded certificates/keys
- **CRT/CER (.crt, .cer)**: Certificate files
- **KEY (.key)**: Private key files

## Security Features

### Certificate Handling
- Certificates are read as Base64 and transmitted securely
- Password-protected certificates are supported
- Certificate files are NOT stored permanently by default
- TLS 1.2 and 1.3 support

### Connection Security
- Hostname verification (configurable)
- Certificate Authority validation
- Mutual TLS (mTLS) support
- Encrypted communication channels

## Monitoring & Metrics

### Web Interface
- Real-time connection status updates (every 5 seconds)
- Message statistics dashboard
- Live message log with auto-scroll
- Connection health indicators

### Prometheus Metrics
Access Prometheus metrics at: **http://localhost:9090/metrics**

Available metrics:
- `omnitak_connections_active` - Active connections
- `omnitak_messages_total` - Total messages processed
- `omnitak_routing_latency_seconds` - Message routing latency
- `omnitak_errors_total` - Error counter by type

## Troubleshooting

### Web Server Won't Start
```bash
# Check if port 8080 is already in use
lsof -i :8080

# Try a different port by modifying web_server.rs:
# Change: let addr = "0.0.0.0:8080";
# To:     let addr = "0.0.0.0:8081";
```

### Certificates Not Uploading
- Ensure the file size is reasonable (< 10MB)
- Check browser console for errors (F12)
- Verify certificate format is supported
- Try converting to PEM format if issues persist

### Connection Fails
1. Use "Test Connection" button before adding
2. Verify server address and port are correct
3. Check firewall settings
4. For TLS, ensure certificates match the server

### API Not Responding
```bash
# Check if backend is running
curl http://localhost:8080/api/v1/status

# View logs
# Logs appear in terminal where you ran the web_server command
```

## Advanced Configuration

### Custom TLS Settings
The UI supports advanced TLS configuration:
- **Hostname Verification**: Disable for testing, enable for production
- **TLS Version**: Use TLS 1.3 for maximum security
- **Certificate Chain**: Upload complete chain including intermediate CAs

### Timeout Configuration
- **Connect Timeout**: Time to wait for initial connection (default: 10s)
- **Read Timeout**: Time to wait for data (default: 30s)
- **Retry Attempts**: Number of reconnection attempts (default: 3)

### Buffer Size
- Increase for high-throughput scenarios
- Decrease for resource-constrained environments
- Default: 64KB (suitable for most use cases)

## Development

### Modify the UI
1. Edit HTML: `/Users/jfuginay/omniTAK/web/index.html`
2. Edit CSS: `/Users/jfuginay/omniTAK/web/css/styles.css`
3. Edit JavaScript: `/Users/jfuginay/omniTAK/web/js/app.js`
4. Refresh browser (server auto-serves latest files)

### Modify the Backend
1. Edit: `/Users/jfuginay/omniTAK/crates/omnitak-pool/examples/web_server.rs`
2. Rebuild: `cargo build --example web_server -p omnitak-pool --release`
3. Restart server

## Production Deployment

### Build Optimized Binary
```bash
cargo build --example web_server -p omnitak-pool --release
```

### Run as Service (macOS)
Create launch daemon at `/Library/LaunchDaemons/com.omnitak.webserver.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.omnitak.webserver</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/jfuginay/omniTAK/target/release/examples/web_server</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load service:
```bash
sudo launchctl load /Library/LaunchDaemons/com.omnitak.webserver.plist
```

### Environment Variables
```bash
# Set custom port
export OMNITAK_WEB_PORT=8080

# Set log level
export RUST_LOG=info
```

## Second Best Alternative

If you prefer a different approach, you could use Docker to containerize the entire application:

```dockerfile
FROM rust:1.90 as builder
WORKDIR /app
COPY . .
RUN cargo build --example web_server -p omnitak-pool --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/examples/web_server /usr/local/bin/
COPY web /web
EXPOSE 8080 9090
CMD ["web_server"]
```

## Support

For issues or questions:
- GitHub Issues: https://github.com/engindearing-projects/omniTAK/issues
- Documentation: See main README.md

---

**Built with Rust, HTML5, CSS3, and Vanilla JavaScript for maximum performance and security.**
