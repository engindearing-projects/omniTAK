//! Types for discovered services and their metadata

use crate::config::ServiceType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use uuid::Uuid;

/// A discovered service on the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Unique identifier for this discovered service
    pub id: String,

    /// Service type (TAK server, ATAK device, etc.)
    pub service_type: ServiceType,

    /// Instance name (e.g., "TAK Server - Alpha")
    pub instance_name: String,

    /// Hostname or domain name
    pub hostname: String,

    /// IP addresses (can be multiple for dual-stack)
    pub addresses: Vec<IpAddr>,

    /// Service port
    pub port: u16,

    /// TXT record properties (key-value metadata)
    pub properties: HashMap<String, String>,

    /// Current status of the service
    pub status: ServiceStatus,

    /// First discovered timestamp
    pub discovered_at: DateTime<Utc>,

    /// Last seen timestamp (updated when service is refreshed)
    pub last_seen_at: DateTime<Utc>,

    /// Number of times this service has been seen
    pub seen_count: u64,
}

impl DiscoveredService {
    /// Creates a new discovered service
    pub fn new(
        service_type: ServiceType,
        instance_name: String,
        hostname: String,
        addresses: Vec<IpAddr>,
        port: u16,
        properties: HashMap<String, String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            service_type,
            instance_name,
            hostname,
            addresses,
            port,
            properties,
            status: ServiceStatus::Active,
            discovered_at: now,
            last_seen_at: now,
            seen_count: 1,
        }
    }

    /// Updates the last seen timestamp
    pub fn mark_seen(&mut self) {
        self.last_seen_at = Utc::now();
        self.seen_count += 1;
        self.status = ServiceStatus::Active;
    }

    /// Marks the service as stale (not seen recently)
    pub fn mark_stale(&mut self) {
        self.status = ServiceStatus::Stale;
    }

    /// Marks the service as lost (removed from network)
    pub fn mark_lost(&mut self) {
        self.status = ServiceStatus::Lost;
    }

    /// Returns the primary address (prefer IPv4)
    pub fn primary_address(&self) -> Option<IpAddr> {
        // Prefer IPv4 addresses
        self.addresses
            .iter()
            .find(|addr| addr.is_ipv4())
            .or_else(|| self.addresses.first())
            .copied()
    }

    /// Returns the connection string (host:port)
    pub fn connection_string(&self) -> String {
        if let Some(addr) = self.primary_address() {
            format!("{}:{}", addr, self.port)
        } else {
            format!("{}:{}", self.hostname, self.port)
        }
    }

    /// Checks if the service supports TLS based on properties
    pub fn supports_tls(&self) -> bool {
        self.properties
            .get("tls")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
    }

    /// Gets the service version if available
    pub fn version(&self) -> Option<&str> {
        self.properties.get("version").map(|s| s.as_str())
    }

    /// Gets the service description if available
    pub fn description(&self) -> Option<&str> {
        self.properties.get("description").map(|s| s.as_str())
    }

    /// Checks if the service is considered alive
    pub fn is_alive(&self) -> bool {
        matches!(self.status, ServiceStatus::Active)
    }

    /// Returns age in seconds since last seen
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.last_seen_at).num_seconds()
    }
}

/// Status of a discovered service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    /// Service is active and responding
    Active,

    /// Service hasn't been seen recently but not yet timed out
    Stale,

    /// Service has been removed or timed out
    Lost,
}

impl ServiceStatus {
    /// Returns a human-readable description
    pub fn description(&self) -> &str {
        match self {
            ServiceStatus::Active => "Active and responding",
            ServiceStatus::Stale => "Not seen recently",
            ServiceStatus::Lost => "No longer available",
        }
    }
}

/// Event emitted when a service changes state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEvent {
    /// Event type
    pub event_type: ServiceEventType,

    /// The service that triggered the event
    pub service: DiscoveredService,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,
}

impl ServiceEvent {
    pub fn new(event_type: ServiceEventType, service: DiscoveredService) -> Self {
        Self {
            event_type,
            service,
            timestamp: Utc::now(),
        }
    }
}

/// Types of service events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceEventType {
    /// New service discovered
    Discovered,

    /// Service information updated
    Updated,

    /// Service marked as stale
    Stale,

    /// Service lost or removed
    Lost,
}

impl ServiceEventType {
    pub fn description(&self) -> &str {
        match self {
            ServiceEventType::Discovered => "Service discovered on network",
            ServiceEventType::Updated => "Service information updated",
            ServiceEventType::Stale => "Service not seen recently",
            ServiceEventType::Lost => "Service lost or removed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_discovered_service_creation() {
        let service = DiscoveredService::new(
            ServiceType::TakServer,
            "Test Server".to_string(),
            "test.local".to_string(),
            vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))],
            8089,
            HashMap::new(),
        );

        assert_eq!(service.instance_name, "Test Server");
        assert_eq!(service.port, 8089);
        assert_eq!(service.status, ServiceStatus::Active);
    }

    #[test]
    fn test_connection_string() {
        let service = DiscoveredService::new(
            ServiceType::TakServer,
            "Test".to_string(),
            "test.local".to_string(),
            vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))],
            8089,
            HashMap::new(),
        );

        assert_eq!(service.connection_string(), "192.168.1.100:8089");
    }

    #[test]
    fn test_tls_detection() {
        let mut props = HashMap::new();
        props.insert("tls".to_string(), "true".to_string());

        let service = DiscoveredService::new(
            ServiceType::TakServer,
            "Test".to_string(),
            "test.local".to_string(),
            vec![],
            8089,
            props,
        );

        assert!(service.supports_tls());
    }

    #[test]
    fn test_status_transitions() {
        let mut service = DiscoveredService::new(
            ServiceType::TakServer,
            "Test".to_string(),
            "test.local".to_string(),
            vec![],
            8089,
            HashMap::new(),
        );

        assert!(service.is_alive());

        service.mark_stale();
        assert_eq!(service.status, ServiceStatus::Stale);
        assert!(!service.is_alive());

        service.mark_lost();
        assert_eq!(service.status, ServiceStatus::Lost);
    }
}
