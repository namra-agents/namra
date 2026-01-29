//! CSV export implementation

use crate::error::{StorageError, StorageResult};
use crate::models::RunRecord;
use std::fs::File;
use std::path::Path;

use super::{ExportOptions, Exporter};

/// CSV exporter
pub struct CsvExporter;

impl Exporter for CsvExporter {
    fn export(
        &self,
        runs: &[RunRecord],
        path: &Path,
        _options: &ExportOptions,
    ) -> StorageResult<()> {
        let file = File::create(path)?;
        let mut wtr = csv::Writer::from_writer(file);

        // Write header
        wtr.write_record([
            "id",
            "agent_name",
            "agent_version",
            "input_prompt",
            "response",
            "success",
            "stop_reason",
            "error_message",
            "iterations",
            "total_tokens",
            "total_cost",
            "execution_time_ms",
            "llm_provider",
            "llm_model",
            "started_at",
            "completed_at",
            "tool_call_count",
            "thought_count",
        ])
        .map_err(|e| StorageError::Export(e.to_string()))?;

        // Write rows
        for run in runs {
            wtr.write_record([
                &run.id,
                &run.agent_name,
                run.agent_version.as_deref().unwrap_or(""),
                &run.input_prompt,
                run.response.as_deref().unwrap_or(""),
                &run.success.to_string(),
                &run.stop_reason.to_string(),
                run.error_message.as_deref().unwrap_or(""),
                &run.iterations.to_string(),
                &run.total_tokens.to_string(),
                &format!("{:.6}", run.total_cost),
                &run.execution_time_ms.to_string(),
                run.llm_provider.as_deref().unwrap_or(""),
                run.llm_model.as_deref().unwrap_or(""),
                &run.started_at.to_rfc3339(),
                &run.completed_at.to_rfc3339(),
                &run.tool_calls.len().to_string(),
                &run.thoughts.len().to_string(),
            ])
            .map_err(|e| StorageError::Export(e.to_string()))?;
        }

        wtr.flush().map_err(|e| StorageError::Export(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::StopReason;
    use chrono::Utc;
    use tempfile::tempdir;

    #[test]
    fn test_csv_export() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("runs.csv");

        let runs = vec![RunRecord {
            id: "test-1".to_string(),
            agent_name: "test_agent".to_string(),
            agent_version: Some("1.0.0".to_string()),
            input_prompt: "Hello".to_string(),
            response: Some("Hi there!".to_string()),
            success: true,
            stop_reason: StopReason::Completed,
            error_message: None,
            iterations: 1,
            total_tokens: 50,
            total_cost: 0.001,
            execution_time_ms: 100,
            llm_provider: Some("anthropic".to_string()),
            llm_model: Some("claude".to_string()),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            tool_calls: vec![],
            thoughts: vec![],
        }];

        let exporter = CsvExporter;
        exporter
            .export(&runs, &path, &ExportOptions::default())
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("id,agent_name")); // Header
        assert!(content.contains("test_agent"));
    }
}
