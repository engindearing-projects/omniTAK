//! Configuration types for service discovery

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the mDNS discovery service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable automatic service discovery
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Enable announcing this aggregator via mDNS
    #[serde(default = "default_announce_enabled")]
    pub announce_enabled: bool,

    /// Service types to discover
    #[serde(default = "default_service_types")]
    pub service_types: Vec<ServiceType>,

    /// Port to announce for this aggregator's REST API
    #[serde(default = "default_announce_port")]
    pub announce_port: u16,

    /// Instance name for this aggregator (auto-generated if not set)
    #[serde(default)]
    pub instance_name: Option<String>,

    /// How often to check for stale services (seconds)
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval_secs: u64,

    /// How long before a service is considered stale (seconds)
    #[serde(default = "default_stale_timeout")]
    pub stale_timeout_secs: u64,

    /// Automatically add discovered TAK servers to the connection pool
    #[serde(default = "default_auto_connect")]
    pub auto_connect: bool,

    /// Only auto-connect to servers with valid TLS certificates
    #[serde(default = "default_require_tls")]
    pub require_tls: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            announce_enabled: default_announce_enabled(),
            service_types: default_service_types(),
            announce_port: default_announce_port(),
            instance_name: None,
            cleanup_interval_secs: default_cleanup_interval(),
            stale_timeout_secs: default_stale_timeout(),
            auto_connect: default_auto_connect(),
            require_tls: default_require_tls(),
        }
    }
}

impl DiscoveryConfig {
    /// Returns the cleanup interval as a Duration
    pub fn cleanup_interval(&self) -> Duration {
        Duration::from_secs(self.cleanup_interval_secs)
    }

    /// Returns the stale timeout as a Duration
    pub fn stale_timeout(&self) -> Duration {
        Duration::from_secs(self.stale_timeout_secs)
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.announce_port == 0 {
            return Err("announce_port cannot be 0".to_string());
        }

        if self.cleanup_interval_secs == 0 {
            return Err("cleanup_interval_secs cannot be 0".to_string());
        }

        if self.stale_timeout_secs == 0 {
            return Err("stale_timeout_secs cannot be 0".to_string());
        }

        if self.service_types.is_empty() {
            return Err("at least one service type must be configured".to_string());
        }

        Ok(())
    }
}

/// Types of services to discover
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    /// TAK servers (streaming CoT data)
    /// Typical service: _tak._tcp.local
    TakServer,

    /// ATAK devices (Android Team Awareness Kit)
    /// Typical service: _atak._tcp.local
    AtakDevice,

    /// TAK aggregators (like this service)
    /// Typical service: _tak-aggregator._tcp.local
    TakAggregator,

    /// Custom service type
    Custom(String),
}

impl ServiceType {
    /// Returns the mDNS service type string
    pub fn to_service_string(&self) -> String {
        match self {
            ServiceType::TakServer => "_tak._tcp.local.".to_string(),
            ServiceType::AtakDevice => "_atak._tcp.local.".to_string(),
            ServiceType::TakAggregator => "_tak-aggregator._tcp.local.".to_string(),
            ServiceType::Custom(s) => {
                if s.ends_with('.') {
                    s.clone()
                } else {
                    format!("{}.local.", s)
                }
            }
        }
    }

    /// Creates a ServiceType from an mDNS service string
    pub fn from_service_string(s: &str) -> Self {
        let normalized = s.trim_end_matches('.');
        match normalized {
            "_tak._tcp.local" => ServiceType::TakServer,
            "_atak._tcp.local" => ServiceType::AtakDevice,
            "_tak-aggregator._tcp.local" => ServiceType::TakAggregator,
            _ => ServiceType::Custom(s.to_string()),
        }
    }

    /// Returns a human-readable description
    pub fn description(&self) -> &str {
        match self {
            ServiceType::TakServer => "TAK Server",
            ServiceType::AtakDevice => "ATAK Device",
            ServiceType::TakAggregator => "TAK Aggregator",
            ServiceType::Custom(_) => "Custom Service",
        }
    }
}

// Default configuration values
fn default_enabled() -> bool {
    true
}

fn default_announce_enabled() -> bool {
    true
}

fn default_service_types() -> Vec<ServiceType> {
    vec![
        ServiceType::TakServer,
        ServiceType::AtakDevice,
        ServiceType::TakAggregator,
    ]
}

fn default_announce_port() -> u16 {
    8080
}

fn default_cleanup_interval() -> u64 {
    30 // Check every 30 seconds
}

fn default_stale_timeout() -> u64 {
    300 // 5 minutes
}

fn default_auto_connect() -> bool {
    false // Require manual approval by default
}

fn default_require_tls() -> bool {
    true // Require TLS for auto-connect
}
