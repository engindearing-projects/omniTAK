# Claude Agent SDK + omniTAK Integration

Natural language interface for TAK operations using Claude Code.

## What This Enables

**Talk to your TAK system in plain English:**

```
You: "Create a 5km exclusion zone around 37.7749, -122.4194"
→ Claude creates a red circle on all TAK maps

You: "Draw a patrol route from Firebase Alpha at 33.123,-117.456
     through Checkpoint Bravo to Objective Charlie at 33.345,-117.678"
→ Claude creates a route with waypoints on TAK

You: "Show me all hostile contacts from the last 30 minutes"
→ Claude queries and summarizes TAK data

You: "Place a medical marker at 34.567,-118.123 called MEDEVAC-1"
→ Claude places a casualty collection point icon
```

## Project Status

### ✅ Completed

- **CoT Message Builders** (`claude_tools/tak_geometry.py`)
  - Circle geometry (exclusion zones, range rings)
  - Polygon geometry (areas, boundaries)
  - Route geometry (patrol paths, movement corridors)
  - Marker creation (objectives, threats, resources)
  - Full XML serialization for TAK compatibility

- **Tool Definitions** (`claude_tools/tak_tools.py`)
  - `create_exclusion_zone()` - Circular zones
  - `create_area_polygon()` - Polygon areas
  - `create_patrol_route()` - Routes with waypoints
  - `place_marker()` - Map markers
  - `get_tak_status()` - Connection status
  - `query_tak_data()` - Data queries

- **Integration Plan** (`INTEGRATION_PLAN.md`)
  - Complete 8-week roadmap
  - Architecture diagrams
  - Implementation phases
  - Security considerations

### 🚧 In Progress

- **omniTAK API Client** - Python wrapper for REST API
- **Geometry Support** - Rust backend enhancements needed

### 📋 Pending

- omniTAK backend enhancements:
  - Add geometry types (polygon, circle, line) to `omnitak-cot`
  - Add `POST /api/v1/cot/send` API endpoint
  - Add message injection to connection pool

- Claude SDK integration:
  - Install and configure Claude Agent SDK
  - Connect tools to real omniTAK API
  - Create conversational interface
  - Add WebSocket streaming for real-time queries

## Quick Start (When Complete)

### 1. Install Dependencies

```bash
cd claude-interface
pip install -r requirements.txt
```

### 2. Configure omniTAK

Ensure omniTAK is running with API enabled:

```yaml
# config.yaml
api:
  bind_addr: "127.0.0.1:9443"
  enable_cot_send: true
```

### 3. Run Claude Interface

```bash
python claude_interface/main.py
```

### 4. Start Creating TAK Objects

```
TAK> Create a 10km exclusion zone around Los Angeles
Claude: ✓ Created 10km Exclusion Zone at (34.0522, -118.2437)

TAK> Draw a patrol route from 33.1,-117.4 to 33.2,-117.5 to 33.3,-117.6
Claude: ✓ Created patrol route with 3 waypoints

TAK> What's our connection status?
Claude: Active connections: 2/2
        - tak-server-1: connected (TLS)
        - tak-server-2: connected (TCP)
```

## TAK Objects You Can Create

### Circles
- Exclusion zones
- Range rings
- Search areas
- Blast radius

### Polygons
- Area of operations (AO)
- Restricted airspace
- Sector boundaries
- Controlled zones

### Routes
- Patrol routes
- Convoy routes
- Evacuation routes
- Ingress/egress corridors

### Markers
- Objectives
- Threats
- Resources (MEDEVAC, supply points)
- Waypoints
- Checkpoints

### Data Packages (Future)
- Mission packages
- KML overlays
- Imagery
- Combined tactical picture

## Example Natural Language Commands

### Creating Objects

```
"Create a circular perimeter 3km radius around coordinates 35.0,-120.0"
"Draw a restricted area with corners at [list of coordinates]"
"Plan a route from point A through points B, C, D to point E"
"Mark location 34.5,-118.2 as an emergency landing zone"
"Place a hostile marker at grid 11S LT 12345 67890"
```

### Querying Data

```
"Show me all friendly units within 5km of my position"
"What hostile contacts have we seen in the last hour?"
"List all active medical emergencies"
"Find units in the northern sector"
"What's the closest supply point to 34.0,-118.0?"
```

### Status and Management

```
"What's the status of our TAK connections?"
"How many messages have we processed?"
"Are all servers connected?"
"Show me connection health"
```

## Architecture

```
User (Natural Language)
    ↓
Claude Agent SDK
    ↓
Python Tools
    ↓
CoT Message Builders
    ↓
omniTAK REST API
    ↓
TAK Servers
    ↓
ATAK/WinTAK Clients (Display objects)
```

## Files

- `claude_tools/tak_geometry.py` - CoT message builders
- `claude_tools/tak_tools.py` - Claude tool definitions
- `omnitak_client/client.py` - API client (pending)
- `main.py` - Main chat interface (pending)
- `INTEGRATION_PLAN.md` - Full implementation plan

## Testing Message Builders

You can test the CoT message builders independently:

```bash
cd claude-interface
python claude_tools/tak_geometry.py
```

This will print example CoT XML messages for:
- 5km exclusion zone around San Francisco
- Area of Operations polygon
- Patrol route with 3 waypoints

## Next Steps

1. **Implement omniTAK backend enhancements** (see INTEGRATION_PLAN.md Phase 1)
2. **Complete Python API client**
3. **Install Claude Agent SDK**
4. **Connect tools to live API**
5. **Test end-to-end workflow**

## Resources

- [Claude Agent SDK](https://github.com/anthropics/claude-agent-sdk-python)
- [TAK Protocol Docs](https://takproto.readthedocs.io/)
- [omniTAK Project](../README.md)

## Contributing

See [INTEGRATION_PLAN.md](INTEGRATION_PLAN.md) for implementation roadmap.

## License

Same as omniTAK main project
