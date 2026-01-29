//! Execution strategies

pub mod react;

use async_trait::async_trait;
use namra_config::AgentConfig;
use namra_llm::adapter::LLMAdapter;
use namra_tools::Tool;
use std::collections::HashMap;
use std::sync::Arc;

use crate::context::ExecutionContext;
use crate::error::Result;

/// Execution strategy trait
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Execute the strategy
    ///
    /// # Arguments
    /// * `config` - Agent configuration
    /// * `llm` - LLM adapter
    /// * `tools` - Available tools
    /// * `context` - Execution context
    ///
    /// # Returns
    /// Final response string
    async fn execute(
        &self,
        config: &AgentConfig,
        llm: &Arc<dyn LLMAdapter>,
        tools: &HashMap<String, Arc<dyn Tool>>,
        context: &mut ExecutionContext,
    ) -> Result<String>;

    /// Strategy name
    fn name(&self) -> &str;
}
