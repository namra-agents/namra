//! Init command implementation

use anyhow::{Context, Result};
use console::style;
use std::fs;
use std::path::Path;

pub fn execute(name: &str, namespace: Option<&str>) -> Result<()> {
    println!("{}", style(format!("Initializing Nexus project: {}", name)).cyan().bold());
    println!();

    // Create project directory
    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir(project_dir)
        .with_context(|| format!("Failed to create directory '{}'", name))?;

    // Create subdirectories
    create_dir(project_dir, "agents")?;
    create_dir(project_dir, "workflows")?;
    create_dir(project_dir, "tools")?;

    // Create example agent configuration
    create_example_agent(project_dir, namespace)?;

    // Create .env.example
    create_env_example(project_dir)?;

    // Create README
    create_readme(project_dir, name)?;

    println!();
    println!("{}", style("✓ Project initialized successfully!").green().bold());
    println!();
    println!("Next steps:");
    println!("  1. cd {}", name);
    println!("  2. Edit agents/example_agent.yaml");
    println!("  3. Copy .env.example to .env and add your API keys");
    println!("  4. Run: nexus validate agents/example_agent.yaml");

    Ok(())
}

fn create_dir(base: &Path, name: &str) -> Result<()> {
    let dir = base.join(name);
    fs::create_dir(&dir)
        .with_context(|| format!("Failed to create directory '{}'", dir.display()))?;
    println!("  Created: {}/", style(name).green());
    Ok(())
}

fn create_example_agent(base: &Path, namespace: Option<&str>) -> Result<()> {
    let namespace_str = namespace.unwrap_or("default");

    let content = format!(r#"# Example Nexus Agent Configuration
name: example_agent
version: 1.0.0
description: An example agent to get you started

metadata:
  namespace: {}
  team: engineering
  owner: you@company.com

llm:
  provider: anthropic
  model: claude-sonnet-4-5-20250929
  temperature: 0.7
  max_tokens: 4096
  stream: true

tools: []

memory:
  type: in_memory
  ttl: 3600s
  max_messages: 50

middleware:
  observability:
    enabled: true
    trace_all_steps: true

  security:
    enabled: true
    validate_inputs: true

execution:
  strategy: react
  max_iterations: 10
  timeout: 60s

system_prompt: |
  You are a helpful AI assistant. Your goal is to assist users
  with their tasks efficiently and accurately.
"#, namespace_str);

    let file_path = base.join("agents/example_agent.yaml");
    fs::write(&file_path, content)
        .with_context(|| format!("Failed to write file '{}'", file_path.display()))?;

    println!("  Created: {}", style("agents/example_agent.yaml").green());
    Ok(())
}

fn create_env_example(base: &Path) -> Result<()> {
    let content = r#"# LLM Provider API Keys
ANTHROPIC_API_KEY=sk-ant-api03-...
# OPENAI_API_KEY=sk-...
# GOOGLE_API_KEY=...

# Optional: Observability
# JAEGER_ENDPOINT=http://localhost:14268/api/traces

# Optional: Redis for caching/memory
# REDIS_URL=redis://localhost:6379

# Optional: Database
# DATABASE_URL=postgresql://localhost/nexus
"#;

    let file_path = base.join(".env.example");
    fs::write(&file_path, content)
        .with_context(|| format!("Failed to write file '{}'", file_path.display()))?;

    println!("  Created: {}", style(".env.example").green());
    Ok(())
}

fn create_readme(base: &Path, name: &str) -> Result<()> {
    let content = format!(r#"# {}

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
{}/
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
"#, name, name);

    let file_path = base.join("README.md");
    fs::write(&file_path, content)
        .with_context(|| format!("Failed to write file '{}'", file_path.display()))?;

    println!("  Created: {}", style("README.md").green());
    Ok(())
}
