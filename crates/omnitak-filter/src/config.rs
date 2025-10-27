//! Configuration for filter rules
//!
//! Supports loading filter rules from YAML configuration files
//! with hot-reload capability.

// Affiliation and Dimension types available for future use
use crate::router::{Route, RouteStrategy, RouteTable, RouteTableBuilder};
use crate::rules::{
    AffiliationFilter, DimensionFilter, GeoBoundingBoxFilter, GroupFilter, TeamFilter, UidFilter,
};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

/// Filter configuration that can be loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FilterConfig {
    /// Filter by affiliation
    Affiliation {
        /// Allowed affiliations (friendly, hostile, neutral, etc.)
        allow: Vec<String>,
    },
    /// Filter by dimension
    Dimension {
        /// Allowed dimensions (air, ground, sea, etc.)
        allow: Vec<String>,
    },
    /// Filter by group name
    Group {
        /// Regex pattern for group matching
        pattern: String,
    },
    /// Filter by team name
    Team {
        /// Allowed team names
        teams: Vec<String>,
    },
    /// Filter by geographic bounding box
    GeoBoundingBox {
        /// Minimum latitude
        min_lat: f64,
        /// Maximum latitude
        max_lat: f64,
        /// Minimum longitude
        min_lon: f64,
        /// Maximum longitude
        max_lon: f64,
    },
    /// Filter by specific UIDs
    Uid {
        /// Allowed UIDs
        uids: Vec<String>,
    },
    /// Composite AND filter
    And {
        /// Filters to combine with AND logic
        filters: Vec<FilterConfig>,
    },
    /// Composite OR filter
    Or {
        /// Filters to combine with OR logic
        filters: Vec<FilterConfig>,
    },
    /// Composite NOT filter
    Not {
        /// Filter to invert
        filter: Box<FilterConfig>,
    },
}

impl FilterConfig {
    /// Convert configuration to a filter rule
    pub fn into_filter_rule(self) -> Result<Arc<dyn crate::rules::FilterRule>> {
        match self {
            FilterConfig::Affiliation { allow } => {
                let filter = AffiliationFilter {
                    allow: allow.into_iter().collect(),
                };
                Ok(Arc::new(filter))
            }
            FilterConfig::Dimension { allow } => {
                let filter = DimensionFilter {
                    dimensions: allow.into_iter().collect(),
                };
                Ok(Arc::new(filter))
            }
            FilterConfig::Group { pattern } => {
                let filter = GroupFilter::new(&pattern)
                    .context("Failed to create group filter with regex pattern")?;
                Ok(Arc::new(filter))
            }
            FilterConfig::Team { teams } => {
                let filter = TeamFilter {
                    teams: teams.into_iter().collect(),
                };
                Ok(Arc::new(filter))
            }
            FilterConfig::GeoBoundingBox {
                min_lat,
                max_lat,
                min_lon,
                max_lon,
            } => {
                let filter = GeoBoundingBoxFilter::new(min_lat, max_lat, min_lon, max_lon);
                Ok(Arc::new(filter))
            }
            FilterConfig::Uid { uids } => {
                let filter = UidFilter {
                    uids: uids.into_iter().collect(),
                };
                Ok(Arc::new(filter))
            }
            FilterConfig::And { filters: _ } => {
                // Note: CompositeFilter doesn't implement Clone, so we can't use it directly
                // For now, we'll just return an error for composite filters in config
                Err(anyhow!("Composite filters not yet supported in config"))
            }
            FilterConfig::Or { filters: _ } => {
                Err(anyhow!("Composite filters not yet supported in config"))
            }
            FilterConfig::Not { filter: _ } => {
                Err(anyhow!("Composite filters not yet supported in config"))
            }
        }
    }

