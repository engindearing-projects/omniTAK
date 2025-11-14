# Example Filter Plugin - Usage Examples

This document shows example inputs and outputs for the example filter plugin.

## Basic Functionality

### Example 1: Clean Message (No Keywords)

**Input:**
```xml
<event version="2.0" uid="ANDROID-123456" type="a-f-G" time="2025-11-14T10:00:00Z" start="2025-11-14T10:00:00Z" stale="2025-11-14T10:05:00Z" how="m-g">
  <point lat="40.7128" lon="-74.0060" hae="10.0" ce="5.0" le="2.0"/>
  <detail>
    <contact callsign="TEAM-1"/>
  </detail>
</event>
```

**Output:**
```xml
<event version="2.0" uid="ANDROID-123456" type="a-f-G" time="2025-11-14T10:00:00Z" start="2025-11-14T10:00:00Z" stale="2025-11-14T10:05:00Z" how="m-g">
  <point lat="40.7128" lon="-74.0060" hae="10.0" ce="5.0" le="2.0"/>
  <detail>
    <contact callsign="TEAM-1"/>
  </detail>
</event>
```

**Log Output:**
```
ExampleFilterPlugin: Processing message...
ExampleFilterPlugin: Message passed filter
```

---

### Example 2: Hostile Keyword Detected

**Input:**
```xml
<event version="2.0" uid="ANDROID-789012" type="a-h-G" time="2025-11-14T10:05:00Z" start="2025-11-14T10:05:00Z" stale="2025-11-14T10:10:00Z" how="m-g">
  <point lat="40.7500" lon="-73.9900" hae="15.0" ce="10.0" le="3.0"/>
  <detail>
    <contact callsign="SCOUT-2"/>
    <remarks>Hostile contact detected at grid reference</remarks>
  </detail>
</event>
```

**Output:**
```xml
<event hostile_detected="true" keywords="hostile" version="2.0" uid="ANDROID-789012" type="a-h-G" time="2025-11-14T10:05:00Z" start="2025-11-14T10:05:00Z" stale="2025-11-14T10:10:00Z" how="m-g">
  <point lat="40.7500" lon="-73.9900" hae="15.0" ce="10.0" le="3.0"/>
  <detail>
    <contact callsign="SCOUT-2"/>
    <remarks>Hostile contact detected at grid reference</remarks>
  </detail>
</event>
```

**Log Output:**
```
ExampleFilterPlugin: Processing message...
ExampleFilterPlugin: Hostile keywords detected
  keywords: hostile
  action: tagged
```

---

### Example 3: Multiple Hostile Keywords

**Input:**
```xml
<event version="2.0" uid="ANDROID-345678" type="a-h-A" time="2025-11-14T10:10:00Z" start="2025-11-14T10:10:00Z" stale="2025-11-14T10:15:00Z" how="m-g">
  <point lat="40.7800" lon="-73.9600" hae="20.0" ce="8.0" le="2.5"/>
  <detail>
    <contact callsign="RECON-3"/>
    <remarks>Enemy threat detected - possible attack imminent</remarks>
  </detail>
</event>
```

**Output:**
```xml
<event hostile_detected="true" keywords="enemy, threat, attack" version="2.0" uid="ANDROID-345678" type="a-h-A" time="2025-11-14T10:10:00Z" start="2025-11-14T10:10:00Z" stale="2025-11-14T10:15:00Z" how="m-g">
  <point lat="40.7800" lon="-73.9600" hae="20.0" ce="8.0" le="2.5"/>
  <detail>
    <contact callsign="RECON-3"/>
    <remarks>Enemy threat detected - possible attack imminent</remarks>
  </detail>
</event>
```

**Log Output:**
```
ExampleFilterPlugin: Processing message...
ExampleFilterPlugin: Hostile keywords detected
  keywords: enemy, threat, attack
  action: tagged
```

---

### Example 4: Case-Insensitive Detection

**Input:**
```xml
<event version="2.0" uid="TEST-001" type="a-f-G" how="m-g">
  <detail>
    <remarks>DANGER zone ahead - proceed with caution</remarks>
  </detail>
</event>
```

**Output:**
```xml
<event hostile_detected="true" keywords="danger" version="2.0" uid="TEST-001" type="a-f-G" how="m-g">
  <detail>
    <remarks>DANGER zone ahead - proceed with caution</remarks>
  </detail>
</event>
```

**Note:** Keywords are detected case-insensitively (DANGER matches "danger").

---

### Example 5: Message Without Event Tag

**Input:**
```xml
<?xml version="1.0"?>
<cot>
  <hostile_unit id="123">Enemy position</hostile_unit>
</cot>
```

**Output:**
```xml
<!-- HOSTILE CONTENT DETECTED: hostile, enemy -->
<?xml version="1.0"?>
<cot>
  <hostile_unit id="123">Enemy position</hostile_unit>
</cot>
```

**Note:** When no `<event>` tag is found, a comment is prepended instead.

---

### Example 6: Empty Message (Error Case)

**Input:**
```
```

**Output:**
```
Error: "Message is empty"
```

**Log Output:**
```
ExampleFilterPlugin: Processing message...
ExampleFilterPlugin: Empty message received
```

---

## Keyword Reference

The plugin currently detects these keywords (case-insensitive):

- `hostile`
- `enemy`
- `threat`
- `attack`
- `danger`

## Customization Ideas

You can modify the plugin to:

1. **Add more keywords:**
   ```rust
   let hostile_keywords = vec![
       "hostile", "enemy", "threat", "attack", "danger",
       "suspicious", "unknown", "unidentified"
   ];
   ```

2. **Implement severity levels:**
   ```rust
   let critical_keywords = vec!["attack", "danger"];
   let warning_keywords = vec!["hostile", "enemy", "threat"];
   ```

3. **Block instead of tag:**
   ```rust
   if !found_keywords.is_empty() {
       return Err(format!("Message blocked: contains {}", keywords_str));
   }
   ```

4. **Add geofencing:**
   ```rust
   // Parse lat/lon from <point> tag
   // Check if within restricted area
   // Tag or block accordingly
   ```

5. **Content sanitization:**
   ```rust
   // Remove or redact sensitive information
   let sanitized = cot_xml.replace("CLASSIFIED", "[REDACTED]");
   ```

## Integration with omniTAK

When integrated with the omniTAK host, the plugin will:

1. Receive CoT messages before they're processed
2. Filter each message through `filter-message()`
3. Pass the result (modified or original) to the next stage
4. Log all actions through the host logging system
5. Return errors for invalid messages

The host can query metadata using:
- `get-name()` → "Example Filter Plugin"
- `get-version()` → "0.1.0"
- `get-description()` → "A simple example filter that detects hostile keywords in CoT messages"
