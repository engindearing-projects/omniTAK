#!/bin/bash
set -e

# OmniTAK Installer Script
# Downloads and installs OmniTAK binaries from GitHub releases
# Usage: ./install.sh [--version VERSION]

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GITHUB_REPO="engindearing-projects/omniTAK"
BINARIES=("omnitak" "omnitak-adb-setup")
INSTALL_DIR=""
VERSION=""

# Print colored message
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                VERSION="$2"
                shift 2
                ;;
            -h|--help)
                echo "OmniTAK Installer"
                echo ""
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --version VERSION    Install specific version (e.g., v0.2.0)"
                echo "  -h, --help          Show this help message"
                echo ""
                echo "Examples:"
                echo "  $0                  # Install latest version"
                echo "  $0 --version v0.2.0 # Install specific version"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
}

# Detect OS and architecture
detect_platform() {
    local os arch target

    # Detect OS
    case "$(uname -s)" in
        Linux*)
            os="unknown-linux-gnu"
            ;;
        Darwin*)
            os="apple-darwin"
            ;;
        *)
            print_error "Unsupported operating system: $(uname -s)"
            print_info "Supported: Linux, macOS"
            exit 1
            ;;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="x86_64"
            ;;
        aarch64|arm64)
            arch="aarch64"
            ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            print_info "Supported: x86_64, aarch64"
            exit 1
            ;;
    esac

    target="${arch}-${os}"
    echo "$target"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for required commands
