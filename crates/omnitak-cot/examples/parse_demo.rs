use omnitak_cot::parser::parse_cot;
use omnitak_cot::proto::{decode_event, encode_event};
use omnitak_cot::validate::validate_event;

fn main() {
    // Example CoT message
    let cot_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="ANDROID-12345678" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="h-e">
    <point lat="37.7749" lon="-122.4194" hae="100.0" ce="10.0" le="5.0"/>
    <detail>
        <contact callsign="Alpha-1"/>
        <remarks>Friendly unit in San Francisco</remarks>
    </detail>
</event>"#;

    println!("Parsing CoT message...\n");

    // Parse the CoT message
    match parse_cot(cot_xml) {
        Ok(event) => {
            println!("Successfully parsed CoT event!");
            println!("  UID: {}", event.uid);
            println!("  Type: {}", event.event_type);
            println!("  Version: {}", event.version);
            println!("  Time: {}", event.time);
            println!(
                "  Location: {:.4}, {:.4} @ {:.1}m",
                event.point.lat, event.point.lon, event.point.hae
            );
            println!(
                "  Accuracy: CE={:.1}m, LE={:.1}m",
                event.point.ce, event.point.le
            );

            // Show affiliation
            if let Some(affiliation) = event.affiliation() {
                println!("  Affiliation: {}", affiliation);
            }

            // Validate the event
            println!("\nValidating event...");
            match validate_event(&event) {
                Ok(_) => println!("  ✓ Event is valid!"),
                Err(e) => println!("  ✗ Validation error: {}", e),
            }

            // Test protobuf serialization
            println!("\nTesting Protobuf serialization...");
            match encode_event(&event) {
                Ok(bytes) => {
                    println!("  Encoded to {} bytes", bytes.len());

                    match decode_event(&bytes) {
                        Ok(decoded) => {
                            println!("  ✓ Successfully decoded!");
                            println!("  Decoded UID: {}", decoded.uid);
                        }
                        Err(e) => println!("  ✗ Decode error: {}", e),
                    }
                }
                Err(e) => println!("  ✗ Encode error: {}", e),
            }
        }
        Err(e) => {
            println!("Error parsing CoT: {}", e);
        }
    }
}
