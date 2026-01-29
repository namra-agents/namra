//! ReAct strategy (Reasoning and Acting)
//!
//! The ReAct pattern alternates between:
//! 1. THINK: Reason about what to do next
//! 2. ACT: Either use a tool or provide final answer
//! 3. OBSERVE: Get tool result and feed back to LLM
//!
//! This continues until the agent provides a final answer or reaches limits.

use async_trait::async_trait;
use nexus_config::AgentConfig;
use nexus_llm::adapter::LLMAdapter;
use nexus_llm::types::{LLMRequest, Message};
use nexus_tools::Tool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use crate::context::{ExecutionContext, ToolCallRecord};
use crate::error::{Result, RuntimeError};
use crate::strategy::Strategy;

/// ReAct strategy implementation
pub struct ReActStrategy;

impl ReActStrategy {
    /// Create a new ReAct strategy
    pub fn new() -> Self {
        Self
    }

    /// Check if response contains a tool call
    /// For now, we use a simple heuristic: look for "TOOL:" prefix
    fn extract_tool_call(&self, response: &str) -> Option<(String, String)> {
        // Simple parsing: look for "TOOL: tool_name(argument)"
        if let Some(tool_start) = response.find("TOOL:") {
            let tool_part = &response[tool_start + 5..].trim();

            // Extract tool name and argument
            if let Some(paren_pos) = tool_part.find('(') {
                let tool_name = tool_part[..paren_pos].trim();
                if let Some(end_paren) = tool_part.find(')') {
                    let argument = tool_part[paren_pos + 1..end_paren].trim();
                    return Some((tool_name.to_string(), argument.to_string()));
                }
            }
        }

        None
    }

    /// Check if response is a final answer
    fn is_final_answer(&self, response: &str) -> bool {
        // Final answer if it contains "ANSWER:" or doesn't request a tool
        response.contains("ANSWER:") || !response.contains("TOOL:")
    }

    /// Extract final answer from response
    fn extract_answer(&self, response: &str) -> String {
        if let Some(answer_pos) = response.find("ANSWER:") {
            response[answer_pos + 7..].trim().to_string()
        } else {
            // Return the whole response if no ANSWER: prefix
            response.trim().to_string()
        }
    }
}

impl Default for ReActStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Strategy for ReActStrategy {
    async fn execute(
        &self,
        config: &AgentConfig,
        llm: &Arc<dyn LLMAdapter>,
        tools: &HashMap<String, Arc<dyn Tool>>,
        context: &mut ExecutionContext,
    ) -> Result<String> {
        // Main ReAct loop
        loop {
            // Check iteration limit
            if context.is_max_iterations_reached() {
                return Err(RuntimeError::MaxIterationsReached(context.max_iterations));
            }

            // Check timeout
            if context.is_timed_out() {
                return Err(RuntimeError::Timeout(context.timeout.as_secs()));
            }

            // Increment iteration
            context.increment_iteration();

            // Build LLM request with current conversation
            let request = LLMRequest {
                messages: context.messages.clone(),
                model: config.llm.model.clone(),
                temperature: Some(config.llm.temperature),
                max_tokens: Some(config.llm.max_tokens),
                top_p: config.llm.top_p,
                stream: false,
                tools: None,
                stop_sequences: None,
                extra: HashMap::new(),
            };

            // Call LLM (THINK phase)
            let response = llm.generate(request).await?;

            // Track tokens and cost
            context.add_tokens(response.usage.clone());
            let cost = llm.estimate_cost(
                response.usage.input_tokens,
                response.usage.output_tokens,
                &config.llm.model,
            ).unwrap_or(0.0);
            context.add_cost(cost);

            // Record the thought/reasoning
            context.record_thought(response.content.clone());

            // Add assistant response to context
            context.add_message(Message::assistant(response.content.clone()));

            // Try to extract tool call first (ACT phase)
            // If there's a tool call, execute it even if there's also an ANSWER
            if let Some((tool_name, argument)) = self.extract_tool_call(&response.content) {
                // Find the tool
                let tool = tools
                    .get(&tool_name)
                    .ok_or_else(|| RuntimeError::ToolNotFound(tool_name.clone()))?;

                // Parse argument as JSON
                let tool_input = if argument.starts_with('{') {
                    // Already JSON
                    serde_json::from_str(&argument).unwrap_or_else(|_| {
                        serde_json::json!({
                            "input": argument
                        })
                    })
                } else {
                    // Wrap in JSON
                    serde_json::json!({
                        "expression": argument
                    })
                };

                // Execute tool (OBSERVE phase)
                let tool_start = SystemTime::now();
                let tool_result = tool.execute(tool_input.clone()).await?;
                let tool_time = tool_start.elapsed().unwrap_or_default().as_millis() as u64;

                // Record tool call
                context.record_tool_call(ToolCallRecord {
                    tool_name: tool_name.clone(),
                    input: tool_input,
                    output: Some(tool_result.content.clone()),
                    success: tool_result.success,
                    execution_time_ms: tool_time,
                    timestamp: tool_start,
                });

                // Add tool result as a user message so LLM can observe it
                let observation = format!(
                    "Tool Result from {}: {}",
                    tool_name, tool_result.content
                );
                context.add_message(Message::user(observation));

                // Continue loop to let agent reason about the result
                continue;
            }

            // No tool call found, check if this is a final answer
            if self.is_final_answer(&response.content) {
                let answer = self.extract_answer(&response.content);
                return Ok(answer);
            }

            // If no tool call and not a final answer, something went wrong
            return Err(RuntimeError::InvalidToolCall(
                "Response contained neither tool call nor final answer".to_string(),
            ));
        }
    }

    fn name(&self) -> &str {
        "react"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tool_call() {
        let strategy = ReActStrategy::new();

        // Valid tool call
        let response = "I need to calculate. TOOL: calculator(2 + 2)";
        let result = strategy.extract_tool_call(response);
        assert!(result.is_some());
        let (tool, arg) = result.unwrap();
        assert_eq!(tool, "calculator");
        assert_eq!(arg, "2 + 2");

        // No tool call
        let response = "The answer is 4";
        let result = strategy.extract_tool_call(response);
        assert!(result.is_none());
    }

    #[test]
    fn test_is_final_answer() {
        let strategy = ReActStrategy::new();

        assert!(strategy.is_final_answer("ANSWER: The result is 4"));
        assert!(strategy.is_final_answer("The result is 4"));
        assert!(!strategy.is_final_answer("I need to use TOOL: calculator(2+2)"));
    }

    #[test]
    fn test_extract_answer() {
        let strategy = ReActStrategy::new();

        let response = "ANSWER: The result is 4";
        assert_eq!(strategy.extract_answer(response), "The result is 4");

        let response = "The result is 4";
        assert_eq!(strategy.extract_answer(response), "The result is 4");
    }
}
