//! Configuration validator

use crate::AgentConfig;
use anyhow::{Context, Result};
use validator::Validate;

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate an agent configuration
    pub fn validate(&self, config: &AgentConfig) -> Result<()> {
        // Use validator crate for basic validation
        config
            .validate()
            .context("Configuration validation failed")?;

        // Additional custom validation
        self.validate_llm_config(config)?;
        self.validate_tools(config)?;
        self.validate_execution_config(config)?;

        Ok(())
    }

    fn validate_llm_config(&self, config: &AgentConfig) -> Result<()> {
        // Validate temperature range
        if !(0.0..=2.0).contains(&config.llm.temperature) {
            anyhow::bail!("LLM temperature must be between 0.0 and 2.0");
        }

        // Validate max_tokens
        if config.llm.max_tokens == 0 {
            anyhow::bail!("LLM max_tokens must be greater than 0");
        }

        // Validate top_p if present
        if let Some(top_p) = config.llm.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                anyhow::bail!("LLM top_p must be between 0.0 and 1.0");
            }
        }

        Ok(())
    }

    fn validate_tools(&self, _config: &AgentConfig) -> Result<()> {
        // Tool-specific validation can be added here
        // For now, just return Ok
        Ok(())
    }

    fn validate_execution_config(&self, config: &AgentConfig) -> Result<()> {
        // Validate max_iterations
        if config.execution.max_iterations == 0 {
            anyhow::bail!("Execution max_iterations must be greater than 0");
        }

        // Validate strategy
        let valid_strategies = ["react", "chain_of_thought", "tree_of_thoughts"];
        if !valid_strategies.contains(&config.execution.strategy.as_str()) {
            anyhow::bail!(
                "Invalid execution strategy: {}. Must be one of: {}",
                config.execution.strategy,
                valid_strategies.join(", ")
            );
        }

        Ok(())
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExecutionConfig, LLMConfig};
    use std::collections::HashMap;

    fn create_minimal_config() -> AgentConfig {
        AgentConfig {
            name: "test_agent".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            metadata: HashMap::new(),
            llm: LLMConfig {
                provider: "anthropic".to_string(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                temperature: 0.7,
                max_tokens: 4096,
                top_p: None,
                stream: true,
                retry: None,
            },
            tools: vec![],
            memory: None,
            middleware: None,
            execution: ExecutionConfig::default(),
            system_prompt: "You are a helpful assistant.".to_string(),
            tenancy: None,
        }
    }

    #[test]
    fn test_valid_config() {
        let config = create_minimal_config();
        let validator = ConfigValidator::new();
        assert!(validator.validate(&config).is_ok());
    }

    #[test]
    fn test_invalid_temperature() {
        let mut config = create_minimal_config();
        config.llm.temperature = 3.0; // Invalid: > 2.0

        let validator = ConfigValidator::new();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_invalid_max_tokens() {
        let mut config = create_minimal_config();
        config.llm.max_tokens = 0; // Invalid: must be > 0

        let validator = ConfigValidator::new();
        assert!(validator.validate(&config).is_err());
    }
}
