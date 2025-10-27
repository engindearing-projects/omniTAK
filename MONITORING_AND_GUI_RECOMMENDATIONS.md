# OmniTAK Monitoring & GUI Recommendations
**Date:** October 27, 2025
**Status:** Analysis & Recommendations

---

## Executive Summary

This document provides:
1. **Root cause analysis** of the utoipa-swagger-ui dependency issue
2. **Solutions** to fix the Swagger UI problem
3. **Comprehensive GUI recommendations** for monitoring across all platforms (web, CLI, embedded)

---

## Part 1: Swagger Dependency Issue

### Problem Identification

**Issue:** `utoipa-swagger-ui` v7.1.0 fails to build with error:
```
Error: `folder` must be a relative path under `compression` feature
```

**Location:** `crates/omnitak-api/Cargo.toml:47`

**Root Cause:**
The `utoipa-swagger-ui` v7.1.0 has a build script issue with the `rust-embed` crate when the `compression` feature is enabled. This is a known third-party dependency bug, not a code issue in OmniTAK.

**Current Usage:**
```rust
// crates/omnitak-api/src/lib.rs:289
app = app.merge(SwaggerUi::new("/swagger-ui")
    .url("/api-docs/openapi.json", ApiDoc::openapi()));
```

### Solutions (Ranked by Preference)

#### Solution 1: Update to Latest utoipa-swagger-ui (RECOMMENDED)

**Action:**
```toml
# crates/omnitak-api/Cargo.toml
utoipa-swagger-ui = { version = "8.0", features = ["axum"] }
```

**Pros:**
- Fixes the build issue
- Gets latest bug fixes and features
- Maintains OpenAPI documentation
- No code changes required (API compatible)

**Cons:**
- May have minor API changes
- Need to verify compatibility

**Verification Steps:**
```bash
# Update dependency
sed -i 's/utoipa-swagger-ui = { version = "7.1"/utoipa-swagger-ui = { version = "8.0"/' crates/omnitak-api/Cargo.toml

# Test build
cargo build -p omnitak-api

# If v8.0 unavailable, try latest 7.x
cargo update -p utoipa-swagger-ui
```

#### Solution 2: Use rapidoc or redoc Instead

**Action:**
Replace `utoipa-swagger-ui` with `utoipa-rapidoc` or `utoipa-redoc`:

```toml
# Cargo.toml
utoipa-rapidoc = { version = "4.0", features = ["axum"] }
# OR
utoipa-redoc = { version = "4.0", features = ["axum"] }
```

```rust
// lib.rs
use utoipa_rapidoc::RapiDoc;
// OR
use utoipa_redoc::{Redoc, Servable};

// In build() method:
app = app.merge(RapiDoc::new("/api-docs/openapi.json")
    .path("/rapidoc"));
// OR
app = app.merge(Redoc::with_url("/redoc", ApiDoc::openapi()));
```

**Pros:**
- Alternative UI for OpenAPI docs
- Often lighter weight than SwaggerUI
- Modern interface

**Cons:**
- Requires code changes
- Different UI paradigm

#### Solution 3: Disable Swagger UI Feature (TEMPORARY)

**Action:**
```toml
# Cargo.toml - comment out the dependency
# utoipa-swagger-ui = { version = "7.1", features = ["axum"] }
```

```rust
// lib.rs - keep OpenAPI spec endpoint, remove UI
// Remove: use utoipa_swagger_ui::SwaggerUi;

// Keep this for raw spec access:
app = app.route("/api-docs/openapi.json",
    axum::routing::get(|| async {
        axum::Json(ApiDoc::openapi())
    }));

// Remove/comment out:
// if self.config.enable_swagger {
//     app = app.merge(SwaggerUi::new("/swagger-ui")...);
// }
```

**Pros:**
- Core API still works
- OpenAPI spec still available at `/api-docs/openapi.json`
- Can use external tools (Postman, Insomnia) to view spec

**Cons:**
- No built-in documentation UI
- Less user-friendly for API exploration

#### Solution 4: Make Swagger UI Optional with Feature Flag

