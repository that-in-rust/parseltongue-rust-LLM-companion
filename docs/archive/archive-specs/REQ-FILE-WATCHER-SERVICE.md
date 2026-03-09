# File Watcher Service Specification

## REQ-FILE-WATCHER: Real-Time File System Monitoring Service

**Document Version**: 1.0.0
**Created**: 2026-01-23
**Status**: Specification Complete
**Phase**: 2.2 - WebSocket Streaming Backend
**Dependencies**: Phase 2.1 (Workspace Management) Complete, notify crate v6.x

---

## Overview

### Problem Statement

Developers need real-time feedback when editing code. Without automated file watching:

1. Changes require manual re-indexing to see impact
2. Blast radius visualization becomes stale immediately after edits
3. Collaboration suffers when multiple developers cannot see live changes
4. Context switching between editing and running diff commands disrupts flow

The File Watcher Service bridges the gap between file system events and the WebSocket streaming system, enabling true real-time diff visualization.

### Solution Architecture

```
File System Events (notify crate)
         |
         v
+------------------------+
| FileWatcherServiceImpl |
|  - Watches directory   |
|  - Filters ignored     |
|  - Batches events      |
+------------------------+
         |
         | Raw Events
         v
+------------------------+
| DebouncerServiceImpl   |
|  - 500ms window        |
|  - Deduplicates        |
|  - Aggregates paths    |
+------------------------+
         |
         | Debounced Event
         v
+------------------------+
| ReindexTriggerService  |
|  - Incremental reindex |
|  - Update live.db      |
|  - Compute diff        |
+------------------------+
         |
         | DiffResultDataPayload
         v
+------------------------+
| WebSocket Broadcaster  |
|  - Stream to clients   |
|  - Entity/Edge events  |
+------------------------+
```

### Integration Points

- **WorkspaceManagerServiceStruct**: Provides workspace metadata (source_directory_path_value)
- **SharedWorkspaceStateContainer**: Stores watcher handles in `watchers` map
- **WebSocket Handler**: Receives diff results via broadcast channel
- **DiffResultDataPayload**: Data format from parseltongue-core/diff module

---

## Data Types

### Core Types

```rust
/// File watcher service for a single workspace
///
/// # 4-Word Name: FileWatcherServiceStruct
pub struct FileWatcherServiceStruct {
    /// Workspace identifier this watcher monitors
    pub workspace_identifier_value: WorkspaceUniqueIdentifierType,
    /// Handle to notify crate watcher (dropped to stop)
    pub watcher_instance_handle: RecommendedWatcher,
    /// Channel to send debounced events
    pub event_sender_channel: mpsc::Sender<DebouncedFileChangeEvent>,
    /// Path patterns to ignore
    pub ignored_patterns_list: Vec<GlobPattern>,
}

/// Debounced file change event after aggregation
///
/// # 4-Word Name: DebouncedFileChangeEvent
#[derive(Debug, Clone)]
pub struct DebouncedFileChangeEvent {
    /// Workspace where changes occurred
    pub workspace_identifier_value: WorkspaceUniqueIdentifierType,
    /// List of changed file paths (deduplicated)
    pub changed_file_paths: Vec<PathBuf>,
    /// Timestamp when debounce window closed
    pub debounce_completed_timestamp: DateTime<Utc>,
    /// Count of raw events before deduplication
    pub raw_event_count_value: usize,
}

/// Raw file event from notify crate
///
/// # 4-Word Name: RawFileEventData
#[derive(Debug, Clone)]
pub struct RawFileEventData {
    /// Type of file system event
    pub event_kind_type: FileEventKindType,
    /// Affected file path(s)
    pub affected_paths_list: Vec<PathBuf>,
    /// Timestamp when event occurred
    pub event_timestamp_value: DateTime<Utc>,
}

/// File event kind classification
///
/// # 4-Word Name: FileEventKindType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEventKindType {
    /// File was created
    FileCreated,
    /// File was modified (content changed)
    FileModified,
    /// File was deleted
    FileDeleted,
    /// File was renamed
    FileRenamed,
}

/// Error types for file watcher operations
///
/// # 4-Word Name: FileWatcherErrorType
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileWatcherErrorType {
    /// Failed to create notify watcher
    WatcherCreationFailed,
    /// Failed to add path to watch
    WatchPathAddFailed,
    /// Failed to remove path from watch
    WatchPathRemoveFailed,
    /// Path does not exist
    PathNotExistsError,
    /// Path is not a directory
    PathNotDirectoryError,
    /// Permission denied accessing path
    PermissionDeniedError,
    /// inotify/FSEvents limit reached
    SystemLimitReachedError,
    /// Watcher already exists for workspace
    WatcherAlreadyExistsError,
    /// Watcher not found for workspace
    WatcherNotFoundError,
    /// Channel send failed
    ChannelSendFailedError,
}
```

### Configuration Constants

```rust
/// Default debounce duration in milliseconds
///
/// # 4-Word Name: DEFAULT_DEBOUNCE_DURATION_MS
pub const DEFAULT_DEBOUNCE_DURATION_MS: u64 = 500;

/// Maximum events to buffer before force-flush
///
/// # 4-Word Name: MAX_BUFFERED_EVENTS_COUNT
pub const MAX_BUFFERED_EVENTS_COUNT: usize = 1000;

/// Default ignore patterns for common build artifacts
///
/// # 4-Word Name: DEFAULT_IGNORE_PATTERNS_LIST
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
];
```

---

# Section 1: Watcher Creation and Lifecycle

## REQ-FILEWATCHER-001: Create Watcher for Workspace

### Problem Statement

When a workspace has `watch_enabled_flag_status` toggled to `true`, the system must create a file watcher that monitors the workspace's source directory for changes.

### Specification

#### REQ-FILEWATCHER-001.1: Successful Watcher Creation

```
WHEN create_watcher_for_workspace is called
  WITH valid workspace_metadata containing:
    - workspace_identifier_value: non-empty string
    - source_directory_path_value: existing directory path
  AND workspace does not have existing watcher
THEN SHALL create notify::RecommendedWatcher instance
  AND SHALL configure watcher for recursive mode (watch subdirectories)
  AND SHALL register watch on source_directory_path_value
  AND SHALL store watcher handle in SharedWorkspaceStateContainer.watchers
  AND SHALL return Ok(FileWatcherServiceStruct)
  AND SHALL complete within 1000ms
```

#### REQ-FILEWATCHER-001.2: Watcher Already Exists

```
WHEN create_watcher_for_workspace is called
  WITH workspace_identifier_value that already has active watcher
THEN SHALL return Err(FileWatcherErrorType::WatcherAlreadyExistsError)
  AND SHALL NOT create duplicate watcher
  AND SHALL NOT modify existing watcher
```

#### REQ-FILEWATCHER-001.3: Source Path Does Not Exist

```
WHEN create_watcher_for_workspace is called
  WITH source_directory_path_value that does not exist on filesystem
THEN SHALL return Err(FileWatcherErrorType::PathNotExistsError)
  AND SHALL NOT create watcher
  AND SHALL include path in error context
```

#### REQ-FILEWATCHER-001.4: Source Path Is Not Directory

```
WHEN create_watcher_for_workspace is called
  WITH source_directory_path_value that is a file (not directory)
THEN SHALL return Err(FileWatcherErrorType::PathNotDirectoryError)
  AND SHALL NOT create watcher
```

#### REQ-FILEWATCHER-001.5: Permission Denied

```
WHEN create_watcher_for_workspace is called
  WITH source_directory_path_value that is not readable by current user
THEN SHALL return Err(FileWatcherErrorType::PermissionDeniedError)
  AND SHALL NOT create watcher
  AND SHALL log error at WARN level
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_001_tests {
    use super::*;
    use tempfile::TempDir;

    /// REQ-FILEWATCHER-001.1: Valid workspace creates watcher
    #[tokio::test]
    async fn test_create_watcher_valid_workspace_succeeds() {
        // GIVEN a temp directory and workspace metadata
        let temp_dir = TempDir::new().unwrap();
        let workspace = create_test_workspace_metadata(temp_dir.path());
        let state = SharedWorkspaceStateContainer::new();

        // WHEN creating watcher
        let result = create_watcher_for_workspace(&state, &workspace).await;

        // THEN should succeed and register in state
        assert!(result.is_ok());
        let watchers = state.watchers.read().await;
        assert!(watchers.contains_key(&workspace.workspace_identifier_value));
    }

    /// REQ-FILEWATCHER-001.2: Duplicate watcher returns error
    #[tokio::test]
    async fn test_create_watcher_duplicate_returns_error() {
        // GIVEN an existing watcher for workspace
        let (state, workspace) = setup_workspace_with_watcher().await;

        // WHEN creating watcher again
        let result = create_watcher_for_workspace(&state, &workspace).await;

        // THEN should return WatcherAlreadyExistsError
        assert_eq!(result.unwrap_err(), FileWatcherErrorType::WatcherAlreadyExistsError);
    }

    /// REQ-FILEWATCHER-001.3: Non-existent path returns error
    #[tokio::test]
    async fn test_create_watcher_nonexistent_path_returns_error() {
        // GIVEN workspace with non-existent source path
        let workspace = create_workspace_with_path("/nonexistent/path/12345");
        let state = SharedWorkspaceStateContainer::new();

        // WHEN creating watcher
        let result = create_watcher_for_workspace(&state, &workspace).await;

        // THEN should return PathNotExistsError
        assert_eq!(result.unwrap_err(), FileWatcherErrorType::PathNotExistsError);
    }

    /// REQ-FILEWATCHER-001.4: File path (not directory) returns error
    #[tokio::test]
    async fn test_create_watcher_file_path_returns_error() {
        // GIVEN a file (not directory)
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.txt");
        std::fs::write(&file_path, "content").unwrap();
        let workspace = create_workspace_with_path(&file_path);
        let state = SharedWorkspaceStateContainer::new();

        // WHEN creating watcher
        let result = create_watcher_for_workspace(&state, &workspace).await;

        // THEN should return PathNotDirectoryError
        assert_eq!(result.unwrap_err(), FileWatcherErrorType::PathNotDirectoryError);
    }
}
```

