# Plugin Management API Reference

REST API endpoints for managing OmniTAK plugins.

## Base URL

```
http://localhost:9443/api/v1
```

## Authentication

All endpoints require authentication via JWT token or API key.

```bash
# JWT Token
curl -H "Authorization: Bearer <token>" ...

# API Key
curl -H "X-API-Key: <key>" ...
```

## Endpoints

### List Plugins

Get all loaded plugins with optional filtering.

**Request:**
```http
GET /api/v1/plugins
```

**Query Parameters:**
- `enabled_only` (boolean): Show only enabled plugins
- `plugin_type` (string): Filter by type ("filter", "transformer")

**Response:**
```json
{
  "plugins": [
    {
      "id": "flightradar24-source",
      "name": "FlightRadar24 Integration",
      "version": "0.1.0",
      "author": "OmniTAK Community",
      "description": "Fetches live flight data",
      "capabilities": ["transform", "network-access"],
      "binaryHash": "a1b2c3d4..."
    }
  ],
  "total": 1
}
```

**Example:**
```bash
curl http://localhost:9443/api/v1/plugins?enabled_only=true
```

### Load Plugin

Load a new plugin into the system.

**Request:**
```http
POST /api/v1/plugins
```

**Body:**
```json
{
  "id": "my-plugin",
  "path": "/path/to/plugin.wasm",
  "enabled": true,
  "pluginType": "filter",
  "config": {
    "center_lat": 35.0,
    "center_lon": -79.0
  }
}
```

**Response:**
```json
{
  "id": "my-plugin",
  "name": "My Custom Plugin",
  "version": "0.1.0",
  "author": "Developer Name",
  "description": "Plugin description",
  "capabilities": ["filter"],
  "binaryHash": "d4c3b2a1..."
}
```

**Example:**
```bash
curl -X POST http://localhost:9443/api/v1/plugins \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "id": "geofence-filter",
    "path": "plugins/geofence.wasm",
    "enabled": true,
    "pluginType": "filter",
    "config": {}
  }'
```

### Get Plugin Details

Get detailed information about a specific plugin.

**Request:**
```http
GET /api/v1/plugins/{id}
```

**Response:**
```json
{
  "info": {
    "id": "flightradar24-source",
    "name": "FlightRadar24 Integration",
    "version": "0.1.0",
    "author": "OmniTAK Community",
    "description": "Fetches live flight data",
    "capabilities": ["transform", "network-access"],
    "binaryHash": "a1b2c3d4..."
  },
  "enabled": true,
  "loadedAt": "2025-01-15T12:00:00Z",
  "executionCount": 1234,
  "errorCount": 0,
  "avgExecutionTimeMs": 1.5
}
```

**Example:**
```bash
curl http://localhost:9443/api/v1/plugins/flightradar24-source
```

### Unload Plugin

Remove a plugin from the system.

**Request:**
```http
DELETE /api/v1/plugins/{id}
```

**Response:**
```
204 No Content
```

**Example:**
```bash
curl -X DELETE http://localhost:9443/api/v1/plugins/my-plugin \
  -H "Authorization: Bearer <token>"
```

### Update Plugin Configuration

Update the configuration for a running plugin.

**Request:**
```http
PUT /api/v1/plugins/{id}/config
```

**Body:**
```json
{
  "config": {
    "center_lat": 40.0,
    "center_lon": -74.0,
    "radius_degrees": 3.0
  }
}
```

**Response:**
```
200 OK
```

**Example:**
```bash
curl -X PUT http://localhost:9443/api/v1/plugins/flightradar24-source/config \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "config": {
      "center_lat": 40.7128,
      "center_lon": -74.0060,
      "radius_degrees": 2.0
    }
  }'
```

### Toggle Plugin

Enable or disable a plugin without unloading it.

**Request:**
```http
POST /api/v1/plugins/{id}/toggle
```

**Body:**
```json
{
  "enabled": false
}
```

**Response:**
```
200 OK
```

**Example:**
```bash
# Disable plugin
curl -X POST http://localhost:9443/api/v1/plugins/my-plugin/toggle \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"enabled": false}'

# Enable plugin
curl -X POST http://localhost:9443/api/v1/plugins/my-plugin/toggle \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"enabled": true}'
```

### Get Plugin Metrics

Get performance metrics for a plugin.

**Request:**
```http
GET /api/v1/plugins/{id}/metrics
```

**Response:**
```json
{
  "pluginId": "flightradar24-source",
  "executionCount": 1234,
  "errorCount": 5,
  "timeoutCount": 0,
  "avgExecutionTimeMs": 1.5,
  "p50ExecutionTimeMs": 1.2,
  "p95ExecutionTimeMs": 3.4,
  "p99ExecutionTimeMs": 5.6,
  "lastExecution": "2025-01-15T12:30:00Z",
  "lastError": "Network timeout"
}
```

**Example:**
```bash
curl http://localhost:9443/api/v1/plugins/flightradar24-source/metrics
```

### Get Plugin Health

Check the health status of a plugin.

**Request:**
```http
GET /api/v1/plugins/{id}/health
```

**Response:**
```json
{
  "pluginId": "flightradar24-source",
  "status": "healthy",
  "healthCheckTime": "2025-01-15T12:30:00Z",
  "uptimeSeconds": 86400,
  "issues": []
}
```

**Status values:**
- `healthy` - Plugin operating normally
- `degraded` - Plugin working but with issues
- `unhealthy` - Plugin failing
- `disabled` - Plugin disabled

**Example:**
```bash
curl http://localhost:9443/api/v1/plugins/flightradar24-source/health
```

### Reload Plugin

Reload a plugin (unload and load again).

**Request:**
```http
POST /api/v1/plugins/{id}/reload
```

