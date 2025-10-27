//! Configuration management for the OmniTAK TAK server aggregator.
//!
//! This module provides a comprehensive configuration system that supports:
//! - Loading from YAML files
//! - Environment variable overrides
//! - Validation of all settings
//! - Server definitions, filter rules, TLS settings, and logging configuration

use crate::error::{ConfigError, Result};
use crate::types::{ServerConfig, TlsConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::Level;

/// Main application configuration.
///
/// This is the root configuration structure that contains all settings
/// for the OmniTAK TAK server aggregator. It can be loaded from YAML files
/// and merged with environment variables.
///
/// # Examples
///
/// ```no_run
/// use omnitak_core::config::AppConfig;
///
/// // Load from file
/// let config = AppConfig::from_file("config.yaml").unwrap();
///
/// // Validate before use
/// config.validate().unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application-wide settings
    #[serde(default)]
    pub app: ApplicationConfig,

    /// List of TAK servers to connect to
    #[serde(default)]
    pub servers: Vec<ServerConfig>,

    /// Filter rules for message processing
    #[serde(default)]
    pub filters: FilterConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// API server configuration
    #[serde(default)]
    pub api: ApiConfig,

    /// Metrics configuration
    #[serde(default)]
    pub metrics: MetricsConfig,

    /// Storage configuration
    #[serde(default)]
    pub storage: StorageConfig,
}

impl AppConfig {
    /// Creates a new default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads configuration from a YAML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path).map_err(|e| {
            ConfigError::LoadFailed {
                path: path.display().to_string(),
                reason: e.to_string(),
            }
        })?;

        Self::from_yaml(&contents)
    }

    /// Loads configuration from a YAML string.
    ///
    /// # Errors
    ///
    /// Returns an error if the YAML cannot be parsed.
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).map_err(|e| {
            ConfigError::InvalidFormat {
                reason: e.to_string(),
            }
            .into()
        })
    }

    /// Loads configuration using the `config` crate, which supports
    /// multiple sources and environment variable overrides.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration cannot be loaded or merged.
    pub fn from_config_builder<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        let config = config::Config::builder()
            // Start with default values
            .add_source(config::File::from(path).required(true))
            // Override with environment variables (OMNICOT_*)
            .add_source(
                config::Environment::with_prefix("OMNICOT")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .map_err(|e| ConfigError::LoadFailed {
                path: path.display().to_string(),
                reason: e.to_string(),
            })?;

        config.try_deserialize().map_err(|e| {
            ConfigError::InvalidFormat {
                reason: e.to_string(),
            }
            .into()
        })
    }

    /// Validates the configuration.
    ///
    /// Checks for:
    /// - At least one server configured
    /// - No duplicate server names
    /// - Valid server configurations
    /// - Valid TLS configurations
    /// - Valid filter rules
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails.
    pub fn validate(&self) -> Result<()> {
        // Check that at least one server is configured
        if self.servers.is_empty() {
            return Err(ConfigError::NoServers.into());
        }

        // Check for duplicate server names
        let mut seen_names = std::collections::HashSet::new();
        for server in &self.servers {
            if !seen_names.insert(&server.name) {
                return Err(ConfigError::DuplicateServerName {
                    name: server.name.clone(),
                }
                .into());
            }
        }

        // Validate each server configuration
        for server in &self.servers {
            server.validate().map_err(|reason| {
                ConfigError::InvalidServerConfig {
                    server: server.name.clone(),
                    reason,
                }
            })?;

            // Validate TLS configuration if present
            if let Some(ref tls) = server.tls {
                self.validate_tls_config(tls, &server.name)?;
            }
        }

        // Validate filter rules
        self.filters.validate()?;

        // Validate API configuration
        self.api.validate()?;

        Ok(())
    }

    /// Validates a TLS configuration.
    fn validate_tls_config(&self, tls: &TlsConfig, server_name: &str) -> Result<()> {
        // Check that CA cert exists
        if !tls.ca_cert_path.exists() {
            return Err(ConfigError::InvalidServerConfig {
                server: server_name.to_string(),
                reason: format!("CA certificate not found: {:?}", tls.ca_cert_path),
            }
            .into());
        }

        // Check client cert and key if configured
        if let Some(ref cert_path) = tls.client_cert_path {
            if !cert_path.exists() {
                return Err(ConfigError::InvalidServerConfig {
                    server: server_name.to_string(),
                    reason: format!("Client certificate not found: {:?}", cert_path),
                }
                .into());
            }
        }

        if let Some(ref key_path) = tls.client_key_path {
            if !key_path.exists() {
                return Err(ConfigError::InvalidServerConfig {
                    server: server_name.to_string(),
                    reason: format!("Client key not found: {:?}", key_path),
                }
                .into());
            }
        }

        // Client cert and key must both be present or both absent
        match (&tls.client_cert_path, &tls.client_key_path) {
            (Some(_), None) | (None, Some(_)) => {
                return Err(ConfigError::InvalidServerConfig {
                    server: server_name.to_string(),
                    reason: "Client cert and key must both be specified or both omitted".to_string(),
                }
                .into());
            }
            _ => {}
        }

        Ok(())
    }

    /// Merges this configuration with another, with the other taking precedence.
    pub fn merge(&mut self, other: AppConfig) {
        self.app = other.app;
        self.servers.extend(other.servers);
        self.filters = other.filters;
        self.logging = other.logging;
        self.api = other.api;
        self.metrics = other.metrics;
        self.storage = other.storage;
    }

    /// Returns a server configuration by name.
    pub fn get_server(&self, name: &str) -> Option<&ServerConfig> {
        self.servers.iter().find(|s| s.name == name)
    }

    /// Returns all enabled servers.
    pub fn enabled_servers(&self) -> Vec<&ServerConfig> {
        self.servers.iter().filter(|s| s.enabled).collect()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: ApplicationConfig::default(),
            servers: Vec::new(),
            filters: FilterConfig::default(),
            logging: LoggingConfig::default(),
            api: ApiConfig::default(),
            metrics: MetricsConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

/// Application-wide settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Application name
    #[serde(default = "default_app_name")]
    pub name: String,

    /// Application version
    #[serde(default = "default_app_version")]
    pub version: String,

    /// Environment (development, staging, production)
    #[serde(default = "default_environment")]
    pub environment: String,

    /// Number of worker threads (0 = number of CPUs)
    #[serde(default)]
    pub worker_threads: usize,

    /// Maximum number of concurrent connections
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    /// Graceful shutdown timeout in seconds
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout_secs: u64,
}

