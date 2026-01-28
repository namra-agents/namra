//! Configuration file parser

use crate::{AgentConfig, WorkflowConfig};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Configuration file format
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    Yaml,
    Toml,
}

impl ConfigFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "toml" => Some(ConfigFormat::Toml),
            _ => None,
        }
    }
}

/// Configuration file parser
pub struct ConfigParser;

impl ConfigParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse an agent configuration from a file
    pub fn parse_agent<P: AsRef<Path>>(&self, path: P) -> Result<AgentConfig> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let format = self.detect_format(path)?;

        match format {
            ConfigFormat::Yaml => self.parse_agent_yaml(&content),
            ConfigFormat::Toml => self.parse_agent_toml(&content),
        }
    }

    /// Parse a workflow configuration from a file
    pub fn parse_workflow<P: AsRef<Path>>(&self, path: P) -> Result<WorkflowConfig> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read workflow file: {}", path.display()))?;

        let format = self.detect_format(path)?;

        match format {
            ConfigFormat::Yaml => self.parse_workflow_yaml(&content),
            ConfigFormat::Toml => self.parse_workflow_toml(&content),
        }
    }

    fn detect_format<P: AsRef<Path>>(&self, path: P) -> Result<ConfigFormat> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow::anyhow!("File has no extension"))?;

        ConfigFormat::from_extension(extension)
            .ok_or_else(|| anyhow::anyhow!("Unsupported file format: {}", extension))
    }

    fn parse_agent_yaml(&self, content: &str) -> Result<AgentConfig> {
        serde_yaml::from_str(content)
            .context("Failed to parse agent configuration from YAML")
    }

    fn parse_agent_toml(&self, content: &str) -> Result<AgentConfig> {
        toml::from_str(content)
            .context("Failed to parse agent configuration from TOML")
    }

    fn parse_workflow_yaml(&self, content: &str) -> Result<WorkflowConfig> {
        serde_yaml::from_str(content)
            .context("Failed to parse workflow configuration from YAML")
    }

    fn parse_workflow_toml(&self, content: &str) -> Result<WorkflowConfig> {
        toml::from_str(content)
            .context("Failed to parse workflow configuration from TOML")
    }
}

impl Default for ConfigParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_format() {
        assert!(matches!(
            ConfigFormat::from_extension("yaml"),
            Some(ConfigFormat::Yaml)
        ));
        assert!(matches!(
            ConfigFormat::from_extension("yml"),
            Some(ConfigFormat::Yaml)
        ));
        assert!(matches!(
            ConfigFormat::from_extension("toml"),
            Some(ConfigFormat::Toml)
        ));
        assert!(ConfigFormat::from_extension("json").is_none());
    }
}