### Acceptance Criteria

- [ ] Valid workspace creates watcher successfully
- [ ] Duplicate watcher request returns WatcherAlreadyExistsError
- [ ] Non-existent path returns PathNotExistsError
- [ ] File path (not directory) returns PathNotDirectoryError
- [ ] Permission denied returns PermissionDeniedError
- [ ] Watcher creation completes within 1000ms

---

## REQ-FILEWATCHER-002: Start Watching Workspace Directory

### Problem Statement

After watcher creation, it must begin monitoring the directory tree and forwarding events to the debouncer.

### Specification

#### REQ-FILEWATCHER-002.1: Recursive Directory Watching

```
WHEN start_watching_workspace_directory is called
  WITH valid FileWatcherServiceStruct
THEN SHALL watch source_directory_path_value recursively
  AND SHALL detect changes in all subdirectories
  AND SHALL detect changes in nested directories (depth > 10)
  AND SHALL NOT follow symbolic links (security)
```

#### REQ-FILEWATCHER-002.2: Event Types Captured

```
WHEN file system event occurs in watched directory
THEN SHALL capture the following event types:
  - Create: new file created
  - Modify: file content changed
  - Remove: file deleted
  - Rename: file renamed or moved
  AND SHALL NOT capture metadata-only changes (atime, permissions)
  AND SHALL NOT capture directory creation/deletion events
```

#### REQ-FILEWATCHER-002.3: Event Channel Forwarding

```
WHEN file event is captured by watcher
THEN SHALL create RawFileEventData with:
  - event_kind_type matching notify event kind
  - affected_paths_list containing all affected paths
  - event_timestamp_value set to now()
  AND SHALL send to event_sender_channel
  AND channel send SHALL NOT block caller for > 10ms
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_002_tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::time::{timeout, Duration};

    /// REQ-FILEWATCHER-002.1: Recursive watching detects nested changes
    #[tokio::test]
    async fn test_recursive_watching_detects_nested_changes() {
        // GIVEN a watcher on temp directory with nested structure
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("level1/level2/level3");
        std::fs::create_dir_all(&nested_dir).unwrap();
        let (watcher, mut rx) = setup_watcher_with_receiver(temp_dir.path()).await;

        // WHEN file is created in nested directory
        let nested_file = nested_dir.join("test.rs");
        std::fs::write(&nested_file, "fn test() {}").unwrap();

        // THEN should receive event for nested file
        let event = timeout(Duration::from_secs(2), rx.recv()).await.unwrap().unwrap();
        assert!(event.affected_paths_list.iter().any(|p| p == &nested_file));
    }

    /// REQ-FILEWATCHER-002.2: Modify events captured for content changes
    #[tokio::test]
    async fn test_modify_events_captured_for_content() {
        // GIVEN a watcher on temp directory with existing file
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        std::fs::write(&file_path, "original").unwrap();
        let (watcher, mut rx) = setup_watcher_with_receiver(temp_dir.path()).await;

        // WHEN file content is modified
        std::fs::write(&file_path, "modified").unwrap();

        // THEN should receive Modify event
        let event = timeout(Duration::from_secs(2), rx.recv()).await.unwrap().unwrap();
        assert_eq!(event.event_kind_type, FileEventKindType::FileModified);
    }

    /// REQ-FILEWATCHER-002.3: Events forwarded to channel quickly
    #[tokio::test]
    async fn test_event_forwarding_is_non_blocking() {
        // GIVEN a watcher with bounded channel
        let temp_dir = TempDir::new().unwrap();
        let (watcher, mut rx) = setup_watcher_with_bounded_channel(temp_dir.path(), 100).await;

        // WHEN creating many files rapidly
        let start = std::time::Instant::now();
        for i in 0..50 {
            let file = temp_dir.path().join(format!("file_{}.rs", i));
            std::fs::write(&file, format!("fn func_{}() {{}}", i)).unwrap();
        }
        let creation_time = start.elapsed();

        // THEN file creation should not be blocked by channel sends
        assert!(creation_time.as_millis() < 500, "File creation blocked: {:?}", creation_time);
    }
}
```

### Acceptance Criteria

- [ ] Recursive watching detects changes in nested directories (depth > 10)
- [ ] Create events captured for new files
- [ ] Modify events captured for content changes
- [ ] Remove events captured for deleted files
- [ ] Rename events captured for moved files
- [ ] Symbolic links are NOT followed
- [ ] Channel forwarding does not block for > 10ms

---

## REQ-FILEWATCHER-003: Stop Watching Workspace Directory

### Problem Statement

When `watch_enabled_flag_status` is toggled to `false`, or workspace is deleted, the watcher must be cleanly stopped to release resources.

### Specification

#### REQ-FILEWATCHER-003.1: Graceful Watcher Shutdown

```
WHEN stop_watching_workspace_directory is called
  WITH workspace_identifier_value that has active watcher
THEN SHALL retrieve watcher handle from SharedWorkspaceStateContainer.watchers
  AND SHALL drop watcher handle (triggers notify cleanup)
  AND SHALL remove entry from SharedWorkspaceStateContainer.watchers
  AND SHALL complete within 500ms
  AND SHALL return Ok(())
```

#### REQ-FILEWATCHER-003.2: Stop Non-Existent Watcher

```
WHEN stop_watching_workspace_directory is called
  WITH workspace_identifier_value that has no active watcher
THEN SHALL return Err(FileWatcherErrorType::WatcherNotFoundError)
  AND SHALL NOT panic or crash
```

#### REQ-FILEWATCHER-003.3: Resource Cleanup Verification

```
WHEN watcher is stopped
THEN file system watch handles SHALL be released
  AND inotify/FSEvents descriptors SHALL be freed
  AND memory usage SHALL decrease by watcher overhead (~50KB per watcher)
  AND pending events in channel SHALL be discarded
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_003_tests {
    use super::*;

    /// REQ-FILEWATCHER-003.1: Stop existing watcher succeeds
    #[tokio::test]
    async fn test_stop_existing_watcher_succeeds() {
        // GIVEN an active watcher
        let (state, workspace) = setup_workspace_with_watcher().await;
        assert!(state.watchers.read().await.contains_key(&workspace.workspace_identifier_value));

        // WHEN stopping watcher
        let result = stop_watching_workspace_directory(&state, &workspace.workspace_identifier_value).await;

        // THEN should succeed and remove from state
        assert!(result.is_ok());
        assert!(!state.watchers.read().await.contains_key(&workspace.workspace_identifier_value));
    }

    /// REQ-FILEWATCHER-003.2: Stop non-existent watcher returns error
    #[tokio::test]
    async fn test_stop_nonexistent_watcher_returns_error() {
        // GIVEN no watcher for workspace
        let state = SharedWorkspaceStateContainer::new();

        // WHEN stopping non-existent watcher
        let result = stop_watching_workspace_directory(&state, "ws_nonexistent").await;

        // THEN should return WatcherNotFoundError
        assert_eq!(result.unwrap_err(), FileWatcherErrorType::WatcherNotFoundError);
    }

    /// REQ-FILEWATCHER-003.3: Stopped watcher does not receive events
    #[tokio::test]
    async fn test_stopped_watcher_receives_no_events() {
        // GIVEN a watcher that is started then stopped
        let temp_dir = TempDir::new().unwrap();
        let (state, workspace, mut rx) = setup_workspace_with_watcher_and_receiver(temp_dir.path()).await;
        stop_watching_workspace_directory(&state, &workspace.workspace_identifier_value).await.unwrap();

        // WHEN files change after stop
        std::fs::write(temp_dir.path().join("new.rs"), "content").unwrap();

        // THEN should not receive events
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_err() || result.unwrap().is_none());
    }
}
```

### Acceptance Criteria

- [ ] Active watcher stops successfully
- [ ] Non-existent watcher returns WatcherNotFoundError
- [ ] Stopped watcher receives no further events
- [ ] Resource cleanup completes within 500ms
- [ ] No file descriptor leaks after stop

---

## REQ-FILEWATCHER-004: Automatic Cleanup on Workspace Deletion

### Problem Statement

When a workspace is deleted, its associated watcher must be automatically cleaned up to prevent resource leaks and orphaned watchers.

### Specification

#### REQ-FILEWATCHER-004.1: Watcher Cleanup on Delete

```
WHEN delete_workspace_by_identifier is called
  WITH workspace_identifier_value that has active watcher
THEN SHALL call stop_watching_workspace_directory first
  AND SHALL proceed with workspace deletion only after watcher stopped
  AND deletion SHALL succeed regardless of stop result (best-effort cleanup)
```

#### REQ-FILEWATCHER-004.2: Orphan Prevention

```
WHEN workspace is deleted via any code path
THEN SharedWorkspaceStateContainer.watchers SHALL NOT contain orphaned entry
  AND memory SHALL be freed for watcher state
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_004_tests {
    use super::*;

    /// REQ-FILEWATCHER-004.1: Workspace deletion stops watcher
    #[tokio::test]
    async fn test_workspace_deletion_stops_watcher() {
        // GIVEN workspace with active watcher
        let (state, workspace) = setup_workspace_with_watcher().await;
        let workspace_id = workspace.workspace_identifier_value.clone();

        // WHEN workspace is deleted
        delete_workspace_with_cleanup(&state, &workspace_id).await.unwrap();

        // THEN watcher should be removed
        assert!(!state.watchers.read().await.contains_key(&workspace_id));
    }

    /// REQ-FILEWATCHER-004.2: No orphaned watchers after deletion
    #[tokio::test]
    async fn test_no_orphaned_watchers_after_deletion() {
        // GIVEN multiple workspaces with watchers
        let state = SharedWorkspaceStateContainer::new();
        let workspace_ids: Vec<_> = (0..5)
            .map(|i| setup_workspace_with_watcher_id(&state, &format!("ws_{}", i)))
            .collect();

        // WHEN deleting all workspaces
        for id in &workspace_ids {
            delete_workspace_with_cleanup(&state, id).await.unwrap();
        }

        // THEN no orphaned watchers
        assert!(state.watchers.read().await.is_empty());
    }
}
```

