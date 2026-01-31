//! File Watcher Module: Watches directories for changes and triggers reindex
//!
//! ## S06 Compliance
//! - All function names are exactly 4 words
//! - Follows RAII resource management (auto-cleanup on drop)
//! - Dependency injection via trait interface
//!
//! ## Acceptance Criteria (WHEN...THEN...SHALL)
//!
//! 1. WHEN a file in the watched directory is saved
//!    THEN the system SHALL trigger file change callback within 500ms
//!
//! 2. WHEN multiple saves occur within 100ms
//!    THEN the system SHALL debounce and trigger only once
//!
//! 3. WHEN a file watcher fails to start
//!    THEN the system SHALL return error (graceful degradation)

use async_trait::async_trait;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;

/// Error types for file watcher operations
///
/// # 4-Word Name: FileWatcherOperationError
#[derive(Error, Debug)]
pub enum FileWatcherOperationError {
    #[error("Failed to create watcher: {0}")]
    WatcherCreationFailed(String),

    #[error("Failed to watch path: {path}")]
    WatchPathFailed { path: PathBuf },

    #[error("Watcher already running")]
    WatcherAlreadyRunning,

    #[error("Watcher not running")]
    WatcherNotRunning,

    #[error("Channel send error: {0}")]
    ChannelSendError(String),
}

/// Result type alias for file watcher operations
pub type WatcherResult<T> = Result<T, FileWatcherOperationError>;

/// Represents a file change event
///
/// # 4-Word Name: FileChangeEventPayload
#[derive(Debug, Clone)]
pub struct FileChangeEventPayload {
    /// Path to the changed file
    pub file_path: PathBuf,
    /// Type of change (create, modify, delete)
    pub change_type: FileChangeType,
}

/// Types of file changes detected
///
/// # 4-Word Name: FileChangeType
#[derive(Debug, Clone, PartialEq)]
pub enum FileChangeType {
    /// File was created
    Created,
    /// File content was modified
    Modified,
    /// File was deleted
    Deleted,
}

/// Callback type for file change events
pub type FileChangeCallback = Box<dyn Fn(FileChangeEventPayload) + Send + Sync>;

/// Trait for file watcher providers (production + mock)
///
/// # 4-Word Name: FileWatchProviderTrait
///
/// All methods follow the 4-word naming convention:
/// - start_watching_directory_recursively
/// - stop_watching_directory_now
/// - check_watcher_running_status
#[async_trait]
pub trait FileWatchProviderTrait: Send + Sync {
    /// Start watching a directory recursively for changes
    ///
    /// # 4-Word Name: start_watching_directory_recursively
    async fn start_watching_directory_recursively(
        &self,
        path: &Path,
        callback: FileChangeCallback,
    ) -> WatcherResult<()>;

    /// Stop watching the directory
    ///
    /// # 4-Word Name: stop_watching_directory_now
    async fn stop_watching_directory_now(&self) -> WatcherResult<()>;

    /// Check if the watcher is currently running
    ///
    /// # 4-Word Name: check_watcher_running_status
    fn check_watcher_running_status(&self) -> bool;
}

/// Production implementation using notify crate with debouncer
///
/// # 4-Word Name: NotifyFileWatcherProvider
pub struct NotifyFileWatcherProvider {
    /// Whether watcher is currently running
    is_running: Arc<AtomicBool>,
    /// Debounce duration (milliseconds)
    debounce_duration_ms: u64,
    /// Sender to stop the watcher
    stop_sender: tokio::sync::Mutex<Option<mpsc::Sender<()>>>,
    /// Handle to the debouncer (stored to keep it alive)
    debouncer_handle_storage: Arc<
        tokio::sync::Mutex<
            Option<notify_debouncer_full::Debouncer<RecommendedWatcher, notify_debouncer_full::FileIdMap>>,
        >,
    >,
    /// Total events processed (for metrics)
    events_processed_total_count: Arc<std::sync::atomic::AtomicU64>,
    /// Total events coalesced (for metrics)
    events_coalesced_total_count: Arc<std::sync::atomic::AtomicU64>,
    /// Timestamp of last event (milliseconds)
    last_event_timestamp_millis: Arc<std::sync::atomic::AtomicU64>,
}

