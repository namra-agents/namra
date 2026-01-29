//! SQLite storage implementation

use crate::error::{StorageError, StorageResult};
use crate::models::{RunFilter, RunRecord, RunStats, StopReason, ThoughtEntry, ToolCallEntry};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};

/// SQLite schema for run storage
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS runs (
    id TEXT PRIMARY KEY,
    agent_name TEXT NOT NULL,
    agent_version TEXT,
    input_prompt TEXT NOT NULL,
    response TEXT,
    success INTEGER NOT NULL,
    stop_reason TEXT NOT NULL,
    error_message TEXT,
    iterations INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    total_cost REAL NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    llm_provider TEXT,
    llm_model TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tool_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    sequence_number INTEGER NOT NULL,
    tool_name TEXT NOT NULL,
    input TEXT NOT NULL,
    output TEXT,
    success INTEGER NOT NULL,
    error_message TEXT,
    execution_time_ms INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES runs(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS thoughts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    sequence_number INTEGER NOT NULL,
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES runs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_runs_agent_name ON runs(agent_name);
CREATE INDEX IF NOT EXISTS idx_runs_started_at ON runs(started_at);
CREATE INDEX IF NOT EXISTS idx_runs_success ON runs(success);
CREATE INDEX IF NOT EXISTS idx_tool_calls_run_id ON tool_calls(run_id);
CREATE INDEX IF NOT EXISTS idx_thoughts_run_id ON thoughts(run_id);
"#;

/// SQLite-based storage for agent runs
pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    /// Open storage at the default location (~/.namra/runs.db)
    pub fn open_default() -> StorageResult<Self> {
        let path = Self::default_path()?;
        Self::open(&path)
    }

    /// Get the default storage path
    pub fn default_path() -> StorageResult<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            StorageError::Config("Could not determine home directory".to_string())
        })?;
        let namra_dir = home.join(".namra");
        std::fs::create_dir_all(&namra_dir)?;
        Ok(namra_dir.join("runs.db"))
    }

    /// Open storage at a specific path
    pub fn open(path: &Path) -> StorageResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Initialize schema
        conn.execute_batch(SCHEMA)?;

        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing)
    pub fn open_memory() -> StorageResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Save a run record with its tool calls and thoughts
    pub fn save_run(&self, run: &RunRecord) -> StorageResult<()> {
        let tx = self.conn.unchecked_transaction()?;

        // Insert run
        tx.execute(
            r#"INSERT INTO runs (
                id, agent_name, agent_version, input_prompt, response,
                success, stop_reason, error_message, iterations,
                total_tokens, total_cost, execution_time_ms,
                llm_provider, llm_model, started_at, completed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"#,
            params![
                run.id,
                run.agent_name,
                run.agent_version,
                run.input_prompt,
                run.response,
                run.success as i32,
                run.stop_reason.to_string(),
                run.error_message,
                run.iterations,
                run.total_tokens,
                run.total_cost,
                run.execution_time_ms as i64,
                run.llm_provider,
                run.llm_model,
                run.started_at.to_rfc3339(),
                run.completed_at.to_rfc3339(),
            ],
        )?;

        // Insert tool calls
        for tc in &run.tool_calls {
            tx.execute(
                r#"INSERT INTO tool_calls (
                    run_id, sequence_number, tool_name, input, output,
                    success, error_message, execution_time_ms, timestamp
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
                params![
                    run.id,
                    tc.sequence_number,
                    tc.tool_name,
                    serde_json::to_string(&tc.input)?,
                    tc.output,
                    tc.success as i32,
                    tc.error_message,
                    tc.execution_time_ms as i64,
                    tc.timestamp.to_rfc3339(),
                ],
            )?;
        }

        // Insert thoughts
        for thought in &run.thoughts {
            tx.execute(
                r#"INSERT INTO thoughts (run_id, sequence_number, content, timestamp)
                   VALUES (?1, ?2, ?3, ?4)"#,
                params![
                    run.id,
                    thought.sequence_number,
                    thought.content,
                    thought.timestamp.to_rfc3339(),
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    /// Get a run by ID, including tool calls and thoughts
    pub fn get_run(&self, id: &str) -> StorageResult<Option<RunRecord>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, agent_name, agent_version, input_prompt, response,
                      success, stop_reason, error_message, iterations,
                      total_tokens, total_cost, execution_time_ms,
                      llm_provider, llm_model, started_at, completed_at
               FROM runs WHERE id = ?1"#,
        )?;

        let run = stmt
            .query_row(params![id], |row| {
                Ok(RunRecord {
                    id: row.get(0)?,
                    agent_name: row.get(1)?,
                    agent_version: row.get(2)?,
                    input_prompt: row.get(3)?,
                    response: row.get(4)?,
                    success: row.get::<_, i32>(5)? != 0,
                    stop_reason: row
                        .get::<_, String>(6)?
                        .parse()
                        .unwrap_or(StopReason::Error),
                    error_message: row.get(7)?,
                    iterations: row.get(8)?,
                    total_tokens: row.get(9)?,
                    total_cost: row.get(10)?,
                    execution_time_ms: row.get::<_, i64>(11)? as u64,
                    llm_provider: row.get(12)?,
                    llm_model: row.get(13)?,
                    started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(14)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    completed_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(15)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    tool_calls: vec![],
                    thoughts: vec![],
                })
            })
            .optional()?;

        if let Some(mut run) = run {
            // Load tool calls
            run.tool_calls = self.get_tool_calls(&run.id)?;
            // Load thoughts
            run.thoughts = self.get_thoughts(&run.id)?;
            Ok(Some(run))
        } else {
            Ok(None)
        }
    }

    /// Get tool calls for a run
    fn get_tool_calls(&self, run_id: &str) -> StorageResult<Vec<ToolCallEntry>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, run_id, sequence_number, tool_name, input, output,
                      success, error_message, execution_time_ms, timestamp
               FROM tool_calls WHERE run_id = ?1 ORDER BY sequence_number"#,
        )?;

        let tool_calls = stmt
            .query_map(params![run_id], |row| {
                let input_str: String = row.get(4)?;
                Ok(ToolCallEntry {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    sequence_number: row.get(2)?,
                    tool_name: row.get(3)?,
                    input: serde_json::from_str(&input_str).unwrap_or(serde_json::Value::Null),
                    output: row.get(5)?,
                    success: row.get::<_, i32>(6)? != 0,
                    error_message: row.get(7)?,
                    execution_time_ms: row.get::<_, i64>(8)? as u64,
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tool_calls)
    }

    /// Get thoughts for a run
    fn get_thoughts(&self, run_id: &str) -> StorageResult<Vec<ThoughtEntry>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, run_id, sequence_number, content, timestamp
               FROM thoughts WHERE run_id = ?1 ORDER BY sequence_number"#,
        )?;

        let thoughts = stmt
            .query_map(params![run_id], |row| {
                Ok(ThoughtEntry {
                    id: row.get(0)?,
                    run_id: row.get(1)?,
                    sequence_number: row.get(2)?,
                    content: row.get(3)?,
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(thoughts)
    }

    /// List runs with optional filtering
    pub fn list_runs(&self, filter: &RunFilter) -> StorageResult<Vec<RunRecord>> {
        let mut sql = String::from(
            r#"SELECT id, agent_name, agent_version, input_prompt, response,
                      success, stop_reason, error_message, iterations,
                      total_tokens, total_cost, execution_time_ms,
                      llm_provider, llm_model, started_at, completed_at
               FROM runs WHERE 1=1"#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(ref agent) = filter.agent_name {
            sql.push_str(" AND agent_name = ?");
            params.push(Box::new(agent.clone()));
        }

        if let Some(success) = filter.success {
            sql.push_str(" AND success = ?");
            params.push(Box::new(success as i32));
        }

        if let Some(ref since) = filter.since {
            sql.push_str(" AND started_at >= ?");
            params.push(Box::new(since.to_rfc3339()));
        }

        if let Some(ref until) = filter.until {
            sql.push_str(" AND started_at <= ?");
            params.push(Box::new(until.to_rfc3339()));
        }

        sql.push_str(" ORDER BY started_at DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = self.conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let runs = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(RunRecord {
                    id: row.get(0)?,
                    agent_name: row.get(1)?,
                    agent_version: row.get(2)?,
                    input_prompt: row.get(3)?,
                    response: row.get(4)?,
                    success: row.get::<_, i32>(5)? != 0,
                    stop_reason: row
                        .get::<_, String>(6)?
                        .parse()
                        .unwrap_or(StopReason::Error),
                    error_message: row.get(7)?,
                    iterations: row.get(8)?,
                    total_tokens: row.get(9)?,
                    total_cost: row.get(10)?,
                    execution_time_ms: row.get::<_, i64>(11)? as u64,
                    llm_provider: row.get(12)?,
                    llm_model: row.get(13)?,
                    started_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(14)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    completed_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(15)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    tool_calls: vec![],
                    thoughts: vec![],
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(runs)
    }

    /// Get summary statistics
    pub fn get_stats(&self, filter: &RunFilter) -> StorageResult<RunStats> {
        let mut sql = String::from(
            r#"SELECT
                COUNT(*) as total,
                SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successful,
                SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END) as failed,
                COALESCE(SUM(total_tokens), 0) as tokens,
                COALESCE(SUM(total_cost), 0.0) as cost,
                COALESCE(AVG(execution_time_ms), 0.0) as avg_time
               FROM runs WHERE 1=1"#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(ref agent) = filter.agent_name {
            sql.push_str(" AND agent_name = ?");
            params.push(Box::new(agent.clone()));
        }

        if let Some(ref since) = filter.since {
            sql.push_str(" AND started_at >= ?");
            params.push(Box::new(since.to_rfc3339()));
        }

        if let Some(ref until) = filter.until {
            sql.push_str(" AND started_at <= ?");
            params.push(Box::new(until.to_rfc3339()));
        }

        let mut stmt = self.conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let stats = stmt.query_row(param_refs.as_slice(), |row| {
            Ok(RunStats {
                total_runs: row.get::<_, i64>(0)? as u64,
                successful_runs: row.get::<_, i64>(1)? as u64,
                failed_runs: row.get::<_, i64>(2)? as u64,
                total_tokens: row.get::<_, i64>(3)? as u64,
                total_cost: row.get(4)?,
                avg_execution_time_ms: row.get(5)?,
            })
        })?;

        Ok(stats)
    }

    /// Delete a run and all related data
    pub fn delete_run(&self, id: &str) -> StorageResult<bool> {
        let rows = self
            .conn
            .execute("DELETE FROM runs WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    /// Delete runs older than specified date
    pub fn delete_runs_before(&self, before: DateTime<Utc>) -> StorageResult<u64> {
        let rows = self.conn.execute(
            "DELETE FROM runs WHERE started_at < ?1",
            params![before.to_rfc3339()],
        )?;
        Ok(rows as u64)
    }

    /// Count total runs
    pub fn count_runs(&self) -> StorageResult<u64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM runs", [], |row| row.get(0))?;
        Ok(count as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_run() -> RunRecord {
        RunRecord {
            id: uuid::Uuid::new_v4().to_string(),
            agent_name: "test_agent".to_string(),
            agent_version: Some("1.0.0".to_string()),
            input_prompt: "Test input".to_string(),
            response: Some("Test response".to_string()),
            success: true,
            stop_reason: StopReason::Completed,
            error_message: None,
            iterations: 2,
            total_tokens: 100,
            total_cost: 0.001,
            execution_time_ms: 500,
            llm_provider: Some("anthropic".to_string()),
            llm_model: Some("claude-sonnet-4-5-20250929".to_string()),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            tool_calls: vec![ToolCallEntry {
                id: 0,
                run_id: String::new(),
                sequence_number: 0,
                tool_name: "calculator".to_string(),
                input: serde_json::json!({"op": "add", "a": 1, "b": 2}),
                output: Some("3".to_string()),
                success: true,
                error_message: None,
                execution_time_ms: 10,
                timestamp: Utc::now(),
            }],
            thoughts: vec![ThoughtEntry {
                id: 0,
                run_id: String::new(),
                sequence_number: 0,
                content: "Thinking about the problem...".to_string(),
                timestamp: Utc::now(),
            }],
        }
    }

    #[test]
    fn test_save_and_get_run() {
        let storage = SqliteStorage::open_memory().unwrap();
        let run = create_test_run();

        storage.save_run(&run).unwrap();

        let retrieved = storage.get_run(&run.id).unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.agent_name, "test_agent");
        assert_eq!(retrieved.iterations, 2);
        assert_eq!(retrieved.tool_calls.len(), 1);
        assert_eq!(retrieved.thoughts.len(), 1);
    }

    #[test]
    fn test_list_runs() {
        let storage = SqliteStorage::open_memory().unwrap();

        // Save multiple runs
        for i in 0..5 {
            let mut run = create_test_run();
            run.agent_name = if i % 2 == 0 {
                "agent_a".to_string()
            } else {
                "agent_b".to_string()
            };
            storage.save_run(&run).unwrap();
        }

        // List all
        let all = storage.list_runs(&RunFilter::default()).unwrap();
        assert_eq!(all.len(), 5);

        // Filter by agent
        let filter = RunFilter {
            agent_name: Some("agent_a".to_string()),
            ..Default::default()
        };
        let filtered = storage.list_runs(&filter).unwrap();
        assert_eq!(filtered.len(), 3);

        // Limit
        let filter = RunFilter {
            limit: Some(2),
            ..Default::default()
        };
        let limited = storage.list_runs(&filter).unwrap();
        assert_eq!(limited.len(), 2);
    }

    #[test]
    fn test_get_stats() {
        let storage = SqliteStorage::open_memory().unwrap();

        // Save runs with different success states
        for i in 0..10 {
            let mut run = create_test_run();
            run.success = i < 7; // 7 successful, 3 failed
            run.total_tokens = 100;
            run.total_cost = 0.01;
            storage.save_run(&run).unwrap();
        }

        let stats = storage.get_stats(&RunFilter::default()).unwrap();
        assert_eq!(stats.total_runs, 10);
        assert_eq!(stats.successful_runs, 7);
        assert_eq!(stats.failed_runs, 3);
        assert_eq!(stats.total_tokens, 1000);
        assert!((stats.total_cost - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_delete_run() {
        let storage = SqliteStorage::open_memory().unwrap();
        let run = create_test_run();

        storage.save_run(&run).unwrap();
        assert!(storage.get_run(&run.id).unwrap().is_some());

        let deleted = storage.delete_run(&run.id).unwrap();
        assert!(deleted);

        assert!(storage.get_run(&run.id).unwrap().is_none());
    }
}