### Acceptance Criteria

- [ ] Workspace deletion automatically stops watcher
- [ ] Watcher state removed from SharedWorkspaceStateContainer
- [ ] No orphaned watchers after deletion
- [ ] Best-effort cleanup does not block deletion

---

## REQ-FILEWATCHER-005: Handle System Resource Limits

### Problem Statement

Operating systems limit the number of file watchers (inotify on Linux, FSEvents on macOS). The service must handle these limits gracefully.

### Specification

#### REQ-FILEWATCHER-005.1: inotify Limit Detection (Linux)

```
WHEN create_watcher_for_workspace is called on Linux
  AND inotify watch limit is reached (default: 8192)
THEN SHALL return Err(FileWatcherErrorType::SystemLimitReachedError)
  AND SHALL include guidance in error message:
    "inotify watch limit reached. Increase via: sudo sysctl fs.inotify.max_user_watches=524288"
  AND SHALL NOT crash or panic
```

#### REQ-FILEWATCHER-005.2: Graceful Degradation

```
WHEN system resource limit is reached
THEN service SHALL continue operating for existing watchers
  AND new watcher creation SHALL fail with clear error
  AND existing watchers SHALL NOT be affected
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_005_tests {
    use super::*;

    /// REQ-FILEWATCHER-005.1: System limit returns clear error
    #[tokio::test]
    #[ignore] // Requires special setup to exhaust inotify limits
    async fn test_system_limit_returns_clear_error() {
        // GIVEN inotify limit is exhausted (requires mock or system manipulation)
        let state = SharedWorkspaceStateContainer::new();
        let workspace = create_workspace_with_many_subdirs(10000);

        // WHEN creating watcher exceeds limit
        let result = create_watcher_for_workspace(&state, &workspace).await;

        // THEN should return SystemLimitReachedError with guidance
        match result {
            Err(FileWatcherErrorType::SystemLimitReachedError) => {
                // Expected
            }
            other => panic!("Expected SystemLimitReachedError, got {:?}", other),
        }
    }
}
```

### Acceptance Criteria

- [ ] System limit detection returns SystemLimitReachedError
- [ ] Error message includes remediation guidance
- [ ] Existing watchers continue to function
- [ ] Service does not crash when limit reached

---

# Section 2: Path Filtering/Ignoring

## REQ-FILEWATCHER-006: Default Ignore Patterns

### Problem Statement

Build artifacts, dependencies, and version control directories generate massive event volumes that are irrelevant for code analysis. These must be filtered out.

### Specification

#### REQ-FILEWATCHER-006.1: Default Ignored Directories

```
WHEN file event occurs in watched directory
  WITH path matching any of DEFAULT_IGNORE_PATTERNS_LIST:
    - **/target/**
    - **/node_modules/**
    - **/.git/**
    - **/.hg/**
    - **/.svn/**
    - **/build/**
    - **/dist/**
    - **/__pycache__/**
    - **/vendor/**
    - **/.idea/**
    - **/.vscode/**
THEN SHALL discard event
  AND SHALL NOT forward to debouncer
  AND SHALL NOT log individual filtered events
```

#### REQ-FILEWATCHER-006.2: Default Ignored Files

```
WHEN file event occurs
  WITH path matching file patterns:
    - **/*.pyc
    - **/Cargo.lock
    - **/package-lock.json
    - **/yarn.lock
    - **/pnpm-lock.yaml
    - **/*.swp
    - **/*.swo
    - **/*~
    - **/.DS_Store
THEN SHALL discard event
  AND SHALL NOT forward to debouncer
```

#### REQ-FILEWATCHER-006.3: Source Files Pass Through

```
WHEN file event occurs
  WITH path NOT matching any ignore pattern
  AND path matches typical source extensions:
    - *.rs, *.py, *.js, *.ts, *.tsx, *.jsx
    - *.go, *.java, *.c, *.cpp, *.h, *.hpp
    - *.rb, *.php, *.cs, *.swift
THEN SHALL forward event to debouncer
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_006_tests {
    use super::*;

    /// REQ-FILEWATCHER-006.1: target/ directory events filtered
    #[tokio::test]
    async fn test_target_directory_filtered() {
        // GIVEN watcher with default ignore patterns
        let temp_dir = TempDir::new().unwrap();
        let target_dir = temp_dir.path().join("target/debug");
        std::fs::create_dir_all(&target_dir).unwrap();
        let (watcher, mut rx) = setup_watcher_with_receiver(temp_dir.path()).await;

        // WHEN file created in target/
        std::fs::write(target_dir.join("binary"), "content").unwrap();

        // THEN event should be filtered
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_err() || result.unwrap().is_none());
    }

    /// REQ-FILEWATCHER-006.1: node_modules/ directory events filtered
    #[tokio::test]
    async fn test_node_modules_filtered() {
        // GIVEN watcher with default ignore patterns
        let temp_dir = TempDir::new().unwrap();
        let node_modules = temp_dir.path().join("node_modules/lodash");
        std::fs::create_dir_all(&node_modules).unwrap();
        let (watcher, mut rx) = setup_watcher_with_receiver(temp_dir.path()).await;

        // WHEN file created in node_modules/
        std::fs::write(node_modules.join("index.js"), "module.exports = {}").unwrap();

        // THEN event should be filtered
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_err() || result.unwrap().is_none());
    }

    /// REQ-FILEWATCHER-006.2: Lock files filtered
    #[tokio::test]
    async fn test_lock_files_filtered() {
        // GIVEN watcher with default ignore patterns
        let temp_dir = TempDir::new().unwrap();
        let (watcher, mut rx) = setup_watcher_with_receiver(temp_dir.path()).await;

        // WHEN lock files modified
        std::fs::write(temp_dir.path().join("Cargo.lock"), "lock content").unwrap();
        std::fs::write(temp_dir.path().join("package-lock.json"), "{}").unwrap();

        // THEN events should be filtered
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_err() || result.unwrap().is_none());
    }

    /// REQ-FILEWATCHER-006.3: Source files pass through
    #[tokio::test]
    async fn test_source_files_pass_through() {
        // GIVEN watcher with default ignore patterns
        let temp_dir = TempDir::new().unwrap();
        let (watcher, mut rx) = setup_watcher_with_receiver(temp_dir.path()).await;

        // WHEN source file created
        std::fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();

        // THEN event should be forwarded
        let event = timeout(Duration::from_secs(2), rx.recv()).await.unwrap().unwrap();
        assert!(event.affected_paths_list.iter().any(|p| p.extension() == Some("rs".as_ref())));
    }
}
```

### Acceptance Criteria

- [ ] target/ directory events filtered
- [ ] node_modules/ directory events filtered
- [ ] .git/ directory events filtered
- [ ] Lock files (Cargo.lock, package-lock.json) filtered
- [ ] Swap files (*.swp, *.swo, *~) filtered
- [ ] Source files (.rs, .py, .js, etc.) pass through
- [ ] Filtering does not block event processing

---

## REQ-FILEWATCHER-007: Custom Ignore Patterns

### Problem Statement

Workspaces may have project-specific patterns that should be ignored (e.g., generated code, custom build directories).

### Specification

#### REQ-FILEWATCHER-007.1: Custom Pattern Addition

```
WHEN create_watcher_for_workspace is called
  WITH custom_ignore_patterns containing additional patterns
THEN SHALL merge custom patterns with DEFAULT_IGNORE_PATTERNS_LIST
  AND SHALL apply all patterns during filtering
  AND custom patterns SHALL NOT override defaults (additive only)
```

#### REQ-FILEWATCHER-007.2: Pattern Validation

```
WHEN custom ignore pattern is provided
  WITH invalid glob syntax
THEN SHALL log warning about invalid pattern
  AND SHALL skip invalid pattern (not fail entire operation)
  AND SHALL continue with valid patterns
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_007_tests {
    use super::*;

    /// REQ-FILEWATCHER-007.1: Custom patterns filter correctly
    #[tokio::test]
    async fn test_custom_patterns_filter_correctly() {
        // GIVEN watcher with custom pattern for generated code
        let temp_dir = TempDir::new().unwrap();
        let custom_patterns = vec!["**/generated/**".to_string()];
        let (watcher, mut rx) = setup_watcher_with_custom_patterns(temp_dir.path(), custom_patterns).await;

        // WHEN file created in generated/
        let generated = temp_dir.path().join("generated");
        std::fs::create_dir(&generated).unwrap();
        std::fs::write(generated.join("code.rs"), "generated").unwrap();

        // THEN event should be filtered
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_err() || result.unwrap().is_none());
    }

    /// REQ-FILEWATCHER-007.1: Custom patterns are additive to defaults
    #[tokio::test]
    async fn test_custom_patterns_additive_to_defaults() {
        // GIVEN watcher with custom pattern
        let temp_dir = TempDir::new().unwrap();
        let custom_patterns = vec!["**/custom/**".to_string()];
        let (watcher, mut rx) = setup_watcher_with_custom_patterns(temp_dir.path(), custom_patterns).await;

        // WHEN files created in target/ (default) and custom/
        std::fs::create_dir_all(temp_dir.path().join("target")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("custom")).unwrap();
        std::fs::write(temp_dir.path().join("target/bin"), "").unwrap();
        std::fs::write(temp_dir.path().join("custom/file"), "").unwrap();

        // THEN both should be filtered (additive)
        let result = timeout(Duration::from_millis(200), rx.recv()).await;
        assert!(result.is_err() || result.unwrap().is_none());
    }
}
```

### Acceptance Criteria

- [ ] Custom patterns filter matching paths
- [ ] Custom patterns are additive to defaults
- [ ] Invalid glob patterns logged but not fatal
- [ ] Valid patterns continue to work despite invalid ones

