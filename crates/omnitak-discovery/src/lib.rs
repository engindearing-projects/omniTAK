//! mDNS Service Discovery for TAK Servers and ATAK Devices
//!
//! This crate provides automatic network discovery capabilities for TAK infrastructure:
//! - Discover TAK servers advertising via mDNS
//! - Discover ATAK devices on the local network
//! - Announce the OmniTAK aggregator as a discoverable service
//! - Support for RFC 6762 (Multicast DNS) and RFC 6763 (DNS-SD)
//!
//! # Architecture
//!
//! The discovery service runs as a background task that:
//! 1. Continuously browses for TAK-related services on the network
//! 2. Announces the aggregator's presence for other systems to discover
//! 3. Maintains a registry of discovered services with health tracking
//! 4. Notifies subscribers when services appear or disappear
//!
//! # RFC Compliance
//!
//! Currently uses the `mdns-sd` crate which provides good RFC 6762/6763 support.
//! For enhanced compliance or performance requirements, this can be swapped with
//! alternative implementations or custom code.
//!
//! # Example
//!
//! ```no_run
//! use omnitak_discovery::{DiscoveryService, DiscoveryConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = DiscoveryConfig::default();
//!     let service = DiscoveryService::new(config)?;
//!
//!     // Start discovery
//!     service.start().await?;
//!
//!     // Get discovered services
//!     let servers = service.get_discovered_servers().await;
//!
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod error;
pub mod service;
pub mod types;

pub use config::{DiscoveryConfig, ServiceType};
pub use error::{DiscoveryError, Result};
pub use service::DiscoveryService;
pub use types::{DiscoveredService, ServiceStatus};
