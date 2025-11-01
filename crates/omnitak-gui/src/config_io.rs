//! Configuration import/export functionality.

use omnitak_core::types::ServerConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    /// Server configurations
    pub servers: Vec<ServerConfig>,
}

impl ConfigFile {
    /// Creates a new configuration file
    pub fn new(servers: Vec<ServerConfig>) -> Self {
        Self { servers }
    }

    /// Loads configuration from a YAML file
    pub fn load_from_yaml<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: ConfigFile = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    /// Saves configuration to a YAML file
    pub fn save_to_yaml<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Loads configuration from a JSON file
    pub fn load_from_json<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: ConfigFile = serde_json::from_str(&contents)?;
        Ok(config)
    }

    /// Saves configuration to a JSON file
    pub fn save_to_json<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Validates all server configurations
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let errors: Vec<String> = self
            .servers
            .iter()
            .enumerate()
            .filter_map(|(idx, server)| {
                server.validate().err().map(|e| format!("Server {}: {}", idx, e))
            })
            .collect();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Configuration format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Yaml,
    Json,
}

impl ConfigFormat {
    /// Returns the file extension for this format
    pub fn extension(&self) -> &str {
        match self {
            ConfigFormat::Yaml => "yaml",
            ConfigFormat::Json => "json",
        }
    }

    /// Detects format from file extension
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "yaml" | "yml" => Some(ConfigFormat::Yaml),
                "json" => Some(ConfigFormat::Json),
                _ => None,
            })
    }
}

/// Imports configuration from a file
pub fn import_config<P: AsRef<Path>>(path: P) -> anyhow::Result<ConfigFile> {
    let format = ConfigFormat::from_path(&path)
        .ok_or_else(|| anyhow::anyhow!("Unsupported file format"))?;

    match format {
        ConfigFormat::Yaml => ConfigFile::load_from_yaml(path),
        ConfigFormat::Json => ConfigFile::load_from_json(path),
    }
}

/// Exports configuration to a file
pub fn export_config<P: AsRef<Path>>(config: &ConfigFile, path: P) -> anyhow::Result<()> {
    let format = ConfigFormat::from_path(&path)
        .ok_or_else(|| anyhow::anyhow!("Unsupported file format"))?;

    match format {
        ConfigFormat::Yaml => config.save_to_yaml(path),
        ConfigFormat::Json => config.save_to_json(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omnitak_core::types::Protocol;

    #[test]
    fn test_config_format_detection() {
        assert_eq!(ConfigFormat::from_path("config.yaml"), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::from_path("config.yml"), Some(ConfigFormat::Yaml));
        assert_eq!(ConfigFormat::from_path("config.json"), Some(ConfigFormat::Json));
        assert_eq!(ConfigFormat::from_path("config.txt"), None);
    }

    #[test]
    fn test_config_validation() {
        let config = ConfigFile::new(vec![
            ServerConfig::builder()
                .name("test")
                .host("localhost")
                .port(8089)
                .protocol(Protocol::Tcp)
                .build(),
        ]);

        assert!(config.validate().is_ok());

        let invalid_config = ConfigFile::new(vec![
            ServerConfig::builder()
                .name("")
                .host("localhost")
                .port(8089)
                .protocol(Protocol::Tcp)
                .build(),
        ]);

        assert!(invalid_config.validate().is_err());
    }
}