---

## REQ-FILEWATCHER-008: Pattern Matching Performance

### Problem Statement

Pattern matching must be efficient to handle high event volumes without becoming a bottleneck.

### Specification

#### REQ-FILEWATCHER-008.1: Pattern Matching Speed

```
WHEN filtering path against ignore patterns
  WITH 20 default patterns + 10 custom patterns
THEN pattern matching SHALL complete in < 100 microseconds per path
  AND SHALL use compiled glob patterns (not re-compilation per event)
```

#### REQ-FILEWATCHER-008.2: Early Exit Optimization

```
WHEN path matches an ignore pattern
THEN SHALL exit pattern matching immediately (short-circuit)
  AND SHALL NOT check remaining patterns after match found
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Pattern match time | < 100us | Time per path evaluation |
| Patterns evaluated before match | <= 1 (common case) | Early exit behavior |
| Memory for compiled patterns | < 1KB | sizeof(GlobSet) |

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_008_tests {
    use super::*;
    use std::time::Instant;

    /// REQ-FILEWATCHER-008.1: Pattern matching is fast
    #[test]
    fn test_pattern_matching_performance() {
        // GIVEN compiled ignore patterns
        let patterns = compile_default_ignore_patterns();
        let test_paths: Vec<PathBuf> = (0..1000)
            .map(|i| PathBuf::from(format!("/project/src/module_{}/file_{}.rs", i % 100, i)))
            .collect();

        // WHEN matching many paths
        let start = Instant::now();
        for path in &test_paths {
            let _ = patterns.is_match(path);
        }
        let elapsed = start.elapsed();

        // THEN should complete quickly
        let per_path = elapsed / test_paths.len() as u32;
        assert!(per_path.as_micros() < 100, "Pattern match too slow: {:?}", per_path);
    }
}
```

### Acceptance Criteria

- [ ] Pattern matching < 100us per path
- [ ] Early exit on first match
- [ ] Compiled patterns used (no re-compilation)
- [ ] Memory usage < 1KB for patterns

---

## REQ-FILEWATCHER-009: Hidden Files Filtering

### Problem Statement

Hidden files (dot-prefixed) often contain configuration that is not code and should be filtered by default.

### Specification

#### REQ-FILEWATCHER-009.1: Hidden File Handling

```
WHEN file event occurs for hidden file (starts with '.')
  AND file is in project root or recognized config location
THEN SHALL filter event by default
  EXCEPT for these recognized source files:
    - .eslintrc.* (ESLint config contains rules)
    - .prettierrc.* (Prettier config)
    - .editorconfig
    - files in .github/ (workflow definitions)
```

#### REQ-FILEWATCHER-009.2: Dotfile Exceptions

```
WHEN file event occurs for hidden file
  WITH extension matching source code (.rs, .py, .js, etc.)
THEN SHALL forward event (hidden file with code extension is likely source)
```

### Acceptance Criteria

- [ ] .git/ contents filtered
- [ ] .env files filtered (security)
- [ ] .eslintrc.js passed through
- [ ] Hidden source files (.hidden.rs) passed through

---

## REQ-FILEWATCHER-010: Symlink Handling

### Problem Statement

Symbolic links can create infinite loops or security issues if followed.

### Specification

#### REQ-FILEWATCHER-010.1: Symlink Non-Following

```
WHEN watcher encounters symbolic link in directory tree
THEN SHALL NOT follow symbolic link
  AND SHALL NOT watch target of symbolic link
  AND SHALL treat symlink as opaque file
```

#### REQ-FILEWATCHER-010.2: Symlink Event Handling

```
WHEN file event occurs for symbolic link itself
THEN SHALL forward event if link points to source file
  AND SHALL NOT forward events for link target changes
```

### Acceptance Criteria

- [ ] Symbolic links are not followed
- [ ] Infinite loop via symlink does not crash
- [ ] Symlink creation/deletion events handled safely

---

# Section 3: Debounce Behavior

## REQ-FILEWATCHER-011: Debounce Window Configuration

### Problem Statement

Rapid file saves (IDE auto-save, build system) generate many events in quick succession. These must be batched to avoid redundant reindexing.

### Specification

#### REQ-FILEWATCHER-011.1: 500ms Default Debounce

```
WHEN process_debounced_file_events is configured
THEN debounce window SHALL default to 500ms
  AND debounce timer SHALL reset on each new event
  AND timer SHALL fire only when no events received for full window
```

#### REQ-FILEWATCHER-011.2: Configurable Debounce Duration

```
WHEN FileWatcherServiceStruct is created
  WITH custom debounce_duration_milliseconds value
THEN SHALL use custom value instead of default
  AND custom value SHALL be clamped to range [100ms, 5000ms]
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_011_tests {
    use super::*;
    use tokio::time::{sleep, Duration, Instant};

    /// REQ-FILEWATCHER-011.1: Debounce window of 500ms
    #[tokio::test]
    async fn test_debounce_window_500ms() {
        // GIVEN debouncer with default config
        let (tx, mut debounced_rx) = create_debouncer_channel();

        // WHEN sending event and waiting
        let start = Instant::now();
        tx.send(create_raw_event()).await.unwrap();

        // THEN debounced event arrives after ~500ms
        let event = debounced_rx.recv().await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(500));
        assert!(elapsed < Duration::from_millis(600));
    }

    /// REQ-FILEWATCHER-011.1: Timer resets on new event
    #[tokio::test]
    async fn test_debounce_timer_resets_on_event() {
        // GIVEN debouncer with default config
        let (tx, mut debounced_rx) = create_debouncer_channel();

        // WHEN events sent every 200ms for 1 second
        let start = Instant::now();
        for _ in 0..5 {
            tx.send(create_raw_event()).await.unwrap();
            sleep(Duration::from_millis(200)).await;
        }

        // THEN debounced event arrives 500ms after LAST event
        let event = debounced_rx.recv().await.unwrap();
        let elapsed = start.elapsed();
        // 5 events * 200ms = 1000ms, then +500ms debounce = ~1500ms
        assert!(elapsed >= Duration::from_millis(1400));
        assert!(elapsed < Duration::from_millis(1600));
    }

    /// REQ-FILEWATCHER-011.2: Custom debounce duration respected
    #[tokio::test]
    async fn test_custom_debounce_duration() {
        // GIVEN debouncer with 200ms custom duration
        let (tx, mut debounced_rx) = create_debouncer_with_duration(200);

        // WHEN sending event
        let start = Instant::now();
        tx.send(create_raw_event()).await.unwrap();

        // THEN debounced event arrives after ~200ms
        let event = debounced_rx.recv().await.unwrap();
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(200));
        assert!(elapsed < Duration::from_millis(300));
    }
}
```

### Acceptance Criteria

- [ ] Default debounce window is 500ms
- [ ] Timer resets when new event arrives
- [ ] Debounced event contains all batched events
- [ ] Custom duration is respected
- [ ] Duration clamped to [100ms, 5000ms]

---

## REQ-FILEWATCHER-012: Event Aggregation

### Problem Statement

Multiple events for the same file should be deduplicated, and events for different files should be aggregated into a single batch.

### Specification

#### REQ-FILEWATCHER-012.1: Path Deduplication

```
WHEN multiple events occur for same file path within debounce window
THEN DebouncedFileChangeEvent.changed_file_paths SHALL contain path only once
  AND raw_event_count_value SHALL reflect total events before deduplication
```

#### REQ-FILEWATCHER-012.2: Multi-File Aggregation

```
WHEN events occur for different files within debounce window
THEN DebouncedFileChangeEvent.changed_file_paths SHALL contain all unique paths
  AND paths SHALL be sorted alphabetically for determinism
```

#### REQ-FILEWATCHER-012.3: Event Kind Merging

```
WHEN same file has Create then Modify events within window
THEN SHALL treat as single Create (final state is what matters)
WHEN same file has Modify then Delete events within window
THEN SHALL treat as single Delete
WHEN same file has Create then Delete events within window
THEN SHALL omit file from batch (net zero change)
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_012_tests {
    use super::*;

    /// REQ-FILEWATCHER-012.1: Duplicate paths deduplicated
    #[tokio::test]
    async fn test_duplicate_paths_deduplicated() {
        // GIVEN debouncer
        let (tx, mut debounced_rx) = create_debouncer_channel();
        let path = PathBuf::from("/project/src/lib.rs");

        // WHEN same path modified 5 times
        for _ in 0..5 {
            tx.send(create_event_for_path(&path, FileEventKindType::FileModified)).await.unwrap();
        }

        // THEN debounced event has path only once
        let event = debounced_rx.recv().await.unwrap();
        assert_eq!(event.changed_file_paths.len(), 1);
        assert_eq!(event.raw_event_count_value, 5);
    }

    /// REQ-FILEWATCHER-012.2: Multiple files aggregated
    #[tokio::test]
    async fn test_multiple_files_aggregated() {
        // GIVEN debouncer
        let (tx, mut debounced_rx) = create_debouncer_channel();

        // WHEN different files modified
        tx.send(create_event_for_path("/project/a.rs", FileEventKindType::FileModified)).await.unwrap();
        tx.send(create_event_for_path("/project/b.rs", FileEventKindType::FileModified)).await.unwrap();
        tx.send(create_event_for_path("/project/c.rs", FileEventKindType::FileModified)).await.unwrap();

        // THEN all paths in debounced event, sorted
        let event = debounced_rx.recv().await.unwrap();
        assert_eq!(event.changed_file_paths.len(), 3);
        assert_eq!(event.changed_file_paths[0], PathBuf::from("/project/a.rs"));
        assert_eq!(event.changed_file_paths[1], PathBuf::from("/project/b.rs"));
        assert_eq!(event.changed_file_paths[2], PathBuf::from("/project/c.rs"));
    }

    /// REQ-FILEWATCHER-012.3: Create+Delete cancels out
    #[tokio::test]
    async fn test_create_delete_cancels_out() {
        // GIVEN debouncer
        let (tx, mut debounced_rx) = create_debouncer_channel();
        let path = PathBuf::from("/project/temp.rs");

        // WHEN file created then deleted in same window
        tx.send(create_event_for_path(&path, FileEventKindType::FileCreated)).await.unwrap();
        tx.send(create_event_for_path(&path, FileEventKindType::FileDeleted)).await.unwrap();

        // THEN path should not be in debounced event
        let event = debounced_rx.recv().await.unwrap();
        assert!(!event.changed_file_paths.contains(&path));
    }
}
```