fn default_app_name() -> String {
    "omnitak".to_string()
}

fn default_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_environment() -> String {
    "production".to_string()
}

fn default_max_connections() -> usize {
    1000
}

fn default_shutdown_timeout() -> u64 {
    30
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            version: default_app_version(),
            environment: default_environment(),
            worker_threads: 0,
            max_connections: default_max_connections(),
            shutdown_timeout_secs: default_shutdown_timeout(),
        }
    }
}

/// Filter configuration for message processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Whether filtering is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Filter mode: "whitelist" or "blacklist"
    #[serde(default = "default_filter_mode")]
    pub mode: FilterMode,

    /// List of filter rules
    #[serde(default)]
    pub rules: Vec<FilterRule>,

    /// Default action when no rules match
    #[serde(default)]
    pub default_action: FilterAction,
}

fn default_true() -> bool {
    true
}

fn default_filter_mode() -> FilterMode {
    FilterMode::Blacklist
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: FilterMode::Blacklist,
            rules: Vec::new(),
            default_action: FilterAction::Accept,
        }
    }
}

impl FilterConfig {
    /// Validates the filter configuration.
    pub fn validate(&self) -> Result<()> {
        for (i, rule) in self.rules.iter().enumerate() {
            rule.validate().map_err(|e| ConfigError::InvalidFilterRule {
                reason: format!("Rule #{}: {}", i + 1, e),
            })?;
        }
        Ok(())
    }
}

/// Filter mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterMode {
    /// Only allow messages that match rules
    Whitelist,
    /// Allow all messages except those that match rules
    Blacklist,
}

/// Filter action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterAction {
    /// Accept the message
    Accept,
    /// Reject the message
    Reject,
    /// Modify the message
    Modify,
}

impl Default for FilterAction {
    fn default() -> Self {
        FilterAction::Accept
    }
}

