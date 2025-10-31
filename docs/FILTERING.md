# OmniTAK Filtering and Data Flow Guide

This guide explains how to configure message filtering and view data flow in OmniTAK. Filtering allows you to control which CoT (Cursor on Target) messages are processed and routed based on various criteria.

## Table of Contents

- [Overview](#overview)
- [Filter Modes](#filter-modes)
- [Filter Types](#filter-types)
- [Configuration Examples](#configuration-examples)
- [Viewing Data Flow](#viewing-data-flow)
- [Performance Tuning](#performance-tuning)
- [Advanced Filtering](#advanced-filtering)

## Overview

OmniTAK provides a powerful filtering system that can:

- **Filter by affiliation** - Friendly, hostile, neutral, unknown
- **Filter by geography** - Bounding box, radius, specific areas
- **Filter by team/group** - Unit designations
- **Filter by CoT type** - Event types (MIL-STD-2525 codes)
- **Filter by UID** - Specific entity identifiers
- **Route messages** - Send different messages to different destinations

Filters are evaluated in real-time with <100ns latency per check, using lock-free data structures for maximum performance.

## Filter Modes

OmniTAK supports two filtering modes:

### Whitelist Mode (Allow List)

Only messages that match filter rules are accepted. All other messages are rejected.

```yaml
filters:
  mode: whitelist
  rules:
    - id: friendly-only
      type: affiliation
      allow: [friend, assumedfriend]
```

**Use case:** When you only want specific types of messages (e.g., only friendly forces).

### Blacklist Mode (Deny List)

All messages are accepted except those matching filter rules.

```yaml
filters:
  mode: blacklist
  rules:
    - id: block-hostiles
      type: affiliation
      allow: [hostile, suspect]
```

**Use case:** When you want most messages but need to block specific types.

### Default Action

When no rules match, the default action is determined by the mode:
- **Whitelist:** Reject (only explicit matches are allowed)
- **Blacklist:** Accept (only explicit matches are rejected)

## Filter Types

### 1. Affiliation Filter

Filter based on MIL-STD-2525 affiliation codes parsed from CoT event types.

**Affiliations:**
- `friend` - Friendly forces (blue)
- `assumedfriend` - Assumed friend
- `neutral` - Neutral forces (green)
- `hostile` - Hostile forces (red)
- `suspect` - Suspect
- `unknown` - Unknown affiliation

**Configuration:**
```yaml
filters:
  mode: whitelist
  rules:
    - id: friendly-forces
      type: affiliation
      allow: [friend, assumedfriend]
      destinations: [tak-server-1]

    - id: hostile-tracking
      type: affiliation
      allow: [hostile, suspect]
      destinations: [tak-server-2]
```

**CoT Type Examples:**
- `a-f-G-E-V-M` - Friendly ground equipment, vehicle, military
- `a-h-G-U-C-I` - Hostile ground unit, combat, infantry
- `a-n-G` - Neutral ground unit
- `a-u-G` - Unknown ground unit

### 2. Geographic Filter

Filter based on geographic location (latitude/longitude).

**Configuration:**
```yaml
filters:
  mode: whitelist
  rules:
    - id: operation-area
      type: geographic
      bounds:
        min_lat: 34.0522    # Southern boundary
        max_lat: 34.1522    # Northern boundary
        min_lon: -118.2437  # Western boundary
        max_lon: -118.1437  # Eastern boundary
      destinations: [tak-server-1]
```

**Use cases:**
- Limit messages to a specific area of operations
- Create multiple zones for different units
- Filter out messages outside your region of interest

### 3. Team Filter

Filter based on team or group name.

**Configuration:**
```yaml
filters:
  mode: whitelist
  rules:
    - id: team-alpha
      type: team
      teams: ["Alpha", "Bravo", "Charlie"]
      destinations: [tak-server-1]

    - id: team-delta
      type: team
      teams: ["Delta", "Echo"]
      destinations: [tak-server-2]
```

### 4. Group Filter

Filter based on unit group designation.

**Configuration:**
```yaml
filters:
  mode: whitelist
  rules:
    - id: first-battalion
      type: group
      groups: ["1st Battalion", "1-501"]
      destinations: [tak-server-1]
```

### 5. UID Filter

Filter based on specific entity unique identifiers.

**Configuration:**
```yaml
filters:
  mode: whitelist
  rules:
    - id: vip-tracking
      type: uid
      uids:
        - "ANDROID-123456789"
        - "ANDROID-987654321"
      destinations: [tak-server-vip]
```

### 6. Field-Based Filter

Filter based on any CoT event field with custom operators.

**Configuration:**
```yaml
filters:
  mode: whitelist
  rules:
    - id: specific-type
      name: "Ground vehicles only"
      enabled: true
      field: "type"
      operator: starts_with
      value: "a-f-G-E-V"
      action: accept
```

**Operators:**
- `equals` - Exact match
- `not_equals` - Not equal
- `contains` - Substring match
- `not_contains` - Does not contain
- `starts_with` - Starts with prefix
- `ends_with` - Ends with suffix
- `regex` - Regular expression match

## Configuration Examples

### Example 1: Tactical Operations Center

Route friendly forces to primary server, threats to intelligence server:

```yaml
filters:
  mode: whitelist
  rules:
    # Friendly forces to operations server
    - id: friendly-ops
      type: affiliation
      allow: [friend, assumedfriend]
      destinations: [tak-ops-server]

    # Hostile/suspect to intelligence server
    - id: threat-intel
      type: affiliation
      allow: [hostile, suspect]
      destinations: [tak-intel-server]

    # Geographic area of interest
    - id: aoi-filter
      type: geographic
      bounds:
        min_lat: 35.0
        max_lat: 36.0
        min_lon: -120.0
        max_lon: -119.0
      destinations: [tak-ops-server, tak-intel-server]
```

### Example 2: Multi-Team Coordination

Separate data streams for different teams:

```yaml
filters:
  mode: whitelist
  rules:
    # Alpha team
    - id: team-alpha
      type: team
      teams: ["Alpha"]
      destinations: [tak-alpha-server]

    # Bravo team
    - id: team-bravo
      type: team
      teams: ["Bravo"]
      destinations: [tak-bravo-server]

    # Shared intelligence
    - id: shared-intel
      type: affiliation
      allow: [hostile, suspect]
      destinations: [tak-alpha-server, tak-bravo-server]
```

### Example 3: Air-Ground Coordination

Separate air and ground tracks:

```yaml
filters:
  mode: whitelist
  rules:
    # Air tracks
    - id: air-tracks
      name: "Air units"
      enabled: true
      field: "type"
      operator: contains
      value: "-A-"  # Air dimension
      action: accept
      destinations: [tak-air-server]

    # Ground tracks
    - id: ground-tracks
      name: "Ground units"
      enabled: true
      field: "type"
      operator: contains
      value: "-G-"  # Ground dimension
      action: accept
      destinations: [tak-ground-server]

    # Maritime tracks
    - id: maritime-tracks
      name: "Maritime units"
      enabled: true
      field: "type"
      operator: contains
      value: "-S-"  # Sea surface dimension
      action: accept
      destinations: [tak-maritime-server]
```

## Viewing Data Flow

### 1. Web UI

Access the web interface at `http://localhost:9443` (or your configured API address).

**Features:**
- Real-time CoT message feed
- Connection status indicators
- Message rate statistics
- Filter match visualization
- Interactive map view

### 2. WebSocket Streaming

Connect to the WebSocket endpoint for real-time message streaming:

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:9443/ws');

// Handle connection
ws.onopen = () => {
    console.log('Connected to OmniTAK');
};

// Receive messages
ws.onmessage = (event) => {
    const data = JSON.parse(event.data);

    switch(data.type) {
        case 'cot_message':
            console.log('CoT:', data.payload);
            // Process CoT message
            displayOnMap(data.payload);
            break;

        case 'connection_status':
            console.log('Connection:', data.payload);
            updateConnectionStatus(data.payload);
            break;

        case 'filter_result':
            console.log('Filter:', data.payload);
            updateFilterStats(data.payload);
            break;
    }
};

// Handle errors
ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};

// Reconnect on close
ws.onclose = () => {
    console.log('Disconnected, reconnecting...');
    setTimeout(connectWebSocket, 1000);
};
```

### 3. REST API

Query historical messages and statistics:

```bash
# Get all connections
curl http://localhost:9443/api/v1/connections

# Example response:
# {
#   "connections": [
#     {
#       "id": "uuid-here",
#       "server_name": "tak-server-1",
#       "status": "connected",
#       "messages_received": 1234,
#       "bytes_received": 567890,
#       "uptime_seconds": 3600
#     }
#   ]
# }

# Get recent messages (last 100)
curl http://localhost:9443/api/v1/messages?limit=100

# Get messages with filters
curl "http://localhost:9443/api/v1/messages?affiliation=friend&limit=50"

# Get filter statistics
curl http://localhost:9443/api/v1/filters/stats

# Get Prometheus metrics
curl http://localhost:9443/api/v1/metrics
```

### 4. Command-Line Monitoring

View logs for message flow:

```bash
# Run with info logging
cargo run -- --config config/config.yaml

# Run with debug logging for detailed filter info
RUST_LOG=debug cargo run -- --config config/config.yaml

# Run with JSON logging for parsing
cargo run -- --config config/config.yaml --log-format json | jq
```

### 5. Prometheus Metrics

OmniTAK exposes Prometheus-compatible metrics:

**Key Metrics:**

```
# Messages received per server
omnitak_messages_received_total{server="tak-server-1"} 1234

# Messages filtered
omnitak_messages_filtered_total{filter="friendly-only",action="accept"} 890
omnitak_messages_filtered_total{filter="friendly-only",action="reject"} 344

# Connection status
omnitak_connection_status{server="tak-server-1"} 1  # 1=connected, 0=disconnected

# Message processing latency
omnitak_message_processing_duration_seconds{quantile="0.99"} 0.001

# Filter evaluation time
omnitak_filter_evaluation_duration_seconds{filter="friendly-only"} 0.0001
```

**Grafana Dashboard:**

Create a Grafana dashboard with these queries:

```promql
# Message rate by server
rate(omnitak_messages_received_total[5m])

# Filter acceptance rate
rate(omnitak_messages_filtered_total{action="accept"}[5m]) /
rate(omnitak_messages_filtered_total[5m])

# P99 latency
histogram_quantile(0.99, omnitak_message_processing_duration_seconds)

# Active connections
count(omnitak_connection_status == 1)
```

## Performance Tuning

### Filter Ordering

Place most selective filters first for better performance:

```yaml
filters:
  mode: whitelist
  rules:
    # Specific UIDs (most selective) - evaluated first
    - id: vip-tracking
      type: uid
      uids: ["ANDROID-123"]
      destinations: [vip-server]

    # Geographic filter (medium selectivity)
    - id: area-of-ops
      type: geographic
      bounds: {...}
      destinations: [ops-server]

    # Affiliation (least selective) - evaluated last
    - id: friendly-all
      type: affiliation
      allow: [friend]
      destinations: [general-server]
```

### Optimize Geographic Filters

Use larger bounding boxes when possible to reduce CPU overhead:

```yaml
# Good - simple rectangular area
geographic:
  bounds:
    min_lat: 34.0
    max_lat: 35.0
    min_lon: -119.0
    max_lon: -118.0

# Avoid - multiple small overlapping boxes
```

### Buffer Sizing

Adjust buffer sizes based on message volume:

```yaml
performance:
  buffer_size: 16384  # Increase for high-volume scenarios
  max_message_size: 2097152  # 2MB for large CoT messages
```

### Connection Pooling

Configure connection pool for optimal throughput:

```yaml
application:
  max_connections: 100  # Adjust based on expected servers
  worker_threads: 8     # Set to number of CPU cores
```

## Advanced Filtering

### Combining Multiple Criteria

Use multiple rules to create complex filtering logic:

```yaml
filters:
  mode: whitelist
  rules:
    # Friendly ground vehicles in operation area
    - id: friendly-ground-aoi
      name: "Friendly ground vehicles in AO"
      enabled: true
      field: "type"
      operator: starts_with
      value: "a-f-G-E-V"
      action: accept
      destinations: [tak-server-1]

    - id: aoi-bounds
      type: geographic
      bounds:
        min_lat: 34.0
        max_lat: 35.0
        min_lon: -119.0
        max_lon: -118.0
      destinations: [tak-server-1]
```

**Note:** In whitelist mode, a message must match ALL applicable rules to pass.

### Regular Expression Filters

Use regex for complex type matching:

```yaml
filters:
  mode: whitelist
  rules:
    - id: specific-equipment
      name: "Specific vehicle types"
      enabled: true
      field: "type"
      operator: regex
      value: "a-f-G-E-V-(M|A|C)"  # Military, Armored, or Cargo vehicles
      action: accept
```

### Dynamic Rule Updates

Update filter rules at runtime via API:

```bash
# Add new filter rule
curl -X POST http://localhost:9443/api/v1/filters \
  -H "Content-Type: application/json" \
  -d '{
    "id": "new-filter",
    "type": "affiliation",
    "allow": ["friend"],
    "destinations": ["tak-server-1"]
  }'

# Enable/disable filter
curl -X PATCH http://localhost:9443/api/v1/filters/friendly-only \
  -H "Content-Type: application/json" \
  -d '{"enabled": false}'

# Delete filter
curl -X DELETE http://localhost:9443/api/v1/filters/old-filter
```

### Filter Priority

Assign priorities to control evaluation order:

```yaml
filters:
  mode: whitelist
  rules:
    - id: vip-priority
      type: uid
      uids: ["VIP-1"]
      priority: 100  # Evaluated first
      destinations: [vip-server]

    - id: general-traffic
      type: affiliation
      allow: [friend]
      priority: 50  # Evaluated after VIP filter
      destinations: [general-server]
```

Higher priority values are evaluated first.

## Filter Testing

### Test Configuration

Validate your filter configuration before deploying:

```bash
# Validate config
cargo run -- --config config/config.yaml --validate

# Dry-run mode (don't actually connect)
cargo run -- --config config/config.yaml --dry-run

# Test with sample CoT messages
cargo run -- --config config/config.yaml --test-file samples/cot-messages.xml
```

### Filter Simulation

Test filters against sample data:

```bash
# Create test CoT message
cat > test-message.xml <<EOF
<?xml version="1.0"?>
<event version="2.0" uid="TEST-1" type="a-f-G-E-V-M" time="2024-01-01T12:00:00Z" start="2024-01-01T12:00:00Z" stale="2024-01-01T12:05:00Z">
  <point lat="34.0522" lon="-118.2437" hae="100" ce="10" le="5"/>
</event>
EOF

# Test filter
cargo run -- --config config/config.yaml --test-message test-message.xml
```

## Best Practices

1. **Start with whitelist mode** for security - explicitly define what's allowed
2. **Use specific filters** - more specific filters are faster and more secure
3. **Monitor filter performance** - check metrics for slow filters
4. **Test filters thoroughly** - validate with sample data before production
5. **Document filter rules** - add descriptions to explain purpose
6. **Use multiple destinations** - route important messages to multiple servers
7. **Enable audit logging** - track which messages were filtered and why

## Troubleshooting

### Messages Not Passing Filters

**Check filter mode:**
```yaml
filters:
  mode: whitelist  # Are you in the right mode?
```

**Enable debug logging:**
```yaml
logging:
  level: "debug"
```

Check logs for filter evaluation details.

**Verify message format:**
- Ensure CoT messages are properly formatted
- Check that affiliation codes are valid
- Verify coordinates are within expected ranges

### Performance Issues

**Symptoms:**
- High CPU usage
- Message backlog
- Increased latency

**Solutions:**
1. Simplify complex regex filters
2. Increase worker threads
3. Optimize filter ordering
4. Increase buffer sizes
5. Check for slow destinations

## Next Steps

- [ADB Setup Guide](ADB_SETUP.md) - Set up TAK client configuration
- [API Documentation](API.md) - REST API and WebSocket reference
- [Configuration Reference](../config.example.yaml) - Full config options
- [Deployment Guide](DEPLOYMENT.md) - Production deployment

## Support

For questions or issues:
- Check logs with `level: "debug"`
- Review filter statistics via API
- Open an issue on GitHub with configuration (redact sensitive info)
