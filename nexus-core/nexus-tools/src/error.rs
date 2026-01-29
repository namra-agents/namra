//! Error types for the tool system

use thiserror::Error;

/// Errors that can occur during tool execution
#[derive(Debug, Error)]
pub enum ToolError {
    /// Tool not found in registry
    #[error("Tool not found: {0}")]
    NotFound(String),

    /// Invalid input provided to tool
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Tool execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Tool execution timed out
    #[error("Timeout after {0}s")]
    Timeout(u64),

    /// HTTP request error
    #[error("HTTP error: {0}")]
    HttpError(String),

    /// File system error
    #[error("Filesystem error: {0}")]
    FilesystemError(String),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    JsonError(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Feature not yet implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Generic error
    #[error("Tool error: {0}")]
    Other(String),
}

impl From<reqwest::Error> for ToolError {
    fn from(err: reqwest::Error) -> Self {
        ToolError::HttpError(err.to_string())
    }
}

impl From<std::io::Error> for ToolError {
    fn from(err: std::io::Error) -> Self {
        ToolError::FilesystemError(err.to_string())
    }
}

impl From<serde_json::Error> for ToolError {
    fn from(err: serde_json::Error) -> Self {
        ToolError::JsonError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ToolError>;
