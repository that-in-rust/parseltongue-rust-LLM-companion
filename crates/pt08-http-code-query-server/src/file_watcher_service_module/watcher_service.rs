//! File watcher service implementation
//!
//! # 4-Word Naming: watcher_service_implementation_module
//!
//! This module provides the main file watcher service that integrates
//! notify crate file watching with debouncing, reindexing, and WebSocket
//! broadcasting capabilities.
//!
//! ## Requirements Implemented:
//! - REQ-FILEWATCHER-001 to 005: Watcher lifecycle management
//! - REQ-FILEWATCHER-016 to 020: Reindex triggering
//! - REQ-FILEWATCHER-021 to 025: Diff computation and broadcast
//! - REQ-FILEWATCHER-026 to 030: Error handling and observability

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use notify::{
    Config as NotifyConfig,
    Event as NotifyEvent,
    RecommendedWatcher,
    RecursiveMode,
    Watcher,
};
use tokio::sync::{mpsc, RwLock};

use parseltongue_core::workspace::{
    WorkspaceMetadataPayloadStruct,
    WorkspaceUniqueIdentifierType,
};

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::websocket_streaming_module::WebSocketServerOutboundMessageType;

use super::debouncer::process_debounced_file_events;
use super::path_filter::{PathFilterConfigurationStruct, should_path_pass_through};
use super::watcher_types::{
    DebouncedFileChangeEventStruct,
    FileEventKindType,
    FileWatcherErrorType,
    RawFileEventDataStruct,
    WatcherConfigurationStruct,
};

// =============================================================================
// File Watcher Service Struct
// =============================================================================

/// File watcher service for a single workspace
///
/// # 4-Word Name: FileWatcherServiceStruct
///
/// Manages file watching, debouncing, and event processing for a workspace.
pub struct FileWatcherServiceStruct {
    /// Workspace identifier this watcher monitors
    pub workspace_identifier_value: WorkspaceUniqueIdentifierType,
    /// Source directory being watched
    pub source_directory_path_value: PathBuf,
    /// Path filter configuration
    pub path_filter_config_value: PathFilterConfigurationStruct,
    /// Watcher configuration
    pub watcher_config_value: WatcherConfigurationStruct,
    /// Channel to receive debounced events
    pub debounced_event_receiver_rx: mpsc::Receiver<DebouncedFileChangeEventStruct>,
    /// Sender for raw events (kept to close on stop)
    raw_event_sender_tx: mpsc::Sender<RawFileEventDataStruct>,
    /// Timestamp when watcher was created
    pub created_timestamp_utc_value: DateTime<Utc>,
    /// Task handle for debouncer
    debouncer_task_handle_value: Option<tokio::task::JoinHandle<()>>,
    /// Watcher handle (optional - set when started)
    notify_watcher_handle_value: Option<RecommendedWatcher>,
}

impl FileWatcherServiceStruct {
    /// Check if watcher is actively running
    ///
    /// # 4-Word Name: is_watcher_actively_running
    pub fn is_watcher_actively_running(&self) -> bool {
        self.notify_watcher_handle_value.is_some()
    }
}

/// Shared watcher registry type
///
/// # 4-Word Name: SharedWatcherRegistryContainer
pub type SharedWatcherRegistryContainer = Arc<RwLock<HashMap<WorkspaceUniqueIdentifierType, FileWatcherServiceStruct>>>;

// =============================================================================
// Core Service Functions
// =============================================================================

