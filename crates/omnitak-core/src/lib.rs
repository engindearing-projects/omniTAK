//! # OmniTAK Core
//!
//! Core types, error handling, and configuration for the OmniTAK TAK server aggregator.
//!
//! This crate provides the foundational building blocks for the OmniTAK system:
//!
//! - **Types**: Core data structures including `ConnectionId`, `Protocol`, `ServerStatus`,
//!   `ServerConfig`, and connection metadata.
//! - **Errors**: Comprehensive error types using `thiserror` for all failure modes,
//!   including connection errors, parse errors, certificate errors, and configuration errors.
//! - **Configuration**: A flexible configuration system supporting YAML files,
//!   environment variable overrides, and validation.
//!
//! ## Features
//!
//! - Type-safe connection tracking with UUID-based identifiers
//! - Support for multiple protocols: TCP, UDP, TLS, and WebSocket
//! - Builder patterns for ergonomic configuration
//! - Serializable errors for API responses
//! - Comprehensive validation with clear error messages
//!
//! ## Example
//!
//! ```
//! use omnitak_core::types::{ServerConfig, Protocol};
//! use omnitak_core::config::AppConfig;
//!
//! // Build a server configuration
//! let server = ServerConfig::builder()
//!     .name("tak-server-1")
//!     .host("192.168.1.100")
//!     .port(8089)
//!     .protocol(Protocol::Tcp)
//!     .build();
//!
//! // Validate it
//! assert!(server.validate().is_ok());
//! ```

pub mod config;
pub mod discovery_config;
pub mod error;
pub mod plugins;
pub mod types;

// Re-export commonly used types for convenience
pub use config::AppConfig;
pub use error::{OmniTAKError, Result};
pub use types::{ConnectionId, Protocol, ServerConfig, ServerStatus};
