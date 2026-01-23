//! File watcher type definitions
//!
//! # 4-Word Naming: watcher_types_definition_module
//!
//! This module defines all types used for file watching operations
//! as part of Phase 2.2: WebSocket Streaming Backend.
//!
//! ## Types Defined:
//! - FileEventKindType: Enum for file system event types
//! - RawFileEventDataStruct: Raw event from notify crate
//! - DebouncedFileChangeEventStruct: Aggregated event after debounce
//! - WatcherConfigurationStruct: Watcher configuration options
//! - WatcherHandleContainerStruct: Handle to running watcher
//! - FileWatcherErrorType: Enum of all error codes
//!
//! ## Requirements Implemented:
//! - REQ-FILEWATCHER-001: Types for watcher creation
//! - REQ-FILEWATCHER-006: Default ignore patterns
//! - REQ-FILEWATCHER-011: Debounce configuration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use parseltongue_core::workspace::WorkspaceUniqueIdentifierType;

// =============================================================================
// Configuration Constants
// =============================================================================

/// Default debounce duration in milliseconds
///
/// # 4-Word Name: DEFAULT_DEBOUNCE_DURATION_MS
///
/// Events occurring within this window are aggregated into a single batch.
/// Set to 500ms to balance responsiveness with efficiency.
pub const DEFAULT_DEBOUNCE_DURATION_MS: u64 = 500;

/// Maximum events to buffer before force-flush
///
/// # 4-Word Name: MAX_BUFFERED_EVENTS_COUNT
///
/// When this limit is reached, the buffer is flushed immediately
/// without waiting for the debounce window to close.
pub const MAX_BUFFERED_EVENTS_COUNT: usize = 1000;

/// Minimum allowed debounce duration (ms)
///
/// # 4-Word Name: MIN_DEBOUNCE_DURATION_MS
pub const MIN_DEBOUNCE_DURATION_MS: u64 = 100;

/// Maximum allowed debounce duration (ms)
///
/// # 4-Word Name: MAX_DEBOUNCE_DURATION_MS
pub const MAX_DEBOUNCE_DURATION_MS: u64 = 5000;

/// Default ignore patterns for common build artifacts
///
/// # 4-Word Name: DEFAULT_IGNORE_PATTERNS_LIST
///
/// These patterns filter out directories and files that are not
/// relevant for code analysis (build artifacts, dependencies, etc.)
pub const DEFAULT_IGNORE_PATTERNS_LIST: &[&str] = &[
    "**/target/**",
    "**/node_modules/**",
    "**/.git/**",
    "**/.hg/**",
    "**/.svn/**",
    "**/build/**",
    "**/dist/**",
    "**/__pycache__/**",
    "**/*.pyc",
    "**/vendor/**",
    "**/.idea/**",
    "**/.vscode/**",
    "**/Cargo.lock",
    "**/package-lock.json",
    "**/yarn.lock",
    "**/pnpm-lock.yaml",
    "**/*.swp",
    "**/*.swo",
    "**/*~",
    "**/.DS_Store",
    "**/.env",
    "**/.env.*",
];

// =============================================================================
// Event Types
// =============================================================================

/// File event kind classification
///
/// # 4-Word Name: FileEventKindType
///
/// Represents the type of file system event that occurred.
/// Maps to notify crate event kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileEventKindType {
    /// File was created
    FileCreated,
    /// File was modified (content changed)
    FileModified,
    /// File was deleted
    FileDeleted,
    /// File was renamed or moved
    FileRenamed,
}

impl std::fmt::Display for FileEventKindType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileCreated => write!(f, "created"),
            Self::FileModified => write!(f, "modified"),
            Self::FileDeleted => write!(f, "deleted"),
            Self::FileRenamed => write!(f, "renamed"),
        }
    }
}

/// Raw file event from notify crate
///
/// # 4-Word Name: RawFileEventDataStruct
///
/// Represents a single file system event before debouncing.
/// Contains the event type, affected paths, and timestamp.
#[derive(Debug, Clone)]
pub struct RawFileEventDataStruct {
    /// Type of file system event
    pub event_kind_type_value: FileEventKindType,
    /// Affected file path(s)
    pub affected_paths_list_value: Vec<PathBuf>,
    /// Timestamp when event occurred
    pub event_timestamp_utc_value: DateTime<Utc>,
}

impl RawFileEventDataStruct {
    /// Create new raw event data
    ///
    /// # 4-Word Name: create_new_event_data
    pub fn create_new_event_data(
        kind: FileEventKindType,
        paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            event_kind_type_value: kind,
            affected_paths_list_value: paths,
            event_timestamp_utc_value: Utc::now(),
        }
    }
}

