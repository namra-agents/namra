//! Export functionality for run history

mod csv_export;
mod excel;
mod json_export;

pub use csv_export::CsvExporter;
pub use excel::ExcelExporter;
pub use json_export::JsonExporter;

use crate::error::StorageResult;
use crate::models::RunRecord;
use std::path::Path;

/// Options for exporting runs
#[derive(Debug, Default, Clone)]
pub struct ExportOptions {
    /// Include tool call details
    pub include_tool_calls: bool,
    /// Include thought/reasoning steps
    pub include_thoughts: bool,
    /// Pretty print output (for JSON)
    pub pretty_print: bool,
}

/// Trait for exporting runs to different formats
pub trait Exporter {
    /// Export runs to a file
    fn export(&self, runs: &[RunRecord], path: &Path, options: &ExportOptions)
        -> StorageResult<()>;
}
