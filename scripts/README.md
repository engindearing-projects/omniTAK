# OmniTAK Scripts

This directory contains utility scripts for OmniTAK installation and certificate management.

## Scripts

### install.sh

Production-ready installer for OmniTAK binaries on Linux and macOS.

**Features:**
- Automatic OS and architecture detection
- Downloads latest or specific version from GitHub Releases
- SHA256 checksum verification
- Supports both sudo and non-sudo installations
- Installs to `/usr/local/bin` or `~/.local/bin`
- Installs both `omnitak` and `omnitak-adb-setup` binaries

**Usage:**

```bash
# Install latest version
curl -fsSL https://raw.githubusercontent.com/engindearing-projects/omniTAK/main/scripts/install.sh | bash

# Download and run locally
curl -fsSL https://raw.githubusercontent.com/engindearing-projects/omniTAK/main/scripts/install.sh -o install.sh
chmod +x install.sh
./install.sh

# Install specific version
./install.sh --version v0.2.0

# Show help
./install.sh --help
```

**Supported Platforms:**
- Linux: x86_64, aarch64
- macOS: x86_64 (Intel), aarch64 (Apple Silicon)

**Requirements:**
- `curl` or `wget`
- `tar`
- `sha256sum` or `shasum` (optional, for checksum verification)

### convert-p12-to-pem.sh

Converts PKCS#12 certificate bundles (.p12) to PEM format for use with OmniTAK.

**Usage:**

```bash
./convert-p12-to-pem.sh <input.p12> <output-directory>
```

See the main [README.md](../README.md) for detailed TAK certificate setup instructions.

## Documentation

For complete installation instructions, see:
- [Installation Guide](../docs/INSTALLATION.md) - Comprehensive installation documentation
- [Main README](../README.md) - Project overview and quick start
- [ADB Setup](../docs/ADB_SETUP.md) - Automatic certificate extraction from Android

## Contributing

When adding new scripts:
1. Make them executable: `chmod +x script-name.sh`
2. Include clear usage instructions in comments
3. Add error handling and user-friendly messages
4. Update this README
