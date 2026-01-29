//! Common types for LLM interactions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// Role of a message in the conversation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Tool call made by the assistant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Request to an LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    /// Conversation messages
    pub messages: Vec<Message>,

    /// Model identifier (e.g., "claude-3-5-sonnet-20241022")
    pub model: String,

    /// Temperature (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Top P sampling (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    /// Enable streaming
    #[serde(default)]
    pub stream: bool,

    /// Available tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,

    /// Additional provider-specific parameters
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl LLMRequest {
    pub fn new(model: impl Into<String>, messages: Vec<Message>) -> Self {
        Self {
            messages,
            model: model.into(),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop_sequences: None,
            stream: false,
            tools: None,
            extra: HashMap::new(),
        }
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Response from an LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    /// Generated content
    pub content: String,

    /// Role of the response (usually Assistant)
    pub role: MessageRole,

    /// Tool calls requested by the assistant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Usage statistics
    pub usage: TokenUsage,

    /// Finish reason
    pub finish_reason: FinishReason,

    /// Provider-specific metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl LLMResponse {
    /// Convert this response to a Message for conversation history
    pub fn to_message(&self) -> Message {
        Message {
            role: self.role,
            content: self.content.clone(),
            name: None,
            tool_calls: self.tool_calls.clone(),
            tool_call_id: None,
        }
    }
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: u32,

    /// Output tokens
    pub output_tokens: u32,

    /// Total tokens (input + output)
    pub total_tokens: u32,

    /// Estimated cost in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
}

impl TokenUsage {
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cost: None,
        }
    }

    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = Some(cost);
        self
    }
}

/// Reason why generation finished
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// Natural stop
    Stop,

    /// Hit max tokens limit
    Length,

    /// Tool call requested
    ToolCalls,

    /// Content filtered
    ContentFilter,

    /// Other/unknown reason
    Other,
}

/// Streaming chunk from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Incremental content (delta)
    pub content: String,

    /// Tool call delta (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_delta: Option<ToolCallDelta>,

    /// Whether this is the final chunk
    #[serde(default)]
    pub is_final: bool,

    /// Usage statistics (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,

    /// Finish reason (only in final chunk)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<FinishReason>,
}

/// Delta for tool call in streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallDelta {
    pub id: Option<String>,
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_constructors() {
        let system = Message::system("You are helpful");
        assert_eq!(system.role, MessageRole::System);
        assert_eq!(system.content, "You are helpful");

        let user = Message::user("Hello");
        assert_eq!(user.role, MessageRole::User);

        let assistant = Message::assistant("Hi there");
        assert_eq!(assistant.role, MessageRole::Assistant);

        let tool = Message::tool("result", "call_123");
        assert_eq!(tool.role, MessageRole::Tool);
        assert_eq!(tool.tool_call_id, Some("call_123".to_string()));
    }

    #[test]
    fn test_llm_request_builder() {
        let request = LLMRequest::new("claude-3-5-sonnet-20241022", vec![Message::user("Hello")])
            .with_temperature(0.7)
            .with_max_tokens(1024)
            .with_streaming(true);

        assert_eq!(request.model, "claude-3-5-sonnet-20241022");
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(1024));
        assert!(request.stream);
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50).with_cost(0.003);

        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
        assert_eq!(usage.cost, Some(0.003));
    }
}
