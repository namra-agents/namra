//! Run command implementation

use anyhow::{Context, Result};
use chrono::Utc;
use console::style;
use namra_config::{parse_agent_config, validate_config, AgentConfig};
use namra_llm::{AnthropicAdapter, LLMAdapter};
use namra_middleware::observability::{NamraTracer, ObservabilityConfig};
use namra_runtime::{AgentExecutorBuilder, ExecutionResult, ReActStrategy, StopReason, ToolFactory};
use namra_storage::{
    RunRecord, SqliteStorage, StopReason as StoredStopReason, ThoughtEntry, ToolCallEntry,
};
use std::env;
use std::path::Path;
use std::sync::Arc;

pub async fn execute(config_path: &Path, input: &str, _stream: bool) -> Result<()> {
    println!("{}", style("Loading agent configuration...").cyan());

    // Parse configuration
    let config = parse_agent_config(config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    // Validate configuration
    validate_config(&config).with_context(|| {
        format!(
            "Configuration validation failed for {}",
            config_path.display()
        )
    })?;

    // Initialize observability/tracing if configured
    let _tracer = initialize_observability(&config)?;

    println!(
        "{}",
        style(format!("✓ Loaded agent: {}", config.name)).green()
    );
    println!();

    // Get API key from environment
    let api_key = match config.llm.provider.as_str() {
        "anthropic" => env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY environment variable not set")?,
        "openai" => {
            env::var("OPENAI_API_KEY").context("OPENAI_API_KEY environment variable not set")?
        }
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
        .config(config.clone())
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

    // Save to run history
    if let Err(e) = save_run_history(&config, input, &result) {
        eprintln!(
            "{}",
            style(format!("Warning: Could not save run history: {}", e)).yellow()
        );
    }

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

    // Display run ID for reference
    println!(
        "{} {}",
        style("Run ID:").dim(),
        style(&result.id[..8]).cyan()
    );

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
        println!("{} {}", style("Time:").dim(), style(time_str).yellow());
    }

    println!("{} {:?}", style("Stop reason:").dim(), result.stop_reason);

    // Display tool calls if any
    if !result.tool_calls.is_empty() {
        println!();
        println!(
            "{}",
            style(format!("Tool calls ({})", result.tool_calls.len())).dim()
        );
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

/// Save the execution result to run history
fn save_run_history(config: &AgentConfig, input: &str, result: &ExecutionResult) -> Result<()> {
    let storage = SqliteStorage::open_default()?;

    let now = Utc::now();
    let started_at = now - chrono::Duration::milliseconds(result.execution_time_ms as i64);

    let run_record = RunRecord {
        id: result.id.clone(),
        agent_name: config.name.clone(),
        agent_version: Some(config.version.clone()),
        input_prompt: input.to_string(),
        response: Some(result.response.clone()),
        success: result.success,
        stop_reason: convert_stop_reason(&result.stop_reason),
        error_message: match &result.stop_reason {
            StopReason::Error(e) => Some(e.clone()),
            _ => None,
        },
        iterations: result.iterations,
        total_tokens: result.total_tokens,
        total_cost: result.total_cost,
        execution_time_ms: result.execution_time_ms,
        llm_provider: Some(config.llm.provider.clone()),
        llm_model: Some(config.llm.model.clone()),
        started_at,
        completed_at: now,
        tool_calls: result
            .tool_calls
            .iter()
            .enumerate()
            .map(|(i, tc)| ToolCallEntry {
                id: 0,
                run_id: result.id.clone(),
                sequence_number: i as u32,
                tool_name: tc.tool_name.clone(),
                input: tc.input.clone(),
                output: tc.output.clone(),
                success: tc.success,
                error_message: None,
                execution_time_ms: tc.execution_time_ms,
                timestamp: tc.timestamp.into(),
            })
            .collect(),
        thoughts: result
            .thoughts
            .iter()
            .enumerate()
            .map(|(i, t)| ThoughtEntry {
                id: 0,
                run_id: result.id.clone(),
                sequence_number: i as u32,
                content: t.clone(),
                timestamp: now,
            })
            .collect(),
    };

    storage.save_run(&run_record)?;
    Ok(())
}

/// Convert runtime StopReason to storage StopReason
fn convert_stop_reason(reason: &StopReason) -> StoredStopReason {
    match reason {
        StopReason::Completed => StoredStopReason::Completed,
        StopReason::MaxIterations => StoredStopReason::MaxIterations,
        StopReason::Timeout => StoredStopReason::Timeout,
        StopReason::Error(_) => StoredStopReason::Error,
        StopReason::UserStop => StoredStopReason::UserStop,
    }
}

/// Initialize OpenTelemetry observability if configured
fn initialize_observability(config: &AgentConfig) -> Result<Option<NamraTracer>> {
    // Check if observability is configured in the agent config or via environment
    let otel_config = if let Some(middleware) = &config.middleware {
        if let Some(obs_config) = &middleware.observability {
            // Convert from agent config to middleware config
            ObservabilityConfig {
                enabled: obs_config.enabled,
                trace_all_steps: obs_config.trace_all_steps,
                export_to: obs_config.export_to.clone(),
                endpoint: obs_config.endpoint.clone(),
                sample_rate: obs_config.sample_rate,
                metrics: obs_config.metrics.clone(),
                capture_content: obs_config.capture_content,
                max_content_size: obs_config.max_content_size,
            }
        } else {
            // No config in YAML, check environment variables
            ObservabilityConfig {
                enabled: env::var("NAMRA_OTEL_ENABLED")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(false),
                trace_all_steps: false,
                export_to: env::var("NAMRA_OTEL_EXPORTER").ok(),
                endpoint: env::var("NAMRA_OTEL_ENDPOINT").ok(),
                sample_rate: env::var("NAMRA_OTEL_SAMPLE_RATE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1.0),
                metrics: vec![],
                capture_content: env::var("NAMRA_OTEL_CAPTURE_CONTENT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(false),
                max_content_size: env::var("NAMRA_OTEL_MAX_CONTENT_SIZE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(4000),
            }
        }
    } else {
        // No middleware config, check environment variables
        ObservabilityConfig {
            enabled: env::var("NAMRA_OTEL_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            trace_all_steps: false,
            export_to: env::var("NAMRA_OTEL_EXPORTER").ok(),
            endpoint: env::var("NAMRA_OTEL_ENDPOINT").ok(),
            sample_rate: env::var("NAMRA_OTEL_SAMPLE_RATE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1.0),
            metrics: vec![],
            capture_content: env::var("NAMRA_OTEL_CAPTURE_CONTENT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            max_content_size: env::var("NAMRA_OTEL_MAX_CONTENT_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4000),
        }
    };

    if otel_config.enabled {
        let exporter = otel_config
            .export_to
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("stdout");
        println!(
            "{}",
            style(format!("✓ OpenTelemetry enabled (exporter: {})", exporter)).green()
        );
        let tracer = NamraTracer::init(&otel_config)
            .context("Failed to initialize OpenTelemetry tracer")?;
        Ok(Some(tracer))
    } else {
        Ok(None)
    }
}
