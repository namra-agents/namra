//! Core tool trait and types

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Instant;

use crate::error::Result;

/// Output from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// Main result content
    pub content: String,

    /// Success/failure status
    pub success: bool,

    /// Optional metadata (could include headers, file info, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

impl ToolOutput {
    /// Create a successful tool output
    pub fn success(content: String, execution_time_ms: u64) -> Self {
        Self {
            content,
            success: true,
            metadata: None,
            execution_time_ms,
        }
    }

    /// Create a successful tool output with metadata
    pub fn success_with_metadata(content: String, metadata: Value, execution_time_ms: u64) -> Self {
        Self {
            content,
            success: true,
            metadata: Some(metadata),
            execution_time_ms,
        }
    }

    /// Create a failed tool output
    pub fn failure(content: String, execution_time_ms: u64) -> Self {
        Self {
            content,
            success: false,
            metadata: None,
            execution_time_ms,
        }
    }
}

/// Tool trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique tool identifier (e.g., "http", "filesystem", "calculator")
    fn name(&self) -> &str;

    /// Human-readable description of what the tool does
    fn description(&self) -> &str;

    /// JSON Schema for tool parameters
    /// This defines what inputs the tool expects
    fn parameters(&self) -> Value;

    /// Execute the tool with given input
    ///
    /// # Arguments
    /// * `input` - JSON value containing tool parameters
    ///
    /// # Returns
    /// * `Result<ToolOutput>` - Tool execution result
    async fn execute(&self, input: Value) -> Result<ToolOutput>;
}

/// Helper to time tool execution
pub struct ToolTimer {
    start: Instant,
}

impl ToolTimer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_output_success() {
        let output = ToolOutput::success("Hello".to_string(), 100);
        assert!(output.success);
        assert_eq!(output.content, "Hello");
        assert_eq!(output.execution_time_ms, 100);
        assert!(output.metadata.is_none());
    }

    #[test]
    fn test_tool_output_failure() {
        let output = ToolOutput::failure("Error".to_string(), 50);
        assert!(!output.success);
        assert_eq!(output.content, "Error");
        assert_eq!(output.execution_time_ms, 50);
    }

    #[test]
    fn test_tool_output_with_metadata() {
        let metadata = serde_json::json!({"status": 200});
        let output = ToolOutput::success_with_metadata("OK".to_string(), metadata.clone(), 75);
        assert!(output.success);
        assert_eq!(output.content, "OK");
        assert_eq!(output.metadata, Some(metadata));
    }

    #[test]
    fn test_tool_timer() {
        let timer = ToolTimer::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
    }
}
