//! Core types for the OmniTAK TAK server aggregator.
//!
//! This module defines the fundamental types used throughout the system, including
//! connection identifiers, protocol specifications, server configurations, and status tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for a connection.
///
/// Wraps a UUID v4 to provide type-safe connection tracking across the system.
/// Each connection to a TAK server gets a unique ConnectionId that persists
/// for the lifetime of that connection.
///
/// # Examples
///
/// ```
/// use omnitak_core::types::ConnectionId;
///
/// let id = ConnectionId::new();
/// println!("Connection: {}", id);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(Uuid);

impl ConnectionId {
    /// Creates a new random connection identifier.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ConnectionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<ConnectionId> for Uuid {
    fn from(id: ConnectionId) -> Self {
        id.0
    }
}

/// Network protocol supported by the TAK server aggregator.
///
/// Defines the available transport protocols for connecting to TAK servers.
/// Each protocol has different characteristics and use cases:
///
/// - TCP: Basic reliable stream transport
/// - UDP: Unreliable datagram transport (lower overhead)
/// - Tls: TLS-encrypted TCP (required for secure communications)
/// - WebSocket: HTTP-upgraded persistent connections (firewall-friendly)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// Unencrypted TCP connection
    Tcp,
    /// Unencrypted UDP connection
    Udp,
    /// TLS-encrypted TCP connection
    Tls,
    /// WebSocket connection (can be secure or insecure)
    WebSocket,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
            Protocol::Tls => write!(f, "TLS"),
            Protocol::WebSocket => write!(f, "WebSocket"),
        }
    }
}

impl Protocol {
    /// Returns true if this protocol uses encryption.
    pub fn is_secure(&self) -> bool {
        matches!(self, Protocol::Tls | Protocol::WebSocket)
    }

    /// Returns true if this protocol is connection-oriented.
    pub fn is_stream_based(&self) -> bool {
        matches!(self, Protocol::Tcp | Protocol::Tls | Protocol::WebSocket)
    }

    /// Returns the default port for this protocol.
    pub fn default_port(&self) -> u16 {
        match self {
            Protocol::Tcp => 8087,
            Protocol::Udp => 8087,
            Protocol::Tls => 8089,
            Protocol::WebSocket => 8443,
        }
    }
}

/// Current status of a server connection.
///
/// Tracks the lifecycle state of a connection to a TAK server, enabling
/// proper reconnection logic and status reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    /// Successfully connected and operational
    Connected,
    /// Not currently connected
    Disconnected,
    /// Attempting to reconnect after a failure
    Reconnecting,
    /// Connection failed and reconnection attempts exhausted
    Failed,
}

impl fmt::Display for ServerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerStatus::Connected => write!(f, "Connected"),
            ServerStatus::Disconnected => write!(f, "Disconnected"),
            ServerStatus::Reconnecting => write!(f, "Reconnecting"),
            ServerStatus::Failed => write!(f, "Failed"),
        }
    }
}

impl ServerStatus {
    /// Returns true if the status indicates an active connection.
    pub fn is_connected(&self) -> bool {
        matches!(self, ServerStatus::Connected)
    }

    /// Returns true if the status indicates a recoverable state.
    pub fn is_recoverable(&self) -> bool {
        matches!(self, ServerStatus::Disconnected | ServerStatus::Reconnecting)
    }
}

/// TLS configuration for secure connections.
///
/// Contains all necessary configuration for establishing TLS connections,
/// including paths to certificate files and optional client authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to the CA certificate file (PEM format)
    pub ca_cert_path: PathBuf,

    /// Path to the client certificate file (PEM format, optional)
    pub client_cert_path: Option<PathBuf>,

    /// Path to the client private key file (PEM format, optional)
    pub client_key_path: Option<PathBuf>,

    /// Whether to verify the server's certificate (default: true)
    #[serde(default = "default_verify_cert")]
    pub verify_cert: bool,

    /// Server name for SNI (Server Name Indication)
    pub server_name: Option<String>,
}