### Acceptance Criteria

- [ ] Duplicate paths are deduplicated
- [ ] Multiple files aggregated into single batch
- [ ] Paths sorted alphabetically
- [ ] raw_event_count reflects pre-deduplication count
- [ ] Create+Delete for same file cancels out

---

## REQ-FILEWATCHER-013: Maximum Buffer Size

### Problem Statement

If a user performs a large operation (e.g., git checkout of many files), the event buffer should not grow unbounded.

### Specification

#### REQ-FILEWATCHER-013.1: Buffer Size Limit

```
WHEN events accumulate in debounce buffer
  AND buffer size exceeds MAX_BUFFERED_EVENTS_COUNT (1000)
THEN SHALL force-flush buffer immediately
  AND SHALL NOT wait for debounce window
  AND SHALL restart debounce timer for subsequent events
```

#### REQ-FILEWATCHER-013.2: Memory Bound

```
WHEN accumulating events
THEN memory usage for buffered events SHALL NOT exceed 10MB
  AND each event SHALL use approximately 500 bytes
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_013_tests {
    use super::*;

    /// REQ-FILEWATCHER-013.1: Buffer force-flushes at limit
    #[tokio::test]
    async fn test_buffer_force_flush_at_limit() {
        // GIVEN debouncer with long window (5s) but low buffer limit
        let (tx, mut debounced_rx) = create_debouncer_with_config(5000, 100);

        // WHEN sending more events than buffer limit
        let start = Instant::now();
        for i in 0..150 {
            tx.send(create_event_for_path(&format!("/project/file_{}.rs", i), FileEventKindType::FileCreated)).await.unwrap();
        }

        // THEN should receive first batch almost immediately (not waiting 5s)
        let event = timeout(Duration::from_millis(200), debounced_rx.recv()).await.unwrap().unwrap();
        assert_eq!(event.changed_file_paths.len(), 100);
    }
}
```

### Acceptance Criteria

- [ ] Buffer force-flushes at MAX_BUFFERED_EVENTS_COUNT
- [ ] Memory does not exceed 10MB for buffered events
- [ ] Subsequent events start new debounce cycle

---

## REQ-FILEWATCHER-014: Debounce Across Workspaces

### Problem Statement

Each workspace should have independent debouncing to prevent cross-workspace interference.

### Specification

#### REQ-FILEWATCHER-014.1: Workspace Isolation

```
WHEN events occur in workspace A and workspace B simultaneously
THEN workspace A events SHALL debounce independently
  AND workspace B events SHALL debounce independently
  AND flush of A SHALL NOT affect B's timer
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_014_tests {
    use super::*;

    /// REQ-FILEWATCHER-014.1: Workspaces debounce independently
    #[tokio::test]
    async fn test_workspaces_debounce_independently() {
        // GIVEN two workspace watchers
        let (state, ws_a, ws_b) = setup_two_workspaces_with_watchers().await;
        let mut rx_a = get_debounced_receiver(&state, &ws_a).await;
        let mut rx_b = get_debounced_receiver(&state, &ws_b).await;

        // WHEN event in A, wait 300ms, event in B
        trigger_file_event(&ws_a).await;
        sleep(Duration::from_millis(300)).await;
        trigger_file_event(&ws_b).await;

        // THEN A should flush at T+500ms, B at T+800ms
        let event_a = timeout(Duration::from_millis(250), rx_a.recv()).await.unwrap().unwrap();
        // B should not be ready yet
        let result_b = timeout(Duration::from_millis(50), rx_b.recv()).await;
        assert!(result_b.is_err());
        // Now B should arrive
        let event_b = timeout(Duration::from_millis(500), rx_b.recv()).await.unwrap().unwrap();
    }
}
```

### Acceptance Criteria

- [ ] Each workspace has independent debounce timer
- [ ] Workspace A flush does not affect workspace B
- [ ] Events correctly routed to workspace-specific debouncer

---

## REQ-FILEWATCHER-015: Debounce Edge Cases

### Problem Statement

Edge cases like rapid start/stop or empty batches must be handled gracefully.

### Specification

#### REQ-FILEWATCHER-015.1: Empty Batch Prevention

```
WHEN debounce window closes
  AND all events were filtered (e.g., all in target/)
THEN SHALL NOT emit DebouncedFileChangeEvent
  AND SHALL NOT trigger reindex
```

#### REQ-FILEWATCHER-015.2: Watcher Stop During Debounce

```
WHEN stop_watching_workspace_directory is called
  AND events are pending in debounce buffer
THEN SHALL cancel pending debounce
  AND SHALL discard buffered events
  AND SHALL NOT emit partial batch
```

### Acceptance Criteria

- [ ] Empty batches do not trigger reindex
- [ ] Stopping watcher cancels pending debounce
- [ ] No events emitted after watcher stopped

---

# Section 4: Reindex Triggering

## REQ-FILEWATCHER-016: Incremental Reindex Trigger

### Problem Statement

When debounced events are ready, the system must trigger an incremental reindex of only the changed files.

### Specification

#### REQ-FILEWATCHER-016.1: Trigger on Debounce

```
WHEN DebouncedFileChangeEvent is emitted
  WITH changed_file_paths containing N files
THEN SHALL call trigger_incremental_reindex_update
  AND SHALL pass only changed_file_paths (not full codebase)
  AND SHALL update live.db (not base.db)
```

#### REQ-FILEWATCHER-016.2: Reindex Started Notification

```
WHEN trigger_incremental_reindex_update begins
THEN SHALL broadcast WebSocketServerOutboundMessageType::DiffAnalysisStartedNotification
  WITH:
    - workspace_id: workspace_identifier_value
    - files_changed: changed_file_paths.len()
    - triggered_by: "file_watcher"
    - timestamp: now()
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_016_tests {
    use super::*;

    /// REQ-FILEWATCHER-016.1: Debounced event triggers reindex
    #[tokio::test]
    async fn test_debounced_event_triggers_reindex() {
        // GIVEN workspace with watcher and mock reindexer
        let (state, workspace, reindex_spy) = setup_workspace_with_mock_reindexer().await;

        // WHEN file changes trigger debounced event
        let file_path = workspace.source_directory_path_value.join("new.rs");
        std::fs::write(&file_path, "fn new() {}").unwrap();
        wait_for_debounce().await;

        // THEN reindexer should be called with changed file
        let calls = reindex_spy.get_calls().await;
        assert_eq!(calls.len(), 1);
        assert!(calls[0].changed_files.contains(&file_path));
    }

    /// REQ-FILEWATCHER-016.2: DiffStarted notification broadcast
    #[tokio::test]
    async fn test_diff_started_notification_broadcast() {
        // GIVEN subscribed WebSocket client
        let (state, workspace, mut ws_rx) = setup_workspace_with_ws_client().await;

        // WHEN file changes
        let file_path = workspace.source_directory_path_value.join("changed.rs");
        std::fs::write(&file_path, "fn changed() {}").unwrap();
        wait_for_debounce().await;

        // THEN should receive diff_started
        let msg = ws_rx.recv().await.unwrap();
        assert!(matches!(msg, WebSocketServerOutboundMessageType::DiffAnalysisStartedNotification { .. }));
    }
}
```

### Acceptance Criteria

- [ ] Debounced event triggers incremental reindex
- [ ] Only changed files passed to reindexer
- [ ] live.db is updated (not base.db)
- [ ] DiffAnalysisStartedNotification broadcast to subscribers

---

## REQ-FILEWATCHER-017: Reindex Completion Handling

### Problem Statement

After reindex completes, the system must compute diff and notify subscribers.

### Specification

#### REQ-FILEWATCHER-017.1: Reindex Completion Flow

```
WHEN trigger_incremental_reindex_update completes successfully
THEN SHALL compute diff between base.db and updated live.db
  AND SHALL call broadcast_diff_to_subscribers with diff result
  AND SHALL broadcast DiffAnalysisCompletedNotification
```

#### REQ-FILEWATCHER-017.2: Completion Notification Content

```
WHEN DiffAnalysisCompletedNotification is broadcast
THEN SHALL contain:
    - workspace_id: workspace_identifier_value
    - summary: DiffSummaryDataPayloadStruct
    - blast_radius_count: affected_entities_count
    - duration_ms: time_from_started_to_completed
    - timestamp: now()
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_017_tests {
    use super::*;

    /// REQ-FILEWATCHER-017.1: Reindex completion triggers diff
    #[tokio::test]
    async fn test_reindex_completion_triggers_diff() {
        // GIVEN workspace with existing base.db
        let (state, workspace) = setup_workspace_with_base_snapshot().await;

        // WHEN file change triggers reindex and completes
        let file_path = workspace.source_directory_path_value.join("added.rs");
        std::fs::write(&file_path, "fn added() {}").unwrap();
        wait_for_diff_completion().await;

        // THEN diff should reflect the addition
        let diff = get_last_computed_diff(&state, &workspace.workspace_identifier_value).await;
        assert!(diff.summary.added_entity_count >= 1);
    }

    /// REQ-FILEWATCHER-017.2: Completion notification has correct content
    #[tokio::test]
    async fn test_completion_notification_content() {
        // GIVEN subscribed WebSocket client
        let (state, workspace, mut ws_rx) = setup_workspace_with_ws_client().await;

        // WHEN file change completes diff cycle
        std::fs::write(workspace.source_directory_path_value.join("test.rs"), "fn test() {}").unwrap();
        drain_until_diff_completed(&mut ws_rx).await;

        // THEN completion notification should have all fields
        let completed = ws_rx.recv().await.unwrap();
        if let WebSocketServerOutboundMessageType::DiffAnalysisCompletedNotification {
            workspace_id, summary, blast_radius_count, duration_ms, timestamp
        } = completed {
            assert_eq!(workspace_id, workspace.workspace_identifier_value);
            assert!(duration_ms > 0);
            assert!(summary.total_after_count >= summary.total_before_count);
        } else {
            panic!("Expected DiffAnalysisCompletedNotification");
        }
    }
}
```

