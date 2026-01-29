# Namra Examples

Example agent configurations to get started with Namra.

## Agent Examples

### Weather Agent
A simple agent that looks up weather information.
```bash
namra run examples/agents/weather_agent.yaml --input "What's the weather in London?"
```

### File Manager Agent
An agent that manages files in a sandboxed directory.
```bash
namra run examples/agents/file_manager_agent.yaml --input "List the files"
```

### Poem Writer Agent
A creative agent that writes poems and saves them to files.
```bash
namra run examples/agents/poem_writer_agent.yaml --input "Write a poem about the stars"
```

## Observability Test Agents

### Jaeger Observability Test
Test OpenTelemetry tracing with Jaeger for distributed tracing.

**Prerequisites**:
```bash
# Start Jaeger
docker run -d --name jaeger -p 16686:16686 -p 4317:4317 jaegertracing/all-in-one:latest
```

**Run**:
```bash
namra run examples/agents/test_observability.yaml --input "Check my IP address"
```

**View traces**: http://localhost:16686

### Phoenix Observability Test
Test OpenTelemetry tracing with Arize Phoenix for LLM-specific observability.

**Prerequisites**:
```bash
# Start Phoenix
docker run -d --name phoenix -p 6006:6006 arizephoenix/phoenix:latest
```

**Run**:
```bash
namra run examples/agents/test_phoenix.yaml --input "Fetch info about headers"
```

**View traces**: http://localhost:6006

## Running Examples

1. Set your API key:
   ```bash
   export ANTHROPIC_API_KEY=your-key-here
   ```

2. Run an example:
   ```bash
   namra run examples/agents/<agent_name>.yaml --input "Your prompt here"
   ```

## Creating Your Own Agent

Copy one of the example agents and modify:
- `name` - Unique name for your agent
- `llm.model` - LLM model to use
- `tools` - Configure tools the agent can use
- `system_prompt` - Instructions for the agent

See [TOOL_CONFIG_GUIDE.md](../TOOL_CONFIG_GUIDE.md) for tool configuration details.
