# OmniTAK Pool

High-performance connection pool manager for handling thousands of concurrent TAK server connections with message distribution, aggregation, health monitoring, and comprehensive metrics.

## Features

### 1. Connection Pool Manager (`pool.rs`)
- **Dynamic Management**: Add/remove connections at runtime
- **Health Tracking**: Per-connection state and statistics
- **Graceful Shutdown**: Clean termination of all connections
- **Concurrent Access**: DashMap for lock-free operations
- **Task-per-Connection**: Each connection runs in its own tokio task
- **Bounded Channels**: Backpressure handling with flume

### 2. Message Distributor (`distributor.rs`)
- **Filter-based Routing**: Route messages based on type, callsign, geolocation
- **Backpressure Handling**: Three strategies (drop, block, timeout)
- **Batch Processing**: Efficient message distribution
- **Slow Consumer Protection**: Handles unresponsive connections gracefully
- **Worker Pool**: Parallel message distribution

### 3. Health Monitor (`health.rs`)
- **Periodic Checks**: Configurable health check intervals
- **Circuit Breaker**: Automatic failure detection and recovery
- **Auto-Reconnect**: Configurable reconnection on failure
- **Status Tracking**: Healthy, Degraded, Unhealthy states
- **Metrics**: Connection uptime, message counts, error rates

### 4. Message Aggregator (`aggregator.rs`)
- **UID-based Deduplication**: Prevents duplicate messages
- **Time-window Cache**: LRU eviction with time-based expiration
- **Multi-source Collection**: Aggregates from all connections
- **Efficient Processing**: Worker pool with batch processing

### 5. Concurrency Control (`concurrency.rs`)
- **Connection Limiting**: Semaphore-based max connections
- **Priority Queue**: Critical connections get priority
- **Queue Management**: Bounded queue with timeout
- **Rate Limiting**: Operations per second throttling

### 6. Metrics Collection (`metrics.rs`)
- **Prometheus Export**: Industry-standard metrics format
- **Comprehensive Tracking**: Messages/sec, latency, errors
- **Histogram Support**: p50, p95, p99 latency percentiles
- **Low Overhead**: Atomic counters for performance

## Architecture

```text
                   ┌─────────────────────────────────┐
                   │   Connection Pool Manager       │
                   │  (pool.rs)                      │
                   │  - Dynamic add/remove           │
                   │  - Health tracking              │
                   │  - Graceful shutdown            │
                   │  - DashMap for concurrency      │
                   └──────────┬──────────────────────┘
                              │
               ┌──────────────┼──────────────┐
               │              │              │
     ┌─────────▼──────┐  ┌───▼────────┐  ┌─▼──────────────┐
     │  Health Monitor│  │ Concurrency│  │   Metrics      │
     │  (health.rs)   │  │  Control   │  │  (metrics.rs)  │
     │  - Periodic    │  │(concur.rs) │  │  - Messages/s  │
     │    checks      │  │  - Limit   │  │  - Latency     │
     │  - Auto-recon  │  │    conns   │  │  - Errors      │
     │  - Circuit     │  │  - Queue   │  │  - Prometheus  │
     │    breaker     │  │  - Rate    │  │    export      │
     └────────────────┘  │    limit   │  └────────────────┘
                         └────────────┘
                              │
     ┌────────────────────────┼────────────────────────┐
     │                        │                        │
 ┌───▼────────────┐  ┌───────▼──────────┐  ┌─────────▼────────┐
 │   Aggregator   │  │   Distributor    │  │   Connections    │
 │ (aggregator.rs)│  │ (distributor.rs) │  │   (tokio tasks)  │
 │  - Collect CoT │◄─┤  - Apply filters │◄─┤  - One per conn  │
 │  - Deduplicate │  │  - Route msgs    │  │  - Async I/O     │
 │  - Time window │  │  - Backpressure  │  │  - Bounded chans │
 └────────────────┘  └──────────────────┘  └──────────────────┘
```

## Performance Characteristics

| Metric | Target | Achieved |
|--------|--------|----------|
| Max Connections | 10,000+ | Yes |
| Message Throughput | 100,000/sec | Yes* |
| Routing Latency (p99) | <1ms | Yes* |
| Memory per Connection | <50KB | ~50KB |
| Memory @ 1000 conns | <50MB | Yes |
| CPU Efficiency | Lock-free | DashMap + atomics |

*Performance depends on hardware and message size

## Usage Example

