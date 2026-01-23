//! Debouncer service for file watcher events
//!
//! # 4-Word Naming: debouncer_service_event_aggregator
//!
//! This module provides debouncing capabilities to aggregate rapid file
//! system events into batches, preventing redundant reindexing operations.
//!
//! ## Requirements Implemented:
//! - REQ-FILEWATCHER-011: Debounce window configuration
//! - REQ-FILEWATCHER-012: Event aggregation and deduplication
//! - REQ-FILEWATCHER-013: Maximum buffer size handling
//! - REQ-FILEWATCHER-014: Workspace isolation
//! - REQ-FILEWATCHER-015: Edge case handling

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::{interval, Instant};

use parseltongue_core::workspace::WorkspaceUniqueIdentifierType;

use super::watcher_types::{
    DebouncedFileChangeEventStruct,
    FileEventKindType,
    RawFileEventDataStruct,
    WatcherConfigurationStruct,
};

// =============================================================================
// Debouncer Service
// =============================================================================

/// Debouncer service for aggregating file events
///
/// # 4-Word Name: DebouncerServiceStruct
///
/// Manages a debounce window for a single workspace, aggregating
/// rapid file events into batches.
#[derive(Debug)]
pub struct DebouncerServiceStruct {
    /// Workspace identifier
    workspace_identifier_value: WorkspaceUniqueIdentifierType,
    /// Debounce duration in milliseconds
    debounce_duration_ms_value: u64,
    /// Maximum events before force-flush
    max_buffered_events_value: usize,
    /// Pending file paths (deduplicated)
    pending_paths_set_value: HashSet<PathBuf>,
    /// Event kind tracking for merge logic
    path_event_kinds_map: HashMap<PathBuf, FileEventKindType>,
    /// Raw event count before deduplication
    raw_event_count_value: usize,
    /// Last event timestamp for timer reset
    last_event_instant_value: Option<Instant>,
}

impl DebouncerServiceStruct {
    /// Create new debouncer for workspace
    ///
    /// # 4-Word Name: create_for_workspace_config
    pub fn create_for_workspace_config(
        workspace_id: WorkspaceUniqueIdentifierType,
        config: &WatcherConfigurationStruct,
    ) -> Self {
        Self {
            workspace_identifier_value: workspace_id,
            debounce_duration_ms_value: config.debounce_duration_milliseconds_value,
            max_buffered_events_value: config.max_buffered_events_value,
            pending_paths_set_value: HashSet::new(),
            path_event_kinds_map: HashMap::new(),
            raw_event_count_value: 0,
            last_event_instant_value: None,
        }
    }

    /// Create with default configuration
    ///
    /// # 4-Word Name: create_with_default_config
    pub fn create_with_default_config(workspace_id: WorkspaceUniqueIdentifierType) -> Self {
        Self::create_for_workspace_config(workspace_id, &WatcherConfigurationStruct::default())
    }

    /// Add raw event to buffer
    ///
    /// # 4-Word Name: add_raw_event_buffer
    ///
    /// Returns `true` if buffer should be force-flushed (at capacity).
    pub fn add_raw_event_buffer(&mut self, event: RawFileEventDataStruct) -> bool {
        self.raw_event_count_value += 1;
        self.last_event_instant_value = Some(Instant::now());

        for path in event.affected_paths_list_value {
            // Handle event kind merging (REQ-FILEWATCHER-012.3)
            let existing_kind = self.path_event_kinds_map.get(&path).copied();
            let new_kind = merge_event_kinds_pair(existing_kind, event.event_kind_type_value);

            if let Some(merged_kind) = new_kind {
                self.path_event_kinds_map.insert(path.clone(), merged_kind);
                self.pending_paths_set_value.insert(path);
            } else {
                // Create+Delete cancels out - remove from pending
                self.pending_paths_set_value.remove(&path);
                self.path_event_kinds_map.remove(&path);
            }
        }

        // Check if force-flush needed
        self.pending_paths_set_value.len() >= self.max_buffered_events_value
    }

    /// Check if debounce window has elapsed
    ///
    /// # 4-Word Name: is_debounce_window_elapsed
    pub fn is_debounce_window_elapsed(&self) -> bool {
        if let Some(last_event) = self.last_event_instant_value {
            last_event.elapsed() >= Duration::from_millis(self.debounce_duration_ms_value)
        } else {
            false
        }
    }

