//! Core error types for Parseltongue.
//!
//! Following steering docs principle: "Structured Error Handling"
//! with thiserror for libraries and comprehensive error coverage.

use thiserror::Error;

/// Core error type for all Parseltongue operations.
///
/// This error type provides structured, actionable error information
/// for debugging and error recovery strategies.
#[derive(Debug, Error)]
pub enum ParseltongError {
    /// Database-related errors
    #[error("Database operation '{operation}' failed: {details}")]
    DatabaseError {
        operation: String,
        details: String,
    },

    /// Entity not found error
    #[error("Entity not found: {isgl1_key}")]
    EntityNotFound {
        isgl1_key: String,
    },

    /// File system operation errors
    #[error("File system error: {path} - {source}")]
    FileSystemError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Parsing and syntax errors
    #[error("Parsing failed: {reason} at {location}")]
    ParseError {
        reason: String,
        location: String,
    },

    /// Temporal versioning errors
    #[error("Temporal versioning error: {details}")]
    TemporalError {
        details: String,
    },

    /// ISGL1 key format errors
    #[error("Invalid ISGL1 key format: {key} - {reason}")]
    InvalidIsgl1Key {
        key: String,
        reason: String,
    },

    /// LLM communication errors
    #[error("LLM communication failed: {reason}")]
    LlmError {
        reason: String,
    },

    /// LSP integration errors
    #[error("LSP integration error: {details}")]
    LspError {
        details: String,
    },

    /// Validation errors
    #[error("Validation failed: {field} - {expected}, got {actual}")]
    ValidationError {
        field: String,
        expected: String,
        actual: String,
    },

    /// Performance constraint violations
    #[error("Performance constraint violated: {constraint} - {details}")]
    PerformanceViolation {
        constraint: String,
        details: String,
    },

    /// Configuration errors
    #[error("Configuration error: {details}")]
    ConfigurationError {
        details: String,
    },

    /// Serialization/deserialization errors
    #[error("Serialization error: {details}")]
    SerializationError {
        details: String,
    },

    /// Dependency tracking errors
    ///
    /// # Example
    /// ```
    /// use parseltongue_core::error::ParseltongError;
    ///
    /// let error = ParseltongError::DependencyError {
    ///     operation: "insert_edge".to_string(),
    ///     reason: "source entity does not exist".to_string(),
    /// };
    /// assert!(error.to_string().contains("Dependency error"));
    /// ```
    #[error("Dependency error: {operation} - {reason}")]
    DependencyError {
        operation: String,
        reason: String,
    },

    /// Circular dependency detected
    #[error("Circular dependency detected: {path}")]
    CircularDependency {
        path: String,
    },

    /// Duplicate dependency edge
    #[error("Duplicate dependency edge: {from_key} -> {to_key} ({edge_type})")]
    DuplicateEdge {
        from_key: String,
        to_key: String,
        edge_type: String,
    },

    /// Missing dependency target
    #[error("Dependency target not found: {to_key} referenced from {from_key}")]
    MissingDependencyTarget {
        from_key: String,
        to_key: String,
    },

    /// Unsupported language/file extension
    #[error("Unsupported language for file: {path}")]
    UnsupportedLanguage {
        path: String,
    },

    /// Parser initialization or parsing error
    #[error("Parser error: {message}")]
    ParserError {
        message: String,
    },
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, ParseltongError>;

/// Error recovery strategy interface
pub trait ErrorRecovery {
    /// Attempt to recover from the given error
    fn recover(&self, error: &ParseltongError) -> Result<RecoveryAction>;
}

/// Recovery actions that can be taken
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryAction {
    /// Retry the operation with exponential backoff
    RetryWithBackoff(std::time::Duration),
    /// Retry with modified parameters
    RetryWithModifiedParameters,
    /// Fall back to alternative implementation
    UseFallback,
    /// Skip this operation and continue
    SkipOperation,
    /// Abort the entire workflow
    AbortWorkflow,
}

impl Default for RecoveryAction {
    fn default() -> Self {
        Self::RetryWithBackoff(std::time::Duration::from_millis(1000))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_formatting_provides_clear_context() {
        let error = ParseltongError::ParseError {
            reason: "unexpected token".to_string(),
            location: "src/main.rs:42".to_string(),
        };

        let formatted = error.to_string();
        assert!(formatted.contains("Parsing failed"));
        assert!(formatted.contains("unexpected token"));
        assert!(formatted.contains("src/main.rs:42"));
    }

    #[test]
    fn error_chain_preserves_context() {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found"
        );

        let parseltong_error = ParseltongError::FileSystemError {
            path: "test.txt".to_string(),
            source: io_error,
        };

        // The error should contain both the path and the underlying IO error
        let error_string = parseltong_error.to_string();
        assert!(error_string.contains("test.txt"));
        assert!(error_string.contains("file not found"));
    }

    #[test]
    fn recovery_action_default_is_sensible() {
        let default_action = RecoveryAction::default();
        assert!(matches!(default_action, RecoveryAction::RetryWithBackoff(_)));
    }

    // ================== Phase 1.2: Dependency Error Tests ==================

    #[test]
    fn test_dependency_error_formatting() {
        let error = ParseltongError::DependencyError {
            operation: "insert_edge".to_string(),
            reason: "source entity does not exist".to_string(),
        };

        let formatted = error.to_string();
        assert!(formatted.contains("Dependency error"));
        assert!(formatted.contains("insert_edge"));
        assert!(formatted.contains("source entity does not exist"));
    }

    #[test]
    fn test_circular_dependency_error() {
        let error = ParseltongError::CircularDependency {
            path: "A -> B -> C -> A".to_string(),
        };

        let formatted = error.to_string();
        assert!(formatted.contains("Circular dependency detected"));
        assert!(formatted.contains("A -> B -> C -> A"));
    }

    #[test]
    fn test_duplicate_edge_error() {
        let error = ParseltongError::DuplicateEdge {
            from_key: "rust:fn:main:src_main_rs:1-10".to_string(),
            to_key: "rust:fn:helper:src_helper_rs:5-20".to_string(),
            edge_type: "Calls".to_string(),
        };

        let formatted = error.to_string();
        assert!(formatted.contains("Duplicate dependency edge"));
        assert!(formatted.contains("rust:fn:main:src_main_rs:1-10"));
        assert!(formatted.contains("rust:fn:helper:src_helper_rs:5-20"));
        assert!(formatted.contains("Calls"));
    }

    #[test]
    fn test_missing_dependency_target_error() {
        let error = ParseltongError::MissingDependencyTarget {
            from_key: "rust:fn:main:src_main_rs:1-10".to_string(),
            to_key: "rust:fn:nonexistent:src_helper_rs:5-20".to_string(),
        };

        let formatted = error.to_string();
        assert!(formatted.contains("Dependency target not found"));
        assert!(formatted.contains("rust:fn:nonexistent:src_helper_rs:5-20"));
        assert!(formatted.contains("referenced from"));
        assert!(formatted.contains("rust:fn:main:src_main_rs:1-10"));
    }

    #[test]
    fn test_dependency_error_is_error_trait() {
        let error = ParseltongError::DependencyError {
            operation: "query_dependencies".to_string(),
            reason: "invalid hop count".to_string(),
        };

        // Verify it implements std::error::Error
        let _: &dyn std::error::Error = &error;
    }
}