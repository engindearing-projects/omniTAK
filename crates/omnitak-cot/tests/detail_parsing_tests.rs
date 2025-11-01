//! Comprehensive tests for structured Detail field parsing

use omnitak_cot::{parse_cot, serialize_event};

#[test]
fn test_parse_contact_detail() {
    let cot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="ANDROID-12345678" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="h-e">
    <point lat="37.7749" lon="-122.4194" hae="100.0" ce="10.0" le="5.0"/>
    <detail>
        <contact callsign="Alpha-1" endpoint="192.168.1.100:4242"/>
    </detail>
</event>"#;

    let event = parse_cot(cot_xml).expect("Failed to parse CoT");

    assert!(event.detail.is_some());
    let detail = event.detail.unwrap();

    assert!(detail.contact.is_some());
    let contact = detail.contact.unwrap();
    assert_eq!(contact.callsign, "Alpha-1");
    assert_eq!(contact.endpoint, Some("192.168.1.100:4242".to_string()));
}

#[test]
fn test_parse_contact_without_endpoint() {
    let cot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="TEST-1" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="h-e">
    <point lat="0.0" lon="0.0" hae="0.0"/>
    <detail>
        <contact callsign="Bravo-2"/>
    </detail>
</event>"#;

    let event = parse_cot(cot_xml).expect("Failed to parse CoT");
    let detail = event.detail.unwrap();
    let contact = detail.contact.unwrap();

    assert_eq!(contact.callsign, "Bravo-2");
    assert_eq!(contact.endpoint, None);
}

#[test]
fn test_parse_group_detail() {
    let cot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="TEST-2" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="h-e">
    <point lat="0.0" lon="0.0" hae="0.0"/>
    <detail>
        <__group name="Cyan" role="Team Member"/>
    </detail>
</event>"#;

    let event = parse_cot(cot_xml).expect("Failed to parse CoT");
    let detail = event.detail.unwrap();

    assert!(detail.group.is_some());
    let group = detail.group.unwrap();
    assert_eq!(group.name, "Cyan");
    assert_eq!(group.role, "Team Member");
}

#[test]
fn test_parse_track_detail() {
    let cot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="TEST-3" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="m-g">
    <point lat="37.7749" lon="-122.4194" hae="100.0"/>
    <detail>
        <track speed="15.5" course="270.0"/>
    </detail>
</event>"#;

    let event = parse_cot(cot_xml).expect("Failed to parse CoT");
    let detail = event.detail.unwrap();

    assert!(detail.track.is_some());
    let track = detail.track.unwrap();
    assert_eq!(track.speed, 15.5);
    assert_eq!(track.course, 270.0);
}

#[test]
fn test_parse_status_detail() {
    let cot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="TEST-4" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="h-e">
    <point lat="0.0" lon="0.0" hae="0.0"/>
    <detail>
        <status battery="75"/>
    </detail>
</event>"#;

    let event = parse_cot(cot_xml).expect("Failed to parse CoT");
    let detail = event.detail.unwrap();

    assert!(detail.status.is_some());
    let status = detail.status.unwrap();
    assert_eq!(status.battery, 75);
}

#[test]
fn test_parse_takv_detail() {
    let cot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="TEST-5" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="h-e">
    <point lat="0.0" lon="0.0" hae="0.0"/>
    <detail>
        <takv device="Samsung Galaxy S21" platform="ATAK" os="Android 12" version="4.5.1.5"/>
    </detail>
</event>"#;

    let event = parse_cot(cot_xml).expect("Failed to parse CoT");
    let detail = event.detail.unwrap();

    assert!(detail.takv.is_some());
    let takv = detail.takv.unwrap();
    assert_eq!(takv.device, "Samsung Galaxy S21");
    assert_eq!(takv.platform, "ATAK");
    assert_eq!(takv.os, "Android 12");
    assert_eq!(takv.version, "4.5.1.5");
}

#[test]
fn test_parse_precision_location_detail() {
    let cot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="TEST-6" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="h-e">
    <point lat="37.7749" lon="-122.4194" hae="100.0"/>
    <detail>
        <precisionlocation geopointsrc="GPS" altsrc="DTED"/>
    </detail>
</event>"#;

    let event = parse_cot(cot_xml).expect("Failed to parse CoT");
    let detail = event.detail.unwrap();

    assert!(detail.precision_location.is_some());
    let pl = detail.precision_location.unwrap();
    assert_eq!(pl.geopointsrc, "GPS");
    assert_eq!(pl.altsrc, "DTED");
}

