//! Configuration parsing and validation for Nexus agents
//!
//! This crate provides functionality to parse and validate YAML/TOML
//! configuration files for agents, workflows, and platform settings.

mod agent;
mod parser;
mod validator;
mod workflow;

pub use agent::{
    AgentConfig, ExecutionConfig, FileSystemBackend, FileSystemToolConfig, FileSystemType,
    HttpToolConfig, LLMConfig, MemoryConfig, MiddlewareConfig, ToolConfig,
};
pub use parser::{ConfigFormat, ConfigParser};
pub use validator::ConfigValidator;
pub use workflow::{WorkflowConfig, WorkflowEdge, WorkflowNode};

use anyhow::Result;
use std::path::Path;

/// Parse an agent configuration from a file
pub fn parse_agent_config<P: AsRef<Path>>(path: P) -> Result<AgentConfig> {
    let parser = ConfigParser::new();
    parser.parse_agent(path)
}

/// Parse a workflow configuration from a file
pub fn parse_workflow_config<P: AsRef<Path>>(path: P) -> Result<WorkflowConfig> {
    let parser = ConfigParser::new();
    parser.parse_workflow(path)
}

/// Validate a configuration
pub fn validate_config(config: &AgentConfig) -> Result<()> {
    let validator = ConfigValidator::new();
    validator.validate(config)
}
