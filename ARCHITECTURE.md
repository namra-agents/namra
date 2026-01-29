# Namra Architecture Overview

**Last Updated**: January 28, 2026
**Status**: Week 2 Complete (MVP 50% done)

---

## High-Level Architecture

Namra is a **config-driven agent framework** with a **Rust core runtime**. The key design principle is: **define agents in YAML, execute in Rust**.

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Layer                              │
│  • YAML Configuration Files                                     │
│  • CLI Commands (namra init, validate, run)                     │
│  • Future: Python SDK, REST API                                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      Namra Core (Rust)                          │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │ namra-config │  │  namra-cli   │  │  namra-llm   │        │
│  │              │  │              │  │              │        │
│  │ • YAML Parse │  │ • init       │  │ • Anthropic  │        │
│  │ • Validation │  │ • validate   │  │ • Streaming  │        │
│  │ • Types      │  │ • run        │  │ • Cost Track │        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
│         ↓                  ↓                  ↓                │
│  ┌──────────────────────────────────────────────────┐         │
│  │           namra-runtime (Week 4)                 │         │
│  │  • Agent Executor                                │         │
│  │  • ReAct Strategy Loop                           │         │
│  │  • Tool Calling Orchestration                    │         │
│  └──────────────────────────────────────────────────┘         │
│         ↓                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │ namra-tools  │  │ namra-memory │  │ namra-mdware │        │
│  │ (Week 3)     │  │ (Week 12)    │  │ (Week 9-11)  │        │
│  │ • HTTP       │  │ • In-memory  │  │ • Observ.    │        │
│  │ • Filesystem │  │ • Redis      │  │ • Security   │        │
│  │ • Calculator │  │ • Vector DB  │  │ • Govern.    │        │
│  └──────────────┘  └──────────────┘  └──────────────┘        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      External Services                          │
│  • Anthropic API (Claude)                                       │
│  • Future: OpenAI, Google, Bedrock                              │
│  • Storage: Redis, PostgreSQL, Vector DBs                       │
│  • Observability: Jaeger, Prometheus                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core Components (Implemented)

### 1. **namra-config** ✅ Complete
**Purpose**: Parse and validate agent configuration files

**Responsibilities**:
- Parse YAML/TOML configuration files
- Validate agent definitions (LLM settings, tools, memory, etc.)
- Type-safe configuration structs
- Error reporting with context

**Key Types**:
```rust
pub struct AgentConfig {
    pub name: String,
    pub version: String,
    pub llm: LLMConfig,
    pub tools: Vec<ToolConfig>,
    pub memory: MemoryConfig,
    pub execution: ExecutionConfig,
    pub system_prompt: String,
}
```

**Status**:
- ~800 LOC
- 4 unit tests
- Supports YAML and TOML

---

### 2. **namra-cli** ✅ Complete
**Purpose**: Command-line interface for agent operations

**Commands Implemented**:
- `namra init <name>` - Scaffold new agent project
- `namra validate <file>` - Validate configuration
- `namra run <file>` - Execute agent with LLM
- `namra version` - Show version info

**Features**:
- Colored output (success/error highlighting)
- Progress indicators for long operations
- Helpful error messages
- Environment variable support (ANTHROPIC_API_KEY)

**Example Usage**:
```bash
# Initialize project
namra init my-agent
cd my-agent

# Validate config
namra validate agents/example_agent.yaml

# Run agent (non-streaming)
namra run agents/example_agent.yaml --input "Hello!"

# Run agent (streaming)
namra run agents/example_agent.yaml --input "Tell me a story" --stream
```

**Status**:
- ~700 LOC
- Async runtime (Tokio)
- Clap for CLI parsing

---

### 3. **namra-llm** ✅ Complete
**Purpose**: LLM provider integrations

