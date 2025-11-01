use chrono::{TimeZone, Utc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use omnitak_cot::event::{Event, Point};
use omnitak_cot::parser::parse_cot;
use omnitak_cot::proto::{decode_event, encode_event};

// Example CoT messages for benchmarking
const SIMPLE_COT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="ANDROID-12345678" type="a-f-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="h-e">
    <point lat="37.7749" lon="-122.4194" hae="100.0" ce="10.0" le="5.0"/>
</event>"#;

const COT_WITH_DETAIL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="ANDROID-87654321" type="a-h-G" time="2024-01-15T10:30:00Z" start="2024-01-15T10:30:00Z" stale="2024-01-15T10:35:00Z" how="m-g">
    <point lat="38.8977" lon="-77.0365" hae="50.0" ce="5.0" le="2.5"/>
    <detail>
        <contact callsign="Bravo-2" endpoint="192.168.1.100:4242"/>
        <remarks>Enemy unit spotted near checkpoint</remarks>
        <link uid="SENSOR-001" type="a-f-G" relation="p-p"/>
        <precisionlocation geopointsrc="GPS" altsrc="GPS"/>
        <status battery="85"/>
    </detail>
</event>"#;

const COMPLEX_COT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<event version="2.0" uid="DEVICE-COMPLEX-999" type="a-n-G-E-V-C" time="2024-01-15T10:30:00.123Z" start="2024-01-15T10:30:00.123Z" stale="2024-01-15T11:30:00.123Z" how="m-g">
    <point lat="51.5074" lon="-0.1278" hae="25.0" ce="2.0" le="1.0"/>
    <detail>
        <contact callsign="Charlie-3" endpoint="10.0.0.50:8080"/>
        <remarks>Complex multi-field CoT event for testing</remarks>
        <link uid="PARENT-001" type="a-f-G-E-V" relation="p-p" production_time="2024-01-15T10:29:00Z"/>
        <link uid="CHILD-001" type="a-f-G-I" relation="c-c"/>
        <precisionlocation geopointsrc="GPS" altsrc="DTED2"/>
        <status battery="92" readiness="true"/>
        <track speed="15.5" course="270.0"/>
        <color argb="-65536"/>
        <usericon iconsetpath="34ae1613-9645-4222-a9d2-e5f243dea2865/Military/UAV.png"/>
    </detail>
</event>"#;

fn create_test_event() -> Event {
    Event {
        version: "2.0".to_string(),
        uid: "BENCH-TEST-001".to_string(),
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
            vec![
                ("callsign".to_string(), "Alpha-1".to_string()),
                ("remarks".to_string(), "Benchmark test event".to_string()),
            ]
            .into_iter()
            .collect(),
        ),
    }
}

fn bench_xml_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("xml_parsing");

    group.bench_function("simple_cot", |b| {
        b.iter(|| parse_cot(black_box(SIMPLE_COT)))
    });

    group.bench_function("cot_with_detail", |b| {
        b.iter(|| parse_cot(black_box(COT_WITH_DETAIL)))
    });

    group.bench_function("complex_cot", |b| {
        b.iter(|| parse_cot(black_box(COMPLEX_COT)))
    });

    group.finish();
}

fn bench_protobuf(c: &mut Criterion) {
    let mut group = c.benchmark_group("protobuf");

    let event = create_test_event();
    let encoded = encode_event(&event).unwrap();

    group.bench_function("encode", |b| b.iter(|| encode_event(black_box(&event))));

    group.bench_function("decode", |b| b.iter(|| decode_event(black_box(&encoded))));

    group.bench_function("roundtrip", |b| {
        b.iter(|| {
            let encoded = encode_event(black_box(&event)).unwrap();
            decode_event(&encoded)
        })
    });

    group.finish();
}

fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("xml_vs_protobuf");

    let event = create_test_event();
    let proto_encoded = encode_event(&event).unwrap();

    group.bench_with_input(
        BenchmarkId::new("xml_parse", "simple"),
        &SIMPLE_COT,
        |b, xml| b.iter(|| parse_cot(black_box(xml))),
    );

    group.bench_with_input(
        BenchmarkId::new("proto_decode", "simple"),
        &proto_encoded,
        |b, data| b.iter(|| decode_event(black_box(data))),
    );

    group.finish();
}

criterion_group!(benches, bench_xml_parsing, bench_protobuf, bench_comparison);
criterion_main!(benches);
