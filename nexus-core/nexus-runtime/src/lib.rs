//! Nexus Runtime - Agent execution engine
//!
//! This crate provides the core agent execution runtime for the Nexus framework.
//! It implements various execution strategies (like ReAct) and manages the agent
//! lifecycle including tool execution, conversation management, and result tracking.
//!
//! # Architecture
//!
//! The runtime consists of several key components:
//!
//! - **AgentExecutor**: The main execution engine that orchestrates agent runs
//! - **ExecutionContext**: Tracks state during execution (messages, tokens, costs)
//! - **Strategy**: Pluggable execution strategies (ReAct, Chain-of-Thought, etc.)
//! - **ToolCallRecord**: Records details of each tool invocation
//!
//! # Example
//!
//! ```rust,ignore
//! use nexus_runtime::{AgentExecutorBuilder, ReActStrategy};
//! use nexus_config::AgentConfig;
//! use nexus_llm::adapter::LLMAdapter;
//! use nexus_tools::builtin::CalculatorTool;
//! use std::sync::Arc;
//! use std::collections::HashMap;
//!
//! # async fn example(llm: Arc<dyn LLMAdapter>, config: AgentConfig) -> Result<(), Box<dyn std::error::Error>> {
//! // Register tools
//! let mut tools = HashMap::new();
//! tools.insert("calculator".to_string(), Arc::new(CalculatorTool::new()) as Arc<dyn nexus_tools::Tool>);
//!
//! // Build executor with ReAct strategy
//! let executor = AgentExecutorBuilder::new()
//!     .config(config)
//!     .llm(llm)
//!     .tools(tools)
//!     .strategy(Box::new(ReActStrategy::new()))
//!     .build()?;
//!
//! // Execute agent
//! let result = executor.execute("What is 42 * 137?").await?;
//! println!("Response: {}", result.response);
//! println!("Tokens used: {}", result.total_tokens);
//! println!("Cost: ${:.4}", result.total_cost);
//! # Ok(())
//! # }
//! ```
//!
//! # Execution Strategies
//!
//! The runtime supports pluggable execution strategies through the `Strategy` trait.
//! Currently implemented strategies:
//!
//! - **ReAct**: Reasoning and Acting - alternates between thinking and tool use
//!
//! # Features
//!
//! - Async/await throughout for efficient I/O
//! - Token usage and cost tracking
//! - Execution limits (max iterations, timeouts)
//! - Tool call recording and history
//! - Comprehensive error handling

pub mod context;
pub mod error;
pub mod executor;
pub mod strategy;
pub mod tool_factory;

// Re-export key types for convenience
pub use context::{ExecutionContext, ExecutionResult, StopReason, ToolCallRecord};
pub use error::{Result, RuntimeError};
pub use executor::{AgentExecutor, AgentExecutorBuilder};
pub use strategy::react::ReActStrategy;
pub use strategy::Strategy;
pub use tool_factory::ToolFactory;
