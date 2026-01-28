//! LLM adapter trait and implementations

use crate::types::{LLMRequest, LLMResponse, StreamChunk};
use async_trait::async_trait;
use std::pin::Pin;

/// Result type for LLM operations
pub type LLMResult<T> = Result<T, LLMError>;

/// Stream of chunks from streaming LLM response
pub type LLMStream = Pin<Box<dyn futures::Stream<Item = LLMResult<StreamChunk>> + Send>>;

/// Error types for LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Rate limited: {retry_after:?}")]
    RateLimited { retry_after: Option<u64> },

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Streaming error: {0}")]
    StreamError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Trait for LLM adapters
///
/// All LLM providers (Anthropic, OpenAI, Google, etc.) implement this trait
/// to provide a unified interface for interacting with language models.
#[async_trait]
pub trait LLMAdapter: Send + Sync {
    /// Get the provider name (e.g., "anthropic", "openai")
    fn provider_name(&self) -> &str;

    /// Generate a completion (non-streaming)
    ///
    /// # Arguments
    /// * `request` - The LLM request with messages and parameters
    ///
    /// # Returns
    /// The complete LLM response
    async fn generate(&self, request: LLMRequest) -> LLMResult<LLMResponse>;

    /// Generate a completion with streaming
    ///
    /// # Arguments
    /// * `request` - The LLM request with messages and parameters
    ///
    /// # Returns
    /// A stream of response chunks
    async fn stream(&self, request: LLMRequest) -> LLMResult<LLMStream>;

    /// Check if the adapter supports streaming
    fn supports_streaming(&self) -> bool {
        true
    }

    /// Check if the adapter supports function/tool calling
    fn supports_tools(&self) -> bool {
        true
    }

    /// Get the maximum context window size for the model
    fn max_context_tokens(&self, model: &str) -> Option<u32>;

    /// Estimate cost for a request (in USD)
    fn estimate_cost(&self, input_tokens: u32, output_tokens: u32, model: &str) -> Option<f64>;
}

/// Helper trait for building LLM adapters
pub trait LLMAdapterBuilder {
    type Adapter: LLMAdapter;

    /// Build the adapter with the given API key
    fn with_api_key(self, api_key: String) -> Self;

    /// Set custom base URL (for testing or alternative endpoints)
    fn with_base_url(self, base_url: String) -> Self;

    /// Set timeout duration
    fn with_timeout(self, timeout_secs: u64) -> Self;

    /// Build the final adapter
    fn build(self) -> Self::Adapter;
}

#[cfg(test)]
mod tests {
    // Mock adapter tests will be added in Week 4
}
