# Namra Data Structures & Type System

**Last Updated**: January 28, 2026

---

## Core Type Hierarchy

```
                        ┌──────────────────┐
                        │  AgentConfig     │
                        │  (namra-config)  │
                        └────────┬─────────┘
                                 │
                    ┌────────────┼────────────┐
                    │            │            │
         ┌──────────▼──────┐ ┌──▼────────┐ ┌▼──────────────┐
         │   LLMConfig     │ │ToolConfig │ │ExecutionConfig│
         └──────────┬──────┘ └───────────┘ └───────────────┘
                    │
         ┌──────────┼──────────┐
         │          │          │
    ┌────▼────┐ ┌──▼──────┐ ┌─▼─────────┐
    │Provider │ │  Model  │ │Parameters │
    └─────────┘ └─────────┘ └───────────┘


                        ┌──────────────────┐
                        │   LLMRequest     │
                        │   (namra-llm)    │
                        └────────┬─────────┘
                                 │
                    ┌────────────┼────────────┐
                    │            │            │
         ┌──────────▼──────┐ ┌──▼────────┐ ┌▼──────────┐
         │   Message[]     │ │ Settings  │ │ Stream?   │
         └──────────┬──────┘ └───────────┘ └───────────┘
                    │
         ┌──────────┼──────────┐
         │          │          │
    ┌────▼────┐ ┌──▼──────┐ ┌─▼────────┐
    │ System  │ │  User   │ │Assistant │
    └─────────┘ └─────────┘ └──────────┘


                        ┌──────────────────┐
                        │  LLMResponse     │
                        │  (namra-llm)     │
                        └────────┬─────────┘
                                 │
                    ┌────────────┼────────────┐
                    │            │            │
         ┌──────────▼──────┐ ┌──▼────────┐ ┌▼──────────┐
         │   Content       │ │TokenUsage │ │FinishRsn  │
         └─────────────────┘ └───┬───────┘ └───────────┘
                                 │
                    ┌────────────┼────────────┐
                    │            │            │
               ┌────▼────┐  ┌───▼──────┐  ┌─▼─────┐
               │  Input  │  │  Output  │  │ Cost  │
               └─────────┘  └──────────┘  └───────┘
```

---

## 1. Configuration Types (namra-config)

### AgentConfig
```rust
pub struct AgentConfig {
    /// Unique agent name
    pub name: String,

    /// Semantic version (e.g., "1.0.0")
    pub version: String,

    /// LLM provider settings
    pub llm: LLMConfig,

    /// Available tools
    pub tools: Vec<ToolConfig>,

    /// Memory/storage settings
    pub memory: MemoryConfig,

    /// Middleware configuration
    pub middleware: MiddlewareConfig,

    /// Execution strategy
    pub execution: ExecutionConfig,

    /// System prompt / instructions
    pub system_prompt: String,
}
```

### LLMConfig
```rust
pub struct LLMConfig {
    /// Provider name: "anthropic", "openai", etc.
    pub provider: String,

    /// Model identifier
    pub model: String,

    /// Temperature (0.0 - 1.0)
    #[validate(range(min = 0.0, max = 1.0))]
    pub temperature: f32,

    /// Max tokens to generate
    #[validate(range(min = 1))]
    pub max_tokens: u32,

    /// Optional API key override
    pub api_key: Option<String>,

    /// Additional provider-specific settings
    pub extra: HashMap<String, serde_json::Value>,
}
```

### ToolConfig
```rust
pub struct ToolConfig {
    /// Tool name (must be unique)
    pub name: String,

    /// Tool type: "builtin.http", "builtin.file", "custom"
    pub tool_type: String,

    /// Tool-specific configuration
    pub config: HashMap<String, serde_json::Value>,

    /// Optional description
    pub description: Option<String>,

    /// Timeout in seconds
    pub timeout: Option<u64>,
}
```

### ExecutionConfig
```rust
pub struct ExecutionConfig {
    /// Strategy: "react", "chain", "plan_and_execute"
    pub strategy: String,

    /// Maximum iterations before stopping
    pub max_iterations: u32,

    /// Total timeout in seconds
    pub timeout: u64,

    /// Whether to allow parallel tool calls
    pub parallel_tools: bool,
}
```

---

## 2. LLM Types (namra-llm)

### LLMRequest
```rust
pub struct LLMRequest {
    /// Conversation messages
    pub messages: Vec<Message>,

    /// Model to use
    pub model: String,

    /// Temperature (0.0 - 1.0)
    pub temperature: f32,

    /// Max tokens to generate
    pub max_tokens: u32,

    /// Enable streaming
    pub stream: bool,

    /// Available tools (for tool use)
    pub tools: Option<Vec<ToolDefinition>>,

    /// Stop sequences
    pub stop_sequences: Option<Vec<String>>,
}

impl LLMRequest {
    pub fn builder() -> LLMRequestBuilder { ... }
}
```

