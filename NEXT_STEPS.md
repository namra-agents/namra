# Next Steps - Quick Reference

**Current Status**: Week 2 Complete ✅
**Next Priority**: Week 3 - Built-in Tools 🚧

---

## What We've Built (Weeks 1-2)

✅ **Rust Workspace**: 9 crates, 3 complete
✅ **Config System**: Full YAML/TOML parsing with validation
✅ **LLM Integration**: Anthropic Claude Sonnet 4.5 with streaming
✅ **CLI Tool**: `namra init`, `validate`, `run` commands working
✅ **Cost Tracking**: Automatic token usage and cost calculation
✅ **Documentation**: README, ROADMAP, WEEK2_COMPLETE, examples

**Try it now**:
```bash
cd /Users/shobhit/Documents/dev/agent-framework
export ANTHROPIC_API_KEY=sk-ant-...

# Initialize project
./target/debug/namra init test-project
cd test-project

# Run agent with streaming
./target/debug/namra run agents/example_agent.yaml \
  --input "Write a haiku about Rust" \
  --stream
```

---

## Week 3: Built-in Tools (NEXT)

### Goal
Implement built-in tools so agents can perform actions (HTTP calls, file operations, calculations).

### Tasks Breakdown

#### 1. Set Up Dependencies (10 min)
Edit `namra-core/namra-tools/Cargo.toml`:
```toml
[dependencies]
tokio.workspace = true
async-trait.workspace = true
reqwest.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
```

#### 2. Create Tool Trait (45 min)
Create `namra-core/namra-tools/src/tool.rs`:
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    async fn execute(&self, input: serde_json::Value) -> Result<ToolOutput>;
}

pub struct ToolOutput {
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}
```

#### 3. Implement HTTP Tool (2 hours)
Create `namra-core/namra-tools/src/http.rs`:
- GET/POST/PUT/DELETE methods
- Headers and authentication
- JSON/text response parsing
- Timeout handling
- Error handling (network, timeout, HTTP errors)

#### 4. Implement File System Tool (1.5 hours)
Create `namra-core/namra-tools/src/filesystem.rs`:
- Read file
- Write file
- List directory
- Safety constraints (sandboxing considerations)

#### 5. Implement Built-in Tools (1 hour)
Create `namra-core/namra-tools/src/builtin.rs`:
- Calculator (basic arithmetic)
- String operations
- JSON manipulation

#### 6. Add Tests (1 hour)
- Unit tests for each tool
- Mock HTTP responses for HTTP tool
- File system tests with temp directories

---

## File Structure for Week 3

```
namra-core/namra-tools/
├── Cargo.toml                    # ← Update dependencies
├── src/
│   ├── lib.rs                    # ← Public API exports (NEW)
│   ├── tool.rs                   # ← Tool trait (NEW)
│   ├── http.rs                   # ← HTTP tool (NEW)
│   ├── filesystem.rs             # ← File operations (NEW)
│   ├── builtin.rs                # ← Calculator, etc. (NEW)
│   └── error.rs                  # ← Error types (NEW)
└── tests/
    ├── http_tests.rs             # ← HTTP tool tests (NEW)
    ├── filesystem_tests.rs       # ← Filesystem tests (NEW)
    └── builtin_tests.rs          # ← Built-in tool tests (NEW)
```

---

## Code Snippets to Get Started

### 1. Tool Trait Definition
```rust
use async_trait::async_trait;
use serde_json::{json, Value};
use anyhow::Result;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    // JSON Schema for parameters
    fn parameters(&self) -> Value;

    // Execute the tool
    async fn execute(&self, input: Value) -> Result<ToolOutput>;
}

pub struct ToolOutput {
    pub content: String,
    pub metadata: Option<Value>,
}
```

### 2. HTTP Tool Implementation
```rust
use reqwest::Client;

pub struct HttpTool {
    client: Client,
}

impl HttpTool {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Tool for HttpTool {
    fn name(&self) -> &str {
        "http"
    }