### Acceptance Criteria

- [ ] Reindex completion triggers diff computation
- [ ] Diff computed between base.db and live.db
- [ ] DiffAnalysisCompletedNotification broadcast
- [ ] Notification contains complete summary
- [ ] duration_ms accurately reflects elapsed time

---

## REQ-FILEWATCHER-018: Concurrent Reindex Prevention

### Problem Statement

If files change rapidly, multiple reindex operations could overlap, causing inconsistent results.

### Specification

#### REQ-FILEWATCHER-018.1: Serialized Reindex

```
WHEN trigger_incremental_reindex_update is in progress
  AND another DebouncedFileChangeEvent arrives
THEN SHALL queue the new event
  AND SHALL NOT start concurrent reindex
  AND SHALL process queued event after current completes
```

#### REQ-FILEWATCHER-018.2: Queue Merging

```
WHEN multiple DebouncedFileChangeEvents are queued
THEN SHALL merge their changed_file_paths
  AND SHALL process as single reindex operation
  AND SHALL NOT lose any changed files
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_018_tests {
    use super::*;

    /// REQ-FILEWATCHER-018.1: Concurrent reindex prevented
    #[tokio::test]
    async fn test_concurrent_reindex_prevented() {
        // GIVEN workspace with slow mock reindexer
        let (state, workspace, reindex_spy) = setup_workspace_with_slow_reindexer(Duration::from_secs(1)).await;

        // WHEN triggering two rapid file changes
        std::fs::write(workspace.source_directory_path_value.join("a.rs"), "").unwrap();
        sleep(Duration::from_millis(600)).await;
        std::fs::write(workspace.source_directory_path_value.join("b.rs"), "").unwrap();

        // Wait for both to complete
        sleep(Duration::from_secs(3)).await;

        // THEN reindexer should be called sequentially (not overlapping)
        let calls = reindex_spy.get_calls().await;
        assert!(calls.len() >= 2);
        // Check no overlapping (second call started after first completed)
        assert!(calls[1].start_time >= calls[0].end_time);
    }
}
```

### Acceptance Criteria

- [ ] No concurrent reindex operations
- [ ] Queued events processed sequentially
- [ ] Queued events merged if possible
- [ ] No changed files lost in queue

---

## REQ-FILEWATCHER-019: Reindex Error Handling

### Problem Statement

Reindex may fail due to parse errors, database errors, or other issues. These must be handled gracefully.

### Specification

#### REQ-FILEWATCHER-019.1: Parse Error Handling

```
WHEN trigger_incremental_reindex_update fails due to parse error
THEN SHALL broadcast WebSocketServerOutboundMessageType::ErrorOccurredNotification
  WITH:
    - code: "REINDEX_PARSE_ERROR"
    - message: detailed error with file path
    - timestamp: now()
  AND SHALL NOT crash watcher
  AND watcher SHALL continue for subsequent changes
```

#### REQ-FILEWATCHER-019.2: Database Error Handling

```
WHEN trigger_incremental_reindex_update fails due to database error
THEN SHALL broadcast ErrorOccurredNotification
  WITH code: "REINDEX_DATABASE_ERROR"
  AND SHALL attempt recovery on next change
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_019_tests {
    use super::*;

    /// REQ-FILEWATCHER-019.1: Parse error broadcasts notification
    #[tokio::test]
    async fn test_parse_error_broadcasts_notification() {
        // GIVEN workspace with watcher and WS client
        let (state, workspace, mut ws_rx) = setup_workspace_with_ws_client().await;

        // WHEN file with syntax error is created
        std::fs::write(
            workspace.source_directory_path_value.join("bad.rs"),
            "fn incomplete("  // Invalid syntax
        ).unwrap();
        wait_for_diff_cycle().await;

        // THEN should receive error notification
        let msg = drain_until_error(&mut ws_rx).await;
        if let WebSocketServerOutboundMessageType::ErrorOccurredNotification { code, message, .. } = msg {
            assert_eq!(code, "REINDEX_PARSE_ERROR");
            assert!(message.contains("bad.rs") || message.contains("parse"));
        } else {
            panic!("Expected ErrorOccurredNotification");
        }
    }

    /// REQ-FILEWATCHER-019.1: Watcher continues after parse error
    #[tokio::test]
    async fn test_watcher_continues_after_error() {
        // GIVEN workspace that had a parse error
        let (state, workspace, mut ws_rx) = setup_workspace_with_ws_client().await;
        std::fs::write(workspace.source_directory_path_value.join("bad.rs"), "invalid").unwrap();
        drain_until_error(&mut ws_rx).await;

        // WHEN valid file is created
        std::fs::write(workspace.source_directory_path_value.join("good.rs"), "fn good() {}").unwrap();
        wait_for_diff_cycle().await;

        // THEN should receive diff events (watcher still working)
        let msg = ws_rx.recv().await.unwrap();
        assert!(!matches!(msg, WebSocketServerOutboundMessageType::ErrorOccurredNotification { .. }));
    }
}
```

### Acceptance Criteria

- [ ] Parse errors broadcast ErrorOccurredNotification
- [ ] Database errors broadcast ErrorOccurredNotification
- [ ] Watcher continues after errors
- [ ] Error messages contain helpful context

---

## REQ-FILEWATCHER-020: Update Timestamp Tracking

### Problem Statement

The workspace's `last_indexed_timestamp_option` must be updated after successful reindex.

### Specification

#### REQ-FILEWATCHER-020.1: Timestamp Update on Success

```
WHEN trigger_incremental_reindex_update completes successfully
THEN SHALL update workspace.last_indexed_timestamp_option to now()
  AND SHALL persist updated metadata to disk
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_020_tests {
    use super::*;

    /// REQ-FILEWATCHER-020.1: Timestamp updated after reindex
    #[tokio::test]
    async fn test_timestamp_updated_after_reindex() {
        // GIVEN workspace with null last_indexed_timestamp
        let (state, workspace) = setup_workspace_without_indexed_timestamp().await;
        assert!(workspace.last_indexed_timestamp_option.is_none());

        // WHEN file change triggers successful reindex
        std::fs::write(workspace.source_directory_path_value.join("new.rs"), "fn new() {}").unwrap();
        wait_for_diff_completion().await;

        // THEN timestamp should be updated
        let updated = load_workspace_metadata(&workspace.workspace_identifier_value).await;
        assert!(updated.last_indexed_timestamp_option.is_some());
    }
}
```

### Acceptance Criteria

- [ ] last_indexed_timestamp updated on successful reindex
- [ ] Metadata persisted to disk
- [ ] Timestamp not updated on failed reindex

---

# Section 5: Diff Computation and Broadcast

## REQ-FILEWATCHER-021: Diff Computation Pipeline

### Problem Statement

After reindex, the system must compute a diff between base.db and live.db and prepare it for broadcast.

### Specification

#### REQ-FILEWATCHER-021.1: Full Diff Computation

```
WHEN compute_diff_after_reindex is called
  WITH base_database_path_value and live_database_path_value
THEN SHALL use EntityDifferImpl to compute entity differences
  AND SHALL use BlastRadiusCalculatorImpl for affected entities
  AND SHALL produce DiffResultDataPayload
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_021_tests {
    use super::*;

    /// REQ-FILEWATCHER-021.1: Diff computed correctly
    #[tokio::test]
    async fn test_diff_computed_correctly() {
        // GIVEN workspace with base.db snapshot
        let (state, workspace) = setup_workspace_with_base_snapshot().await;
        // Base has: fn original() {}

        // WHEN new function added
        std::fs::write(
            workspace.source_directory_path_value.join("lib.rs"),
            "fn original() {}\nfn added() {}"
        ).unwrap();
        wait_for_diff_completion().await;

        // THEN diff should show addition
        let diff = get_last_diff(&state, &workspace.workspace_identifier_value).await;
        assert_eq!(diff.summary.added_entity_count, 1);
        assert!(diff.entity_changes.iter().any(|c|
            c.stable_identity.contains("added") &&
            c.change_type == EntityChangeTypeClassification::AddedToCodebase
        ));
    }
}
```

### Acceptance Criteria

- [ ] Diff computed between base.db and live.db
- [ ] Entity differences detected correctly
- [ ] Blast radius calculated
- [ ] DiffResultDataPayload produced

---

## REQ-FILEWATCHER-022: Broadcast to Subscribers

### Problem Statement

Computed diff results must be broadcast to all WebSocket clients subscribed to the workspace.

### Specification

#### REQ-FILEWATCHER-022.1: Entity Event Streaming

```
WHEN broadcast_diff_to_subscribers is called
  WITH DiffResultDataPayload containing entity changes
THEN SHALL broadcast events in order:
  1. DiffAnalysisStartedNotification (if not already sent)
  2. EntityRemovedEventNotification for each removed entity (sorted by key)
  3. EntityAddedEventNotification for each added entity (sorted by key)
  4. EntityModifiedEventNotification for each modified entity (sorted by key)
  5. EdgeRemovedEventNotification for each removed edge
  6. EdgeAddedEventNotification for each added edge
  7. DiffAnalysisCompletedNotification
```

#### REQ-FILEWATCHER-022.2: Multi-Client Broadcast

