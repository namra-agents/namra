//! Filesystem tool for file operations

use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::error::{Result, ToolError};
use crate::tool::{Tool, ToolOutput, ToolTimer};

/// Filesystem tool for reading, writing, and listing files
pub struct FileSystemTool {
    /// Optional base directory for sandboxing
    base_dir: Option<PathBuf>,
}

impl FileSystemTool {
    /// Create a new filesystem tool without sandboxing
    pub fn new() -> Self {
        Self { base_dir: None }
    }

    /// Create a new filesystem tool with a base directory for sandboxing
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        // Canonicalize the base_dir to handle symlinks (like /tmp -> /private/tmp on macOS)
        let canonical_base = base_dir.canonicalize().unwrap_or(base_dir);
        Self {
            base_dir: Some(canonical_base),
        }
    }

    /// Validate and resolve path (applies sandboxing if configured)
    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let path = Path::new(path);

        // If base_dir is set, ensure path is within it
        if let Some(ref base) = self.base_dir {
            let absolute_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                base.join(path)
            };

            // For sandboxing, check if the resolved path would be within base_dir
            // We need to handle paths that don't exist yet (for write operations)
            let mut check_path = absolute_path.clone();

            // Find the first existing parent
            while !check_path.exists() && check_path.parent().is_some() {
                check_path = check_path.parent().unwrap().to_path_buf();
            }

            // Canonicalize the existing part
            if check_path.exists() {
                let canonical = check_path
                    .canonicalize()
                    .map_err(|e| ToolError::FilesystemError(format!("Invalid path: {}", e)))?;

                // Ensure it's within base_dir
                if !canonical.starts_with(base) {
                    return Err(ToolError::PermissionDenied(
                        "Path outside allowed directory".to_string(),
                    ));
                }
            }

            Ok(absolute_path)
        } else {
            // No sandboxing, use path as-is
            Ok(path.to_path_buf())
        }
    }

    /// Read file operation
    async fn read_file(&self, path: &str) -> Result<ToolOutput> {
        let timer = ToolTimer::start();
        let resolved_path = self.resolve_path(path)?;

        let content = fs::read_to_string(&resolved_path).await?;

        let metadata = json!({
            "path": resolved_path.to_string_lossy(),
            "size": content.len(),
            "operation": "read"
        });

        Ok(ToolOutput::success_with_metadata(
            content,
            metadata,
            timer.elapsed_ms(),
        ))
    }

    /// Write file operation
    async fn write_file(&self, path: &str, content: &str) -> Result<ToolOutput> {
        let timer = ToolTimer::start();
        let resolved_path = self.resolve_path(path)?;

        // Create parent directories if they don't exist
        if let Some(parent) = resolved_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&resolved_path, content).await?;

        let metadata = json!({
            "path": resolved_path.to_string_lossy(),
            "size": content.len(),
            "operation": "write"
        });

        Ok(ToolOutput::success_with_metadata(
            format!("Successfully wrote {} bytes to file", content.len()),
            metadata,
            timer.elapsed_ms(),
        ))
    }

    /// List directory operation
    async fn list_directory(&self, path: &str) -> Result<ToolOutput> {
        let timer = ToolTimer::start();
        let resolved_path = self.resolve_path(path)?;

        let mut entries = fs::read_dir(&resolved_path).await?;
        let mut files = Vec::new();
        let mut dirs = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                dirs.push(name);
            } else {
                files.push(name);
            }
        }

        let content = format!(
            "Directories ({}):\n{}\n\nFiles ({}):\n{}",
            dirs.len(),
            if dirs.is_empty() {
                "  (none)".to_string()
            } else {
                dirs.iter()
                    .map(|d| format!("  {}/", d))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            files.len(),
            if files.is_empty() {
                "  (none)".to_string()
            } else {
                files
                    .iter()
                    .map(|f| format!("  {}", f))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        );

        let metadata = json!({
            "path": resolved_path.to_string_lossy(),
            "directories": dirs,
            "files": files,
            "operation": "list"
        });

        Ok(ToolOutput::success_with_metadata(
            content,
            metadata,
            timer.elapsed_ms(),
        ))
    }

    /// Delete file operation
    async fn delete_file(&self, path: &str) -> Result<ToolOutput> {
        let timer = ToolTimer::start();
        let resolved_path = self.resolve_path(path)?;

        fs::remove_file(&resolved_path).await?;

        let metadata = json!({
            "path": resolved_path.to_string_lossy(),
            "operation": "delete"
        });

        Ok(ToolOutput::success_with_metadata(
            format!("Successfully deleted file"),
            metadata,
            timer.elapsed_ms(),
        ))
    }
}