/// A single filter rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// Rule name
    pub name: String,

    /// Rule description
    #[serde(default)]
    pub description: String,

    /// Whether the rule is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Field to match against
    pub field: String,

    /// Match operator
    pub operator: FilterOperator,

    /// Value to match
    pub value: String,

    /// Action to take if rule matches
    pub action: FilterAction,
}

impl FilterRule {
    /// Validates the filter rule.
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(ConfigError::InvalidFilterRule {
                reason: "Rule name cannot be empty".to_string(),
            }
            .into());
        }

        if self.field.is_empty() {
            return Err(ConfigError::InvalidFilterRule {
                reason: "Field name cannot be empty".to_string(),
            }
            .into());
        }

        Ok(())
    }
}

/// Filter match operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    /// Exact match
    Equals,
    /// Not equal
    NotEquals,
    /// Contains substring
    Contains,
    /// Does not contain substring
    NotContains,
    /// Regular expression match
    Regex,
    /// Starts with
    StartsWith,
    /// Ends with
    EndsWith,
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format: "text" or "json"
    #[serde(default = "default_log_format")]
    pub format: LogFormat,

    /// Whether to log to stdout
    #[serde(default = "default_true")]
    pub stdout: bool,

    /// Optional log file path
    pub file: Option<PathBuf>,

    /// Whether to include timestamps
    #[serde(default = "default_true")]
    pub timestamps: bool,

    /// Whether to include file/line info
    #[serde(default)]
    pub file_line: bool,

    /// Per-module log levels
    #[serde(default)]
    pub module_levels: HashMap<String, String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> LogFormat {
    LogFormat::Text
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: LogFormat::Text,
            stdout: true,
            file: None,
            timestamps: true,
            file_line: false,
            module_levels: HashMap::new(),
        }
    }
}

impl LoggingConfig {
    /// Parses the log level string to a tracing Level.
    pub fn parse_level(&self) -> Result<Level> {
        self.level
            .parse()
            .map_err(|_| ConfigError::InvalidValue {
                field: "logging.level".to_string(),
                reason: format!("Invalid log level: {}", self.level),
            }
            .into())
    }
}

/// Log format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable text format
    Text,
    /// JSON format for structured logging
    Json,
}

/// API server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Whether the API server is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// API server bind address
    #[serde(default = "default_api_host")]
    pub host: String,

    /// API server port
    #[serde(default = "default_api_port")]
    pub port: u16,

    /// Whether to enable CORS
    #[serde(default = "default_true")]
    pub cors: bool,

    /// Maximum request body size in bytes
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,

    /// Request timeout in seconds
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,

    /// Optional TLS configuration for HTTPS
    pub tls: Option<TlsConfig>,
}

fn default_api_host() -> String {
    "0.0.0.0".to_string()
}

fn default_api_port() -> u16 {
    8080
}

fn default_max_body_size() -> usize {
    1024 * 1024 // 1MB
}

fn default_request_timeout() -> u64 {
    30
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            host: default_api_host(),
            port: default_api_port(),
            cors: true,
            max_body_size: default_max_body_size(),
            request_timeout_secs: default_request_timeout(),
            tls: None,
        }
    }
}

impl ApiConfig {
    /// Validates the API configuration.
    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            return Err(ConfigError::InvalidValue {
                field: "api.port".to_string(),
                reason: "Port cannot be 0".to_string(),
            }
            .into());
        }

        Ok(())
    }

    /// Returns the API server bind address.
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Metrics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether metrics are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Metrics endpoint path
    #[serde(default = "default_metrics_path")]
    pub path: String,

    /// Metrics collection interval in seconds
    #[serde(default = "default_metrics_interval")]
    pub interval_secs: u64,
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}

fn default_metrics_interval() -> u64 {
    60
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: default_metrics_path(),
            interval_secs: default_metrics_interval(),
        }
    }
}

impl MetricsConfig {
    /// Returns the metrics collection interval as a Duration.
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_secs)
    }
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend type
    #[serde(default)]
    pub backend: StorageBackend,

    /// Data directory path
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// Maximum message retention in seconds (0 = unlimited)
    #[serde(default)]
    pub retention_secs: u64,

    /// Maximum storage size in bytes (0 = unlimited)
    #[serde(default)]
    pub max_size_bytes: u64,
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Memory,
            data_dir: default_data_dir(),
            retention_secs: 0,
            max_size_bytes: 0,
        }
    }
}