fn default_verify_cert() -> bool {
    true
}

impl TlsConfig {
    /// Creates a new TLS configuration with the specified CA certificate.
    pub fn new(ca_cert_path: PathBuf) -> Self {
        Self {
            ca_cert_path,
            client_cert_path: None,
            client_key_path: None,
            verify_cert: true,
            server_name: None,
        }
    }

    /// Sets the client certificate and key paths for mutual TLS.
    pub fn with_client_cert(mut self, cert_path: PathBuf, key_path: PathBuf) -> Self {
        self.client_cert_path = Some(cert_path);
        self.client_key_path = Some(key_path);
        self
    }

    /// Sets whether to verify the server's certificate.
    pub fn with_verify_cert(mut self, verify: bool) -> Self {
        self.verify_cert = verify;
        self
    }

    /// Sets the server name for SNI.
    pub fn with_server_name(mut self, name: String) -> Self {
        self.server_name = Some(name);
        self
    }
}

/// Reconnection strategy configuration.
///
/// Defines how the system should handle connection failures and attempt
/// to reconnect to TAK servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectConfig {
    /// Whether to automatically reconnect on failure
    #[serde(default = "default_auto_reconnect")]
    pub auto_reconnect: bool,

    /// Initial delay before first reconnection attempt
    #[serde(
        default = "default_initial_delay",
        with = "humantime_serde",
        rename = "initial_delay_secs"
    )]
    pub initial_delay: Duration,

    /// Maximum delay between reconnection attempts
    #[serde(
        default = "default_max_delay",
        with = "humantime_serde",
        rename = "max_delay_secs"
    )]
    pub max_delay: Duration,

    /// Multiplier for exponential backoff
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,

    /// Maximum number of reconnection attempts (None = infinite)
    pub max_attempts: Option<u32>,
}

fn default_auto_reconnect() -> bool {
    true
}

fn default_initial_delay() -> Duration {
    Duration::from_secs(1)
}

fn default_max_delay() -> Duration {
    Duration::from_secs(300) // 5 minutes
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300),
            backoff_multiplier: 2.0,
            max_attempts: None,
        }
    }
}

/// Configuration for a TAK server connection.
///
/// Contains all parameters needed to establish and maintain a connection
/// to a TAK server, including network details, authentication, and retry logic.
///
/// # Examples
///
/// ```
/// use omnitak_core::types::{ServerConfig, Protocol};
///
/// let config = ServerConfig::builder()
///     .name("tak-server-1")
///     .host("192.168.1.100")
///     .port(8089)
///     .protocol(Protocol::Tls)
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Unique name for this server configuration
    pub name: String,

    /// Hostname or IP address
    pub host: String,

    /// Port number
    pub port: u16,

    /// Transport protocol
    pub protocol: Protocol,

    /// TLS configuration (required for TLS protocol)
    pub tls: Option<TlsConfig>,

    /// Reconnection strategy
    #[serde(default)]
    pub reconnect: ReconnectConfig,

    /// Connection timeout
    #[serde(
        default = "default_connect_timeout",
        with = "humantime_serde",
        rename = "connect_timeout_secs"
    )]
    pub connect_timeout: Duration,

    /// Read timeout for receiving data
    #[serde(
        default = "default_read_timeout",
        with = "humantime_serde",
        rename = "read_timeout_secs"
    )]
    pub read_timeout: Duration,

    /// Whether this server is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Optional tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_connect_timeout() -> Duration {
    Duration::from_secs(10)
}

fn default_read_timeout() -> Duration {
    Duration::from_secs(30)
}

fn default_enabled() -> bool {
    true
}