impl Default for FileSystemTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileSystemTool {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn description(&self) -> &str {
        "Read, write, list, and delete files. \
         Supports operations: read, write, list, delete."
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
                    "description": "Content to write (required for 'write' operation)"
                }
            },
            "required": ["operation", "path"]
        })
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput> {
        let operation = input["operation"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("Missing 'operation' field".to_string()))?;

        let path = input["path"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput("Missing 'path' field".to_string()))?;

        match operation {
            "read" => self.read_file(path).await,
            "write" => {
                let content = input["content"].as_str().ok_or_else(|| {
                    ToolError::InvalidInput(
                        "Missing 'content' field for write operation".to_string(),
                    )
                })?;
                self.write_file(path, content).await
            }
            "list" => self.list_directory(path).await,
            "delete" => self.delete_file(path).await,
            _ => Err(ToolError::InvalidInput(format!(
                "Unknown operation: {}",
                operation
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_filesystem_tool_name() {
        let tool = FileSystemTool::new();
        assert_eq!(tool.name(), "filesystem");
    }

    #[test]
    fn test_filesystem_tool_description() {
        let tool = FileSystemTool::new();
        assert!(tool.description().contains("Read"));
    }

    #[test]
    fn test_filesystem_tool_parameters() {
        let tool = FileSystemTool::new();
        let params = tool.parameters();
        assert!(params["properties"]["operation"].is_object());
        assert!(params["properties"]["path"].is_object());
        assert!(params["required"].is_array());
    }

    #[tokio::test]
    async fn test_write_and_read_file() {
        let temp_dir = tempdir().unwrap();
        let tool = FileSystemTool::with_base_dir(temp_dir.path().to_path_buf());

        // Write file
        let write_input = json!({
            "operation": "write",
            "path": "test.txt",
            "content": "Hello, World!"
        });

        let write_result = tool.execute(write_input).await;
        if let Err(ref e) = write_result {
            eprintln!("Write error: {:?}", e);
        }
        assert!(write_result.is_ok());

        let write_output = write_result.unwrap();
        assert!(write_output.success);

        // Read file
        let read_input = json!({
            "operation": "read",
            "path": "test.txt"
        });

        let read_result = tool.execute(read_input).await;
        assert!(read_result.is_ok());

        let read_output = read_result.unwrap();
        assert!(read_output.success);
        assert_eq!(read_output.content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_list_directory() {
        let temp_dir = tempdir().unwrap();
        let tool = FileSystemTool::with_base_dir(temp_dir.path().to_path_buf());

        // Create some files
        fs::write(temp_dir.path().join("file1.txt"), "content1")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2")
            .await
            .unwrap();
        fs::create_dir(temp_dir.path().join("subdir"))
            .await
            .unwrap();

        // List directory
        let list_input = json!({
            "operation": "list",
            "path": "."
        });

        let result = tool.execute(list_input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.content.contains("file1.txt"));
        assert!(output.content.contains("file2.txt"));
        assert!(output.content.contains("subdir"));
    }

    #[tokio::test]
    async fn test_delete_file() {
        let temp_dir = tempdir().unwrap();
        let tool = FileSystemTool::with_base_dir(temp_dir.path().to_path_buf());

        // Create a file
        let file_path = temp_dir.path().join("to_delete.txt");
        fs::write(&file_path, "content").await.unwrap();
        assert!(file_path.exists());

        // Delete file
        let delete_input = json!({
            "operation": "delete",
            "path": "to_delete.txt"
        });

        let result = tool.execute(delete_input).await;
        assert!(result.is_ok());
        assert!(!file_path.exists());
    }

    #[tokio::test]
    async fn test_read_nonexistent_file() {
        let temp_dir = tempdir().unwrap();
        let tool = FileSystemTool::with_base_dir(temp_dir.path().to_path_buf());

        let read_input = json!({
            "operation": "read",
            "path": "nonexistent.txt"
        });

        let result = tool.execute(read_input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_operation() {
        let tool = FileSystemTool::new();

        let input = json!({
            "operation": "invalid",
            "path": "/tmp/test.txt"
        });

        let result = tool.execute(input).await;
        assert!(result.is_err());
    }
}
