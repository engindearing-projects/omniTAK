# OmniTAK ADB Setup Guide

This guide explains how to use the **OmniTAK ADB Setup Tool** to automatically extract TAK server certificates and configuration from an Android device running ATAK/WinTAK, making it easy to configure OmniTAK as a TAK client.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Detailed Usage](#detailed-usage)
- [Certificate Conversion](#certificate-conversion)
- [Troubleshooting](#troubleshooting)
- [Manual Setup](#manual-setup)

## Overview

The ADB Setup Tool automates the process of:

1. **Connecting** to your Android device via ADB
2. **Extracting** TAK server certificates from the device
3. **Pulling** TAK server configuration (addresses, ports)
4. **Converting** certificates to OmniTAK-compatible format
5. **Generating** a ready-to-use `config.yaml` file

This eliminates the need to manually locate and transfer certificate files, making setup much faster and less error-prone.

## Prerequisites

### 1. Install Android SDK Platform-Tools

The ADB (Android Debug Bridge) tool is required to communicate with Android devices.

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install android-sdk-platform-tools
```

**macOS:**
```bash
brew install android-platform-tools
```

**Windows:**
Download from [Google's Android SDK Platform-Tools](https://developer.android.com/tools/releases/platform-tools)

**Verify Installation:**
```bash
adb version
```

### 2. Enable USB Debugging on Android Device

1. Go to **Settings** → **About Phone**
2. Tap **Build Number** 7 times to enable Developer Options
3. Go to **Settings** → **Developer Options**
4. Enable **USB Debugging**

### 3. Connect Device

1. Connect your Android device via USB
2. Accept the "Allow USB Debugging" prompt on your device
3. Verify connection:
   ```bash
   adb devices
   ```
   You should see your device listed as "device" (not "unauthorized")

## Quick Start

### Step 1: Build OmniTAK

```bash
cd omnitak
cargo build --release
```

### Step 2: Run ADB Setup Tool

```bash
# List available devices (optional)
./target/release/omnitak-adb-setup --list-devices

# Pull certificates and generate config
./target/release/omnitak-adb-setup --output config/config.yaml --cert-dir certs
```

### Step 3: Convert Certificates

TAK certificates are typically in PKCS#12 format (.p12) and need to be converted to PEM format:

```bash
./scripts/convert-p12-to-pem.sh certs/your-cert.p12 certs
```

### Step 4: Update Config

Edit `config/config.yaml` and update the certificate paths if needed.

### Step 5: Start OmniTAK

```bash
cargo run --release -- --config config/config.yaml
```

## Detailed Usage

### Command-Line Options

```bash
omnitak-adb-setup [OPTIONS]
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `-l, --list-devices` | List available ADB devices | - |
| `-d, --device <SERIAL>` | Specify device serial number | Auto-detect |
| `-o, --output <FILE>` | Output configuration file path | `config/config.yaml` |
| `-C, --cert-dir <DIR>` | Certificate output directory | `certs` |
| `--package <PACKAGE>` | ATAK package name | `com.atakmap.app.civ` |
| `--skip-validation` | Skip certificate validation | `false` |
| `-v, --verbose` | Enable verbose logging | `false` |
| `-h, --help` | Print help | - |

### Examples

**List Available Devices:**
```bash
omnitak-adb-setup --list-devices
```

**Pull from Specific Device:**
```bash
omnitak-adb-setup --device ABC123456789 --output my-config.yaml
```

**Use Different ATAK Variant:**
```bash
# For ATAK-MIL
omnitak-adb-setup --package com.atakmap.app.mil

# For ATAK-CIV (default)
omnitak-adb-setup --package com.atakmap.app.civ
```

**Custom Certificate Directory:**
```bash
omnitak-adb-setup --cert-dir /path/to/certs --output config/prod.yaml
```

**Verbose Output:**
```bash
omnitak-adb-setup --verbose
```

## Certificate Conversion

### Understanding Certificate Formats

TAK typically uses **PKCS#12** (.p12 or .pfx) format, which bundles:
- Client certificate
- Private key (encrypted)
- CA certificate chain

OmniTAK requires **PEM** format with **traditional RSA** keys:
- `client.pem` - Client certificate
- `client.key` - Private key (traditional RSA format, not PKCS#8)
- `ca.pem` - CA certificate chain

### Automatic Conversion

Use the provided conversion script:

```bash
./scripts/convert-p12-to-pem.sh certs/your-cert.p12 certs [password]
```

**With password:**
```bash
./scripts/convert-p12-to-pem.sh certs/user.p12 certs atakatak
```

**Without password (interactive):**
```bash
./scripts/convert-p12-to-pem.sh certs/user.p12 certs
# You'll be prompted for password 3 times
```

### Manual Conversion

If you prefer to convert manually:

```bash
# Extract client certificate
openssl pkcs12 -in cert.p12 -out client.pem -clcerts -nokeys

# Extract private key (PKCS#8 format)
openssl pkcs12 -in cert.p12 -out temp.key -nocerts -nodes

# Convert to traditional RSA format (required!)
openssl rsa -in temp.key -out client.key -traditional

# Extract CA certificates
openssl pkcs12 -in cert.p12 -out ca.pem -cacerts -nokeys

# Clean up
rm temp.key
```

**Important:** The private key MUST be in traditional RSA format. OmniTAK will fail with PKCS#8 format keys.

### Verify Certificate Format

```bash
# Check certificate
openssl x509 -in certs/client.pem -text -noout

# Check key format (should show "RSA PRIVATE KEY")
head -n 1 certs/client.key
# Expected: -----BEGIN RSA PRIVATE KEY-----
# NOT: -----BEGIN PRIVATE KEY----- (this is PKCS#8)

# Check CA certificate
openssl x509 -in certs/ca.pem -text -noout
```

## Configuration

After running the ADB setup tool, your `config.yaml` will look like this:

```yaml
application:
  max_connections: 100
  worker_threads: 4

servers:
  - id: tak-server-from-device
    address: "takserver.example.com:8089"
    protocol: tls
    auto_reconnect: true
    reconnect_delay_ms: 5000
    tls:
      cert_path: "certs/client.pem"
      key_path: "certs/client.key"
      ca_path: "certs/ca.pem"
      validate_certs: true

filters:
  mode: whitelist
  rules:
    - id: all-friendly
      type: affiliation
      allow: [friend, assumedfriend]
      destinations: [tak-server-from-device]

api:
  bind_addr: "127.0.0.1:9443"
  enable_tls: false

logging:
  level: "info"
  format: "text"

metrics:
  enabled: true
```

### Customizing Configuration

**Add Multiple Servers:**
```yaml
servers:
  - id: tak-server-primary
    address: "takserver1.example.com:8089"
    protocol: tls
    tls:
      cert_path: "certs/client.pem"
      key_path: "certs/client.key"
      ca_path: "certs/ca.pem"

  - id: tak-server-backup
    address: "takserver2.example.com:8089"
    protocol: tls
    tls:
      cert_path: "certs/client.pem"
      key_path: "certs/client.key"
      ca_path: "certs/ca.pem"
```

**Configure Filters for Data Flow:**
```yaml
filters:
  mode: whitelist
  rules:
    # Only friendly forces
    - id: friendly-only
      type: affiliation
      allow: [friend, assumedfriend]
      destinations: [tak-server-primary]

    # Geographic filter
    - id: operation-area
      type: geographic
      bounds:
        min_lat: 34.0
        max_lat: 35.0
        min_lon: -119.0
        max_lon: -118.0
      destinations: [tak-server-primary]

    # Team filter
    - id: team-alpha
      type: team
      teams: ["Alpha", "Bravo"]
      destinations: [tak-server-backup]
```

## Viewing Data Flow

Once OmniTAK is connected to TAK servers, you can view the data flow through:

### 1. Web UI

Open your browser to: `http://localhost:9443`

The web UI shows:
- Real-time CoT message feed
- Connection status for each server
- Message statistics and metrics
- Filter results

### 2. WebSocket API

Connect to the WebSocket endpoint to stream CoT messages:

```javascript
const ws = new WebSocket('ws://localhost:9443/ws');

ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    console.log('CoT Message:', message);
};
```

### 3. REST API

Query messages via REST API:

```bash
# Get all connections
curl http://localhost:9443/api/v1/connections

# Get messages
curl http://localhost:9443/api/v1/messages

# Get metrics
curl http://localhost:9443/api/v1/metrics
```

### 4. Command-Line Filtering

You can filter messages by type using the configuration:

```yaml
filters:
  mode: whitelist
  rules:
    # Filter by CoT event type
    - id: ground-vehicles
      type: affiliation
      field: "type"
      operator: starts_with
      value: "a-f-G-E-V"  # Friendly ground equipment vehicle
      action: accept
      destinations: [tak-server-primary]

    # Filter hostile entities
    - id: hostile-entities
      type: affiliation
      field: "type"
      operator: starts_with
      value: "a-h"  # Hostile
      action: accept
      destinations: [tak-server-backup]
```

## Troubleshooting

### ADB Connection Issues

**Problem:** "No devices found"

**Solution:**
1. Check USB connection
2. Enable USB Debugging on device
3. Accept USB debugging prompt on device
4. Try: `adb kill-server && adb start-server`

**Problem:** "unauthorized"

**Solution:**
1. Unplug and replug USB cable
2. Revoke USB debugging authorizations on device
3. Re-enable USB debugging
4. Accept the new authorization prompt

### Certificate Issues

**Problem:** "No certificates found"

**Possible causes:**
- Certificates are in app private directory (requires root)
- Using different ATAK variant (try `--package` option)
- Certificates stored in non-standard location

**Solution:**
1. Try manual pull:
   ```bash
   adb shell ls -la /sdcard/atak/cert
   adb shell ls -la /data/data/com.atakmap.app.civ/files/cert
   ```

2. Pull manually if found:
   ```bash
   adb pull /sdcard/atak/cert ./certs
   ```

3. For rooted devices:
   ```bash
   adb shell su -c 'ls /data/data/com.atakmap.app.civ/files/cert'
   adb shell su -c 'cp /data/data/com.atakmap.app.civ/files/cert/* /sdcard/tmp/'
   adb pull /sdcard/tmp ./certs
   ```

**Problem:** "Failed to load private key" or TLS connection errors

**Solution:** Verify key format:
```bash
head -n 1 certs/client.key
```

Should show `-----BEGIN RSA PRIVATE KEY-----`, NOT `-----BEGIN PRIVATE KEY-----`

If in PKCS#8 format, reconvert:
```bash
openssl rsa -in certs/client.key -out certs/client-rsa.key -traditional
mv certs/client-rsa.key certs/client.key
```

### Connection Issues

**Problem:** OmniTAK can't connect to TAK server

**Checklist:**
1. Verify server address and port
2. Check firewall rules
3. Verify certificate validity:
   ```bash
   openssl verify -CAfile certs/ca.pem certs/client.pem
   ```
4. Enable debug logging:
   ```yaml
   logging:
     level: "debug"
   ```
5. Check TAK server allows this client certificate

## Manual Setup

If ADB setup doesn't work for your situation, you can manually configure OmniTAK:

### 1. Obtain Certificates

**From ATAK device:**
- Export certificates from ATAK settings
- Transfer via email, cloud storage, or USB

**From TAK Server admin:**
- Request client certificate package
- Usually provided as .p12 or .zip file

### 2. Convert Certificates

Follow the [Certificate Conversion](#certificate-conversion) section above.

### 3. Create Configuration

Copy `config.example.yaml` to `config/config.yaml` and update:

```yaml
servers:
  - id: my-tak-server
    address: "your-takserver.com:8089"
    protocol: tls
    tls:
      cert_path: "certs/client.pem"
      key_path: "certs/client.key"
      ca_path: "certs/ca.pem"
```

### 4. Test Connection

```bash
cargo run -- --config config/config.yaml
```

Check logs for successful connection:
```
INFO omnitak: Connecting to tak-server: your-takserver.com:8089
INFO omnitak: Successfully connected to tak-server
INFO omnitak: Receiving messages from tak-server
```

## Security Best Practices

1. **Never commit certificates to version control**
   ```bash
   echo "certs/" >> .gitignore
   echo "*.p12" >> .gitignore
   echo "*.pem" >> .gitignore
   echo "*.key" >> .gitignore
   ```

2. **Set appropriate file permissions**
   ```bash
   chmod 600 certs/*.key
   chmod 644 certs/*.pem
   ```

3. **Use environment variables for sensitive config**
   ```bash
   export OMNITAK_ADMIN_PASSWORD="your-secure-password"
   ```

4. **Rotate certificates regularly** according to your organization's security policy

5. **Use TLS for the API server in production**
   ```yaml
   api:
     enable_tls: true
     tls_cert_path: "certs/api-cert.pem"
     tls_key_path: "certs/api-key.pem"
   ```

## Next Steps

- [Configuration Guide](../config.example.yaml) - Full configuration reference
- [API Documentation](API.md) - REST API and WebSocket endpoints
- [Filtering Guide](FILTERING.md) - Advanced message filtering
- [Deployment Guide](DEPLOYMENT.md) - Production deployment

## Support

If you encounter issues:

1. Check the [Troubleshooting](#troubleshooting) section
2. Enable verbose logging with `--verbose` or `logging.level: "debug"`
3. Review logs for error messages
4. Open an issue on GitHub with logs and configuration (redact sensitive info)
