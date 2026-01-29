//! Agent configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// Complete agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AgentConfig {
    /// Agent name (unique identifier)
    #[validate(length(min = 1, max = 64))]
    pub name: String,

    /// Configuration version (semver)
    pub version: String,

    /// Optional description
    pub description: Option<String>,

    /// Metadata (namespace, team, owner, etc.)
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// LLM configuration
    pub llm: LLMConfig,

    /// Tools available to the agent
    #[serde(default)]
    pub tools: Vec<ToolConfig>,

    /// Memory configuration
    #[serde(default)]
    pub memory: Option<MemoryConfig>,

    /// Middleware configuration
    #[serde(default)]
    pub middleware: Option<MiddlewareConfig>,

    /// Execution configuration
    #[serde(default)]
    pub execution: ExecutionConfig,

    /// System prompt
    pub system_prompt: String,

    /// Multi-tenancy configuration
    #[serde(default)]
    pub tenancy: Option<TenancyConfig>,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Provider (anthropic, openai, google, bedrock, etc.)
    pub provider: String,

    /// Model name
    pub model: String,

    /// Temperature (0.0 - 2.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Max tokens
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Top P (0.0 - 1.0)
    #[serde(default)]
    pub top_p: Option<f32>,

    /// Enable streaming
    #[serde(default = "default_true")]
    pub stream: bool,

    /// Retry configuration
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    #[serde(default = "default_backoff")]
    pub backoff: String,

    #[serde(default = "default_initial_delay")]
    pub initial_delay: String,
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolConfig {
    #[serde(rename = "builtin.http")]
    BuiltinHttp {
        name: String,
        config: HttpToolConfig,
    },

    #[serde(rename = "builtin.filesystem")]
    BuiltinFilesystem {
        name: String,
        config: FileSystemToolConfig,
    },

    #[serde(rename = "builtin.database")]
    BuiltinDatabase {
        name: String,
        config: DatabaseToolConfig,
    },

    #[serde(rename = "builtin.vector_search")]
    BuiltinVectorSearch {
        name: String,
        config: VectorSearchToolConfig,
    },

    #[serde(rename = "plugin.python")]
    PluginPython {
        name: String,
        #[serde(default)]
        module: Option<String>,
        #[serde(default)]
        function: Option<String>,
        #[serde(default)]
        endpoint: Option<String>,
        #[serde(default = "default_timeout")]
        timeout: String,
        #[serde(default)]
        require_approval: bool,
    },

    #[serde(rename = "agent")]
    Agent {
        name: String,
        agent_ref: String,
        #[serde(default)]
        timeout: Option<String>,
        #[serde(default)]
        fallback: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpToolConfig {
    pub url: String,
    pub method: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub auth: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout: String,
    #[serde(default)]
    pub retry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemToolConfig {
    #[serde(rename = "type")]
    pub fs_type: FileSystemType,

    #[serde(default)]
    pub read_only: bool,

    #[serde(flatten)]
    pub backend: FileSystemBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSystemType {
    Local,
    S3,
    GCS,
    Azure,
    SFTP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "backend_type", rename_all = "lowercase")]
pub enum FileSystemBackend {
    Local {
        base_dir: String,
    },
    S3 {
        bucket: String,
        region: String,
        #[serde(default)]
        prefix: Option<String>,
        #[serde(default)]
        credentials: Option<String>,
    },
    GCS {
        bucket: String,
        #[serde(default)]
        project: Option<String>,
        #[serde(default)]
        prefix: Option<String>,
        #[serde(default)]
        credentials: Option<String>,
    },
    Azure {
        container: String,
        account: String,
        #[serde(default)]
        prefix: Option<String>,
        #[serde(default)]
        credentials: Option<String>,
    },
    SFTP {
        host: String,
        #[serde(default = "default_sftp_port")]
        port: u16,
        username: String,
        #[serde(default)]
        base_path: Option<String>,
        #[serde(default)]
        credentials: Option<String>,
    },
}

fn default_sftp_port() -> u16 {
    22
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseToolConfig {
    pub connection: String,
    pub connection_string: String,
    pub query_type: String,
    #[serde(default)]
    pub max_rows: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchToolConfig {
    pub index: String,
    #[serde(default = "default_top_k")]
    pub top_k: u32,
    #[serde(default)]
    pub similarity_threshold: Option<f32>,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(rename = "type")]
    pub memory_type: String,

    #[serde(default)]
    pub connection_string: Option<String>,

    #[serde(default)]
    pub namespace: Option<String>,

    #[serde(default = "default_ttl")]
    pub ttl: String,

    #[serde(default = "default_max_messages")]
    pub max_messages: u32,

    #[serde(default)]
    pub summarization: Option<SummarizationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarizationConfig {
    pub enabled: bool,
    pub trigger_at: u32,
    pub keep_recent: u32,
}

/// Middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MiddlewareConfig {
    #[serde(default)]
    pub observability: Option<ObservabilityConfig>,

    #[serde(default)]
    pub security: Option<SecurityConfig>,

    #[serde(default)]
    pub governance: Option<GovernanceConfig>,

    #[serde(default)]
    pub resilience: Option<ResilienceConfig>,

    #[serde(default)]
    pub caching: Option<CachingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub trace_all_steps: bool,

    #[serde(default)]
    pub export_to: Option<String>,

    #[serde(default)]
    pub endpoint: Option<String>,

    #[serde(default = "default_sample_rate")]
    pub sample_rate: f32,

    #[serde(default)]
    pub metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub rate_limit: Option<String>,

    #[serde(default)]
    pub validate_inputs: bool,

    #[serde(default)]
    pub sanitize_outputs: bool,

    #[serde(default)]
    pub detect_prompt_injection: bool,

    #[serde(default)]
    pub secrets_backend: Option<String>,

    #[serde(default)]
    pub allowed_domains: Vec<String>,

    #[serde(default)]
    pub blocked_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub max_cost_per_request: Option<f64>,

    #[serde(default)]
    pub max_cost_per_hour: Option<f64>,

    #[serde(default)]
    pub require_approval_for: Vec<String>,

    #[serde(default)]
    pub policies: Vec<String>,

    #[serde(default)]
    pub compliance_standards: Vec<String>,

    #[serde(default)]
    pub audit_log: Option<AuditLogConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogConfig {
    pub enabled: bool,
    pub destination: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub circuit_breaker: Option<CircuitBreakerConfig>,

    #[serde(default = "default_timeout")]
    pub timeout: String,

    #[serde(default)]
    pub fallback_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub failure_threshold: u32,
    pub timeout: String,
    pub half_open_timeout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    #[serde(default)]
    pub enabled: bool,

    pub backend: String,

    #[serde(default = "default_cache_ttl")]
    pub ttl: String,

    #[serde(default = "default_cache_strategy")]
    pub strategy: String,

    #[serde(default)]
    pub similarity_threshold: Option<f32>,
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    #[serde(default = "default_strategy")]
    pub strategy: String,

    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,

    #[serde(default = "default_timeout")]
    pub timeout: String,

    #[serde(default)]
    pub parallel_tool_calls: bool,

    #[serde(default)]
    pub stop_sequences: Vec<String>,
}

/// Multi-tenancy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenancyConfig {
    pub tenant_id: String,
    pub isolation_level: String,
    #[serde(default)]
    pub resource_quota: Option<ResourceQuota>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub max_requests_per_hour: u32,
    pub max_cost_per_hour: f64,
}

// Default value functions
fn default_temperature() -> f32 {
    0.7
}
fn default_max_tokens() -> u32 {
    4096
}
fn default_true() -> bool {
    true
}
fn default_max_attempts() -> u32 {
    3
}
fn default_backoff() -> String {
    "exponential".to_string()
}
fn default_initial_delay() -> String {
    "1s".to_string()
}
fn default_timeout() -> String {
    "30s".to_string()
}
fn default_top_k() -> u32 {
    5
}
fn default_ttl() -> String {
    "3600s".to_string()
}
fn default_max_messages() -> u32 {
    50
}
fn default_sample_rate() -> f32 {
    1.0
}
fn default_cache_ttl() -> String {
    "1800s".to_string()
}
fn default_cache_strategy() -> String {
    "exact".to_string()
}
fn default_strategy() -> String {
    "react".to_string()
}
fn default_max_iterations() -> u32 {
    10
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            strategy: default_strategy(),
            max_iterations: default_max_iterations(),
            timeout: default_timeout(),
            parallel_tool_calls: false,
            stop_sequences: vec![],
        }
    }
}