**Architecture**:
```rust
// Trait for all LLM providers
#[async_trait]
pub trait LLMAdapter: Send + Sync {
    async fn generate(&self, request: LLMRequest) -> Result<LLMResponse>;
    async fn stream(&self, request: LLMRequest) -> Result<LLMStream>;
    fn provider_name(&self) -> &str;
    fn estimate_cost(&self, tokens: &TokenUsage) -> f64;
}

// Currently implemented
pub struct AnthropicAdapter {
    client: reqwest::Client,
    api_key: String,
    model: String,
}
```

**Features**:
- **Non-streaming mode**: Wait for complete response
- **Streaming mode**: Real-time SSE (Server-Sent Events)
- **Cost calculation**: Automatic per-request cost tracking
- **Token usage**: Input/output token counting
- **Error handling**: 401 (auth), 429 (rate limit), 400 (validation), 500 (server errors)
- **Model support**: Claude Sonnet 4.5, Sonnet 3.5, Opus, Haiku

**Key Types**:
```rust
pub struct LLMRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

pub struct LLMResponse {
    pub content: String,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
}

pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

**Cost Calculation**:
```rust
// Claude Sonnet 4.5 pricing
Input:  $3.00 / 1M tokens
Output: $15.00 / 1M tokens

// Example: 1000 input + 500 output tokens
Cost = (1000/1M × $3) + (500/1M × $15) = $0.0105
```

**Status**:
- ~1,400 LOC
- 5 unit tests (1 ignored integration test)
- Full Anthropic integration
- OpenAI adapter deferred to Week 5+

---

## Data Flow: Current System

### Scenario: User runs an agent

```
1. USER INPUT
   ┌─────────────────────────────────────┐
   │ $ namra run agent.yaml --input "Hi" │
   └─────────────────────────────────────┘
                  ↓
2. CLI PARSING (namra-cli)
   ┌─────────────────────────────────────┐
   │ • Parse command-line args           │
   │ • Read agent.yaml file              │
   │ • Extract --input parameter         │
   └─────────────────────────────────────┘
                  ↓
3. CONFIG LOADING (namra-config)
   ┌─────────────────────────────────────┐
   │ • Parse YAML → AgentConfig struct   │
   │ • Validate LLM settings             │
   │ • Validate tools, memory, etc.      │
   └─────────────────────────────────────┘
                  ↓
4. LLM ADAPTER SETUP (namra-llm)
   ┌─────────────────────────────────────┐
   │ • Read ANTHROPIC_API_KEY            │
   │ • Initialize AnthropicAdapter       │
   │ • Set model, temperature, max_tokens│
   └─────────────────────────────────────┘
                  ↓
5. LLM REQUEST (namra-llm)
   ┌─────────────────────────────────────┐
   │ • Build LLMRequest:                 │
   │   - messages: [system, user]        │
   │   - model: claude-sonnet-4-5-...    │
   │   - temperature: 0.7                │
   │ • POST to Anthropic API             │
   └─────────────────────────────────────┘
                  ↓
6. ANTHROPIC API
   ┌─────────────────────────────────────┐
   │ POST https://api.anthropic.com/v1/  │
   │      messages                        │
   │                                      │
   │ Headers:                             │
   │   x-api-key: sk-ant-...             │
   │   anthropic-version: 2023-06-01     │
   └─────────────────────────────────────┘
                  ↓
7. STREAMING RESPONSE (if --stream)
   ┌─────────────────────────────────────┐
   │ event: message_start                │
   │ event: content_block_start          │
   │ event: content_block_delta          │
   │   data: {"text": "Hello"}           │
   │ event: content_block_delta          │
   │   data: {"text": " there"}          │
   │ event: message_stop                 │
   └─────────────────────────────────────┘
                  ↓
8. OUTPUT RENDERING (namra-cli)
   ┌─────────────────────────────────────┐
   │ Agent:                              │
   │ Hello there! How can I help you?    │
   │ ─────────────────────────────────   │
   │ Tokens: 23 tokens (in: 15, out: 8) │
   │ Cost: $0.0002                       │
   │ ─────────────────────────────────   │
   └─────────────────────────────────────┘
