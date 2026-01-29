//! Local filesystem backend implementation

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

use super::backend::{FileEntry, FileMetadata, FileSystemBackend};
use crate::error::{Result, ToolError};

/// Local filesystem backend with optional sandboxing
pub struct LocalBackend {
    base_dir: Option<PathBuf>,
    read_only: bool,
}

impl LocalBackend {
    /// Create a new local backend without sandboxing
    pub fn new() -> Self {
        Self {
            base_dir: None,
            read_only: false,
        }
    }

    /// Create a new local backend with sandboxing
    pub fn with_sandbox(base_dir: PathBuf, read_only: bool) -> Self {
        // Canonicalize to handle symlinks (e.g., /tmp -> /private/tmp on macOS)
        let canonical_base = base_dir.canonicalize().unwrap_or(base_dir);
        Self {
            base_dir: Some(canonical_base),
            read_only,
        }
    }

    /// Validate and resolve path (applies sandboxing if configured)
    fn resolve_path(&self, path: &str) -> Result<PathBuf> {
        let path = Path::new(path);

        if let Some(ref base) = self.base_dir {
            // Sandboxing enabled
            let absolute_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                base.join(path)
            };

            // Find first existing parent for validation
            let mut check_path = absolute_path.clone();
            while !check_path.exists() && check_path.parent().is_some() {
                check_path = check_path.parent().unwrap().to_path_buf();
            }

            // Canonicalize existing part and verify it's within base_dir
            if check_path.exists() {
                let canonical = check_path
                    .canonicalize()
                    .map_err(|e| ToolError::FilesystemError(format!("Invalid path: {}", e)))?;

                if !canonical.starts_with(base) {
                    return Err(ToolError::PermissionDenied(
                        "Path outside allowed directory".to_string(),
                    ));
                }
            }

            Ok(absolute_path)
        } else {
            // No sandboxing
            Ok(path.to_path_buf())
        }
    }
}

impl Default for LocalBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FileSystemBackend for LocalBackend {
    async fn read(&self, path: &str) -> Result<String> {
        let resolved = self.resolve_path(path)?;
        fs::read_to_string(&resolved)
            .await
            .map_err(|e| ToolError::FilesystemError(format!("Read error: {}", e)))
    }

    async fn write(&self, path: &str, content: &str) -> Result<()> {
        if self.read_only {
            return Err(ToolError::PermissionDenied(
                "Write operation not allowed on read-only filesystem".to_string(),
            ));
        }

        let resolved = self.resolve_path(path)?;

        // Create parent directories if needed
        if let Some(parent) = resolved.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                ToolError::FilesystemError(format!("Failed to create directories: {}", e))
            })?;
        }

        fs::write(&resolved, content)
            .await
            .map_err(|e| ToolError::FilesystemError(format!("Write error: {}", e)))
    }

    async fn list(&self, path: &str) -> Result<Vec<FileEntry>> {
        let resolved = self.resolve_path(path)?;

        if !resolved.is_dir() {
            return Err(ToolError::InvalidInput(format!(
                "Path is not a directory: {}",
                path
            )));
        }

        let mut entries = Vec::new();
        let mut dir = fs::read_dir(&resolved)
            .await
            .map_err(|e| ToolError::FilesystemError(format!("List error: {}", e)))?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|e| ToolError::FilesystemError(format!("List error: {}", e)))?
        {
            let metadata = entry.metadata().await.ok();
            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = metadata
                .as_ref()
                .and_then(|m| if m.is_file() { Some(m.len()) } else { None });
            let modified = metadata.and_then(|m| m.modified().ok());

            entries.push(FileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                is_directory: is_dir,
                size,
                modified,
            });
        }

        Ok(entries)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        if self.read_only {
            return Err(ToolError::PermissionDenied(
                "Delete operation not allowed on read-only filesystem".to_string(),
            ));
        }

        let resolved = self.resolve_path(path)?;

        let metadata = fs::metadata(&resolved)
            .await
            .map_err(|e| ToolError::FilesystemError(format!("Delete error: {}", e)))?;

        if metadata.is_dir() {
            fs::remove_dir_all(&resolved)
                .await
                .map_err(|e| ToolError::FilesystemError(format!("Delete error: {}", e)))?;
        } else {
            fs::remove_file(&resolved)
                .await
                .map_err(|e| ToolError::FilesystemError(format!("Delete error: {}", e)))?;
        }

        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let resolved = self.resolve_path(path)?;
        Ok(resolved.exists())
    }

    async fn metadata(&self, path: &str) -> Result<FileMetadata> {
        let resolved = self.resolve_path(path)?;

        let metadata = fs::metadata(&resolved)
            .await
            .map_err(|e| ToolError::FilesystemError(format!("Metadata error: {}", e)))?;

        Ok(FileMetadata {
            size: metadata.len(),
            is_directory: metadata.is_dir(),
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
            accessed: metadata.accessed().ok(),
        })
    }

    fn backend_type(&self) -> &str {
        "local"
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local_read_write() {
        let dir = tempdir().unwrap();
        let backend = LocalBackend::with_sandbox(dir.path().to_path_buf(), false);

        // Write
        backend.write("test.txt", "hello world").await.unwrap();

        // Read
        let content = backend.read("test.txt").await.unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_local_read_only() {
        let dir = tempdir().unwrap();
        let backend = LocalBackend::with_sandbox(dir.path().to_path_buf(), true);

        // Write should fail
        let result = backend.write("test.txt", "content").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_local_sandboxing() {
        let dir = tempdir().unwrap();
        let backend = LocalBackend::with_sandbox(dir.path().to_path_buf(), false);

        // Try to escape sandbox
        let result = backend.read("../../etc/passwd").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_local_list() {
        let dir = tempdir().unwrap();
        let backend = LocalBackend::with_sandbox(dir.path().to_path_buf(), false);

        // Create files
        backend.write("file1.txt", "content1").await.unwrap();
        backend.write("file2.txt", "content2").await.unwrap();

        // List
        let entries = backend.list(".").await.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn test_local_delete() {
        let dir = tempdir().unwrap();
        let backend = LocalBackend::with_sandbox(dir.path().to_path_buf(), false);

        // Create and delete
        backend.write("test.txt", "content").await.unwrap();
        assert!(backend.exists("test.txt").await.unwrap());

        backend.delete("test.txt").await.unwrap();
        assert!(!backend.exists("test.txt").await.unwrap());
    }
}
