//! LLM adapters for the Nexus agent framework
//!
//! This crate provides unified interfaces to various LLM providers:
//! - Anthropic (Claude)
//! - OpenAI (GPT-4, GPT-3.5)
//! - Google (Gemini)
//! - AWS Bedrock
//! - Local models (Ollama, vLLM)

pub mod adapter;
pub mod anthropic;
pub mod types;

// Re-export commonly used types
pub use adapter::{LLMAdapter, LLMError, LLMResult, LLMStream};
pub use anthropic::AnthropicAdapter;
pub use types::{
    FinishReason, LLMRequest, LLMResponse, Message, MessageRole, StreamChunk, TokenUsage, ToolCall,
    ToolDefinition,
};
