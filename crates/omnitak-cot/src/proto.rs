//! Protobuf support for CoT messages

use crate::event::{Event, Point};
use chrono::{DateTime, Utc};
use prost::Message;
use thiserror::Error;

// Include generated protobuf code
pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/omnitak.cot.rs"));
}

#[derive(Error, Debug)]
pub enum ProtoError {
    #[error("Protobuf encoding error: {0}")]
    EncodeError(#[from] prost::EncodeError),

    #[error("Protobuf decoding error: {0}")]
    DecodeError(#[from] prost::DecodeError),

    #[error("Invalid datetime format: {0}")]
    InvalidDateTime(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Convert Event to protobuf format
impl From<&Event> for pb::CotEvent {
    fn from(event: &Event) -> Self {
        pb::CotEvent {
            version: event.version.clone(),
            uid: event.uid.clone(),
            r#type: event.event_type.clone(),
            time: event.time.to_rfc3339(),
            start: event.start.to_rfc3339(),
            stale: event.stale.to_rfc3339(),
            how: event.how.clone(),
            point: Some(pb::Point {
                lat: event.point.lat,
                lon: event.point.lon,
                hae: event.point.hae,
                ce: event.point.ce,
                le: event.point.le,
            }),
            detail: event.detail.clone().unwrap_or_default(),
        }
    }
}

/// Convert protobuf format to Event
impl TryFrom<pb::CotEvent> for Event {
    type Error = ProtoError;

    fn try_from(proto: pb::CotEvent) -> Result<Self, Self::Error> {
        let time = proto
            .time
            .parse::<DateTime<Utc>>()
            .map_err(|_| ProtoError::InvalidDateTime(proto.time.clone()))?;

        let start = proto
            .start
            .parse::<DateTime<Utc>>()
            .map_err(|_| ProtoError::InvalidDateTime(proto.start.clone()))?;

        let stale = proto
            .stale
            .parse::<DateTime<Utc>>()
            .map_err(|_| ProtoError::InvalidDateTime(proto.stale.clone()))?;

        let point = proto
            .point
            .ok_or_else(|| ProtoError::MissingField("point".into()))?;

        Ok(Event {
            version: proto.version,
            uid: proto.uid,
            event_type: proto.r#type,
            time,
            start,
            stale,
            how: proto.how,
            point: Point {
                lat: point.lat,
                lon: point.lon,
                hae: point.hae,
                ce: point.ce,
                le: point.le,
            },
            detail: if proto.detail.is_empty() {
                None
            } else {
                Some(proto.detail)
            },
        })
    }
}

/// Encode an Event to protobuf binary format
pub fn encode_event(event: &Event) -> Result<Vec<u8>, ProtoError> {
    let proto_event = pb::CotEvent::from(event);
    let mut buf = Vec::new();
    proto_event.encode(&mut buf)?;
    Ok(buf)
}

/// Decode an Event from protobuf binary format
pub fn decode_event(data: &[u8]) -> Result<Event, ProtoError> {
    let proto_event = pb::CotEvent::decode(data)?;
    Event::try_from(proto_event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_event() -> Event {
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
            detail: Some(
                vec![("callsign".to_string(), "Alpha-1".to_string())]
                    .into_iter()
                    .collect(),
            ),
        }
    }

    #[test]
    fn test_encode_decode() {
        let event = create_test_event();
        let encoded = encode_event(&event).expect("Failed to encode");
        let decoded = decode_event(&encoded).expect("Failed to decode");

        assert_eq!(event.uid, decoded.uid);
        assert_eq!(event.version, decoded.version);
        assert_eq!(event.event_type, decoded.event_type);
        assert_eq!(event.point.lat, decoded.point.lat);
        assert_eq!(event.point.lon, decoded.point.lon);
    }

    #[test]
    fn test_proto_conversion() {
        let event = create_test_event();
        let proto = pb::CotEvent::from(&event);

        assert_eq!(proto.uid, "test-123");
        assert_eq!(proto.version, "2.0");
        assert_eq!(proto.r#type, "a-f-G");
        assert!(proto.point.is_some());
    }
}
