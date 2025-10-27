//! Filter rules for CoT messages
//!
//! Provides various filter implementations that can be composed together.

use crate::affiliation::{Affiliation, CotType, Dimension};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;

/// Result of a filter evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterResult {
    /// Message passes the filter
    Pass,
    /// Message is blocked by the filter
    Block,
}

impl FilterResult {
    /// Check if the result is Pass
    #[inline]
    pub fn is_pass(&self) -> bool {
        matches!(self, FilterResult::Pass)
    }

    /// Check if the result is Block
    #[inline]
    pub fn is_block(&self) -> bool {
        matches!(self, FilterResult::Block)
    }
}

/// Trait for filter rules
pub trait FilterRule: Send + Sync {
    /// Evaluate the filter against a CoT message
    fn evaluate(&self, msg: &CotMessage) -> FilterResult;

    /// Get a human-readable description of this filter
    fn describe(&self) -> String;
}

/// Simplified CoT message structure for filtering
#[derive(Debug, Clone)]
pub struct CotMessage<'a> {
    /// CoT type (e.g., "a-f-G-E-V-C")
    pub cot_type: &'a str,
    /// Unique identifier
    pub uid: &'a str,
    /// Callsign/name
    pub callsign: Option<&'a str>,
    /// Group name
    pub group: Option<&'a str>,
    /// Team name
    pub team: Option<&'a str>,
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
    /// Altitude (HAE - height above ellipsoid)
    pub hae: Option<f64>,
}

/// Filter by affiliation (friendly, hostile, neutral, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffiliationFilter {
    /// Affiliations to allow
    pub allow: HashSet<String>,
}

impl AffiliationFilter {
    /// Create a new affiliation filter
    pub fn new(affiliations: Vec<Affiliation>) -> Self {
        let allow = affiliations
            .iter()
            .map(|a| format!("{:?}", a).to_lowercase())
            .collect();
        Self { allow }
    }

    /// Create a filter that allows only friendly units
    pub fn friendly_only() -> Self {
        Self::new(vec![
            Affiliation::Friend,
            Affiliation::AssumedFriend,
            Affiliation::Joker,
        ])
    }

    /// Create a filter that allows only hostile units
    pub fn hostile_only() -> Self {
        Self::new(vec![
            Affiliation::Hostile,
            Affiliation::Suspect,
            Affiliation::Faker,
        ])
    }
}

impl FilterRule for AffiliationFilter {
    #[inline]
    fn evaluate(&self, msg: &CotMessage) -> FilterResult {
        let cot = CotType::parse(msg.cot_type);

        if let Some(affiliation) = cot.affiliation {
            let aff_str = format!("{:?}", affiliation).to_lowercase();
            if self.allow.contains(&aff_str) {
                return FilterResult::Pass;
            }
        }

        FilterResult::Block
    }

    fn describe(&self) -> String {
        format!("AffiliationFilter(allow: {:?})", self.allow)
    }
}

/// Filter by group name (supports regex)
#[derive(Debug, Clone)]
pub struct GroupFilter {
    /// Regex pattern for group matching
    pattern: Regex,
    /// Original pattern string for description
    pattern_str: String,
}

impl GroupFilter {
    /// Create a new group filter with a regex pattern
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
            pattern_str: pattern.to_string(),
        })
    }

    /// Create a filter for exact group match
    pub fn exact(group: &str) -> Self {
        let pattern = format!("^{}$", regex::escape(group));
        Self {
            pattern: Regex::new(&pattern).unwrap(),
            pattern_str: pattern,
        }
    }
}

impl FilterRule for GroupFilter {
    #[inline]
    fn evaluate(&self, msg: &CotMessage) -> FilterResult {
        if let Some(group) = msg.group {
            if self.pattern.is_match(group) {
                return FilterResult::Pass;
            }
        }
        FilterResult::Block
    }

    fn describe(&self) -> String {
        format!("GroupFilter(pattern: {})", self.pattern_str)
    }
}

/// Filter by team name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamFilter {
    /// Allowed team names
    pub teams: HashSet<String>,
}

impl TeamFilter {
    /// Create a new team filter
    pub fn new(teams: Vec<String>) -> Self {
        Self {
            teams: teams.into_iter().collect(),
        }
    }
}

impl FilterRule for TeamFilter {
    #[inline]
    fn evaluate(&self, msg: &CotMessage) -> FilterResult {
        if let Some(team) = msg.team {
            if self.teams.contains(team) {
                return FilterResult::Pass;
            }
        }
        FilterResult::Block
    }

