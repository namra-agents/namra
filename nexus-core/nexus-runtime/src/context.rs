//! Execution context for agent runs

use nexus_llm::types::{Message, TokenUsage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};


/// Execution context tracks the state of an agent run
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Execution ID
    pub id: String,

    /// Conversation messages
    pub messages: Vec<Message>,

    /// Current iteration number
    pub iteration: u32,

    /// Maximum iterations allowed
    pub max_iterations: u32,

    /// Execution timeout
    pub timeout: Duration,

    /// Start time
    pub started_at: SystemTime,

    /// Total tokens used
    pub total_tokens: TokenUsage,

    /// Total cost
    pub total_cost: f64,

    /// Tool call history
    pub tool_calls: Vec<ToolCallRecord>,

    /// Intermediate thoughts/reasoning
    pub thoughts: Vec<String>,

    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Record of a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    /// Tool name
    pub tool_name: String,

    /// Input to the tool
    pub input: serde_json::Value,

    /// Output from the tool
    pub output: Option<String>,

    /// Success status
    pub success: bool,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Timestamp
    pub timestamp: SystemTime,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(max_iterations: u32, timeout: Duration) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            messages: Vec::new(),
            iteration: 0,
            max_iterations,
            timeout,
            started_at: SystemTime::now(),
            total_tokens: TokenUsage::new(0, 0),
            total_cost: 0.0,
            tool_calls: Vec::new(),
            thoughts: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Increment iteration count
    pub fn increment_iteration(&mut self) {
        self.iteration += 1;
    }

    /// Check if max iterations reached
    pub fn is_max_iterations_reached(&self) -> bool {
        self.iteration >= self.max_iterations
    }

    /// Check if execution has timed out
    pub fn is_timed_out(&self) -> bool {
        self.started_at.elapsed().unwrap_or(Duration::from_secs(0)) >= self.timeout
    }

    /// Add token usage
    pub fn add_tokens(&mut self, usage: TokenUsage) {
        self.total_tokens.input_tokens += usage.input_tokens;
        self.total_tokens.output_tokens += usage.output_tokens;
    }

    /// Add cost
    pub fn add_cost(&mut self, cost: f64) {
        self.total_cost += cost;
    }

    /// Record a tool call
    pub fn record_tool_call(&mut self, record: ToolCallRecord) {
        self.tool_calls.push(record);
    }

    /// Record a thought/reasoning step
    pub fn record_thought(&mut self, thought: String) {
        self.thoughts.push(thought);
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed().unwrap_or(Duration::from_secs(0))
    }

    /// Get total tokens
    pub fn total_tokens(&self) -> u32 {
        self.total_tokens.input_tokens + self.total_tokens.output_tokens
    }
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Execution ID
    pub id: String,

    /// Final response
    pub response: String,

    /// Success status
    pub success: bool,

    /// Number of iterations
    pub iterations: u32,

    /// Tool calls made
    pub tool_calls: Vec<ToolCallRecord>,

    /// Total tokens used
    pub total_tokens: u32,

    /// Total cost
    pub total_cost: f64,

    /// Execution time
    pub execution_time_ms: u64,

    /// Stop reason
    pub stop_reason: StopReason,

    /// Intermediate thoughts/reasoning from each iteration
    pub thoughts: Vec<String>,
}

/// Reason execution stopped
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StopReason {
    /// Task completed successfully
    Completed,

    /// Max iterations reached
    MaxIterations,

    /// Timeout
    Timeout,

    /// Error occurred
    Error(String),

    /// User requested stop
    UserStop,
}

impl ExecutionResult {
    /// Create a successful result
    pub fn success(
        id: String,
        response: String,
        iterations: u32,
        tool_calls: Vec<ToolCallRecord>,
        total_tokens: u32,
        total_cost: f64,
        execution_time_ms: u64,
        thoughts: Vec<String>,
    ) -> Self {
        Self {
            id,
            response,
            success: true,
            iterations,
            tool_calls,
            total_tokens,
            total_cost,
            execution_time_ms,
            stop_reason: StopReason::Completed,
            thoughts,
        }
    }

    /// Create a failure result
    pub fn failure(
        id: String,
        error: String,
        iterations: u32,
        tool_calls: Vec<ToolCallRecord>,
        total_tokens: u32,
        total_cost: f64,
        execution_time_ms: u64,
        thoughts: Vec<String>,
    ) -> Self {
        Self {
            id,
            response: String::new(),
            success: false,
            iterations,
            tool_calls,
            total_tokens,
            total_cost,
            execution_time_ms,
            stop_reason: StopReason::Error(error),
            thoughts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ExecutionContext::new(10, Duration::from_secs(60));
        assert_eq!(ctx.iteration, 0);
        assert_eq!(ctx.max_iterations, 10);
        assert_eq!(ctx.messages.len(), 0);
    }

    #[test]
    fn test_iteration_increment() {
        let mut ctx = ExecutionContext::new(10, Duration::from_secs(60));
        assert_eq!(ctx.iteration, 0);

        ctx.increment_iteration();
        assert_eq!(ctx.iteration, 1);

        ctx.increment_iteration();
        assert_eq!(ctx.iteration, 2);
    }

    #[test]
    fn test_max_iterations_check() {
        let mut ctx = ExecutionContext::new(2, Duration::from_secs(60));
        assert!(!ctx.is_max_iterations_reached());

        ctx.increment_iteration();
        assert!(!ctx.is_max_iterations_reached());

        ctx.increment_iteration();
        assert!(ctx.is_max_iterations_reached());
    }

    #[test]
    fn test_token_tracking() {
        let mut ctx = ExecutionContext::new(10, Duration::from_secs(60));
        assert_eq!(ctx.total_tokens(), 0);

        ctx.add_tokens(TokenUsage::new(100, 50));
        assert_eq!(ctx.total_tokens(), 150);

        ctx.add_tokens(TokenUsage::new(200, 100));
        assert_eq!(ctx.total_tokens(), 450);
    }

    #[test]
    fn test_cost_tracking() {
        let mut ctx = ExecutionContext::new(10, Duration::from_secs(60));
        assert_eq!(ctx.total_cost, 0.0);

        ctx.add_cost(0.001);
        assert!((ctx.total_cost - 0.001).abs() < 0.0001);

        ctx.add_cost(0.002);
        assert!((ctx.total_cost - 0.003).abs() < 0.0001);
    }
}