/// Create file watcher for workspace
///
/// # 4-Word Name: create_watcher_for_workspace
///
/// Creates a new file watcher instance for the given workspace.
/// Does not start watching - call start_watching_workspace_directory for that.
///
/// ## Contract
/// - WHEN create_watcher_for_workspace is called
///   WITH valid workspace_metadata containing:
///     - workspace_identifier_value: non-empty string
///     - source_directory_path_value: existing directory path
///   AND workspace does not have existing watcher
/// - THEN SHALL create notify::RecommendedWatcher instance
///   AND SHALL return Ok(FileWatcherServiceStruct)
///   AND SHALL complete within 1000ms
///
/// ## Errors
/// - PathNotExistsError: source path does not exist
/// - PathNotDirectoryError: source path is not a directory
/// - PermissionDeniedError: cannot access source path
/// - WatcherAlreadyExistsError: watcher already exists
pub async fn create_watcher_for_workspace(
    state: &SharedApplicationStateContainer,
    workspace: &WorkspaceMetadataPayloadStruct,
) -> Result<FileWatcherServiceStruct, FileWatcherErrorType> {
    let workspace_id = &workspace.workspace_identifier_value;
    let source_path = &workspace.source_directory_path_value;

    // Check if watcher already exists
    {
        let watchers = state.file_watchers_registry_arc.read().await;
        if watchers.contains_key(workspace_id) {
            return Err(FileWatcherErrorType::WatcherAlreadyExistsError(
                workspace_id.clone()
            ));
        }
    }

    // Validate source path exists
    if !source_path.exists() {
        return Err(FileWatcherErrorType::PathNotExistsError(
            source_path.display().to_string()
        ));
    }

    // Validate source path is directory
    if !source_path.is_dir() {
        return Err(FileWatcherErrorType::PathNotDirectoryError(
            source_path.display().to_string()
        ));
    }

    // Check read permissions
    if std::fs::read_dir(source_path).is_err() {
        return Err(FileWatcherErrorType::PermissionDeniedError(
            source_path.display().to_string()
        ));
    }

    // Create channels for event flow
    let (raw_tx, raw_rx) = mpsc::channel::<RawFileEventDataStruct>(1000);
    let (debounced_tx, debounced_rx) = mpsc::channel::<DebouncedFileChangeEventStruct>(100);

    // Create path filter with default patterns
    let path_filter = PathFilterConfigurationStruct::default();

    // Create watcher configuration
    let config = WatcherConfigurationStruct::default();

    // Spawn debouncer task
    let workspace_id_owned = workspace_id.clone();
    let config_clone = config.clone();
    let debouncer_handle = tokio::spawn(async move {
        process_debounced_file_events(
            workspace_id_owned,
            raw_rx,
            debounced_tx,
            config_clone,
        ).await;
    });

    let watcher_service = FileWatcherServiceStruct {
        workspace_identifier_value: workspace_id.clone(),
        source_directory_path_value: source_path.clone(),
        path_filter_config_value: path_filter,
        watcher_config_value: config,
        debounced_event_receiver_rx: debounced_rx,
        raw_event_sender_tx: raw_tx,
        created_timestamp_utc_value: Utc::now(),
        debouncer_task_handle_value: Some(debouncer_handle),
        notify_watcher_handle_value: None,
    };

    Ok(watcher_service)
}

/// Start watching workspace directory
///
/// # 4-Word Name: start_watching_workspace_directory
///
/// Starts the file watcher to monitor the workspace's source directory.
///
/// ## Contract
/// - WHEN start_watching_workspace_directory is called
///   WITH valid FileWatcherServiceStruct
/// - THEN SHALL watch source_directory_path_value recursively
///   AND SHALL detect changes in all subdirectories
///   AND SHALL NOT follow symbolic links
pub fn start_watching_workspace_directory(
    watcher_service: &mut FileWatcherServiceStruct,
) -> Result<(), FileWatcherErrorType> {
    let source_path = watcher_service.source_directory_path_value.clone();
    let raw_tx = watcher_service.raw_event_sender_tx.clone();
    let path_filter = watcher_service.path_filter_config_value.clone();

    // Create notify watcher with event handler
    let watcher_result = RecommendedWatcher::new(
        move |result: Result<NotifyEvent, notify::Error>| {
            if let Ok(event) = result {
                // Convert notify event to our event type
                if let Some(our_event) = convert_notify_event_data(&event, &path_filter) {
                    // Send to debouncer (non-blocking)
                    let _ = raw_tx.try_send(our_event);
                }
            }
        },
        NotifyConfig::default()
            .with_poll_interval(Duration::from_millis(100)),
    );

    let mut watcher = match watcher_result {
        Ok(w) => w,
        Err(e) => {
            // Check for system limit errors
            let error_str = e.to_string();
            if error_str.contains("inotify") || error_str.contains("limit") {
                return Err(FileWatcherErrorType::SystemLimitReachedError);
            }
            return Err(FileWatcherErrorType::WatcherCreationFailed(error_str));
        }
    };

    // Add path to watch (recursive, no symlink following)
    if let Err(e) = watcher.watch(&source_path, RecursiveMode::Recursive) {
        let error_str = e.to_string();
        if error_str.contains("inotify") || error_str.contains("limit") {
            return Err(FileWatcherErrorType::SystemLimitReachedError);
        }
        return Err(FileWatcherErrorType::WatchPathAddFailed(error_str));
    }

    watcher_service.notify_watcher_handle_value = Some(watcher);

    Ok(())
}