    fn description(&self) -> &str {
        "Make HTTP requests (GET, POST, PUT, DELETE)"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "method": {"type": "string", "enum": ["GET", "POST", "PUT", "DELETE"]},
                "url": {"type": "string"},
                "headers": {"type": "object"},
                "body": {"type": "string"}
            },
            "required": ["method", "url"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput> {
        let method = input["method"].as_str().unwrap();
        let url = input["url"].as_str().unwrap();

        let response = match method {
            "GET" => self.client.get(url).send().await?,
            "POST" => self.client.post(url).json(&input["body"]).send().await?,
            _ => todo!(),
        };

        let text = response.text().await?;
        Ok(ToolOutput {
            content: text,
            metadata: None,
        })
    }
}
```

---

## Testing Strategy

### Unit Tests (Mock HTTP)
```rust
#[tokio::test]
async fn test_http_tool_get() {
    let tool = HttpTool::new();
    let input = json!({
        "method": "GET",
        "url": "https://api.example.com/data"
    });
    // Use mockito or similar to mock HTTP responses
    let result = tool.execute(input).await.unwrap();
    assert!(result.content.contains("expected data"));
}
```

### File System Tests (Temp Directories)
```rust
#[tokio::test]
async fn test_filesystem_read_write() {
    let temp_dir = tempfile::tempdir().unwrap();
    let tool = FileSystemTool::new(temp_dir.path());

    // Test write
    let write_input = json!({
        "operation": "write",
        "path": "test.txt",
        "content": "Hello, World!"
    });
    tool.execute(write_input).await.unwrap();

    // Test read
    let read_input = json!({
        "operation": "read",
        "path": "test.txt"
    });
    let result = tool.execute(read_input).await.unwrap();
    assert_eq!(result.content, "Hello, World!");
}
```

Run tests:
```bash
cargo test -p namra-tools
```

---

## Expected Output (End of Week 3)

```bash
# Test HTTP tool directly (unit test)
cargo test -p namra-tools test_http_tool

# Test file system tool
cargo test -p namra-tools test_filesystem_tool

# Tools will be integrated into agent runtime in Week 4
# For now, we're building the tool implementations

# Example: Using tool directly in code
use namra_tools::{HttpTool, Tool};

let tool = HttpTool::new();
let result = tool.execute(json!({
    "method": "GET",
    "url": "https://api.github.com/users/octocat"
})).await?;

println!("Result: {}", result.content);
```

---

## Commands Reference

```bash
# Build and test
cargo build                          # Build all
cargo test -p namra-tools           # Test tools crate
cargo test -p namra-tools -- --nocapture  # See test output

# Check specific crate
cargo check -p namra-tools          # Fast compile check
cargo clippy -p namra-tools         # Lint

# Format code
cargo fmt                            # Format all
cargo fmt -p namra-tools            # Format one crate

# Run CLI (still works from Week 2)
cargo run --bin namra -- init test
cargo run --bin namra -- run agents/example_agent.yaml --input "Hello"
```

---

## Helpful Resources

### Tool System Design
- [LangChain Tools](https://python.langchain.com/docs/modules/agents/tools/) - Reference implementation
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling) - Tool spec format
- [JSON Schema](https://json-schema.org/) - For parameter definitions

### Rust Libraries
- [reqwest](https://docs.rs/reqwest) - HTTP client (already using)
- [tokio](https://docs.rs/tokio) - Async runtime (already using)
- [async-trait](https://docs.rs/async-trait) - Async traits (already using)
- [tempfile](https://docs.rs/tempfile) - For testing file operations
- [mockito](https://docs.rs/mockito) - For mocking HTTP in tests

### Project Files
- [Full Roadmap](ROADMAP.md) - 20-week plan
- [Week 2 Summary](WEEK2_COMPLETE.md) - What we just finished
- [Project Status](PROJECT_STATUS.md) - Current capabilities
- [README](README.md) - Project overview

---

## Questions to Consider

1. **Tool Security**: How to sandbox tools? (basic path validation for Week 3, full sandbox in Week 9)
2. **Timeout Handling**: What's a reasonable default timeout? (30 seconds for HTTP, 5 seconds for file ops)
3. **Error Handling**: How verbose should tool errors be? (detailed for debugging, can sanitize later)
4. **Tool Registry**: Static registration or dynamic loading? (static for built-in tools, dynamic for plugins in Week 7)

---

## Definition of Done (Week 3)

- [ ] `Tool` trait defined with async support
- [ ] HTTP tool implemented (GET, POST, PUT, DELETE)
- [ ] File system tool implemented (read, write, list)
- [ ] Built-in tools (calculator, string ops)
- [ ] Unit tests passing for all tools
- [ ] Error handling comprehensive
- [ ] Timeout mechanisms in place
- [ ] Documentation updated (inline docs)

---

## Tips for Next Session

1. **Start with the trait** - Define `Tool` trait first, get the API right
2. **Test as you go** - Write tests alongside implementation
3. **Keep it simple** - Tools should be single-purpose and focused
4. **Think about Week 4** - These tools will be called by the agent runtime
5. **Security later** - Basic validation now, full sandboxing in Week 9

---

**Ready to start?** Begin with `namra-tools/src/tool.rs`! 🚀
