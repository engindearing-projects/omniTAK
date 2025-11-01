//! High-performance routing engine for CoT messages
//!
//! Routes messages to specific destinations based on filter rules.
//! Uses lock-free data structures for concurrent access.

use crate::rules::{CotMessage, FilterResult, FilterRule, FilterStats};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, trace};

/// Destination identifier for routing
pub type DestinationId = String;

/// A route maps a filter to one or more destinations
#[derive(Clone)]
pub struct Route {
    /// Unique route identifier
    pub id: String,
    /// Description of this route
    pub description: String,
    /// Filter rule for this route
    pub filter: Arc<dyn FilterRule>,
    /// Destination IDs for messages that pass the filter
    pub destinations: Vec<DestinationId>,
    /// Priority (higher priority routes evaluated first)
    pub priority: i32,
    /// Statistics for this route
    stats: Arc<RwLock<FilterStats>>,
}

impl Route {
    /// Create a new route
    pub fn new(
        id: String,
        description: String,
        filter: Arc<dyn FilterRule>,
        destinations: Vec<DestinationId>,
        priority: i32,
    ) -> Self {
        Self {
            id,
            description,
            filter,
            destinations,
            priority,
            stats: Arc::new(RwLock::new(FilterStats::new())),
        }
    }

    /// Evaluate this route against a message
    #[inline]
    pub fn evaluate(&self, msg: &CotMessage) -> FilterResult {
        let result = self.filter.evaluate(msg);
        self.stats.write().record(result);

        trace!(
            route_id = %self.id,
            result = ?result,
            "Route evaluation"
        );

        result
    }

    /// Get statistics for this route
    pub fn stats(&self) -> FilterStats {
        self.stats.read().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = FilterStats::new();
    }
}

/// Routing result for a message
#[derive(Debug, Clone)]
pub struct RoutingResult {
    /// Destinations this message should be routed to
    pub destinations: Vec<DestinationId>,
    /// Routes that matched (for audit trail)
    pub matched_routes: Vec<String>,
}

impl RoutingResult {
    /// Create an empty routing result
    pub fn empty() -> Self {
        Self {
            destinations: Vec::new(),
            matched_routes: Vec::new(),
        }
    }

    /// Check if this message should be routed anywhere
    #[inline]
    pub fn has_destinations(&self) -> bool {
        !self.destinations.is_empty()
    }
}

/// Route evaluation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteStrategy {
    /// Evaluate all routes (multicast)
    All,
    /// Stop at first matching route (unicast)
    FirstMatch,
}

/// High-performance routing table
pub struct RouteTable {
    /// Routes indexed by ID
    routes: DashMap<String, Arc<Route>>,
    /// Ordered list of route IDs (sorted by priority)
    route_order: RwLock<Vec<String>>,
    /// Routing strategy
    strategy: RouteStrategy,
    /// Default destination (if no routes match)
    default_destination: Option<DestinationId>,
}

impl RouteTable {
    /// Create a new route table
    pub fn new(strategy: RouteStrategy) -> Self {
        Self {
            routes: DashMap::new(),
            route_order: RwLock::new(Vec::new()),
            strategy,
            default_destination: None,
        }
    }

    /// Create a route table with multicast (all routes evaluated)
    pub fn multicast() -> Self {
        Self::new(RouteStrategy::All)
    }

    /// Create a route table with unicast (first match only)
    pub fn unicast() -> Self {
        Self::new(RouteStrategy::FirstMatch)
    }

    /// Set the default destination for unmatched messages
    pub fn set_default_destination(&mut self, dest: DestinationId) {
        self.default_destination = Some(dest);
    }

    /// Add a route to the table
    pub fn add_route(&self, route: Route) {
        let route_id = route.id.clone();
        let priority = route.priority;

        self.routes.insert(route_id.clone(), Arc::new(route));

        // Insert into ordered list based on priority
        let mut order = self.route_order.write();
        let insert_pos = order
            .iter()
            .position(|id| {
                self.routes
                    .get(id)
                    .map(|r| r.priority < priority)
                    .unwrap_or(false)
            })
            .unwrap_or(order.len());

        order.insert(insert_pos, route_id);

        debug!(route_count = order.len(), "Route added to table");
    }