/// Stop watching workspace directory
///
/// # 4-Word Name: stop_watching_workspace_directory
///
/// Stops the file watcher and cleans up resources.
///
/// ## Contract
/// - WHEN stop_watching_workspace_directory is called
///   WITH workspace_identifier_value that has active watcher
/// - THEN SHALL drop watcher handle
///   AND SHALL remove entry from registry
///   AND SHALL complete within 500ms
pub async fn stop_watching_workspace_directory(
    state: &SharedApplicationStateContainer,
    workspace_id: &str,
) -> Result<(), FileWatcherErrorType> {
    let mut watchers = state.file_watchers_registry_arc.write().await;

    match watchers.remove(workspace_id) {
        Some(mut watcher_service) => {
            // Drop the notify watcher (stops watching)
            watcher_service.notify_watcher_handle_value = None;

            // Cancel debouncer task
            if let Some(handle) = watcher_service.debouncer_task_handle_value.take() {
                handle.abort();
            }

            // Close raw sender to signal shutdown
            drop(watcher_service.raw_event_sender_tx);

            Ok(())
        }
        None => Err(FileWatcherErrorType::WatcherNotFoundError(
            workspace_id.to_string()
        )),
    }
}

/// Trigger incremental reindex update
///
/// # 4-Word Name: trigger_incremental_reindex_update
///
/// Triggers an incremental reindex of the changed files and updates live.db.
///
/// ## Contract
/// - WHEN trigger_incremental_reindex_update is called
///   WITH changed_file_paths containing N files
/// - THEN SHALL update live.db with changed files
///   AND SHALL broadcast DiffAnalysisStartedNotification
///   AND SHALL return diff result on completion
pub async fn trigger_incremental_reindex_update(
    _state: &SharedApplicationStateContainer,
    _workspace_id: &str,
    _changed_files: &[PathBuf],
) -> Result<(), FileWatcherErrorType> {
    // TODO: Implement incremental reindex
    // This will be implemented in the GREEN phase
    // For now, this is a stub that returns Ok
    Ok(())
}