    /// Validate the filter configuration
    pub fn validate(&self) -> Result<()> {
        match self {
            FilterConfig::Affiliation { allow } => {
                if allow.is_empty() {
                    return Err(anyhow!("Affiliation filter must have at least one allowed value"));
                }
                // Validate affiliation strings
                for aff in allow {
                    if !matches!(
                        aff.to_lowercase().as_str(),
                        "pending" | "unknown" | "assumedfriend" | "friend" | "neutral"
                            | "suspect" | "hostile" | "joker" | "faker"
                    ) {
                        warn!("Unknown affiliation value: {}", aff);
                    }
                }
                Ok(())
            }
            FilterConfig::Dimension { allow } => {
                if allow.is_empty() {
                    return Err(anyhow!("Dimension filter must have at least one allowed value"));
                }
                Ok(())
            }
            FilterConfig::Group { pattern } => {
                // Validate regex pattern
                regex::Regex::new(pattern)
                    .context("Invalid regex pattern for group filter")?;
                Ok(())
            }
            FilterConfig::Team { teams } => {
                if teams.is_empty() {
                    return Err(anyhow!("Team filter must have at least one team"));
                }
                Ok(())
            }
            FilterConfig::GeoBoundingBox {
                min_lat,
                max_lat,
                min_lon,
                max_lon,
            } => {
                if min_lat >= max_lat {
                    return Err(anyhow!("min_lat must be less than max_lat"));
                }
                if min_lon >= max_lon {
                    return Err(anyhow!("min_lon must be less than max_lon"));
                }
                if !(-90.0..=90.0).contains(min_lat) || !(-90.0..=90.0).contains(max_lat) {
                    return Err(anyhow!("Latitude must be between -90 and 90"));
                }
                if !(-180.0..=180.0).contains(min_lon) || !(-180.0..=180.0).contains(max_lon) {
                    return Err(anyhow!("Longitude must be between -180 and 180"));
                }
                Ok(())
            }
            FilterConfig::Uid { uids } => {
                if uids.is_empty() {
                    return Err(anyhow!("UID filter must have at least one UID"));
                }
                Ok(())
            }
            FilterConfig::And { filters } => {
                if filters.is_empty() {
                    return Err(anyhow!("AND filter must have at least one child filter"));
                }
                for filter in filters {
                    filter.validate()?;
                }
                Ok(())
            }
            FilterConfig::Or { filters } => {
                if filters.is_empty() {
                    return Err(anyhow!("OR filter must have at least one child filter"));
                }
                for filter in filters {
                    filter.validate()?;
                }
                Ok(())
            }
            FilterConfig::Not { filter } => filter.validate(),
        }
    }
}

/// Route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Unique route identifier
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// Filter configuration
    pub filter: FilterConfig,
    /// Destination IDs for matching messages
    pub destinations: Vec<String>,
    /// Priority (higher values evaluated first)
    #[serde(default = "default_priority")]
    pub priority: i32,
    /// Whether this route is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_priority() -> i32 {
    0
}

fn default_enabled() -> bool {
    true
}

impl RouteConfig {
    /// Convert to a Route
    pub fn into_route(self) -> Result<Route> {
        self.filter.validate()?;
        let filter = self.filter.into_filter_rule()?;

        Ok(Route::new(
            self.id,
            self.description,
            filter,
            self.destinations,
            self.priority,
        ))
    }

    /// Validate the route configuration
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(anyhow!("Route ID cannot be empty"));
        }
        if self.destinations.is_empty() {
            return Err(anyhow!("Route must have at least one destination"));
        }
        self.filter.validate()?;
        Ok(())
    }
}

/// Complete routing table configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Routing strategy (all or first_match)
    #[serde(default = "default_strategy")]
    pub strategy: String,
    /// Default destination for unmatched messages
    pub default_destination: Option<String>,
    /// List of routes
    pub routes: Vec<RouteConfig>,
}

fn default_strategy() -> String {
    "all".to_string()
}

impl RoutingConfig {
    /// Load routing configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: RoutingConfig = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        config.validate()?;

        info!(
            path = %path.display(),
            route_count = config.routes.len(),
            strategy = %config.strategy,
            "Loaded routing configuration"
        );

        Ok(config)
    }

    /// Convert to a RouteTable
    pub fn into_route_table(self) -> Result<RouteTable> {
        let strategy = match self.strategy.to_lowercase().as_str() {
            "all" | "multicast" => RouteStrategy::All,
            "first_match" | "first" | "unicast" => RouteStrategy::FirstMatch,
            _ => {
                return Err(anyhow!(
                    "Invalid strategy: {}. Must be 'all' or 'first_match'",
                    self.strategy
                ))
            }
        };

        let mut builder = match strategy {
            RouteStrategy::All => RouteTableBuilder::multicast(),
            RouteStrategy::FirstMatch => RouteTableBuilder::unicast(),
        };

        if let Some(default) = self.default_destination {
            builder = builder.default_destination(default);
        }

        for route_config in self.routes {
            if !route_config.enabled {
                info!(route_id = %route_config.id, "Skipping disabled route");
                continue;
            }

            let route = route_config
                .into_route()
                .with_context(|| "Failed to create route")?;
            builder = builder.add_route(route);
        }

        Ok(builder.build())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate strategy
        match self.strategy.to_lowercase().as_str() {
            "all" | "multicast" | "first_match" | "first" | "unicast" => {}
            _ => {
                return Err(anyhow!(
                    "Invalid strategy: {}. Must be 'all' or 'first_match'",
                    self.strategy
                ))
            }
        }

        // Check for duplicate route IDs
        let mut seen_ids = HashSet::new();
        for route in &self.routes {
            if !seen_ids.insert(&route.id) {
                return Err(anyhow!("Duplicate route ID: {}", route.id));
            }
            route.validate()?;
        }

        Ok(())
    }

    /// Save configuration to a YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let yaml = serde_yaml::to_string(self).context("Failed to serialize config")?;

        fs::write(path, yaml)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        info!(path = %path.display(), "Saved routing configuration");

        Ok(())
    }
}

