//! Error types for the OmniTAK TAK server aggregator.
//!
//! This module provides comprehensive error handling for all operations in the system.
//! All errors implement `std::error::Error` and are serializable for API responses.

use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

/// Result type alias using OmniTAKError as the error type.
pub type Result<T> = std::result::Result<T, OmniTAKError>;

/// Top-level error type for all OmniTAK operations.
///
/// This enum encompasses all possible errors that can occur in the system,
/// from network failures to configuration issues. All variants are serializable
/// for API responses and logging.
#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum OmniTAKError {
    /// Connection-related errors
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    /// Parsing and deserialization errors
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    /// Certificate and TLS errors
    #[error("Certificate error: {0}")]
    Certificate(#[from] CertificateError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    /// Timeout errors
    #[error("Timeout error: {0}")]
    Timeout(#[from] TimeoutError),

    /// Internal errors that shouldn't normally occur
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Errors related to network connections.
///
/// These errors cover the entire lifecycle of a connection, from establishment
/// to disconnection, including protocol-specific issues.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ConnectionError {
    /// Failed to establish a connection
    #[error("Failed to connect to {host}:{port}: {reason}")]
    ConnectionFailed {
        host: String,
        port: u16,
        reason: String,
    },

    /// Connection was unexpectedly closed
    #[error("Connection closed unexpectedly: {reason}")]
    ConnectionClosed { reason: String },

    /// Connection was reset by peer
    #[error("Connection reset by peer")]
    ConnectionReset,

    /// Connection timeout
    #[error("Connection timeout after {timeout_secs}s")]
    ConnectionTimeout { timeout_secs: u64 },

    /// Protocol mismatch or unsupported protocol
    #[error("Unsupported protocol: {protocol}")]
    UnsupportedProtocol { protocol: String },

    /// TLS handshake failed
    #[error("TLS handshake failed: {reason}")]
    TlsHandshakeFailed { reason: String },

    /// WebSocket handshake failed
    #[error("WebSocket handshake failed: {reason}")]
    WebSocketHandshakeFailed { reason: String },

    /// Invalid server response
    #[error("Invalid server response: {details}")]
    InvalidServerResponse { details: String },

    /// Server returned an error
    #[error("Server error: {message}")]
    ServerError { message: String },

    /// Authentication failed
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    /// Maximum reconnection attempts reached
    #[error("Maximum reconnection attempts ({max_attempts}) reached")]
    MaxReconnectAttemptsReached { max_attempts: u32 },

    /// Connection is already established
    #[error("Connection already established")]
    AlreadyConnected,

    /// Connection is not established
    #[error("Not connected")]
    NotConnected,

    /// Network is unreachable
    #[error("Network unreachable: {host}")]
    NetworkUnreachable { host: String },

    /// DNS resolution failed
    #[error("DNS resolution failed for {host}: {reason}")]
    DnsResolutionFailed { host: String, reason: String },

    /// Address already in use
    #[error("Address already in use: {address}")]
    AddressInUse { address: String },

    /// Permission denied
    #[error("Permission denied: {details}")]
    PermissionDenied { details: String },
}

impl ConnectionError {
    /// Creates a connection failed error.
    pub fn failed(host: impl Into<String>, port: u16, reason: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            host: host.into(),
            port,
            reason: reason.into(),
        }
    }

    /// Creates a connection closed error.
    pub fn closed(reason: impl Into<String>) -> Self {
        Self::ConnectionClosed {
            reason: reason.into(),
        }
    }

    /// Creates a TLS handshake failed error.
    pub fn tls_handshake_failed(reason: impl Into<String>) -> Self {
        Self::TlsHandshakeFailed {
            reason: reason.into(),
        }
    }

    /// Returns true if this error is transient and the operation can be retried.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            ConnectionError::ConnectionTimeout { .. }
                | ConnectionError::ConnectionReset
                | ConnectionError::NetworkUnreachable { .. }
                | ConnectionError::DnsResolutionFailed { .. }
        )
    }

    /// Returns true if this error is permanent and retrying won't help.
    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            ConnectionError::AuthenticationFailed { .. }
                | ConnectionError::UnsupportedProtocol { .. }
                | ConnectionError::MaxReconnectAttemptsReached { .. }
                | ConnectionError::PermissionDenied { .. }
        )
    }
}

