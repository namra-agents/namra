//! Export runs command

use anyhow::Result;
use console::style;
use namra_storage::{
    CsvExporter, ExcelExporter, ExportOptions, Exporter, JsonExporter, RunFilter, SqliteStorage,
};
use std::path::Path;

pub fn execute(
    output: &Path,
    format: &str,
    agent: Option<&str>,
    include_tools: bool,
    include_thoughts: bool,
) -> Result<()> {
    let storage = SqliteStorage::open_default()?;

    let mut filter = RunFilter::default();
    if let Some(agent_name) = agent {
        filter.agent_name = Some(agent_name.to_string());
    }

    // Get runs (with tool calls and thoughts if requested)
    let mut runs = storage.list_runs(&filter)?;

    // Load full details for each run if including tools/thoughts
    if include_tools || include_thoughts {
        for run in &mut runs {
            if let Some(full_run) = storage.get_run(&run.id)? {
                run.tool_calls = full_run.tool_calls;
                run.thoughts = full_run.thoughts;
            }
        }
    }

    if runs.is_empty() {
        println!("{}", style("No runs to export.").yellow());
        return Ok(());
    }

    let options = ExportOptions {
        include_tool_calls: include_tools,
        include_thoughts,
        pretty_print: true,
    };

    // Export based on format
    match format.to_lowercase().as_str() {
        "json" => {
            let exporter = JsonExporter;
            exporter.export(&runs, output, &options)?;
        }
        "csv" => {
            let exporter = CsvExporter;
            exporter.export(&runs, output, &options)?;
        }
        "excel" | "xlsx" => {
            let exporter = ExcelExporter;
            exporter.export(&runs, output, &options)?;
        }
        _ => {
            anyhow::bail!("Unsupported format: {}. Use: json, csv, or excel", format);
        }
    }

    println!(
        "{}",
        style(format!(
            "Exported {} runs to {}",
            runs.len(),
            output.display()
        ))
        .green()
    );

    if include_tools {
        let tool_count: usize = runs.iter().map(|r| r.tool_calls.len()).sum();
        println!(
            "{}",
            style(format!("Includes: {} tool calls", tool_count)).dim()
        );
    }

    if include_thoughts {
        let thought_count: usize = runs.iter().map(|r| r.thoughts.len()).sum();
        println!(
            "{}",
            style(format!("Includes: {} reasoning steps", thought_count)).dim()
        );
    }

    Ok(())
}
