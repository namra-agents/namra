//! Show run details command

use anyhow::Result;
use console::style;
use namra_storage::{RunFilter, SqliteStorage};

pub fn execute(id: &str, verbose: bool) -> Result<()> {
    let storage = SqliteStorage::open_default()?;

    // Try to find run by ID prefix
    let run = if id.len() < 36 {
        // Partial ID - search for matching run
        let runs = storage.list_runs(&RunFilter {
            limit: Some(100),
            ..Default::default()
        })?;

        runs.into_iter().find(|r| r.id.starts_with(id))
    } else {
        storage.get_run(id)?
    };

    let run = match run {
        Some(r) => r,
        None => {
            println!("{}", style(format!("Run not found: {}", id)).red());
            return Ok(());
        }
    };

    // Header
    println!();
    println!("{}", style("Run Details").cyan().bold());
    println!("{}", style("─".repeat(50)).dim());

    // Basic info
    println!("{:<15} {}", style("ID:").dim(), style(&run.id).cyan());
    println!(
        "{:<15} {} {}",
        style("Agent:").dim(),
        run.agent_name,
        run.agent_version
            .as_ref()
            .map(|v| format!("(v{})", v))
            .unwrap_or_default()
    );

    let status = if run.success {
        style("✓ Completed").green()
    } else {
        style("✗ Failed").red()
    };
    println!("{:<15} {}", style("Status:").dim(), status);

    println!();

    // Input/Output
    println!("{}", style("Input:").dim());
    println!("  {}", truncate_string(&run.input_prompt, 200));
    println!();

    println!("{}", style("Response:").dim());
    if let Some(ref response) = run.response {
        println!("  {}", truncate_string(response, 500));
    } else {
        println!("  {}", style("(no response)").dim());
    }
    println!();

    // Metrics
    println!("{}", style("Metrics:").dim());
    println!(
        "  Iterations:    {}",
        style(run.iterations.to_string()).yellow()
    );
    println!(
        "  Tokens:        {}",
        style(run.total_tokens.to_string()).yellow()
    );
    println!(
        "  Cost:          {}",
        style(format!("${:.4}", run.total_cost)).yellow()
    );
    let time = if run.execution_time_ms < 1000 {
        format!("{}ms", run.execution_time_ms)
    } else {
        format!("{:.2}s", run.execution_time_ms as f64 / 1000.0)
    };
    println!("  Duration:      {}", style(time).yellow());
    println!("  Stop Reason:   {:?}", run.stop_reason);

    if let Some(ref provider) = run.llm_provider {
        println!();
        println!("{}", style("LLM:").dim());
        println!("  Provider: {}", provider);
        if let Some(ref model) = run.llm_model {
            println!("  Model:    {}", model);
        }
    }

    // Tool calls
    if !run.tool_calls.is_empty() {
        println!();
        println!(
            "{}",
            style(format!("Tool Calls ({}):", run.tool_calls.len())).dim()
        );
        for tc in &run.tool_calls {
            let status_icon = if tc.success { "✓" } else { "✗" };
            println!(
                "  {}. {} {} ({}ms)",
                tc.sequence_number + 1,
                status_icon,
                style(&tc.tool_name).cyan(),
                tc.execution_time_ms
            );

            if verbose {
                println!(
                    "     Input:  {}",
                    truncate_string(&tc.input.to_string(), 100)
                );
                if let Some(ref output) = tc.output {
                    println!("     Output: {}", truncate_string(output, 100));
                }
            }
        }
    }

    // Thoughts/reasoning
    if !run.thoughts.is_empty() {
        println!();
        println!(
            "{}",
            style(format!("Reasoning Steps ({}):", run.thoughts.len())).dim()
        );
        for thought in &run.thoughts {
            println!(
                "  {}. {}",
                thought.sequence_number + 1,
                truncate_string(&thought.content, if verbose { 500 } else { 100 })
            );
        }
    }

    // Timestamps
    println!();
    println!("{}", style("Timestamps:").dim());
    println!("  Started:   {}", run.started_at.format("%Y-%m-%d %H:%M:%S"));
    println!(
        "  Completed: {}",
        run.completed_at.format("%Y-%m-%d %H:%M:%S")
    );

    println!();

    Ok(())
}

fn truncate_string(s: &str, max_len: usize) -> String {
    // Replace newlines with spaces for display
    let s = s.replace('\n', " ").replace("  ", " ");

    if s.len() <= max_len {
        s
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