    fn describe(&self) -> String {
        format!("TeamFilter(teams: {:?})", self.teams)
    }
}

/// Filter by geographic bounding box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoBoundingBoxFilter {
    /// Minimum latitude (south)
    pub min_lat: f64,
    /// Maximum latitude (north)
    pub max_lat: f64,
    /// Minimum longitude (west)
    pub min_lon: f64,
    /// Maximum longitude (east)
    pub max_lon: f64,
}

impl GeoBoundingBoxFilter {
    /// Create a new geo bounding box filter
    pub fn new(min_lat: f64, max_lat: f64, min_lon: f64, max_lon: f64) -> Self {
        Self {
            min_lat,
            max_lat,
            min_lon,
            max_lon,
        }
    }

    /// Check if coordinates are within the bounding box
    #[inline]
    pub fn contains(&self, lat: f64, lon: f64) -> bool {
        lat >= self.min_lat && lat <= self.max_lat && lon >= self.min_lon && lon <= self.max_lon
    }
}

impl FilterRule for GeoBoundingBoxFilter {
    #[inline]
    fn evaluate(&self, msg: &CotMessage) -> FilterResult {
        if self.contains(msg.lat, msg.lon) {
            FilterResult::Pass
        } else {
            FilterResult::Block
        }
    }

    fn describe(&self) -> String {
        format!(
            "GeoBoundingBoxFilter(lat: {:.4} to {:.4}, lon: {:.4} to {:.4})",
            self.min_lat, self.max_lat, self.min_lon, self.max_lon
        )
    }
}

/// Filter by specific UIDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UidFilter {
    /// Allowed UIDs
    pub uids: HashSet<String>,
}

impl UidFilter {
    /// Create a new UID filter
    pub fn new(uids: Vec<String>) -> Self {
        Self {
            uids: uids.into_iter().collect(),
        }
    }
}

impl FilterRule for UidFilter {
    #[inline]
    fn evaluate(&self, msg: &CotMessage) -> FilterResult {
        if self.uids.contains(msg.uid) {
            FilterResult::Pass
        } else {
            FilterResult::Block
        }
    }

    fn describe(&self) -> String {
        format!("UidFilter(count: {})", self.uids.len())
    }
}

/// Filter by dimension (air, ground, sea, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionFilter {
    /// Allowed dimensions
    pub dimensions: HashSet<String>,
}

impl DimensionFilter {
    /// Create a new dimension filter
    pub fn new(dimensions: Vec<Dimension>) -> Self {
        let dimensions = dimensions
            .iter()
            .map(|d| format!("{:?}", d).to_lowercase())
            .collect();
        Self { dimensions }
    }

    /// Create a filter for air units only
    pub fn air_only() -> Self {
        Self::new(vec![Dimension::Air])
    }

    /// Create a filter for ground units only
    pub fn ground_only() -> Self {
        Self::new(vec![Dimension::Ground])
    }
}

impl FilterRule for DimensionFilter {
    #[inline]
    fn evaluate(&self, msg: &CotMessage) -> FilterResult {
        let cot = CotType::parse(msg.cot_type);

        if let Some(dimension) = cot.dimension {
            let dim_str = format!("{:?}", dimension).to_lowercase();
            if self.dimensions.contains(&dim_str) {
                return FilterResult::Pass;
            }
        }

        FilterResult::Block
    }

    fn describe(&self) -> String {
        format!("DimensionFilter(dimensions: {:?})", self.dimensions)
    }
}

/// Composite filter that combines multiple filters with AND/OR logic
///
/// Note: Cannot derive Clone because it contains trait objects
pub enum CompositeFilter {
    /// All filters must pass (AND)
    And(Vec<Arc<dyn FilterRule>>),
    /// At least one filter must pass (OR)
    Or(Vec<Arc<dyn FilterRule>>),
    /// Invert the result of a filter (NOT)
    Not(Arc<dyn FilterRule>),
}

impl FilterRule for CompositeFilter {
    fn evaluate(&self, msg: &CotMessage) -> FilterResult {
        match self {
            CompositeFilter::And(filters) => {
                for filter in filters {
                    if filter.evaluate(msg).is_block() {
                        return FilterResult::Block;
                    }
                }
                FilterResult::Pass
            }
            CompositeFilter::Or(filters) => {
                for filter in filters {
                    if filter.evaluate(msg).is_pass() {
                        return FilterResult::Pass;
                    }
                }
                FilterResult::Block
            }
            CompositeFilter::Not(filter) => match filter.evaluate(msg) {
                FilterResult::Pass => FilterResult::Block,
                FilterResult::Block => FilterResult::Pass,
            },
        }
    }

