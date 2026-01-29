//! Workflow configuration types

use serde::{Deserialize, Serialize};

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub name: String,
    pub version: String,
    pub description: Option<String>,

    /// Workflow nodes (agents or custom logic)
    pub nodes: Vec<WorkflowNode>,

    /// Edges define the flow between nodes
    pub edges: Vec<WorkflowEdge>,

    /// Entry point node ID
    pub entry_point: String,

    /// Exit point node ID
    pub exit_point: String,

    /// Workflow-level configuration
    #[serde(default)]
    pub config: WorkflowExecutionConfig,
}

/// Workflow node (represents an agent or custom logic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,

    #[serde(rename = "type")]
    pub node_type: String,

    #[serde(default)]
    pub agent_config: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub timeout: Option<String>,
}

/// Workflow edge (connection between nodes)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkflowEdge {
    /// Simple edge from one node to another
    Direct { from: String, to: String },

    /// Conditional edge with logic
    Conditional { from: String, to: ConditionalTarget },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalTarget {
    #[serde(rename = "type")]
    pub target_type: String,

    pub logic: String,
}

/// Workflow execution configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowExecutionConfig {
    #[serde(default = "default_workflow_timeout")]
    pub max_execution_time: String,

    #[serde(default)]
    pub checkpoint: Option<CheckpointConfig>,

    #[serde(default)]
    pub retry_failed_nodes: bool,

    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    pub enabled: bool,
    pub interval: String,
}

fn default_workflow_timeout() -> String {
    "300s".to_string()
}

fn default_max_retries() -> u32 {
    2
}
