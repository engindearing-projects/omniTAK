//! High-performance XML parser for CoT messages using quick-xml

use crate::event::{Event, Point};
use chrono::{DateTime, Utc};
use quick_xml::events::Event as XmlEvent;
use quick_xml::Reader;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("XML parsing error: {0}")]
    XmlError(#[from] quick_xml::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid datetime format: {0}")]
    InvalidDateTime(String),

    #[error("Invalid number format: {0}")]
    InvalidNumber(String),

    #[error("Invalid event structure: {0}")]
    InvalidStructure(String),
}

/// Parse a CoT message from XML string
pub fn parse_cot(xml: &str) -> Result<Event, ParseError> {
    parse_cot_bytes(xml.as_bytes())
}

/// Parse a CoT message from XML bytes (zero-copy where possible)
pub fn parse_cot_bytes(xml: &[u8]) -> Result<Event, ParseError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    // Event fields
    let mut version = None;
    let mut uid = None;
    let mut event_type = None;
    let mut time = None;
    let mut start = None;
    let mut stale = None;
    let mut how = None;
    let mut point = None;
    let mut detail = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XmlEvent::Start(e)) | Ok(XmlEvent::Empty(e)) => {
                match e.name().as_ref() {
                    b"event" => {
                        // Parse event attributes
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                ParseError::XmlError(quick_xml::Error::InvalidAttr(e))
                            })?;
                            match attr.key.as_ref() {
                                b"version" => {
                                    version = Some(
                                        String::from_utf8_lossy(attr.value.as_ref()).into_owned(),
                                    );
                                }
                                b"uid" => {
                                    uid = Some(
                                        String::from_utf8_lossy(attr.value.as_ref()).into_owned(),
                                    );
                                }
                                b"type" => {
                                    event_type = Some(
                                        String::from_utf8_lossy(attr.value.as_ref()).into_owned(),
                                    );
                                }
                                b"time" => {
                                    let time_str = String::from_utf8_lossy(attr.value.as_ref());
                                    time = Some(parse_datetime(&time_str)?);
                                }
                                b"start" => {
                                    let start_str = String::from_utf8_lossy(attr.value.as_ref());
                                    start = Some(parse_datetime(&start_str)?);
                                }
                                b"stale" => {
                                    let stale_str = String::from_utf8_lossy(attr.value.as_ref());
                                    stale = Some(parse_datetime(&stale_str)?);
                                }
                                b"how" => {
                                    how = Some(
                                        String::from_utf8_lossy(attr.value.as_ref()).into_owned(),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    b"point" => {
                        // Parse point attributes
                        let mut lat = None;
                        let mut lon = None;
                        let mut hae = None;
                        let mut ce = None;
                        let mut le = None;

                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                ParseError::XmlError(quick_xml::Error::InvalidAttr(e))
                            })?;
                            match attr.key.as_ref() {
                                b"lat" => {
                                    let lat_str = String::from_utf8_lossy(attr.value.as_ref());
                                    lat = Some(parse_f64(&lat_str)?);
                                }
                                b"lon" => {
                                    let lon_str = String::from_utf8_lossy(attr.value.as_ref());
                                    lon = Some(parse_f64(&lon_str)?);
                                }
                                b"hae" => {
                                    let hae_str = String::from_utf8_lossy(attr.value.as_ref());
                                    hae = Some(parse_f64(&hae_str)?);
                                }
                                b"ce" => {
                                    let ce_str = String::from_utf8_lossy(attr.value.as_ref());
                                    ce = Some(parse_f64(&ce_str)?);
                                }
                                b"le" => {
                                    let le_str = String::from_utf8_lossy(attr.value.as_ref());
                                    le = Some(parse_f64(&le_str)?);
                                }
                                _ => {}
                            }
                        }

                        point = Some(Point {
                            lat: lat.ok_or_else(|| ParseError::MissingField("lat".into()))?,
                            lon: lon.ok_or_else(|| ParseError::MissingField("lon".into()))?,
                            hae: hae.unwrap_or(0.0),
                            ce: ce.unwrap_or(9999999.0),
                            le: le.unwrap_or(9999999.0),
                        });
                    }
                    b"detail" => {
                        // Parse detail section (simplified as HashMap for now)
                        detail = Some(parse_detail(&mut reader, &mut buf)?);
                    }
                    _ => {}
                }
            }
            Ok(XmlEvent::Eof) => break,
            Err(e) => return Err(ParseError::XmlError(e)),
            _ => {}
        }
        buf.clear();
    }

    // Build Event struct
    Ok(Event {
        version: version.ok_or_else(|| ParseError::MissingField("version".into()))?,
        uid: uid.ok_or_else(|| ParseError::MissingField("uid".into()))?,
        event_type: event_type.ok_or_else(|| ParseError::MissingField("type".into()))?,
        time: time.ok_or_else(|| ParseError::MissingField("time".into()))?,
        start: start.ok_or_else(|| ParseError::MissingField("start".into()))?,
        stale: stale.ok_or_else(|| ParseError::MissingField("stale".into()))?,
        how: how.ok_or_else(|| ParseError::MissingField("how".into()))?,
        point: point.ok_or_else(|| ParseError::MissingField("point".into()))?,
        detail,
    })
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ParseError> {
    s.parse::<DateTime<Utc>>()
        .map_err(|_| ParseError::InvalidDateTime(s.to_string()))
}

