//! Filesystem backend abstraction

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::error::Result;

/// Filesystem backend trait for different storage providers
#[async_trait]
pub trait FileSystemBackend: Send + Sync {
    /// Read file contents as string
    async fn read(&self, path: &str) -> Result<String>;

    /// Write string content to file
    async fn write(&self, path: &str, content: &str) -> Result<()>;

    /// List directory contents
    async fn list(&self, path: &str) -> Result<Vec<FileEntry>>;

    /// Delete file or directory
    async fn delete(&self, path: &str) -> Result<()>;

    /// Check if path exists
    async fn exists(&self, path: &str) -> Result<bool>;

    /// Get file metadata
    async fn metadata(&self, path: &str) -> Result<FileMetadata>;

    /// Get backend type name
    fn backend_type(&self) -> &str;

    /// Check if backend is read-only
    fn is_read_only(&self) -> bool;
}

/// File or directory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Entry name (file or directory name)
    pub name: String,

    /// Full path
    pub path: String,

    /// Whether this is a directory
    pub is_directory: bool,

    /// File size in bytes (None for directories)
    pub size: Option<u64>,

    /// Last modified time
    pub modified: Option<SystemTime>,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File size in bytes
    pub size: u64,

    /// Whether this is a directory
    pub is_directory: bool,

    /// Creation time (if available)
    pub created: Option<SystemTime>,

    /// Last modified time
    pub modified: Option<SystemTime>,

    /// Last accessed time (if available)
    pub accessed: Option<SystemTime>,
}
