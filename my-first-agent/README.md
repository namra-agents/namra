# my-first-agent

A Nexus agent project.

## Getting Started

1. Install dependencies:
   ```bash
   # Add your API keys
   cp .env.example .env
   # Edit .env with your API keys
   ```

2. Validate your agent configuration:
   ```bash
   nexus validate agents/example_agent.yaml
   ```

3. Run your agent (coming soon):
   ```bash
   nexus run agents/example_agent.yaml
   ```

## Project Structure

```
my-first-agent/
├── agents/              # Agent configurations
│   └── example_agent.yaml
├── workflows/           # Workflow configurations
├── tools/               # Custom Python tools
├── .env.example         # Environment variables template
└── README.md
```

## Learn More

- [Nexus Documentation](https://docs.nexus.dev)
- [Agent Configuration Reference](https://docs.nexus.dev/config)
- [Examples](https://github.com/nexus-agents/nexus/tree/main/examples)
