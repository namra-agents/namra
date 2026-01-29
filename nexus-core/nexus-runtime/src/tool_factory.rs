//! Tool factory for creating tool instances from configuration

use nexus_config::{AgentConfig, FileSystemBackend, FileSystemToolConfig, HttpToolConfig, ToolConfig};
use nexus_tools::{
    CalculatorTool, FileSystemTool, HttpTool, LocalBackend, StringTool, Tool,
    S3Backend, S3Config, GCSBackend, GCSConfig, AzureBackend, AzureConfig, SFTPBackend, SFTPConfig,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::error::{Result, RuntimeError};

/// Tool factory that creates tool instances from configuration
pub struct ToolFactory;

impl ToolFactory {
    /// Create a new tool factory
    pub fn new() -> Self {
        Self
    }

    /// Build all tools from agent configuration
    ///
    /// This creates both:
    /// 1. Built-in generic tools (calculator, string, http, filesystem)
    /// 2. Configured tools from the agent's tools section
    pub fn build_tools(&self, config: &AgentConfig) -> Result<HashMap<String, Arc<dyn Tool>>> {
        let mut tools: HashMap<String, Arc<dyn Tool>> = HashMap::new();

        // Add built-in generic tools (always available)
        tools.insert("calculator".to_string(), Arc::new(CalculatorTool::new()));
        tools.insert("string".to_string(), Arc::new(StringTool::new()));

        // Add configured tools from agent config
        for tool_config in &config.tools {
            let (name, tool) = self.build_tool_from_config(tool_config)?;
            tools.insert(name, tool);
        }

        Ok(tools)
    }

    /// Build a single tool from configuration
    fn build_tool_from_config(
        &self,
        tool_config: &ToolConfig,
    ) -> Result<(String, Arc<dyn Tool>)> {
        match tool_config {
            ToolConfig::BuiltinHttp { name, config } => {
                let tool = self.build_http_tool(config)?;
                Ok((name.clone(), Arc::new(tool)))
            }

            ToolConfig::BuiltinFilesystem { name, config } => {
                let tool = self.build_filesystem_tool(name, config)?;
                Ok((name.clone(), Arc::new(tool)))
            }

            ToolConfig::BuiltinDatabase { name, config: _ } => {
                // Database tool not yet implemented
                Err(RuntimeError::ConfigError(format!(
                    "Database tool '{}' not yet implemented",
                    name
                )))
            }

            ToolConfig::BuiltinVectorSearch { name, config: _ } => {
                // Vector search tool not yet implemented
                Err(RuntimeError::ConfigError(format!(
                    "Vector search tool '{}' not yet implemented",
                    name
                )))
            }

            ToolConfig::PluginPython { name, .. } => {
                // Python plugin not yet implemented
                Err(RuntimeError::ConfigError(format!(
                    "Python plugin tool '{}' not yet implemented (Week 7+)",
                    name
                )))
            }

            ToolConfig::Agent { name, .. } => {
                // Agent-as-tool not yet implemented
                Err(RuntimeError::ConfigError(format!(
                    "Agent-as-tool '{}' not yet implemented (Week 13+)",
                    name
                )))
            }
        }
    }

    /// Build a configured HTTP tool
    fn build_http_tool(&self, config: &HttpToolConfig) -> Result<ConfiguredHttpTool> {
        // Parse timeout
        let timeout = self.parse_timeout(&config.timeout)?;

        Ok(ConfiguredHttpTool {
            name: "http_configured".to_string(),
            base_url: config.url.clone(),
            method: config.method.clone(),
            headers: config.headers.clone(),
            timeout,
            http_tool: HttpTool::with_timeout(timeout),
        })
    }

    /// Build a configured filesystem tool
    fn build_filesystem_tool(
        &self,
        name: &str,
        config: &FileSystemToolConfig,
    ) -> Result<FileSystemTool> {
        let backend: Box<dyn nexus_tools::FileSystemBackend> = match &config.backend {
            FileSystemBackend::Local { base_dir } => {
                let base_path = PathBuf::from(base_dir);
                Box::new(LocalBackend::with_sandbox(base_path, config.read_only))
            }

            FileSystemBackend::S3 {
                bucket,
                region,
                prefix,
                credentials: _,
            } => {
                let s3_config = S3Config {
                    bucket: bucket.clone(),
                    region: region.clone(),
                    prefix: prefix.clone(),
                };
                Box::new(S3Backend::new(s3_config, config.read_only))
            }

            FileSystemBackend::GCS {
                bucket,
                project,
                prefix,
                credentials: _,
            } => {
                let gcs_config = GCSConfig {
                    bucket: bucket.clone(),
                    project: project.clone(),
                    prefix: prefix.clone(),
                };
                Box::new(GCSBackend::new(gcs_config, config.read_only))
            }

            FileSystemBackend::Azure {
                container,
                account,
                prefix,
                credentials: _,
            } => {
                let azure_config = AzureConfig {
                    container: container.clone(),
                    account: account.clone(),
                    prefix: prefix.clone(),
                };
                Box::new(AzureBackend::new(azure_config, config.read_only))
            }

            FileSystemBackend::SFTP {
                host,
                port,
                username,
                base_path,
                credentials: _,
            } => {
                let sftp_config = SFTPConfig {
                    host: host.clone(),
                    port: *port,
                    username: username.clone(),
                    base_path: base_path.clone(),
                };
                Box::new(SFTPBackend::new(sftp_config, config.read_only))
            }
        };

        Ok(FileSystemTool::new(name.to_string(), backend))
    }

    /// Parse timeout string like "30s" into Duration
    fn parse_timeout(&self, timeout_str: &str) -> Result<Duration> {
        let timeout_str = timeout_str.trim();
        // Check "ms" before "s" since "ms" ends with "s"
        if timeout_str.ends_with("ms") {
            let ms = timeout_str[..timeout_str.len() - 2]
                .parse::<u64>()
                .map_err(|e| RuntimeError::ConfigError(format!("Invalid timeout format: {}", e)))?;
            Ok(Duration::from_millis(ms))
        } else if timeout_str.ends_with('s') {
            let secs = timeout_str[..timeout_str.len() - 1]
                .parse::<u64>()
                .map_err(|e| RuntimeError::ConfigError(format!("Invalid timeout format: {}", e)))?;
            Ok(Duration::from_secs(secs))
        } else {
            // Assume seconds if no unit
            let secs = timeout_str
                .parse::<u64>()
                .map_err(|e| RuntimeError::ConfigError(format!("Invalid timeout format: {}", e)))?;
            Ok(Duration::from_secs(secs))
        }
    }
}

impl Default for ToolFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// A pre-configured HTTP tool that wraps the generic HTTP tool
///
/// This allows agents to define named API endpoints in their config
/// without repeating URL, headers, and other settings in each call.
pub struct ConfiguredHttpTool {
    name: String,
    base_url: String,
    method: String,
    headers: HashMap<String, String>,
    #[allow(dead_code)]
    timeout: Duration,
    http_tool: HttpTool,
}

use async_trait::async_trait;
use nexus_tools::ToolOutput;
use serde_json::{json, Value};

#[async_trait]
impl Tool for ConfiguredHttpTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Pre-configured HTTP API endpoint"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to append to base URL (optional)"
                },
                "query": {
                    "type": "object",
                    "description": "Query parameters (optional)"
                },
                "body": {
                    "type": "string",
                    "description": "Request body (optional)"
                }
            }
        })
    }

    async fn execute(&self, input: Value) -> nexus_tools::Result<ToolOutput> {
        // Start with base URL
        let mut url = self.base_url.clone();

        // Add path if provided
        if let Some(path) = input.get("path") {
            if let Some(path_str) = path.as_str() {
                // Ensure path starts with /
                if !path_str.starts_with('/') && !url.ends_with('/') {
                    url.push('/');
                }
                url.push_str(path_str);
            }
        }

        // Add query parameters if provided
        if let Some(query) = input.get("query") {
            if let Some(query_obj) = query.as_object() {
                if !query_obj.is_empty() {
                    url.push('?');
                    let params: Vec<String> = query_obj
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "{}={}",
                                k,
                                v.as_str().unwrap_or(&v.to_string())
                            )
                        })
                        .collect();
                    url.push_str(&params.join("&"));
                }
            }
        }

        // Build the full request
        let mut request = json!({
            "url": url,
            "method": self.method.clone(),
            "headers": self.headers.clone(),
        });

        // Add body if provided
        if let Some(body) = input.get("body") {
            request["body"] = body.clone();
        }

        // Execute using the underlying HTTP tool
        self.http_tool.execute(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        let factory = ToolFactory::new();
        assert!(true); // Factory can be created
    }

    #[test]
    fn test_parse_timeout() {
        let factory = ToolFactory::new();

        assert_eq!(
            factory.parse_timeout("30s").unwrap(),
            Duration::from_secs(30)
        );
        assert_eq!(
            factory.parse_timeout("1000ms").unwrap(),
            Duration::from_millis(1000)
        );
        assert_eq!(factory.parse_timeout("60").unwrap(), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_build_generic_tools() {
        let config = AgentConfig {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            metadata: HashMap::new(),
            llm: nexus_config::LLMConfig {
                provider: "anthropic".to_string(),
                model: "claude-sonnet-4-5-20250929".to_string(),
                temperature: 0.7,
                max_tokens: 4096,
                top_p: None,
                stream: true,
                retry: None,
            },
            tools: vec![],
            memory: None,
            middleware: None,
            execution: nexus_config::ExecutionConfig::default(),
            system_prompt: "test".to_string(),
            tenancy: None,
        };

        let factory = ToolFactory::new();
        let tools = factory.build_tools(&config).unwrap();

        // Should have the 2 built-in generic tools
        assert!(tools.contains_key("calculator"));
        assert!(tools.contains_key("string"));
        assert_eq!(tools.len(), 2);
    }
}
