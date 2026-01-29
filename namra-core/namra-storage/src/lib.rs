//! # namra-storage
//!
//! Storage and persistence for Namra agent runs.
//!
//! This crate provides SQLite-based storage for agent execution history,
//! with support for querying, filtering, and exporting runs.
//!
//! ## Quick Start
//!
//! ```no_run
//! use namra_storage::{SqliteStorage, RunRecord, StopReason};
//! use chrono::Utc;
//!
//! // Open storage at default location (~/.namra/runs.db)
//! let storage = SqliteStorage::open_default().unwrap();
//!
//! // List recent runs
//! let runs = storage.list_runs(&Default::default()).unwrap();
//!
//! // Get a specific run
//! if let Some(run) = storage.get_run("run-id").unwrap() {
//!     println!("Agent: {}, Tokens: {}", run.agent_name, run.total_tokens);
//! }
//! ```

pub mod error;
pub mod export;
pub mod models;
pub mod sqlite;

// Re-exports
pub use error::{StorageError, StorageResult};
pub use export::{CsvExporter, ExcelExporter, ExportOptions, Exporter, JsonExporter};
pub use models::{RunFilter, RunRecord, RunStats, StopReason, ThoughtEntry, ToolCallEntry};
pub use sqlite::SqliteStorage;