/// Debounced file change event after aggregation
///
/// # 4-Word Name: DebouncedFileChangeEventStruct
///
/// Represents an aggregated event after the debounce window closes.
/// Contains deduplicated file paths and metadata about the batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebouncedFileChangeEventStruct {
    /// Workspace where changes occurred
    pub workspace_identifier_value: WorkspaceUniqueIdentifierType,
    /// List of changed file paths (deduplicated, sorted)
    pub changed_file_paths_list: Vec<PathBuf>,
    /// Timestamp when debounce window closed
    pub debounce_completed_timestamp_utc: DateTime<Utc>,
    /// Count of raw events before deduplication
    pub raw_event_count_value: usize,
}

impl DebouncedFileChangeEventStruct {
    /// Create new debounced event
    ///
    /// # 4-Word Name: create_new_debounced_event
    pub fn create_new_debounced_event(
        workspace_id: WorkspaceUniqueIdentifierType,
        paths: Vec<PathBuf>,
        raw_count: usize,
    ) -> Self {
        Self {
            workspace_identifier_value: workspace_id,
            changed_file_paths_list: paths,
            debounce_completed_timestamp_utc: Utc::now(),
            raw_event_count_value: raw_count,
        }
    }

    /// Check if event is empty (no changes)
    ///
    /// # 4-Word Name: is_empty_change_event
    pub fn is_empty_change_event(&self) -> bool {
        self.changed_file_paths_list.is_empty()
    }
}

// =============================================================================
// Configuration Types
// =============================================================================

/// Watcher configuration options
///
/// # 4-Word Name: WatcherConfigurationStruct
///
/// Contains all configuration options for a file watcher instance.
#[derive(Debug, Clone)]
pub struct WatcherConfigurationStruct {
    /// Debounce duration in milliseconds
    pub debounce_duration_milliseconds_value: u64,
    /// Custom ignore patterns (added to defaults)
    pub custom_ignore_patterns_list: Vec<String>,
    /// Maximum events to buffer before force-flush
    pub max_buffered_events_value: usize,
}

impl Default for WatcherConfigurationStruct {
    fn default() -> Self {
        Self {
            debounce_duration_milliseconds_value: DEFAULT_DEBOUNCE_DURATION_MS,
            custom_ignore_patterns_list: Vec::new(),
            max_buffered_events_value: MAX_BUFFERED_EVENTS_COUNT,
        }
    }
}

impl WatcherConfigurationStruct {
    /// Create configuration with custom debounce duration
    ///
    /// # 4-Word Name: with_debounce_duration_ms
    ///
    /// Duration is clamped to [100ms, 5000ms] range.
    pub fn with_debounce_duration_ms(mut self, duration_ms: u64) -> Self {
        self.debounce_duration_milliseconds_value = duration_ms
            .clamp(MIN_DEBOUNCE_DURATION_MS, MAX_DEBOUNCE_DURATION_MS);
        self
    }

    /// Add custom ignore patterns
    ///
    /// # 4-Word Name: with_custom_ignore_patterns
    pub fn with_custom_ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        self.custom_ignore_patterns_list = patterns;
        self
    }
}

/// Handle container for running watcher
///
/// # 4-Word Name: WatcherHandleContainerStruct
///
/// Contains the watcher handle and associated channels for communication.
/// Dropping this struct stops the watcher.
pub struct WatcherHandleContainerStruct {
    /// Workspace identifier this watcher monitors
    pub workspace_identifier_value: WorkspaceUniqueIdentifierType,
    /// Sender for raw events from watcher thread
    pub event_sender_channel_tx: mpsc::Sender<RawFileEventDataStruct>,
    /// Watcher stop signal sender
    pub stop_signal_sender_tx: mpsc::Sender<()>,
    /// Timestamp when watcher was created
    pub created_timestamp_utc_value: DateTime<Utc>,
}

impl WatcherHandleContainerStruct {
    /// Check if watcher is still running
    ///
    /// # 4-Word Name: is_watcher_still_running
    pub fn is_watcher_still_running(&self) -> bool {
        !self.event_sender_channel_tx.is_closed()
    }
}

/// Shared watcher handle type
///
/// # 4-Word Name: SharedWatcherHandleContainer
pub type SharedWatcherHandleContainer = Arc<RwLock<WatcherHandleContainerStruct>>;

// =============================================================================
// Error Types
// =============================================================================

