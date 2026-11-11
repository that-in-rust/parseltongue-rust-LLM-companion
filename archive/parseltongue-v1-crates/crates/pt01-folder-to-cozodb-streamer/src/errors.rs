//! Error types for folder-to-cozoDB-streamer.

use thiserror::Error;
use parseltongue_core::error::ParseltongError;

/// Streaming tool specific errors
#[derive(Debug, Error)]
pub enum StreamerError {
    /// File system operation errors
    #[error("File system error: {path} - {source}")]
    FileSystemError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Tree-sitter parsing errors
    #[error("Tree-sitter parsing failed: {file} - {reason}")]
    ParsingError {
        file: String,
        reason: String,
    },

    /// ISGL1 key generation errors
    #[error("ISGL1 key generation failed: {input} - {reason}")]
    KeyGenerationError {
        input: String,
        reason: String,
    },

    /// Database storage errors
    #[error("Database storage error: {details}")]
    StorageError {
        details: String,
    },

    /// Configuration errors
    #[error("Configuration error: {field} - {reason}")]
    ConfigurationError {
        field: String,
        reason: String,
    },

    /// File too large errors
    #[error("File too large: {path} ({size} bytes > {limit} bytes)")]
    FileTooLarge {
        path: String,
        size: usize,
        limit: usize,
    },

    /// Unsupported file type
    #[error("Unsupported file type: {path}")]
    UnsupportedFileType {
        path: String,
    },
}

impl From<StreamerError> for ParseltongError {
    fn from(err: StreamerError) -> Self {
        match err {
            StreamerError::FileSystemError { path, source } => {
                ParseltongError::FileSystemError { path, source }
            }
            StreamerError::ParsingError { file, reason } => {
                ParseltongError::ParseError { reason, location: file }
            }
            StreamerError::StorageError { details } => {
                ParseltongError::DatabaseError {
                    operation: "storage".to_string(),
                    details,
                }
            }
            StreamerError::ConfigurationError { field, reason } => {
                ParseltongError::ConfigurationError { details: format!("{}: {}", field, reason) }
            }
            _ => ParseltongError::ConfigurationError {
                details: err.to_string(),
            },
        }
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, StreamerError>;