# Tool Testing Guide

**Current Status**: Week 3 Complete ✅
**What Works**: Direct tool usage, LLM chat
**What's Coming**: Agent runtime with automatic tool calling (Week 4)

---

## What You Can Do RIGHT NOW (Week 3)

### Option 1: Test Tools Directly ✅ (WORKS NOW!)

Run the tool testing example:

```bash
cd examples/tool-testing
cargo run
```

**Output**:
```
=== Nexus Tools Manual Testing ===

1. Testing Calculator Tool
   Expression: 25 * 4
   Result: 25 * 4 = 100
   Success: true
   Time: 0ms

2. Testing String Tool
   Operation: uppercase
   Text: hello from nexus
   Result: HELLO FROM NEXUS
   Time: 0ms

3. Testing HTTP Tool
   Method: GET
   URL: https://httpbin.org/get
   Success: true
   Response length: 220 bytes
   Time: 1865ms
   Status: 200

4. Testing Filesystem Tool
   Writing file: /tmp/nexus_test.txt
   Successfully wrote 23 bytes to file
   ...
```

### Option 2: Use LLM Without Tools ✅ (WORKS NOW!)

You can chat with Claude, but it won't call tools automatically yet:

```bash
cd /path/to/agent-framework
export ANTHROPIC_API_KEY=sk-ant-...

nexus run my-first-agent/agents/example_agent.yaml \
  --input "What is 25 * 4?" \
  --stream
```

**What happens**:
- ✅ Claude will respond with text answer: "100"
- ❌ Claude **cannot** call the calculator tool automatically

---

## What You'll Be Able to Do (Week 4 - Coming Soon!)

### Full Agent with Tool Calling

After Week 4, the agent will **automatically** decide when to use tools:

```bash
nexus run agents/calculator_agent.yaml \
  --input "What is 25 * 4?"
```

**Expected Flow** (Week 4):
```
[Agent] Thinking: I need to calculate 25 * 4. I'll use the calculator tool.
[Agent] Action: calculator.execute({"expression": "25 * 4"})
[Tool] Result: "25 * 4 = 100"
[Agent] Response: The answer is 100.

Cost: $0.002 | Time: 1.2s | Iterations: 2
```

---

## Step-by-Step: Testing Tools NOW

### 1. Clone and Build

```bash
cd /Users/shobhit/Documents/dev/agent-framework
cargo build --release
```

### 2. Test Individual Tools

#### Calculator Tool

```bash
cd examples/tool-testing
cargo run
```

Or write your own:

```rust
use nexus_tools::{CalculatorTool, Tool};
use serde_json::json;

#[tokio::main]
async fn main() {
    let tool = CalculatorTool::new();

    let result = tool.execute(json!({
        "expression": "2 + 2"
    })).await.unwrap();

    println!("Result: {}", result.content); // "2 + 2 = 4"
}
```

#### HTTP Tool

```rust
use nexus_tools::{HttpTool, Tool};
use serde_json::json;

#[tokio::main]
async fn main() {
    let tool = HttpTool::new();

    let result = tool.execute(json!({
        "method": "GET",
        "url": "https://api.github.com/users/octocat"
    })).await.unwrap();

    println!("GitHub user data: {}", result.content);
}
```

#### Filesystem Tool

```rust
use nexus_tools::{FileSystemTool, Tool};
use serde_json::json;

#[tokio::main]
async fn main() {
    let tool = FileSystemTool::new();

    // Write
    tool.execute(json!({
        "operation": "write",
        "path": "/tmp/test.txt",
        "content": "Hello, Nexus!"
    })).await.unwrap();

    // Read
    let result = tool.execute(json!({
        "operation": "read",
        "path": "/tmp/test.txt"
    })).await.unwrap();

    println!("File content: {}", result.content);
}
```

#### String Tool

```rust
use nexus_tools::{StringTool, Tool};
use serde_json::json;

#[tokio::main]
async fn main() {
    let tool = StringTool::new();

    let result = tool.execute(json!({
        "operation": "uppercase",
        "text": "hello world"
    })).await.unwrap();

    println!("Result: {}", result.content); // "HELLO WORLD"
}
```

### 3. Test LLM (Without Tools)

Current capability:

```bash
export ANTHROPIC_API_KEY=sk-ant-...

nexus run my-first-agent/agents/example_agent.yaml \
  --input "Write a haiku about Rust" \
  --stream
```

**Output**:
```
Loading agent configuration...
✓ Loaded agent: example_agent

Using anthropic (claude-sonnet-4-5-20250929)

Agent:
Rust's memory safe,
Compiler guards every step,
Zero-cost abstracts.
────────────────────────────────────────────────────────
Tokens: 45 tokens (input: 20, output: 25)
Cost: $0.0004
────────────────────────────────────────────────────────
```

---

## Week 4: The Missing Piece - Agent Runtime

### What Week 4 Will Add

