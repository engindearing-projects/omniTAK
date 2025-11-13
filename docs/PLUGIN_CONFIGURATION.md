# Plugin Configuration Guide

This guide explains how to configure and use the OmniTAK plugin system through the configuration file.

## Overview

The OmniTAK plugin system allows you to extend the application with WebAssembly (WASM) plugins. The configuration system provides a declarative way to:

- Define plugin locations and settings
- Configure resource limits and security policies
- Load plugins automatically at startup
- Pass plugin-specific configuration

## Configuration Structure

### Main Plugin Section

```yaml
plugins:
  # Base directory for plugin files
  plugin_dir: "./plugins"

  # Enable hot-reload (development only)
  hot_reload: false

  # Resource limits
  resource_limits:
    max_execution_time_ms: 10000      # 10 seconds
    max_memory_bytes: 52428800         # 50 MB
    max_concurrent_executions: 100

  # Sandbox policy
  sandbox_policy:
    allow_network: false
    allow_filesystem_read: false
    allow_filesystem_write: false
    allow_env_vars: false
    allowed_paths: []

  # Plugin definitions
  filters: []
  transformers: []
```

### Resource Limits

Control how much resources each plugin can consume:

- `max_execution_time_ms`: Maximum time a plugin can execute per call (milliseconds)
- `max_memory_bytes`: Maximum memory a plugin instance can allocate (bytes)
- `max_concurrent_executions`: Maximum number of concurrent plugin executions

### Sandbox Policy

Security policy controlling what plugins can access:

- `allow_network`: Allow plugins to make network requests
- `allow_filesystem_read`: Allow plugins to read files
- `allow_filesystem_write`: Allow plugins to write files
- `allow_env_vars`: Allow plugins to read environment variables
- `allowed_paths`: Specific filesystem paths accessible (if filesystem access enabled)

**Security Note**: By default, all permissions are disabled for maximum security.

## Filter Plugins

Filter plugins process TAK messages and decide whether to accept or reject them.

### Example: Geographic Fence Filter

```yaml
plugins:
  filters:
    - id: geofence-filter
      path: geofence_filter.wasm
      enabled: true
      config:
        description: "Only allow messages within operational area"
        min_lat: 35.1
        max_lat: 35.3
        min_lon: -79.1
        max_lon: -78.9
```

### Example: Affiliation Filter

```yaml
plugins:
  filters:
    - id: affiliation-filter
      path: affiliation_filter.wasm
      enabled: true
      config:
        description: "Filter by force affiliation"
        allowed_affiliations: ["friend", "assumed_friend", "neutral"]
        block_hostile: true
```

## Transformer Plugins

Transformer plugins enrich or transform TAK messages.

### Example: FlightRadar24 Source

```yaml
plugins:
  # Enable network access for this plugin
  sandbox_policy:
    allow_network: true

  transformers:
    - id: flightradar24-source
      path: flightradar24_source.wasm
      enabled: true
      config:
        description: "Import aircraft from FlightRadar24 API"
        center_lat: 35.0
        center_lon: -79.0
        radius_degrees: 2.0
        update_interval_secs: 30
        friendly_icon: true
        min_altitude_ft: 0
```

### Example: Position Enrichment

```yaml
plugins:
  # Allow read-only filesystem access
  sandbox_policy:
    allow_filesystem_read: true
    allowed_paths:
      - "/data/terrain"

  transformers:
    - id: position-enricher
      path: position_enricher.wasm
      enabled: true
      config:
        description: "Enrich positions with terrain data"
        add_altitude_msl: true
        add_speed: true
        add_bearing: true
        terrain_data_path: "/data/terrain"
```

## Loading Plugins at Runtime

### Using the Configuration in Code

