# GitHub Actions Workflows

This directory contains automated CI/CD workflows for OmniTAK.

## Workflows

### `ci.yml` - Continuous Integration
**Triggers**: Push/PR to `main` or `develop` branches

**Jobs**:
1. **Test** - Runs on Ubuntu and macOS
   - Code formatting check
   - Clippy linting
   - Build verification
   - Test suite execution
   - GUI and tools build verification

2. **Build** - Multi-platform release builds
   - Linux x86_64
   - macOS x86_64 (Intel)
   - macOS aarch64 (Apple Silicon)
   - Uploads artifacts for each platform

**Duration**: ~5-10 minutes

---

### `main.yml` - Main Branch Validation
**Triggers**: Push to `main` branch

**Jobs**:
1. **Build and Test** - Full release builds
   - Optimized release builds for all platforms
   - Complete test suite
   - Binary size reporting
   - Smoke tests

2. **Security Audit** - Dependency scanning
   - Runs `cargo audit`
   - Checks for known vulnerabilities

3. **Code Coverage** - Coverage analysis
   - Generates coverage reports
   - Uploads to Codecov

**Duration**: ~10-15 minutes

---

### `release.yml` - Automated Releases
**Triggers**: Git tags matching `v*.*.*` pattern

**Jobs**:
1. **Create Release** - GitHub release creation
   - Parses version from tag
   - Creates release with auto-generated notes
   - Sets up upload URLs

2. **Build Release** - Multi-platform builds
   - Linux x86_64
   - macOS x86_64
   - macOS aarch64
   - Packages with documentation
   - Creates `.tar.gz` archives

3. **Generate Checksums** - Verification
   - Generates SHA256 checksums
   - Uploads `SHA256SUMS.txt`

**Duration**: ~15-20 minutes

**Output**: Public release with downloadable binaries

---

## Quick Reference

### To trigger CI tests:
```bash
git push origin your-branch
# Or create a pull request
```

### To create a release:
```bash
git tag -a v0.3.0 -m "Release 0.3.0"
git push origin v0.3.0
```

### To check workflow status:
Visit: https://github.com/engindearing-projects/omniTAK/actions

---

## Environment Variables

All workflows use:
- `CARGO_TERM_COLOR: always` - Colored output
- `RUST_BACKTRACE: 1` - Full backtraces on errors

---

## Caching Strategy

Workflows cache:
- `~/.cargo/registry` - Cargo registry
- `~/.cargo/git` - Git dependencies
- `target/` - Build artifacts

**Benefits**:
- Faster builds (2-3x speedup)
- Reduced network usage
- Consistent build environment

---

## Secrets Required

None currently. All workflows use:
- `GITHUB_TOKEN` - Automatically provided by GitHub Actions

---

## Maintenance

### Adding a new workflow:
1. Create `*.yml` file in this directory
2. Define triggers and jobs
3. Test with a draft PR
4. Document in this README

### Modifying existing workflows:
1. Edit the workflow file
2. Test changes on a feature branch
3. Verify in GitHub Actions UI
4. Update documentation

---

## Platform Support

| Platform | CI | Release | Status |
|----------|----|----|--------|
| Linux x86_64 | ✅ | ✅ | Stable |
| macOS x86_64 | ✅ | ✅ | Stable |
| macOS aarch64 | ✅ | ✅ | Stable |
| Windows | ❌ | ❌ | Planned |

---

## Troubleshooting

### Build fails with dependency errors
- Check if system dependencies are installed
- Review `Install Linux dependencies` step
- Verify `Cargo.lock` is committed

### Workflow not triggering
- Check trigger conditions (branch names, tag format)
- Verify GitHub Actions are enabled
- Check workflow file syntax

### Release not created
- Ensure tag matches `v*.*.*` pattern
- Check workflow logs for errors
- Verify GITHUB_TOKEN permissions

---

## Links

- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Release Process Guide](../docs/RELEASE_PROCESS.md)
- [Contributing Guide](../CONTRIBUTING.md)

---

**Questions?** Open an issue with the `ci/cd` label.