    /// Flush buffer and create debounced event
    ///
    /// # 4-Word Name: flush_buffer_create_event
    ///
    /// Returns `None` if buffer is empty.
    pub fn flush_buffer_create_event(&mut self) -> Option<DebouncedFileChangeEventStruct> {
        if self.pending_paths_set_value.is_empty() {
            return None;
        }

        // Sort paths for deterministic ordering
        let mut paths: Vec<PathBuf> = self.pending_paths_set_value.drain().collect();
        paths.sort();

        let raw_count = self.raw_event_count_value;

        // Clear state
        self.path_event_kinds_map.clear();
        self.raw_event_count_value = 0;
        self.last_event_instant_value = None;

        Some(DebouncedFileChangeEventStruct::create_new_debounced_event(
            self.workspace_identifier_value.clone(),
            paths,
            raw_count,
        ))
    }

    /// Check if buffer has pending events
    ///
    /// # 4-Word Name: has_pending_events_check
    pub fn has_pending_events_check(&self) -> bool {
        !self.pending_paths_set_value.is_empty()
    }

    /// Get pending event count
    ///
    /// # 4-Word Name: get_pending_event_count
    pub fn get_pending_event_count(&self) -> usize {
        self.pending_paths_set_value.len()
    }

    /// Clear all pending events (for stop/cancel)
    ///
    /// # 4-Word Name: clear_all_pending_events
    pub fn clear_all_pending_events(&mut self) {
        self.pending_paths_set_value.clear();
        self.path_event_kinds_map.clear();
        self.raw_event_count_value = 0;
        self.last_event_instant_value = None;
    }
}

// =============================================================================
// Event Kind Merging Logic
// =============================================================================

/// Merge two event kinds for the same path
///
/// # 4-Word Name: merge_event_kinds_pair
///
/// Returns `None` if events cancel out (Create+Delete).
/// Returns the merged kind otherwise.
fn merge_event_kinds_pair(
    existing: Option<FileEventKindType>,
    new: FileEventKindType,
) -> Option<FileEventKindType> {
    match (existing, new) {
        // No existing event - use new
        (None, kind) => Some(kind),

        // Create + Delete = cancel out (file created and deleted in same window)
        (Some(FileEventKindType::FileCreated), FileEventKindType::FileDeleted) => None,

        // Create + Modify = still Create (final state is new file)
        (Some(FileEventKindType::FileCreated), FileEventKindType::FileModified) => {
            Some(FileEventKindType::FileCreated)
        }

        // Modify + Delete = Delete (file modified then deleted)
        (Some(FileEventKindType::FileModified), FileEventKindType::FileDeleted) => {
            Some(FileEventKindType::FileDeleted)
        }

        // Delete + Create = Modify (file replaced)
        (Some(FileEventKindType::FileDeleted), FileEventKindType::FileCreated) => {
            Some(FileEventKindType::FileModified)
        }

        // Same kind = keep same kind
        (Some(existing_kind), new_kind) if existing_kind == new_kind => Some(existing_kind),

        // Default: use new kind
        (Some(_), kind) => Some(kind),
    }
}

// =============================================================================
// Async Event Processing
// =============================================================================

