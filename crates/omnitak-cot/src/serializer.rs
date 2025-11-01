//! XML serialization for CoT messages

use crate::event::{Contact, Detail, Event, Group, Link, Point, PrecisionLocation, Shape, Status, Takv, Track};
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

    // Serialize links
    for link in &detail.link {
        serialize_link(xml, link);
    }

    // Serialize shape
    if let Some(ref shape) = detail.shape {
        serialize_shape(xml, shape);
    }

    // Serialize color attributes
    if let Some(color) = detail.color {
        write!(xml, r#"<color value="{}"/>"#, color).unwrap();
    }

    if let Some(fill_color) = detail.fill_color {
        write!(xml, r#"<fillColor value="{}"/>"#, fill_color).unwrap();
    }

    if let Some(stroke_color) = detail.stroke_color {
        write!(xml, r#"<strokeColor value="{}"/>"#, stroke_color).unwrap();
    }

    if let Some(stroke_weight) = detail.stroke_weight {
        write!(xml, r#"<strokeWeight value="{}"/>"#, stroke_weight).unwrap();
    }

    if let Some(labels_on) = detail.labels_on {
        write!(xml, r#"<labels_on value="{}"/>"#, labels_on).unwrap();
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

fn serialize_link(xml: &mut String, link: &Link) {
    write!(xml, r#"<link uid="{}" relation="{}""#, link.uid, link.relation).unwrap();
    if let Some(ref link_type) = link.link_type {
        write!(xml, r#" type="{}""#, link_type).unwrap();
    }
    write!(xml, "/>").unwrap();
}

fn serialize_shape(xml: &mut String, shape: &Shape) {
    write!(xml, "<shape>").unwrap();
    match shape {
        Shape::Ellipse { major, minor, angle } => {
            write!(
                xml,
                r#"<ellipse major="{}" minor="{}" angle="{}"/>"#,
                major, minor, angle
            ).unwrap();
        }
        Shape::Polyline { vertices, closed } => {
            write!(xml, r#"<polyline closed="{}">"#, closed).unwrap();
            for vertex in vertices {
                write!(
                    xml,
                    r#"<vertex lat="{}" lon="{}" hae="{}"/>"#,
                    vertex.lat, vertex.lon, vertex.hae
                ).unwrap();
            }
            write!(xml, "</polyline>").unwrap();
        }
    }
    write!(xml, "</shape>").unwrap();
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

    #[test]
    fn test_serialize_event_with_circle() {
        use crate::event::Shape;

        let event = Event {
            version: "2.0".to_string(),
            uid: "circle-test".to_string(),
            event_type: "u-d-f".to_string(),
            time: Utc::now(),
            start: Utc::now(),
            stale: Utc::now(),
            how: "h-g-i-g-o".to_string(),
            point: Point::new(37.7749, -122.4194, 0.0),
            detail: Some(Detail {
                contact: Some(Contact {
                    callsign: "5km Exclusion Zone".to_string(),
                    endpoint: None,
                }),
                shape: Some(Shape::Ellipse {
                    major: 5000.0,
                    minor: 5000.0,
                    angle: 0.0,
                }),
                color: Some(-65536), // Red
                stroke_color: Some(-65536),
                stroke_weight: Some(2.0),
                labels_on: Some(true),
                ..Default::default()
            }),
        };

        let xml = serialize_event(&event);
        assert!(xml.contains(r#"callsign="5km Exclusion Zone""#));
        assert!(xml.contains(r#"<shape>"#));
        assert!(xml.contains(r#"<ellipse major="5000" minor="5000" angle="0"/>"#));
        assert!(xml.contains(r#"<color value="-65536"/>"#));
        assert!(xml.contains(r#"<strokeColor value="-65536"/>"#));
        assert!(xml.contains(r#"<strokeWeight value="2"/>"#));
        assert!(xml.contains(r#"<labels_on value="true"/>"#));
    }

    #[test]
    fn test_serialize_event_with_polygon() {
        use crate::event::Shape;

        let event = Event {
            version: "2.0".to_string(),
            uid: "polygon-test".to_string(),
            event_type: "u-d-f".to_string(),
            time: Utc::now(),
            start: Utc::now(),
            stale: Utc::now(),
            how: "h-g-i-g-o".to_string(),
            point: Point::new(34.0, -118.0, 0.0),
            detail: Some(Detail {
                contact: Some(Contact {
                    callsign: "Area of Operations".to_string(),
                    endpoint: None,
                }),
                shape: Some(Shape::Polyline {
                    vertices: vec![
                        Point::new(34.0, -118.0, 0.0),
                        Point::new(34.0, -117.0, 0.0),
                        Point::new(33.5, -117.0, 0.0),
                        Point::new(33.5, -118.0, 0.0),
                    ],
                    closed: true,
                }),
                color: Some(-16711936), // Green
                fill_color: Some(1342177280), // Semi-transparent green
                ..Default::default()
            }),
        };

        let xml = serialize_event(&event);
        assert!(xml.contains(r#"<shape>"#));
        assert!(xml.contains(r#"<polyline closed="true">"#));
        assert!(xml.contains(r#"<vertex lat="34" lon="-118" hae="0"/>"#));
        assert!(xml.contains(r#"<vertex lat="34" lon="-117" hae="0"/>"#));
        assert!(xml.contains(r#"<color value="-16711936"/>"#));
        assert!(xml.contains(r#"<fillColor value="1342177280"/>"#));
    }

    #[test]
    fn test_serialize_event_with_links() {
        use crate::event::Link;

        let event = Event {
            version: "2.0".to_string(),
            uid: "route-test".to_string(),
            event_type: "b-m-p-s-p-loc".to_string(),
            time: Utc::now(),
            start: Utc::now(),
            stale: Utc::now(),
            how: "h-g-i-g-o".to_string(),
            point: Point::new(33.123, -117.456, 0.0),
            detail: Some(Detail {
                contact: Some(Contact {
                    callsign: "Patrol Route Alpha".to_string(),
                    endpoint: None,
                }),
                link: vec![
                    Link {
                        uid: "waypoint-1".to_string(),
                        link_type: Some("b-m-p-s-p-loc".to_string()),
                        relation: "c".to_string(),
                    },
                    Link {
                        uid: "waypoint-2".to_string(),
                        link_type: Some("b-m-p-s-p-loc".to_string()),
                        relation: "c".to_string(),
                    },
                ],
                color: Some(-256), // Yellow
                labels_on: Some(true),
                ..Default::default()
            }),
        };

        let xml = serialize_event(&event);
        assert!(xml.contains(r#"<link uid="waypoint-1" relation="c" type="b-m-p-s-p-loc"/>"#));
        assert!(xml.contains(r#"<link uid="waypoint-2" relation="c" type="b-m-p-s-p-loc"/>"#));
        assert!(xml.contains(r#"<color value="-256"/>"#));
    }
}