```
WHEN workspace has N subscribed clients
THEN all N clients SHALL receive identical events
  AND event delivery SHALL be within 100ms across all clients
  AND failure to send to one client SHALL NOT affect others
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_022_tests {
    use super::*;

    /// REQ-FILEWATCHER-022.1: Events broadcast in correct order
    #[tokio::test]
    async fn test_events_broadcast_in_order() {
        // GIVEN subscribed WebSocket client
        let (state, workspace, mut ws_rx) = setup_workspace_with_ws_client().await;

        // WHEN changes include add/remove/modify
        make_complex_changes(&workspace.source_directory_path_value).await;

        // THEN events should arrive in order
        let events = collect_all_diff_events(&mut ws_rx).await;
        let event_types: Vec<_> = events.iter().map(|e| event_type_name(e)).collect();

        // Verify ordering: started, removed, added, modified, edge_removed, edge_added, completed
        let started_idx = event_types.iter().position(|t| t == "diff_started").unwrap();
        let completed_idx = event_types.iter().position(|t| t == "diff_completed").unwrap();
        assert!(started_idx < completed_idx);

        // All entity events between started and completed
        for (idx, t) in event_types.iter().enumerate() {
            if t.starts_with("entity_") || t.starts_with("edge_") {
                assert!(idx > started_idx && idx < completed_idx);
            }
        }
    }

    /// REQ-FILEWATCHER-022.2: Multi-client receives identical events
    #[tokio::test]
    async fn test_multi_client_identical_events() {
        // GIVEN multiple subscribed clients
        let (state, workspace, mut rx1, mut rx2, mut rx3) = setup_workspace_with_three_clients().await;

        // WHEN change triggers diff
        std::fs::write(workspace.source_directory_path_value.join("new.rs"), "fn new() {}").unwrap();

        // THEN all clients receive same events
        let events1 = collect_all_diff_events(&mut rx1).await;
        let events2 = collect_all_diff_events(&mut rx2).await;
        let events3 = collect_all_diff_events(&mut rx3).await;

        assert_eq!(events1.len(), events2.len());
        assert_eq!(events2.len(), events3.len());
        for i in 0..events1.len() {
            assert_eq!(event_type_name(&events1[i]), event_type_name(&events2[i]));
            assert_eq!(event_type_name(&events2[i]), event_type_name(&events3[i]));
        }
    }
}
```

### Acceptance Criteria

- [ ] Events broadcast in correct order
- [ ] All subscribed clients receive events
- [ ] Events identical across clients
- [ ] Delivery within 100ms across clients
- [ ] Client failure does not affect others

---

## REQ-FILEWATCHER-023: Empty Diff Handling

### Problem Statement

If reindex produces no changes (e.g., only whitespace changed), the system should handle this gracefully.

### Specification

#### REQ-FILEWATCHER-023.1: Empty Diff Notification

```
WHEN diff computation produces zero changes
  WITH summary showing added=0, removed=0, modified=0
THEN SHALL still broadcast DiffAnalysisStartedNotification
  AND SHALL broadcast DiffAnalysisCompletedNotification with zero counts
  AND SHALL NOT broadcast entity/edge events
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_023_tests {
    use super::*;

    /// REQ-FILEWATCHER-023.1: Empty diff still notifies
    #[tokio::test]
    async fn test_empty_diff_still_notifies() {
        // GIVEN workspace with base identical to live
        let (state, workspace, mut ws_rx) = setup_workspace_with_identical_snapshots().await;

        // WHEN whitespace-only change
        let content = std::fs::read_to_string(workspace.source_directory_path_value.join("lib.rs")).unwrap();
        std::fs::write(
            workspace.source_directory_path_value.join("lib.rs"),
            format!("{}\n", content)  // Just add newline
        ).unwrap();

        // Wait for cycle
        let events = collect_all_diff_events(&mut ws_rx).await;

        // THEN should receive started and completed (but no entity events)
        let has_started = events.iter().any(|e| matches!(e, WebSocketServerOutboundMessageType::DiffAnalysisStartedNotification { .. }));
        let has_completed = events.iter().any(|e| matches!(e, WebSocketServerOutboundMessageType::DiffAnalysisCompletedNotification { .. }));
        let entity_events: Vec<_> = events.iter().filter(|e|
            matches!(e, WebSocketServerOutboundMessageType::EntityAddedEventNotification { .. }) ||
            matches!(e, WebSocketServerOutboundMessageType::EntityRemovedEventNotification { .. })
        ).collect();

        assert!(has_started);
        assert!(has_completed);
        assert!(entity_events.is_empty());
    }
}
```

### Acceptance Criteria

- [ ] Empty diff broadcasts started notification
- [ ] Empty diff broadcasts completed notification with zeros
- [ ] No entity/edge events for empty diff

---

## REQ-FILEWATCHER-024: Large Diff Batching

### Problem Statement

Very large diffs (thousands of entities) should be batched to prevent overwhelming clients.

### Specification

#### REQ-FILEWATCHER-024.1: Event Batching Threshold

```
WHEN diff contains > 1000 entity changes
THEN SHALL batch events into groups of 100
  AND SHALL include batch metadata:
    - batch_number: 1..N
    - total_batches: N
    - events_in_batch: count
  AND SHALL delay 10ms between batches
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Small diff (< 100 entities) | < 100ms broadcast | Start to completed |
| Medium diff (100-1000 entities) | < 1000ms broadcast | Start to completed |
| Large diff (> 1000 entities) | < 10s broadcast | Start to completed |

### Acceptance Criteria

- [ ] Large diffs batched into groups of 100
- [ ] Batch metadata included in events
- [ ] 10ms delay between batches
- [ ] All clients receive all batches

---

## REQ-FILEWATCHER-025: Visualization Data Preparation

### Problem Statement

In addition to raw diff data, the service must prepare visualization-ready data for the frontend.

### Specification

#### REQ-FILEWATCHER-025.1: Visualization Payload

```
WHEN broadcast_diff_to_subscribers completes entity events
THEN DiffAnalysisCompletedNotification SHALL include:
    - summary: DiffSummaryDataPayloadStruct
    - blast_radius_count: total affected entities
  AND frontend CAN request full VisualizationGraphDataPayload via separate endpoint
```

### Acceptance Criteria

- [ ] Summary included in completed notification
- [ ] blast_radius_count reflects affected entities
- [ ] Visualization data available for frontend

---

# Section 6: Error Handling

## REQ-FILEWATCHER-026: Graceful Error Recovery

### Problem Statement

The file watcher service must be resilient to various error conditions without requiring manual intervention.

### Specification

#### REQ-FILEWATCHER-026.1: Transient Error Recovery

```
WHEN file event processing fails with transient error
  (e.g., file locked, temporary network issue)
THEN SHALL retry up to 3 times with exponential backoff (100ms, 200ms, 400ms)
  AND SHALL proceed after successful retry
  AND SHALL log warning for each retry
  AND SHALL broadcast error only after all retries exhausted
```

#### REQ-FILEWATCHER-026.2: Unrecoverable Error Handling

```
WHEN unrecoverable error occurs
  (e.g., database corruption, workspace deleted)
THEN SHALL stop watcher for affected workspace
  AND SHALL broadcast ErrorOccurredNotification
  AND SHALL update workspace.watch_enabled_flag_status to false
  AND SHALL NOT affect other workspaces
```

### Verification Test Template

```rust
#[cfg(test)]
mod req_filewatcher_026_tests {
    use super::*;

    /// REQ-FILEWATCHER-026.1: Transient errors retried
    #[tokio::test]
    async fn test_transient_errors_retried() {
        // GIVEN workspace with mock that fails twice then succeeds
        let (state, workspace, reindex_spy) = setup_workspace_with_flaky_reindexer(2).await;

        // WHEN file change triggers reindex
        std::fs::write(workspace.source_directory_path_value.join("test.rs"), "fn test() {}").unwrap();
        wait_for_diff_completion().await;

        // THEN should succeed after retries
        let calls = reindex_spy.get_calls().await;
        assert_eq!(calls.len(), 3);  // 2 failures + 1 success
    }

    /// REQ-FILEWATCHER-026.2: Unrecoverable error stops watcher
    #[tokio::test]
    async fn test_unrecoverable_error_stops_watcher() {
        // GIVEN workspace with watcher
        let (state, workspace, mut ws_rx) = setup_workspace_with_ws_client().await;

        // WHEN database is corrupted (simulated)
        corrupt_workspace_database(&workspace.workspace_identifier_value).await;
        std::fs::write(workspace.source_directory_path_value.join("test.rs"), "").unwrap();
        wait_for_error(&mut ws_rx).await;

        // THEN watcher should be stopped
        assert!(!state.watchers.read().await.contains_key(&workspace.workspace_identifier_value));
        let updated = load_workspace_metadata(&workspace.workspace_identifier_value).await;
        assert!(!updated.watch_enabled_flag_status);
    }
}
```

### Acceptance Criteria

- [ ] Transient errors retried 3 times
- [ ] Exponential backoff between retries
- [ ] Unrecoverable errors stop watcher
- [ ] Error notification broadcast
- [ ] Other workspaces unaffected

---

## REQ-FILEWATCHER-027: Channel Error Handling

### Problem Statement

The mpsc channels between components may error (closed, full). These must be handled without crashing.

### Specification

#### REQ-FILEWATCHER-027.1: Closed Channel Detection

```
WHEN watcher attempts to send event
  AND receiver channel is closed (subscriber disconnected)
THEN SHALL detect channel closed error
  AND SHALL log warning
  AND SHALL NOT crash or panic
  AND SHALL continue processing other events
```

#### REQ-FILEWATCHER-027.2: Full Channel Handling

```
WHEN event buffer channel is full
  AND new event arrives
THEN SHALL drop oldest event (not block)
  AND SHALL log warning about dropped events
  AND SHALL NOT block watcher thread
```

### Acceptance Criteria

- [ ] Closed channel detected without panic
- [ ] Full channel drops oldest events
- [ ] Watcher continues after channel errors
- [ ] Warnings logged for dropped events

---

## REQ-FILEWATCHER-028: Timeout Handling

### Problem Statement

Long-running operations must have timeouts to prevent indefinite hangs.

### Specification

#### REQ-FILEWATCHER-028.1: Reindex Timeout

```
WHEN trigger_incremental_reindex_update runs
  AND operation exceeds 30000ms (30 seconds)
