# OmniTAK Release Process

This document describes the CI/CD setup and release process for OmniTAK.

## Overview

OmniTAK uses GitHub Actions for continuous integration and automated releases. When you merge to `main` or create a release tag, automated workflows will build, test, and publish binaries.

## CI/CD Workflows

### 1. CI Workflow (`ci.yml`)
**Triggers**: Push to `main` or `develop`, Pull Requests

**What it does**:
- Runs on Ubuntu and macOS
- Checks code formatting (`cargo fmt`)
- Runs Clippy linter
- Builds all binaries (omnitak, omnitak-gui, omnitak-adb-setup)
- Runs all tests
- Caches dependencies for faster builds
- Uploads build artifacts

**Platform Support**:
- ✅ Ubuntu (x86_64)
- ✅ macOS (x86_64 Intel)
- ✅ macOS (aarch64 Apple Silicon)

### 2. Main Branch Build (`main.yml`)
**Triggers**: Push to `main` branch

**What it does**:
- Full release builds on multiple platforms
- Security audit with `cargo-audit`
- Code coverage analysis
- Smoke tests for all binaries
- Reports binary sizes

**Additional Checks**:
- Security vulnerabilities in dependencies
- Code coverage metrics (uploaded to Codecov)
- Binary size tracking

### 3. Release Workflow (`release.yml`)
**Triggers**: Git tags matching `v*.*.*` (e.g., `v0.2.1`)

**What it does**:
- Creates a GitHub Release
- Builds optimized release binaries for all platforms
- Packages binaries with documentation
- Generates SHA256 checksums
- Uploads release assets to GitHub

**Release Artifacts**:
- `omnitak-linux-x86_64.tar.gz` - Linux binaries
- `omnitak-macos-x86_64.tar.gz` - macOS Intel binaries
- `omnitak-macos-aarch64.tar.gz` - macOS Apple Silicon binaries
- `SHA256SUMS.txt` - Checksums for verification

Each archive contains:
- `omnitak` - Main server binary
- `omnitak-gui` - Desktop GUI application
- `omnitak-adb-setup` - ADB certificate extraction tool
- `README.md` - Project documentation
- `CHANGELOG_GUI.md` - Change history
- `docs/` - Complete documentation
- `config.example.yaml` - Example configuration

## Creating a Release

### Step 1: Update Version Numbers

Update version in `Cargo.toml`:

```toml
[workspace.package]
version = "0.3.0"  # Update this
```

### Step 2: Update CHANGELOG

Update `CHANGELOG_GUI.md` with release notes:

```markdown
## Version 0.3.0 - 2024-11-02

### New Features
- Added file picker integration
- Enhanced connection testing
...
```

### Step 3: Commit Changes

```bash
git add Cargo.toml CHANGELOG_GUI.md
git commit -m "chore: Bump version to 0.3.0"
git push origin main
```

### Step 4: Create and Push Tag

```bash
# Create annotated tag
git tag -a v0.3.0 -m "Release version 0.3.0"

# Push tag to trigger release workflow
git push origin v0.3.0
```

### Step 5: Monitor Release Build

1. Go to GitHub Actions: `https://github.com/engindearing-projects/omniTAK/actions`
2. Watch the "Release" workflow
3. Builds typically take 10-15 minutes

### Step 6: Verify Release

1. Go to Releases: `https://github.com/engindearing-projects/omniTAK/releases`
2. Verify all assets are uploaded:
   - ✅ Linux tarball
   - ✅ macOS Intel tarball
   - ✅ macOS ARM tarball
   - ✅ SHA256SUMS.txt
3. Test download and extraction
4. Verify checksums:
   ```bash
   sha256sum -c SHA256SUMS.txt
   ```

### Step 7: Announce Release

Update the following:
- GitHub Discussions (if enabled)
- Project README if major changes
- Documentation links
- Community channels

## Merge to Main Process

### What Happens on Merge

When you merge a PR or push to `main`:

1. **CI Workflow Runs**:
   - Code formatting check
   - Linting with Clippy
   - Full test suite
   - Multi-platform builds

2. **Main Branch Build Runs**:
   - Release-optimized builds
   - Security audit
   - Code coverage
   - Binary verification