/// Storage backend type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    /// In-memory storage (volatile)
    Memory,
    /// File-based storage
    File,
    /// SQLite database
    Sqlite,
    /// PostgreSQL database
    Postgres,
}

impl Default for StorageBackend {
    fn default() -> Self {
        StorageBackend::Memory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Protocol;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.app.name, "omnitak");
        assert!(config.servers.is_empty());
        assert!(config.api.enabled);
        assert_eq!(config.api.port, 8080);
    }

    #[test]
    fn test_config_validation_no_servers() {
        let config = AppConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_duplicate_names() {
        let mut config = AppConfig::default();
        config.servers.push(
            ServerConfig::builder()
                .name("server1")
                .host("localhost")
                .port(8089)
                .protocol(Protocol::Tcp)
                .build(),
        );
        config.servers.push(
            ServerConfig::builder()
                .name("server1")
                .host("localhost")
                .port(8090)
                .protocol(Protocol::Tcp)
                .build(),
        );

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_from_yaml() {
        let yaml = r#"
app:
  name: test-app
  environment: development

servers:
  - name: tak-server-1
    host: 192.168.1.100
    port: 8089
    protocol: tcp
    enabled: true

filters:
  enabled: true
  mode: blacklist
  rules: []

logging:
  level: debug
  format: json

api:
  enabled: true
  host: 127.0.0.1
  port: 8080
"#;

        let config = AppConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.app.name, "test-app");
        assert_eq!(config.app.environment, "development");
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].name, "tak-server-1");
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.logging.format, LogFormat::Json);
    }

    #[test]
    fn test_enabled_servers() {
        let mut config = AppConfig::default();
        config.servers.push(
            ServerConfig::builder()
                .name("server1")
                .host("localhost")
                .port(8089)
                .protocol(Protocol::Tcp)
                .enabled(true)
                .build(),
        );
        config.servers.push(
            ServerConfig::builder()
                .name("server2")
                .host("localhost")
                .port(8090)
                .protocol(Protocol::Tcp)
                .enabled(false)
                .build(),
        );

        let enabled = config.enabled_servers();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "server1");
    }

    #[test]
    fn test_get_server() {
        let mut config = AppConfig::default();
        config.servers.push(
            ServerConfig::builder()
                .name("server1")
                .host("localhost")
                .port(8089)
                .protocol(Protocol::Tcp)
                .build(),
        );

        assert!(config.get_server("server1").is_some());
        assert!(config.get_server("server2").is_none());
    }

    #[test]
    fn test_filter_rule_validation() {
        let rule = FilterRule {
            name: "test-rule".to_string(),
            description: "Test rule".to_string(),
            enabled: true,
            field: "type".to_string(),
            operator: FilterOperator::Equals,
            value: "a-f-G-E-V-M".to_string(),
            action: FilterAction::Accept,
        };

        assert!(rule.validate().is_ok());

        let invalid_rule = FilterRule {
            name: String::new(),
            description: String::new(),
            enabled: true,
            field: "type".to_string(),
            operator: FilterOperator::Equals,
            value: "value".to_string(),
            action: FilterAction::Accept,
        };

        assert!(invalid_rule.validate().is_err());
    }

    #[test]
    fn test_api_bind_address() {
        let api = ApiConfig::default();
        assert_eq!(api.bind_address(), "0.0.0.0:8080");

        let custom_api = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            ..Default::default()
        };
        assert_eq!(custom_api.bind_address(), "127.0.0.1:3000");
    }

    #[test]
    fn test_logging_parse_level() {
        let logging = LoggingConfig {
            level: "debug".to_string(),
            ..Default::default()
        };
        assert!(logging.parse_level().is_ok());

        let invalid = LoggingConfig {
            level: "invalid".to_string(),
            ..Default::default()
        };
        assert!(invalid.parse_level().is_err());
    }

    #[test]
    fn test_metrics_interval() {
        let metrics = MetricsConfig {
            interval_secs: 120,
            ..Default::default()
        };
        assert_eq!(metrics.interval(), Duration::from_secs(120));
    }
}
