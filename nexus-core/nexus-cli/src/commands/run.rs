//! Run command implementation

use anyhow::{Context, Result};
use console::style;
use futures::StreamExt;
use nexus_config::parse_agent_config;
use nexus_llm::{AnthropicAdapter, LLMAdapter, LLMRequest, Message};
use std::path::Path;
use std::env;

pub async fn execute(config_path: &Path, input: &str, stream: bool) -> Result<()> {
    println!("{}", style("Loading agent configuration...").cyan());

    // Parse configuration
    let config = parse_agent_config(config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

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
    let adapter: Box<dyn LLMAdapter> = match config.llm.provider.as_str() {
        "anthropic" => Box::new(AnthropicAdapter::new(api_key)),
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
    println!();

    // Build messages
    let mut messages = Vec::new();

    // Add system prompt if present
    if !config.system_prompt.is_empty() {
        messages.push(Message::system(config.system_prompt.clone()));
    }

    // Add user input
    messages.push(Message::user(input));

    // Create request
    let request = LLMRequest::new(&config.llm.model, messages)
        .with_temperature(config.llm.temperature)
        .with_max_tokens(config.llm.max_tokens)
        .with_streaming(stream);

    // Execute
    if stream {
        println!("{}", style("Agent:").cyan().bold());
        execute_streaming(adapter.as_ref(), request).await?;
    } else {
        println!("{}", style("Agent is thinking...").cyan().dim());
        execute_non_streaming(adapter.as_ref(), request).await?;
    }

    Ok(())
}

async fn execute_non_streaming(adapter: &dyn LLMAdapter, request: LLMRequest) -> Result<()> {
    let response = adapter
        .generate(request)
        .await
        .context("Failed to generate response")?;

    println!();
    println!("{}", style("Response:").cyan().bold());
    println!("{}", response.content);
    println!();

    // Display usage stats
    println!("{}", style("─".repeat(60)).dim());
    println!(
        "{} {} tokens (input: {}, output: {})",
        style("Tokens:").dim(),
        style(response.usage.total_tokens).yellow(),
        response.usage.input_tokens,
        response.usage.output_tokens
    );

    if let Some(cost) = response.usage.cost {
        println!("{} {}", style("Cost:").dim(), style(format!("${:.4}", cost)).yellow());
    }

    println!(
        "{} {:?}",
        style("Finish:").dim(),
        response.finish_reason
    );
    println!("{}", style("─".repeat(60)).dim());

    Ok(())
}

async fn execute_streaming(adapter: &dyn LLMAdapter, request: LLMRequest) -> Result<()> {
    let mut stream = adapter
        .stream(request)
        .await
        .context("Failed to start streaming")?;

    let mut total_tokens = 0;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.context("Stream error")?;

        if !chunk.content.is_empty() {
            print!("{}", chunk.content);
            use std::io::Write;
            std::io::stdout().flush()?;
        }

        if chunk.is_final {
            if let Some(usage) = chunk.usage {
                total_tokens = usage.total_tokens;
            }
            break;
        }
    }

    println!();
    println!();
    println!("{}", style("─".repeat(60)).dim());

    if total_tokens > 0 {
        println!(
            "{} {}",
            style("Tokens:").dim(),
            style(total_tokens).yellow()
        );
    }

    println!("{}", style("─".repeat(60)).dim());

    Ok(())
}