### Message
```rust
pub struct Message {
    /// Message role
    pub role: MessageRole,

    /// Message content
    pub content: String,

    /// Optional tool calls (for assistant messages)
    pub tool_calls: Option<Vec<ToolCall>>,
}

pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self { ... }
    pub fn user(content: impl Into<String>) -> Self { ... }
    pub fn assistant(content: impl Into<String>) -> Self { ... }
}
```

### LLMResponse
```rust
pub struct LLMResponse {
    /// Generated content
    pub content: String,

    /// Why generation stopped
    pub finish_reason: FinishReason,

    /// Token usage statistics
    pub usage: TokenUsage,

    /// Optional tool calls requested by LLM
    pub tool_calls: Option<Vec<ToolCall>>,
}

pub enum FinishReason {
    /// Natural completion
    Stop,

    /// Hit max_tokens limit
    Length,

    /// LLM wants to use a tool
    ToolUse,

    /// Hit stop sequence
    StopSequence,

    /// Content filtered
    ContentFilter,

    /// Error occurred
    Error,
}
```

### TokenUsage
```rust
pub struct TokenUsage {
    /// Input tokens consumed
    pub input_tokens: u32,

    /// Output tokens generated
    pub output_tokens: u32,
}

impl TokenUsage {
    /// Total tokens used
    pub fn total(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }

    /// Calculate cost for a given model
    pub fn calculate_cost(&self, pricing: &ModelPricing) -> f64 {
        let input_cost = (self.input_tokens as f64 / 1_000_000.0)
            * pricing.input_price;
        let output_cost = (self.output_tokens as f64 / 1_000_000.0)
            * pricing.output_price;
        input_cost + output_cost
    }
}
```

### StreamChunk
```rust
pub enum StreamChunk {
    /// Content delta (text fragment)
    Content(String),

    /// Metadata about the stream
    Metadata(StreamMetadata),

    /// Stream complete with final stats
    Done(TokenUsage),
}

pub struct StreamMetadata {
    pub message_id: String,
    pub model: String,
}
```

---

## 3. Tool Types (namra-tools - Week 3)

### Tool Trait
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique tool identifier
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// JSON Schema for parameters
    fn parameters(&self) -> serde_json::Value;

    /// Execute the tool
    async fn execute(
        &self,
        input: serde_json::Value
    ) -> Result<ToolOutput>;
}
```

### ToolOutput
```rust
pub struct ToolOutput {
    /// Main result content
    pub content: String,

    /// Success/failure status
    pub success: bool,

    /// Optional metadata
    pub metadata: Option<serde_json::Value>,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}
```

### ToolDefinition
```rust
/// Tool definition for LLM (Anthropic/OpenAI format)
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value, // JSON Schema
}
```

### ToolCall
```rust
/// Tool call request from LLM
pub struct ToolCall {
    /// Unique ID for this call
    pub id: String,

    /// Tool name
    pub name: String,

    /// Tool input (JSON)
    pub input: serde_json::Value,
}
```

---

## 4. Runtime Types (namra-runtime - Week 4)

### ExecutionContext
```rust
pub struct ExecutionContext {
    /// Agent configuration
    pub config: AgentConfig,

    /// LLM adapter
    pub llm: Box<dyn LLMAdapter>,

    /// Available tools
    pub tools: HashMap<String, Box<dyn Tool>>,

    /// Memory store
    pub memory: Box<dyn MemoryStore>,

    /// Conversation history
    pub messages: Vec<Message>,

    /// Current iteration
    pub iteration: u32,

    /// Total tokens used
    pub total_tokens: u32,

    /// Total cost so far
    pub total_cost: f64,
}
```

### AgentExecution
```rust
pub struct AgentExecution {
    /// Execution ID
    pub id: String,

    /// Status
    pub status: ExecutionStatus,

    /// Start time
    pub started_at: SystemTime,

    /// End time (if complete)
    pub completed_at: Option<SystemTime>,

    /// Final result
    pub result: Option<String>,

    /// Execution trace
    pub trace: Vec<ExecutionStep>,
}

pub enum ExecutionStatus {
    Running,
    Completed,
    Failed(String),
    Timeout,
}
```

### ExecutionStep
```rust
pub struct ExecutionStep {
    /// Step number
    pub step: u32,

    /// Step type
    pub step_type: StepType,

    /// Input to step
    pub input: String,

    /// Output from step
    pub output: String,

    /// Tokens used in this step
    pub tokens: u32,

