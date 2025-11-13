use std::sync::Arc;
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::error::{PluginError, PluginResult};
use crate::security::{ResourceLimits, SandboxPolicy};

/// WASM plugin runtime environment
pub struct PluginRuntime {
    engine: Engine,
    resource_limits: ResourceLimits,
    sandbox_policy: SandboxPolicy,
}

impl PluginRuntime {
    /// Create a new plugin runtime with default configuration
    pub fn new() -> PluginResult<Self> {
        Self::with_config(ResourceLimits::default(), SandboxPolicy::default())
    }

    /// Create a plugin runtime with custom configuration
    pub fn with_config(
        resource_limits: ResourceLimits,
        sandbox_policy: SandboxPolicy,
    ) -> PluginResult<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);

        // Configure resource limits
        config.epoch_interruption(true);
        config.max_wasm_stack(1024 * 1024); // 1MB stack

        // Use Cranelift optimizer for best performance
        config.cranelift_opt_level(OptLevel::Speed);

        let engine = Engine::new(&config)
            .map_err(|e| PluginError::CompilationError(e.to_string()))?;

        Ok(Self {
            engine,
            resource_limits,
            sandbox_policy,
        })
    }

    /// Get the WASM engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Get resource limits
    pub fn resource_limits(&self) -> &ResourceLimits {
        &self.resource_limits
    }

    /// Get sandbox policy
    pub fn sandbox_policy(&self) -> &SandboxPolicy {
        &self.sandbox_policy
    }

    /// Create a new store with configured limits
    pub fn create_store(&self) -> Store<PluginState> {
        let wasi_ctx = self.create_wasi_context();
        let state = PluginState {
            wasi_ctx,
            limits: self.resource_limits.clone(),
        };
        let mut store = Store::new(self.engine(), state);

        // Set memory limits
        store.limiter(|state| &mut state.limits);

        store
    }

    /// Create WASI context based on sandbox policy
    fn create_wasi_context(&self) -> WasiCtx {
        let mut builder = WasiCtxBuilder::new();

        // Configure based on sandbox policy
        if self.sandbox_policy.allow_env_vars {
            builder = builder.inherit_env();
        }

        if self.sandbox_policy.allow_network {
            builder = builder.inherit_network();
        }

        // Add allowed filesystem paths
        for path in &self.sandbox_policy.allowed_paths {
            if self.sandbox_policy.allow_filesystem_write {
                // Read-write access
                builder = builder.preopened_dir(
                    wasmtime_wasi::Dir::open_ambient_dir(path, wasmtime_wasi::ambient_authority())
                        .unwrap_or_else(|_| {
                            tracing::warn!("Failed to open directory for plugin: {}", path);
                            wasmtime_wasi::Dir::open_ambient_dir(".", wasmtime_wasi::ambient_authority())
                                .unwrap()
                        }),
                    path,
                );
            } else if self.sandbox_policy.allow_filesystem_read {
                // Read-only access
                builder = builder.preopened_dir(
                    wasmtime_wasi::Dir::open_ambient_dir(path, wasmtime_wasi::ambient_authority())
                        .unwrap_or_else(|_| {
                            tracing::warn!("Failed to open directory for plugin: {}", path);
                            wasmtime_wasi::Dir::open_ambient_dir(".", wasmtime_wasi::ambient_authority())
                                .unwrap()
                        }),
                    path,
                );
            }
        }

        builder.build()
    }

    /// Load and compile a plugin from bytes
    pub fn load_plugin(&self, wasm_bytes: &[u8]) -> PluginResult<Component> {
        Component::from_binary(&self.engine, wasm_bytes)
            .map_err(|e| PluginError::LoadError(e.to_string()))
    }

    /// Load and compile a plugin from file
    pub fn load_plugin_from_file(&self, path: &str) -> PluginResult<Component> {
        let wasm_bytes = std::fs::read(path)?;
        self.load_plugin(&wasm_bytes)
    }
}

impl Default for PluginRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create default plugin runtime")
    }
}

/// Plugin execution state
pub struct PluginState {
    wasi_ctx: WasiCtx,
    limits: ResourceLimits,
}

impl WasiView for PluginState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }

    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        unimplemented!("Resource table not yet implemented")
    }
}

// Resource limiter implementation
impl wasmtime::ResourceLimiter for ResourceLimits {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        if desired as u64 > self.max_memory_bytes {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn table_growing(
        &mut self,
        _current: u32,
        desired: u32,
        _maximum: Option<u32>,
    ) -> anyhow::Result<bool> {
        // Allow reasonable table growth
        Ok(desired < 10000)
    }
}
