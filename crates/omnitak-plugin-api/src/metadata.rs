use serde::{Deserialize, Serialize};

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Plugin capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PluginCapability {
    /// Can filter CoT messages
    Filter,
    /// Can transform CoT messages
    Transform,
    /// Requires network access
    NetworkAccess,
    /// Requires filesystem access
    FilesystemAccess,
}

/// Complete plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub capabilities: Vec<PluginCapability>,
    /// SHA-256 hash of the plugin binary
    pub binary_hash: String,
}

/// Plugin metadata (filter-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct FilterMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    /// Maximum expected execution time in microseconds
    pub max_execution_time_us: u64,
}

/// Plugin metadata (transformer-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct TransformerMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    /// Supported CoT types (glob patterns)
    pub supported_types: Vec<String>,
}

/// Generic plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PluginMetadata {
    Filter(FilterMetadata),
    Transformer(TransformerMetadata),
}

impl PluginMetadata {
    pub fn id(&self) -> &str {
        match self {
            PluginMetadata::Filter(m) => &m.id,
            PluginMetadata::Transformer(m) => &m.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            PluginMetadata::Filter(m) => &m.name,
            PluginMetadata::Transformer(m) => &m.name,
        }
    }

    pub fn version(&self) -> &str {
        match self {
            PluginMetadata::Filter(m) => &m.version,
            PluginMetadata::Transformer(m) => &m.version,
        }
    }
}
