//! File watcher service module for real-time file change detection
//!
//! # 4-Word Naming: file_watcher_service_module
//!
//! This module provides file system monitoring capabilities for workspaces,
//! enabling real-time diff visualization when files change. It integrates
//! with the WebSocket streaming system to push updates to connected clients.
//!
//! ## Architecture
//!
//! ```text
//! File System Events (notify crate)
//!          |
//!          v
//! +------------------------+
//! | FileWatcherServiceStruct |
//! |  - Watches directory   |
//! |  - Filters ignored     |
//! |  - Batches events      |
//! +------------------------+
//!          |
//!          | Raw Events
//!          v
//! +------------------------+
//! | DebouncerServiceStruct   |
//! |  - 500ms window        |
//! |  - Deduplicates        |
//! |  - Aggregates paths    |
//! +------------------------+
//!          |
//!          | Debounced Event
//!          v
//! +------------------------+
//! | ReindexTriggerService  |
//! |  - Incremental reindex |
//! |  - Update live.db      |
//! |  - Compute diff        |
//! +------------------------+
//!          |
//!          | DiffResultDataPayload
//!          v
//! +------------------------+
//! | WebSocket Broadcaster  |
//! |  - Stream to clients   |
//! |  - Entity/Edge events  |
//! +------------------------+
//! ```
//!
//! ## Requirements Implemented
//!
//! - REQ-FILEWATCHER-001 to 005: Watcher lifecycle management
//! - REQ-FILEWATCHER-006 to 010: Path filtering and ignore patterns
//! - REQ-FILEWATCHER-011 to 015: Debounce behavior and event aggregation
//! - REQ-FILEWATCHER-016 to 020: Reindex triggering and completion
//! - REQ-FILEWATCHER-021 to 025: Diff computation and broadcast
//! - REQ-FILEWATCHER-026 to 030: Error handling and observability
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use pt08_http_code_query_server::file_watcher_service_module::{
//!     create_watcher_for_workspace,
//!     start_watching_workspace_directory,
//!     stop_watching_workspace_directory,
//! };
//!
//! // Create watcher for workspace
//! let watcher = create_watcher_for_workspace(&state, &workspace).await?;
//!
//! // Start watching
//! start_watching_workspace_directory(&mut watcher)?;
//!
//! // Later, stop watching
//! stop_watching_workspace_directory(&state, &workspace_id).await?;
//! ```

pub mod watcher_types;
pub mod path_filter;
pub mod debouncer;
pub mod watcher_service;

// Re-export core types for convenience
pub use watcher_types::{
    FileEventKindType,
    RawFileEventDataStruct,
    DebouncedFileChangeEventStruct,
    WatcherConfigurationStruct,
    WatcherHandleContainerStruct,
    FileWatcherErrorType,
    DEFAULT_DEBOUNCE_DURATION_MS,
    MAX_BUFFERED_EVENTS_COUNT,
    DEFAULT_IGNORE_PATTERNS_LIST,
};

pub use path_filter::{
    PathFilterConfigurationStruct,
    filter_path_against_patterns,
    compile_ignore_patterns_list,
};

pub use debouncer::{
    DebouncerServiceStruct,
    process_debounced_file_events,
};

pub use watcher_service::{
    FileWatcherServiceStruct,
    create_watcher_for_workspace,
    start_watching_workspace_directory,
    stop_watching_workspace_directory,
    trigger_incremental_reindex_update,
    broadcast_diff_to_subscribers,
};
