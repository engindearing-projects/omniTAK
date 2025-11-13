# Plugin System GUI and API Integration Guide

Guide for integrating the plugin system into the OmniTAK application.

## Overview

This guide covers:
1. Adding plugin API routes to the REST server
2. Integrating the plugin manager panel into the GUI
3. Connecting the GUI to the backend API
4. Testing the complete integration

## Part 1: API Integration

### Step 1: Update omnitak-api Dependencies

Add plugin API dependency to `crates/omnitak-api/Cargo.toml`:

```toml
[dependencies]
omnitak-plugin-api = { path = "../omnitak-plugin-api" }
```

### Step 2: Add Plugin Module to REST API

Update `crates/omnitak-api/src/rest.rs`:

```rust
pub mod plugins;
use plugins::{create_plugin_router, PluginApiState};

// In create_rest_router function:
pub fn create_rest_router(state: ApiState, plugin_state: PluginApiState) -> Router {
    Router::new()
        // Existing routes...
        .route("/api/v1/status", get(get_system_status))
        // ... other routes ...

        // Merge plugin routes
        .merge(create_plugin_router(plugin_state))

        .with_state(state)
}
```

### Step 3: Initialize Plugin Manager in Main App

Update `src/main_integrated.rs`:

```rust
use omnitak_plugin_api::{PluginManager, PluginManagerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config
    let config = load_config()?;

    // Initialize plugin manager
    let plugin_config = PluginManagerConfig {
        resource_limits: config.plugins.resource_limits.clone(),
        sandbox_policy: config.plugins.sandbox_policy.clone(),
        plugin_dir: config.plugins.plugin_dir.clone(),
        hot_reload: config.plugins.hot_reload,
    };

    let plugin_manager = Arc::new(RwLock::new(
        PluginManager::new(plugin_config)?
    ));

    // Load plugins from config
    {
        let manager = plugin_manager.write().await;
        manager.load_all_plugins().await?;
    }

    // Create plugin API state
    let plugin_state = PluginApiState {
        plugin_manager: plugin_manager.clone(),
        audit_logger: audit_logger.clone(),
    };

    // Create API server with plugin routes
    let api_router = create_rest_router(api_state, plugin_state);

    // Start server
    // ...
}
```

### Step 4: Add Plugin Configuration Schema

Update `crates/omnitak-core/src/config.rs`:

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct PluginConfig {
    pub plugin_dir: String,
    pub hot_reload: bool,
    pub resource_limits: ResourceLimits,
    pub sandbox_policy: SandboxPolicy,
    pub filters: Vec<FilterPluginConfig>,
    pub transformers: Vec<TransformerPluginConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FilterPluginConfig {
    pub id: String,
    pub path: String,
    pub enabled: bool,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransformerPluginConfig {
    pub id: String,
    pub path: String,
    pub enabled: bool,
    #[serde(default)]
    pub config: serde_json::Value,
}
```

## Part 2: GUI Integration

### Step 1: Add Plugin Module to GUI

Update `crates/omnitak-gui/src/ui/mod.rs`:

```rust
pub mod connections;
pub mod dashboard;
pub mod messages;
pub mod plugins;  // Add this line
pub mod server_dialog;
pub mod settings;

pub use plugins::PluginPanelState;
```

### Step 2: Add Plugins Tab

Update `crates/omnitak-gui/src/lib.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Connections,
    Messages,
    Plugins,      // Add this line
    Settings,
}

// Update UiState
pub struct UiState {
    pub selected_tab: Tab,
    pub server_dialog: Option<ServerDialogState>,
    pub message_filter: String,
    pub affiliation_filter: AffiliationFilter,
    pub server_filter: String,
    pub auto_scroll: bool,
    pub message_details_dialog: Option<MessageLog>,
    pub expanded_messages: std::collections::HashSet<String>,
    pub plugin_panel: PluginPanelState,  // Add this line
}
```

### Step 3: Wire Up Plugin Panel in Main UI

Update the main UI rendering code:

```rust
impl eframe::App for OmniTakApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel with tabs
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.ui_state.selected_tab, Tab::Dashboard, "Dashboard");
                ui.selectable_value(&mut self.ui_state.selected_tab, Tab::Connections, "Connections");
                ui.selectable_value(&mut self.ui_state.selected_tab, Tab::Messages, "Messages");
                ui.selectable_value(&mut self.ui_state.selected_tab, Tab::Plugins, "Plugins");  // Add this
                ui.selectable_value(&mut self.ui_state.selected_tab, Tab::Settings, "Settings");
            });
        });

        // Central panel with content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.ui_state.selected_tab {
                Tab::Dashboard => {
                    if let Some(msg) = ui::dashboard::render_dashboard(ui, &self.state) {
                        self.set_status_message(msg.0, msg.1);
                    }
                }
                Tab::Connections => {
                    if let Some(msg) = ui::connections::render_connections_panel(
                        ui,
                        &self.state,
                        &mut self.ui_state.server_dialog,
                    ) {
                        self.set_status_message(msg.0, msg.1);
                    }
                }
                Tab::Messages => {
                    if let Some(msg) = ui::messages::render_messages_panel(
                        ui,
                        &self.state,
                        &mut self.ui_state,
                    ) {
                        self.set_status_message(msg.0, msg.1);
                    }
                }
                Tab::Plugins => {  // Add this case
                    if let Some(msg) = ui::plugins::render_plugins_panel(
                        ui,
                        &self.state,
                        &mut self.ui_state.plugin_panel,
                    ) {
                        self.set_status_message(msg.0, msg.1);
                    }
                }
                Tab::Settings => {
                    if let Some(msg) = ui::settings::render_settings_panel(ui, &self.state) {
                        self.set_status_message(msg.0, msg.1);
                    }
                }
            }
        });

        // Status bar at bottom
        // ...
    }
}
```

### Step 4: Add Plugin API Client

Create `crates/omnitak-gui/src/plugin_client.rs`:

```rust
//! Plugin API client for the GUI