fn parse_f64(s: &str) -> Result<f64, ParseError> {
    s.parse::<f64>()
        .map_err(|_| ParseError::InvalidNumber(s.to_string()))
}

fn parse_detail(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
) -> Result<HashMap<String, String>, ParseError> {
    let mut detail_map = HashMap::new();
    let mut current_tag = String::new();
    let mut depth = 1;

    loop {
        match reader.read_event_into(buf) {
            Ok(XmlEvent::Start(e)) => {
                depth += 1;
                current_tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
            }
            Ok(XmlEvent::Empty(e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                // Store empty tags with empty string value
                detail_map.insert(tag, String::new());
            }
            Ok(XmlEvent::Text(e)) => {
                if !current_tag.is_empty() {
                    let text = e
                        .unescape()
                        .map_err(|e| ParseError::XmlError(e.into()))?
                        .into_owned();
                    detail_map.insert(current_tag.clone(), text);
                }
            }
            Ok(XmlEvent::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
                current_tag.clear();
            }
            Ok(XmlEvent::Eof) => break,
            Err(e) => return Err(ParseError::XmlError(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(detail_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_COT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="ANDROID-12345678" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="h-e">
    <point lat="37.7749" lon="-122.4194" hae="100.0" ce="10.0" le="5.0"/>
    <detail>
        <contact callsign="Alpha-1"/>
        <remarks>Test CoT message</remarks>
    </detail>
</event>"#;

    #[test]
    fn test_parse_cot() {
        let event = parse_cot(EXAMPLE_COT).expect("Failed to parse CoT");

        assert_eq!(event.version, "2.0");
        assert_eq!(event.uid, "ANDROID-12345678");
        assert_eq!(event.event_type, "a-f-G");
        assert_eq!(event.how, "h-e");

        assert_eq!(event.point.lat, 37.7749);
        assert_eq!(event.point.lon, -122.4194);
        assert_eq!(event.point.hae, 100.0);
        assert_eq!(event.point.ce, 10.0);
        assert_eq!(event.point.le, 5.0);

        assert!(event.detail.is_some());
    }

    #[test]
    fn test_parse_minimal_cot() {
        let minimal_cot = r#"<event version="2.0" uid="test-1" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="m-g">
    <point lat="0.0" lon="0.0" hae="0.0"/>
</event>"#;

        let event = parse_cot(minimal_cot).expect("Failed to parse minimal CoT");
        assert_eq!(event.version, "2.0");
        assert_eq!(event.uid, "test-1");
    }

    #[test]
    fn test_parse_malformed_cot() {
        let malformed = r#"<event version="2.0">invalid</event>"#;
        assert!(parse_cot(malformed).is_err());
    }

    #[test]
    fn test_parse_cot_bytes() {
        let event = parse_cot_bytes(EXAMPLE_COT.as_bytes()).expect("Failed to parse CoT bytes");
        assert_eq!(event.uid, "ANDROID-12345678");
    }
}
