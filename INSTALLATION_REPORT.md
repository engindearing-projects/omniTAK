# OmniTAK Installation Report
**Date:** October 27, 2025
**Platform:** macOS (Apple Silicon)

## Summary
Attempted to clone, install, and test the OmniTAK project. The project **does not currently build** due to multiple compilation errors in the codebase.

## Installation Steps Performed

### 1. Repository Cloning ✅
```bash
git clone https://github.com/engindearing-projects/omniTAK.git
cd omniTAK
```
**Status:** Success

### 2. Rust Installation ✅
- **Original README stated:** Rust 1.90+ required
- **Action taken:** Installed Rust via rustup
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```
- **Version installed:** rustc 1.90.0
- **Status:** Success

### 3. First Build Attempt ❌
```bash
cargo build --release
```
**Status:** Failed
**Error:** Missing `protoc` (Protocol Buffers compiler)

### 4. Installing Missing Dependency ✅
- **Issue:** README did not mention protoc as a prerequisite
- **Action taken:** Installed protobuf via Homebrew
```bash
brew install protobuf
```
- **Version installed:** protobuf 33.0
- **Status:** Success

### 5. Second Build Attempt ❌
```bash
cargo build --release
```
**Status:** Failed due to compilation errors in source code

## Issues Found

### Critical: Missing Prerequisites in README
The original README's Prerequisites section only listed:
- Rust 1.90+
- (Optional) Docker

**Missing requirement:**
- Protocol Buffers compiler (protoc) - **REQUIRED, not optional**

### Critical: Code Does Not Compile
Multiple compilation errors prevent the project from building:

#### Error 1: Multiple Mutable Borrows (Borrow Checker Violation)
**File:** `crates/omnitak-client/src/tcp.rs`
**Lines:** 148, 149
**Error Type:** E0499 - Cannot borrow `*self` as mutable more than once at a time

```rust
// Lines 142-149
let stream = self.stream  // First mutable borrow
    .as_mut()
    .ok_or_else(|| Error::Connection("Stream not initialized".to_string()))?;

match self.config.framing {
    FramingMode::Newline => self.read_newline_frame(stream, buffer).await,  // Second borrow
    FramingMode::LengthPrefixed => self.read_length_prefixed_frame(stream, buffer).await,  // Second borrow
}
```

#### Error 2: Missing Method
**File:** `crates/omnitak-client/src/udp.rs`
**Line:** 119
**Error Type:** E0599 - No method named `set_recv_buffer_size` found

```rust
socket.set_recv_buffer_size(self.config.recv_buffer_size)
```

**Probable cause:** This method may have been removed or renamed in the version of Tokio being used.

#### Error 3: Closure Capture Issues (Multiple Files)
**Files:**
- `crates/omnitak-client/src/tcp.rs` line 397
- `crates/omnitak-client/src/tls.rs` line 423
- `crates/omnitak-client/src/websocket.rs` line 302

**Error:** Captured variable cannot escape `FnMut` closure body

```rust
async fn connect(&mut self) -> Result<()> {
    retry_with_backoff(
        || async { self.establish_connection().await },  // Variable captured escapes
        // ...
    ).await
}
```

### Non-Critical: Warnings
The build also generated 7 warnings:
- Unused imports: `tokio::sync::mpsc`, `debug`, `warn`, `PrivateKeyDer`, `Buf`
- Unused variables: `request`
- Unnecessary `mut` qualifiers

## Actions Taken

### Updated README.md
1. **Added complete prerequisites list** including:
   - Protocol Buffers compiler installation for multiple platforms
   - Specific package manager commands for macOS, Linux distributions, and Windows

2. **Added build status warning** at the top of Quick Start section alerting users to compilation issues

3. **Created "Known Issues" section** documenting:
   - All compilation errors with file locations and line numbers
   - Brief explanations of each error
   - List of missing prerequisites from original documentation

## Recommendations for Project Maintainers

### Immediate (Required for Project to Build)
1. **Fix borrow checker violations in tcp.rs**
   - Refactor to avoid simultaneous mutable borrows
   - Consider using RefCell or splitting into multiple methods

2. **Fix UDP socket configuration**
   - Replace `set_recv_buffer_size` with current Tokio API
   - Or update Tokio dependency to compatible version

3. **Fix closure capture issues**
   - Refactor retry logic to avoid escaping mutable references
   - Consider using different async patterns or ownership model

### High Priority (Documentation)
1. **Update Prerequisites section** to include all required dependencies
2. **Add build status badge** to README showing current build state
3. **Test fresh installation** on clean system before releases

### Medium Priority (Code Quality)
1. **Clean up unused imports and variables** (7 warnings)
2. **Add CI/CD pipeline** to catch build failures before commits
3. **Add integration tests** for installation process

## System Information
- **OS:** macOS (Darwin 24.5.0)
- **Architecture:** Apple Silicon (aarch64)
- **Rust Version:** 1.90.0 (1159e78c4 2025-09-14)
- **Cargo Version:** 1.90.0
- **Protoc Version:** libprotoc 33.0

## Conclusion
The OmniTAK project shows promise as a TAK server aggregator, but **cannot currently be installed or tested** due to compilation errors. The codebase requires fixes to multiple files before it can be built and evaluated. The documentation also needs updates to reflect actual prerequisites.

**Installation Status:** ❌ FAILED
**Next Steps:** Project maintainers must resolve compilation errors before the project can be used.
