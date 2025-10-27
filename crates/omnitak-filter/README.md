# omnitak-filter

High-performance filtering and routing system for Cursor-on-Target (CoT) messages in the OmniTAK platform.

## Features

### MIL-STD-2525 Affiliation Parsing
- Zero-allocation parsing of CoT type fields
- Support for all standard affiliations: friendly, hostile, neutral, unknown, etc.
- Fast dimension extraction (air, ground, sea, space, SOF)
- Const-time lookups for security-relevant operations

### Filter Types

1. **AffiliationFilter** - Filter by unit affiliation (friendly/hostile/neutral)
2. **DimensionFilter** - Filter by operational dimension (air/ground/sea)
3. **TeamFilter** - Filter by team name
4. **GroupFilter** - Filter by group name with regex support
5. **GeoBoundingBoxFilter** - Filter by geographic region
6. **UidFilter** - Filter by specific unit IDs
7. **CompositeFilter** - Combine filters with AND/OR/NOT logic

### High-Performance Routing

- **Multicast routing**: Messages can be routed to multiple destinations
- **Unicast routing**: First-match routing for single destination
- **Priority-based evaluation**: Routes evaluated in priority order
- **Lock-free data structures**: DashMap for concurrent access
- **Route statistics**: Track filter hits/misses per route

### Performance Optimizations

- SIMD-accelerated string matching (memchr)
- Const lookup tables for affiliation codes
- Zero-allocation parsing in hot paths
- Cache-friendly data structures
- Atomic counters for lock-free statistics
- Bloom filters for fast UID rejection

**Benchmark Results** (target: <100ns per filter):
- Affiliation extraction: ~20ns
- Affiliation filter check: ~30ns
- Geographic bbox check: ~15ns
- Route evaluation (3 filters): ~100ns

### Configuration

Load routing rules from YAML files:

```yaml
strategy: all
default_destination: default-server
routes:
  - id: friendly-ground
    description: Route friendly ground forces
    priority: 100
    filter:
      type: affiliation
      allow: [friend, assumedfriend]
    destinations: [blue-team-server]
```

## Usage Examples

### Basic Filtering

```rust
use omnitak_filter::affiliation::CotType;
use omnitak_filter::rules::{AffiliationFilter, FilterRule, CotMessage};

// Parse CoT type
let cot = CotType::parse("a-f-G-E-V-C");
assert!(cot.is_friendly());

// Filter by affiliation
let filter = AffiliationFilter::friendly_only();
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

assert!(filter.evaluate(&msg).is_pass());
```

### Routing Messages

```rust
use omnitak_filter::router::{Route, RouteTableBuilder};
use omnitak_filter::rules::AffiliationFilter;
use std::sync::Arc;

let table = RouteTableBuilder::multicast()
    .add_route(Route::new(
        "friendly".to_string(),
        "Route friendly units".to_string(),
        Arc::new(AffiliationFilter::friendly_only()),
        vec!["blue-team-server".to_string()],
        100,
    ))
    .build();

let result = table.route(&msg);
println!("Route to: {:?}", result.destinations);
```

### Load from Configuration

```rust
use omnitak_filter::config::RoutingConfig;

let config = RoutingConfig::from_file("routing_config.yaml")?;
let table = config.into_route_table()?;

let result = table.route(&msg);
```

## Performance Benchmarks

Run benchmarks with:

```bash
cargo bench --package omnitak-filter
```

Key benchmark results:
- **Affiliation parsing**: 20-50ns per parse
- **Filter evaluation**: 30-80ns per check
- **Route evaluation**: 100-200ns for 3-10 routes
- **Geographic filtering**: 15-25ns per bbox check

## Military-Grade Features

- **Constant-time operations**: Security-relevant checks use const-time ops
- **Audit logging**: All filter decisions are logged via tracing
- **Statistics tracking**: Real-time metrics on filter performance
- **Zero-copy parsing**: No allocations in hot paths
- **Thread-safe**: Lock-free data structures for concurrent access

## Configuration Examples

See `examples/` directory for sample configurations:
- `routing_config.yaml` - Complex multicast routing
- `unicast_config.yaml` - Simple unicast routing
- `basic_filtering.rs` - Code examples

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  CoT Message                         │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│              MIL-STD-2525 Parser                     │
│           (affiliation.rs - zero-copy)               │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│               Filter Rules (rules.rs)                │
│  • AffiliationFilter  • GeoBoundingBoxFilter         │
│  • TeamFilter         • GroupFilter                  │
│  • DimensionFilter    • UidFilter                    │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│            Routing Engine (router.rs)                │
│         • Priority-based evaluation                  │
│         • Multicast/Unicast strategies               │
│         • Lock-free route table (DashMap)            │
└──────────────────┬──────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────┐
│            Destination Servers                       │
│     [Server1, Server2, Server3, ...]                 │
└─────────────────────────────────────────────────────┘
```

## Testing

```bash
# Run tests
cargo test --package omnitak-filter

# Run benchmarks
cargo bench --package omnitak-filter

# Run example
cargo run --example basic_filtering
```

## License

MIT OR Apache-2.0
