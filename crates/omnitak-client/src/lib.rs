//! # omnitak-client
//!
//! Async protocol clients for connecting to multiple TAK servers simultaneously.
//!
//! This crate provides robust, production-ready client implementations for various
//! TAK server connection protocols:
//!
//! - **TCP**: Frame-based protocol with newline-delimited or length-prefixed framing
//! - **UDP**: Connectionless datagram handling with optional multicast support
//! - **TLS**: Secure connections with client certificate authentication (TLS 1.3)
//! - **WebSocket**: Binary and text frames with automatic ping/pong keepalive
//!
//! ## Features
//!
//! - Async I/O using Tokio
//! - Auto-reconnect with exponential backoff
//! - Connection state tracking and metrics
//! - Comprehensive error handling
//! - Configurable timeouts and backpressure handling
//! - Distributed tracing support
//!
//! ## Example
//!
//! ```rust,no_run
//! use omnitak_client::{TakClient, tcp::{TcpClient, TcpClientConfig}};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut config = TcpClientConfig::default();
//!     config.base.server_addr = "127.0.0.1:8087".to_string();
//!
//!     let mut client = TcpClient::new(config);
//!     client.connect().await?;
//!
//!     // Use the client...
//!
//!     client.disconnect().await?;
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod state;
pub mod tcp;
pub mod udp;
pub mod tls;
pub mod websocket;

// Re-export commonly used types
pub use client::{
    ClientConfig, CotMessage, HealthCheck, HealthStatus, MessageMetadata,
    ReconnectConfig, TakClient,
};
pub use state::{ConnectionMetrics, ConnectionState, ConnectionStatus, MetricsSnapshot};

// Re-export bytes for convenience
pub use bytes::{Bytes, BytesMut};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that key types are accessible
        let _config = ClientConfig::default();
        let _status = ConnectionStatus::new();
    }
}