check_requirements() {
    local missing=()

    if ! command_exists curl && ! command_exists wget; then
        missing+=("curl or wget")
    fi

    if ! command_exists tar; then
        missing+=("tar")
    fi

    if [ ${#missing[@]} -gt 0 ]; then
        print_error "Missing required commands: ${missing[*]}"
        exit 1
    fi
}

# Download file using curl or wget
download_file() {
    local url="$1"
    local output="$2"

    print_info "Downloading from: $url"

    if command_exists curl; then
        curl -fsSL -o "$output" "$url" || return 1
    elif command_exists wget; then
        wget -q -O "$output" "$url" || return 1
    else
        return 1
    fi

    return 0
}

# Get latest release version from GitHub
get_latest_version() {
    local api_url="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"
    local version

    print_info "Fetching latest release version..."

    if command_exists curl; then
        version=$(curl -fsSL "$api_url" | grep '"tag_name":' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')
    elif command_exists wget; then
        version=$(wget -qO- "$api_url" | grep '"tag_name":' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')
    fi

    if [ -z "$version" ]; then
        print_error "Failed to fetch latest version"
        exit 1
    fi

    echo "$version"
}

# Determine install directory
determine_install_dir() {
    # Try /usr/local/bin first (requires sudo)
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    elif [ "$(id -u)" = "0" ]; then
        # Running as root
        INSTALL_DIR="/usr/local/bin"
    else
        # Try sudo
        if command_exists sudo && sudo -n true 2>/dev/null; then
            INSTALL_DIR="/usr/local/bin"
        else
            # Fall back to user local
            INSTALL_DIR="$HOME/.local/bin"
            print_warning "No sudo access, installing to $INSTALL_DIR"
            print_info "Make sure $INSTALL_DIR is in your PATH"

            # Create directory if it doesn't exist
            mkdir -p "$INSTALL_DIR"
        fi
    fi

    print_info "Install directory: $INSTALL_DIR"
}

# Install binary to directory
install_binary() {
    local binary="$1"
    local source="$2"
    local dest="$INSTALL_DIR/$binary"

    if [ ! -f "$source" ]; then
        print_error "Binary not found: $source"
        return 1
    fi

    print_info "Installing $binary to $dest"

    # Check if we need sudo
    if [ "$INSTALL_DIR" = "/usr/local/bin" ] && [ ! -w "/usr/local/bin" ]; then
        sudo install -m 755 "$source" "$dest" || return 1
    else
        install -m 755 "$source" "$dest" || return 1
    fi

    return 0
}

# Verify installation
verify_installation() {
    local binary="$1"
    local path="$INSTALL_DIR/$binary"

    if [ ! -f "$path" ]; then
        print_error "$binary not found at $path"
        return 1
    fi

    if [ ! -x "$path" ]; then
        print_error "$binary is not executable"
        return 1
    fi

    # Test if binary runs (just --version)
    if "$path" --version >/dev/null 2>&1; then
        local version_output=$("$path" --version 2>&1 | head -n 1)
        print_success "$binary installed: $version_output"
    else
        print_warning "$binary installed but version check failed"
    fi

    return 0
}

# Main installation function
main() {
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║      OmniTAK Installer Script         ║"
    echo "║    TAK Server Aggregator in Rust      ║"
    echo "╚════════════════════════════════════════╝"
    echo ""

    # Parse arguments
    parse_args "$@"

    # Check requirements
    print_info "Checking system requirements..."
    check_requirements

    # Detect platform
    print_info "Detecting platform..."
    local target
    target=$(detect_platform)
    print_success "Platform detected: $target"

    # Get version
    if [ -z "$VERSION" ]; then
        VERSION=$(get_latest_version)
    fi
    print_success "Installing version: $VERSION"

    # Determine install directory
    determine_install_dir

    # Construct download URL
    local filename="omnitak-${VERSION}-${target}.tar.gz"
    local download_url="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/${filename}"

    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf '$temp_dir'" EXIT

    print_info "Using temporary directory: $temp_dir"

    # Download release
    local archive_path="$temp_dir/$filename"
    if ! download_file "$download_url" "$archive_path"; then
        print_error "Failed to download release"
        print_info "URL: $download_url"
        print_info "This might mean:"
        print_info "  1. The release doesn't exist for your platform"
        print_info "  2. The version doesn't exist"
        print_info "  3. Network connectivity issues"
        exit 1
    fi
    print_success "Downloaded: $filename"

    # Download and verify checksum if available
    local checksum_url="${download_url}.sha256"
    local checksum_path="$temp_dir/${filename}.sha256"

    if download_file "$checksum_url" "$checksum_path" 2>/dev/null; then
        print_info "Verifying checksum..."
        if command_exists sha256sum; then
            (cd "$temp_dir" && sha256sum -c "${filename}.sha256") || {
                print_error "Checksum verification failed"
                exit 1
            }
            print_success "Checksum verified"
        elif command_exists shasum; then
            (cd "$temp_dir" && shasum -a 256 -c "${filename}.sha256") || {
                print_error "Checksum verification failed"
                exit 1
            }
            print_success "Checksum verified"
        else
            print_warning "sha256sum/shasum not found, skipping checksum verification"
        fi
    else
        print_warning "No checksum file available, skipping verification"
    fi

    # Extract archive
    print_info "Extracting archive..."
    tar -xzf "$archive_path" -C "$temp_dir" || {
        print_error "Failed to extract archive"
        exit 1
    }
    print_success "Archive extracted"

    # Install binaries
    local failed_installs=0
    for binary in "${BINARIES[@]}"; do
        local binary_path="$temp_dir/$binary"

        if [ -f "$binary_path" ]; then
            if install_binary "$binary" "$binary_path"; then
                if verify_installation "$binary"; then
                    print_success "$binary installed successfully"
                else
                    print_warning "$binary verification failed"
                    ((failed_installs++))
                fi
            else
                print_error "Failed to install $binary"
                ((failed_installs++))
            fi
        else
            print_warning "Binary not found in archive: $binary"
            ((failed_installs++))
        fi
    done

    # Print final status
    echo ""
    echo "════════════════════════════════════════"
    if [ $failed_installs -eq 0 ]; then
        print_success "Installation complete!"
        echo ""
        print_info "Installed binaries:"
        for binary in "${BINARIES[@]}"; do
            echo "  - $binary"
        done
        echo ""
        print_info "Run 'omnitak --help' to get started"

        # Check if install dir is in PATH
        if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
            echo ""
            print_warning "$INSTALL_DIR is not in your PATH"
            print_info "Add it to your PATH by running:"
            echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
            print_info "Or add this line to your ~/.bashrc or ~/.zshrc"
        fi
    else
        print_error "Installation completed with $failed_installs error(s)"
        exit 1
    fi
    echo "════════════════════════════════════════"
    echo ""
}

# Run main function
main "$@"
