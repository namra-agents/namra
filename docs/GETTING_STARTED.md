# Getting Started with Namra

This guide will help you install Namra and run your first AI agent.

## Installation

### Homebrew (macOS/Linux) - Recommended

```bash
brew tap namra-agents/tap
brew install namra
```

### Direct Download

Download the latest release for your platform from [GitHub Releases](https://github.com/namra-agents/namra/releases).

**macOS (Apple Silicon):**
```bash
curl -LO https://github.com/namra-agents/namra/releases/latest/download/namra-aarch64-apple-darwin.tar.gz
tar xzf namra-aarch64-apple-darwin.tar.gz
sudo mv namra /usr/local/bin/
```

**macOS (Intel):**
```bash
curl -LO https://github.com/namra-agents/namra/releases/latest/download/namra-x86_64-apple-darwin.tar.gz
tar xzf namra-x86_64-apple-darwin.tar.gz
sudo mv namra /usr/local/bin/
```

**Linux (x86_64):**
```bash
curl -LO https://github.com/namra-agents/namra/releases/latest/download/namra-x86_64-unknown-linux-gnu.tar.gz
tar xzf namra-x86_64-unknown-linux-gnu.tar.gz
sudo mv namra /usr/local/bin/
```

### Verify Installation

```bash
namra --version
# Output: namra 0.1.0
```

## Setup

### 1. Get an API Key

Namra currently supports Anthropic's Claude. Get your API key from [console.anthropic.com](https://console.anthropic.com).

### 2. Set Environment Variable

```bash
export ANTHROPIC_API_KEY=your-api-key-here
```

Add this to your `~/.bashrc` or `~/.zshrc` for persistence.

## Your First Agent

### 1. Initialize a New Project

```bash
namra init my-agent
cd my-agent
```

This creates the following project structure:

```
my-agent/
├── agents/                  # Agent configurations
│   └── example_agent.yaml   # Example agent to get started
├── workflows/               # Workflow configurations (for multi-step pipelines)
├── tools/                   # Custom Python tools (for advanced use cases)
├── .env.example             # Environment variables template
└── README.md                # Project documentation
```

### 2. Configure Your API Key

```bash
cp .env.example .env
# Edit .env and add your ANTHROPIC_API_KEY
```

Or just export it:
```bash
export ANTHROPIC_API_KEY=your-api-key-here
```

### 3. Run the Agent

```bash
namra run agents/example_agent.yaml --input "Hello! What can you help me with?"
```

The `run` command automatically validates your configuration before executing. If there are any errors in your YAML, you'll see them immediately.

### 4. View the Output

You'll see the agent's reasoning process and final answer:

```
Loading agent configuration...
✓ Loaded agent: example_agent

Using anthropic (claude-sonnet-4-5-20250929)
Available tools: calculator, string

Agent is thinking...

════════════════════════════════════════════════════════════
Agent Reasoning:

Step 1:
THINK: The user is greeting me and asking what I can help with...
ANSWER: Hello! I'm a helpful AI assistant...

════════════════════════════════════════════════════════════

Final Answer:
Hello! I'm a helpful AI assistant...

────────────────────────────────────────────────────────────
✓ Execution completed successfully
Iterations: 1
Tokens: 45
Cost: $0.0005
Time: 1.23s
```

## Validating Configurations

While `namra run` validates automatically, you can also validate configs without running them:

```bash
namra validate agents/example_agent.yaml
```

**What validation checks:**
- YAML syntax is correct
- Required fields exist (name, llm, system_prompt, etc.)
- Field types are valid (e.g., temperature is a number)
- Semantic rules (e.g., max_iterations > 0)

This is useful for:
- Checking configs before deploying
- Validating multiple files at once
- CI/CD pipelines

## Adding Tools

Agents become powerful when they can use tools. Here's an example with an HTTP tool:

```yaml
name: weather_agent
version: "1.0.0"

llm:
  provider: anthropic
  model: claude-sonnet-4-5-20250929
  temperature: 0.7
  max_tokens: 4096
  stream: true

tools:
  - type: builtin.http
    name: weather_api
    config:
      url: https://wttr.in
      method: GET
      headers:
        User-Agent: curl/7.68.0
      timeout: 10s

execution:
  max_iterations: 5
  timeout: 60s

system_prompt: |
  You are a weather assistant with access to the weather_api tool.

  To check weather, use:
  TOOL: weather_api({"path": "/CityName", "query": {"format": "3"}})

  Format your responses as:
  THINK: <reasoning>
  TOOL: <tool call if needed>
  OBSERVE: <tool result>
  ANSWER: <final answer>
```

Run it:
```bash
namra run weather_agent.yaml --input "What's the weather in Tokyo?"
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `namra init <name>` | Initialize a new project with folder structure |
| `namra validate <file>` | Validate configuration without running (optional) |
| `namra run <file> --input "..."` | Validate and run an agent |
| `namra version` | Display version |
| `namra help` | Show help |

## Next Steps

- Explore [example agents](../examples/README.md)
- Learn about [tool configuration](../TOOL_CONFIG_GUIDE.md)
- See [core tools documentation](../CORE_TOOLS.md)

## Troubleshooting

### "ANTHROPIC_API_KEY not set"

Make sure you've exported your API key:
```bash
export ANTHROPIC_API_KEY=your-key-here
```

### "Configuration validation failed"

Check the error message for details. Common issues:
- Missing required fields
- Invalid field types
- YAML syntax errors

### "Tool not found"

Ensure the tool is defined in your agent's `tools:` section with the correct type.

### "Request timeout"

Increase the timeout in your agent config:
```yaml
execution:
  timeout: 120s
```
