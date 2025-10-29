# Changelog

All notable changes to OmniTAK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-10-29

### Added
- **Beta Release**: Core functionality complete and production-ready
- Full TLS 1.2 support for official TAK Server compatibility
- Comprehensive certificate setup documentation
- Traditional RSA key format support for legacy TAK Server connections
- TAKy server compatibility (basic TCP)
- REST API for dynamic connection management
- WebSocket streaming API for real-time CoT messages
- Connection status monitoring and health checks
- Automatic reconnection with configurable retry logic
- Message deduplication across multiple sources

### Changed
- Updated TLS implementation to support both TLS 1.2 and TLS 1.3
- Improved certificate loading to handle traditional RSA format
- Enhanced error messages for TLS handshake failures
- Updated documentation with step-by-step certificate conversion guide

### Fixed
- TLS handshake failures with official TAK Server due to key format
- Certificate validation errors with Rustls
- Connection timeout handling for unreachable servers
- Memory leaks in long-running connections

### Security
- Implemented Rustls for memory-safe TLS (no OpenSSL vulnerabilities)
- Added support for client certificate authentication
- Enhanced certificate validation and error reporting

### Documentation
- Added comprehensive TAK Server certificate setup guide
- Included troubleshooting section for common TLS issues
- Updated README with beta status and compatibility matrix
- Added testing instructions for TLS connectivity

### Known Issues
- FreeTAKServer compatibility testing in progress
- OpenTAKServer compatibility testing in progress
- High-volume message throughput optimization ongoing

## [0.1.0] - 2025-10-20

### Added
- Initial alpha release
- Basic TCP/UDP connection support
- CoT message parsing (XML)
- Configuration file support
- REST API scaffold
- Multi-protocol client architecture

---

**Legend:**
- `Added` - New features
- `Changed` - Changes to existing functionality
- `Deprecated` - Soon-to-be removed features
- `Removed` - Removed features
- `Fixed` - Bug fixes
- `Security` - Security improvements
