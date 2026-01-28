//! Nexus Tools - Built-in tool system for agents
//!
//! This crate provides a flexible tool system that allows agents to perform
//! various operations like HTTP requests, file operations, and calculations.
//!
//! # Architecture
//!
//! The tool system is built around the `Tool` trait, which defines a common
//! interface for all tools:
//!
//! - `name()` - Unique identifier for the tool
//! - `description()` - Human-readable description
//! - `parameters()` - JSON Schema defining expected inputs
//! - `execute()` - Async execution method
//!
//! # Built-in Tools
//!
//! ## HTTP Tool
//! Make HTTP requests (GET, POST, PUT, DELETE, PATCH)
//!
//! ```rust,no_run
//! use nexus_tools::{HttpTool, Tool};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let tool = HttpTool::new();
//! let result = tool.execute(json!({
//!     "method": "GET",
//!     "url": "https://api.example.com/data"
//! })).await?;
//! println!("Response: {}", result.content);
//! # Ok(())
//! # }
//! ```
//!
//! ## Filesystem Tool
//! Read, write, list, and delete files
//!
//! ```rust,no_run
//! use nexus_tools::{FileSystemTool, Tool};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let tool = FileSystemTool::new();
//! let result = tool.execute(json!({
//!     "operation": "read",
//!     "path": "/tmp/test.txt"
//! })).await?;
//! println!("File contents: {}", result.content);
//! # Ok(())
//! # }
//! ```
//!
//! ## Calculator Tool
//! Perform arithmetic calculations
//!
//! ```rust,no_run
//! use nexus_tools::{CalculatorTool, Tool};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let tool = CalculatorTool::new();
//! let result = tool.execute(json!({
//!     "expression": "25 * 4"
//! })).await?;
//! println!("Result: {}", result.content); // "25 * 4 = 100"
//! # Ok(())
//! # }
//! ```
//!
//! ## String Tool
//! String manipulation operations
//!
//! ```rust,no_run
//! use nexus_tools::{StringTool, Tool};
//! use serde_json::json;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let tool = StringTool::new();
//! let result = tool.execute(json!({
//!     "operation": "uppercase",
//!     "text": "hello world"
//! })).await?;
//! println!("Result: {}", result.content); // "HELLO WORLD"
//! # Ok(())
//! # }
//! ```

pub mod builtin;
pub mod error;
pub mod filesystem;
pub mod http;
pub mod tool;

// Re-export commonly used types
pub use builtin::{CalculatorTool, StringTool};
pub use error::{Result, ToolError};
pub use filesystem::FileSystemTool;
pub use http::HttpTool;
pub use tool::{Tool, ToolOutput, ToolTimer};