    /// Timestamp
    pub timestamp: SystemTime,
}

pub enum StepType {
    Thinking,
    ToolCall(String),
    Response,
}
```

---

## 5. Anthropic API Types (namra-llm internals)

### Anthropic Request
```rust
#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,

    #[serde(default)]
    stream: bool,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String, // "user" or "assistant"
    content: String,
}
```

### Anthropic Response
```rust
#[derive(Deserialize)]
struct AnthropicResponse {
    id: String,

    #[serde(rename = "type")]
    response_type: String, // "message"

    role: String, // "assistant"

    content: Vec<ContentBlock>,

    model: String,

    stop_reason: Option<String>,

    usage: AnthropicUsage,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String, // "text"

    text: String,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}
```

### Streaming Events
```rust
#[derive(Deserialize)]
#[serde(tag = "type")]
enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: MessageStartData },

    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: u32, content_block: ContentBlockStart },

    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: u32, delta: Delta },

    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },

    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessageDeltaData, usage: AnthropicUsage },

    #[serde(rename = "message_stop")]
    MessageStop,
}

#[derive(Deserialize)]
struct Delta {
    #[serde(rename = "type")]
    delta_type: String, // "text_delta"

    text: String,
}
```

---

## 6. Error Types

### Unified Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum NamraError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("LLM error: {0}")]
    LLM(#[from] LLMError),

    #[error("Tool error: {0}")]
    Tool(#[from] ToolError),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("Unauthorized: Invalid API key")]
    Unauthorized,

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Timeout after {0}s")]
    Timeout(u64),
}
```

---

## 7. Type Conversions

### From Config to LLM Request
```rust
impl From<&AgentConfig> for LLMRequest {
    fn from(config: &AgentConfig) -> Self {
        LLMRequest {
            model: config.llm.model.clone(),
            temperature: config.llm.temperature,
            max_tokens: config.llm.max_tokens,
            messages: vec![
                Message::system(config.system_prompt.clone())
            ],
            stream: false,
            tools: None,
            stop_sequences: None,
        }
    }
}
```

### From Anthropic to Internal Types
```rust
impl From<AnthropicResponse> for LLMResponse {
    fn from(resp: AnthropicResponse) -> Self {
        let content = resp.content
            .into_iter()
            .map(|block| block.text)
            .collect::<Vec<_>>()
            .join("");

        let finish_reason = match resp.stop_reason.as_deref() {
            Some("end_turn") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("tool_use") => FinishReason::ToolUse,
            _ => FinishReason::Stop,
        };

        LLMResponse {
            content,
            finish_reason,
            usage: TokenUsage {
                input_tokens: resp.usage.input_tokens,
                output_tokens: resp.usage.output_tokens,
            },
            tool_calls: None,
        }
    }
}
```

---

## 8. Memory Layout (Rust)

### Small Types (Stack Allocated)
```rust
// 24 bytes on 64-bit systems
pub struct TokenUsage {
    input_tokens: u32,      // 4 bytes
    output_tokens: u32,     // 4 bytes
    // + 16 bytes padding
}

// 8 bytes (enum discriminant + no data)
pub enum MessageRole {
    System,    // 0
    User,      // 1
    Assistant, // 2
    Tool,      // 3
}
```

### Large Types (Heap Allocated)
```rust
// String is 24 bytes (ptr + len + capacity)
// but actual data is on heap
pub struct Message {
    role: MessageRole,         // 8 bytes
    content: String,           // 24 bytes (ptr to heap)
    tool_calls: Option<Vec<_>>, // 24 bytes (ptr to heap)
}
// Total: ~56 bytes stack + heap data
```

### Zero-Copy Optimizations
```rust
// Using Cow for efficient cloning
use std::borrow::Cow;

pub struct MessageRef<'a> {
    role: MessageRole,
    content: Cow<'a, str>, // Borrows if possible, owns if needed
}
```

---

## Summary

### Type Safety Benefits
- **Compile-time validation**: Invalid types caught before runtime
- **No null pointers**: Option<T> for nullable values
- **Exhaustive pattern matching**: All enum cases handled
- **Memory safety**: No use-after-free, no buffer overflows

### Performance Characteristics
- **Small enums**: 8 bytes (discriminant)
- **Strings**: 24 bytes (stack) + data (heap)
- **Vectors**: 24 bytes (stack) + elements (heap)
- **Zero-copy**: Borrow when possible, clone when necessary

### Serialization
- **Serde**: Efficient JSON/YAML/TOML conversion
- **Compile-time codegen**: No reflection overhead
- **Custom serializers**: For special formats

---

**For API examples**: See [ARCHITECTURE.md](../ARCHITECTURE.md)
**For implementation**: See source code in `namra-core/*/src/`
