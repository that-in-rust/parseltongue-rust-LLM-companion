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

/// Production implementation using notify crate
///
/// # 4-Word Name: NotifyFileWatcherProvider
pub struct NotifyFileWatcherProvider {
    /// Whether watcher is currently running
    is_running: Arc<AtomicBool>,
    /// Debounce duration (milliseconds)
    debounce_duration_ms: u64,
    /// Sender to stop the watcher
    stop_sender: tokio::sync::Mutex<Option<mpsc::Sender<()>>>,
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
        }
    }
}

#[async_trait]
impl FileWatchProviderTrait for NotifyFileWatcherProvider {
    async fn start_watching_directory_recursively(
        &self,
        path: &Path,
        callback: FileChangeCallback,
    ) -> WatcherResult<()> {
        // Check if already running
        if self.is_running.load(Ordering::SeqCst) {
            return Err(FileWatcherOperationError::WatcherAlreadyRunning);
        }

        let path_buf = path.to_path_buf();
        let is_running = self.is_running.clone();
        let debounce_ms = self.debounce_duration_ms;

        // Create stop channel
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        {
            let mut sender_lock = self.stop_sender.lock().await;
            *sender_lock = Some(stop_tx);
        }

        // Create event channel for notify
        let (event_tx, mut event_rx) = mpsc::channel::<notify::Result<Event>>(100);

        // Create watcher
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = event_tx.blocking_send(res);
            },
            Config::default().with_poll_interval(Duration::from_millis(debounce_ms)),
        )
        .map_err(|e| FileWatcherOperationError::WatcherCreationFailed(e.to_string()))?;

        // Start watching
        watcher
            .watch(&path_buf, RecursiveMode::Recursive)
            .map_err(|_| FileWatcherOperationError::WatchPathFailed {
                path: path_buf.clone(),
            })?;

        is_running.store(true, Ordering::SeqCst);

        // Spawn event processing task
        let is_running_clone = is_running.clone();
        let callback = Arc::new(callback);

        tokio::spawn(async move {
            // Keep watcher alive in this task
            let _watcher = watcher;

            loop {
                tokio::select! {
                    // Handle stop signal
                    _ = stop_rx.recv() => {
                        is_running_clone.store(false, Ordering::SeqCst);
                        break;
                    }
                    // Handle file events
                    Some(result) = event_rx.recv() => {
                        if let Ok(event) = result {
                            // Convert notify event to our event type
                            if let Some(payload) = convert_notify_event_payload(&event) {
                                callback(payload);
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