```

---

## Component Details

### Configuration System (namra-config)

**File Format**:
```yaml
name: example_agent
version: 1.0.0

llm:
  provider: anthropic                    # ← LLM provider
  model: claude-sonnet-4-5-20250929     # ← Model name
  temperature: 0.7                       # ← Randomness (0-1)
  max_tokens: 4096                       # ← Response length limit

tools:                                   # ← Available tools (Week 3+)
  - name: web_search
    type: builtin.http
    config:
      url: https://api.tavily.com/search

memory:                                  # ← Conversation memory (Week 12+)
  type: in_memory
  max_messages: 100

execution:                               # ← Execution strategy (Week 4+)
  strategy: react                        # ← ReAct pattern
  max_iterations: 10                     # ← Stop after N loops
  timeout: 60s                           # ← Total timeout

system_prompt: |                         # ← System instructions
  You are a helpful assistant.
```

**Validation Rules**:
- `temperature`: Must be between 0.0 and 1.0
- `max_tokens`: Must be positive, typically ≤ 100,000
- `provider`: Must be valid (anthropic, openai, etc.)
- `model`: Must exist for the provider
- `tools`: Must reference valid tool types
- `execution.strategy`: Must be valid (react, chain, etc.)

---

### LLM Adapter (namra-llm)

**Request Flow**:
```rust
// 1. Create adapter
let adapter = AnthropicAdapter::builder()
    .api_key(env::var("ANTHROPIC_API_KEY")?)
    .model("claude-sonnet-4-5-20250929")
    .build();

// 2. Build request
let request = LLMRequest {
    messages: vec![
        Message::system("You are a helpful assistant"),
        Message::user("What is 2+2?"),
    ],
    model: "claude-sonnet-4-5-20250929".to_string(),
    temperature: 0.7,
    max_tokens: 1024,
    stream: false,
};

// 3. Generate response
let response = adapter.generate(request).await?;

// 4. Access result
println!("Response: {}", response.content);
println!("Cost: ${:.4}", adapter.estimate_cost(&response.usage));
```

**Streaming Flow**:
```rust
// 1. Create streaming request
let request = LLMRequest {
    stream: true,
    // ... other fields
};

// 2. Get stream
let mut stream = adapter.stream(request).await?;

// 3. Process chunks
while let Some(chunk) = stream.next().await {
    match chunk? {
        StreamChunk::Content(text) => print!("{}", text),
        StreamChunk::Done(usage) => {
            println!("\n\nTokens: {}", usage.input_tokens + usage.output_tokens);
        }
    }
}
```

**Error Handling**:
```rust
pub enum LLMError {
    Unauthorized,           // 401 - Invalid API key
    RateLimited,           // 429 - Too many requests
    InvalidRequest(String), // 400 - Bad parameters
    ModelNotFound,         // 404 - Model doesn't exist
    ServerError,           // 500+ - Anthropic issue
    NetworkError(String),  // Connection problems
}
```

---

## Stub Components (Not Yet Implemented)

### 4. **namra-tools** 🚧 Week 3
**Purpose**: Built-in tools for agents

**Planned Tools**:
- **HTTP Tool**: Make REST API calls
- **File System Tool**: Read/write files
- **Calculator Tool**: Basic arithmetic
- **Database Tool**: SQL queries
- **Shell Tool**: Execute commands

**Interface**:
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value; // JSON Schema
    async fn execute(&self, input: serde_json::Value) -> Result<ToolOutput>;
}
```

---

### 5. **namra-runtime** 📅 Week 4 (MVP Goal)
**Purpose**: Agent execution engine

**Responsibilities**:
- Execute agents using ReAct pattern (Reasoning + Action loop)
- Manage conversation history
- Call tools based on LLM decisions
- Handle stop conditions (max iterations, timeout)
- Error recovery

