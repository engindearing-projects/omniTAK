# OmniTAK Setup Guide - Ubuntu/Debian Linux

Complete installation and setup guide for running OmniTAK on Ubuntu, Debian, and Debian-based distributions.

## System Requirements

- Ubuntu 20.04 LTS or later (or Debian 11+)
- 4GB RAM minimum (8GB recommended)
- 2GB free disk space
- Internet connection for downloading dependencies
- sudo privileges

## Supported Distributions

This guide works for:
- Ubuntu 20.04 LTS, 22.04 LTS, 24.04 LTS
- Debian 11 (Bullseye), 12 (Bookworm)
- Linux Mint 20+
- Pop!_OS 20.04+
- Elementary OS 6+

## Step 1: Update System Packages

First, update your package lists and upgrade existing packages:

```bash
sudo apt update
sudo apt upgrade -y
```

## Step 2: Install Build Dependencies

Install required build tools and libraries:

```bash
sudo apt install -y \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates
```

**What these packages do:**
- `build-essential`: GCC compiler, make, and other build tools
- `curl`: Download tool for fetching installers
- `git`: Version control for cloning the repository
- `pkg-config`: Helper tool for compiling with libraries
- `libssl-dev`: OpenSSL development headers (required by Rust crates)
- `ca-certificates`: SSL/TLS certificate authorities

## Step 3: Install Rust

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

**Make Rust available permanently:**
```bash
echo 'source "$HOME/.cargo/env"' >> ~/.bashrc
# Or for Zsh users:
echo 'source "$HOME/.cargo/env"' >> ~/.zshrc
```

Verify installation:
```bash
rustc --version
cargo --version
```

Expected output:
```
rustc 1.90.0 (1159e78c4 2025-09-14)
cargo 1.90.0 (1159e78c4 2025-09-14)
```

## Step 4: Install Protocol Buffers Compiler

OmniTAK requires `protoc` to compile protobuf definitions:

### Ubuntu 22.04+ / Debian 12+

```bash
sudo apt install -y protobuf-compiler
```

### Ubuntu 20.04 / Debian 11

The package manager version might be outdated. Install a newer version manually:

```bash
# Download latest protoc
PROTOC_VERSION=25.1
curl -LO "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-x86_64.zip"

# Install to /usr/local
sudo unzip protoc-${PROTOC_VERSION}-linux-x86_64.zip -d /usr/local

# Clean up
rm protoc-${PROTOC_VERSION}-linux-x86_64.zip

# Verify installation
protoc --version
```

Expected output:
```
libprotoc 25.1
```

## Step 5: Clone the Repository

```bash
# Navigate to your projects directory (or wherever you prefer)
cd ~

# Clone the repository
git clone https://github.com/engindearing-projects/omniTAK.git
cd omniTAK
```

## Step 6: Build the Project

Build the core crates in release mode for optimal performance:

```bash
cargo build --release -p omnitak-client -p omnitak-pool
```

This will:
- Download and compile all dependencies
- Build the omnitak-client and omnitak-pool crates
- Take 5-20 minutes on first build (depending on your CPU)
- Create optimized binaries in `target/release/`

**Note:** You may see some warnings about unused code - these are non-critical and can be ignored.

### For slower systems (optional):

Reduce parallel compilation to avoid memory issues:
```bash
cargo build --release -p omnitak-client -p omnitak-pool -j 2
```

## Step 7: Verify the Build

Check that the build completed successfully:

```bash
ls -lh target/release/
```

You should see compiled library files (.rlib and .so files).

## Step 8: Run Tests (Optional)

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

## Running as a Service (Optional)

To run OmniTAK as a systemd service:

### Create Service File

```bash
sudo tee /etc/systemd/system/omnitak.service > /dev/null <<EOF
[Unit]
Description=OmniTAK - Military-Grade TAK Server Aggregator
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$HOME/omniTAK
ExecStart=$HOME/omniTAK/target/release/omnitak --config config/config.yaml
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
```

### Enable and Start Service

```bash
# Reload systemd to recognize new service
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable omnitak

# Start the service
sudo systemctl start omnitak

# Check status
sudo systemctl status omnitak

# View logs
journalctl -u omnitak -f
```

### Service Management Commands

```bash
# Start service
sudo systemctl start omnitak

# Stop service
sudo systemctl stop omnitak

# Restart service
sudo systemctl restart omnitak

# View logs
journalctl -u omnitak -f

# Disable auto-start
sudo systemctl disable omnitak
```