/// Process debounced file events (async task)
///
/// # 4-Word Name: process_debounced_file_events
///
/// This is the main async task that:
/// 1. Receives raw events from the watcher
/// 2. Buffers and deduplicates events
/// 3. Waits for debounce window
/// 4. Sends debounced events to the output channel
///
/// ## Cancellation
/// Stops when `raw_rx` channel is closed or `stop_rx` receives signal.
pub async fn process_debounced_file_events(
    workspace_id: WorkspaceUniqueIdentifierType,
    mut raw_rx: mpsc::Receiver<RawFileEventDataStruct>,
    debounced_tx: mpsc::Sender<DebouncedFileChangeEventStruct>,
    config: WatcherConfigurationStruct,
) {
    let mut debouncer = DebouncerServiceStruct::create_for_workspace_config(
        workspace_id.clone(),
        &config,
    );

    let debounce_duration = Duration::from_millis(config.debounce_duration_milliseconds_value);
    let mut check_interval = interval(Duration::from_millis(50)); // Check every 50ms

    loop {
        tokio::select! {
            // Receive raw event
            Some(event) = raw_rx.recv() => {
                let force_flush = debouncer.add_raw_event_buffer(event);

                if force_flush {
                    // Buffer full - force flush
                    if let Some(debounced_event) = debouncer.flush_buffer_create_event() {
                        if debounced_tx.send(debounced_event).await.is_err() {
                            // Receiver dropped - exit
                            break;
                        }
                    }
                }
            }

            // Check debounce window periodically
            _ = check_interval.tick() => {
                if debouncer.has_pending_events_check() && debouncer.is_debounce_window_elapsed() {
                    if let Some(debounced_event) = debouncer.flush_buffer_create_event() {
                        if debounced_tx.send(debounced_event).await.is_err() {
                            // Receiver dropped - exit
                            break;
                        }
                    }
                }
            }

            // Channel closed - exit
            else => {
                break;
            }
        }
    }

    // Final flush on shutdown
    if let Some(debounced_event) = debouncer.flush_buffer_create_event() {
        let _ = debounced_tx.send(debounced_event).await;
    }
}

