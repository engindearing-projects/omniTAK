//! Performance test for filter operations
//!
//! Run with: cargo run --release --example performance_test

use omnitak_filter::affiliation::CotType;
use omnitak_filter::fast_path::{fast_extract_affiliation, fast_is_friendly, fast_in_bbox};
use omnitak_filter::router::{Route, RouteTableBuilder};
use omnitak_filter::rules::{AffiliationFilter, CotMessage, FilterRule, GeoBoundingBoxFilter};
use std::sync::Arc;
use std::time::Instant;

fn main() {
    println!("=== OmniTAK Filter Performance Test ===\n");

    let iterations = 1_000_000;

    // Test 1: Affiliation parsing
    println!("1. Affiliation Parsing Performance:");
    let cot_type = "a-f-G-E-V-C-U-I";

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = std::hint::black_box(CotType::parse(cot_type));
    }
    let duration = start.elapsed();
    println!(
        "   Normal parse: {:.2}ns per operation",
        duration.as_nanos() as f64 / iterations as f64
    );

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = std::hint::black_box(fast_extract_affiliation(cot_type));
    }
    let duration = start.elapsed();
    println!(
        "   Fast extract: {:.2}ns per operation",
        duration.as_nanos() as f64 / iterations as f64
    );

    // Test 2: Friendly check
    println!("\n2. Affiliation Check Performance:");
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = std::hint::black_box(fast_is_friendly(cot_type));
    }
    let duration = start.elapsed();
    println!(
        "   Fast is_friendly: {:.2}ns per operation",
        duration.as_nanos() as f64 / iterations as f64
    );

    let start = Instant::now();
    for _ in 0..iterations {
        let cot = CotType::parse(cot_type);
        let _ = std::hint::black_box(cot.is_friendly());
    }
    let duration = start.elapsed();
    println!(
        "   Normal is_friendly: {:.2}ns per operation",
        duration.as_nanos() as f64 / iterations as f64
    );

    // Test 3: Geographic bounding box
    println!("\n3. Geographic Filtering Performance:");
    let bbox = [40.0, 45.0, -75.0, -70.0];
    let lat = 42.0;
    let lon = -72.0;

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = std::hint::black_box(fast_in_bbox(lat, lon, &bbox));
    }
    let duration = start.elapsed();
    println!(
        "   Fast bbox check: {:.2}ns per operation",
        duration.as_nanos() as f64 / iterations as f64
    );

    // Test 4: Filter evaluation
    println!("\n4. Filter Evaluation Performance:");
    let msg = CotMessage {
        cot_type: "a-f-G-E-V-C",
        uid: "TEST-001",
        callsign: Some("ALPHA-1"),
        group: Some("Blue Force"),
        team: Some("Alpha"),
        lat: 40.7128,
        lon: -74.0060,
        hae: Some(100.0),
    };

    let aff_filter = AffiliationFilter::friendly_only();
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = std::hint::black_box(aff_filter.evaluate(&msg));
    }
    let duration = start.elapsed();
    println!(
        "   Affiliation filter: {:.2}ns per operation",
        duration.as_nanos() as f64 / iterations as f64
    );

    let geo_filter = GeoBoundingBoxFilter::new(40.0, 41.0, -75.0, -73.0);
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = std::hint::black_box(geo_filter.evaluate(&msg));
    }
    let duration = start.elapsed();
    println!(
        "   Geographic filter: {:.2}ns per operation",
        duration.as_nanos() as f64 / iterations as f64
    );

    // Test 5: Routing performance
    println!("\n5. Routing Performance:");
    let table = RouteTableBuilder::multicast()
        .add_route(Route::new(
            "friendly".to_string(),
            "Friendly units".to_string(),
            Arc::new(AffiliationFilter::friendly_only()),
            vec!["dest1".to_string()],
            100,
        ))
        .add_route(Route::new(
            "nyc".to_string(),
            "NYC area".to_string(),
            Arc::new(GeoBoundingBoxFilter::new(40.0, 41.0, -75.0, -73.0)),
            vec!["dest2".to_string()],
            90,
        ))
        .build();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = std::hint::black_box(table.route(&msg));
    }
    let duration = start.elapsed();
    println!(
        "   Route with 2 filters: {:.2}ns per operation",
        duration.as_nanos() as f64 / iterations as f64
    );

    // Test 6: Larger route table
    let large_table = RouteTableBuilder::multicast()
        .add_route(Route::new(
            "r1".to_string(),
            "Route 1".to_string(),
            Arc::new(AffiliationFilter::friendly_only()),
            vec!["d1".to_string()],
            100,
        ))
        .add_route(Route::new(
            "r2".to_string(),
            "Route 2".to_string(),
            Arc::new(GeoBoundingBoxFilter::new(40.0, 41.0, -75.0, -73.0)),
            vec!["d2".to_string()],
            90,
        ))
        .add_route(Route::new(
            "r3".to_string(),
            "Route 3".to_string(),
            Arc::new(GeoBoundingBoxFilter::new(30.0, 50.0, -80.0, -70.0)),
            vec!["d3".to_string()],
            80,
        ))
        .add_route(Route::new(
            "r4".to_string(),
            "Route 4".to_string(),
            Arc::new(AffiliationFilter::friendly_only()),
            vec!["d4".to_string()],
            70,
        ))
        .build();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = std::hint::black_box(large_table.route(&msg));
    }
    let duration = start.elapsed();
    println!(
        "   Route with 4 filters: {:.2}ns per operation",
        duration.as_nanos() as f64 / iterations as f64
    );

    println!("\n=== Performance Test Complete ===");
    println!("\nTarget: <100ns per filter check");
    println!("All operations should be well below this threshold.");
}
