//! Error types for namra-storage

use thiserror::Error;

/// Storage error types
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Migration failed: {0}")]
    Migration(String),

    #[error("Run not found: {0}")]
    NotFound(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("Invalid filter: {0}")]
    InvalidFilter(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias for storage operations
pub type StorageResult<T> = Result<T, StorageError>;