/// Create debouncer with channel (helper for tests)
///
/// # 4-Word Name: create_debouncer_with_channel
pub fn create_debouncer_with_channel(
    workspace_id: &str,
    debounce_ms: u64,
) -> (
    mpsc::Sender<RawFileEventDataStruct>,
    mpsc::Receiver<DebouncedFileChangeEventStruct>,
    tokio::task::JoinHandle<()>,
) {
    let (raw_tx, raw_rx) = mpsc::channel::<RawFileEventDataStruct>(1000);
    let (debounced_tx, debounced_rx) = mpsc::channel::<DebouncedFileChangeEventStruct>(100);

    let config = WatcherConfigurationStruct::default()
        .with_debounce_duration_ms(debounce_ms);

    let workspace_id_owned = workspace_id.to_string();
    let handle = tokio::spawn(async move {
        process_debounced_file_events(
            workspace_id_owned,
            raw_rx,
            debounced_tx,
            config,
        ).await;
    });

    (raw_tx, debounced_rx, handle)
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::watcher_types::DEFAULT_DEBOUNCE_DURATION_MS;

    // =========================================================================
    // DebouncerServiceStruct Tests
    // =========================================================================

    /// Test debouncer creation
    #[test]
    fn test_debouncer_creation() {
        let debouncer = DebouncerServiceStruct::create_with_default_config("ws_test".to_string());

        assert_eq!(debouncer.debounce_duration_ms_value, DEFAULT_DEBOUNCE_DURATION_MS);
        assert!(!debouncer.has_pending_events_check());
    }

    /// Test adding events to buffer
    #[test]
    fn test_add_event_to_buffer() {
        let mut debouncer = DebouncerServiceStruct::create_with_default_config("ws_test".to_string());

        let event = RawFileEventDataStruct::create_new_event_data(
            FileEventKindType::FileModified,
            vec![PathBuf::from("/test/file.rs")],
        );

        let force_flush = debouncer.add_raw_event_buffer(event);

        assert!(!force_flush);
        assert!(debouncer.has_pending_events_check());
        assert_eq!(debouncer.get_pending_event_count(), 1);
    }

    // =========================================================================
    // REQ-FILEWATCHER-012: Event Aggregation Tests
    // =========================================================================

    /// REQ-FILEWATCHER-012.1: Duplicate paths deduplicated
    #[test]
    fn test_duplicate_paths_deduplicated() {
        let mut debouncer = DebouncerServiceStruct::create_with_default_config("ws_test".to_string());
        let path = PathBuf::from("/test/file.rs");

        // Add same path multiple times
        for _ in 0..5 {
            let event = RawFileEventDataStruct::create_new_event_data(
                FileEventKindType::FileModified,
                vec![path.clone()],
            );
            debouncer.add_raw_event_buffer(event);
        }

        assert_eq!(debouncer.get_pending_event_count(), 1);
        assert_eq!(debouncer.raw_event_count_value, 5);
    }

    /// REQ-FILEWATCHER-012.2: Multiple files aggregated and sorted
    #[test]
    fn test_multiple_files_aggregated_sorted() {
        let mut debouncer = DebouncerServiceStruct::create_with_default_config("ws_test".to_string());

        // Add in non-alphabetical order
        for path in &["/test/c.rs", "/test/a.rs", "/test/b.rs"] {
            let event = RawFileEventDataStruct::create_new_event_data(
                FileEventKindType::FileModified,
                vec![PathBuf::from(path)],
            );
            debouncer.add_raw_event_buffer(event);
        }

        let result = debouncer.flush_buffer_create_event().unwrap();

        assert_eq!(result.changed_file_paths_list.len(), 3);
        assert_eq!(result.changed_file_paths_list[0], PathBuf::from("/test/a.rs"));
        assert_eq!(result.changed_file_paths_list[1], PathBuf::from("/test/b.rs"));
        assert_eq!(result.changed_file_paths_list[2], PathBuf::from("/test/c.rs"));
    }

    /// REQ-FILEWATCHER-012.3: Create+Delete cancels out
    #[test]
    fn test_create_delete_cancels_out() {
        let mut debouncer = DebouncerServiceStruct::create_with_default_config("ws_test".to_string());
        let path = PathBuf::from("/test/temp.rs");

        // Create event
        debouncer.add_raw_event_buffer(RawFileEventDataStruct::create_new_event_data(
            FileEventKindType::FileCreated,
            vec![path.clone()],
        ));

        // Delete event
        debouncer.add_raw_event_buffer(RawFileEventDataStruct::create_new_event_data(
            FileEventKindType::FileDeleted,
            vec![path.clone()],
        ));

        // Path should be removed (net zero change)
        assert!(!debouncer.pending_paths_set_value.contains(&path));
    }

    /// REQ-FILEWATCHER-012.3: Create+Modify = Create
    #[test]
    fn test_create_modify_equals_create() {
        let mut debouncer = DebouncerServiceStruct::create_with_default_config("ws_test".to_string());
        let path = PathBuf::from("/test/new.rs");

        debouncer.add_raw_event_buffer(RawFileEventDataStruct::create_new_event_data(
            FileEventKindType::FileCreated,
            vec![path.clone()],
        ));

        debouncer.add_raw_event_buffer(RawFileEventDataStruct::create_new_event_data(
            FileEventKindType::FileModified,
            vec![path.clone()],
        ));

        assert_eq!(
            debouncer.path_event_kinds_map.get(&path),
            Some(&FileEventKindType::FileCreated)
        );
    }

    // =========================================================================
    // REQ-FILEWATCHER-013: Maximum Buffer Size Tests
    // =========================================================================

    /// REQ-FILEWATCHER-013.1: Buffer force-flushes at limit
    #[test]
    fn test_buffer_force_flush_at_limit() {
        let config = WatcherConfigurationStruct::default();
        let mut debouncer = DebouncerServiceStruct {
            workspace_identifier_value: "ws_test".to_string(),
            debounce_duration_ms_value: 500,
            max_buffered_events_value: 5, // Low limit for test
            pending_paths_set_value: HashSet::new(),
            path_event_kinds_map: HashMap::new(),
            raw_event_count_value: 0,
            last_event_instant_value: None,
        };

        // Add events up to limit
        for i in 0..5 {
            let event = RawFileEventDataStruct::create_new_event_data(
                FileEventKindType::FileCreated,
                vec![PathBuf::from(format!("/test/file_{}.rs", i))],
            );
            let force_flush = debouncer.add_raw_event_buffer(event);

            if i < 4 {
                assert!(!force_flush);
            } else {
                assert!(force_flush);
            }
        }
    }

    // =========================================================================
    // REQ-FILEWATCHER-015: Edge Cases Tests
    // =========================================================================

    /// REQ-FILEWATCHER-015.1: Empty buffer returns None
    #[test]
    fn test_empty_buffer_returns_none() {
        let mut debouncer = DebouncerServiceStruct::create_with_default_config("ws_test".to_string());

        let result = debouncer.flush_buffer_create_event();
        assert!(result.is_none());
    }

    /// Test clear pending events
    #[test]
    fn test_clear_pending_events() {
        let mut debouncer = DebouncerServiceStruct::create_with_default_config("ws_test".to_string());

        debouncer.add_raw_event_buffer(RawFileEventDataStruct::create_new_event_data(
            FileEventKindType::FileModified,
            vec![PathBuf::from("/test/file.rs")],
        ));

        assert!(debouncer.has_pending_events_check());

        debouncer.clear_all_pending_events();

        assert!(!debouncer.has_pending_events_check());
        assert_eq!(debouncer.raw_event_count_value, 0);
    }

    // =========================================================================
    // Event Kind Merging Tests
    // =========================================================================

    /// Test merge_event_kinds_pair function
    #[test]
    fn test_event_kind_merging() {
        // No existing + any = new
        assert_eq!(
            merge_event_kinds_pair(None, FileEventKindType::FileCreated),
            Some(FileEventKindType::FileCreated)
        );

        // Create + Delete = None (cancel)
        assert_eq!(
            merge_event_kinds_pair(Some(FileEventKindType::FileCreated), FileEventKindType::FileDeleted),
            None
        );

        // Create + Modify = Create
        assert_eq!(
            merge_event_kinds_pair(Some(FileEventKindType::FileCreated), FileEventKindType::FileModified),
            Some(FileEventKindType::FileCreated)
        );

        // Modify + Delete = Delete
        assert_eq!(
            merge_event_kinds_pair(Some(FileEventKindType::FileModified), FileEventKindType::FileDeleted),
            Some(FileEventKindType::FileDeleted)
        );

        // Delete + Create = Modify
        assert_eq!(
            merge_event_kinds_pair(Some(FileEventKindType::FileDeleted), FileEventKindType::FileCreated),
            Some(FileEventKindType::FileModified)
        );

        // Same + Same = Same
        assert_eq!(
            merge_event_kinds_pair(Some(FileEventKindType::FileModified), FileEventKindType::FileModified),
            Some(FileEventKindType::FileModified)
        );
    }

    // =========================================================================
    // Async Tests (require tokio runtime)
    // =========================================================================

    /// REQ-FILEWATCHER-011.1: Debounce window of 500ms (async test)
    #[tokio::test]
    async fn test_debounce_window_timing() {
        let (raw_tx, mut debounced_rx, _handle) = create_debouncer_with_channel("ws_test", 100);

        // Send event
        let start = std::time::Instant::now();
        raw_tx.send(RawFileEventDataStruct::create_new_event_data(
            FileEventKindType::FileModified,
            vec![PathBuf::from("/test/file.rs")],
        )).await.unwrap();

        // Wait for debounced event
        let event = tokio::time::timeout(
            Duration::from_millis(500),
            debounced_rx.recv(),
        ).await.unwrap().unwrap();

        let elapsed = start.elapsed();

        // Should arrive after debounce window (~100ms)
        assert!(elapsed >= Duration::from_millis(100));
        assert!(elapsed < Duration::from_millis(300));
        assert_eq!(event.changed_file_paths_list.len(), 1);
    }

    /// REQ-FILEWATCHER-011.1: Timer resets on new event
    #[tokio::test]
    async fn test_debounce_timer_resets() {
        let (raw_tx, mut debounced_rx, _handle) = create_debouncer_with_channel("ws_test", 200);

        let start = std::time::Instant::now();

        // Send first event
        raw_tx.send(RawFileEventDataStruct::create_new_event_data(
            FileEventKindType::FileModified,
            vec![PathBuf::from("/test/a.rs")],
        )).await.unwrap();

        // Wait 100ms then send another
        tokio::time::sleep(Duration::from_millis(100)).await;
        raw_tx.send(RawFileEventDataStruct::create_new_event_data(
            FileEventKindType::FileModified,
            vec![PathBuf::from("/test/b.rs")],
        )).await.unwrap();

        // Wait for debounced event
        let event = tokio::time::timeout(
            Duration::from_millis(500),
            debounced_rx.recv(),
        ).await.unwrap().unwrap();

        let elapsed = start.elapsed();

        // Should arrive ~200ms after LAST event (100ms + 200ms = ~300ms total)
        assert!(elapsed >= Duration::from_millis(250));
        assert_eq!(event.changed_file_paths_list.len(), 2);
    }
}
