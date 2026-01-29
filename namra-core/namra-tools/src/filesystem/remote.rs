//! Remote filesystem backend stubs (S3, GCS, Azure, SFTP)
//!
//! These are placeholder implementations that return NotImplemented errors.
//! Full implementations will be added in Week 12+.

use async_trait::async_trait;

use super::backend::{FileEntry, FileMetadata, FileSystemBackend};
use crate::error::{Result, ToolError};

/// S3 backend configuration
#[derive(Debug, Clone)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub prefix: Option<String>,
}

/// S3 filesystem backend (stub - not yet implemented)
pub struct S3Backend {
    _config: S3Config,
    read_only: bool,
}

impl S3Backend {
    pub fn new(config: S3Config, read_only: bool) -> Self {
        Self {
            _config: config,
            read_only,
        }
    }
}

#[async_trait]
impl FileSystemBackend for S3Backend {
    async fn read(&self, _path: &str) -> Result<String> {
        Err(ToolError::NotImplemented(
            "S3 backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn write(&self, _path: &str, _content: &str) -> Result<()> {
        Err(ToolError::NotImplemented(
            "S3 backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn list(&self, _path: &str) -> Result<Vec<FileEntry>> {
        Err(ToolError::NotImplemented(
            "S3 backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn delete(&self, _path: &str) -> Result<()> {
        Err(ToolError::NotImplemented(
            "S3 backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn exists(&self, _path: &str) -> Result<bool> {
        Err(ToolError::NotImplemented(
            "S3 backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn metadata(&self, _path: &str) -> Result<FileMetadata> {
        Err(ToolError::NotImplemented(
            "S3 backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    fn backend_type(&self) -> &str {
        "s3"
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }
}

/// GCS backend configuration
#[derive(Debug, Clone)]
pub struct GCSConfig {
    pub bucket: String,
    pub project: Option<String>,
    pub prefix: Option<String>,
}

/// GCS filesystem backend (stub - not yet implemented)
pub struct GCSBackend {
    _config: GCSConfig,
    read_only: bool,
}

impl GCSBackend {
    pub fn new(config: GCSConfig, read_only: bool) -> Self {
        Self {
            _config: config,
            read_only,
        }
    }
}

#[async_trait]
impl FileSystemBackend for GCSBackend {
    async fn read(&self, _path: &str) -> Result<String> {
        Err(ToolError::NotImplemented(
            "GCS backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn write(&self, _path: &str, _content: &str) -> Result<()> {
        Err(ToolError::NotImplemented(
            "GCS backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn list(&self, _path: &str) -> Result<Vec<FileEntry>> {
        Err(ToolError::NotImplemented(
            "GCS backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn delete(&self, _path: &str) -> Result<()> {
        Err(ToolError::NotImplemented(
            "GCS backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn exists(&self, _path: &str) -> Result<bool> {
        Err(ToolError::NotImplemented(
            "GCS backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn metadata(&self, _path: &str) -> Result<FileMetadata> {
        Err(ToolError::NotImplemented(
            "GCS backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    fn backend_type(&self) -> &str {
        "gcs"
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }
}

/// Azure blob storage backend configuration
#[derive(Debug, Clone)]
pub struct AzureConfig {
    pub container: String,
    pub account: String,
    pub prefix: Option<String>,
}

/// Azure blob storage backend (stub - not yet implemented)
pub struct AzureBackend {
    _config: AzureConfig,
    read_only: bool,
}

impl AzureBackend {
    pub fn new(config: AzureConfig, read_only: bool) -> Self {
        Self {
            _config: config,
            read_only,
        }
    }
}

#[async_trait]
impl FileSystemBackend for AzureBackend {
    async fn read(&self, _path: &str) -> Result<String> {
        Err(ToolError::NotImplemented(
            "Azure backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn write(&self, _path: &str, _content: &str) -> Result<()> {
        Err(ToolError::NotImplemented(
            "Azure backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn list(&self, _path: &str) -> Result<Vec<FileEntry>> {
        Err(ToolError::NotImplemented(
            "Azure backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn delete(&self, _path: &str) -> Result<()> {
        Err(ToolError::NotImplemented(
            "Azure backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn exists(&self, _path: &str) -> Result<bool> {
        Err(ToolError::NotImplemented(
            "Azure backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn metadata(&self, _path: &str) -> Result<FileMetadata> {
        Err(ToolError::NotImplemented(
            "Azure backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    fn backend_type(&self) -> &str {
        "azure"
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }
}

/// SFTP backend configuration
#[derive(Debug, Clone)]
pub struct SFTPConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub base_path: Option<String>,
}

/// SFTP filesystem backend (stub - not yet implemented)
pub struct SFTPBackend {
    _config: SFTPConfig,
    read_only: bool,
}

impl SFTPBackend {
    pub fn new(config: SFTPConfig, read_only: bool) -> Self {
        Self {
            _config: config,
            read_only,
        }
    }
}

#[async_trait]
impl FileSystemBackend for SFTPBackend {
    async fn read(&self, _path: &str) -> Result<String> {
        Err(ToolError::NotImplemented(
            "SFTP backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn write(&self, _path: &str, _content: &str) -> Result<()> {
        Err(ToolError::NotImplemented(
            "SFTP backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn list(&self, _path: &str) -> Result<Vec<FileEntry>> {
        Err(ToolError::NotImplemented(
            "SFTP backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn delete(&self, _path: &str) -> Result<()> {
        Err(ToolError::NotImplemented(
            "SFTP backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn exists(&self, _path: &str) -> Result<bool> {
        Err(ToolError::NotImplemented(
            "SFTP backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    async fn metadata(&self, _path: &str) -> Result<FileMetadata> {
        Err(ToolError::NotImplemented(
            "SFTP backend not yet implemented (Week 12+)".to_string(),
        ))
    }

    fn backend_type(&self) -> &str {
        "sftp"
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }
}
