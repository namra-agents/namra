# Agent Reasoning Display

## Overview

The CLI now displays the agent's intermediate thoughts and reasoning at each step of execution, showing you exactly what the agent is thinking as it works through the problem.

## What Was Added

### 1. Thought Tracking in ExecutionContext
- New `thoughts: Vec<String>` field to store each reasoning step
- `record_thought()` method to capture agent's thinking
- Thoughts automatically collected during execution

### 2. ReAct Strategy Integration
- After each LLM call (THINK phase), the agent's response is recorded as a thought
- Includes the reasoning before tool calls
- Includes the reasoning for final answers

### 3. CLI Display
- Beautiful formatted output showing each step
- Step-by-step reasoning display
- Final answer highlighted separately

## Example Output

### Before (No Reasoning)
```
Agent is thinking...

Response:
A round-trip flight from NYC to LAX costs $350 USD.

────────────────────────────────────────────────────────────
✓ Execution completed successfully
```

### After (With Reasoning)
```
Agent is thinking...

════════════════════════════════════════════════════════════
Agent Reasoning:

Step 1:
I need to look up the flight price from NYC to LAX. Let me use the
flight_prices tool to get this information.

TOOL: flight_prices({"origin": "NYC", "destination": "LAX"})

Step 2:
Based on the tool result, the flight from NYC to LAX costs $350 USD
for a round-trip economy ticket. Let me provide this information to
the user.

ANSWER: A round-trip economy flight from New York to Los Angeles
costs approximately $350 USD.

════════════════════════════════════════════════════════════

Final Answer:
A round-trip economy flight from New York to Los Angeles costs
approximately $350 USD.

────────────────────────────────────────────────────────────
✓ Execution completed successfully
Iterations: 2
Tool calls (1)
  1. ✓ flight_prices (1ms)
────────────────────────────────────────────────────────────
```

## What You See

### Step-by-Step Breakdown

**Step 1: Initial Reasoning**
- Agent analyzes the user's question
- Decides which tool to use
- Formats the tool call

**Step 2: After Tool Result**
- Agent receives and processes tool output
- Reasons about the result
- Formulates final answer

**Step 3+: Additional Iterations**
- If the agent needs multiple tool calls, each gets its own step
- You can see the agent's thought process evolving

## Benefits

### 1. **Transparency**
See exactly what the agent is thinking at each step

### 2. **Debugging**
Understand why the agent made certain decisions or called specific tools

### 3. **Trust**
Watch the agent reason through complex problems step-by-step

### 4. **Learning**
Observe how the ReAct pattern works in practice

## Technical Details

### Data Flow

```
ReAct Strategy Loop
    │
    ├── LLM Call (THINK)
    │   └── response.content → context.record_thought()
    │
    ├── Parse Response (ACT)
    │   ├── Tool call? → Execute tool → Add observation
    │   └── Final answer? → Return answer
    │
    └── Repeat until done

ExecutionContext
    └── thoughts: Vec<String>

ExecutionResult
    └── thoughts: Vec<String> (cloned from context)

CLI Display
    └── Shows all thoughts with step numbers
```

### Code Changes

**Added to ExecutionContext:**
```rust
pub struct ExecutionContext {
    // ... other fields
    pub thoughts: Vec<String>,
}

pub fn record_thought(&mut self, thought: String) {
    self.thoughts.push(thought);
}
```

**Added to ExecutionResult:**
```rust
pub struct ExecutionResult {
    // ... other fields
    pub thoughts: Vec<String>,
}
```

**Modified in ReAct Strategy:**
```rust
// After LLM response
let response = llm.generate(request).await?;
context.record_thought(response.content.clone());
```

**Modified in CLI:**
```rust
// Display thoughts before final answer
if !result.thoughts.is_empty() {
    for (idx, thought) in result.thoughts.iter().enumerate() {
        println!("Step {}:", idx + 1);
        println!("{}", thought);
    }
}
```

## Testing

Try it with any agent:

**Flight prices:**
```bash
cargo run --bin nexus -- run test-agents/flight_agent.yaml \
  --input "How much is a flight from NYC to LAX?"
```

**Weather (with tool configuration):**
```bash
cargo run --bin nexus -- run test-agents/weather_agent.yaml \
  --input "What's the weather in London?"
```

**Complex reasoning:**
```bash
cargo run --bin nexus -- run test-agents/flight_agent.yaml \
  --input "Compare flights from NYC to LAX vs NYC to SFO and tell me which is cheaper"
```

## What Each Step Shows

### Tool-Based Reasoning
When the agent uses tools, you'll see:
1. Initial analysis of the question
2. Decision to use a specific tool
3. The tool call syntax
4. After receiving results, interpretation
5. Final answer formulation

### No-Tool Reasoning
Even for simple questions that don't need tools:
1. Understanding the question
2. Reasoning about the answer
3. Formulating the response

## Future Enhancements

### Potential Additions
1. **Real-time streaming** - Show thoughts as they happen
2. **Filtering** - Option to hide/show reasoning with `--verbose` flag
3. **Formatting** - Syntax highlighting for tool calls
4. **Truncation** - Summarize very long thoughts
5. **Export** - Save reasoning to file for analysis

## Comparison: Before vs After

| Aspect | Before | After |
|--------|--------|-------|
| Visibility | Black box | Transparent |
| Debugging | Difficult | Easy |
| Trust | Unknown process | Observable reasoning |
| Learning | Hidden | Educational |
| Token usage | Unknown breakdown | See per-step |

## Notes

- Thoughts are captured at **every iteration** of the ReAct loop
- The full LLM response is stored, including any reasoning text
- Tool observations (results) are not included in thoughts - they're in tool_calls
- The final answer is extracted from the last thought

## Conclusion

The reasoning display gives you complete visibility into the agent's decision-making process. You can now:

✅ See what the agent is thinking
✅ Understand why it calls specific tools
✅ Debug issues more easily
✅ Learn how ReAct works in practice
✅ Build trust in agent behavior

This makes the system more transparent, debuggable, and educational!