```rust
use omnitak_core::config::AppConfig;
use omnitak_plugin_api::{PluginManager, PluginManagerConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from file
    let config = AppConfig::from_file("config.yaml")?;
    config.validate()?;

    // Create plugin manager configuration
    let plugin_manager_config = PluginManagerConfig {
        plugin_dir: config.plugins.plugin_dir.clone(),
        hot_reload: config.plugins.hot_reload,
        resource_limits: omnitak_plugin_api::security::ResourceLimits {
            max_execution_time: config.plugins.resource_limits.max_execution_time(),
            max_memory_bytes: config.plugins.resource_limits.max_memory_bytes,
            max_concurrent_executions: config.plugins.resource_limits.max_concurrent_executions,
        },
        sandbox_policy: omnitak_plugin_api::security::SandboxPolicy {
            allow_network: config.plugins.sandbox_policy.allow_network,
            allow_filesystem_read: config.plugins.sandbox_policy.allow_filesystem_read,
            allow_filesystem_write: config.plugins.sandbox_policy.allow_filesystem_write,
            allow_env_vars: config.plugins.sandbox_policy.allow_env_vars,
            allowed_paths: config.plugins.sandbox_policy.allowed_paths.clone(),
        },
    };

    // Create plugin manager
    let plugin_manager = Arc::new(RwLock::new(
        PluginManager::new(plugin_manager_config)?
    ));

    // Load filter plugins from configuration
    {
        let manager = plugin_manager.write().await;
        for filter_config in &config.plugins.filters {
            if !filter_config.enabled {
                continue;
            }

            let plugin_path = config.plugins.get_plugin_path(&filter_config.path);
            let metadata = omnitak_plugin_api::metadata::FilterMetadata {
                id: filter_config.id.clone(),
                name: filter_config.id.clone(),
                version: "1.0.0".to_string(),
                author: "Unknown".to_string(),
                description: "Filter plugin".to_string(),
                config: filter_config.config.clone(),
            };

            manager.load_filter_plugin(
                plugin_path.to_str().unwrap(),
                metadata
            )?;

            tracing::info!("Loaded filter plugin: {}", filter_config.id);
        }
    }

    // Load transformer plugins from configuration
    {
        let manager = plugin_manager.write().await;
        for transformer_config in &config.plugins.transformers {
            if !transformer_config.enabled {
                continue;
            }

            let plugin_path = config.plugins.get_plugin_path(&transformer_config.path);
            let metadata = omnitak_plugin_api::metadata::TransformerMetadata {
                id: transformer_config.id.clone(),
                name: transformer_config.id.clone(),
                version: "1.0.0".to_string(),
                author: "Unknown".to_string(),
                description: "Transformer plugin".to_string(),
                config: transformer_config.config.clone(),
            };

            manager.load_transformer_plugin(
                plugin_path.to_str().unwrap(),
                metadata
            )?;

            tracing::info!("Loaded transformer plugin: {}", transformer_config.id);
        }
    }

    Ok(())
}
```

## Path Resolution

Plugin paths can be:

1. **Relative**: Resolved relative to `plugin_dir`
   ```yaml
   path: geofence_filter.wasm  # Becomes ./plugins/geofence_filter.wasm
   ```

2. **Absolute**: Used as-is
   ```yaml
   path: /opt/omnitak/plugins/geofence_filter.wasm
   ```

## Environment Variables

You can use environment variables in configuration values (if your config loader supports it):

```yaml
plugins:
  transformers:
    - id: flightradar24-source
      path: flightradar24_source.wasm
      enabled: true
      config:
        api_key: "${FR24_API_KEY}"  # From environment
```

## Plugin Configuration Schema

Each plugin receives its configuration through the `config` field, which is a flexible JSON object:

```yaml
config:
  # Simple values
  enabled: true
  threshold: 100

  # Nested objects
  bounds:
    min_lat: 35.0
    max_lat: 36.0

  # Arrays
  allowed_types: ["a-f-G", "a-f-A"]
```

The plugin implementation determines what configuration it expects.

## Validation

The configuration system validates:

- Plugin IDs are unique across all plugins
- Plugin IDs and paths are not empty
- No duplicate plugin definitions
- Plugin directory is accessible (warning if missing)

## Best Practices

1. **Use descriptive plugin IDs**: Choose IDs that clearly identify the plugin's purpose
2. **Document plugin configurations**: Add comments explaining what each plugin does
3. **Start with strict security**: Only grant permissions that plugins actually need
4. **Set reasonable resource limits**: Prevent plugins from consuming excessive resources
5. **Disable unused plugins**: Set `enabled: false` for plugins you're not using
6. **Use hot-reload only in development**: Set `hot_reload: false` in production
7. **Version your configurations**: Keep configuration in version control
8. **Test plugins individually**: Enable one plugin at a time when troubleshooting

## Troubleshooting

### Plugin fails to load

- Check that the plugin file exists at the specified path
- Verify the plugin file is a valid WASM file
- Check file permissions
- Look at logs for detailed error messages

### Plugin execution times out

- Increase `max_execution_time_ms` if the plugin legitimately needs more time
- Check if the plugin is stuck in an infinite loop

### Plugin needs network/filesystem access

- Update `sandbox_policy` to grant required permissions
- Be specific with `allowed_paths` to maintain security

### Duplicate plugin ID error

- Ensure all plugin IDs are unique
- Check both `filters` and `transformers` sections

## Next Steps

- [Plugin Development Guide](PLUGIN_DEVELOPMENT.md) - Learn to build your own plugins
- [Plugin API Reference](PLUGIN_API.md) - Detailed API documentation
- [Plugin Quickstart](PLUGIN_QUICKSTART.md) - Build your first plugin
- [Plugin GUI Integration](PLUGIN_GUI_INTEGRATION.md) - Manage plugins through the GUI
