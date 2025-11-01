//! XML serialization for CoT messages

use crate::event::{Contact, Detail, Event, Group, PrecisionLocation, Status, Takv, Track};
use std::fmt::Write;

/// Serialize an Event to XML string
pub fn serialize_event(event: &Event) -> String {
    let mut xml = String::new();

    // XML header
    writeln!(xml, r#"<?xml version="1.0" encoding="UTF-8"?>"#).unwrap();

    // Event element
    write!(
        xml,
        r#"<event version="{}" uid="{}" type="{}" time="{}" start="{}" stale="{}" how="{}">"#,
        event.version,
        event.uid,
        event.event_type,
        event.time.to_rfc3339(),
        event.start.to_rfc3339(),
        event.stale.to_rfc3339(),
        event.how
    ).unwrap();

    // Point element
    write!(
        xml,
        r#"<point lat="{}" lon="{}" hae="{}" ce="{}" le="{}"/>"#,
        event.point.lat,
        event.point.lon,
        event.point.hae,
        event.point.ce,
        event.point.le
    ).unwrap();

    // Detail section
    if let Some(ref detail) = event.detail {
        write!(xml, "<detail>").unwrap();
        serialize_detail(&mut xml, detail);
        write!(xml, "</detail>").unwrap();
    }

    writeln!(xml, "</event>").unwrap();
    xml
}

/// Serialize Detail to XML (internal helper)
fn serialize_detail(xml: &mut String, detail: &Detail) {
    // Serialize structured fields
    if let Some(ref contact) = detail.contact {
        serialize_contact(xml, contact);
    }

    if let Some(ref group) = detail.group {
        serialize_group(xml, group);
    }

    if let Some(ref track) = detail.track {
        serialize_track(xml, track);
    }

    if let Some(ref status) = detail.status {
        serialize_status(xml, status);
    }

    if let Some(ref takv) = detail.takv {
        serialize_takv(xml, takv);
    }

    if let Some(ref precision_location) = detail.precision_location {
        serialize_precision_location(xml, precision_location);
    }

    // Add remaining XML detail (unparsed content)
    if let Some(ref xml_detail) = detail.xml_detail {
        write!(xml, "{}", xml_detail).unwrap();
    }
}

fn serialize_contact(xml: &mut String, contact: &Contact) {
    write!(xml, r#"<contact callsign="{}""#, contact.callsign).unwrap();
    if let Some(ref endpoint) = contact.endpoint {
        write!(xml, r#" endpoint="{}""#, endpoint).unwrap();
    }
    write!(xml, "/>").unwrap();
}

fn serialize_group(xml: &mut String, group: &Group) {
    write!(
        xml,
        r#"<__group name="{}" role="{}"/>"#,
        group.name, group.role
    ).unwrap();
}

fn serialize_track(xml: &mut String, track: &Track) {
    write!(
        xml,
        r#"<track speed="{}" course="{}"/>"#,
        track.speed, track.course
    ).unwrap();
}

fn serialize_status(xml: &mut String, status: &Status) {
    write!(xml, r#"<status battery="{}"/>"#, status.battery).unwrap();
}

fn serialize_takv(xml: &mut String, takv: &Takv) {
    write!(
        xml,
        r#"<takv device="{}" platform="{}" os="{}" version="{}"/>"#,
        takv.device, takv.platform, takv.os, takv.version
    ).unwrap();
}

fn serialize_precision_location(xml: &mut String, pl: &PrecisionLocation) {
    write!(
        xml,
        r#"<precisionlocation geopointsrc="{}" altsrc="{}"/>"#,
        pl.geopointsrc, pl.altsrc
    ).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Point;
    use chrono::Utc;

    #[test]
    fn test_serialize_event_minimal() {
        let event = Event {
            version: "2.0".to_string(),
            uid: "test-1".to_string(),
            event_type: "a-f-G".to_string(),
            time: Utc::now(),
            start: Utc::now(),
            stale: Utc::now(),
            how: "h-e".to_string(),
            point: Point::new(37.7749, -122.4194, 100.0),
            detail: None,
        };

        let xml = serialize_event(&event);
        assert!(xml.contains(r#"uid="test-1""#));
        assert!(xml.contains(r#"type="a-f-G""#));
    }

    #[test]
    fn test_serialize_event_with_contact() {
        let event = Event {
            version: "2.0".to_string(),
            uid: "test-1".to_string(),
            event_type: "a-f-G".to_string(),
            time: Utc::now(),
            start: Utc::now(),
            stale: Utc::now(),
            how: "h-e".to_string(),
            point: Point::new(37.7749, -122.4194, 100.0),
            detail: Some(Detail {
                contact: Some(Contact {
                    callsign: "Alpha-1".to_string(),
                    endpoint: Some("192.168.1.100:4242".to_string()),
                }),
                ..Default::default()
            }),
        };

        let xml = serialize_event(&event);
        assert!(xml.contains(r#"callsign="Alpha-1""#));
        assert!(xml.contains(r#"endpoint="192.168.1.100:4242""#));
    }

    #[test]
    fn test_serialize_event_with_track() {
        let event = Event {
            version: "2.0".to_string(),
            uid: "test-1".to_string(),
            event_type: "a-f-G".to_string(),
            time: Utc::now(),
            start: Utc::now(),
            stale: Utc::now(),
            how: "h-e".to_string(),
            point: Point::new(37.7749, -122.4194, 100.0),
            detail: Some(Detail {
                track: Some(Track {
                    speed: 10.5,
                    course: 270.0,
                }),
                ..Default::default()
            }),
        };

        let xml = serialize_event(&event);
        assert!(xml.contains(r#"speed="10.5""#));
        assert!(xml.contains(r#"course="270""#));
    }
}