## Troubleshooting

### Build Fails with "linker `cc` not found"

Install build tools:
```bash
sudo apt install -y build-essential
```

### Build Fails with "openssl not found"

Install OpenSSL development headers:
```bash
sudo apt install -y libssl-dev pkg-config
```

### "protoc not found" Error

```bash
# For Ubuntu 22.04+
sudo apt install -y protobuf-compiler

# Verify installation
which protoc
protoc --version
```

### Rust Not Found After Installation

```bash
# Manually source the Rust environment
source "$HOME/.cargo/env"

# Add to your shell profile permanently
echo 'source "$HOME/.cargo/env"' >> ~/.bashrc
source ~/.bashrc
```

### Out of Memory During Build

Reduce parallel jobs:
```bash
cargo build --release -p omnitak-client -p omnitak-pool -j 1
```

Or increase swap space:
```bash
# Create 4GB swap file
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Make permanent
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

### Permission Denied on Port 8087

Non-root users can't bind to ports below 1024. Either:

**Option 1:** Use a higher port (recommended):
```yaml
api:
  bind_addr: "127.0.0.1:8443"
```

**Option 2:** Grant capability (advanced):
```bash
sudo setcap 'cap_net_bind_service=+ep' target/release/omnitak
```

### Firewall Blocking Connections

Allow OmniTAK through UFW firewall:
```bash
# Allow specific port
sudo ufw allow 8443/tcp

# Check status
sudo ufw status
```

## Development Setup (Optional)

For development work, you may want additional tools:

```bash
# Install Rust development tools
rustup component add rustfmt clippy

# Install cargo-watch for auto-rebuild
cargo install cargo-watch

# Format code
cargo fmt

# Run linter
cargo clippy

# Watch for changes and rebuild
cargo watch -x "build -p omnitak-client"
```

## Performance Tuning (Optional)

### Increase File Descriptor Limits

For handling many connections:

```bash
# Check current limits
ulimit -n

# Increase for current session
ulimit -n 65535

# Make permanent - edit /etc/security/limits.conf
echo "* soft nofile 65535" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 65535" | sudo tee -a /etc/security/limits.conf
```

### Network Buffer Tuning

For high-throughput scenarios:

```bash
# Temporary
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728

# Permanent - edit /etc/sysctl.conf
echo "net.core.rmem_max=134217728" | sudo tee -a /etc/sysctl.conf
echo "net.core.wmem_max=134217728" | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

## Updating

To update OmniTAK to the latest version:

```bash
cd ~/omniTAK
git pull origin main
cargo build --release -p omnitak-client -p omnitak-pool
```

If running as a service:
```bash
sudo systemctl restart omnitak
```

To update Rust:
```bash
rustup update stable
```

## Uninstalling

To remove OmniTAK:

```bash
# Stop service if running
sudo systemctl stop omnitak
sudo systemctl disable omnitak
sudo rm /etc/systemd/system/omnitak.service
sudo systemctl daemon-reload

# Remove files
cd ~
rm -rf omniTAK
```

To uninstall Rust:
```bash
rustup self uninstall
```

To uninstall build dependencies:
```bash
sudo apt remove --purge -y protobuf-compiler build-essential
sudo apt autoremove -y
```

## Next Steps

- Read the [main README](README.md) for architecture overview
- Check [BUILD_FIXES_SUMMARY.md](BUILD_FIXES_SUMMARY.md) for technical details
- Configure your TAK server connections in `config/config.yaml`
- Review the API documentation in `crates/omnitak-api/README.md`

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/engindearing-projects/omniTAK/issues)
- **Documentation**: [Wiki](https://github.com/engindearing-projects/omniTAK/wiki)

## Linux-Specific Notes

### Running on Raspberry Pi (ARM)

OmniTAK compiles on ARM64 (aarch64) systems:

```bash
# Same installation steps work
# Build may take longer (30-60 minutes)
cargo build --release -p omnitak-client -p omnitak-pool -j 2
```

### Running in Docker (Alternative)

For containerized deployment, see the `Dockerfile` in the repository:

```bash
docker build -t omnitak:latest .
docker run -p 8443:8443 -v $(pwd)/config.yaml:/app/config.yaml omnitak:latest
```

### SELinux Considerations

If running on RHEL/CentOS/Fedora with SELinux:

```bash
# Check SELinux status
getenforce

# If enforcing, you may need to adjust policies
# Or run in permissive mode for testing
sudo setenforce 0
```
