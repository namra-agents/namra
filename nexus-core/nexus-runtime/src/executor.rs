//! Agent executor - core execution engine

use nexus_config::AgentConfig;
use nexus_llm::adapter::LLMAdapter;
use nexus_llm::types::Message;
use nexus_tools::Tool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::context::{ExecutionContext, ExecutionResult};
use crate::error::{Result, RuntimeError};
use crate::strategy::Strategy;

/// Agent executor
pub struct AgentExecutor {
    /// Agent configuration
    config: AgentConfig,

    /// LLM adapter
    llm: Arc<dyn LLMAdapter>,

    /// Available tools (name -> tool)
    tools: HashMap<String, Arc<dyn Tool>>,

    /// Execution strategy
    strategy: Box<dyn Strategy>,
}

impl AgentExecutor {
    /// Create a new agent executor
    pub fn new(
        config: AgentConfig,
        llm: Arc<dyn LLMAdapter>,
        tools: HashMap<String, Arc<dyn Tool>>,
        strategy: Box<dyn Strategy>,
    ) -> Self {
        Self {
            config,
            llm,
            tools,
            strategy,
        }
    }

    /// Execute the agent with a given input
    pub async fn execute(&self, input: &str) -> Result<ExecutionResult> {
        // Create execution context
        let timeout_secs = self.parse_timeout(&self.config.execution.timeout)?;
        let timeout = Duration::from_secs(timeout_secs);
        let mut context = ExecutionContext::new(self.config.execution.max_iterations, timeout);

        // Add system message if provided
        if !self.config.system_prompt.is_empty() {
            context.add_message(Message::system(self.config.system_prompt.clone()));
        }

        // Add user input
        context.add_message(Message::user(input.to_string()));

        // Run the strategy
        let result = self.strategy.execute(&self.config, &self.llm, &self.tools, &mut context).await;

        // Build final result
        match result {
            Ok(response) => {
                let execution_time = context.elapsed().as_millis() as u64;
                Ok(ExecutionResult::success(
                    context.id.clone(),
                    response,
                    context.iteration,
                    context.tool_calls.clone(),
                    context.total_tokens(),
                    context.total_cost,
                    execution_time,
                    context.thoughts.clone(),
                ))
            }
            Err(e) => {
                let execution_time = context.elapsed().as_millis() as u64;
                Ok(ExecutionResult::failure(
                    context.id.clone(),
                    e.to_string(),
                    context.iteration,
                    context.tool_calls.clone(),
                    context.total_tokens(),
                    context.total_cost,
                    execution_time,
                    context.thoughts.clone(),
                ))
            }
        }
    }

    /// Get agent configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get available tools
    pub fn tools(&self) -> &HashMap<String, Arc<dyn Tool>> {
        &self.tools
    }

    /// Parse timeout string like "30s" into seconds
    fn parse_timeout(&self, timeout_str: &str) -> Result<u64> {
        let timeout_str = timeout_str.trim();
        if timeout_str.ends_with('s') {
            let secs = timeout_str[..timeout_str.len() - 1]
                .parse::<u64>()
                .map_err(|e| RuntimeError::ConfigError(format!("Invalid timeout format: {}", e)))?;
            Ok(secs)
        } else if timeout_str.ends_with("ms") {
            let ms = timeout_str[..timeout_str.len() - 2]
                .parse::<u64>()
                .map_err(|e| RuntimeError::ConfigError(format!("Invalid timeout format: {}", e)))?;
            Ok(ms / 1000)
        } else {
            // Assume seconds if no unit
            timeout_str
                .parse::<u64>()
                .map_err(|e| RuntimeError::ConfigError(format!("Invalid timeout format: {}", e)))
        }
    }
}

/// Builder for AgentExecutor
pub struct AgentExecutorBuilder {
    config: Option<AgentConfig>,
    llm: Option<Arc<dyn LLMAdapter>>,
    tools: HashMap<String, Arc<dyn Tool>>,
    strategy: Option<Box<dyn Strategy>>,
}

impl AgentExecutorBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: None,
            llm: None,
            tools: HashMap::new(),
            strategy: None,
        }
    }

    /// Set the agent configuration
    pub fn config(mut self, config: AgentConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the LLM adapter
    pub fn llm(mut self, llm: Arc<dyn LLMAdapter>) -> Self {
        self.llm = Some(llm);
        self
    }

    /// Add a tool
    pub fn tool(mut self, name: String, tool: Arc<dyn Tool>) -> Self {
        self.tools.insert(name, tool);
        self
    }

    /// Add multiple tools
    pub fn tools(mut self, tools: HashMap<String, Arc<dyn Tool>>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Set the execution strategy
    pub fn strategy(mut self, strategy: Box<dyn Strategy>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Build the executor
    pub fn build(self) -> Result<AgentExecutor> {
        let config = self
            .config
            .ok_or_else(|| RuntimeError::ConfigError("Missing agent config".to_string()))?;

        let llm = self
            .llm
            .ok_or_else(|| RuntimeError::ConfigError("Missing LLM adapter".to_string()))?;

        let strategy = self
            .strategy
            .ok_or_else(|| RuntimeError::ConfigError("Missing execution strategy".to_string()))?;

        Ok(AgentExecutor::new(config, llm, self.tools, strategy))
    }
}

impl Default for AgentExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = AgentExecutorBuilder::new();
        assert!(builder.config.is_none());
        assert!(builder.llm.is_none());
        assert!(builder.strategy.is_none());
    }
}