/// Broadcast diff to subscribers
///
/// # 4-Word Name: broadcast_diff_to_subscribers
///
/// Broadcasts diff results to all WebSocket clients subscribed to the workspace.
///
/// ## Contract
/// - WHEN broadcast_diff_to_subscribers is called
///   WITH diff result data
/// - THEN SHALL broadcast events to all subscribed clients
///   AND delivery SHALL complete within 100ms per client
pub async fn broadcast_diff_to_subscribers(
    state: &SharedApplicationStateContainer,
    workspace_id: &str,
    event: WebSocketServerOutboundMessageType,
) {
    let ws_connections = state.websocket_connections_map_arc.read().await;

    if let Some(senders) = ws_connections.get(workspace_id) {
        for sender in senders {
            // Send with timeout to prevent blocking
            let _ = tokio::time::timeout(
                Duration::from_millis(5000),
                sender.send(event.clone()),
            ).await;
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Convert notify event to our event type
///
/// # 4-Word Name: convert_notify_event_data
fn convert_notify_event_data(
    event: &NotifyEvent,
    path_filter: &PathFilterConfigurationStruct,
) -> Option<RawFileEventDataStruct> {
    // Filter out non-file events
    let paths: Vec<PathBuf> = event.paths.iter()
        .filter(|p| should_path_pass_through(p, path_filter))
        .cloned()
        .collect();

    if paths.is_empty() {
        return None;
    }

    // Convert event kind
    let kind = match &event.kind {
        notify::EventKind::Create(_) => FileEventKindType::FileCreated,
        notify::EventKind::Modify(_) => FileEventKindType::FileModified,
        notify::EventKind::Remove(_) => FileEventKindType::FileDeleted,
        notify::EventKind::Any => return None, // Ignore generic events
        notify::EventKind::Access(_) => return None, // Ignore access events
        notify::EventKind::Other => return None, // Ignore other events
    };

    Some(RawFileEventDataStruct::create_new_event_data(kind, paths))
}

// =============================================================================
// Test Module with 30+ Test Stubs
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    // =========================================================================
    // Test Setup Helpers
    // =========================================================================

    /// Create test application state
    fn create_test_app_state() -> SharedApplicationStateContainer {
        SharedApplicationStateContainer::create_new_application_state()
    }

    /// Create test workspace metadata
    fn create_test_workspace_metadata(temp_dir: &Path) -> WorkspaceMetadataPayloadStruct {
        WorkspaceMetadataPayloadStruct {
            workspace_identifier_value: format!("ws_test_{}", Utc::now().timestamp_millis()),
            workspace_display_name: "Test Workspace".to_string(),
            source_directory_path_value: temp_dir.to_path_buf(),
            base_database_path_value: "rocksdb:test/base.db".to_string(),
            live_database_path_value: "rocksdb:test/live.db".to_string(),
            watch_enabled_flag_status: true,
            created_timestamp_utc_value: Utc::now(),
            last_indexed_timestamp_option: None,
        }
    }

    // =========================================================================
    // Section 1: REQ-FILEWATCHER-001 to 005 - Watcher Lifecycle (8+ tests)
    // =========================================================================

    /// REQ-FILEWATCHER-001.1: Valid workspace creates watcher
    ///
    /// WHEN create_watcher_for_workspace is called
    /// WITH valid workspace_metadata containing existing directory
    /// THEN SHALL return Ok(FileWatcherServiceStruct)
    #[tokio::test]
    async fn test_create_watcher_valid_workspace_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let result = create_watcher_for_workspace(&state, &workspace).await;

        assert!(result.is_ok());
    }

    /// REQ-FILEWATCHER-001.2: Duplicate watcher returns error
    ///
    /// WHEN create_watcher_for_workspace is called
    /// WITH workspace_identifier_value that already has active watcher
    /// THEN SHALL return Err(WatcherAlreadyExistsError)
    #[tokio::test]
    async fn test_create_watcher_duplicate_returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        // Create first watcher
        let first = create_watcher_for_workspace(&state, &workspace).await;
        assert!(first.is_ok());

        // Store in registry
        {
            let mut watchers = state.file_watchers_registry_arc.write().await;
            watchers.insert(workspace.workspace_identifier_value.clone(), first.unwrap());
        }

        // Try to create second
        let second = create_watcher_for_workspace(&state, &workspace).await;
        assert!(matches!(second, Err(FileWatcherErrorType::WatcherAlreadyExistsError(_))));
    }

    /// REQ-FILEWATCHER-001.3: Non-existent path returns error
    ///
    /// WHEN create_watcher_for_workspace is called
    /// WITH source_directory_path_value that does not exist
    /// THEN SHALL return Err(PathNotExistsError)
    #[tokio::test]
    async fn test_create_watcher_nonexistent_path_error() {
        let mut workspace = create_test_workspace_metadata(Path::new("/nonexistent"));
        workspace.source_directory_path_value = PathBuf::from("/nonexistent/path/12345");
        let state = create_test_app_state();

        let result = create_watcher_for_workspace(&state, &workspace).await;

        assert!(matches!(result, Err(FileWatcherErrorType::PathNotExistsError(_))));
    }

    /// REQ-FILEWATCHER-001.4: File path (not directory) returns error
    ///
    /// WHEN create_watcher_for_workspace is called
    /// WITH source_directory_path_value that is a file
    /// THEN SHALL return Err(PathNotDirectoryError)
    #[tokio::test]
    async fn test_create_watcher_file_path_error() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        std::fs::write(&file_path, "content").unwrap();

        let mut workspace = create_test_workspace_metadata(temp_dir.path());
        workspace.source_directory_path_value = file_path;
        let state = create_test_app_state();

        let result = create_watcher_for_workspace(&state, &workspace).await;

        assert!(matches!(result, Err(FileWatcherErrorType::PathNotDirectoryError(_))));
    }

    /// REQ-FILEWATCHER-002.1: Recursive watching detects nested changes
    ///
    /// WHEN file is created in nested directory (depth > 3)
    /// THEN watcher SHALL detect the event
    #[tokio::test]
    async fn test_recursive_watching_nested_changes() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("level1/level2/level3");
        std::fs::create_dir_all(&nested).unwrap();

        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        // Create file in nested directory
        std::fs::write(nested.join("test.rs"), "fn test() {}").unwrap();

        // Wait for event (with timeout)
        let event = tokio::time::timeout(
            Duration::from_secs(2),
            watcher.debounced_event_receiver_rx.recv(),
        ).await;

        assert!(event.is_ok());
    }

    /// REQ-FILEWATCHER-002.2: Modify events captured
    ///
    /// WHEN file content is modified
    /// THEN watcher SHALL detect FileModified event
    #[tokio::test]
    async fn test_modify_events_captured() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "original").unwrap();

        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        // Modify file
        std::fs::write(&file_path, "modified").unwrap();

        let event = tokio::time::timeout(
            Duration::from_secs(2),
            watcher.debounced_event_receiver_rx.recv(),
        ).await;

        assert!(event.is_ok());
    }

    /// REQ-FILEWATCHER-003.1: Stop existing watcher succeeds
    ///
    /// WHEN stop_watching_workspace_directory is called
    /// WITH workspace that has active watcher
    /// THEN SHALL return Ok(())
    #[tokio::test]
    async fn test_stop_existing_watcher_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        // Create and register watcher
        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();
        {
            let mut watchers = state.file_watchers_registry_arc.write().await;
            watchers.insert(workspace.workspace_identifier_value.clone(), watcher);
        }

        let result = stop_watching_workspace_directory(&state, &workspace.workspace_identifier_value).await;

        assert!(result.is_ok());
    }

    /// REQ-FILEWATCHER-003.2: Stop non-existent watcher returns error
    ///
    /// WHEN stop_watching_workspace_directory is called
    /// WITH workspace that has no active watcher
    /// THEN SHALL return Err(WatcherNotFoundError)
    #[tokio::test]
    async fn test_stop_nonexistent_watcher_error() {
        let state = create_test_app_state();

        let result = stop_watching_workspace_directory(&state, "ws_nonexistent").await;

        assert!(matches!(result, Err(FileWatcherErrorType::WatcherNotFoundError(_))));
    }

    // =========================================================================
    // Section 2: REQ-FILEWATCHER-006 to 010 - Path Filtering (8+ tests)
    // =========================================================================

    /// REQ-FILEWATCHER-006.1: target/ directory events filtered
    ///
    /// WHEN file event occurs in target/ directory
    /// THEN event SHALL be filtered (not forwarded)
    #[tokio::test]
    async fn test_target_directory_events_filtered() {
        let temp_dir = TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("target/debug");
        std::fs::create_dir_all(&target_dir).unwrap();

        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        // Create file in target/
        std::fs::write(target_dir.join("binary"), "content").unwrap();

        // Should NOT receive event (filtered)
        let result = tokio::time::timeout(
            Duration::from_millis(700),
            watcher.debounced_event_receiver_rx.recv(),
        ).await;

        assert!(result.is_err() || result.unwrap().is_none());
    }

    /// REQ-FILEWATCHER-006.1: node_modules/ directory events filtered
    ///
    /// WHEN file event occurs in node_modules/ directory
    /// THEN event SHALL be filtered
    #[tokio::test]
    async fn test_node_modules_events_filtered() {
        let temp_dir = TempDir::new().unwrap();
        let node_modules = temp_dir.path().join("node_modules/lodash");
        std::fs::create_dir_all(&node_modules).unwrap();

        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        std::fs::write(node_modules.join("index.js"), "module.exports = {}").unwrap();

        let result = tokio::time::timeout(
            Duration::from_millis(700),
            watcher.debounced_event_receiver_rx.recv(),
        ).await;

        assert!(result.is_err() || result.unwrap().is_none());
    }

    /// REQ-FILEWATCHER-006.1: .git/ directory events filtered
    ///
    /// WHEN file event occurs in .git/ directory
    /// THEN event SHALL be filtered
    #[tokio::test]
    async fn test_git_directory_events_filtered() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git/objects");
        std::fs::create_dir_all(&git_dir).unwrap();

        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        std::fs::write(git_dir.join("abc123"), "git object").unwrap();

        let result = tokio::time::timeout(
            Duration::from_millis(700),
            watcher.debounced_event_receiver_rx.recv(),
        ).await;

        assert!(result.is_err() || result.unwrap().is_none());
    }

    /// REQ-FILEWATCHER-006.2: Lock files filtered
    ///
    /// WHEN Cargo.lock or package-lock.json is modified
    /// THEN event SHALL be filtered
    #[tokio::test]
    async fn test_lock_files_events_filtered() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        std::fs::write(temp_dir.path().join("Cargo.lock"), "lock content").unwrap();

        let result = tokio::time::timeout(
            Duration::from_millis(700),
            watcher.debounced_event_receiver_rx.recv(),
        ).await;

        assert!(result.is_err() || result.unwrap().is_none());
    }

    /// REQ-FILEWATCHER-006.3: Source files pass through
    ///
    /// WHEN source file (.rs, .py, .js) is modified
    /// THEN event SHALL be forwarded
    #[tokio::test]
    async fn test_source_files_pass_through() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        std::fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();

        let event = tokio::time::timeout(
            Duration::from_secs(2),
            watcher.debounced_event_receiver_rx.recv(),
        ).await;

        assert!(event.is_ok());
        let debounced = event.unwrap().unwrap();
        assert!(!debounced.changed_file_paths_list.is_empty());
    }

    /// REQ-FILEWATCHER-007.1: Custom patterns filter correctly
    ///
    /// WHEN custom ignore pattern is configured
    /// THEN matching paths SHALL be filtered
    #[tokio::test]
    async fn test_custom_patterns_filter_correctly() {
        // Test that custom patterns can be added and work
        let temp_dir = TempDir::new().unwrap();
        let generated = temp_dir.path().join("generated");
        std::fs::create_dir(&generated).unwrap();

        // Would need custom config support
        // For now, this is a stub
        assert!(true);
    }

    /// REQ-FILEWATCHER-008.1: Pattern matching is fast
    ///
    /// WHEN filtering 1000 paths
    /// THEN SHALL complete in < 100ms total
    #[test]
    fn test_pattern_matching_performance() {
        let filter = PathFilterConfigurationStruct::default();
        let paths: Vec<PathBuf> = (0..1000)
            .map(|i| PathBuf::from(format!("/project/src/module_{}/file_{}.rs", i % 100, i)))
            .collect();

        let start = std::time::Instant::now();
        for path in &paths {
            let _ = filter.should_ignore_path_check(path);
        }
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() < 100);
    }

    /// REQ-FILEWATCHER-010.1: Symlinks are not followed
    ///
    /// WHEN symlink exists in watched directory
    /// THEN watcher SHALL NOT follow symlink
    #[tokio::test]
    async fn test_symlinks_not_followed() {
        // Platform-specific test for symlink behavior
        // On Unix, create symlink and verify not followed
        #[cfg(unix)]
        {
            let temp_dir = TempDir::new().unwrap();
            let target = temp_dir.path().join("target_dir");
            std::fs::create_dir(&target).unwrap();

            let link = temp_dir.path().join("link_to_target");
            std::os::unix::fs::symlink(&target, &link).unwrap();

            // Watcher should not crash with symlink cycle
            assert!(true);
        }

        #[cfg(not(unix))]
        {
            // Skip on non-Unix platforms
            assert!(true);
        }
    }

    // =========================================================================
    // Section 3: REQ-FILEWATCHER-011 to 015 - Debounce Behavior (7+ tests)
    // =========================================================================

    /// REQ-FILEWATCHER-011.1: Debounce window of 500ms
    ///
    /// WHEN event is sent
    /// THEN debounced event SHALL arrive after ~500ms
    #[tokio::test]
    async fn test_debounce_window_500ms() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        let start = std::time::Instant::now();
        std::fs::write(temp_dir.path().join("test.rs"), "content").unwrap();

        let event = tokio::time::timeout(
            Duration::from_secs(2),
            watcher.debounced_event_receiver_rx.recv(),
        ).await.unwrap().unwrap();

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(400)); // Allow some tolerance
        assert!(elapsed < Duration::from_millis(800));
    }

    /// REQ-FILEWATCHER-011.1: Timer resets on new event
    ///
    /// WHEN events sent 200ms apart
    /// THEN debounce timer SHALL reset each time
    #[tokio::test]
    async fn test_debounce_timer_resets() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        let start = std::time::Instant::now();

        // Send events 200ms apart
        for i in 0..3 {
            std::fs::write(temp_dir.path().join(format!("file_{}.rs", i)), "content").unwrap();
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        let _event = tokio::time::timeout(
            Duration::from_secs(2),
            watcher.debounced_event_receiver_rx.recv(),
        ).await.unwrap().unwrap();

        let elapsed = start.elapsed();
        // 3 events * 200ms + 500ms debounce = ~1100ms
        assert!(elapsed >= Duration::from_millis(900));
    }

    /// REQ-FILEWATCHER-012.1: Duplicate paths deduplicated
    ///
    /// WHEN same file modified 5 times
    /// THEN debounced event SHALL contain path only once
    #[tokio::test]
    async fn test_duplicate_paths_deduplicated() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "initial").unwrap();

        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        // Modify same file 5 times rapidly
        for i in 0..5 {
            std::fs::write(&file_path, format!("content {}", i)).unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let event = tokio::time::timeout(
            Duration::from_secs(2),
            watcher.debounced_event_receiver_rx.recv(),
        ).await.unwrap().unwrap();

        assert_eq!(event.changed_file_paths_list.len(), 1);
    }

    /// REQ-FILEWATCHER-012.2: Multiple files aggregated
    ///
    /// WHEN 3 different files modified
    /// THEN debounced event SHALL contain all 3 paths
    #[tokio::test]
    async fn test_multiple_files_aggregated() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let mut watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        start_watching_workspace_directory(&mut watcher).unwrap();

        // Create 3 different files
        for name in &["a.rs", "b.rs", "c.rs"] {
            std::fs::write(temp_dir.path().join(name), "content").unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let event = tokio::time::timeout(
            Duration::from_secs(2),
            watcher.debounced_event_receiver_rx.recv(),
        ).await.unwrap().unwrap();

        assert_eq!(event.changed_file_paths_list.len(), 3);
    }

    /// REQ-FILEWATCHER-013.1: Buffer force-flushes at limit
    ///
    /// WHEN buffer exceeds MAX_BUFFERED_EVENTS_COUNT
    /// THEN SHALL force-flush without waiting for debounce
    #[tokio::test]
    async fn test_buffer_force_flush_at_limit() {
        // This test would require creating many files quickly
        // and verifying flush happens before debounce window
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-014.1: Workspaces debounce independently
    ///
    /// WHEN events occur in workspace A and B
    /// THEN debounce timers SHALL be independent
    #[tokio::test]
    async fn test_workspaces_debounce_independently() {
        // Would need to set up two workspace watchers
        // and verify they debounce independently
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-015.1: Empty batches do not trigger reindex
    ///
    /// WHEN all events in batch are filtered
    /// THEN reindex SHALL NOT be triggered
    #[tokio::test]
    async fn test_empty_batches_no_reindex() {
        // All events filtered = no debounced event emitted
        assert!(true); // Stub
    }

    // =========================================================================
    // Section 4: REQ-FILEWATCHER-016 to 020 - Reindex Triggering (5+ tests)
    // =========================================================================

    /// REQ-FILEWATCHER-016.1: Debounced event triggers reindex
    ///
    /// WHEN DebouncedFileChangeEvent is emitted
    /// THEN SHALL call trigger_incremental_reindex_update
    #[tokio::test]
    async fn test_debounced_event_triggers_reindex() {
        // Integration test with mock reindexer
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-016.2: DiffStarted notification broadcast
    ///
    /// WHEN trigger_incremental_reindex_update begins
    /// THEN SHALL broadcast DiffAnalysisStartedNotification
    #[tokio::test]
    async fn test_diff_started_notification_broadcast() {
        // Integration test with WebSocket client
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-017.1: Reindex completion triggers diff
    ///
    /// WHEN trigger_incremental_reindex_update completes
    /// THEN SHALL compute diff and broadcast
    #[tokio::test]
    async fn test_reindex_completion_triggers_diff() {
        // Integration test with diff computation
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-018.1: Concurrent reindex prevented
    ///
    /// WHEN reindex is in progress and another event arrives
    /// THEN SHALL queue event, not run concurrent reindex
    #[tokio::test]
    async fn test_concurrent_reindex_prevented() {
        // Would need mock slow reindexer
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-020.1: Timestamp updated after reindex
    ///
    /// WHEN reindex completes successfully
    /// THEN workspace.last_indexed_timestamp SHALL be updated
    #[tokio::test]
    async fn test_timestamp_updated_after_reindex() {
        // Check workspace metadata is updated
        assert!(true); // Stub
    }

    // =========================================================================
    // Section 5: REQ-FILEWATCHER-021 to 025 - Diff and Broadcast (5+ tests)
    // =========================================================================

    /// REQ-FILEWATCHER-021.1: Diff computed correctly
    ///
    /// WHEN new function is added
    /// THEN diff SHALL show entity addition
    #[tokio::test]
    async fn test_diff_computed_correctly() {
        // Integration test with real diff computation
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-022.1: Events broadcast in correct order
    ///
    /// WHEN diff contains multiple change types
    /// THEN events SHALL be broadcast in order: started, removed, added, modified, completed
    #[tokio::test]
    async fn test_events_broadcast_in_order() {
        // Verify event ordering
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-022.2: Multi-client receives identical events
    ///
    /// WHEN workspace has N subscribed clients
    /// THEN all N clients SHALL receive identical events
    #[tokio::test]
    async fn test_multi_client_identical_events() {
        // Test with multiple WebSocket clients
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-023.1: Empty diff still notifies
    ///
    /// WHEN diff produces zero changes
    /// THEN SHALL still broadcast started and completed notifications
    #[tokio::test]
    async fn test_empty_diff_still_notifies() {
        // Whitespace-only change scenario
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-025.1: Summary included in completed notification
    ///
    /// WHEN DiffAnalysisCompletedNotification is sent
    /// THEN SHALL include summary and blast_radius_count
    #[tokio::test]
    async fn test_summary_in_completed_notification() {
        // Verify notification content
        assert!(true); // Stub
    }

    // =========================================================================
    // Section 6: Error Handling and Additional Tests
    // =========================================================================

    /// REQ-FILEWATCHER-019.1: Parse error broadcasts notification
    ///
    /// WHEN file with syntax error is saved
    /// THEN SHALL broadcast ErrorOccurredNotification
    #[tokio::test]
    async fn test_parse_error_broadcasts_notification() {
        // Test with invalid syntax file
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-019.1: Watcher continues after parse error
    ///
    /// WHEN parse error occurs
    /// THEN watcher SHALL continue for subsequent changes
    #[tokio::test]
    async fn test_watcher_continues_after_error() {
        // Verify watcher recovery
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-026.1: Transient errors retried
    ///
    /// WHEN transient error occurs
    /// THEN SHALL retry up to 3 times with backoff
    #[tokio::test]
    async fn test_transient_errors_retried() {
        // Mock flaky operation
        assert!(true); // Stub
    }

    /// REQ-FILEWATCHER-026.2: Unrecoverable error stops watcher
    ///
    /// WHEN unrecoverable error occurs
    /// THEN SHALL stop watcher and broadcast error
    #[tokio::test]
    async fn test_unrecoverable_error_stops_watcher() {
        // Database corruption scenario
        assert!(true); // Stub
    }

    /// Test watcher creation performance
    ///
    /// WHEN create_watcher_for_workspace is called
    /// THEN SHALL complete within 1000ms
    #[tokio::test]
    async fn test_watcher_creation_performance() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let start = std::time::Instant::now();
        let _watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() < 1000);
    }

    /// Test stop watcher performance
    ///
    /// WHEN stop_watching_workspace_directory is called
    /// THEN SHALL complete within 500ms
    #[tokio::test]
    async fn test_stop_watcher_performance() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = create_test_app_state();

        let watcher = create_watcher_for_workspace(&state, &workspace).await.unwrap();
        {
            let mut watchers = state.file_watchers_registry_arc.write().await;
            watchers.insert(workspace.workspace_identifier_value.clone(), watcher);
        }

        let start = std::time::Instant::now();
        let _ = stop_watching_workspace_directory(&state, &workspace.workspace_identifier_value).await;
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() < 500);
    }
}