impl NotifyFileWatcherProvider {
    /// Create a new notify-based file watcher
    ///
    /// # 4-Word Name: create_notify_watcher_provider
    pub fn create_notify_watcher_provider() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            debounce_duration_ms: 100,
            stop_sender: tokio::sync::Mutex::new(None),
            debouncer_handle_storage: Arc::new(tokio::sync::Mutex::new(None)),
            events_processed_total_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            events_coalesced_total_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            last_event_timestamp_millis: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Create watcher with custom debounce
    ///
    /// # 4-Word Name: create_with_debounce_duration
    pub fn create_with_debounce_duration(debounce_ms: u64) -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            debounce_duration_ms: debounce_ms,
            stop_sender: tokio::sync::Mutex::new(None),
            debouncer_handle_storage: Arc::new(tokio::sync::Mutex::new(None)),
            events_processed_total_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            events_coalesced_total_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            last_event_timestamp_millis: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Get metrics: total events processed
    ///
    /// # 4-Word Name: get_events_processed_count
    pub fn get_events_processed_count(&self) -> u64 {
        self.events_processed_total_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Get metrics: total events coalesced
    ///
    /// # 4-Word Name: get_events_coalesced_count
    pub fn get_events_coalesced_count(&self) -> u64 {
        self.events_coalesced_total_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Get metrics: last event timestamp
    ///
    /// # 4-Word Name: get_last_event_timestamp
    pub fn get_last_event_timestamp(&self) -> u64 {
        self.last_event_timestamp_millis
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait]
impl FileWatchProviderTrait for NotifyFileWatcherProvider {
    async fn start_watching_directory_recursively(
        &self,
        path: &Path,
        callback: FileChangeCallback,
    ) -> WatcherResult<()> {
        use notify_debouncer_full::{new_debouncer, DebounceEventResult, FileIdMap};

        // Check if already running
        if self.is_running.load(Ordering::SeqCst) {
            return Err(FileWatcherOperationError::WatcherAlreadyRunning);
        }

        let path_buf = path.to_path_buf();
        let is_running = self.is_running.clone();
        let debounce_ms = self.debounce_duration_ms;

        // Clone metrics for use in event handler
        let events_processed = self.events_processed_total_count.clone();
        let events_coalesced = self.events_coalesced_total_count.clone();
        let last_timestamp = self.last_event_timestamp_millis.clone();

        // Create stop channel
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        {
            let mut sender_lock = self.stop_sender.lock().await;
            *sender_lock = Some(stop_tx);
        }

        // Create event channel for debounced events
        let (event_tx, mut event_rx) = mpsc::channel::<DebounceEventResult>(100);

        // Create debouncer with event handler
        let mut debouncer = new_debouncer(
            Duration::from_millis(debounce_ms),
            None, // No extra timeout
            move |result: DebounceEventResult| {
                // Send debounced events through channel (non-blocking)
                let _ = event_tx.try_send(result);
            },
        )
        .map_err(|e| FileWatcherOperationError::WatcherCreationFailed(e.to_string()))?;

        // Start watching the path
        debouncer
            .watcher()
            .watch(&path_buf, RecursiveMode::Recursive)
            .map_err(|_| FileWatcherOperationError::WatchPathFailed {
                path: path_buf.clone(),
            })?;

        // Store debouncer handle to keep it alive
        {
            let mut handle_lock = self.debouncer_handle_storage.lock().await;
            *handle_lock = Some(debouncer);
        }

        is_running.store(true, Ordering::SeqCst);

        // Spawn event processing task
        let is_running_clone = is_running.clone();
        let callback = Arc::new(callback);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle stop signal
                    _ = stop_rx.recv() => {
                        is_running_clone.store(false, Ordering::SeqCst);
                        break;
                    }
                    // Handle debounced file events
                    Some(result) = event_rx.recv() => {
                        match result {
                            Ok(events) => {
                                // Update timestamp
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u64;
                                last_timestamp.store(now, Ordering::SeqCst);

                                // Track coalescing: if multiple events, count them
                                let event_count = events.len() as u64;
                                if event_count > 1 {
                                    events_coalesced.fetch_add(event_count - 1, Ordering::SeqCst);
                                }

                                // Process each debounced event
                                for event in events {
                                    if let Some(payload) = convert_debounced_event_to_payload(event) {
                                        // Increment processed counter
                                        events_processed.fetch_add(1, Ordering::SeqCst);

                                        // Invoke callback
                                        callback(payload);
                                    }
                                }
                            }
                            Err(errors) => {
                                // Log errors but continue watching
                                eprintln!("File watcher errors: {:?}", errors);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop_watching_directory_now(&self) -> WatcherResult<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(FileWatcherOperationError::WatcherNotRunning);
        }

        let sender_lock = self.stop_sender.lock().await;
        if let Some(ref sender) = *sender_lock {
            sender
                .send(())
                .await
                .map_err(|e| FileWatcherOperationError::ChannelSendError(e.to_string()))?;
        }

        Ok(())
    }

    fn check_watcher_running_status(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

/// Convert notify event to our event payload
///
/// # 4-Word Name: convert_notify_event_payload
fn convert_notify_event_payload(event: &Event) -> Option<FileChangeEventPayload> {
    let path = event.paths.first()?.clone();

    let change_type = match event.kind {
        EventKind::Create(_) => FileChangeType::Created,
        EventKind::Modify(_) => FileChangeType::Modified,
        EventKind::Remove(_) => FileChangeType::Deleted,
        _ => return None,
    };

    Some(FileChangeEventPayload {
        file_path: path,
        change_type,
    })
}

/// Convert debounced event to our event payload
///
/// # 4-Word Name: convert_debounced_event_to_payload
///
/// Converts `notify-debouncer-full::DebouncedEvent` to our internal event type.
/// This is the adapter between the debouncer's event format and our domain model.
///
/// # Arguments
/// * `event` - Debounced event from notify-debouncer-full
///
/// # Returns
/// * `Some(FileChangeEventPayload)` - If event is relevant (Create/Modify/Remove)
/// * `None` - If event should be ignored (Access, Other, or empty paths)
fn convert_debounced_event_to_payload(
    event: notify_debouncer_full::DebouncedEvent,
) -> Option<FileChangeEventPayload> {
    // Extract first path (events can have multiple paths, we take the first)
    let path = event.event.paths.first()?.clone();

    // Convert event kind to our change type
    let change_type = match event.event.kind {
        EventKind::Create(_) => FileChangeType::Created,
        EventKind::Modify(_) => FileChangeType::Modified,
        EventKind::Remove(_) => FileChangeType::Deleted,
        _ => return None, // Ignore Access, Other, etc.
    };

    Some(FileChangeEventPayload {
        file_path: path,
        change_type,
    })
}

/// Mock implementation for testing
///
/// # 4-Word Name: MockFileWatcherProvider
pub struct MockFileWatcherProvider {
    /// Simulated running state
    is_running: Arc<AtomicBool>,
    /// Recorded paths that were watched
    watched_paths: tokio::sync::Mutex<Vec<PathBuf>>,
}

impl MockFileWatcherProvider {
    /// Create a new mock file watcher
    ///
    /// # 4-Word Name: create_mock_watcher_provider
    pub fn create_mock_watcher_provider() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            watched_paths: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    /// Get paths that were watched (for test assertions)
    ///
    /// # 4-Word Name: get_watched_paths_list
    pub async fn get_watched_paths_list(&self) -> Vec<PathBuf> {
        self.watched_paths.lock().await.clone()
    }
}

#[async_trait]
impl FileWatchProviderTrait for MockFileWatcherProvider {
    async fn start_watching_directory_recursively(
        &self,
        path: &Path,
        _callback: FileChangeCallback,
    ) -> WatcherResult<()> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(FileWatcherOperationError::WatcherAlreadyRunning);
        }

        self.watched_paths.lock().await.push(path.to_path_buf());
        self.is_running.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn stop_watching_directory_now(&self) -> WatcherResult<()> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(FileWatcherOperationError::WatcherNotRunning);
        }

        self.is_running.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn check_watcher_running_status(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
#[path = "file_watcher_tests.rs"]
mod file_watcher_tests;
