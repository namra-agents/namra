//! JSON export implementation

use crate::error::{StorageError, StorageResult};
use crate::models::RunRecord;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use super::{ExportOptions, Exporter};

/// JSON exporter
pub struct JsonExporter;

impl Exporter for JsonExporter {
    fn export(
        &self,
        runs: &[RunRecord],
        path: &Path,
        options: &ExportOptions,
    ) -> StorageResult<()> {
        let mut file = File::create(path)?;

        // Create export data, optionally stripping tool calls and thoughts
        let export_data: Vec<serde_json::Value> = runs
            .iter()
            .map(|run| {
                let mut value = serde_json::to_value(run).unwrap_or(serde_json::Value::Null);

                if !options.include_tool_calls {
                    if let serde_json::Value::Object(ref mut map) = value {
                        map.remove("tool_calls");
                    }
                }

                if !options.include_thoughts {
                    if let serde_json::Value::Object(ref mut map) = value {
                        map.remove("thoughts");
                    }
                }

                value
            })
            .collect();

        let json_str = if options.pretty_print {
            serde_json::to_string_pretty(&export_data)
        } else {
            serde_json::to_string(&export_data)
        }
        .map_err(|e| StorageError::Export(e.to_string()))?;

        file.write_all(json_str.as_bytes())?;

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
    fn test_json_export() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("runs.json");

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

        let exporter = JsonExporter;
        let options = ExportOptions {
            pretty_print: true,
            ..Default::default()
        };

        exporter.export(&runs, &path, &options).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("test_agent"));
        assert!(content.contains("Hello"));
    }
}