/// Example configurations
impl RoutingConfig {
    /// Create an example configuration
    pub fn example() -> Self {
        RoutingConfig {
            strategy: "all".to_string(),
            default_destination: Some("default-server".to_string()),
            routes: vec![
                RouteConfig {
                    id: "friendly-ground".to_string(),
                    description: "Route friendly ground forces to blue team server".to_string(),
                    filter: FilterConfig::And {
                        filters: vec![
                            FilterConfig::Affiliation {
                                allow: vec!["friend".to_string(), "assumedfriend".to_string()],
                            },
                            FilterConfig::Dimension {
                                allow: vec!["ground".to_string()],
                            },
                        ],
                    },
                    destinations: vec!["blue-ground-server".to_string()],
                    priority: 100,
                    enabled: true,
                },
                RouteConfig {
                    id: "hostile-air".to_string(),
                    description: "Route hostile air contacts to air defense".to_string(),
                    filter: FilterConfig::And {
                        filters: vec![
                            FilterConfig::Affiliation {
                                allow: vec!["hostile".to_string(), "suspect".to_string()],
                            },
                            FilterConfig::Dimension {
                                allow: vec!["air".to_string()],
                            },
                        ],
                    },
                    destinations: vec!["air-defense-server".to_string()],
                    priority: 90,
                    enabled: true,
                },
                RouteConfig {
                    id: "team-alpha".to_string(),
                    description: "Route Team Alpha to their dedicated server".to_string(),
                    filter: FilterConfig::Team {
                        teams: vec!["Alpha".to_string()],
                    },
                    destinations: vec!["team-alpha-server".to_string()],
                    priority: 80,
                    enabled: true,
                },
                RouteConfig {
                    id: "aor-northeast".to_string(),
                    description: "Route northeast AOR to regional server".to_string(),
                    filter: FilterConfig::GeoBoundingBox {
                        min_lat: 40.0,
                        max_lat: 45.0,
                        min_lon: -75.0,
                        max_lon: -70.0,
                    },
                    destinations: vec!["northeast-server".to_string()],
                    priority: 50,
                    enabled: true,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_config_validation() {
        let config = FilterConfig::Affiliation {
            allow: vec!["friend".to_string()],
        };
        assert!(config.validate().is_ok());

        let config = FilterConfig::Affiliation { allow: vec![] };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_geo_bounding_box_validation() {
        let config = FilterConfig::GeoBoundingBox {
            min_lat: 40.0,
            max_lat: 45.0,
            min_lon: -75.0,
            max_lon: -70.0,
        };
        assert!(config.validate().is_ok());

        // Invalid: min_lat >= max_lat
        let config = FilterConfig::GeoBoundingBox {
            min_lat: 45.0,
            max_lat: 40.0,
            min_lon: -75.0,
            max_lon: -70.0,
        };
        assert!(config.validate().is_err());

        // Invalid: latitude out of range
        let config = FilterConfig::GeoBoundingBox {
            min_lat: -100.0,
            max_lat: 45.0,
            min_lon: -75.0,
            max_lon: -70.0,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_route_config_validation() {
        let config = RouteConfig {
            id: "test".to_string(),
            description: "Test route".to_string(),
            filter: FilterConfig::Affiliation {
                allow: vec!["friend".to_string()],
            },
            destinations: vec!["dest1".to_string()],
            priority: 100,
            enabled: true,
        };
        assert!(config.validate().is_ok());

        // Invalid: no destinations
        let config = RouteConfig {
            id: "test".to_string(),
            description: "Test route".to_string(),
            filter: FilterConfig::Affiliation {
                allow: vec!["friend".to_string()],
            },
            destinations: vec![],
            priority: 100,
            enabled: true,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_routing_config_duplicate_ids() {
        let config = RoutingConfig {
            strategy: "all".to_string(),
            default_destination: None,
            routes: vec![
                RouteConfig {
                    id: "test".to_string(),
                    description: "Test 1".to_string(),
                    filter: FilterConfig::Affiliation {
                        allow: vec!["friend".to_string()],
                    },
                    destinations: vec!["dest1".to_string()],
                    priority: 100,
                    enabled: true,
                },
                RouteConfig {
                    id: "test".to_string(),
                    description: "Test 2".to_string(),
                    filter: FilterConfig::Affiliation {
                        allow: vec!["hostile".to_string()],
                    },
                    destinations: vec!["dest2".to_string()],
                    priority: 90,
                    enabled: true,
                },
            ],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_example_config() {
        let config = RoutingConfig::example();
        assert!(config.validate().is_ok());
        assert_eq!(config.routes.len(), 4);
    }
}