THEN SHALL cancel reindex operation
  AND SHALL broadcast ErrorOccurredNotification:
    {
      "code": "REINDEX_TIMEOUT",
      "message": "Reindex operation timed out after 30 seconds"
    }
  AND SHALL allow next reindex attempt
```

#### REQ-FILEWATCHER-028.2: Broadcast Timeout

```
WHEN broadcasting event to WebSocket client
  AND send exceeds 5000ms (5 seconds)
THEN SHALL timeout the send
  AND SHALL remove unresponsive client
  AND SHALL continue to other clients
```

### Acceptance Criteria

- [ ] Reindex timeout at 30 seconds
- [ ] Timeout error broadcast
- [ ] Broadcast timeout at 5 seconds
- [ ] Unresponsive clients removed

---

## REQ-FILEWATCHER-029: Logging and Observability

### Problem Statement

Operations must be logged at appropriate levels for debugging and monitoring.

### Specification

#### REQ-FILEWATCHER-029.1: Log Levels

```
WHEN file watcher operations occur
THEN SHALL log at appropriate levels:
  - INFO: Watcher started, watcher stopped, reindex completed
  - DEBUG: File events received, debounce triggered
  - WARN: Filtered events, retry attempts, slow operations
  - ERROR: Unrecoverable errors, channel failures
```

#### REQ-FILEWATCHER-029.2: Structured Logging

```
WHEN logging file watcher events
THEN logs SHALL include structured fields:
  - workspace_id: workspace identifier
  - event_count: number of events (for batches)
  - duration_ms: operation duration
  - path: affected file path (when relevant)
```

### Acceptance Criteria

- [ ] Appropriate log levels used
- [ ] Structured fields included
- [ ] Sensitive paths not logged at INFO level
- [ ] Performance metrics logged

---

## REQ-FILEWATCHER-030: Metrics Collection

### Problem Statement

The service should expose metrics for monitoring and alerting.

### Specification

#### REQ-FILEWATCHER-030.1: Key Metrics

```
WHEN file watcher service operates
THEN SHALL track and expose metrics:
  - file_watcher_events_total: counter of raw events by workspace
  - file_watcher_filtered_total: counter of filtered events
  - file_watcher_debounce_duration_seconds: histogram of debounce times
  - file_watcher_reindex_duration_seconds: histogram of reindex times
  - file_watcher_active_watchers: gauge of active watcher count
  - file_watcher_errors_total: counter of errors by type
```

### Performance Contract

| Metric | Alert Threshold | Description |
|--------|-----------------|-------------|
| reindex_duration_seconds p99 | > 30s | Reindex too slow |
| errors_total rate | > 10/min | Too many errors |
| active_watchers | > 100 | Too many watchers |
| filtered_total / events_total | < 0.5 | Filter config issue |

### Acceptance Criteria

- [ ] All specified metrics exposed
- [ ] Metrics labeled by workspace
- [ ] Histograms have appropriate buckets
- [ ] Metrics available via /metrics endpoint

---

# Implementation Guide

## Function Signatures (4-Word Naming Convention)

```rust
/// Create file watcher for workspace
///
/// # 4-Word Name: create_watcher_for_workspace
pub async fn create_watcher_for_workspace(
    state: &SharedWorkspaceStateContainer,
    workspace: &WorkspaceMetadataPayloadStruct,
) -> Result<FileWatcherServiceStruct, FileWatcherErrorType>

/// Start watching workspace directory
///
/// # 4-Word Name: start_watching_workspace_directory
pub fn start_watching_workspace_directory(
    watcher: &mut FileWatcherServiceStruct,
) -> Result<(), FileWatcherErrorType>

/// Stop watching workspace directory
///
/// # 4-Word Name: stop_watching_workspace_directory
pub async fn stop_watching_workspace_directory(
    state: &SharedWorkspaceStateContainer,
    workspace_id: &str,
) -> Result<(), FileWatcherErrorType>

/// Process debounced file events
///
/// # 4-Word Name: process_debounced_file_events
pub async fn process_debounced_file_events(
    raw_rx: mpsc::Receiver<RawFileEventData>,
    debounced_tx: mpsc::Sender<DebouncedFileChangeEvent>,
    config: DebounceConfig,
)

/// Trigger incremental reindex update
///
/// # 4-Word Name: trigger_incremental_reindex_update
pub async fn trigger_incremental_reindex_update(
    state: &SharedWorkspaceStateContainer,
    workspace_id: &str,
    changed_files: &[PathBuf],
) -> Result<(), FileWatcherErrorType>

/// Broadcast diff to subscribers
///
/// # 4-Word Name: broadcast_diff_to_subscribers
pub async fn broadcast_diff_to_subscribers(
    state: &SharedWorkspaceStateContainer,
    workspace_id: &str,
    diff: &DiffResultDataPayload,
)

/// Filter path against patterns
///
/// # 4-Word Name: filter_path_against_patterns
pub fn filter_path_against_patterns(
    path: &Path,
    patterns: &GlobSet,
) -> bool

/// Compile ignore patterns list
///
/// # 4-Word Name: compile_ignore_patterns_list
pub fn compile_ignore_patterns_list(
    patterns: &[&str],
) -> Result<GlobSet, FileWatcherErrorType>
```

## File Structure

```
crates/pt08-http-code-query-server/
  src/
    file_watcher_service_module/
      mod.rs                        # Module exports
      types.rs                      # FileWatcherServiceStruct, events, errors
      service.rs                    # create/start/stop watcher functions
      debouncer.rs                  # process_debounced_file_events
      filter.rs                     # filter_path_against_patterns
      reindex_trigger.rs            # trigger_incremental_reindex_update
      broadcaster.rs                # broadcast_diff_to_subscribers
      config.rs                     # DEFAULT_DEBOUNCE_DURATION_MS, patterns
```

---

## Quality Checklist

Before implementation is complete, verify:

- [ ] All quantities are specific and measurable (ms, counts, sizes)
- [ ] All behaviors are testable with provided templates
- [ ] Error conditions specified with exact codes
- [ ] Performance boundaries defined with p99 targets
- [ ] Test templates provided for all 30 requirements
- [ ] Acceptance criteria are binary (pass/fail)
- [ ] No ambiguous language remains
- [ ] 4-word naming convention followed for all 8 functions
- [ ] Integration with WebSocket message types verified
- [ ] Integration with WorkspaceManagerServiceStruct verified
- [ ] Integration with DiffResultDataPayload verified

---

## Traceability Matrix

| Requirement | Category | Test Count | Functions |
|-------------|----------|------------|-----------|
| REQ-FILEWATCHER-001 | Watcher Lifecycle | 4 | create_watcher_for_workspace |
| REQ-FILEWATCHER-002 | Watcher Lifecycle | 3 | start_watching_workspace_directory |
| REQ-FILEWATCHER-003 | Watcher Lifecycle | 3 | stop_watching_workspace_directory |
| REQ-FILEWATCHER-004 | Watcher Lifecycle | 2 | delete_workspace_with_cleanup |
| REQ-FILEWATCHER-005 | Watcher Lifecycle | 1 | create_watcher_for_workspace |
| REQ-FILEWATCHER-006 | Path Filtering | 4 | filter_path_against_patterns |
| REQ-FILEWATCHER-007 | Path Filtering | 2 | compile_ignore_patterns_list |
| REQ-FILEWATCHER-008 | Path Filtering | 1 | filter_path_against_patterns |
| REQ-FILEWATCHER-009 | Path Filtering | 0 | filter_path_against_patterns |
| REQ-FILEWATCHER-010 | Path Filtering | 0 | notify config |
| REQ-FILEWATCHER-011 | Debounce | 3 | process_debounced_file_events |
| REQ-FILEWATCHER-012 | Debounce | 3 | process_debounced_file_events |
| REQ-FILEWATCHER-013 | Debounce | 1 | process_debounced_file_events |
| REQ-FILEWATCHER-014 | Debounce | 1 | process_debounced_file_events |
| REQ-FILEWATCHER-015 | Debounce | 0 | process_debounced_file_events |
| REQ-FILEWATCHER-016 | Reindex | 2 | trigger_incremental_reindex_update |
| REQ-FILEWATCHER-017 | Reindex | 2 | trigger_incremental_reindex_update |
| REQ-FILEWATCHER-018 | Reindex | 1 | trigger_incremental_reindex_update |
| REQ-FILEWATCHER-019 | Reindex | 2 | trigger_incremental_reindex_update |
| REQ-FILEWATCHER-020 | Reindex | 1 | update_last_indexed_timestamp |
| REQ-FILEWATCHER-021 | Diff/Broadcast | 1 | compute_diff_after_reindex |
| REQ-FILEWATCHER-022 | Diff/Broadcast | 2 | broadcast_diff_to_subscribers |
| REQ-FILEWATCHER-023 | Diff/Broadcast | 1 | broadcast_diff_to_subscribers |
| REQ-FILEWATCHER-024 | Diff/Broadcast | 0 | broadcast_diff_to_subscribers |
| REQ-FILEWATCHER-025 | Diff/Broadcast | 0 | visualization prep |
| REQ-FILEWATCHER-026 | Error Handling | 2 | various |
| REQ-FILEWATCHER-027 | Error Handling | 0 | channel handling |
| REQ-FILEWATCHER-028 | Error Handling | 0 | timeout handling |
| REQ-FILEWATCHER-029 | Error Handling | 0 | logging |
| REQ-FILEWATCHER-030 | Error Handling | 0 | metrics |
| **Total** | **6 Categories** | **~40 tests** | **8 functions** |

---

*Specification document created 2026-01-23*
*Phase 2.2 target: File Watcher Service*
*Test target: ~40 new tests across 30 requirements*
*Depends on: Phase 2.1 (Workspace Management) - Complete*
*Integrates with: WebSocket Streaming (REQ-WEBSOCKET-*)*
