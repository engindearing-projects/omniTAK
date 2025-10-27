//! Validation for CoT events

use crate::event::{Event, Point};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid latitude: {0} (must be between -90 and 90)")]
    InvalidLatitude(f64),

    #[error("Invalid longitude: {0} (must be between -180 and 180)")]
    InvalidLongitude(f64),

    #[error("Invalid circular error: {0} (must be positive)")]
    InvalidCircularError(f64),

    #[error("Invalid linear error: {0} (must be positive)")]
    InvalidLinearError(f64),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid timestamp order: stale ({0}) must be after start ({1})")]
    InvalidTimestampOrder(String, String),

    #[error("Invalid CoT type format: {0}")]
    InvalidCotType(String),

    #[error("Empty UID")]
    EmptyUid,

    #[error("Empty version")]
    EmptyVersion,
}

/// Validates a CoT Event
pub fn validate_event(event: &Event) -> Result<(), ValidationError> {
    // Validate version
    if event.version.is_empty() {
        return Err(ValidationError::EmptyVersion);
    }

    // Validate UID
    if event.uid.is_empty() {
        return Err(ValidationError::EmptyUid);
    }

    // Validate CoT type format (should be dash-separated, e.g., "a-f-G")
    if !event.event_type.contains('-') {
        return Err(ValidationError::InvalidCotType(event.event_type.clone()));
    }

    // Validate timestamps
    if event.stale <= event.start {
        return Err(ValidationError::InvalidTimestampOrder(
            event.stale.to_rfc3339(),
            event.start.to_rfc3339(),
        ));
    }

    // Validate point
    validate_point(&event.point)?;

    Ok(())
}

/// Validates a Point
pub fn validate_point(point: &Point) -> Result<(), ValidationError> {
    // Validate latitude range
    if point.lat < -90.0 || point.lat > 90.0 {
        return Err(ValidationError::InvalidLatitude(point.lat));
    }

    // Validate longitude range
    if point.lon < -180.0 || point.lon > 180.0 {
        return Err(ValidationError::InvalidLongitude(point.lon));
    }

    // Validate circular error (must be positive)
    if point.ce < 0.0 {
        return Err(ValidationError::InvalidCircularError(point.ce));
    }

    // Validate linear error (must be positive)
    if point.le < 0.0 {
        return Err(ValidationError::InvalidLinearError(point.le));
    }

    Ok(())
}

/// Strict validation that enforces additional constraints
pub fn validate_event_strict(event: &Event) -> Result<(), ValidationError> {
    // Run standard validation first
    validate_event(event)?;

    // Additional strict checks could go here
    // For example: checking if version is exactly "2.0"
    // or validating specific CoT type patterns

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, Point};
    use chrono::{TimeZone, Utc};

    fn create_valid_event() -> Event {
        Event {
            version: "2.0".to_string(),
            uid: "test-123".to_string(),
            event_type: "a-f-G".to_string(),
            time: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            start: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            stale: Utc.with_ymd_and_hms(2024, 1, 15, 10, 35, 0).unwrap(),
            how: "h-e".to_string(),
            point: Point {
                lat: 37.7749,
                lon: -122.4194,
                hae: 100.0,
                ce: 10.0,
                le: 5.0,
            },
            detail: None,
        }
    }

    #[test]
    fn test_valid_event() {
        let event = create_valid_event();
        assert!(validate_event(&event).is_ok());
    }

    #[test]
    fn test_invalid_latitude() {
        let mut event = create_valid_event();
        event.point.lat = 91.0;
        assert!(matches!(
            validate_event(&event),
            Err(ValidationError::InvalidLatitude(_))
        ));
    }

    #[test]
    fn test_invalid_longitude() {
        let mut event = create_valid_event();
        event.point.lon = -181.0;
        assert!(matches!(
            validate_event(&event),
            Err(ValidationError::InvalidLongitude(_))
        ));
    }

    #[test]
    fn test_invalid_timestamp_order() {
        let mut event = create_valid_event();
        event.stale = event.start;
        assert!(matches!(
            validate_event(&event),
            Err(ValidationError::InvalidTimestampOrder(_, _))
        ));
    }

    #[test]
    fn test_empty_uid() {
        let mut event = create_valid_event();
        event.uid = String::new();
        assert!(matches!(
            validate_event(&event),
            Err(ValidationError::EmptyUid)
        ));
    }

    #[test]
    fn test_invalid_cot_type() {
        let mut event = create_valid_event();
        event.event_type = "invalid".to_string();
        assert!(matches!(
            validate_event(&event),
            Err(ValidationError::InvalidCotType(_))
        ));
    }

    #[test]
    fn test_negative_circular_error() {
        let mut event = create_valid_event();
        event.point.ce = -10.0;
        assert!(matches!(
            validate_event(&event),
            Err(ValidationError::InvalidCircularError(_))
        ));
    }

    #[test]
    fn test_point_validation() {
        let valid_point = Point {
            lat: 0.0,
            lon: 0.0,
            hae: 0.0,
            ce: 10.0,
            le: 5.0,
        };
        assert!(validate_point(&valid_point).is_ok());

        let invalid_point = Point {
            lat: 100.0,
            lon: 0.0,
            hae: 0.0,
            ce: 10.0,
            le: 5.0,
        };
        assert!(validate_point(&invalid_point).is_err());
    }
}