use anyhow::Result;
use omnitak_plugin_api::PluginInfo;
use serde::{Deserialize, Serialize};

pub struct PluginClient {
    base_url: String,
    client: reqwest::Client,
}

impl PluginClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn list_plugins(&self) -> Result<Vec<PluginInfo>> {
        let url = format!("{}/api/v1/plugins", self.base_url);
        let response: PluginListResponse = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(response.plugins)
    }

    pub async fn load_plugin(
        &self,
        id: String,
        path: String,
        plugin_type: String,
        enabled: bool,
        config: serde_json::Value,
    ) -> Result<PluginInfo> {
        let url = format!("{}/api/v1/plugins", self.base_url);
        let request = LoadPluginRequest {
            id,
            path,
            enabled,
            plugin_type,
            config,
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn unload_plugin(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/plugins/{}", self.base_url, id);
        self.client
            .delete(&url)
            .send()
            .await?;

        Ok(())
    }

    pub async fn toggle_plugin(&self, id: &str, enabled: bool) -> Result<()> {
        let url = format!("{}/api/v1/plugins/{}/toggle", self.base_url, id);
        let request = TogglePluginRequest { enabled };

        self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        Ok(())
    }

    pub async fn update_config(
        &self,
        id: &str,
        config: serde_json::Value,
    ) -> Result<()> {
        let url = format!("{}/api/v1/plugins/{}/config", self.base_url, id);
        let request = UpdatePluginConfigRequest { config };

        self.client
            .put(&url)
            .json(&request)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_metrics(&self, id: &str) -> Result<PluginMetrics> {
        let url = format!("{}/api/v1/plugins/{}/metrics", self.base_url, id);
        let response = self.client
            .get(&url)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn reload_plugin(&self, id: &str) -> Result<()> {
        let url = format!("{}/api/v1/plugins/{}/reload", self.base_url, id);
        self.client
            .post(&url)
            .send()
            .await?;

        Ok(())
    }
}

// Request/Response types
#[derive(Serialize)]
struct LoadPluginRequest {
    id: String,
    path: String,
    enabled: bool,
    plugin_type: String,
    config: serde_json::Value,
}

#[derive(Serialize)]
struct TogglePluginRequest {
    enabled: bool,
}

#[derive(Serialize)]
struct UpdatePluginConfigRequest {
    config: serde_json::Value,
}

#[derive(Deserialize)]
struct PluginListResponse {
    plugins: Vec<PluginInfo>,
    total: usize,
}

#[derive(Deserialize)]
pub struct PluginMetrics {
    pub plugin_id: String,
    pub execution_count: u64,
    pub error_count: u64,
    pub timeout_count: u64,
    pub avg_execution_time_ms: f64,
    pub p50_execution_time_ms: f64,
    pub p95_execution_time_ms: f64,
    pub p99_execution_time_ms: f64,
    pub last_execution: Option<String>,
    pub last_error: Option<String>,
}
```

### Step 5: Connect GUI to API

Update `crates/omnitak-gui/src/ui/plugins.rs` to use the client:

```rust
use crate::plugin_client::PluginClient;

// In render_plugins_panel, replace mock data with API calls:
async fn load_plugins(client: &PluginClient) -> Result<Vec<PluginInfo>> {
    client.list_plugins().await
}

// Call from the UI:
// (Use poll-promise for async in egui)
use poll_promise::Promise;

pub struct PluginPanelState {
    // ... existing fields ...
    plugins_promise: Option<Promise<Result<Vec<PluginInfo>>>>,
    plugin_client: Arc<PluginClient>,
}

// In render function:
if panel_state.plugins_promise.is_none() {
    let client = panel_state.plugin_client.clone();
    panel_state.plugins_promise = Some(Promise::spawn_async(async move {
        client.list_plugins().await
    }));
}

if let Some(promise) = &panel_state.plugins_promise {
    match promise.ready() {
        Some(Ok(plugins)) => {
            // Render plugins
        }
        Some(Err(e)) => {
            ui.label(format!("Error loading plugins: {}", e));
        }
        None => {
            ui.spinner(); // Loading
        }
    }
}
```

## Part 3: Testing the Integration

### Test 1: API Endpoints

```bash
# Start the server
cargo run

# Test plugin listing
curl http://localhost:9443/api/v1/plugins

# Test loading a plugin
curl -X POST http://localhost:9443/api/v1/plugins \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-plugin",
    "path": "plugins/test.wasm",
    "enabled": true,
    "pluginType": "filter",
    "config": {}
  }'
```

### Test 2: GUI Integration

```bash
# Start the GUI
cargo run --bin omnitak-gui

# Navigate to Plugins tab
# Verify:
# - Plugins list loads
# - Can click "Load Plugin" button
# - Can toggle plugins on/off
# - Can view metrics
# - Can configure plugins
```

### Test 3: End-to-End

```bash
# 1. Build a plugin
cd examples/plugins/flightradar24-source
./build.sh

# 2. Start OmniTAK
cd ../../..
cargo run

# 3. Load plugin via API
curl -X POST http://localhost:9443/api/v1/plugins \
  -H "Content-Type: application/json" \
  -d @load-fr24-plugin.json

# 4. Verify in GUI
cargo run --bin omnitak-gui
# Check Plugins tab shows the loaded plugin

# 5. Monitor metrics
curl http://localhost:9443/api/v1/plugins/flightradar24-source/metrics
```

## Part 4: Configuration Example

Complete `config.yaml` with plugins:

```yaml
application:
  max_connections: 1000
  worker_threads: 8

# Plugin system configuration
plugins:
  plugin_dir: "./plugins"
  hot_reload: false

  resource_limits:
    max_execution_time_ms: 10000
    max_memory_bytes: 52428800
    max_concurrent_executions: 100

  sandbox_policy:
    allow_network: false
    allow_filesystem_read: false
    allow_filesystem_write: false
    allow_env_vars: false
    allowed_paths: []

  # Filter plugins
  filters:
    - id: geofence-filter
      path: plugins/geofence_filter.wasm
      enabled: true
      config:
        min_lat: 35.1
        max_lat: 35.3
        min_lon: -79.1
        max_lon: -78.9

  # Transformer plugins
  transformers:
    - id: flightradar24-source
      path: plugins/flightradar24_source.wasm
      enabled: true
      config:
        center_lat: 35.0
        center_lon: -79.0
        radius_degrees: 2.0
        update_interval_secs: 30
        enabled: true

# TAK server connections
servers:
  - id: local-server
    address: "localhost:8087"
    protocol: tcp

# API settings
api:
  bind_addr: "0.0.0.0:9443"
  enable_tls: false
```

## Troubleshooting

### Issue: GUI not showing plugins

**Check:**
1. Is the API server running?
2. Is the GUI pointing to correct API URL?
3. Check browser console for errors
4. Verify authentication token is valid

### Issue: Plugin load fails

**Check:**
1. Plugin file exists at specified path
2. Plugin has correct format (.wasm)
3. Plugin metadata is valid
4. Sandbox policy allows required capabilities

### Issue: Metrics not updating

**Check:**
1. Plugin is actually executing
2. Metrics collection is enabled
3. WebSocket connection is active
4. Browser has network access

## Next Steps

1. Add WebSocket streaming for real-time updates
2. Implement plugin marketplace
3. Add drag-and-drop plugin installation
4. Create plugin dependency management
5. Add plugin versioning and updates

---

For more information:
- [Plugin Development Guide](PLUGIN_DEVELOPMENT.md)
- [Plugin API Reference](PLUGIN_API.md)
- [GUI Development Guide](GUI_DEVELOPMENT.md)
