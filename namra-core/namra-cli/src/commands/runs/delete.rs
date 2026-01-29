//! Delete runs command

use anyhow::Result;
use chrono::{Duration, Utc};
use console::style;
use namra_storage::SqliteStorage;

pub fn execute(id: Option<&str>, older_than: Option<&str>, confirm: bool) -> Result<()> {
    let storage = SqliteStorage::open_default()?;

    if let Some(duration_str) = older_than {
        // Delete runs older than specified duration
        if !confirm {
            println!(
                "{}",
                style("Use --confirm to actually delete runs.").yellow()
            );
            return Ok(());
        }

        let cutoff = parse_duration(duration_str)?;
        let count = storage.delete_runs_before(cutoff)?;

        if count > 0 {
            println!(
                "{}",
                style(format!(
                    "Deleted {} run(s) older than {}",
                    count, duration_str
                ))
                .green()
            );
        } else {
            println!("{}", style("No runs found matching criteria.").dim());
        }
    } else if let Some(run_id) = id {
        // Delete specific run
        if !confirm {
            println!(
                "{}",
                style(format!(
                    "This will delete run {}. Use --confirm to proceed.",
                    run_id
                ))
                .yellow()
            );
            return Ok(());
        }

        // Try to find by prefix if not full ID
        let full_id = if run_id.len() < 36 {
            let runs = storage.list_runs(&Default::default())?;
            runs.into_iter()
                .find(|r| r.id.starts_with(run_id))
                .map(|r| r.id)
        } else {
            Some(run_id.to_string())
        };

        if let Some(id) = full_id {
            let deleted = storage.delete_run(&id)?;
            if deleted {
                println!("{}", style(format!("Deleted run {}", &id[..8])).green());
            } else {
                println!("{}", style(format!("Run not found: {}", run_id)).red());
            }
        } else {
            println!("{}", style(format!("Run not found: {}", run_id)).red());
        }
    } else {
        println!(
            "{}",
            style("Specify a run ID or use --older-than <duration>").yellow()
        );
    }

    Ok(())
}

/// Parse duration string like "7d", "30d" into a DateTime
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
