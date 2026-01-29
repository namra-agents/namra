//! Filesystem tool for file operations across multiple backends

pub mod backend;
pub mod local;
pub mod remote;

use async_trait::async_trait;
use serde_json::{json, Value};

use self::backend::FileSystemBackend;
use crate::error::{Result, ToolError};
use crate::tool::{Tool, ToolOutput, ToolTimer};

/// Filesystem tool that supports multiple storage backends
pub struct FileSystemTool {
    name: String,
    backend: Box<dyn FileSystemBackend>,
}

impl FileSystemTool {
    /// Create a new filesystem tool with a specific backend
    pub fn new(name: String, backend: Box<dyn FileSystemBackend>) -> Self {
        Self { name, backend }
    }

    /// Create a filesystem tool with local backend (no sandboxing)
    pub fn new_local() -> Self {
        Self {
            name: "filesystem".to_string(),
            backend: Box::new(local::LocalBackend::new()),
        }
    }

    /// Create a filesystem tool with local backend and sandboxing
    pub fn new_local_sandboxed(base_dir: std::path::PathBuf, read_only: bool) -> Self {
        Self {
            name: "filesystem".to_string(),
            backend: Box::new(local::LocalBackend::with_sandbox(base_dir, read_only)),
        }
    }
}

#[async_trait]
impl Tool for FileSystemTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Perform filesystem operations (read, write, list, delete)"
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["read", "write", "list", "delete"],
                    "description": "Operation to perform"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory path"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write (for write operation)"
                }
            },
            "required": ["operation", "path"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput> {
        let timer = ToolTimer::start();

        let operation = input["operation"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("Missing operation".to_string()))?;

        let path = input["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("Missing path".to_string()))?;

        let result = match operation {
            "read" => {
                let content = self.backend.read(path).await?;
                let metadata = json!({
                    "operation": "read",
                    "path": path,
                    "size": content.len(),
                    "backend": self.backend.backend_type(),
                });
                Ok(ToolOutput::success_with_metadata(
                    content,
                    metadata,
                    timer.elapsed_ms(),
                ))
            }

            "write" => {
                let content = input["content"]
                    .as_str()
                    .ok_or_else(|| ToolError::InvalidInput("Missing content for write".to_string()))?;

                self.backend.write(path, content).await?;

                let metadata = json!({
                    "operation": "write",
                    "path": path,
                    "size": content.len(),
                    "backend": self.backend.backend_type(),
                });
                Ok(ToolOutput::success_with_metadata(
                    format!("Successfully wrote {} bytes to {}", content.len(), path),
                    metadata,
                    timer.elapsed_ms(),
                ))
            }

            "list" => {
                let entries = self.backend.list(path).await?;
                let entry_names: Vec<String> = entries.iter().map(|e| {
                    if e.is_directory {
                        format!("{}/", e.name)
                    } else {
                        e.name.clone()
                    }
                }).collect();

                let content = entry_names.join("\n");
                let metadata = json!({
                    "operation": "list",
                    "path": path,
                    "count": entries.len(),
                    "entries": entries,
                    "backend": self.backend.backend_type(),
                });

                Ok(ToolOutput::success_with_metadata(
                    content,
                    metadata,
                    timer.elapsed_ms(),
                ))
            }

            "delete" => {
                self.backend.delete(path).await?;

                let metadata = json!({
                    "operation": "delete",
                    "path": path,
                    "backend": self.backend.backend_type(),
                });
                Ok(ToolOutput::success_with_metadata(
                    format!("Successfully deleted {}", path),
                    metadata,
                    timer.elapsed_ms(),
                ))
            }

            _ => Err(ToolError::InvalidInput(format!(
                "Unknown operation: {}. Use read, write, list, or delete.",
                operation
            ))),
        };

        result
    }
}

// Re-export key types
pub use backend::{FileEntry, FileMetadata};
pub use local::LocalBackend;
pub use remote::{AzureBackend, AzureConfig, GCSBackend, GCSConfig, S3Backend, S3Config, SFTPBackend, SFTPConfig};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_filesystem_tool_read_write() {
        let dir = tempdir().unwrap();
        let tool = FileSystemTool::new_local_sandboxed(dir.path().to_path_buf(), false);

        // Write
        let write_input = json!({
            "operation": "write",
            "path": "test.txt",
            "content": "hello world"
        });
        let result = tool.execute(write_input).await.unwrap();
        assert!(result.success);

        // Read
        let read_input = json!({
            "operation": "read",
            "path": "test.txt"
        });
        let result = tool.execute(read_input).await.unwrap();
        assert_eq!(result.content, "hello world");
    }

    #[tokio::test]
    async fn test_filesystem_tool_list() {
        let dir = tempdir().unwrap();
        let tool = FileSystemTool::new_local_sandboxed(dir.path().to_path_buf(), false);

        // Create files
        tool.execute(json!({"operation": "write", "path": "file1.txt", "content": "a"})).await.unwrap();
        tool.execute(json!({"operation": "write", "path": "file2.txt", "content": "b"})).await.unwrap();

        // List
        let result = tool.execute(json!({"operation": "list", "path": "."})).await.unwrap();
        assert!(result.content.contains("file1.txt"));
        assert!(result.content.contains("file2.txt"));
    }

    #[tokio::test]
    async fn test_filesystem_tool_delete() {
        let dir = tempdir().unwrap();
        let tool = FileSystemTool::new_local_sandboxed(dir.path().to_path_buf(), false);

        // Create and delete
        tool.execute(json!({"operation": "write", "path": "test.txt", "content": "x"})).await.unwrap();
        let result = tool.execute(json!({"operation": "delete", "path": "test.txt"})).await.unwrap();
        assert!(result.success);

        // Verify deleted
        let read_result = tool.execute(json!({"operation": "read", "path": "test.txt"})).await;
        assert!(read_result.is_err());
    }
}