3. **Artifacts Created**:
   - Build artifacts available for download
   - Coverage reports uploaded
   - Security scan results

### What Does NOT Happen

❌ **Does NOT automatically create a release**
❌ **Does NOT publish binaries publicly**
❌ **Does NOT update version numbers**

You must manually create a git tag to trigger a release.

## Pre-Release Checklist

Before creating a release tag:

- [ ] All tests passing on `main`
- [ ] Version updated in `Cargo.toml`
- [ ] `CHANGELOG_GUI.md` updated
- [ ] Documentation reviewed and updated
- [ ] Breaking changes documented
- [ ] Migration guide written (if needed)
- [ ] Tested on Ubuntu
- [ ] Tested on macOS
- [ ] Security audit clean (`cargo audit`)

## Platform-Specific Build Requirements

### Linux (Ubuntu)
Required packages:
```bash
sudo apt-get install -y \
  libxcb-render0-dev \
  libxcb-shape0-dev \
  libxcb-xfixes0-dev \
  libxkbcommon-dev \
  libssl-dev \
  libfontconfig1-dev \
  pkg-config
```

### macOS
No additional dependencies required. Xcode Command Line Tools included by default on GitHub Actions runners.

### Windows (Future)
Will require additional setup for cross-compilation or Windows runners.

## Troubleshooting Builds

### Build Fails on Linux

**Issue**: Missing system libraries
**Solution**: Check `.github/workflows/*.yml` for required packages

### Build Fails on macOS

**Issue**: Target not installed
**Solution**: Ensure `targets` field in workflow specifies correct target

### Release Workflow Not Triggered

**Issue**: Tag format incorrect
**Solution**: Tags must match `v*.*.*` pattern (e.g., `v0.2.1`)

### Checksums Mismatch

**Issue**: Files corrupted during upload
**Solution**: Re-run the release workflow

## Manual Build Instructions

If you need to build locally:

### Linux
```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

### macOS Intel
```bash
cargo build --release --target x86_64-apple-darwin
```

### macOS Apple Silicon
```bash
cargo build --release --target aarch64-apple-darwin
```

### Cross-compilation

For cross-platform builds, use `cross`:

```bash
# Install cross
cargo install cross

# Build for different platforms
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
```

## Security Considerations

### Dependency Auditing

Runs automatically on every `main` branch push:
```bash
cargo audit
```

### Supply Chain Security

- All dependencies pinned in `Cargo.lock`
- Security advisories checked
- Minimal dependency tree
- Prefer well-maintained crates

### Binary Verification

Users can verify downloads:
```bash
# Download checksums
curl -LO https://github.com/engindearing-projects/omniTAK/releases/download/v0.3.0/SHA256SUMS.txt

# Verify
sha256sum -c SHA256SUMS.txt
```

## Release Cadence

**Recommended Schedule**:
- **Patch releases** (0.2.x): As needed for bug fixes
- **Minor releases** (0.x.0): Monthly or when features are ready
- **Major releases** (x.0.0): When breaking changes are introduced

## Rolling Back a Release

If a critical issue is found:

1. **Mark release as pre-release**:
   - Edit release on GitHub
   - Check "This is a pre-release"

2. **Create hotfix**:
   ```bash
   git checkout v0.3.0
   git checkout -b hotfix/v0.3.1
   # Fix issue
   git commit -m "fix: Critical bug"
   git tag v0.3.1
   git push origin v0.3.1
   ```

3. **Notify users**:
   - Update release notes
   - Announce hotfix availability

## Future Improvements

Planned CI/CD enhancements:

- [ ] Windows build support
- [ ] Docker image builds
- [ ] Automated changelog generation
- [ ] Release draft auto-creation
- [ ] Homebrew formula updates
- [ ] APT/YUM repository publishing
- [ ] Binary signature verification (GPG)
- [ ] Automated benchmarking
- [ ] Performance regression detection
- [ ] Documentation site deployment

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Cargo Release Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)

## Support

If you encounter issues with the release process:
1. Check GitHub Actions logs
2. Review this documentation
3. Open an issue with the `ci/cd` label
4. Contact maintainers

---

**Last Updated**: 2024-11-01
**Maintained By**: OmniTAK Contributors