/// Errors related to parsing and deserialization.
///
/// These errors occur when processing data received from TAK servers or
/// configuration files.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ParseError {
    /// Failed to parse XML data
    #[error("XML parse error at line {line}, column {column}: {message}")]
    XmlError {
        line: usize,
        column: usize,
        message: String,
    },

    /// Failed to parse JSON data
    #[error("JSON parse error at line {line}, column {column}: {message}")]
    JsonError {
        line: usize,
        column: usize,
        message: String,
    },

    /// Failed to parse YAML data
    #[error("YAML parse error: {message}")]
    YamlError { message: String },

    /// Failed to parse protobuf data
    #[error("Protobuf decode error: {message}")]
    ProtobufError { message: String },

    /// Invalid UTF-8 encoding
    #[error("Invalid UTF-8 encoding: {details}")]
    InvalidUtf8 { details: String },

    /// Invalid data format
    #[error("Invalid data format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },

    /// Missing required field
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    /// Invalid field value
    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue { field: String, reason: String },

    /// Invalid CoT (Cursor on Target) message
    #[error("Invalid CoT message: {details}")]
    InvalidCotMessage { details: String },

    /// Invalid timestamp format
    #[error("Invalid timestamp: {value}")]
    InvalidTimestamp { value: String },

    /// Invalid UUID format
    #[error("Invalid UUID: {value}")]
    InvalidUuid { value: String },

    /// Invalid IP address or hostname
    #[error("Invalid address: {value}")]
    InvalidAddress { value: String },

    /// Invalid port number
    #[error("Invalid port: {value}")]
    InvalidPort { value: String },
}

impl ParseError {
    /// Creates an XML parse error.
    pub fn xml(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::XmlError {
            line,
            column,
            message: message.into(),
        }
    }

    /// Creates a JSON parse error.
    pub fn json(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self::JsonError {
            line,
            column,
            message: message.into(),
        }
    }