    fn describe(&self) -> String {
        match self {
            CompositeFilter::And(filters) => {
                let descriptions: Vec<_> = filters.iter().map(|f| f.describe()).collect();
                format!("AND({})", descriptions.join(", "))
            }
            CompositeFilter::Or(filters) => {
                let descriptions: Vec<_> = filters.iter().map(|f| f.describe()).collect();
                format!("OR({})", descriptions.join(", "))
            }
            CompositeFilter::Not(filter) => format!("NOT({})", filter.describe()),
        }
    }
}

/// Statistics for filter evaluation
#[derive(Debug, Clone, Default)]
pub struct FilterStats {
    /// Total evaluations
    pub total: u64,
    /// Number of passes
    pub passes: u64,
    /// Number of blocks
    pub blocks: u64,
}

impl FilterStats {
    /// Create new filter stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a filter result
    pub fn record(&mut self, result: FilterResult) {
        self.total += 1;
        match result {
            FilterResult::Pass => self.passes += 1,
            FilterResult::Block => self.blocks += 1,
        }
    }

    /// Get pass rate as a percentage
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.passes as f64 / self.total as f64) * 100.0
        }
    }
}

impl fmt::Display for FilterStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FilterStats(total: {}, passes: {}, blocks: {}, pass_rate: {:.2}%)",
            self.total,
            self.passes,
            self.blocks,
            self.pass_rate()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_affiliation_filter() {
        let filter = AffiliationFilter::friendly_only();
        let msg = create_test_message();

        let result = filter.evaluate(&msg);
        assert_eq!(result, FilterResult::Pass);
    }

    #[test]
    fn test_affiliation_filter_blocks_hostile() {
        let filter = AffiliationFilter::friendly_only();
        let mut msg = create_test_message();
        msg.cot_type = "a-h-G-E-V-C"; // hostile

        let result = filter.evaluate(&msg);
        assert_eq!(result, FilterResult::Block);
    }

    #[test]
    fn test_group_filter_exact() {
        let filter = GroupFilter::exact("Blue Force");
        let msg = create_test_message();

        let result = filter.evaluate(&msg);
        assert_eq!(result, FilterResult::Pass);
    }

    #[test]
    fn test_group_filter_regex() {
        let filter = GroupFilter::new("Blue.*").unwrap();
        let msg = create_test_message();

        let result = filter.evaluate(&msg);
        assert_eq!(result, FilterResult::Pass);
    }

    #[test]
    fn test_team_filter() {
        let filter = TeamFilter::new(vec!["Alpha".to_string(), "Bravo".to_string()]);
        let msg = create_test_message();

        let result = filter.evaluate(&msg);
        assert_eq!(result, FilterResult::Pass);
    }

    #[test]
    fn test_geo_bounding_box() {
        let filter = GeoBoundingBoxFilter::new(40.0, 41.0, -75.0, -73.0);
        let msg = create_test_message();

        let result = filter.evaluate(&msg);
        assert_eq!(result, FilterResult::Pass);
    }

    #[test]
    fn test_geo_bounding_box_outside() {
        let filter = GeoBoundingBoxFilter::new(30.0, 35.0, -120.0, -115.0);
        let msg = create_test_message();

        let result = filter.evaluate(&msg);
        assert_eq!(result, FilterResult::Block);
    }

    #[test]
    fn test_uid_filter() {
        let filter = UidFilter::new(vec!["TEST-001".to_string()]);
        let msg = create_test_message();

        let result = filter.evaluate(&msg);
        assert_eq!(result, FilterResult::Pass);
    }

    #[test]
    fn test_dimension_filter() {
        let filter = DimensionFilter::ground_only();
        let msg = create_test_message();

        let result = filter.evaluate(&msg);
        assert_eq!(result, FilterResult::Pass);
    }

    #[test]
    fn test_filter_stats() {
        let mut stats = FilterStats::new();

        stats.record(FilterResult::Pass);
        stats.record(FilterResult::Pass);
        stats.record(FilterResult::Block);

        assert_eq!(stats.total, 3);
        assert_eq!(stats.passes, 2);
        assert_eq!(stats.blocks, 1);
        assert!((stats.pass_rate() - 66.67).abs() < 0.01);
    }
}
