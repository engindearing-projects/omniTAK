# CoT Message Handling - Robustness Notes

## Context

Based on review of the gyb_detect drone detection code, we need to ensure omniTAK handles CoT messages with military-grade robustness:

**Priority: Show data > Perfect data**

## Current Drone Detection System

The ESP32-based drone detector:
- Scans WiFi (beacon/NAN) and Bluetooth for RemoteID broadcasts
- Supports ODID (OpenDroneID) and French formats
- Sends JSON over Bluetooth Serial
- Handles partial data (some fields may be missing)

## Required Robustness Features

### 1. Accept Partial CoT Messages

**Good Example:**
```xml
<event uid="drone-123" type="a-u-A" ...>
  <point lat="37.77" lon="-122.41" hae="100" ce="9999999" le="9999999"/>
  <!-- Missing detail section - that's OK! -->
</event>
```

**Should work even if:**
- No callsign provided → use UID or "Unknown"
- No altitude → use 0 or last known
- No speed/heading → use 0 or omit
- Malformed XML → try to extract what we can

### 2. Provide Sensible Defaults

```rust
// Instead of failing on missing fields:
let callsign = detail.contact
    .as_ref()
    .map(|c| c.callsign.as_str())
    .unwrap_or_else(|| event.uid.as_str()); // Fallback to UID

let altitude = point.hae
    .or_else(|| Some(0.0)) // Default to sea level if unknown
    .unwrap();
```

### 3. Validation Levels

**Level 1: Critical (must have)**
- uid
- lat/lon (even if inaccurate)
- timestamp

**Level 2: Important (should have)**
- type/affiliation
- altitude
- callsign

**Level 3: Nice to have**
- speed/heading
- operator location
- detailed classifications

**Action:**
- Missing Level 1 → Warn and try to synthesize
- Missing Level 2 → Warn and use defaults
- Missing Level 3 → Silent default

### 4. Error Handling Philosophy

```rust
// BAD: Fail completely
if detail.contact.is_none() {
    return Err("No contact information");
}

// GOOD: Show what we have
let callsign = detail.contact
    .as_ref()
    .map(|c| c.callsign.as_str())
    .unwrap_or("UNKNOWN");
warn!("No contact info for {}, using default callsign", uid);
```

### 5. Logging Strategy

For military ops, operators need to know what's missing:

```
[WARN] CoT message drone-123: Missing altitude, defaulting to 0m
[WARN] CoT message drone-123: Missing callsign, using UID
[INFO] CoT message drone-123: Displayed with 8/12 fields
```

## Implementation Checklist

- [ ] POST /api/v1/cot/send accepts partial messages
- [ ] Parser provides defaults for missing fields
- [ ] Validation warns but doesn't fail on non-critical fields
- [ ] Log missing fields at WARN level
- [ ] Track data quality metrics (% complete)
- [ ] Display messages even if only lat/lon/uid present
- [ ] Handle malformed XML gracefully (try multiple parsers)
- [ ] Support both XML and JSON input (for drone detector)

## Real-World Scenario

**Drone detected via WiFi beacon:**
- Has: MAC address, lat/lon, altitude
- Missing: Operator ID, speed, heading

**omniTAK should:**
1. Accept the message
2. Create CoT with:
   - uid = MAC address
   - point = lat/lon/alt from data
   - callsign = "DRONE-{last 4 of MAC}"
   - type = "a-u-A" (unknown air)
   - speed/heading = 0 (with warning log)
3. Forward to TAK clients
4. Log: "Drone DRONE-AB12: Partial data (5/10 fields), displayed"

**Operator sees:** A contact on the map, even if incomplete. Better than nothing!

## Testing

Test with intentionally broken messages:
- Missing required fields
- Malformed XML
- Out-of-range values (lat > 90, etc.)
- Empty callsigns
- Negative altitudes

Goal: Still show something useful.