    /// Creates a missing field error.
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Creates an invalid value error.
    pub fn invalid_value(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidValue {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

/// Errors related to certificates and TLS.
///
/// These errors occur when loading, validating, or using certificates
/// for secure connections.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum CertificateError {
    /// Certificate file not found
    #[error("Certificate file not found: {path}")]
    CertificateNotFound { path: String },

    /// Failed to load certificate
    #[error("Failed to load certificate from {path}: {reason}")]
    LoadFailed { path: String, reason: String },

    /// Invalid certificate format
    #[error("Invalid certificate format in {path}: {reason}")]
    InvalidFormat { path: String, reason: String },

    /// Certificate has expired
    #[error("Certificate has expired: {path}")]
    Expired { path: String },

    /// Certificate is not yet valid
    #[error("Certificate is not yet valid: {path}")]
    NotYetValid { path: String },

    /// Certificate validation failed
    #[error("Certificate validation failed: {reason}")]
    ValidationFailed { reason: String },

    /// Certificate chain is incomplete
    #[error("Incomplete certificate chain: {details}")]
    IncompleteChain { details: String },

    /// Certificate hostname mismatch
    #[error("Certificate hostname mismatch: expected {expected}, got {actual}")]
    HostnameMismatch { expected: String, actual: String },

    /// Certificate is self-signed
    #[error("Self-signed certificate: {path}")]
    SelfSigned { path: String },

    /// Certificate authority is not trusted
    #[error("Untrusted certificate authority: {details}")]
    UntrustedAuthority { details: String },

    /// Private key error
    #[error("Private key error: {reason}")]
    PrivateKeyError { reason: String },

    /// Certificate and key mismatch
    #[error("Certificate and private key do not match")]
    KeyMismatch,

    /// Invalid PEM format
    #[error("Invalid PEM format in {path}: {reason}")]
    InvalidPemFormat { path: String, reason: String },
}

impl CertificateError {
    /// Creates a certificate not found error.
    pub fn not_found(path: impl Into<String>) -> Self {
        Self::CertificateNotFound { path: path.into() }
    }

    /// Creates a certificate load failed error.
    pub fn load_failed(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::LoadFailed {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Creates a validation failed error.
    pub fn validation_failed(reason: impl Into<String>) -> Self {
        Self::ValidationFailed {
            reason: reason.into(),
        }
    }
}

/// Errors related to configuration.
///
/// These errors occur when loading, parsing, or validating configuration files.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    /// Failed to load configuration
    #[error("Failed to load configuration from {path}: {reason}")]
    LoadFailed { path: String, reason: String },

    /// Invalid configuration format
    #[error("Invalid configuration format: {reason}")]
    InvalidFormat { reason: String },

    /// Missing required configuration field
    #[error("Missing required configuration field: {field}")]
    MissingField { field: String },

    /// Invalid configuration value
    #[error("Invalid configuration value for '{field}': {reason}")]
    InvalidValue { field: String, reason: String },

    /// Configuration validation failed
    #[error("Configuration validation failed: {reason}")]
    ValidationFailed { reason: String },

    /// Duplicate server name
    #[error("Duplicate server name: {name}")]
    DuplicateServerName { name: String },

    /// No servers configured
    #[error("No servers configured")]
    NoServers,

    /// Invalid server configuration
    #[error("Invalid server configuration for '{server}': {reason}")]
    InvalidServerConfig { server: String, reason: String },

    /// Invalid filter rule
    #[error("Invalid filter rule: {reason}")]
    InvalidFilterRule { reason: String },

    /// Environment variable error
    #[error("Environment variable error: {details}")]
    EnvironmentVariableError { details: String },

    /// Configuration merge conflict
    #[error("Configuration merge conflict: {details}")]
    MergeConflict { details: String },
}

impl ConfigError {
    /// Creates a file not found error.
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Creates a missing field error.
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Creates an invalid value error.
    pub fn invalid_value(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidValue {
            field: field.into(),
            reason: reason.into(),
        }
    }

    /// Creates a validation failed error.
    pub fn validation_failed(reason: impl Into<String>) -> Self {
        Self::ValidationFailed {
            reason: reason.into(),
        }
    }
}

/// Wrapper for I/O errors to make them serializable.
#[derive(Debug, Error, Serialize, Deserialize)]
#[error("I/O error: {kind:?}: {message}")]
pub struct IoError {
    pub kind: IoErrorKind,
    pub message: String,
}

impl From<io::Error> for IoError {
    fn from(err: io::Error) -> Self {
        Self {
            kind: err.kind().into(),
            message: err.to_string(),
        }
    }
}

impl From<io::Error> for OmniTAKError {
    fn from(err: io::Error) -> Self {
        OmniTAKError::Io(err.into())
    }
}

/// Serializable version of std::io::ErrorKind.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IoErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    Interrupted,
    UnexpectedEof,
    Other,
}

impl From<io::ErrorKind> for IoErrorKind {
    fn from(kind: io::ErrorKind) -> Self {
        match kind {
            io::ErrorKind::NotFound => IoErrorKind::NotFound,
            io::ErrorKind::PermissionDenied => IoErrorKind::PermissionDenied,
            io::ErrorKind::ConnectionRefused => IoErrorKind::ConnectionRefused,
            io::ErrorKind::ConnectionReset => IoErrorKind::ConnectionReset,
            io::ErrorKind::ConnectionAborted => IoErrorKind::ConnectionAborted,
            io::ErrorKind::NotConnected => IoErrorKind::NotConnected,
            io::ErrorKind::AddrInUse => IoErrorKind::AddrInUse,
            io::ErrorKind::AddrNotAvailable => IoErrorKind::AddrNotAvailable,
            io::ErrorKind::BrokenPipe => IoErrorKind::BrokenPipe,
            io::ErrorKind::AlreadyExists => IoErrorKind::AlreadyExists,
            io::ErrorKind::WouldBlock => IoErrorKind::WouldBlock,
            io::ErrorKind::InvalidInput => IoErrorKind::InvalidInput,
            io::ErrorKind::InvalidData => IoErrorKind::InvalidData,
            io::ErrorKind::TimedOut => IoErrorKind::TimedOut,
            io::ErrorKind::WriteZero => IoErrorKind::WriteZero,
            io::ErrorKind::Interrupted => IoErrorKind::Interrupted,
            io::ErrorKind::UnexpectedEof => IoErrorKind::UnexpectedEof,
            _ => IoErrorKind::Other,
        }
    }
}

/// Timeout errors for various operations.
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum TimeoutError {
    /// Operation timed out
    #[error("Operation timed out after {timeout_secs}s: {operation}")]
    OperationTimeout {
        operation: String,
        timeout_secs: u64,
    },

    /// Read operation timed out
    #[error("Read timeout after {timeout_secs}s")]
    ReadTimeout { timeout_secs: u64 },

    /// Write operation timed out
    #[error("Write timeout after {timeout_secs}s")]
    WriteTimeout { timeout_secs: u64 },

    /// Connection timeout
    #[error("Connection timeout after {timeout_secs}s")]
    ConnectTimeout { timeout_secs: u64 },
}

impl TimeoutError {
    /// Creates an operation timeout error.
    pub fn operation(operation: impl Into<String>, timeout_secs: u64) -> Self {
        Self::OperationTimeout {
            operation: operation.into(),
            timeout_secs,
        }
    }
}

/// Extension trait for converting Results to OmniTAKError.
pub trait ResultExt<T> {
    /// Converts the error to an internal error with context.
    fn internal_context(self, context: &str) -> Result<T>;
}

impl<T, E: std::error::Error> ResultExt<T> for std::result::Result<T, E> {
    fn internal_context(self, context: &str) -> Result<T> {
        self.map_err(|e| OmniTAKError::Internal(format!("{}: {}", context, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_error_transient() {
        let err = ConnectionError::ConnectionTimeout { timeout_secs: 30 };
        assert!(err.is_transient());
        assert!(!err.is_permanent());
    }

    #[test]
    fn test_connection_error_permanent() {
        let err = ConnectionError::AuthenticationFailed {
            reason: "invalid credentials".to_string(),
        };
        assert!(!err.is_transient());
        assert!(err.is_permanent());
    }

    #[test]
    fn test_error_serialization() {
        let err = OmniTAKError::Connection(ConnectionError::failed(
            "example.com",
            8089,
            "connection refused",
        ));
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("Connection"));
        assert!(json.contains("example.com"));
    }

    #[test]
    fn test_parse_error_helpers() {
        let err = ParseError::missing_field("uid");
        assert!(matches!(err, ParseError::MissingField { .. }));

        let err = ParseError::invalid_value("port", "must be between 1 and 65535");
        assert!(matches!(err, ParseError::InvalidValue { .. }));
    }

    #[test]
    fn test_certificate_error_helpers() {
        let err = CertificateError::not_found("/path/to/cert.pem");
        assert!(matches!(err, CertificateError::CertificateNotFound { .. }));

        let err = CertificateError::validation_failed("certificate expired");
        assert!(matches!(err, CertificateError::ValidationFailed { .. }));
    }

    #[test]
    fn test_config_error_helpers() {
        let err = ConfigError::file_not_found("/etc/omnitak/config.yaml");
        assert!(matches!(err, ConfigError::FileNotFound { .. }));

        let err = ConfigError::missing_field("servers");
        assert!(matches!(err, ConfigError::MissingField { .. }));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let omnitak_err: OmniTAKError = io_err.into();
        assert!(matches!(omnitak_err, OmniTAKError::Io(_)));
    }

    #[test]
    fn test_timeout_error() {
        let err = TimeoutError::operation("database query", 30);
        let display = format!("{}", err);
        assert!(display.contains("database query"));
        assert!(display.contains("30"));
    }
}
