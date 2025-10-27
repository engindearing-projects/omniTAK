# OmniTAK Setup Guide - macOS

Complete installation and setup guide for running OmniTAK on macOS (Intel and Apple Silicon).

## System Requirements

- macOS 10.15 (Catalina) or later
- 4GB RAM minimum (8GB recommended)
- 2GB free disk space
- Internet connection for downloading dependencies

## Step 1: Install Homebrew (if not already installed)

Homebrew is the package manager for macOS. If you don't have it installed:

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Follow the on-screen instructions. After installation, you may need to add Homebrew to your PATH:

```bash
# For Apple Silicon Macs
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
eval "$(/opt/homebrew/bin/brew shellenv)"

# For Intel Macs
echo 'eval "$(/usr/local/bin/brew shellenv)"' >> ~/.zprofile
eval "$(/usr/local/bin/brew shellenv)"
```

Verify installation:
```bash
brew --version
```

## Step 2: Install Rust

Install Rust using rustup (official Rust installer):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

The installer will:
1. Download and install rustup
2. Install the latest stable Rust toolchain
3. Configure your PATH

Load Rust into your current shell:
```bash
source "$HOME/.cargo/env"
```

Verify installation:
```bash
rustc --version
cargo --version
```

You should see output like:
```
rustc 1.90.0 (1159e78c4 2025-09-14)
cargo 1.90.0 (1159e78c4 2025-09-14)
```

## Step 3: Install Protocol Buffers Compiler

OmniTAK requires `protoc` to compile protobuf definitions:

```bash
brew install protobuf
```

Verify installation:
```bash
protoc --version
```

Expected output:
```
libprotoc 33.0
```

## Step 4: Clone the Repository

```bash
# Clone the repository
git clone https://github.com/engindearing-projects/omniTAK.git
cd omniTAK
```

## Step 5: Build the Project

Build the core crates in release mode for optimal performance:

```bash
cargo build --release -p omnitak-client -p omnitak-pool
```

This will:
- Download and compile all dependencies
- Build the omnitak-client and omnitak-pool crates
- Take 5-15 minutes on first build (depending on your Mac)
- Create optimized binaries in `target/release/`

**Note:** You may see some warnings about unused code - these are non-critical and can be ignored.

## Step 6: Verify the Build

Check that the build completed successfully:

```bash
ls -lh target/release/
```

You should see compiled library files (.rlib and .dylib files).

## Step 7: Run Tests (Optional)

Verify everything works correctly:

```bash
cargo test -p omnitak-client -p omnitak-pool
```

## Configuration

Before running OmniTAK, you need to create a configuration file.

### Create Basic Configuration

```bash
# Create a config directory
mkdir -p config

# Create a basic configuration file
cat > config/config.yaml <<'EOF'
application:
  max_connections: 100
  worker_threads: 4

servers:
  - id: example-tak-server
    address: "192.168.1.100:8087"
    protocol: tcp

filters:
  mode: whitelist
  rules:
    - id: allow-all
      type: affiliation
      allow: [friend, assumedfriend, hostile, neutral, unknown]

api:
  bind_addr: "127.0.0.1:8443"
  enable_tls: false
EOF
```

### For TLS Connections

If connecting to TAK servers with TLS, you'll need certificates:

```bash
# Create certificates directory
mkdir -p certs

# Place your certificates in the certs directory:
# - certs/client.pem (client certificate)
# - certs/client.key (client private key)
# - certs/ca.pem (certificate authority)
```

Update your config to use TLS:
```yaml
servers:
  - id: secure-tak-server
    address: "192.168.1.100:8089"
    protocol: tls
    tls:
      cert_path: "certs/client.pem"
      key_path: "certs/client.key"
      ca_path: "certs/ca.pem"
```

## Running OmniTAK

Start the TAK server aggregator:

```bash
./target/release/omnitak --config config/config.yaml
```

**Note:** The main binary may not be available if only library crates were built. Check the `src/main.rs` to ensure it exists.

## Troubleshooting

### Build Fails with "protoc not found"

```bash
# Reinstall protobuf
brew reinstall protobuf

# Verify it's in your PATH
which protoc
```

### Rust Not Found After Installation

```bash
# Manually source the Rust environment
source "$HOME/.cargo/env"

# Add to your shell profile permanently
echo 'source "$HOME/.cargo/env"' >> ~/.zprofile
```

### Permission Denied Errors

```bash
# Make sure you're not running as root
whoami  # should show your username, not 'root'

# Fix permissions if needed
sudo chown -R $(whoami) ~/.cargo
```

### Slow Build Times

First builds can take 10-20 minutes. Speed it up:

```bash
# Use more CPU cores (replace 8 with your core count)
cargo build --release -p omnitak-client -p omnitak-pool -j 8
```

### Homebrew Installation Issues (Apple Silicon)

If Homebrew commands aren't found after installation:

```bash
# Add to PATH for current session
eval "$(/opt/homebrew/bin/brew shellenv)"

# Add permanently to shell profile
echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
```

## Development Setup (Optional)

For development work, you may want additional tools:

```bash
# Install Rust development tools
rustup component add rustfmt clippy

# Format code
cargo fmt

# Run linter
cargo clippy

# Watch for changes and rebuild
cargo install cargo-watch
cargo watch -x "build -p omnitak-client"
```

## Updating

To update OmniTAK to the latest version:

```bash
cd omniTAK
git pull origin main
cargo build --release -p omnitak-client -p omnitak-pool
```

To update Rust:

```bash
rustup update stable
```

## Uninstalling

To remove OmniTAK:

```bash
cd ..
rm -rf omniTAK
```

To uninstall Rust:

```bash
rustup self uninstall
```

To uninstall Homebrew packages:

```bash
brew uninstall protobuf
```

## Next Steps

- Read the [main README](README.md) for architecture overview
- Check [BUILD_FIXES_SUMMARY.md](BUILD_FIXES_SUMMARY.md) for technical details
- Configure your TAK server connections in `config/config.yaml`
- Review the API documentation in `crates/omnitak-api/README.md`

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/engindearing-projects/omniTAK/issues)
- **Documentation**: [Wiki](https://github.com/engindearing-projects/omniTAK/wiki)

## macOS-Specific Notes

### Apple Silicon (M1/M2/M3) Macs

OmniTAK builds natively on Apple Silicon. No special configuration needed.

### Rosetta 2

Not required - OmniTAK runs natively on both Intel and Apple Silicon.

### Firewall Settings

If you encounter connection issues:

1. Open **System Preferences** > **Security & Privacy** > **Firewall**
2. Click **Firewall Options**
3. Add the OmniTAK binary to allowed applications

### Network Performance

For maximum performance on 10GbE networks:

```bash
# Increase network buffer sizes
sudo sysctl -w net.inet.tcp.sendspace=4194304
sudo sysctl -w net.inet.tcp.recvspace=4194304
```

Make permanent by adding to `/etc/sysctl.conf`
