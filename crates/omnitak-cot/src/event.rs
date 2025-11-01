//! CoT Event structures and affiliation parsing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// CoT Event represents a Cursor on Target message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// CoT version (typically "2.0")
    pub version: String,
    /// Unique identifier for this event
    pub uid: String,
    /// CoT type (e.g., "a-f-G" for atom-friendly-ground)
    #[serde(rename = "type")]
    pub event_type: String,
    /// Event timestamp
    pub time: DateTime<Utc>,
    /// Event start time
    pub start: DateTime<Utc>,
    /// Event stale time (when the event becomes invalid)
    pub stale: DateTime<Utc>,
    /// How the event was generated (e.g., "h-e" for human-entered)
    pub how: String,
    /// Geographic location and accuracy
    pub point: Point,
    /// Optional detail section with arbitrary XML data
    pub detail: Option<HashMap<String, String>>,
}

/// Geographic point with accuracy metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// Latitude in decimal degrees (-90 to 90)
    pub lat: f64,
    /// Longitude in decimal degrees (-180 to 180)
    pub lon: f64,
    /// Height above ellipsoid in meters
    pub hae: f64,
    /// Circular error in meters (95% confidence)
    pub ce: f64,
    /// Linear error in meters (95% confidence)
    pub le: f64,
}

/// MIL-STD-2525 affiliation parsed from CoT type field
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Affiliation {
    /// Pending (p)
    Pending,
    /// Unknown (u)
    Unknown,
    /// Assumed Friend (a)
    AssumedFriend,
    /// Friend (f)
    Friend,
    /// Neutral (n)
    Neutral,
    /// Suspect (s)
    Suspect,
    /// Hostile (h)
    Hostile,
    /// Joker (j)
    Joker,
    /// Faker (k)
    Faker,
    /// None specified (o)
    None,
}

impl Affiliation {
    /// Parse affiliation from CoT type field
    /// CoT type format: "a-f-G" where second character is affiliation
    pub fn from_cot_type(cot_type: &str) -> Option<Self> {
        let parts: Vec<&str> = cot_type.split('-').collect();
        if parts.len() < 2 {
            return None;
        }

        match parts[1].chars().next() {
            Some('p') => Some(Affiliation::Pending),
            Some('u') => Some(Affiliation::Unknown),
            Some('a') => Some(Affiliation::AssumedFriend),
            Some('f') => Some(Affiliation::Friend),
            Some('n') => Some(Affiliation::Neutral),
            Some('s') => Some(Affiliation::Suspect),
            Some('h') => Some(Affiliation::Hostile),
            Some('j') => Some(Affiliation::Joker),
            Some('k') => Some(Affiliation::Faker),
            Some('o') => Some(Affiliation::None),
            _ => None,
        }
    }
}

impl fmt::Display for Affiliation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Affiliation::Pending => write!(f, "Pending"),
            Affiliation::Unknown => write!(f, "Unknown"),
            Affiliation::AssumedFriend => write!(f, "Assumed Friend"),
            Affiliation::Friend => write!(f, "Friend"),
            Affiliation::Neutral => write!(f, "Neutral"),
            Affiliation::Suspect => write!(f, "Suspect"),
            Affiliation::Hostile => write!(f, "Hostile"),
            Affiliation::Joker => write!(f, "Joker"),
            Affiliation::Faker => write!(f, "Faker"),
            Affiliation::None => write!(f, "None"),
        }
    }
}

impl Event {
    /// Get the affiliation from the event type
    pub fn affiliation(&self) -> Option<Affiliation> {
        Affiliation::from_cot_type(&self.event_type)
    }
}

impl Point {
    /// Create a new Point with default accuracy values
    pub fn new(lat: f64, lon: f64, hae: f64) -> Self {
        Self {
            lat,
            lon,
            hae,
            ce: 9999999.0,
            le: 9999999.0,
        }
    }

    /// Create a new Point with specified accuracy
    pub fn with_accuracy(lat: f64, lon: f64, hae: f64, ce: f64, le: f64) -> Self {
        Self {
            lat,
            lon,
            hae,
            ce,
            le,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_affiliation_parsing() {
        assert_eq!(
            Affiliation::from_cot_type("a-f-G"),
            Some(Affiliation::Friend)
        );
        assert_eq!(
            Affiliation::from_cot_type("a-h-G"),
            Some(Affiliation::Hostile)
        );
        assert_eq!(
            Affiliation::from_cot_type("a-n-G"),
            Some(Affiliation::Neutral)
        );
        assert_eq!(
            Affiliation::from_cot_type("a-u-G"),
            Some(Affiliation::Unknown)
        );
        assert_eq!(Affiliation::from_cot_type("invalid"), None);
    }

    #[test]
    fn test_point_creation() {
        let point = Point::new(37.7749, -122.4194, 100.0);
        assert_eq!(point.lat, 37.7749);
        assert_eq!(point.lon, -122.4194);
        assert_eq!(point.hae, 100.0);
        assert_eq!(point.ce, 9999999.0);
        assert_eq!(point.le, 9999999.0);
    }
}