#[test]
fn test_parse_multiple_structured_fields() {
    let cot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="TEST-7" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="m-g">
    <point lat="37.7749" lon="-122.4194" hae="100.0" ce="10.0" le="5.0"/>
    <detail>
        <contact callsign="Delta-5" endpoint="10.0.0.5:4242"/>
        <__group name="Red" role="Team Lead"/>
        <track speed="25.0" course="180.0"/>
        <status battery="90"/>
        <takv device="iPhone 13" platform="iTAK" os="iOS 16" version="1.2.3"/>
        <precisionlocation geopointsrc="GPS" altsrc="GPS"/>
    </detail>
</event>"#;

    let event = parse_cot(cot_xml).expect("Failed to parse CoT");
    let detail = event.detail.as_ref().unwrap();

    // Verify all fields were parsed
    assert!(detail.contact.is_some());
    assert!(detail.group.is_some());
    assert!(detail.track.is_some());
    assert!(detail.status.is_some());
    assert!(detail.takv.is_some());
    assert!(detail.precision_location.is_some());

    // Verify contact
    let contact = detail.contact.as_ref().unwrap();
    assert_eq!(contact.callsign, "Delta-5");
    assert_eq!(contact.endpoint, Some("10.0.0.5:4242".to_string()));

    // Verify group
    let group = detail.group.as_ref().unwrap();
    assert_eq!(group.name, "Red");
    assert_eq!(group.role, "Team Lead");

    // Verify track
    let track = detail.track.as_ref().unwrap();
    assert_eq!(track.speed, 25.0);
    assert_eq!(track.course, 180.0);

    // Verify status
    let status = detail.status.as_ref().unwrap();
    assert_eq!(status.battery, 90);

    // Verify takv
    let takv = detail.takv.as_ref().unwrap();
    assert_eq!(takv.device, "iPhone 13");
    assert_eq!(takv.platform, "iTAK");

    // Verify precision location
    let pl = detail.precision_location.as_ref().unwrap();
    assert_eq!(pl.geopointsrc, "GPS");
    assert_eq!(pl.altsrc, "GPS");
}

#[test]
fn test_roundtrip_xml_parsing_serialization() {
    let original_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="ROUNDTRIP-1" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="h-e">
    <point lat="37.7749" lon="-122.4194" hae="100.0" ce="10.0" le="5.0"/>
    <detail>
        <contact callsign="Echo-1"/>
        <track speed="10.0" course="90.0"/>
    </detail>
</event>"#;

    // Parse the XML
    let event = parse_cot(original_xml).expect("Failed to parse CoT");

    // Serialize it back to XML
    let serialized_xml = serialize_event(&event);

    // Parse the serialized XML
    let reparsed_event = parse_cot(&serialized_xml).expect("Failed to reparse CoT");

    // Verify key fields match
    assert_eq!(event.uid, reparsed_event.uid);
    assert_eq!(event.event_type, reparsed_event.event_type);
    assert_eq!(event.point.lat, reparsed_event.point.lat);
    assert_eq!(event.point.lon, reparsed_event.point.lon);

    // Verify detail fields match
    let original_detail = event.detail.as_ref().unwrap();
    let reparsed_detail = reparsed_event.detail.as_ref().unwrap();

    let original_contact = original_detail.contact.as_ref().unwrap();
    let reparsed_contact = reparsed_detail.contact.as_ref().unwrap();
    assert_eq!(original_contact.callsign, reparsed_contact.callsign);

    let original_track = original_detail.track.as_ref().unwrap();
    let reparsed_track = reparsed_detail.track.as_ref().unwrap();
    assert_eq!(original_track.speed, reparsed_track.speed);
    assert_eq!(original_track.course, reparsed_track.course);
}

#[test]
fn test_event_convenience_methods() {
    let cot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="TEST-8" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="m-g">
    <point lat="37.7749" lon="-122.4194" hae="100.0"/>
    <detail>
        <contact callsign="Foxtrot-1"/>
        <__group name="Blue" role="Scout"/>
        <track speed="12.5" course="45.0"/>
    </detail>
</event>"#;

    let event = parse_cot(cot_xml).expect("Failed to parse CoT");

    // Test convenience methods
    assert_eq!(event.callsign(), Some("Foxtrot-1"));
    assert_eq!(event.group_name(), Some("Blue"));
    assert_eq!(event.speed(), Some(12.5));
    assert_eq!(event.course(), Some(45.0));
}
