# Quick Demo Guide - Claude Creating TAK Polygons

## 🎬 Demo Script (2-3 minutes)

### Setup (Before Recording)
```bash
# Terminal 1: Start omniTAK server
cd /Users/iesouskurios/omniTAK
target/release/omnitak --config config.yaml
# Wait for "✅ Successfully connected to TAK server: local-ots"
```

### Demo Recording

**[Show Terminal]**

```bash
# Navigate to Claude interface
cd /Users/iesouskurios/omniTAK/claude-interface

# Run the demo
python3 demo_polygon.py
```

**[Narration while script runs]:**

"I'm using Claude AI to create a TAK polygon through natural language.

The Python script:
1. Uses Claude's geometry tools to generate a CoT XML message
2. Authenticates with the omniTAK API
3. Sends the polygon to the server
4. Which forwards it to my OpenTAKServer
5. And displays on all connected TAK clients"

**[Show output - successful creation]:**

✅ SUCCESS! Polygon sent!

**[Switch to TAK Map]**

"And here's the polygon appearing on my TAK map in real-time!"

---

## 📋 One-Liners for Terminal

### Start Server:
```bash
cd /Users/iesouskurios/omniTAK && target/release/omnitak --config config.yaml
```

### Create Polygon:
```bash
cd /Users/iesouskurios/omniTAK/claude-interface && python3 demo_polygon.py
```

### Check Server Status:
```bash
curl http://127.0.0.1:9443/api/v1/status | python3 -m json.tool
```

---

## 🎯 What to Show in Video

1. **Terminal 1**: omniTAK server running, showing connection to OTS
2. **Terminal 2**: Run demo_polygon.py, show success message
3. **TAK Client**: Show the polygon appearing on the map
4. **Explain**: This is Claude AI generating military-grade TAK geometries

---

## 🎨 Custom Polygon Commands

Want to create different polygons? Edit `demo_polygon.py`:

```python
# Different coordinates:
vertices = [
    LatLon(34.0522, -118.2437),  # Los Angeles
    LatLon(34.0, -118.0),
    LatLon(33.5, -118.0),
]

# Different colors:
cot_xml = builder.create_polygon_event(
    polygon,
    "My Custom Area",
    "red"  # or "blue", "green", "yellow"
)
```

---

## 🚀 Also Available (For Extended Demo)

### Create a Circle:
```python
from claude_tools.tak_geometry import CotMessageBuilder, Circle, LatLon

builder = CotMessageBuilder()
circle = Circle(center=LatLon(34.0522, -118.2437), radius_meters=5000)
cot_xml = builder.create_circle_event(circle, "Exclusion Zone", "red")
```

### Create a Route:
```python
from claude_tools.tak_geometry import CotMessageBuilder, Route, LatLon

builder = CotMessageBuilder()
route = Route(waypoints=[
    (LatLon(34.0, -118.0), "Start"),
    (LatLon(34.1, -118.1), "Checkpoint"),
    (LatLon(34.2, -118.2), "End"),
])
route_msgs = builder.create_route_event(route, "Patrol Route", "yellow")
```

---

## 💡 Key Talking Points

1. **Natural Language → TAK Geometry**: Claude interprets commands and generates precise military CoT messages
2. **Real-Time**: Instant propagation to all TAK clients
3. **Production Ready**: Full API, authentication, audit logging
4. **Extensible**: Supports circles, polygons, routes, markers
5. **Open Source**: Rust-based, memory-safe, high-performance

---

## 🎥 Suggested Video Flow

1. **Intro** (0:00-0:15)
   - "Building a natural language interface for TAK using Claude AI"

2. **Show Architecture** (0:15-0:30)
   - Quick diagram or terminal showing components

3. **Live Demo** (0:30-2:00)
   - Start server
   - Run Python script
   - Show polygon on map

4. **Show Code** (2:00-2:30)
   - Brief look at the Python geometry builder
   - Show the CoT XML generated

5. **Wrap Up** (2:30-3:00)
   - "Fully working, ready for conversational AI integration"
   - "Check the repo for full details"

---

## ✨ Server Running?

Check if omniTAK is running:
```bash
ps aux | grep omnitak
curl http://127.0.0.1:9443/health
```

---

## 🎯 The Magic

**Before**: Manually create XML, calculate coordinates, format CoT messages

**After**: "Claude, create a 5km exclusion zone around LAX"

**Future**: Full conversational interface for all TAK operations!
