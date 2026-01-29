# Namra: Enterprise Agent Framework

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

**Namra** is an enterprise-grade AI agent framework with a **Rust core runtime** and **declarative YAML configuration**. Build AI agents without code, deploy anywhere, scale to production.

## Why Namra?


1. **Config-First**: Define agents in YAML - no code required for 80% of use cases
2. **Rust Performance**: Faster than Python frameworks, minimal memory footprint
3. **LLM Agnostic**: Use Anthropic, OpenAI, Google, AWS Bedrock, or local models
4. **Enterprise-Native**: Observability, security, and governance built into the core
5. **Zero Dependencies**: Single static binary, no Python/Node required

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

### From Source (requires Rust)

```bash
git clone https://github.com/namra-agents/namra
cd namra
cargo build --release
./target/release/namra --version
```

### Verify Installation

```bash
namra --version
# Output: namra 0.1.0
```

## Quick Start

### 1. Set Up Your API Key

```bash
export ANTHROPIC_API_KEY=your-api-key-here
```

### 2. Initialize a New Project

```bash
namra init my-agent
cd my-agent
```

This creates the following project structure:

```
my-agent/
├── agents/                  # Agent configurations
│   └── example_agent.yaml   # Example agent to get started
├── workflows/               # Workflow configurations
├── tools/                   # Custom tools
├── .env.example             # Environment variables template
└── README.md                # Project documentation
```

### 3. Run the Agent

```bash
namra run agents/example_agent.yaml --input "Hello! What can you help me with?"
```

The `run` command automatically validates your configuration before executing.

## Example Agent Configuration

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

## CLI Commands

| Command | Description |
|---------|-------------|
| `namra init <name>` | Initialize a new project with folder structure |
| `namra validate <file>` | Validate configuration without running |
| `namra run <file> --input "..."` | Validate and run an agent |
| `namra version` | Display version |
| `namra help` | Show help |

## Architecture

```
┌─────────────────────────────────────────────────┐
│         User Interface (YAML/CLI/SDK)           │
└─────────────────────────────────────────────────┘
                      │
┌─────────────────────────────────────────────────┐
│           Namra Runtime (Rust Core)             │
│  • ReAct Engine    • Tool System                │
│  • LLM Adapters    • Memory                     │
│  • Observability   • Security                   │
└─────────────────────────────────────────────────┘
                      │
┌─────────────────────────────────────────────────┐
│  LLM APIs  │  Tools  │  Memory  │  Monitoring  │
└─────────────────────────────────────────────────┘
```

## Features

### Core Capabilities
- **ReAct Pattern**: Reasoning and action loop with THINK/TOOL/OBSERVE/ANSWER
- **Tool System**: Built-in tools (HTTP, Filesystem, Calculator, String) + custom tools
- **Memory**: In-memory, Redis, PostgreSQL support
- **Streaming**: Real-time streaming responses

### Built-in Tools
- **HTTP**: Make HTTP requests to external APIs
- **Filesystem**: Read/write files with configurable base paths
- **Calculator**: Mathematical operations
- **String**: Text manipulation utilities

### LLM Providers
- Anthropic (Claude Sonnet, Opus, Haiku)
- OpenAI (GPT-4, GPT-3.5) - coming soon
- Google (Gemini) - coming soon
- Local models (Ollama) - coming soon

## Project Structure

```
namra/
├── namra-core/
│   ├── namra-config/         # Configuration parsing
│   ├── namra-llm/            # LLM adapters
│   ├── namra-tools/          # Built-in tools
│   ├── namra-memory/         # Memory implementations
│   ├── namra-middleware/     # Middleware system
│   ├── namra-plugin/         # Plugin system
│   ├── namra-api/            # gRPC/HTTP API
│   ├── namra-runtime/        # Core runtime
│   └── namra-cli/            # CLI tool
├── examples/                 # Example configurations
├── docs/                     # Documentation
└── deployment/               # Deployment configs
```

## Documentation

- [Getting Started](docs/GETTING_STARTED.md)
- [Architecture Overview](ARCHITECTURE.md)
- [Core Tools Reference](CORE_TOOLS.md)
- [Tool Configuration Guide](TOOL_CONFIG_GUIDE.md)
- [Examples](examples/)

## Development

### Prerequisites
- Rust 1.75+ (only for contributors)

### Build from Source
```bash
git clone https://github.com/namra-agents/namra
cd namra
cargo build --release
cargo test
./target/release/namra --help
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

## Community & Support

- [Report Issues](https://github.com/namra-agents/namra/issues)
- [Discussions](https://github.com/namra-agents/namra/discussions)
- [Examples](examples/)

---

Built with Rust by the Namra team.