impl ServerConfig {
    /// Creates a new builder for ServerConfig.
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
    }

    /// Returns the socket address for this server.
    pub fn socket_addr(&self) -> Result<SocketAddr, std::io::Error> {
        use std::net::ToSocketAddrs;
        let addr_str = format!("{}:{}", self.host, self.port);
        addr_str
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No address found"))
    }

    /// Returns true if this server requires TLS.
    pub fn requires_tls(&self) -> bool {
        self.protocol == Protocol::Tls
    }

    /// Validates the server configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Server name cannot be empty".to_string());
        }

        if self.host.is_empty() {
            return Err("Server host cannot be empty".to_string());
        }

        if self.port == 0 {
            return Err("Server port cannot be 0".to_string());
        }

        if self.requires_tls() && self.tls.is_none() {
            return Err("TLS configuration required for TLS protocol".to_string());
        }

        Ok(())
    }
}

/// Builder for ServerConfig.
#[derive(Default)]
pub struct ServerConfigBuilder {
    name: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    protocol: Option<Protocol>,
    tls: Option<TlsConfig>,
    reconnect: Option<ReconnectConfig>,
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    enabled: Option<bool>,
    tags: Vec<String>,
}

impl ServerConfigBuilder {
    /// Sets the server name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the server host.
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets the server port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Sets the protocol.
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = Some(protocol);
        self
    }

    /// Sets the TLS configuration.
    pub fn tls(mut self, tls: TlsConfig) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Sets the reconnect configuration.
    pub fn reconnect(mut self, reconnect: ReconnectConfig) -> Self {
        self.reconnect = Some(reconnect);
        self
    }

    /// Sets the connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Sets the read timeout.
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = Some(timeout);
        self
    }

    /// Sets whether the server is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Adds a tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Sets all tags.
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Builds the ServerConfig.
    pub fn build(self) -> ServerConfig {
        let protocol = self.protocol.unwrap_or(Protocol::Tcp);
        ServerConfig {
            name: self.name.unwrap_or_else(|| "unnamed".to_string()),
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or_else(|| protocol.default_port()),
            protocol,
            tls: self.tls,
            reconnect: self.reconnect.unwrap_or_default(),
            connect_timeout: self.connect_timeout.unwrap_or_else(default_connect_timeout),
            read_timeout: self.read_timeout.unwrap_or_else(default_read_timeout),
            enabled: self.enabled.unwrap_or(true),
            tags: self.tags,
        }
    }
}

/// Connection metadata and statistics.
///
/// Tracks runtime information about an active connection to a TAK server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    /// Unique connection identifier
    pub connection_id: ConnectionId,

    /// Server configuration name
    pub server_name: String,

    /// Current connection status
    pub status: ServerStatus,

    /// When the connection was established (if connected)
    pub connected_at: Option<DateTime<Utc>>,

    /// When the connection was last disconnected (if applicable)
    pub disconnected_at: Option<DateTime<Utc>>,

    /// Number of reconnection attempts
    pub reconnect_attempts: u32,

    /// Number of messages received
    pub messages_received: u64,

    /// Number of messages sent
    pub messages_sent: u64,

    /// Number of bytes received
    pub bytes_received: u64,

    /// Number of bytes sent
    pub bytes_sent: u64,

    /// Last error message (if any)
    pub last_error: Option<String>,
}

impl ConnectionMetadata {
    /// Creates new connection metadata.
    pub fn new(connection_id: ConnectionId, server_name: String) -> Self {
        Self {
            connection_id,
            server_name,
            status: ServerStatus::Disconnected,
            connected_at: None,
            disconnected_at: None,
            reconnect_attempts: 0,
            messages_received: 0,
            messages_sent: 0,
            bytes_received: 0,
            bytes_sent: 0,
            last_error: None,
        }
    }

    /// Marks the connection as connected.
    pub fn mark_connected(&mut self) {
        self.status = ServerStatus::Connected;
        self.connected_at = Some(Utc::now());
        self.reconnect_attempts = 0;
        self.last_error = None;
    }