**Action:**
```toml
# Cargo.toml
[features]
default = []
swagger-ui = ["utoipa-swagger-ui"]

[dependencies]
utoipa-swagger-ui = { version = "7.1", features = ["axum"], optional = true }
```

```rust
// lib.rs
#[cfg(feature = "swagger-ui")]
use utoipa_swagger_ui::SwaggerUi;

// In build():
#[cfg(feature = "swagger-ui")]
if self.config.enable_swagger {
    app = app.merge(SwaggerUi::new("/swagger-ui")...);
}
```

**Pros:**
- Build works without swagger-ui
- Can enable later when fixed
- Clean separation of concerns

**Cons:**
- Adds complexity
- Still doesn't fix the underlying issue

---

## Part 2: GUI & Monitoring Solutions

### Requirements Analysis

Based on your requirements, the GUI system needs:
1. **Extensible** - Easy to add new features/metrics
2. **Universal Access** - Work on anything that runs this program
3. **CLI Compatible** - Terminal-based monitoring option
4. **Real-time** - Live updates of system state
5. **Military-Grade** - Secure, reliable, production-ready

### Recommended Multi-Tier Approach

Given that OmniTAK is designed for tactical military environments (including resource-constrained edge devices), I recommend a **three-tier monitoring solution**:

---

## Tier 1: Terminal UI (TUI) - CLI Monitoring

### Technology: Ratatui (formerly tui-rs)

**Why Ratatui:**
- Pure Rust, no external dependencies
- Works over SSH
- Low resource usage (<5MB RAM)
- Works in disconnected/degraded networks
- Perfect for tactical edge environments

**Implementation Plan:**

```toml
# Create new crate: crates/omnitak-tui/Cargo.toml
[package]
name = "omnitak-tui"
version = "0.1.0"
edition = "2021"

[dependencies]
omnitak-core = { path = "../omnitak-core" }
ratatui = "0.28"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["rustls-tls", "json"] }
serde_json = "1.0"
```

**Features to Implement:**

1. **Dashboard View**
   ```
   ┌─ OmniTAK Monitor ─────────────────────────────────┐
   │ Status: OPERATIONAL    Uptime: 3d 14h 32m         │
   │ Connections: 47/1000   Messages/s: 1,247          │
   ├───────────────────────────────────────────────────┤
   │ Active Connections               │ Message Flow   │
   │  ✓ TAK-SRV-01  192.168.1.10:8087│  ↑ 847/s      │
   │  ✓ TAK-SRV-02  192.168.1.11:8087│  ↓ 400/s      │
   │  ✗ TAK-SRV-03  192.168.1.12:8087│                │
   │  ✓ TAK-SRV-04  10.0.0.50:8087   │ Errors: 0      │
   ├───────────────────────────────────────────────────┤
   │ Latest Events                                      │
   │ [12:34:01] Connection TAK-SRV-05 established      │
   │ [12:33:45] Filtered 23 messages (affiliation)     │
   │ [12:33:12] Health check passed (47/47 ok)         │
   └───────────────────────────────────────────────────┘
   [q] Quit [c] Connections [f] Filters [m] Metrics [h] Help
   ```

2. **Connection Details View**
   - List all connections with status
   - Connection statistics (bytes in/out, messages, errors)
   - Add/remove connections interactively

3. **Filter Management View**
   - Active filters list
   - Create/edit/delete filters
   - Filter statistics (matched/dropped)

4. **Metrics View**
   - Real-time graphs (sparklines)
   - Message throughput histogram
   - Latency percentiles
   - Memory/CPU usage

5. **Logs View**
   - Scrollable log viewer
   - Filter by level (ERROR, WARN, INFO, DEBUG)
   - Search functionality

**Sample Implementation Structure:**

