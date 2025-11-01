# Interactive TAK Object Creator

## 🚀 Quick Start

**Terminal 1 - Start omniTAK Server** (if not already running):
```bash
cd /Users/iesouskurios/omniTAK
target/release/omnitak --config config.yaml
```

**Terminal 2 - Launch Interactive Demo**:
```bash
cd /Users/iesouskurios/omniTAK/claude-interface
python3 interactive_demo.py
```

## 💬 Example Commands

### Create a Polygon
```
🎯 > polygon 34.0,-118.0 34.0,-117.0 33.5,-117.0 MyArea blue
```
Creates a blue polygon with 3 vertices around Los Angeles area.

### Create a Circle (Exclusion Zone)
```
🎯 > circle 34.0522,-118.2437 5 LAX_Zone red
```
Creates a 5km red circle around LAX coordinates.

### Create a Route
```
🎯 > route 34.0,-118.0:Firebase 34.1,-118.1:Checkpoint 34.2,-118.2:Objective PatrolAlpha yellow
```
Creates a yellow patrol route with 3 named waypoints.

## 📍 Command Reference

### Polygon Command
```
polygon <lat,lon> <lat,lon> <lat,lon> [name] [color]
```
- At least 3 coordinate pairs required
- Optional name (defaults to "Interactive Polygon")
- Optional color: red, blue, green, yellow (defaults to blue)

**Example:**
```
polygon 34.0,-118.0 34.0,-117.0 33.5,-117.0 33.5,-118.0 OpArea blue
```

### Circle Command
```
circle <lat,lon> <radius_km> [name] [color]
```
- Center coordinates (lat,lon)
- Radius in kilometers
- Optional name and color (defaults to red)

**Example:**
```
circle 37.7749,-122.4194 10 SFExclusionZone red
```

### Route Command
```
route <lat,lon:waypoint_name> <lat,lon:waypoint_name> ... [route_name] [color]
```
- At least 2 waypoints required
- Each waypoint: lat,lon:name
- Optional route name and color (defaults to yellow)

**Example:**
```
route 33.123,-117.456:Start 33.234,-117.567:Mid 33.345,-117.678:End Patrol yellow
```

## 🎯 Live Demo Tips

### For Video Recording:

1. **Start with a simple polygon:**
   ```
   polygon 34.0,-118.0 34.0,-117.0 33.5,-117.0 TestArea blue
   ```

2. **Add a circle nearby:**
   ```
   circle 34.0,-118.0 3 Exclusion red
   ```

3. **Create a patrol route:**
   ```
   route 34.0,-118.0:Alpha 34.1,-118.1:Bravo PatrolRoute yellow
   ```

4. **Show it appearing on your TAK map in real-time!**

### What You'll See:

```
🎯 OmniTAK Interactive Demo - Claude-Powered TAK Object Creator
================================================================================

✅ Connected and authenticated!

📍 Available Commands:
  polygon <lat,lon> <lat,lon> <lat,lon> [name] [color]
  circle <lat,lon> <radius_km> [name] [color]
  route <lat,lon:name> <lat,lon:name> [route_name] [color]
  help     - Show this help
  quit     - Exit program

🎯 > polygon 34.0,-118.0 34.0,-117.0 33.5,-117.0 MyArea blue

📐 Creating polygon with 3 vertices:
   Point 1: 34.0, -118.0
   Point 2: 34.0, -117.0
   Point 3: 33.5, -117.0
   Name: MyArea
   Color: blue

✅ Polygon 'MyArea' sent successfully! (ID: aa5b17de...)

🎯 >
```

## 🗺️ Finding Your Coordinates

### Quick Coordinate Finder:
- Google Maps: Right-click → "What's here?" → Copy coordinates
- ATAK: Long-press on map → Copy coordinates
- OpenStreetMap: Click location → See coordinates at bottom

### Some Example Areas:
- Los Angeles: `34.0522,-118.2437`
- San Francisco: `37.7749,-122.4194`
- San Diego: `32.7157,-117.1611`
- Your OTS Server area: Use coordinates near `192.168.1.71`

## 🎬 Perfect for Demo Videos!

This interactive mode is perfect for showing:
1. Type a command
2. See the output/confirmation
3. Switch to TAK map
4. Point to the object that just appeared
5. Repeat with different shapes!

## 🔧 Troubleshooting

**"Failed to authenticate":**
- Make sure omniTAK server is running
- Check it's listening on http://127.0.0.1:9443

**"Invalid coordinates":**
- Use decimal format: 34.0,-118.0
- Can use space or comma as separator
- Latitude first, then longitude

**Colors available:**
- red, blue, green, yellow
- Case insensitive

## 🎯 Exit the Program
Type `quit` or `exit`, or press Ctrl+C

---

**Ready to create TAK objects live!** Just type your commands and watch them appear on your map! 🗺️