    /// Marks the connection as disconnected.
    pub fn mark_disconnected(&mut self, error: Option<String>) {
        self.status = ServerStatus::Disconnected;
        self.disconnected_at = Some(Utc::now());
        self.last_error = error;
    }

    /// Marks the connection as reconnecting.
    pub fn mark_reconnecting(&mut self) {
        self.status = ServerStatus::Reconnecting;
        self.reconnect_attempts += 1;
    }

    /// Marks the connection as failed.
    pub fn mark_failed(&mut self, error: String) {
        self.status = ServerStatus::Failed;
        self.disconnected_at = Some(Utc::now());
        self.last_error = Some(error);
    }

    /// Records a received message.
    pub fn record_message_received(&mut self, bytes: u64) {
        self.messages_received += 1;
        self.bytes_received += bytes;
    }

    /// Records a sent message.
    pub fn record_message_sent(&mut self, bytes: u64) {
        self.messages_sent += 1;
        self.bytes_sent += bytes;
    }

    /// Returns the uptime duration if connected.
    pub fn uptime(&self) -> Option<Duration> {
        self.connected_at.map(|connected| {
            let now = Utc::now();
            (now - connected).to_std().unwrap_or_default()
        })
    }
}

// Custom serde module for Duration using humantime
mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_id_creation() {
        let id1 = ConnectionId::new();
        let id2 = ConnectionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_protocol_security() {
        assert!(!Protocol::Tcp.is_secure());
        assert!(!Protocol::Udp.is_secure());
        assert!(Protocol::Tls.is_secure());
        assert!(Protocol::WebSocket.is_secure());
    }

    #[test]
    fn test_protocol_stream_based() {
        assert!(Protocol::Tcp.is_stream_based());
        assert!(!Protocol::Udp.is_stream_based());
        assert!(Protocol::Tls.is_stream_based());
        assert!(Protocol::WebSocket.is_stream_based());
    }

    #[test]
    fn test_server_status() {
        assert!(ServerStatus::Connected.is_connected());
        assert!(!ServerStatus::Disconnected.is_connected());
        assert!(ServerStatus::Reconnecting.is_recoverable());
        assert!(!ServerStatus::Failed.is_recoverable());
    }

    #[test]
    fn test_server_config_builder() {
        let config = ServerConfig::builder()
            .name("test-server")
            .host("127.0.0.1")
            .port(8089)
            .protocol(Protocol::Tcp)
            .enabled(true)
            .tag("production")
            .build();

        assert_eq!(config.name, "test-server");
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8089);
        assert_eq!(config.protocol, Protocol::Tcp);
        assert!(config.enabled);
        assert_eq!(config.tags, vec!["production"]);
    }

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::builder()
            .name("test")
            .host("localhost")
            .port(8089)
            .protocol(Protocol::Tcp)
            .build();

        assert!(config.validate().is_ok());

        config.name = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_connection_metadata() {
        let id = ConnectionId::new();
        let mut metadata = ConnectionMetadata::new(id, "test-server".to_string());

        assert_eq!(metadata.status, ServerStatus::Disconnected);
        assert_eq!(metadata.reconnect_attempts, 0);

        metadata.mark_connected();
        assert!(metadata.status.is_connected());
        assert!(metadata.connected_at.is_some());

        metadata.record_message_received(100);
        assert_eq!(metadata.messages_received, 1);
        assert_eq!(metadata.bytes_received, 100);
    }

    #[test]
    fn test_tls_config_builder() {
        let config = TlsConfig::new("/path/to/ca.pem".into())
            .with_client_cert("/path/to/cert.pem".into(), "/path/to/key.pem".into())
            .with_verify_cert(true)
            .with_server_name("example.com".to_string());

        assert!(config.client_cert_path.is_some());
        assert!(config.client_key_path.is_some());
        assert!(config.verify_cert);
        assert_eq!(config.server_name, Some("example.com".to_string()));
    }
}