**ReAct Loop**:
```
1. Agent receives task
2. THINK: LLM reasons about what to do
3. ACT: LLM decides to use a tool (or respond)
4. OBSERVE: Execute tool, get result
5. Repeat steps 2-4 until task complete
6. Return final answer
```

**Example**:
```
User: "What's the weather in San Francisco?"

Iteration 1:
  THINK: "I need to check the weather, I'll use the web_search tool"
  ACT: Call web_search("weather San Francisco")
  OBSERVE: "Current temp: 65°F, Sunny"

Iteration 2:
  THINK: "I have the weather info, I can answer now"
  ACT: Respond to user
  OUTPUT: "It's currently 65°F and sunny in San Francisco!"
```

---

### 6. **namra-memory** 📅 Week 12
**Purpose**: Conversation and context storage

**Implementations**:
- **In-Memory**: Simple Vec<Message> storage
- **Redis**: Persistent key-value storage
- **PostgreSQL**: Relational storage with search
- **Vector DB**: Semantic search (Pinecone, Weaviate, Qdrant)

---

### 7. **namra-middleware** 📅 Weeks 9-11
**Purpose**: Cross-cutting concerns

**Types**:
- **Observability**: OpenTelemetry tracing, metrics, logs
- **Security**: Input validation, rate limiting, auth
- **Governance**: Policy enforcement, cost tracking, compliance

---

### 8. **namra-api** 📅 Week 5
**Purpose**: gRPC/HTTP server for remote execution

**Endpoints**:
- `POST /v1/agents/execute` - Run agent
- `POST /v1/agents/validate` - Validate config
- `GET /v1/agents/{id}/status` - Check status

---

### 9. **namra-plugin** 📅 Week 7
**Purpose**: Python custom tool integration

Allows users to write custom tools in Python:
```python
from namra import tool

@tool
def custom_search(query: str) -> str:
    """Search internal knowledge base"""
    # Custom logic here
    return results
```

---

## Design Principles

### 1. **Config-Driven Architecture**
- **80% use cases**: No code required, just YAML
- **20% use cases**: Custom tools via Python plugins
- **Benefits**:
  - Non-programmers can create agents
  - Version control configurations (GitOps)
  - No recompilation needed

### 2. **Performance-First**
- **Rust core**: 10-100× faster than Python
- **Zero-copy parsing**: Efficient memory usage
- **Async everywhere**: Tokio for concurrency
- **Streaming**: Real-time responses

### 3. **LLM Agnostic**
- Unified `LLMAdapter` trait
- Easy to add new providers
- Swap providers via config only

### 4. **Enterprise-Native**
- Observability built-in (not bolted-on)
- Security by default
- Multi-tenancy support
- Cost tracking

### 5. **Production-Ready**
- Single binary deployment
- Kubernetes-ready (but not required)
- Graceful shutdown
- Circuit breakers and retries

---

## Technology Stack

### Core
- **Language**: Rust 1.75+
- **Async Runtime**: Tokio
- **Serialization**: Serde (JSON, YAML, TOML)
- **HTTP Client**: Reqwest
- **CLI Framework**: Clap

### Future
- **gRPC**: Tonic
- **HTTP Server**: Axum
- **Observability**: OpenTelemetry
- **Testing**: Mockito, Criterion (benchmarks)

---

## Performance Characteristics

### Current (Week 2)
- **Binary Size**: ~12 MB (debug), ~6 MB (release)
- **Cold Start**: <5ms for CLI commands
- **Memory**: <10 MB for CLI operations
- **Compile Time**: ~25s (clean), ~2s (incremental)
- **LLM Latency**: ~500-2000ms (network-bound, depends on Anthropic)