```rust
// crates/omnitak-tui/src/main.rs
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Layout, Constraint, Direction},
    widgets::{Block, Borders, Paragraph, List, Gauge},
};

#[derive(Default)]
struct AppState {
    connections: Vec<Connection>,
    metrics: Metrics,
    events: Vec<Event>,
    current_view: View,
}

enum View {
    Dashboard,
    Connections,
    Filters,
    Metrics,
    Logs,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize terminal
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    // Connect to OmniTAK API
    let client = ApiClient::new("https://localhost:8443")?;

    // Main event loop
    let mut app = AppState::default();
    loop {
        // Fetch latest data
        app.update(&client).await?;

        // Render UI
        terminal.draw(|f| {
            render_dashboard(f, &app);
        })?;

        // Handle input
        if let Event::Key(key) = crossterm::event::read()? {
            handle_key_event(&mut app, key);
        }
    }
}
```

**Usage:**
```bash
# Run TUI monitor
./omnitak-tui --endpoint https://localhost:8443 --token $TOKEN

# Or as a command in main binary
omnitak monitor --tui
```

---

## Tier 2: Web Dashboard - Full-Featured GUI

### Technology: React + WebSocket + Tailwind CSS

**Why This Stack:**
- React: Widely adopted, component-based, excellent ecosystem
- WebSocket: Already implemented in omnitak-api
- Tailwind CSS: Rapid UI development, responsive design
- Vite: Fast development builds

**Alternative Options:**
- **SvelteKit**: Lighter weight, potentially better performance
- **Leptos (Rust WASM)**: Pure Rust stack, excellent performance
- **Dioxus (Rust)**: Rust-based React-like framework

**Implementation Plan:**

```bash
# Create web frontend
mkdir -p web/dashboard
cd web/dashboard
npm create vite@latest . -- --template react-ts
```

```json
// package.json dependencies
{
  "dependencies": {
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "recharts": "^2.12.0",      // Charts
    "react-map-gl": "^7.1.0",    // Map display
    "maplibre-gl": "^4.0.0",     // Military map rendering
    "zustand": "^4.5.0",         // State management
    "axios": "^1.7.0",           // HTTP client
    "date-fns": "^3.0.0"         // Date formatting
  }
}
```

**Key Features:**

### 1. Real-Time Map View
```typescript
// components/MapView.tsx
import Map from 'react-map-gl/maplibre';
import { useEffect, useState } from 'react';

export function MapView() {
  const [entities, setEntities] = useState([]);

  useEffect(() => {
    // Connect to WebSocket
    const ws = new WebSocket('wss://localhost:8443/api/v1/stream');

    ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      if (msg.type === 'cot_message') {
        updateEntity(msg);
      }
    };

    // Send subscription
    ws.onopen = () => {
      ws.send(JSON.stringify({
        type: 'subscribe',
        event_types: ['a-f-G', 'a-h-G', 'a-n-G']
      }));
    };

    return () => ws.close();
  }, []);

  return (
    <Map
      initialViewState={{
        latitude: 35.0,
        longitude: -118.0,
        zoom: 10
      }}
      mapStyle="maplibre://tactical"
    >
      {entities.map(entity => (
        <Marker key={entity.uid} {...entity} />
      ))}
    </Map>
  );
}
```

### 2. Connection Dashboard
```typescript
// components/ConnectionDashboard.tsx
import { useQuery } from '@tanstack/react-query';

export function ConnectionDashboard() {
  const { data } = useQuery({
    queryKey: ['connections'],
    queryFn: () => fetch('/api/v1/connections').then(r => r.json()),
    refetchInterval: 2000 // Poll every 2 seconds
  });

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {data?.connections.map(conn => (
        <ConnectionCard key={conn.id} connection={conn} />
      ))}
    </div>
  );
}
```

### 3. Metrics Dashboard
```typescript
// components/MetricsDashboard.tsx
import { LineChart, Line, XAxis, YAxis } from 'recharts';

export function MetricsDashboard() {
  return (
    <div className="grid grid-cols-2 gap-4">
      <Card title="Message Throughput">
        <LineChart data={throughputData}>
          <Line type="monotone" dataKey="messages" stroke="#8884d8" />
        </LineChart>
      </Card>

      <Card title="Connection Status">
        <PieChart data={connectionStats} />
      </Card>

      <Card title="Latency (p99)">
        <AreaChart data={latencyData} />
      </Card>

      <Card title="Active Filters">
        <FilterList filters={activeFilters} />
      </Card>
    </div>
  );
}
```

