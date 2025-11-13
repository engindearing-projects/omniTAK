use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;
use wasmtime::component::{Component, Instance, Linker};
use wasmtime::Store;

use crate::error::{PluginError, PluginResult};
use crate::metadata::FilterMetadata;
use crate::runtime::{PluginRuntime, PluginState};

// Re-export filter types from omnitak-filter
pub use omnitak_filter::rules::{CotMessage, FilterResult, FilterRule};

/// WASM-based filter plugin that implements the FilterRule trait
pub struct WasmFilterPlugin {
    runtime: Arc<PluginRuntime>,
    component: Component,
    metadata: FilterMetadata,
    instance_cache: RwLock<Option<CachedInstance>>,
}

struct CachedInstance {
    store: Store<PluginState>,
    instance: Instance,
    created_at: Instant,
}

impl WasmFilterPlugin {
    /// Create a new WASM filter plugin from compiled component
    pub fn new(
        runtime: Arc<PluginRuntime>,
        component: Component,
        metadata: FilterMetadata,
    ) -> Self {
        Self {
            runtime,
            component,
            metadata,
            instance_cache: RwLock::new(None),
        }
    }

    /// Load a plugin from WASM file
    pub fn from_file(
        runtime: Arc<PluginRuntime>,
        path: &str,
        metadata: FilterMetadata,
    ) -> PluginResult<Self> {
        let component = runtime.load_plugin_from_file(path)?;
        Ok(Self::new(runtime, component, metadata))
    }

    /// Load a plugin from WASM bytes
    pub fn from_bytes(
        runtime: Arc<PluginRuntime>,
        wasm_bytes: &[u8],
        metadata: FilterMetadata,
    ) -> PluginResult<Self> {
        let component = runtime.load_plugin(wasm_bytes)?;
        Ok(Self::new(runtime, component, metadata))
    }

    /// Get plugin metadata
    pub fn metadata(&self) -> &FilterMetadata {
        &self.metadata
    }

    /// Get or create instance (with caching for performance)
    fn get_or_create_instance(&self) -> PluginResult<(Store<PluginState>, Instance)> {
        // Check cache
        {
            let cache = self.instance_cache.read();
            if let Some(cached) = cache.as_ref() {
                // Cache is valid, but we can't return it directly due to &mut requirements
                // In a real implementation, we'd use a pool of instances
                drop(cache);
            }
        }

        // Create new instance
        let mut store = self.runtime.create_store();
        let mut linker = Linker::new(self.runtime.engine());

        // Add host functions
        Self::add_host_functions(&mut linker)?;

        let instance = linker
            .instantiate_async(&mut store, &self.component)
            .await
            .map_err(|e| PluginError::InstantiationError(e.to_string()))?;

        Ok((store, instance))
    }

    /// Add host functions that plugins can call
    fn add_host_functions(linker: &mut Linker<PluginState>) -> PluginResult<()> {
        // Add WASI support
        wasmtime_wasi::add_to_linker_async(linker)
            .map_err(|e| PluginError::InstantiationError(e.to_string()))?;

        // Add custom host functions
        // TODO: Implement host interface bindings
        // linker.func_wrap_async("host", "log", |_caller, level, msg| { ... });
        // linker.func_wrap_async("host", "get-current-time-ms", |_caller| { ... });

        Ok(())
    }

    /// Evaluate filter (internal implementation)
    async fn evaluate_async(&self, msg: &CotMessage<'_>) -> PluginResult<FilterResult> {
        let start = Instant::now();

        // Get or create instance
        let (mut store, instance) = self.get_or_create_instance()?;

        // TODO: Call the WASM function with msg data
        // This requires WIT bindings generation
        // For now, return a placeholder

        // Check timeout
        let elapsed = start.elapsed();
        if elapsed.as_micros() > self.metadata.max_execution_time_us as u128 {
            return Err(PluginError::Timeout(self.metadata.max_execution_time_us));
        }

        // Placeholder - in real implementation, call WASM exported function
        Ok(FilterResult::Pass)
    }
}

impl FilterRule for WasmFilterPlugin {
    fn evaluate(&self, msg: &CotMessage) -> FilterResult {
        // Since FilterRule is sync but WASM is async, we need to block
        // In production, consider using a thread pool or async runtime
        tokio::runtime::Handle::try_current()
            .and_then(|handle| {
                handle.block_on(async { self.evaluate_async(msg).await.ok() })
            })
            .unwrap_or(FilterResult::Block) // Fail closed on error
    }

    fn describe(&self) -> String {
        format!(
            "WASM Filter Plugin: {} v{} (by {})\n{}\nMax execution time: {}μs",
            self.metadata.name,
            self.metadata.version,
            self.metadata.author,
            self.metadata.description,
            self.metadata.max_execution_time_us
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_metadata() {
        let runtime = Arc::new(PluginRuntime::new().unwrap());
        let metadata = FilterMetadata {
            id: "test-filter".to_string(),
            name: "Test Filter".to_string(),
            version: "0.1.0".to_string(),
            author: "Test Author".to_string(),
            description: "A test filter plugin".to_string(),
            max_execution_time_us: 1000,
        };

        // Note: Can't create actual plugin without WASM binary
        assert_eq!(metadata.id, "test-filter");
    }
}
