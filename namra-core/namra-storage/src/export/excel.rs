//! Excel export implementation

use crate::error::{StorageError, StorageResult};
use crate::models::RunRecord;
use rust_xlsxwriter::{Format, Workbook};
use std::path::Path;

use super::{ExportOptions, Exporter};

/// Excel exporter (xlsx format)
pub struct ExcelExporter;

impl Exporter for ExcelExporter {
    fn export(
        &self,
        runs: &[RunRecord],
        path: &Path,
        options: &ExportOptions,
    ) -> StorageResult<()> {
        let mut workbook = Workbook::new();

        // Create header format
        let header_format = Format::new().set_bold();

        // Create "Runs" sheet
        let runs_sheet = workbook.add_worksheet();
        runs_sheet
            .set_name("Runs")
            .map_err(|e| StorageError::Export(e.to_string()))?;

        // Write headers
        let headers = [
            "ID",
            "Agent",
            "Version",
            "Input",
            "Response",
            "Success",
            "Stop Reason",
            "Error",
            "Iterations",
            "Tokens",
            "Cost",
            "Time (ms)",
            "Provider",
            "Model",
            "Started",
            "Completed",
        ];

        for (col, header) in headers.iter().enumerate() {
            runs_sheet
                .write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| StorageError::Export(e.to_string()))?;
        }

        // Write data rows
        for (row_idx, run) in runs.iter().enumerate() {
            let row = (row_idx + 1) as u32;

            runs_sheet
                .write_string(row, 0, &run.id)
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 1, &run.agent_name)
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 2, run.agent_version.as_deref().unwrap_or(""))
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 3, &run.input_prompt)
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 4, run.response.as_deref().unwrap_or(""))
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 5, if run.success { "Yes" } else { "No" })
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 6, &run.stop_reason.to_string())
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 7, run.error_message.as_deref().unwrap_or(""))
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_number(row, 8, run.iterations as f64)
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_number(row, 9, run.total_tokens as f64)
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_number(row, 10, run.total_cost)
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_number(row, 11, run.execution_time_ms as f64)
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 12, run.llm_provider.as_deref().unwrap_or(""))
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 13, run.llm_model.as_deref().unwrap_or(""))
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 14, &run.started_at.to_rfc3339())
                .map_err(|e| StorageError::Export(e.to_string()))?;
            runs_sheet
                .write_string(row, 15, &run.completed_at.to_rfc3339())
                .map_err(|e| StorageError::Export(e.to_string()))?;
        }

        // Create "Tool Calls" sheet if requested
        if options.include_tool_calls {
            let tools_sheet = workbook.add_worksheet();
            tools_sheet
                .set_name("Tool Calls")
                .map_err(|e| StorageError::Export(e.to_string()))?;

            // Headers
            let tool_headers = [
                "Run ID",
                "Sequence",
                "Tool Name",
                "Input",
                "Output",
                "Success",
                "Error",
                "Time (ms)",
            ];
            for (col, header) in tool_headers.iter().enumerate() {
                tools_sheet
                    .write_string_with_format(0, col as u16, *header, &header_format)
                    .map_err(|e| StorageError::Export(e.to_string()))?;
            }

            // Data
            let mut row = 1u32;
            for run in runs {
                for tc in &run.tool_calls {
                    tools_sheet
                        .write_string(row, 0, &run.id)
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    tools_sheet
                        .write_number(row, 1, tc.sequence_number as f64)
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    tools_sheet
                        .write_string(row, 2, &tc.tool_name)
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    tools_sheet
                        .write_string(row, 3, &tc.input.to_string())
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    tools_sheet
                        .write_string(row, 4, tc.output.as_deref().unwrap_or(""))
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    tools_sheet
                        .write_string(row, 5, if tc.success { "Yes" } else { "No" })
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    tools_sheet
                        .write_string(row, 6, tc.error_message.as_deref().unwrap_or(""))
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    tools_sheet
                        .write_number(row, 7, tc.execution_time_ms as f64)
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    row += 1;
                }
            }
        }

        // Create "Thoughts" sheet if requested
        if options.include_thoughts {
            let thoughts_sheet = workbook.add_worksheet();
            thoughts_sheet
                .set_name("Thoughts")
                .map_err(|e| StorageError::Export(e.to_string()))?;

            // Headers
            let thought_headers = ["Run ID", "Sequence", "Content"];
            for (col, header) in thought_headers.iter().enumerate() {
                thoughts_sheet
                    .write_string_with_format(0, col as u16, *header, &header_format)
                    .map_err(|e| StorageError::Export(e.to_string()))?;
            }

            // Data
            let mut row = 1u32;
            for run in runs {
                for thought in &run.thoughts {
                    thoughts_sheet
                        .write_string(row, 0, &run.id)
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    thoughts_sheet
                        .write_number(row, 1, thought.sequence_number as f64)
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    thoughts_sheet
                        .write_string(row, 2, &thought.content)
                        .map_err(|e| StorageError::Export(e.to_string()))?;
                    row += 1;
                }
            }
        }

        // Save workbook
        workbook
            .save(path)
            .map_err(|e| StorageError::Export(e.to_string()))?;

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
    fn test_excel_export() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("runs.xlsx");

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

        let exporter = ExcelExporter;
        exporter
            .export(&runs, &path, &ExportOptions::default())
            .unwrap();

        // Verify file exists and has content
        assert!(path.exists());
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0);
    }
}