### Target (MVP - Week 4)
- **Cold Start**: <20ms for agent execution
- **Memory**: <50MB per agent instance
- **Throughput**: 100+ requests/second (single instance)
- **Binary Size**: <10MB (release)

---

## Comparison to Similar Systems

| Feature | Namra | LangChain | AutoGPT | Agno |
|---------|-------|-----------|---------|------|
| **Language** | Rust | Python | Python | Python |
| **Config** | YAML | Code | JSON | Code |
| **Cold Start** | <20ms | ~500ms | ~1000ms | ~150ms |
| **Memory** | <50MB | ~300MB | ~500MB | ~80MB |
| **LLM Agnostic** | ✅ | ✅ | ⚠️ OpenAI-first | ✅ |
| **Streaming** | ✅ | ✅ | ❌ | ✅ |
| **Tools** | Built-in + Python | Python only | Python only | Python + Go |
| **Observability** | Built-in | 3rd party | Limited | Basic |
| **Multi-Agent** | ✅ (Week 13) | ✅ | ✅ | ⚠️ |

---

## Security Considerations

### Current (Week 2)
- ✅ API keys from environment variables (not hardcoded)
- ✅ HTTPS for all LLM API calls
- ❌ No input sanitization yet
- ❌ No rate limiting yet
- ❌ No secret management yet

### Planned (Week 9-11)
- Input validation and sanitization
- Rate limiting (per-user, per-agent)
- Secret management (HashiCorp Vault integration)
- RBAC (Role-Based Access Control)
- Audit logging

---

## What's Next: Week 3 & 4

### Week 3: Tools
```
namra-tools/
├── src/
│   ├── tool.rs          # Tool trait
│   ├── http.rs          # HTTP tool
│   ├── filesystem.rs    # File operations
│   └── builtin.rs       # Calculator, etc.
```

**Goal**: Agents can perform actions (not just chat)

### Week 4: Runtime (MVP!)
```
namra-runtime/
├── src/
│   ├── executor.rs      # Agent executor
│   ├── context.rs       # Execution context
│   └── strategies/
│       └── react.rs     # ReAct pattern
```

**Goal**: Complete agent loop with tool calling

**MVP Demo** (End of Week 4):
```bash
$ namra run research_agent.yaml \
  --input "Find the latest Rust news and summarize"

[Agent] Thinking: I need to search for Rust news
[Agent] Action: Using web_search tool...
[Tool] Found 5 articles about Rust
[Agent] Thinking: I'll summarize these articles
[Agent] Response: Here's a summary of the latest Rust news...

Cost: $0.045
Time: 3.2s
Tokens: 1,234
```

---

## Project Timeline

```
Week 1-2:  Foundation (Config, CLI, LLM)          ✅ DONE
Week 3:    Tools (HTTP, Filesystem, Calculator)   🚧 IN PROGRESS
Week 4:    Runtime (ReAct, Tool Loop)             📅 NEXT WEEK
──────────────────────────────────────────────────
Week 5-8:  API & Python SDK                       📅 FUTURE
Week 9-12: Observability & Resilience            📅 FUTURE
Week 13-16: Multi-Agent & Workflows              📅 FUTURE
Week 17-20: Enterprise & Polish                  📅 FUTURE
```

**MVP Target**: End of Week 4
**Production-Ready Target**: End of Week 20

---

## Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace definition |
| `PROJECT_STATUS.md` | Current status, metrics |
| `ROADMAP.md` | 20-week implementation plan |
| `NEXT_STEPS.md` | Step-by-step guide for current week |
| `WEEK2_COMPLETE.md` | Week 2 summary |
| `README.md` | Project overview |
| `ARCHITECTURE.md` | This file - system architecture |

---

**For detailed implementation**: See [NEXT_STEPS.md](NEXT_STEPS.md)
**For project timeline**: See [ROADMAP.md](ROADMAP.md)
**For current status**: See [PROJECT_STATUS.md](PROJECT_STATUS.md)
