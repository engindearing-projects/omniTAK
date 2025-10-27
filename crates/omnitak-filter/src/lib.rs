//! High-performance filtering and routing for Cursor-on-Target (CoT) messages
//!
//! This crate provides military-grade filtering capabilities for CoT messages,
//! including:
//!
//! - MIL-STD-2525 affiliation parsing (friendly, hostile, neutral, etc.)
//! - Multiple filter types (affiliation, geographic, team, group, UID)
//! - High-performance routing engine with multicast/unicast support
//! - YAML configuration with hot-reload capability
//! - Lock-free data structures for concurrent access
//! - Optimized fast-path operations with SIMD acceleration
//!
//! # Performance Goals
//!
//! - Filter evaluation: <100ns per check
//! - Zero allocations in hot paths
//! - Constant-time operations for security-relevant checks
//! - Lock-free concurrent access
//!
//! # Examples
//!
//! ## Basic filtering
//!
//! ```rust
//! use omnitak_filter::affiliation::{Affiliation, CotType};
//! use omnitak_filter::rules::{AffiliationFilter, FilterRule, CotMessage};
//!
//! // Parse CoT type
//! let cot = CotType::parse("a-f-G-E-V-C");
//! assert!(cot.is_friendly());
//!
//! // Filter by affiliation
//! let filter = AffiliationFilter::friendly_only();
//! let msg = CotMessage {
//!     cot_type: "a-f-G-E-V-C",
//!     uid: "TEST-001",
//!     callsign: Some("ALPHA-1"),
//!     group: Some("Blue Force"),
//!     team: Some("Alpha"),
//!     lat: 40.7128,
//!     lon: -74.0060,
//!     hae: Some(100.0),
//! };
//!
//! assert!(filter.evaluate(&msg).is_pass());
//! ```
//!
//! ## Routing messages
//!
//! ```rust
//! use omnitak_filter::router::{Route, RouteTableBuilder};
//! use omnitak_filter::rules::{AffiliationFilter, CotMessage};
//! use std::sync::Arc;
//!
//! let table = RouteTableBuilder::multicast()
//!     .add_route(Route::new(
//!         "friendly".to_string(),
//!         "Route friendly units".to_string(),
//!         Arc::new(AffiliationFilter::friendly_only()),
//!         vec!["blue-team-server".to_string()],
//!         100,
//!     ))
//!     .build();
//!
//! let msg = CotMessage {
//!     cot_type: "a-f-G-E-V-C",
//!     uid: "TEST-001",
//!     callsign: Some("ALPHA-1"),
//!     group: None,
//!     team: None,
//!     lat: 40.0,
//!     lon: -74.0,
//!     hae: None,
//! };
//!
//! let result = table.route(&msg);
//! assert!(result.has_destinations());
//! ```
//!
//! ## Configuration from YAML
//!
//! ```yaml
//! strategy: all
//! default_destination: default-server
//! routes:
//!   - id: friendly-ground
//!     description: Route friendly ground forces
//!     filter:
//!       type: affiliation
//!       allow: [friend, assumedfriend]
//!     destinations: [blue-team-server]
//!     priority: 100
//! ```

pub mod affiliation;
pub mod config;
pub mod fast_path;
pub mod router;
pub mod rules;

// Re-export commonly used types
pub use affiliation::{Affiliation, CotType, Dimension};
pub use config::{FilterConfig, RouteConfig, RoutingConfig};
pub use router::{DestinationId, Route, RouteStrategy, RouteTable, RouteTableBuilder, RoutingResult};
pub use rules::{
    AffiliationFilter, CotMessage, DimensionFilter, FilterResult, FilterRule, FilterStats,
    GeoBoundingBoxFilter, GroupFilter, TeamFilter, UidFilter,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_filtering() {
        let filter = AffiliationFilter::friendly_only();
        let msg = CotMessage {
            cot_type: "a-f-G-E-V-C",
            uid: "TEST-001",
            callsign: Some("ALPHA-1"),
            group: Some("Blue Force"),
            team: Some("Alpha"),
            lat: 40.7128,
            lon: -74.0060,
            hae: Some(100.0),
        };

        assert!(filter.evaluate(&msg).is_pass());
    }

    #[test]
    fn test_cot_type_parsing() {
        let cot = CotType::parse("a-f-G-E-V-C");
        assert_eq!(cot.affiliation, Some(Affiliation::Friend));
        assert_eq!(cot.dimension, Some(Dimension::Ground));
        assert!(cot.is_friendly());
    }
}
