//! Run statistics command

use anyhow::Result;
use chrono::{Duration, Utc};
use console::style;
use namra_storage::{RunFilter, SqliteStorage};

pub fn execute(agent: Option<&str>, range: &str) -> Result<()> {
    let storage = SqliteStorage::open_default()?;

    let since = parse_duration(range)?;

    let mut filter = RunFilter {
        since: Some(since),
        ..Default::default()
    };

    if let Some(agent_name) = agent {
        filter.agent_name = Some(agent_name.to_string());
    }

    let stats = storage.get_stats(&filter)?;

    println!();
    println!(
        "{}",
        style(format!("Run Statistics (last {})", range))
            .cyan()
            .bold()
    );
    if let Some(ref agent_name) = filter.agent_name {
        println!("{}", style(format!("Agent: {}", agent_name)).dim());
    }
    println!("{}", style("─".repeat(40)).dim());

    println!(
        "{:<20} {}",
        style("Total Runs:").dim(),
        style(stats.total_runs.to_string()).yellow()
    );

    let success_rate = if stats.total_runs > 0 {
        (stats.successful_runs as f64 / stats.total_runs as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "{:<20} {} {}",
        style("Successful:").dim(),
        style(stats.successful_runs.to_string()).green(),
        style(format!("({:.1}%)", success_rate)).dim()
    );

    println!(
        "{:<20} {}",
        style("Failed:").dim(),
        if stats.failed_runs > 0 {
            style(stats.failed_runs.to_string()).red()
        } else {
            style(stats.failed_runs.to_string()).dim()
        }
    );

    println!();

    println!(
        "{:<20} {}",
        style("Total Tokens:").dim(),
        style(format_number(stats.total_tokens)).yellow()
    );

    println!(
        "{:<20} {}",
        style("Total Cost:").dim(),
        style(format!("${:.4}", stats.total_cost)).yellow()
    );

    let avg_time = if stats.avg_execution_time_ms < 1000.0 {
        format!("{:.0}ms", stats.avg_execution_time_ms)
    } else {
        format!("{:.2}s", stats.avg_execution_time_ms / 1000.0)
    };
    println!(
        "{:<20} {}",
        style("Avg Duration:").dim(),
        style(avg_time).yellow()
    );

    println!();

    Ok(())
}

/// Parse duration string like "7d", "24h" into a DateTime
fn parse_duration(s: &str) -> Result<chrono::DateTime<Utc>> {
    let s = s.trim().to_lowercase();

    let (num_str, unit) = if s.ends_with('d') {
        (&s[..s.len() - 1], "d")
    } else if s.ends_with('h') {
        (&s[..s.len() - 1], "h")
    } else {
        anyhow::bail!("Invalid duration format. Use formats like: 7d, 30d, 24h");
    };

    let num: i64 = num_str.parse()?;

    let duration = match unit {
        "d" => Duration::days(num),
        "h" => Duration::hours(num),
        _ => anyhow::bail!("Invalid duration unit"),
    };

    Ok(Utc::now() - duration)
}

/// Format large numbers with commas
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(*c);
    }

    result
}