    /// Remove a route from the table
    pub fn remove_route(&self, route_id: &str) -> Option<Arc<Route>> {
        let route = self.routes.remove(route_id).map(|(_, r)| r);

        let mut order = self.route_order.write();
        if let Some(pos) = order.iter().position(|id| id == route_id) {
            order.remove(pos);
        }

        route
    }

    /// Get a route by ID
    pub fn get_route(&self, route_id: &str) -> Option<Arc<Route>> {
        self.routes.get(route_id).map(|r| r.clone())
    }

    /// Get all route IDs
    pub fn route_ids(&self) -> Vec<String> {
        self.route_order.read().clone()
    }

    /// Get number of routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Route a message to appropriate destinations
    pub fn route(&self, msg: &CotMessage) -> RoutingResult {
        let mut result = RoutingResult::empty();
        let route_order = self.route_order.read();

        for route_id in route_order.iter() {
            if let Some(route) = self.routes.get(route_id) {
                if route.evaluate(msg).is_pass() {
                    // Add destinations (avoiding duplicates)
                    for dest in &route.destinations {
                        if !result.destinations.contains(dest) {
                            result.destinations.push(dest.clone());
                        }
                    }
                    result.matched_routes.push(route_id.clone());

                    // Short-circuit for FirstMatch strategy
                    if self.strategy == RouteStrategy::FirstMatch {
                        break;
                    }
                }
            }
        }

        // Apply default destination if no matches
        if !result.has_destinations() {
            if let Some(default) = &self.default_destination {
                result.destinations.push(default.clone());
            }
        }

        debug!(
            destinations = ?result.destinations,
            matched_routes = ?result.matched_routes,
            "Message routed"
        );

        result
    }

    /// Get statistics for all routes
    pub fn get_all_stats(&self) -> Vec<(String, FilterStats)> {
        let route_order = self.route_order.read();
        route_order
            .iter()
            .filter_map(|id| self.routes.get(id).map(|route| (id.clone(), route.stats())))
            .collect()
    }

    /// Reset statistics for all routes
    pub fn reset_all_stats(&self) {
        for route in self.routes.iter() {
            route.reset_stats();
        }
    }

    /// Clear all routes
    pub fn clear(&self) {
        self.routes.clear();
        self.route_order.write().clear();
    }
}

/// Builder for constructing route tables
pub struct RouteTableBuilder {
    strategy: RouteStrategy,
    default_destination: Option<DestinationId>,
    routes: Vec<Route>,
}

impl RouteTableBuilder {
    /// Create a new builder with multicast strategy
    pub fn multicast() -> Self {
        Self {
            strategy: RouteStrategy::All,
            default_destination: None,
            routes: Vec::new(),
        }
    }

    /// Create a new builder with unicast strategy
    pub fn unicast() -> Self {
        Self {
            strategy: RouteStrategy::FirstMatch,
            default_destination: None,
            routes: Vec::new(),
        }
    }

    /// Set the default destination
    pub fn default_destination(mut self, dest: DestinationId) -> Self {
        self.default_destination = Some(dest);
        self
    }

    /// Add a route
    pub fn add_route(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }

    /// Build the route table
    pub fn build(self) -> RouteTable {
        let mut table = RouteTable::new(self.strategy);
        table.default_destination = self.default_destination;

        for route in self.routes {
            table.add_route(route);
        }

        table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{AffiliationFilter, GeoBoundingBoxFilter, TeamFilter};

    fn create_test_message() -> CotMessage<'static> {
        CotMessage {
            cot_type: "a-f-G-E-V-C",
            uid: "TEST-001",
            callsign: Some("ALPHA-1"),
            group: Some("Blue Force"),
            team: Some("Alpha"),
            lat: 40.7128,
            lon: -74.0060,
            hae: Some(100.0),
        }
    }

    #[test]
    fn test_route_evaluation() {
        let filter = Arc::new(AffiliationFilter::friendly_only());
        let route = Route::new(
            "route1".to_string(),
            "Friendly units".to_string(),
            filter,
            vec!["dest1".to_string()],
            100,
        );

        let msg = create_test_message();
        let result = route.evaluate(&msg);

        assert_eq!(result, FilterResult::Pass);
    }

