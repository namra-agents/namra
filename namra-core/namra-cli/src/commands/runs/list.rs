//! List runs command

use anyhow::Result;
use chrono::{Duration, Utc};
use console::style;
use namra_storage::{RunFilter, SqliteStorage};

pub fn execute(
    agent: Option<&str>,
    limit: u32,
    since: Option<&str>,
    success_only: bool,
    failed_only: bool,
) -> Result<()> {
    let storage = SqliteStorage::open_default()?;

    let mut filter = RunFilter {
        limit: Some(limit),
        ..Default::default()
    };

    if let Some(agent_name) = agent {
        filter.agent_name = Some(agent_name.to_string());
    }

    if success_only {
        filter.success = Some(true);
    } else if failed_only {
        filter.success = Some(false);
    }

    if let Some(since_str) = since {
        filter.since = Some(parse_duration(since_str)?);
    }

    let runs = storage.list_runs(&filter)?;

    if runs.is_empty() {
        println!("{}", style("No runs found.").dim());
        return Ok(());
    }

    // Print header
    println!(
        "  {:<10} {:<20} {:<8} {:<10} {:<10} {:<10}",
        style("ID").bold(),
        style("AGENT").bold(),
        style("STATUS").bold(),
        style("TOKENS").bold(),
        style("COST").bold(),
        style("TIME").bold(),
    );
    println!("{}", style("─".repeat(75)).dim());

    // Print rows
    for run in &runs {
        let status = if run.success {
            style("✓").green().to_string()
        } else {
            style("✗").red().to_string()
        };

        let cost = format!("${:.4}", run.total_cost);
        let time = if run.execution_time_ms < 1000 {
            format!("{}ms", run.execution_time_ms)
        } else {
            format!("{:.1}s", run.execution_time_ms as f64 / 1000.0)
        };

        // Truncate agent name if too long
        let agent_name = if run.agent_name.len() > 18 {
            format!("{}...", &run.agent_name[..15])
        } else {
            run.agent_name.clone()
        };

        println!(
            "  {:<10} {:<20} {:<8} {:<10} {:<10} {:<10}",
            style(&run.id[..8]).cyan(),
            agent_name,
            status,
            run.total_tokens,
            cost,
            time,
        );
    }

    println!("{}", style("─".repeat(75)).dim());

    let total = storage.count_runs()?;
    if total > runs.len() as u64 {
        println!(
            "{}",
            style(format!(
                "Showing {} of {} runs. Use --limit to show more.",
                runs.len(),
                total
            ))
            .dim()
        );
    }

    Ok(())
}

/// Parse duration string like "1h", "24h", "7d" into a DateTime
fn parse_duration(s: &str) -> Result<chrono::DateTime<Utc>> {
    let s = s.trim().to_lowercase();

    let (num_str, unit) = if s.ends_with('h') {
        (&s[..s.len() - 1], "h")
    } else if s.ends_with('d') {
        (&s[..s.len() - 1], "d")
    } else if s.ends_with('m') {
        (&s[..s.len() - 1], "m")
    } else {
        anyhow::bail!("Invalid duration format. Use formats like: 1h, 24h, 7d, 30m");
    };

    let num: i64 = num_str.parse()?;

    let duration = match unit {
        "h" => Duration::hours(num),
        "d" => Duration::days(num),
        "m" => Duration::minutes(num),
        _ => anyhow::bail!("Invalid duration unit"),
    };

    Ok(Utc::now() - duration)
}
