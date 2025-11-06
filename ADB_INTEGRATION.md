# OmniTAK ADB Integration Guide

## Overview

OmniTAK now supports automatic certificate extraction from Android devices running ATAK via USB connection. This integration allows you to:

1. **Connect your ATAK device via USB** to the computer running omni-TAK
2. **Automatically pull TAK certificates** from the device
3. **Auto-connect to TAK servers** using the extracted certificates

## Architecture

### Components

- **`omnitak-adb` crate**: Core ADB functionality
  - Device detection and monitoring
  - Certificate extraction from ATAK directories
  - TAK server configuration parsing

- **REST API endpoints**:
  - `GET /api/v1/adb/devices` - List connected Android devices
  - `POST /api/v1/adb/pull-certs` - Pull certificates and optionally auto-connect

- **CLI tool**: `omnitak-adb-setup` - Standalone certificate extraction tool

## Prerequisites

### 1. Install Android SDK Platform-Tools

**Linux (Ubuntu/Debian)**:
```bash
sudo apt-get install android-tools-adb android-tools-fastboot
```

**macOS**:
```bash
brew install android-platform-tools
```

**Windows**:
Download from: https://developer.android.com/tools/releases/platform-tools

### 2. Enable USB Debugging on Android Device

1. Go to **Settings** → **About Phone**
2. Tap **Build Number** 7 times to enable Developer Options
3. Go to **Settings** → **Developer Options**
4. Enable **USB Debugging**

### 3. Connect Device via USB

1. Connect Android device to computer via USB cable
2. Accept the USB debugging authorization prompt on your device
3. Verify connection: `adb devices`

## Usage

### Method 1: REST API (Recommended)

#### List Connected Devices

```bash
curl -X GET http://localhost:9443/api/v1/adb/devices \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

**Response**:
```json
{
  "devices": [
    {
      "serial": "abc123",
      "state": "device",
      "model": "Pixel 7",
      "product": "cheetah"
    }
  ]
}
```

#### Pull Certificates and Auto-Connect

```bash
curl -X POST http://localhost:9443/api/v1/adb/pull-certs \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "device_serial": "abc123",
    "package": "com.atakmap.app.civ",
    "cert_dir": "certs/from-device",
    "auto_connect": true
  }'