### 4. Page Structure
```
web/dashboard/
├── src/
│   ├── components/
│   │   ├── MapView.tsx          # Real-time tactical map
│   │   ├── ConnectionList.tsx   # Connection management
│   │   ├── MetricsDashboard.tsx # Charts and stats
│   │   ├── FilterManager.tsx    # Filter CRUD
│   │   ├── EventLog.tsx         # Real-time event stream
│   │   └── Settings.tsx         # Configuration
│   ├── hooks/
│   │   ├── useWebSocket.ts      # WebSocket connection hook
│   │   ├── useApi.ts            # REST API hook
│   │   └── useAuth.ts           # Authentication hook
│   ├── stores/
│   │   └── appStore.ts          # Global state
│   ├── types/
│   │   └── api.ts               # TypeScript types for API
│   └── App.tsx                  # Main app component
└── package.json
```

**Embedding in Rust Binary:**

```toml
# crates/omnitak-api/Cargo.toml
[dependencies]
rust-embed = { version = "8.5", features = ["compression"] }
```

```rust
// crates/omnitak-api/src/static_files.rs
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../web/dashboard/dist"]
struct WebAssets;

pub fn create_static_router() -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/assets/*file", get(serve_asset))
}
```

**Build Process:**
```bash
# Build frontend
cd web/dashboard
npm run build

# Build backend (includes embedded frontend)
cargo build --release -p omnitak-api

# Frontend is now embedded in the binary!
```

---

## Tier 3: Embedded Status Display (CLI One-Liners)

### Quick Status Commands

For ultimate simplicity, add these CLI commands:

```rust
// In main.rs or new crates/omnitak-cli/
#[derive(Subcommand)]
enum Commands {
    /// Run the main server
    Run {
        #[arg(long)]
        config: PathBuf
    },

    /// Show real-time status
    Status {
        #[arg(long, default_value = "https://localhost:8443")]
        endpoint: String,
    },

    /// Monitor in terminal UI
    Monitor {
        #[arg(long)]
        endpoint: String,
    },

    /// List connections
    Connections {
        #[arg(long)]
        endpoint: String,
    },

    /// Show metrics
    Metrics {
        #[arg(long)]
        endpoint: String,
    },
}
```

**Usage Examples:**
```bash
# Quick status check
omnitak status
# Output:
# Status: OPERATIONAL
# Uptime: 3d 14h 32m
# Connections: 47/1000 (4.7%)
# Messages/s: 1,247
# Errors/hr: 0

# Watch mode (updates every 2s)
omnitak status --watch

# List connections with filtering
omnitak connections --status active --protocol tls

# Show metrics in Prometheus format
omnitak metrics --format prometheus

# Or as compact JSON
omnitak metrics --format json --compact
```

---

## Implementation Priority & Timeline

### Phase 1: Foundation (Week 1-2)
- [ ] Fix swagger dependency (Solution 1 or 2)
- [ ] Test API endpoints work correctly
- [ ] Document API thoroughly

### Phase 2: Web Dashboard MVP (Week 3-4)
- [ ] Set up React + Vite project
- [ ] Create basic layout and routing
- [ ] Implement connection list view
- [ ] Add metrics dashboard with charts
- [ ] WebSocket integration for real-time updates
- [ ] Embed in Rust binary

### Phase 3: TUI Development (Week 5-6)
- [ ] Create omnitak-tui crate
- [ ] Implement dashboard view
- [ ] Add connection management
- [ ] Add metrics graphs (sparklines)
- [ ] Add log viewer

### Phase 4: Enhanced Features (Week 7-8)
- [ ] Map view with CoT entity display
- [ ] Filter management UI (web)
- [ ] Advanced metrics and alerting
- [ ] User management UI
- [ ] Audit log viewer