    #[test]
    fn test_route_table_multicast() {
        let table = RouteTable::multicast();

        // Add route for friendly units
        let filter1 = Arc::new(AffiliationFilter::friendly_only());
        let route1 = Route::new(
            "route1".to_string(),
            "Friendly units".to_string(),
            filter1,
            vec!["dest1".to_string()],
            100,
        );
        table.add_route(route1);

        // Add route for NYC area
        let filter2 = Arc::new(GeoBoundingBoxFilter::new(40.0, 41.0, -75.0, -73.0));
        let route2 = Route::new(
            "route2".to_string(),
            "NYC area".to_string(),
            filter2,
            vec!["dest2".to_string()],
            90,
        );
        table.add_route(route2);

        let msg = create_test_message();
        let result = table.route(&msg);

        // Both routes should match (multicast)
        assert_eq!(result.destinations.len(), 2);
        assert!(result.destinations.contains(&"dest1".to_string()));
        assert!(result.destinations.contains(&"dest2".to_string()));
        assert_eq!(result.matched_routes.len(), 2);
    }

    #[test]
    fn test_route_table_unicast() {
        let table = RouteTable::unicast();

        // Add high priority route
        let filter1 = Arc::new(AffiliationFilter::friendly_only());
        let route1 = Route::new(
            "route1".to_string(),
            "Friendly units".to_string(),
            filter1,
            vec!["dest1".to_string()],
            100,
        );
        table.add_route(route1);

        // Add lower priority route
        let filter2 = Arc::new(GeoBoundingBoxFilter::new(40.0, 41.0, -75.0, -73.0));
        let route2 = Route::new(
            "route2".to_string(),
            "NYC area".to_string(),
            filter2,
            vec!["dest2".to_string()],
            90,
        );
        table.add_route(route2);

        let msg = create_test_message();
        let result = table.route(&msg);

        // Only first matching route should be used (unicast)
        assert_eq!(result.destinations.len(), 1);
        assert_eq!(result.destinations[0], "dest1");
        assert_eq!(result.matched_routes.len(), 1);
    }

    #[test]
    fn test_default_destination() {
        let mut table = RouteTable::multicast();
        table.set_default_destination("default".to_string());

        // Add route that won't match
        let filter = Arc::new(TeamFilter::new(vec!["Bravo".to_string()]));
        let route = Route::new(
            "route1".to_string(),
            "Team Bravo".to_string(),
            filter,
            vec!["dest1".to_string()],
            100,
        );
        table.add_route(route);

        let msg = create_test_message();
        let result = table.route(&msg);

        // Should use default destination
        assert_eq!(result.destinations.len(), 1);
        assert_eq!(result.destinations[0], "default");
        assert_eq!(result.matched_routes.len(), 0);
    }

    #[test]
    fn test_route_priority_ordering() {
        let table = RouteTable::unicast();

        // Add routes in reverse priority order
        let filter1 = Arc::new(AffiliationFilter::friendly_only());
        let route1 = Route::new(
            "low".to_string(),
            "Low priority".to_string(),
            filter1,
            vec!["dest1".to_string()],
            10,
        );
        table.add_route(route1);

        let filter2 = Arc::new(AffiliationFilter::friendly_only());
        let route2 = Route::new(
            "high".to_string(),
            "High priority".to_string(),
            filter2,
            vec!["dest2".to_string()],
            100,
        );
        table.add_route(route2);

        // Verify ordering
        let order = table.route_ids();
        assert_eq!(order[0], "high");
        assert_eq!(order[1], "low");
    }

    #[test]
    fn test_route_statistics() {
        let filter = Arc::new(AffiliationFilter::friendly_only());
        let route = Route::new(
            "route1".to_string(),
            "Friendly units".to_string(),
            filter,
            vec!["dest1".to_string()],
            100,
        );

        let msg = create_test_message();

        // Evaluate multiple times
        route.evaluate(&msg);
        route.evaluate(&msg);
        route.evaluate(&msg);

        let stats = route.stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.passes, 3);
        assert_eq!(stats.blocks, 0);
    }

    #[test]
    fn test_builder() {
        let filter = Arc::new(AffiliationFilter::friendly_only());
        let route = Route::new(
            "route1".to_string(),
            "Friendly units".to_string(),
            filter,
            vec!["dest1".to_string()],
            100,
        );

        let table = RouteTableBuilder::multicast()
            .default_destination("default".to_string())
            .add_route(route)
            .build();

        assert_eq!(table.route_count(), 1);
        assert_eq!(table.default_destination, Some("default".to_string()));
    }
}