```

**Response**:
```json
{
  "success": true,
  "message": "Successfully pulled 3 certificate files from device",
  "bundle": {
    "server_address": "takserver.example.com:8089",
    "server_name": "tak-server-from-device",
    "cert_count": 3,
    "cert_files": [
      "certs/from-device/client.pem",
      "certs/from-device/client.key",
      "certs/from-device/ca.pem"
    ]
  },
  "connection_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

### Method 2: CLI Tool

```bash
# List available devices
omnitak-adb-setup --list-devices

# Pull certificates and generate config
omnitak-adb-setup --output config/config.yaml

# Specify device if multiple connected
omnitak-adb-setup --device abc123 --output config/config.yaml

# Custom certificate directory
omnitak-adb-setup --cert-dir ./my-certs --output config/config.yaml
```

## Certificate Storage Locations

The integration automatically checks these locations on the Android device:

1. `/sdcard/atak/cert/` - Primary location (user-accessible)
2. `/sdcard/atak/certs/` - Alternative location
3. `/storage/emulated/0/atak/cert/` - Emulated storage
4. `/data/data/com.atakmap.app.civ/files/cert/` - App private storage (requires root)

## Certificate Formats

### Supported Formats

- **PEM** (`.pem`, `.crt`, `.cer`) - Preferred format ✅
- **Private Key** (`.key`) - Required for TLS ✅
- **PKCS#12** (`.p12`, `.pfx`) - Requires conversion ⚠️

### PKCS#12 Conversion

If your device uses `.p12` certificates, you'll need to convert them to PEM format:

```bash
# Extract client certificate
openssl pkcs12 -in client.p12 -out client.pem -clcerts -nokeys

# Extract private key
openssl pkcs12 -in client.p12 -out client.key -nocerts -nodes

# Extract CA certificate
openssl pkcs12 -in client.p12 -out ca.pem -cacerts -nokeys
```

## ATAK Package Variants

The integration supports multiple ATAK variants:

- **Civilian ATAK**: `com.atakmap.app.civ` (default)
- **Military ATAK**: `com.atakmap.app.mil`
- **WinTAK**: `com.atakmap.app.wintak`
- **Custom packages**: Specify any package name

## Configuration Parsing

The integration attempts to extract TAK server configuration from ATAK preferences:

**Preference file locations**:
- `/data/data/com.atakmap.app.civ/shared_prefs/com.atakmap.app.civ_preferences.xml`
- `/data/data/com.atakmap.app.civ/shared_prefs/com.atakmap.app_preferences.xml`

**Extracted information**:
- Server address and port
- Certificate password (if stored)
- Server description/name

**Note**: Accessing preference files may require root access on the device.

## Auto-Connection Flow

When `auto_connect: true` is set:

1. ✅ Certificates are pulled from device
2. ✅ Certificate files are validated
3. ✅ TLS client configuration is created
4. ✅ Connection to TAK server is established
5. ✅ Connection is registered in the pool
6. ✅ Connection ID is returned

## API Authentication

All ADB endpoints require authentication:

- **JWT Token**: Include in `Authorization: Bearer <token>` header
- **API Key**: Include in `X-API-Key` header
- **Required Role**: Operator or Admin

Get a JWT token:
```bash
curl -X POST http://localhost:9443/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "your_password"}'
```

## Troubleshooting

### Device Not Found

```bash
# Check ADB connection
adb devices

# If "unauthorized", accept prompt on device
# If "no devices", check USB cable and drivers
```

### Permission Denied

```bash
# Ensure USB debugging is enabled
# Re-authorize the computer on the device
adb kill-server
adb start-server
```

### Certificates Not Found

- Ensure ATAK has connected to a TAK server at least once
- Check if certificates are stored in a custom location
- Try accessing `/sdcard/atak/` directory manually:
  ```bash
  adb shell ls -la /sdcard/atak/cert/
  ```

### PKCS#12 Password Required

If you get an error about PKCS#12 requiring a password:
1. Convert certificates to PEM format (see above)
2. Use the converted PEM files instead
3. If you know the password, add support for password parameter

## Security Considerations

1. **USB Debugging**: Disable USB debugging when not in use
2. **Device Authorization**: Only authorize trusted computers
3. **Certificate Security**: Ensure extracted certificates are stored securely
4. **File Permissions**: Set appropriate permissions on certificate files:
   ```bash
   chmod 600 certs/from-device/*.key
   chmod 644 certs/from-device/*.pem
   ```

## Future Enhancements

- [ ] USB device hotplug detection (automatic pulling when device connects)
- [ ] Support for PKCS#12 password prompts
- [ ] Automatic certificate validation
- [ ] Multiple server configurations from single device
- [ ] Wireless ADB support
- [ ] Certificate expiration monitoring

## Example Workflow

```bash
# 1. Connect ATAK device via USB
# 2. Verify device is connected
curl http://localhost:9443/api/v1/adb/devices \
  -H "Authorization: Bearer $TOKEN"

# 3. Pull certificates and auto-connect
curl -X POST http://localhost:9443/api/v1/adb/pull-certs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"auto_connect": true}'

# 4. Verify connection
curl http://localhost:9443/api/v1/connections \
  -H "Authorization: Bearer $TOKEN"

# 5. Start using TAK server!
```

## Integration with omni-cot Plugin

The omni-cot plugin (https://github.com/engindearing-projects/omni-COT) runs on the ATAK device. This ADB integration complements it by:

1. **Extracting certificates** stored by ATAK
2. **Auto-configuring** omni-TAK with server details
3. **Enabling rapid deployment** without manual certificate management

The omni-cot plugin focuses on CoT message management and AOI detection, while this integration handles the certificate infrastructure.

## Support

For issues or questions:
- GitHub Issues: https://github.com/engindearing-projects/omniTAK/issues
- Documentation: See project README.md

## License

This integration is part of OmniTAK and is licensed under MIT OR Apache-2.0.
