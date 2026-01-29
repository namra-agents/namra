//! Run command implementation

use anyhow::{Context, Result};
use console::style;
use namra_config::{parse_agent_config, validate_config};
use namra_llm::{AnthropicAdapter, LLMAdapter};
use namra_runtime::{AgentExecutorBuilder, ReActStrategy, ToolFactory};
use std::env;
use std::path::Path;
use std::sync::Arc;

pub async fn execute(config_path: &Path, input: &str, _stream: bool) -> Result<()> {
    println!("{}", style("Loading agent configuration...").cyan());

    // Parse configuration
    let config = parse_agent_config(config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    // Validate configuration
    validate_config(&config)
        .with_context(|| format!("Configuration validation failed for {}", config_path.display()))?;

    println!(
        "{}",
        style(format!("✓ Loaded agent: {}", config.name)).green()
    );
    println!();

    // Get API key from environment
    let api_key = match config.llm.provider.as_str() {
        "anthropic" => env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY environment variable not set")?,
        "openai" => env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable not set")?,
        provider => anyhow::bail!("Unsupported LLM provider: {}", provider),
    };

    // Create LLM adapter
    let adapter: Arc<dyn LLMAdapter> = match config.llm.provider.as_str() {
        "anthropic" => Arc::new(AnthropicAdapter::new(api_key)),
        _ => anyhow::bail!("Unsupported provider: {}", config.llm.provider),
    };

    println!(
        "{}",
        style(format!(
            "Using {} ({})",
            config.llm.provider, config.llm.model
        ))
        .dim()
    );

    // Build tools from configuration
    let tool_factory = ToolFactory::new();
    let tools = tool_factory
        .build_tools(&config)
        .context("Failed to build tools from configuration")?;

    // Print available tools
    let mut tool_names: Vec<_> = tools.keys().map(|k| k.as_str()).collect();
    tool_names.sort();
    println!(
        "{}",
        style(format!("Available tools: {}", tool_names.join(", "))).dim()
    );
    println!();

    // Build agent executor with ReAct strategy
    let executor = AgentExecutorBuilder::new()
        .config(config)
        .llm(adapter)
        .tools(tools)
        .strategy(Box::new(ReActStrategy::new()))
        .build()
        .context("Failed to build agent executor")?;

    println!("{}", style("Agent is thinking...").cyan().dim());
    println!();

    // Execute
    let result = executor
        .execute(input)
        .await
        .context("Agent execution failed")?;

    // Display intermediate thoughts/reasoning
    if !result.thoughts.is_empty() {
        println!("{}", style("═".repeat(60)).dim());
        println!("{}", style("Agent Reasoning:").cyan().bold());
        println!();
        for (idx, thought) in result.thoughts.iter().enumerate() {
            println!("{}", style(format!("Step {}:", idx + 1)).yellow().bold());
            println!("{}", thought);
            println!();
        }
        println!("{}", style("═".repeat(60)).dim());
        println!();
    }

    // Display result
    println!("{}", style("Final Answer:").cyan().bold());
    println!("{}", result.response);
    println!();

    // Display execution stats
    println!("{}", style("─".repeat(60)).dim());

    if result.success {
        println!("{}", style("✓ Execution completed successfully").green());
    } else {
        println!("{}", style("✗ Execution failed").red());
    }

    println!(
        "{} {}",
        style("Iterations:").dim(),
        style(result.iterations).yellow()
    );

    println!(
        "{} {} tokens",
        style("Tokens:").dim(),
        style(result.total_tokens).yellow()
    );

    if result.total_cost > 0.0 {
        println!(
            "{} {}",
            style("Cost:").dim(),
            style(format!("${:.4}", result.total_cost)).yellow()
        );
    }

    if result.execution_time_ms > 0 {
        let time_str = if result.execution_time_ms < 1000 {
            format!("{}ms", result.execution_time_ms)
        } else {
            format!("{:.2}s", result.execution_time_ms as f64 / 1000.0)
        };
        println!(
            "{} {}",
            style("Time:").dim(),
            style(time_str).yellow()
        );
    }

    println!(
        "{} {:?}",
        style("Stop reason:").dim(),
        result.stop_reason
    );

    // Display tool calls if any
    if !result.tool_calls.is_empty() {
        println!();
        println!("{}", style(format!("Tool calls ({})", result.tool_calls.len())).dim());
        for (idx, call) in result.tool_calls.iter().enumerate() {
            let status = if call.success { "✓" } else { "✗" };
            println!(
                "  {}. {} {} ({}ms)",
                idx + 1,
                status,
                style(&call.tool_name).cyan(),
                call.execution_time_ms
            );
        }
    }

    println!("{}", style("─".repeat(60)).dim());

    Ok(())
}