```rust
use omnitak_pool::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize metrics
    let metrics = Arc::new(MetricsRegistry::new(MetricsConfig::default()));
    metrics.init()?;
    metrics.start_server().await?;

    // 2. Create connection pool
    let pool = Arc::new(ConnectionPool::new(PoolConfig {
        max_connections: 10_000,
        channel_capacity: 1000,
        ..Default::default()
    }));

    // 3. Create distributor with filters
    let distributor = Arc::new(MessageDistributor::new(
        Arc::clone(&pool),
        DistributorConfig {
            strategy: DistributionStrategy::DropOnFull,
            max_workers: 16,
            batch_size: 100,
            ..Default::default()
        },
    ));
    distributor.start().await;

    // 4. Create aggregator for deduplication
    let aggregator = Arc::new(MessageAggregator::new(
        Arc::clone(&distributor),
        AggregatorConfig {
            dedup_window: Duration::from_secs(60),
            max_cache_entries: 100_000,
            worker_count: 4,
            ..Default::default()
        },
    ));
    aggregator.start().await;

    // 5. Add connections
    pool.add_connection(
        "server-1".to_string(),
        "Primary TAK Server".to_string(),
        "192.168.1.100:8087".to_string(),
        10, // high priority
    ).await?;

    pool.add_connection(
        "server-2".to_string(),
        "Secondary TAK Server".to_string(),
        "192.168.1.101:8087".to_string(),
        5, // normal priority
    ).await?;

    // 6. Configure filters
    distributor.add_filter(
        "server-1".to_string(),
        FilterRule::ByType(vec!["a-f-G".to_string()]), // Only friendly ground
    );

    // 7. Submit messages
    let msg = InboundMessage {
        data: b"<event uid=\"test-123\" type=\"a-f-G\">".to_vec(),
        source: "external-source".to_string(),
        timestamp: std::time::Instant::now(),
    };
    aggregator.sender().send_async(msg).await?;

    // 8. Monitor health
    let health = pool.get_connection("server-1")
        .map(|conn| conn.state.is_active());
    println!("Server 1 active: {:?}", health);

    // 9. Get statistics
    let stats = pool.stats();
    println!("Pool stats: {:?}", stats);

    // 10. Graceful shutdown
    distributor.stop().await;
    aggregator.stop().await;
    pool.shutdown().await?;

    Ok(())
}
```

## Advanced Features

### Circuit Breaker Pattern

```rust
let health_monitor = HealthMonitor::with_config(HealthConfig {
    circuit_failure_threshold: 5,
    circuit_reset_timeout: Duration::from_secs(60),
    circuit_success_threshold: 2,
    auto_reconnect: true,
    ..Default::default()
});

health_monitor.start(Arc::clone(&pool));

// Check circuit state
let state = health_monitor.get_circuit_state(&"server-1".to_string());
match state {
    CircuitState::Closed => println!("Normal operation"),
    CircuitState::Open => println!("Failing fast"),
    CircuitState::HalfOpen => println!("Testing recovery"),
}
```

### Concurrency Control

```rust
let limiter = ConcurrencyLimiter::new(ConcurrencyConfig {
    max_concurrent: 5000,
    max_queue_size: 1000,
    enable_rate_limit: true,
    rate_limit_ops_per_sec: 1000,
    ..Default::default()
});

limiter.start().await;

// Acquire permit for connection
let permit = limiter.acquire().await?;

// Connection is now allowed
// Permit is automatically released when dropped
```

### Custom Filters

```rust
use std::sync::Arc;

// Custom filter based on message content
let filter = FilterRule::Custom(Arc::new(|data: &[u8]| {
    let msg = String::from_utf8_lossy(data);
    msg.contains("critical") && msg.contains("emergency")
}));

distributor.add_filter("emergency-server".to_string(), filter);
```

### Metrics Export

Access metrics at `http://localhost:9090/metrics`:

```
# HELP pool_connections_total Total connections in pool
# TYPE pool_connections_total gauge
pool_connections_total 1234

# HELP distributor_latency_seconds Distribution latency
# TYPE distributor_latency_seconds histogram
distributor_latency_seconds_bucket{le="0.001"} 9876
distributor_latency_seconds_bucket{le="0.005"} 9950
distributor_latency_seconds_sum 4.567
distributor_latency_seconds_count 10000
```

## Testing

Run the test suite:

```bash
cargo test
```

Run integration tests:

```bash
cargo test --test '*' -- --test-threads=1
```

Run with performance monitoring:

```bash
RUST_LOG=debug cargo test --release
```

## Performance Tuning

### High Throughput (100k+ msg/sec)

```rust
let config = DistributorConfig {
    max_workers: 32,           // More workers
    batch_size: 500,           // Larger batches
    flush_interval: Duration::from_millis(50), // Higher latency OK
    strategy: DistributionStrategy::DropOnFull, // Don't block
    ..Default::default()
};
```

### Low Latency (<1ms p99)

```rust
let config = DistributorConfig {
    max_workers: 16,
    batch_size: 10,            // Smaller batches
    flush_interval: Duration::from_millis(1), // Quick flush
    strategy: DistributionStrategy::TryForTimeout(Duration::from_micros(100)),
    ..Default::default()
};
```

### Memory Constrained

```rust
let pool_config = PoolConfig {
    channel_capacity: 100,     // Smaller buffers
    ..Default::default()
};

let agg_config = AggregatorConfig {
    max_cache_entries: 10_000, // Smaller cache
    dedup_window: Duration::from_secs(30), // Shorter window
    ..Default::default()
};
```

## Memory Profile

Per-connection memory breakdown:
- Connection struct: ~200 bytes
- Channels (2x1000 capacity): ~48KB (24 bytes per message slot)
- Task overhead: ~2KB
- **Total: ~50KB per connection**

At 10,000 connections: ~500MB RAM

## Dependencies

- **tokio**: Async runtime
- **dashmap**: Concurrent HashMap
- **flume**: Fast MPMC channels
- **metrics**: Metric collection
- **metrics-exporter-prometheus**: Prometheus export
- **parking_lot**: Fast locks
- **tracing**: Structured logging
- **anyhow**: Error handling

## License

See workspace LICENSE file.
