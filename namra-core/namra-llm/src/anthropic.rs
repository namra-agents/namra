//! Anthropic (Claude) LLM adapter

use crate::adapter::{LLMAdapter, LLMError, LLMResult, LLMStream};
use crate::types::*;
use async_trait::async_trait;
use futures::stream::StreamExt;
use namra_middleware::observability::{llm_request_span, record_llm_metrics};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::Instrument;

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic API adapter for Claude models
pub struct AnthropicAdapter {
    client: Client,
    api_key: String,
    base_url: String,
    timeout: Duration,
}

impl AnthropicAdapter {
    /// Create a new Anthropic adapter
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: ANTHROPIC_API_BASE.to_string(),
            timeout: Duration::from_secs(120),
        }
    }

    /// Create builder for custom configuration
    pub fn builder() -> AnthropicAdapterBuilder {
        AnthropicAdapterBuilder::default()
    }

    /// Convert our Message type to Anthropic's format
    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system_prompt = None;
        let mut converted = Vec::new();

        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    // Anthropic uses a separate system parameter
                    system_prompt = Some(msg.content.clone());
                }
                MessageRole::User => {
                    converted.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: msg.content.clone(),
                    });
                }
                MessageRole::Assistant => {
                    converted.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: msg.content.clone(),
                    });
                }
                MessageRole::Tool => {
                    // For now, convert tool results to user messages
                    // Full tool support will be added later
                    converted.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: format!("Tool result: {}", msg.content),
                    });
                }
            }
        }

        (system_prompt, converted)
    }

    /// Calculate cost for Anthropic models
    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32, model: &str) -> f64 {
        // Pricing as of 2024 (per million tokens)
        let (input_price, output_price) = match model {
            m if m.contains("claude-3-5-sonnet") => (3.0, 15.0),
            m if m.contains("claude-3-opus") => (15.0, 75.0),
            m if m.contains("claude-3-sonnet") => (3.0, 15.0),
            m if m.contains("claude-3-haiku") => (0.25, 1.25),
            _ => (3.0, 15.0), // Default to Sonnet pricing
        };

        let input_cost = (input_tokens as f64 / 1_000_000.0) * input_price;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * output_price;

        input_cost + output_cost
    }
}

#[async_trait]
impl LLMAdapter for AnthropicAdapter {
    fn provider_name(&self) -> &str {
        "anthropic"
    }

    async fn generate(&self, request: LLMRequest) -> LLMResult<LLMResponse> {
        let span = llm_request_span("anthropic", &request.model);

        async move {
            let (system, messages) = self.convert_messages(&request.messages);

            let body = AnthropicRequest {
                model: request.model.clone(),
                messages,
                max_tokens: request.max_tokens.unwrap_or(4096),
                system,
                temperature: request.temperature,
                top_p: request.top_p,
                stop_sequences: request.stop_sequences.clone(),
                stream: false,
                metadata: None,
            };

            let response = self
                .client
                .post(format!("{}/v1/messages", self.base_url))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("content-type", "application/json")
                .timeout(self.timeout)
                .json(&body)
                .send()
                .await?;

            let status = response.status();

            if !status.is_success() {
                let error_text = response.text().await.unwrap_or_default();
                return Err(self.handle_error(status.as_u16(), error_text));
            }

            let anthropic_response: AnthropicResponse = response.json().await?;

            // Extract content from response
            let content = anthropic_response
                .content
                .iter()
                .map(|c| {
                    let AnthropicContent::Text { text } = c;
                    text.as_str()
                })
                .collect::<Vec<_>>()
                .join("\n");

            let cost = self.calculate_cost(
                anthropic_response.usage.input_tokens,
                anthropic_response.usage.output_tokens,
                &request.model,
            );

            let usage = TokenUsage::new(
                anthropic_response.usage.input_tokens,
                anthropic_response.usage.output_tokens,
            )
            .with_cost(cost);

            // Record LLM metrics on span
            let current_span = tracing::Span::current();
            record_llm_metrics(
                &current_span,
                anthropic_response.usage.input_tokens,
                anthropic_response.usage.output_tokens,
                cost,
            );

            let finish_reason = match anthropic_response.stop_reason.as_deref() {
                Some("end_turn") => FinishReason::Stop,
                Some("max_tokens") => FinishReason::Length,
                Some("stop_sequence") => FinishReason::Stop,
                _ => FinishReason::Other,
            };

            Ok(LLMResponse {
                content,
                role: MessageRole::Assistant,
                tool_calls: None,
                usage,
                finish_reason,
                metadata: HashMap::new(),
            })
        }
        .instrument(span)
        .await
    }

