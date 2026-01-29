//! Domain models for run storage

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a complete agent run stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    pub id: String,
    pub agent_name: String,
    pub agent_version: Option<String>,

    pub input_prompt: String,
    pub response: Option<String>,

    pub success: bool,
    pub stop_reason: StopReason,
    pub error_message: Option<String>,

    pub iterations: u32,
    pub total_tokens: u32,
    pub total_cost: f64,
    pub execution_time_ms: u64,

    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,

    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,

    /// Tool calls made during execution
    #[serde(default)]
    pub tool_calls: Vec<ToolCallEntry>,

    /// Reasoning steps/thoughts
    #[serde(default)]
    pub thoughts: Vec<ThoughtEntry>,
}

/// Why the agent execution stopped
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    Completed,
    MaxIterations,
    Timeout,
    Error,
    UserStop,
}

impl std::fmt::Display for StopReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StopReason::Completed => write!(f, "completed"),
            StopReason::MaxIterations => write!(f, "max_iterations"),
            StopReason::Timeout => write!(f, "timeout"),
            StopReason::Error => write!(f, "error"),
            StopReason::UserStop => write!(f, "user_stop"),
        }
    }
}

impl std::str::FromStr for StopReason {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "completed" => Ok(StopReason::Completed),
            "max_iterations" => Ok(StopReason::MaxIterations),
            "timeout" => Ok(StopReason::Timeout),
            "error" => Ok(StopReason::Error),
            "user_stop" => Ok(StopReason::UserStop),
            _ => Err(format!("Unknown stop reason: {}", s)),
        }
    }
}

/// A single tool call made during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEntry {
    pub id: i64,
    pub run_id: String,
    pub sequence_number: u32,
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// A reasoning step/thought during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtEntry {
    pub id: i64,
    pub run_id: String,
    pub sequence_number: u32,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// Query filters for listing runs
#[derive(Debug, Default, Clone)]
pub struct RunFilter {
    pub agent_name: Option<String>,
    pub success: Option<bool>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Summary statistics for runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStats {
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub avg_execution_time_ms: f64,
}

impl Default for RunStats {
    fn default() -> Self {
        Self {
            total_runs: 0,
            successful_runs: 0,
            failed_runs: 0,
            total_tokens: 0,
            total_cost: 0.0,
            avg_execution_time_ms: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_reason_display() {
        assert_eq!(StopReason::Completed.to_string(), "completed");
        assert_eq!(StopReason::MaxIterations.to_string(), "max_iterations");
        assert_eq!(StopReason::Error.to_string(), "error");
    }

    #[test]
    fn test_stop_reason_parse() {
        assert_eq!(
            "completed".parse::<StopReason>().unwrap(),
            StopReason::Completed
        );
        assert_eq!(
            "max_iterations".parse::<StopReason>().unwrap(),
            StopReason::MaxIterations
        );
    }
}
