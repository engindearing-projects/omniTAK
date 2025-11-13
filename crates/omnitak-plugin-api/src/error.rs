use thiserror::Error;

/// Plugin system error types
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Failed to load plugin: {0}")]
    LoadError(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin execution timeout (exceeded {0}μs)")]
    Timeout(u64),

    #[error("Plugin memory limit exceeded: {current} > {limit} bytes")]
    MemoryLimitExceeded { current: u64, limit: u64 },

    #[error("Plugin execution error: {0}")]
    ExecutionError(String),

    #[error("Plugin compilation error: {0}")]
    CompilationError(String),

    #[error("Plugin instantiation error: {0}")]
    InstantiationError(String),

    #[error("Invalid plugin metadata: {0}")]
    InvalidMetadata(String),

    #[error("Security violation: {0}")]
    SecurityViolation(String),

    #[error("Unsupported plugin capability: {0}")]
    UnsupportedCapability(String),

    #[error("Plugin signature verification failed")]
    SignatureVerificationFailed,

    #[error("WASM runtime error: {0}")]
    WasmError(#[from] wasmtime::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type PluginResult<T> = Result<T, PluginError>;
