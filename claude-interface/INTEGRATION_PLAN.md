# Claude Agent SDK + omniTAK Integration Plan

## Overview

This document outlines the integration of the [Claude Agent SDK for Python](https://github.com/anthropics/claude-agent-sdk-python) with omniTAK to create a **natural language interface for TAK server operations**.

Users will be able to use conversational commands to:
- Create tactical objects (circles, polygons, routes, markers)
- Query TAK data with natural language filters
- Manage server connections
- Explore CoT messages and tactical picture

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│  User: Natural Language Interface                       │
│  "Create a 5km exclusion zone around 37.7749, -122.4194" │
└────────────────────────┬─────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────┐
│  Claude Agent SDK (Python)                               │
│  - Parse user intent                                     │
│  - Extract parameters (coordinates, types, etc.)         │
│  - Validate inputs                                       │
│  - Handle context and conversation flow                  │
└────────────────────────┬─────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────┐
│  Custom Tools (@tool decorators)                         │
│  - create_exclusion_zone()                               │
│  - create_area_polygon()                                 │
│  - create_patrol_route()                                 │
│  - place_marker()                                        │
│  - get_tak_status()                                      │
│  - query_tak_data()                                      │
└────────────────────────┬─────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────┐
│  CoT Message Builder (Python)                            │
│  - Build geometry objects (Circle, Polygon, Route)       │
│  - Serialize to CoT XML format                           │
│  - Generate UIDs and timestamps                          │
│  - Apply MIL-STD-2525 types                              │
└────────────────────────┬─────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────┐
│  omniTAK REST API Client (Python)                        │
│  - HTTP: POST /api/v1/cot/send                           │
│  - HTTP: GET /api/v1/connections                         │
│  - HTTP: GET /api/v1/status                              │
│  - WebSocket: /api/v1/stream (message query)             │
└────────────────────────┬─────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────┐
│  omniTAK (Rust Backend) - ENHANCED                       │
│  - NEW: POST /api/v1/cot/send endpoint                   │
│  - NEW: Geometry support in omnitak-cot crate            │
│  - EXISTING: Connection pool management                  │
│  - EXISTING: Message routing and filtering               │
└────────────────────────┬─────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────┐
│  TAK Servers (Multiple protocols)                        │
│  - TLS client certificate authentication                 │
│  - TCP/UDP/WebSocket protocols                           │
└────────────────────────┬─────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────┐
│  TAK Clients (ATAK, WinTAK, iTAK)                        │
│  - Display shapes, routes, markers on map                │
│  - Real-time updates                                     │
└──────────────────────────────────────────────────────────┘
```

## Implementation Phases

### Phase 1: Foundation (Week 1-2)

#### 1.1 omniTAK Backend Enhancements

**Add Geometry Support to omnitak-cot**

Location: `/crates/omnitak-cot/`

- [ ] Extend protobuf definitions (`proto/geometry.proto`):
  ```proto
  message Polygon {
      repeated Point points = 1;
  }

  message LineString {
      repeated Point points = 1;
  }

  message Circle {
      Point center = 1;
      double radius_meters = 2;
  }
  ```

- [ ] Update `src/event.rs` with geometry enum:
  ```rust
  pub enum Geometry {
      Point(Point),
      Polygon(Vec<Point>),
      LineString(Vec<Point>),
      Circle { center: Point, radius: f64 },
  }
  ```

- [ ] Extend `src/serializer.rs` to serialize all geometry types to XML
- [ ] Update `src/parser.rs` to parse shape elements from CoT XML
- [ ] Add unit tests for geometry serialization/deserialization

**Add CoT Send API Endpoint**

Location: `/crates/omnitak-api/`

- [ ] Create `POST /api/v1/cot/send` endpoint in `src/routes.rs`:
  ```rust
  #[derive(Deserialize)]
  struct SendCotRequest {
      message: String,  // XML or base64-encoded protobuf
      server_id: Option<String>,  // Send to specific server or all
  }

  async fn send_cot_message(
      State(pool): State<Arc<ConnectionPool>>,
      Json(request): Json<SendCotRequest>
  ) -> Result<Json<SendCotResponse>, ApiError> {
      // Parse message
      // Route to server(s)
      // Return confirmation
  }
  ```

- [ ] Add message validation
- [ ] Add authentication/authorization checks
- [ ] Add rate limiting

**Update omnitak-pool for Message Injection**

Location: `/crates/omnitak-pool/`

- [ ] Add `inject_message()` method to ConnectionPool:
  ```rust
  pub async fn inject_message(
      &self,
      message: CotMessage,
      target_server: Option<&str>
  ) -> Result<(), PoolError>
  ```

- [ ] Route injected messages through existing client send path

#### 1.2 Python API Client

Location: `/claude-interface/omnitak_client/`

- [x] Create `tak_geometry.py` - CoT message builders ✓
- [x] Create `tak_tools.py` - Tool function stubs ✓
- [ ] Create `client.py` - omniTAK HTTP/WebSocket client:
  ```python
  class OmniTAKClient:
      async def send_cot_message(xml: str, server_id: str = None)
      async def get_connections()
      async def get_status()
      async def stream_messages(filters: dict)
  ```

- [ ] Add authentication (JWT/API key)
- [ ] Add retry logic with exponential backoff
- [ ] Add connection pooling
- [ ] Add comprehensive error handling

### Phase 2: Claude Integration (Week 3)

#### 2.1 Claude Agent SDK Setup

- [ ] Install dependencies:
  ```bash
  pip install claude-agent-sdk aiohttp pydantic
  ```

- [ ] Create `claude_interface/main.py`:
  ```python
  from claude_agent_sdk import ClaudeSDKClient
  from claude_tools import (
      create_exclusion_zone,
      create_area_polygon,
      create_patrol_route,
      place_marker,
      get_tak_status,
      query_tak_data
  )

  async def main():
      client = ClaudeSDKClient(
          tools=[
              create_exclusion_zone,
              create_area_polygon,
              create_patrol_route,
              place_marker,
              get_tak_status,
              query_tak_data
          ]
      )

      # Start conversational loop
      while True:
          user_input = input("TAK> ")
          response = await client.query(user_input)
          print(response)
  ```

#### 2.2 Tool Implementation

- [ ] Implement all tools in `tak_tools.py`:
  - [x] Stub implementations created ✓
  - [ ] Connect to actual OmniTAKClient
  - [ ] Add error handling
  - [ ] Add parameter validation
  - [ ] Add success/failure responses

- [ ] Create system prompt for Claude:
  ```
  You are a TAK operations assistant. You help users create tactical objects,
  query data, and manage TAK server connections using natural language.

  When users describe locations, objects, or queries, translate them into
  the appropriate tool calls with correct parameters.

  Available capabilities:
  - Create circles (exclusion zones, range rings)
  - Create polygons (areas, zones, boundaries)
  - Create routes (patrol paths, movement corridors)
  - Place markers (objectives, threats, resources)
  - Query tactical data (filtered by time, location, affiliation)
  - Check server status
  ```

### Phase 3: Advanced Features (Week 4-5)

#### 3.1 Data Packages

- [ ] Implement mission package creation:
  ```python
  @tool
  async def create_mission_package(
      name: str,
      items: List[str],  # UIDs of objects to include
      imagery: Optional[str] = None
  ) -> str:
      """Create a TAK data package (.zip)"""
  ```

- [ ] Include: CoT objects, KML, imagery, routes
- [ ] Upload to TAK server

#### 3.2 Real-Time Streaming

- [ ] WebSocket client for live CoT messages:
  ```python
  @tool
  async def monitor_contacts(
      affiliation: str,
      area: Tuple[float, float, float, float],
      duration_minutes: int = 60
  ) -> str:
      """Monitor contacts in real-time"""
  ```

- [ ] Stream filtered messages
- [ ] Generate summaries and alerts

#### 3.3 Intelligence Analysis

- [ ] Pattern recognition tools:
  ```python
  @tool
  async def analyze_movement_patterns(
      time_window_hours: int = 24,
      area: Optional[Tuple] = None
  ) -> str:
      """Analyze movement patterns in tactical data"""
  ```

- [ ] Anomaly detection
- [ ] Threat assessment

### Phase 4: User Interface (Week 6)

#### 4.1 CLI Interface

- [ ] Rich terminal UI with:
  - Command history
  - Autocomplete
  - Syntax highlighting
  - Map ASCII visualization

#### 4.2 Web Interface (Optional)

- [ ] Create web UI with chat interface
- [ ] Integrate with omniTAK's existing web interface
- [ ] Real-time map updates

## Example User Workflows

### Workflow 1: Create Tactical Boundaries

```
User: "I need to set up our area of operations for tonight's mission"

Claude: "I can help you create the tactical boundaries. What are the coordinates
         of your area of operations?"

User: "Northwest corner is 34.0, -118.0. Northeast is 34.0, -117.0.
       Southeast is 33.5, -117.0. Southwest is 33.5, -118.0"

Claude: [Creates polygon] "I've created your area of operations as a green
         polygon on the map. All TAK clients should now see it. Would you
         like me to add any exclusion zones or objective markers?"

User: "Yes, create a 2km exclusion zone around the center"

Claude: [Calculates center, creates circle] "I've added a 2km red exclusion
         zone centered at 33.75, -117.5. Anything else?"
```

### Workflow 2: Plan a Route

```
User: "Plan a patrol route from our base at 33.123, -117.456 to the
       checkpoint at 33.234, -117.567 and then to the objective at 33.345, -117.678"

Claude: [Creates route with 3 waypoints] "I've created a patrol route with:
         - Start: Base (33.123, -117.456)
         - Waypoint 1: Checkpoint (33.234, -117.567)
         - End: Objective (33.345, -117.678)

         The route is displayed in yellow on all TAK clients. The total
         distance is approximately 28.4km. Would you like me to create
         a mission package with this route?"
```

### Workflow 3: Intelligence Queries

```
User: "Show me all hostile contacts from the last 30 minutes within 10km
       of our position at 34.0, -118.0"

Claude: [Queries CoT stream] "I found 3 hostile contacts in the last 30 minutes:
         1. H-01 at 34.05, -118.03 (7.2km NE) - Ground unit, last seen 5 min ago
         2. H-02 at 33.97, -117.95 (4.8km SE) - Vehicle, moving west at 15 km/h
         3. H-03 at 34.08, -118.08 (9.1km NW) - Unknown, stationary

         Would you like me to create markers for these positions or
         generate a threat assessment?"
```

## Technical Requirements

### Python Environment

```bash
# requirements.txt
claude-agent-sdk>=0.1.0
aiohttp>=3.9.0
pydantic>=2.0.0
python-dateutil>=2.8.0
```

### omniTAK Configuration

Add to `config.yaml`:

```yaml
api:
  bind_addr: "127.0.0.1:9443"
  enable_cot_send: true  # NEW: Enable CoT message injection
  max_injection_rate: 100  # NEW: Messages per second limit

claude_integration:  # NEW section
  enabled: true
  allowed_ips: ["127.0.0.1"]  # Only allow localhost by default
  require_auth: true
```

## Security Considerations

1. **Authentication**: All API calls must use JWT or API keys
2. **Rate Limiting**: Prevent message flooding (100 msg/sec default)
3. **Input Validation**: Sanitize all user inputs before creating CoT messages
4. **Access Control**: Role-based permissions for message creation
5. **Audit Logging**: Log all Claude-generated TAK objects
6. **Network Isolation**: Run Claude interface on localhost by default

## Testing Strategy

### Unit Tests
- [ ] CoT message builders (geometry validation)
- [ ] Tool parameter parsing
- [ ] API client error handling

### Integration Tests
- [ ] End-to-end message creation and delivery
- [ ] Multi-server routing
- [ ] WebSocket streaming

### User Acceptance Tests
- [ ] Natural language command parsing accuracy
- [ ] Object creation on live TAK servers
- [ ] Performance with 100+ concurrent users

## Success Metrics

1. **Accuracy**: 95%+ correct interpretation of user commands
2. **Latency**: <1 second from command to TAK object creation
3. **Reliability**: 99.9% uptime for API endpoints
4. **Usability**: Users can create complex tactical pictures in <5 minutes

## Future Enhancements

1. **Voice Interface**: Speech-to-text for hands-free operation
2. **Mobile App**: ATAK plugin with Claude integration
3. **Multi-Language**: Support for non-English commands
4. **AI Planning**: Autonomous route planning and optimization
5. **3D Visualization**: Integrate with 3D terrain and airspace models
6. **Collaborative Planning**: Multi-user mission planning sessions

## Resources

- [Claude Agent SDK Documentation](https://github.com/anthropics/claude-agent-sdk-python)
- [TAK Protocol Documentation](https://takproto.readthedocs.io/)
- [MIL-STD-2525 Symbology](https://en.wikipedia.org/wiki/NATO_Joint_Military_Symbology)
- [CoT Developer's Guide (PDF)](https://tutorials.techrad.co.za/wp-content/uploads/2021/06/The-Developers-Guide-to-Cursor-on-Target-1.pdf)

## Timeline

- **Week 1-2**: Backend enhancements (geometry support, API endpoints)
- **Week 3**: Claude SDK integration and tool implementation
- **Week 4-5**: Advanced features (data packages, streaming, analysis)
- **Week 6**: User interface and documentation
- **Week 7**: Testing and refinement
- **Week 8**: Deployment and training

## Team Roles

- **Rust Developer**: omniTAK backend enhancements
- **Python Developer**: Claude SDK integration and tools
- **DevOps**: Deployment, monitoring, security
- **TAK SME**: Requirements, testing, validation
- **Documentation**: User guides, API docs, training materials