    async fn stream(&self, request: LLMRequest) -> LLMResult<LLMStream> {
        let (system, messages) = self.convert_messages(&request.messages);

        let body = AnthropicRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens.unwrap_or(4096),
            system,
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: request.stop_sequences.clone(),
            stream: true,
            metadata: None,
        };

        let response = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .timeout(self.timeout)
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(self.handle_error(status.as_u16(), error_text));
        }

        // Parse SSE stream
        let stream = response.bytes_stream();
        let sse_stream = eventsource_stream::EventStream::new(stream);

        let mapped_stream = sse_stream.filter_map(move |event_result| {
            async move {
                match event_result {
                    Ok(event) => {
                        if event.event == "message_delta" || event.event == "content_block_delta" {
                            // Parse the event data
                            if let Ok(delta) =
                                serde_json::from_str::<serde_json::Value>(&event.data)
                            {
                                if let Some(delta_obj) = delta.get("delta") {
                                    if let Some(text) =
                                        delta_obj.get("text").and_then(|t| t.as_str())
                                    {
                                        return Some(Ok(StreamChunk {
                                            content: text.to_string(),
                                            tool_call_delta: None,
                                            is_final: false,
                                            usage: None,
                                            finish_reason: None,
                                        }));
                                    }
                                }
                            }
                        } else if event.event == "message_stop" {
                            // Final chunk
                            return Some(Ok(StreamChunk {
                                content: String::new(),
                                tool_call_delta: None,
                                is_final: true,
                                usage: None,
                                finish_reason: Some(FinishReason::Stop),
                            }));
                        }
                        None
                    }
                    Err(e) => Some(Err(LLMError::StreamError(e.to_string()))),
                }
            }
        });

        Ok(Box::pin(mapped_stream))
    }

    fn max_context_tokens(&self, model: &str) -> Option<u32> {
        Some(match model {
            m if m.contains("claude-3") => 200_000,
            m if m.contains("claude-2") => 100_000,
            _ => 200_000,
        })
    }

    fn estimate_cost(&self, input_tokens: u32, output_tokens: u32, model: &str) -> Option<f64> {
        Some(self.calculate_cost(input_tokens, output_tokens, model))
    }
}

impl AnthropicAdapter {
    fn handle_error(&self, status: u16, body: String) -> LLMError {
        match status {
            401 => LLMError::AuthenticationError("Invalid API key".to_string()),
            429 => LLMError::RateLimited { retry_after: None },
            400 => LLMError::InvalidRequest(body),
            _ => LLMError::ApiError {
                status,
                message: body,
            },
        }
    }
}

/// Builder for Anthropic adapter
#[derive(Default)]
pub struct AnthropicAdapterBuilder {
    api_key: Option<String>,
    base_url: Option<String>,
    timeout_secs: Option<u64>,
}

impl AnthropicAdapterBuilder {
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }

    pub fn build(self) -> AnthropicAdapter {
        let api_key = self.api_key.expect("API key is required");

        AnthropicAdapter {
            client: Client::new(),
            api_key,
            base_url: self
                .base_url
                .unwrap_or_else(|| ANTHROPIC_API_BASE.to_string()),
            timeout: Duration::from_secs(self.timeout_secs.unwrap_or(120)),
        }
    }
}

// Anthropic API types

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,

    stream: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicContent {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        let adapter = AnthropicAdapter::new("test-key");

        // Claude 3.5 Sonnet: $3/$15 per million tokens
        let cost = adapter.calculate_cost(1000, 500, "claude-3-5-sonnet-20241022");
        assert!((cost - 0.0105).abs() < 0.0001); // (1000/1M)*3 + (500/1M)*15

        // Claude 3 Opus: $15/$75 per million tokens
        let cost = adapter.calculate_cost(1000, 500, "claude-3-opus-20240229");
        assert!((cost - 0.0525).abs() < 0.0001); // (1000/1M)*15 + (500/1M)*75
    }

    #[test]
    fn test_message_conversion() {
        let adapter = AnthropicAdapter::new("test-key");

        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
            Message::assistant("Hi there"),
        ];

        let (system, converted) = adapter.convert_messages(&messages);

        assert_eq!(system, Some("You are helpful".to_string()));
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].role, "user");
        assert_eq!(converted[1].role, "assistant");
    }

    #[tokio::test]
    #[ignore] // Only run with real API key
    async fn test_real_api_call() {
        let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
        let adapter = AnthropicAdapter::new(api_key);

        let request = LLMRequest::new(
            "claude-3-5-sonnet-20241022",
            vec![Message::user("Say 'Hello, Nexus!' and nothing else.")],
        )
        .with_max_tokens(50)
        .with_temperature(0.0);

        let response = adapter.generate(request).await.unwrap();

        assert!(response.content.contains("Hello"));
        assert!(response.usage.total_tokens > 0);
        assert!(response.usage.cost.is_some());
    }
}