/// Error types for file watcher operations
///
/// # 4-Word Name: FileWatcherErrorType
///
/// Provides specific error codes for all file watcher failure modes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, thiserror::Error)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileWatcherErrorType {
    /// Failed to create notify watcher
    #[error("Failed to create file watcher: {0}")]
    WatcherCreationFailed(String),

    /// Failed to add path to watch
    #[error("Failed to add path to watch: {0}")]
    WatchPathAddFailed(String),

    /// Failed to remove path from watch
    #[error("Failed to remove path from watch")]
    WatchPathRemoveFailed,

    /// Path does not exist
    #[error("Path does not exist: {0}")]
    PathNotExistsError(String),

    /// Path is not a directory
    #[error("Path is not a directory: {0}")]
    PathNotDirectoryError(String),

    /// Permission denied accessing path
    #[error("Permission denied: {0}")]
    PermissionDeniedError(String),

    /// inotify/FSEvents limit reached
    #[error("System watch limit reached. On Linux: sudo sysctl fs.inotify.max_user_watches=524288")]
    SystemLimitReachedError,

    /// Watcher already exists for workspace
    #[error("Watcher already exists for workspace: {0}")]
    WatcherAlreadyExistsError(String),

    /// Watcher not found for workspace
    #[error("Watcher not found for workspace: {0}")]
    WatcherNotFoundError(String),

    /// Channel send failed
    #[error("Channel send failed: receiver disconnected")]
    ChannelSendFailedError,

    /// Invalid glob pattern
    #[error("Invalid glob pattern: {0}")]
    InvalidGlobPatternError(String),

    /// Reindex operation failed
    #[error("Reindex failed: {0}")]
    ReindexOperationFailed(String),

    /// Reindex operation timed out
    #[error("Reindex operation timed out after {0} seconds")]
    ReindexTimeoutError(u64),

    /// Database error during reindex
    #[error("Database error: {0}")]
    DatabaseOperationError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl FileWatcherErrorType {
    /// Get error code as string
    ///
    /// # 4-Word Name: get_error_code_string
    pub fn get_error_code_string(&self) -> &'static str {
        match self {
            Self::WatcherCreationFailed(_) => "WATCHER_CREATION_FAILED",
            Self::WatchPathAddFailed(_) => "WATCH_PATH_ADD_FAILED",
            Self::WatchPathRemoveFailed => "WATCH_PATH_REMOVE_FAILED",
            Self::PathNotExistsError(_) => "PATH_NOT_EXISTS",
            Self::PathNotDirectoryError(_) => "PATH_NOT_DIRECTORY",
            Self::PermissionDeniedError(_) => "PERMISSION_DENIED",
            Self::SystemLimitReachedError => "SYSTEM_LIMIT_REACHED",
            Self::WatcherAlreadyExistsError(_) => "WATCHER_ALREADY_EXISTS",
            Self::WatcherNotFoundError(_) => "WATCHER_NOT_FOUND",
            Self::ChannelSendFailedError => "CHANNEL_SEND_FAILED",
            Self::InvalidGlobPatternError(_) => "INVALID_GLOB_PATTERN",
            Self::ReindexOperationFailed(_) => "REINDEX_OPERATION_FAILED",
            Self::ReindexTimeoutError(_) => "REINDEX_TIMEOUT",
            Self::DatabaseOperationError(_) => "DATABASE_OPERATION_ERROR",
            Self::InternalError(_) => "INTERNAL_ERROR",
        }
    }
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // FileEventKindType Tests
    // =========================================================================

    /// Test FileEventKindType display formatting
    #[test]
    fn test_event_kind_display_format() {
        assert_eq!(FileEventKindType::FileCreated.to_string(), "created");
        assert_eq!(FileEventKindType::FileModified.to_string(), "modified");
        assert_eq!(FileEventKindType::FileDeleted.to_string(), "deleted");
        assert_eq!(FileEventKindType::FileRenamed.to_string(), "renamed");
    }

    /// Test FileEventKindType serialization
    #[test]
    fn test_event_kind_serialization() {
        let json = serde_json::to_string(&FileEventKindType::FileCreated).unwrap();
        assert_eq!(json, "\"file_created\"");

        let deserialized: FileEventKindType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, FileEventKindType::FileCreated);
    }

    // =========================================================================
    // RawFileEventDataStruct Tests
    // =========================================================================

    /// Test raw event creation
    #[test]
    fn test_raw_event_creation() {
        let paths = vec![PathBuf::from("/test/file.rs")];
        let event = RawFileEventDataStruct::create_new_event_data(
            FileEventKindType::FileModified,
            paths.clone(),
        );

        assert_eq!(event.event_kind_type_value, FileEventKindType::FileModified);
        assert_eq!(event.affected_paths_list_value, paths);
    }

    // =========================================================================
    // DebouncedFileChangeEventStruct Tests
    // =========================================================================

    /// Test debounced event creation
    #[test]
    fn test_debounced_event_creation() {
        let paths = vec![
            PathBuf::from("/test/a.rs"),
            PathBuf::from("/test/b.rs"),
        ];
        let event = DebouncedFileChangeEventStruct::create_new_debounced_event(
            "ws_test_123".to_string(),
            paths.clone(),
            5,
        );

        assert_eq!(event.workspace_identifier_value, "ws_test_123");
        assert_eq!(event.changed_file_paths_list, paths);
        assert_eq!(event.raw_event_count_value, 5);
        assert!(!event.is_empty_change_event());
    }

    /// Test empty debounced event detection
    #[test]
    fn test_empty_debounced_event_detection() {
        let event = DebouncedFileChangeEventStruct::create_new_debounced_event(
            "ws_test".to_string(),
            vec![],
            0,
        );

        assert!(event.is_empty_change_event());
    }

    // =========================================================================
    // WatcherConfigurationStruct Tests
    // =========================================================================

    /// Test default configuration values
    #[test]
    fn test_default_configuration_values() {
        let config = WatcherConfigurationStruct::default();

        assert_eq!(config.debounce_duration_milliseconds_value, DEFAULT_DEBOUNCE_DURATION_MS);
        assert!(config.custom_ignore_patterns_list.is_empty());
        assert_eq!(config.max_buffered_events_value, MAX_BUFFERED_EVENTS_COUNT);
    }

    /// Test debounce duration clamping - below minimum
    #[test]
    fn test_debounce_duration_clamp_minimum() {
        let config = WatcherConfigurationStruct::default()
            .with_debounce_duration_ms(50);

        assert_eq!(config.debounce_duration_milliseconds_value, MIN_DEBOUNCE_DURATION_MS);
    }

    /// Test debounce duration clamping - above maximum
    #[test]
    fn test_debounce_duration_clamp_maximum() {
        let config = WatcherConfigurationStruct::default()
            .with_debounce_duration_ms(10000);

        assert_eq!(config.debounce_duration_milliseconds_value, MAX_DEBOUNCE_DURATION_MS);
    }

    /// Test custom ignore patterns
    #[test]
    fn test_custom_ignore_patterns_setting() {
        let patterns = vec![
            "**/custom/**".to_string(),
            "**/generated/**".to_string(),
        ];
        let config = WatcherConfigurationStruct::default()
            .with_custom_ignore_patterns(patterns.clone());

        assert_eq!(config.custom_ignore_patterns_list, patterns);
    }

    // =========================================================================
    // FileWatcherErrorType Tests
    // =========================================================================

    /// Test error code strings are SCREAMING_SNAKE_CASE
    #[test]
    fn test_error_codes_screaming_snake_case() {
        let errors = vec![
            FileWatcherErrorType::WatcherCreationFailed("test".into()),
            FileWatcherErrorType::PathNotExistsError("test".into()),
            FileWatcherErrorType::SystemLimitReachedError,
            FileWatcherErrorType::WatcherAlreadyExistsError("test".into()),
        ];

        for error in errors {
            let code = error.get_error_code_string();
            assert!(
                code.chars().all(|c| c.is_uppercase() || c == '_'),
                "Error code {} is not SCREAMING_SNAKE_CASE",
                code
            );
        }
    }

    /// Test error display messages
    #[test]
    fn test_error_display_messages() {
        let error = FileWatcherErrorType::PathNotExistsError("/test/path".into());
        assert!(error.to_string().contains("/test/path"));

        let error = FileWatcherErrorType::SystemLimitReachedError;
        assert!(error.to_string().contains("sysctl"));
    }

    // =========================================================================
    // Constants Tests
    // =========================================================================

    /// Test default ignore patterns include common directories
    #[test]
    fn test_default_ignore_patterns_coverage() {
        let patterns = DEFAULT_IGNORE_PATTERNS_LIST;

        // Should include common build/dependency directories
        assert!(patterns.iter().any(|p| p.contains("target")));
        assert!(patterns.iter().any(|p| p.contains("node_modules")));
        assert!(patterns.iter().any(|p| p.contains(".git")));
        assert!(patterns.iter().any(|p| p.contains("__pycache__")));

        // Should include lock files
        assert!(patterns.iter().any(|p| p.contains("Cargo.lock")));
        assert!(patterns.iter().any(|p| p.contains("package-lock.json")));

        // Should include editor swap files
        assert!(patterns.iter().any(|p| p.contains(".swp")));
        assert!(patterns.iter().any(|p| p.contains(".DS_Store")));
    }

    /// Test constant values are reasonable
    #[test]
    fn test_constant_values_reasonable() {
        assert!(DEFAULT_DEBOUNCE_DURATION_MS >= MIN_DEBOUNCE_DURATION_MS);
        assert!(DEFAULT_DEBOUNCE_DURATION_MS <= MAX_DEBOUNCE_DURATION_MS);
        assert!(MAX_BUFFERED_EVENTS_COUNT > 0);
        assert!(MAX_BUFFERED_EVENTS_COUNT <= 10000);
    }
}
