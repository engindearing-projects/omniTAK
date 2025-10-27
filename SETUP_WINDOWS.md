# OmniTAK Setup Guide - Windows

Complete installation and setup guide for running OmniTAK on Windows 10 and Windows 11.

## System Requirements

- Windows 10 (64-bit) version 1809 or later, or Windows 11
- 4GB RAM minimum (8GB recommended)
- 5GB free disk space (includes build tools)
- Internet connection for downloading dependencies
- Administrator access

## Installation Methods

Choose one of two methods:

1. **Native Windows** - Run directly on Windows
2. **WSL2** - Run in Windows Subsystem for Linux (recommended for best compatibility)

---

## Method 1: Native Windows Installation

### Step 1: Install Visual Studio Build Tools

Rust on Windows requires the MSVC (Microsoft Visual C++) compiler.

**Option A: Visual Studio 2022 (Full IDE)**
1. Download [Visual Studio 2022 Community](https://visualstudio.microsoft.com/downloads/)
2. During installation, select "Desktop development with C++"
3. This includes the C++ compiler, Windows SDK, and other required tools

**Option B: Build Tools Only (Smaller Download)**
1. Download [Build Tools for Visual Studio 2022](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
2. Run the installer
3. Select "C++ build tools"
4. Check "Windows 10 SDK" and "MSVC v143 - VS 2022 C++ x64/x86 build tools"
5. Click Install (requires ~7GB disk space)

### Step 2: Install Git for Windows

1. Download [Git for Windows](https://git-scm.com/download/win)
2. Run the installer with default options
3. Git Bash will be installed (provides a Unix-like terminal)

### Step 3: Install Rust

1. Download [rustup-init.exe](https://rustup.rs/)
2. Run the installer
3. Press Enter to proceed with default installation
4. Wait for installation to complete (may take 5-10 minutes)
5. Close and reopen your terminal (Command Prompt or PowerShell)

Verify installation:
```powershell
rustc --version
cargo --version
```

Expected output:
```
rustc 1.90.0 (1159e78c4 2025-09-14)
cargo 1.90.0 (1159e78c4 2025-09-14)
```

### Step 4: Install Protocol Buffers Compiler

**Download and Install Manually:**

1. Go to [protobuf releases](https://github.com/protocolbuffers/protobuf/releases)
2. Download `protoc-25.1-win64.zip` (or latest version)
3. Extract to `C:\protobuf`
4. Add to PATH:
   - Press `Win + X`, select "System"
   - Click "Advanced system settings"
   - Click "Environment Variables"
   - Under "System variables", select "Path", click "Edit"
   - Click "New", add `C:\protobuf\bin`
   - Click "OK" on all dialogs
5. Open a new terminal and verify:

```powershell
protoc --version
```

Expected output:
```
libprotoc 25.1
```

**Alternative: Using Chocolatey**

If you have [Chocolatey](https://chocolatey.org/install) package manager:

```powershell
choco install protoc
```

### Step 5: Clone the Repository

Open Git Bash or PowerShell:

```powershell
# Navigate to your projects directory
cd C:\Users\YourUsername\Documents

# Clone the repository
git clone https://github.com/engindearing-projects/omniTAK.git
cd omniTAK
```

### Step 6: Build the Project

```powershell
cargo build --release -p omnitak-client -p omnitak-pool
```

This will:
- Download and compile all dependencies
- Build the omnitak-client and omnitak-pool crates
- Take 10-30 minutes on first build
- Create optimized binaries in `target\release\`

**Note:** You may see warnings about unused code - these are non-critical.

### Step 7: Verify the Build

```powershell
dir target\release\
```

You should see compiled library files (.rlib and .dll files).

### Step 8: Run Tests (Optional)

```powershell
cargo test -p omnitak-client -p omnitak-pool
```

## Method 2: WSL2 Installation (Recommended)

Windows Subsystem for Linux 2 provides better compatibility and performance for Unix-targeted applications.

### Step 1: Enable WSL2

Open PowerShell as Administrator and run:

```powershell
wsl --install
```

This will:
- Enable WSL and Virtual Machine Platform features
- Download and install Ubuntu (default distribution)
- Require a reboot

After reboot, Ubuntu will finish installing. Create a username and password when prompted.

**Alternative: Manual WSL2 Setup**

If `wsl --install` doesn't work:

```powershell
# Enable WSL
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart

# Enable Virtual Machine Platform
dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart

# Restart computer

# Set WSL 2 as default
wsl --set-default-version 2

# Install Ubuntu from Microsoft Store
# Search for "Ubuntu" in Microsoft Store and install
```

### Step 2: Update WSL Ubuntu

Open Ubuntu from the Start menu and run:

```bash
sudo apt update
sudo apt upgrade -y
```

### Step 3: Follow Ubuntu Setup Guide

Once inside WSL2 Ubuntu, follow the [Ubuntu setup guide](SETUP_UBUNTU.md) starting from Step 2.

Quick summary:
```bash
# Install dependencies
sudo apt install -y build-essential curl git pkg-config libssl-dev protobuf-compiler

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Clone and build
git clone https://github.com/engindearing-projects/omniTAK.git
cd omniTAK
cargo build --release -p omnitak-client -p omnitak-pool
```

### Accessing WSL Files from Windows

WSL files are accessible from Windows Explorer at:
```
\\wsl$\Ubuntu\home\yourusername\omniTAK
```

Or in PowerShell:
```powershell
cd \\wsl$\Ubuntu\home\yourusername\omniTAK
```

---

## Configuration (Both Methods)

### Create Basic Configuration

**PowerShell (Native Windows):**
```powershell
# Create config directory
New-Item -ItemType Directory -Path config -Force

# Create basic config file
@"
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
"@ | Out-File -FilePath config\config.yaml -Encoding UTF8
```

**Bash (WSL2):**
```bash
# Create config directory
mkdir -p config

# Create basic config file
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

Place certificates in the `certs` directory and update config:

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

**Native Windows (PowerShell):**
```powershell
.\target\release\omnitak.exe --config config\config.yaml
```

**WSL2 (Bash):**
```bash
./target/release/omnitak --config config/config.yaml
```

## Running as a Windows Service (Native Only)

To run OmniTAK as a Windows service, you'll need [NSSM (Non-Sucking Service Manager)](https://nssm.cc/):

### Install NSSM

1. Download [NSSM](https://nssm.cc/download)
2. Extract to `C:\nssm`
3. Add `C:\nssm\win64` to PATH (see Step 4 of native installation)

### Create Service

Open PowerShell as Administrator:

```powershell
# Install service
nssm install OmniTAK "C:\Users\YourUsername\Documents\omniTAK\target\release\omnitak.exe"

# Set parameters
nssm set OmniTAK AppParameters "--config config\config.yaml"
nssm set OmniTAK AppDirectory "C:\Users\YourUsername\Documents\omniTAK"

# Set to auto-start
nssm set OmniTAK Start SERVICE_AUTO_START

# Start service
nssm start OmniTAK

# Check status
nssm status OmniTAK
```

### Manage Service

```powershell
# Start service
nssm start OmniTAK

# Stop service
nssm stop OmniTAK

# Restart service
nssm restart OmniTAK

# Remove service
nssm remove OmniTAK confirm
```

## Troubleshooting

### Rust Not Found After Installation

Close and reopen your terminal. Rust adds itself to PATH, but terminals need to be restarted.

Or manually add to PATH:
```powershell
# Add to current session
$env:Path += ";$env:USERPROFILE\.cargo\bin"
```

### "link.exe not found" or "LINK : fatal error"

Visual Studio Build Tools not installed properly. Reinstall:
1. Download Build Tools for Visual Studio 2022
2. Select "C++ build tools"
3. Ensure "Windows 10 SDK" is checked

### "protoc not found" Error

protoc not in PATH:
```powershell
# Verify protoc location
where.exe protoc

# If not found, check if it exists
dir C:\protobuf\bin\protoc.exe

# Add to PATH (see Step 4)
```

### Windows Defender Blocking Build

Add exclusion for cargo directory:
1. Open Windows Security
2. Go to "Virus & threat protection"
3. Click "Manage settings"
4. Scroll to "Exclusions", click "Add or remove exclusions"
5. Add folder: `C:\Users\YourUsername\.cargo`

### Slow Build Times

First builds can take 20-40 minutes on Windows. Speed up:

```powershell
# Use more CPU cores
cargo build --release -p omnitak-client -p omnitak-pool -j 8
```

### Firewall Blocking Connections

Allow through Windows Firewall:
1. Open Windows Security
2. Go to "Firewall & network protection"
3. Click "Allow an app through firewall"
4. Click "Change settings"
5. Click "Allow another app"
6. Browse to `omnitak.exe` and add

### WSL2 Network Issues

WSL2 uses a virtual network adapter. To access from Windows:
```bash
# In WSL2, get IP address
ip addr show eth0 | grep inet

# Use this IP from Windows instead of 127.0.0.1
```

## Development Setup (Optional)

**PowerShell:**
```powershell
# Install development components
rustup component add rustfmt clippy

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Updating

**Native Windows:**
```powershell
cd C:\Users\YourUsername\Documents\omniTAK
git pull origin main
cargo build --release -p omnitak-client -p omnitak-pool
```

**WSL2:**
```bash
cd ~/omniTAK
git pull origin main
cargo build --release -p omnitak-client -p omnitak-pool
```

To update Rust:
```powershell
rustup update stable
```

## Uninstalling

**Native Windows:**
```powershell
# Remove service if created
nssm stop OmniTAK
nssm remove OmniTAK confirm

# Remove files
Remove-Item -Recurse -Force C:\Users\YourUsername\Documents\omniTAK

# Uninstall Rust
rustup self uninstall
```

**WSL2:**
```bash
rm -rf ~/omniTAK
rustup self uninstall
```

## Next Steps

- Read the [main README](README.md) for architecture overview
- Check [BUILD_FIXES_SUMMARY.md](BUILD_FIXES_SUMMARY.md) for technical details
- Configure your TAK server connections in `config/config.yaml`
- Review the API documentation in `crates/omnitak-api/README.md`

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/engindearing-projects/omniTAK/issues)
- **Documentation**: [Wiki](https://github.com/engindearing-projects/omniTAK/wiki)

## Windows-Specific Notes

### Performance Comparison

- **WSL2**: Generally better performance for Rust compilation and network I/O
- **Native Windows**: Better integration with Windows tools and services

### Which Method to Choose?

**Choose WSL2 if:**
- You're familiar with Linux
- You want better performance
- You plan to deploy on Linux in production
- You need Docker integration

**Choose Native Windows if:**
- You prefer Windows tools
- You need Windows service integration
- You want to use Visual Studio for debugging
- Your production environment is Windows

### WSL2 Limitations

- WSL2 uses virtualization (Hyper-V required)
- Can't run if other hypervisors (VirtualBox, VMware) are active
- Slightly more complex networking setup

### Development Tips

For the best Windows development experience:
1. Use WSL2 for building and running
2. Use VS Code with "Remote - WSL" extension
3. Edit code in VS Code (Windows) while running in WSL2
4. Access files from both Windows and WSL2

### Windows Terminal (Recommended)

Install [Windows Terminal](https://www.microsoft.com/store/productId/9N0DX20HK701) from Microsoft Store for a better command-line experience with tabs, Unicode support, and GPU acceleration.