**Response:**
```
200 OK
```

**Example:**
```bash
curl -X POST http://localhost:9443/api/v1/plugins/my-plugin/reload \
  -H "Authorization: Bearer <token>"
```

### Reload All Plugins

Reload all plugins in the system.

**Request:**
```http
POST /api/v1/plugins/reload-all
```

**Response:**
```
200 OK
```

**Example:**
```bash
curl -X POST http://localhost:9443/api/v1/plugins/reload-all \
  -H "Authorization: Bearer <token>"
```

## Error Responses

All endpoints return standard error responses:

```json
{
  "error": "Plugin not found: my-plugin",
  "code": "NOT_FOUND",
  "timestamp": "2025-01-15T12:00:00Z"
}
```

**Status Codes:**
- `200` - Success
- `201` - Created
- `204` - No Content (success with no body)
- `400` - Bad Request (validation error)
- `401` - Unauthorized (authentication required)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found
- `500` - Internal Server Error

## Permissions

Plugin management requires specific role permissions:

| Endpoint | Required Role |
|----------|---------------|
| GET /plugins | User |
| GET /plugins/:id | User |
| GET /plugins/:id/metrics | User |
| GET /plugins/:id/health | User |
| POST /plugins | Admin |
| DELETE /plugins/:id | Admin |
| PUT /plugins/:id/config | Operator |
| POST /plugins/:id/toggle | Operator |
| POST /plugins/:id/reload | Admin |
| POST /plugins/reload-all | Admin |

## WebSocket Updates

Subscribe to real-time plugin status updates:

```javascript
const ws = new WebSocket('ws://localhost:9443/api/v1/stream');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  if (data.type === 'plugin_status') {
    console.log('Plugin status:', data.plugin_id, data.status);
  }

  if (data.type === 'plugin_metric') {
    console.log('Plugin metric:', data.plugin_id, data.execution_time_ms);
  }
};

// Subscribe to plugin events
ws.send(JSON.stringify({
  type: 'subscribe',
  events: ['plugin_status', 'plugin_metric']
}));
```

## Example Workflow

### Loading and Configuring a Plugin

```bash
# 1. Load the plugin
curl -X POST http://localhost:9443/api/v1/plugins \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "id": "flightradar24",
    "path": "plugins/flightradar24_source.wasm",
    "enabled": true,
    "pluginType": "transformer",
    "config": {
      "center_lat": 35.0,
      "center_lon": -79.0,
      "radius_degrees": 2.0,
      "update_interval_secs": 30,
      "enabled": true
    }
  }'

# 2. Check plugin status
curl http://localhost:9443/api/v1/plugins/flightradar24

# 3. Monitor metrics
curl http://localhost:9443/api/v1/plugins/flightradar24/metrics

# 4. Update configuration
curl -X PUT http://localhost:9443/api/v1/plugins/flightradar24/config \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "config": {
      "center_lat": 40.7128,
      "center_lon": -74.0060,
      "radius_degrees": 5.0
    }
  }'

# 5. Check health
curl http://localhost:9443/api/v1/plugins/flightradar24/health

# 6. Reload if needed
curl -X POST http://localhost:9443/api/v1/plugins/flightradar24/reload \
  -H "Authorization: Bearer $TOKEN"
```

## Rate Limiting

API requests are rate-limited to:
- 100 requests per minute for read operations
- 10 requests per minute for write operations

Rate limit headers are included in responses:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1642262400
```

## Pagination

List endpoints support pagination:

```bash
curl "http://localhost:9443/api/v1/plugins?page=1&per_page=20"
```

Response includes pagination metadata:
```json
{
  "plugins": [...],
  "total": 45,
  "page": 1,
  "perPage": 20,
  "totalPages": 3
}
```

## Filtering and Sorting

List endpoints support advanced filtering:

```bash
# Filter by capabilities
curl "http://localhost:9443/api/v1/plugins?capabilities=filter,transform"

# Sort by name
curl "http://localhost:9443/api/v1/plugins?sort=name&order=asc"

# Combine filters
curl "http://localhost:9443/api/v1/plugins?enabled_only=true&sort=name"
```

## Monitoring and Observability

### Prometheus Metrics

Plugin metrics are exposed at `/api/v1/metrics`:

```
# HELP omnitak_plugin_executions_total Total plugin executions
# TYPE omnitak_plugin_executions_total counter
omnitak_plugin_executions_total{plugin_id="flightradar24"} 1234

# HELP omnitak_plugin_execution_duration_seconds Plugin execution duration
# TYPE omnitak_plugin_execution_duration_seconds histogram
omnitak_plugin_execution_duration_seconds_bucket{plugin_id="flightradar24",le="0.001"} 950
omnitak_plugin_execution_duration_seconds_bucket{plugin_id="flightradar24",le="0.01"} 1200
omnitak_plugin_execution_duration_seconds_count{plugin_id="flightradar24"} 1234
omnitak_plugin_execution_duration_seconds_sum{plugin_id="flightradar24"} 1.85

# HELP omnitak_plugin_errors_total Total plugin errors
# TYPE omnitak_plugin_errors_total counter
omnitak_plugin_errors_total{plugin_id="flightradar24",error_type="timeout"} 0
```

### Health Checks

Plugin health is included in the system health endpoint:

```bash
curl http://localhost:9443/api/v1/health
```

```json
{
  "status": "healthy",
  "plugins": {
    "total": 5,
    "healthy": 4,
    "degraded": 1,
    "unhealthy": 0
  }
}
```

---

For more information, see:
- [Plugin Development Guide](PLUGIN_DEVELOPMENT.md)
- [API Authentication](AUTH_API.md)
- [WebSocket Streaming](WEBSOCKET_API.md)