```
Week 3 (NOW):     Week 4 (SOON):
┌──────────┐      ┌──────────┐      ┌──────────┐
│   LLM    │      │   LLM    │◄────►│ Runtime  │
│ (Claude) │      │ (Claude) │      │ (ReAct)  │
└──────────┘      └──────────┘      └────┬─────┘
                                         │
┌──────────┐                        ┌────▼─────┐
│  Tools   │      │  Tools   │◄─────┤ Executor │
│ (Manual) │      │ (Auto!)  │      └──────────┘
└──────────┘      └──────────┘
```

### ReAct Loop (Week 4)

The agent will follow this pattern:

1. **THINK**: Reason about what needs to be done
2. **ACT**: Decide to use a tool or respond
3. **OBSERVE**: Get tool result
4. **REPEAT**: Until task complete

### Example Agent Config (Week 4)

```yaml
# agents/calculator_agent.yaml
name: calculator_agent
version: 1.0.0

llm:
  provider: anthropic
  model: claude-sonnet-4-5-20250929
  temperature: 0.0
  max_tokens: 2048

tools:
  - name: calculator
    type: builtin.calculator
    description: "Perform arithmetic calculations"

execution:
  strategy: react
  max_iterations: 5
  timeout: 30s

system_prompt: |
  You are a helpful assistant that can perform calculations.
  When given a math problem, use the calculator tool.
  Always show your work.
```

### Running It (Week 4)

```bash
# This will work after Week 4:
nexus run agents/calculator_agent.yaml \
  --input "What is (25 * 4) + (100 / 2)?"

# Expected output:
[Agent] Thinking: I need to calculate (25 * 4) + (100 / 2)
[Agent] Action: calculator(expression="25 * 4")
[Tool] Result: 100
[Agent] Action: calculator(expression="100 / 2")
[Tool] Result: 50
[Agent] Action: calculator(expression="100 + 50")
[Tool] Result: 150
[Agent] Response: The answer is 150.

  Breakdown:
  - 25 * 4 = 100
  - 100 / 2 = 50
  - 100 + 50 = 150

Cost: $0.008 | Time: 3.5s | Iterations: 4
```

---

## FAQ

### Q: Can I make agents call tools now?

**A**: Not automatically. You can:
1. ✅ Call tools directly in Rust code (see examples above)
2. ✅ Chat with LLM (but it won't call tools)
3. ❌ Have agent automatically decide to use tools (Week 4)

### Q: What's stopping automatic tool calling?

**A**: The **agent runtime** (Week 4). This component:
- Parses LLM responses for tool calls
- Executes the requested tools
- Feeds results back to the LLM
- Repeats until task is complete

### Q: Can I build the runtime myself?

**A**: Yes! If you want to get started early, the runtime needs:
1. `Executor` - Runs the agent loop
2. `Context` - Tracks conversation and state
3. `ReAct Strategy` - Implements Think→Act→Observe pattern

See `ROADMAP.md` Week 4 for details.

### Q: How do I test if my tool works?

**A**: Run the tool testing example:

```bash
cd examples/tool-testing
cargo run
```

All 4 tools (HTTP, Filesystem, Calculator, String) will be tested.

### Q: Can tools call other tools?

**A**: Not directly. Tool calling is orchestrated by the agent runtime:

```
Agent → Tool A → Agent → Tool B → Agent → Response
```

The agent decides the sequence. This prevents infinite loops and ensures the agent remains in control.

---

## What to Expect in Week 4

### Timeline
- **Start**: After Week 3 commit
- **Duration**: ~6-8 hours
- **End**: Full MVP with tool calling

### Deliverables
1. `nexus-runtime` crate
2. Agent executor
3. ReAct strategy implementation
4. Tool calling integration
5. End-to-end tests

### Test Command (Week 4)
```bash
# This will work after Week 4:
nexus run agents/research_agent.yaml \
  --input "What's the weather in San Francisco and write it to weather.txt"

# Agent will:
# 1. Use HTTP tool to call weather API
# 2. Use filesystem tool to write result
# 3. Respond with summary
```

---

## Current Capabilities Summary

| Feature | Status | Notes |
|---------|--------|-------|
| **Tools** | ✅ Works | HTTP, Filesystem, Calculator, String |
| **LLM Chat** | ✅ Works | Anthropic Claude with streaming |
| **Config System** | ✅ Works | YAML-based agent configs |
| **CLI** | ✅ Works | init, validate, run commands |
| **Tool Calling** | ❌ Week 4 | Agent can't automatically use tools |
| **ReAct Loop** | ❌ Week 4 | Think→Act→Observe pattern |
| **Multi-Step Tasks** | ❌ Week 4 | Requires agent runtime |

---

## Next Steps

1. **Now**: Test tools with `cargo run` in `examples/tool-testing`
2. **Week 4**: Implement agent runtime
3. **After Week 4**: Build complete agents that use tools automatically

See [NEXT_STEPS.md](../NEXT_STEPS.md) for Week 4 implementation guide.
