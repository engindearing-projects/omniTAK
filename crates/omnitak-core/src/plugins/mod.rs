//! Plugin system for OmniTAK.
//!
//! This module provides a WASM-based plugin system for extending OmniTAK's
//! message filtering and processing capabilities. Plugins are loaded dynamically
//! at runtime and can inspect, modify, or drop CoT messages.
//!
//! # Architecture
//!
//! The plugin system uses WebAssembly Component Model and Wasmtime for safe,
//! sandboxed plugin execution:
//!
//! - **WIT Interface**: Defines the contract between host and plugins
//! - **Host Implementation**: Exposes OmniTAK functionality to plugins
//! - **Plugin Loader**: Manages plugin lifecycle and execution
//! - **Bindings**: Auto-generated type-safe bindings from WIT
//!
//! # Plugin Interface
//!
//! Plugins must implement the `message-filter` interface defined in
//! `wit/omnitak-plugins.wit`. This includes:
//!
//! - `filter-message`: Process a CoT message (pass/modify/drop)
//! - `initialize`: Called once when the plugin is loaded
//! - `shutdown`: Called when the plugin is unloaded
//! - `get-info`: Return plugin metadata
//!
//! # Host Functions
//!
//! Plugins can call back to the host to:
//!
//! - Log messages at different severity levels
//! - Retrieve configuration values
//! - Query server information
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use omnitak_core::plugins::{PluginManager, MessageMetadata};
//! use std::collections::HashMap;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a plugin manager
//! let config = HashMap::new();
//! let servers = vec![];
//! let mut manager = PluginManager::new("./plugins", config, servers);
//!
//! // Load all plugins from the directory
//! let count = manager.load_all()?;
//! println!("Loaded {} plugins", count);
//!
//! // Filter a message through all plugins
//! let xml = "<event>...</event>";
//! let metadata = MessageMetadata {
//!     server_id: "server-1".to_string(),
//!     server_name: "TAK Server".to_string(),
//!     received_at: 1234567890,
//!     source_addr: Some("192.168.1.100".to_string()),
//!     protocol: "TCP".to_string(),
//! };
//!
//! match manager.filter_message(xml, metadata)? {
//!     Some(filtered_xml) => println!("Message passed: {}", filtered_xml),
//!     None => println!("Message dropped by plugin"),
//! }
//!
//! // Reload a specific plugin
//! manager.reload_plugin("my-filter")?;
//!
//! // Shutdown all plugins
//! manager.shutdown_all()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Hot Reloading
//!
//! Plugins support hot-reloading without restarting the server:
//!
//! ```rust,no_run
//! # use omnitak_core::plugins::PluginManager;
//! # use std::collections::HashMap;
//! # fn example(mut manager: PluginManager) -> Result<(), Box<dyn std::error::Error>> {
//! // Reload all plugins
//! let count = manager.reload_all()?;
//! println!("Reloaded {} plugins", count);
//!
//! // Or reload a specific plugin
//! manager.reload_plugin("geo-filter")?;
//! # Ok(())
//! # }
//! ```
//!
//! # Error Handling
//!
//! Plugin operations return `OmniTAKError::PluginError` on failure. The error
//! message includes context about what operation failed and why. Plugins that
//! fail to load or execute are logged but don't crash the host process.

mod bindings;
mod host;
mod loader;

// Re-export public types
pub use loader::{
    FilterAction, FilterResult, MessageFilterPlugin, MessageMetadata, PluginInfo, PluginManager,
};

// Re-export host for advanced users who want to customize it
pub use host::PluginHost;