### Phase 5: Polish & Documentation (Week 9-10)
- [ ] CLI quick commands
- [ ] Comprehensive documentation
- [ ] Video tutorials
- [ ] Example configurations
- [ ] Performance optimization

---

## Alternative: Leptos (Pure Rust Full-Stack)

If you want to keep everything in Rust, consider **Leptos**:

```toml
[dependencies]
leptos = { version = "0.6", features = ["csr"] }
leptos_router = "0.6"
```

**Advantages:**
- Pure Rust ecosystem
- Excellent performance (WASM)
- Type safety across frontend/backend
- Smaller bundle sizes
- No JavaScript toolchain needed

**Disadvantages:**
- Less mature ecosystem than React
- Steeper learning curve
- Fewer UI component libraries

---

## Security Considerations

### Web Dashboard
- [ ] Enforce HTTPS only
- [ ] JWT token authentication
- [ ] CORS configuration for production
- [ ] Content Security Policy headers
- [ ] Rate limiting on API endpoints
- [ ] Audit logging for all actions

### TUI
- [ ] Secure token storage
- [ ] Certificate validation
- [ ] No password echoing
- [ ] Session timeout

---

## Testing Strategy

### Web Dashboard
```bash
# Unit tests
npm run test

# E2E tests with Playwright
npm run test:e2e

# Visual regression tests
npm run test:visual
```

### TUI
```bash
# Unit tests
cargo test -p omnitak-tui

# Integration tests
cargo test -p omnitak-tui --test integration
```

---

## Resource Requirements

### Web Dashboard
- **Development**: Node.js 20+, npm/pnpm
- **Runtime**: Embedded in Rust binary (0 bytes extra!)
- **Browser**: Modern browser with WebSocket support

### TUI
- **Development**: Rust 1.90+
- **Runtime**: ~5MB RAM, works in any terminal
- **Terminal**: Any ANSI-compatible terminal (xterm, tmux, ssh)

---

## Deployment Scenarios

### Scenario 1: Tactical Operations Center
- **GUI**: Web dashboard on local network
- **Access**: Multiple operators on workstations
- **Network**: LAN only, no internet

### Scenario 2: Field Operations (Degraded Network)
- **GUI**: TUI over SSH
- **Access**: Single operator via tactical radio
- **Network**: Low bandwidth, high latency

### Scenario 3: Remote Monitoring
- **GUI**: Web dashboard + TUI
- **Access**: SOC team via VPN
- **Network**: Internet with TLS

---

## Conclusion

### Recommended Approach

**Immediate (This Sprint):**
1. Fix swagger dependency using Solution 1 (update to v8.0)
2. Verify all API endpoints work
3. Create basic TUI dashboard (1-2 day task)

**Short-term (Next Sprint):**
4. Build React-based web dashboard MVP
5. Implement real-time WebSocket updates
6. Add map view for CoT entities

**Long-term (Future):**
7. Enhanced metrics and alerting
8. Mobile-responsive design
9. Offline mode support
10. Custom dashboard builder

### Why This Approach?

✅ **Extensible**: Component-based architecture, easy to add features
✅ **Universal**: Works on web, CLI, SSH, embedded systems
✅ **Military-Grade**: Secure by default, works in degraded networks
✅ **Production-Ready**: Built on proven technologies
✅ **Rust-Native**: TUI is pure Rust, web assets embedded in binary

---

## Next Steps

1. **Review this document** and provide feedback
2. **Choose swagger fix solution** (recommend Solution 1)
3. **Decide on GUI priority**: TUI first or Web first?
4. **Allocate development time** based on operational needs
5. **Start implementation** with chosen priorities

---

## References

- [Ratatui Documentation](https://ratatui.rs/)
- [React Documentation](https://react.dev/)
- [Leptos Book](https://leptos-rs.github.io/leptos/)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [utoipa Documentation](https://github.com/juhaku/utoipa)
- [MIL-STD-2525 Symbology](https://www.milsymbol.com/)

---

**Document Version:** 1.0
**Author:** Claude (AI Assistant)
**Last Updated:** October 27, 2025
