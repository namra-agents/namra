//! Error types for the runtime

use thiserror::Error;

/// Errors that can occur during agent execution
#[derive(Debug, Error)]
pub enum RuntimeError {
    /// LLM error
    #[error("LLM error: {0}")]
    LLMError(#[from] nexus_llm::adapter::LLMError),

    /// Tool error
    #[error("Tool error: {0}")]
    ToolError(#[from] nexus_tools::ToolError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Execution timeout
    #[error("Execution timeout after {0}s")]
    Timeout(u64),

    /// Max iterations reached
    #[error("Max iterations reached: {0}")]
    MaxIterationsReached(u32),

    /// Tool not found
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Invalid tool call
    #[error("Invalid tool call: {0}")]
    InvalidToolCall(String),

    /// Execution stopped
    #[error("Execution stopped: {0}")]
    Stopped(String),

    /// Generic error
    #[error("Runtime error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for RuntimeError {
    fn from(err: anyhow::Error) -> Self {
        RuntimeError::Other(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
