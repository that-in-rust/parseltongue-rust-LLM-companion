//! File Watcher Integration Service for pt08 HTTP Server
//!
//! ## S06 Compliance
//! - All function names are exactly 4 words
//! - Follows STUB → RED → GREEN → REFACTOR cycle
//! - RAII resource management (auto-cleanup on drop)
//! - Dependency injection via trait interface
//!
//! ## Acceptance Criteria (WHEN...THEN...SHALL)
//!
//! 1. WHEN a file in the watched directory is saved
//!    THEN the system SHALL trigger incremental reindex within 500ms
//!
//! 2. WHEN multiple saves occur within 100ms
//!    THEN the system SHALL debounce and trigger only once
//!
//! 3. WHEN file watcher fails to start
//!    THEN the system SHALL log error and continue (graceful degradation)

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;

// Import from pt01
use pt01_folder_to_cozodb_streamer::file_watcher::{
    FileChangeCallback, FileChangeEventPayload, FileWatchProviderTrait,
    FileWatcherOperationError, MockFileWatcherProvider, NotifyFileWatcherProvider,
};

use crate::http_server_startup_runner::SharedApplicationStateContainer;
// TODO v1.4.3: Re-enable after implementing file_parser and entity_conversion
// use crate::incremental_reindex_core_logic::execute_incremental_reindex_core;

/// Error types for file watcher integration
///
/// # 4-Word Name: FileWatcherIntegrationError
#[derive(Error, Debug)]
pub enum FileWatcherIntegrationError {
    #[error("Watcher operation failed: {0}")]
    WatcherOperationFailed(#[from] FileWatcherOperationError),

    #[error("Service already running")]
    ServiceAlreadyRunning,

    #[error("Service not running")]
    ServiceNotRunning,

    #[error("Reindex operation failed: {0}")]
    ReindexOperationFailed(String),
}

/// Result type for file watcher integration
pub type IntegrationResult<T> = Result<T, FileWatcherIntegrationError>;

/// Configuration for file watcher integration
///
/// # 4-Word Name: FileWatcherIntegrationConfig
#[derive(Debug, Clone)]
pub struct FileWatcherIntegrationConfig {
    /// Directory to watch for changes
    pub watch_directory_path_value: PathBuf,
    /// Debounce duration in milliseconds
    pub debounce_duration_milliseconds_value: u64,
    /// File extensions to watch (e.g., ["rs", "py", "js"])
    pub watched_extensions_list_vec: Vec<String>,
    /// Whether file watching is enabled
    pub file_watching_enabled_flag: bool,
}

impl Default for FileWatcherIntegrationConfig {
    fn default() -> Self {
        Self {
            watch_directory_path_value: PathBuf::from("."),
            debounce_duration_milliseconds_value: 100,
            watched_extensions_list_vec: vec![
                "rs".to_string(),
                "py".to_string(),
                "js".to_string(),
                "ts".to_string(),
                "go".to_string(),
                "java".to_string(),
            ],
            file_watching_enabled_flag: false,
        }
    }
}

/// Service to manage file watcher lifecycle and trigger reindex
///
/// # 4-Word Name: FileWatcherIntegrationService
pub struct FileWatcherIntegrationService<W: FileWatchProviderTrait> {
    /// The file watcher provider (production or mock)
    watcher_provider: Arc<W>,
    /// Reference to application state for triggering reindex
    application_state: SharedApplicationStateContainer,
    /// Configuration for the service
    config: FileWatcherIntegrationConfig,
    /// Debounce state to coalesce rapid changes
    pending_changes_map_arc: Arc<RwLock<HashMap<PathBuf, Instant>>>,
    /// Whether service is running
    service_running_status_flag: Arc<AtomicBool>,
    /// Count of events processed (for testing)
    events_processed_count_arc: Arc<AtomicUsize>,
}

impl<W: FileWatchProviderTrait + 'static> FileWatcherIntegrationService<W> {
    /// Create new file watcher integration service
    ///
    /// # 4-Word Name: create_file_watcher_service
    pub fn create_file_watcher_service(
        watcher_provider: Arc<W>,
        application_state: SharedApplicationStateContainer,
        config: FileWatcherIntegrationConfig,
    ) -> Self {
        Self {
            watcher_provider,
            application_state,
            config,
            pending_changes_map_arc: Arc::new(RwLock::new(HashMap::new())),
            service_running_status_flag: Arc::new(AtomicBool::new(false)),
            events_processed_count_arc: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Start the file watcher integration
    ///
    /// # 4-Word Name: start_file_watcher_service
    pub async fn start_file_watcher_service(&self) -> IntegrationResult<()> {
        if self.service_running_status_flag.load(Ordering::SeqCst) {
            return Err(FileWatcherIntegrationError::ServiceAlreadyRunning);
        }

        let events_count = self.events_processed_count_arc.clone();
        let pending_changes = self.pending_changes_map_arc.clone();
        let debounce_ms = self.config.debounce_duration_milliseconds_value;
        let extensions = self.config.watched_extensions_list_vec.clone();
        let state = self.application_state.clone();

        // Create callback that handles file changes
        // GREEN PHASE: Implements debounce + reindex trigger
        let callback: FileChangeCallback = Box::new(move |event: FileChangeEventPayload| {
            // Check if file extension is watched
            if let Some(ext) = event.file_path.extension() {
                let ext_str = ext.to_string_lossy().to_string();
                if !extensions.contains(&ext_str) {
                    return; // Skip non-watched extensions
                }
            } else {
                return; // Skip files without extensions
            }

            events_count.fetch_add(1, Ordering::SeqCst);

            // Clone Arcs for the async task
            let pending_changes = pending_changes.clone();
            let state = state.clone();
            let file_path = event.file_path.clone();
            let change_type = event.change_type;

            // Spawn async task for debounce + reindex (non-blocking)
            tokio::spawn(async move {
                // Record this change with current timestamp
                let event_time = Instant::now();
                {
                    let mut map = pending_changes.write().await;
                    map.insert(file_path.clone(), event_time);
                }

                // Wait for debounce period
                tokio::time::sleep(Duration::from_millis(debounce_ms)).await;

                // Check if this is still the most recent event for this file
                let should_process = {
                    let map = pending_changes.read().await;
                    if let Some(&recorded_time) = map.get(&file_path) {
                        // Only process if our event_time matches the recorded time
                        // (no newer events have overwritten it)
                        recorded_time == event_time
                    } else {
                        false
                    }
                };

                if should_process {
                    // Remove from pending before processing
                    {
                        let mut map = pending_changes.write().await;
                        map.remove(&file_path);
                    }

                    let file_path_str = file_path.to_string_lossy().to_string();
                    println!(
                        "[FileWatcher] Processing {:?}: {}",
                        change_type, file_path_str
                    );

                    // TODO v1.4.3: Re-enable after implementing file_parser and entity_conversion
                    // Trigger incremental reindex
                    // match execute_incremental_reindex_core(&file_path_str, &state).await {
                    //     Ok(result) => {
                    //         if result.hash_changed {
                    //             println!(
                    //                 "[FileWatcher] Reindexed {}: +{} entities, -{} entities, +{} edges, -{} edges ({}ms)",
                    //                 result.file_path,
                    //                 result.entities_added,
                    //                 result.entities_removed,
                    //                 result.edges_added,
                    //                 result.edges_removed,
                    //                 result.processing_time_ms
                    //             );
                    //         } else {
                    //             println!(
                    //                 "[FileWatcher] Skipped {} (content unchanged)",
                    //                 result.file_path
                    //             );
                    //         }
                    //     }
                    //     Err(e) => {
                    //         // Graceful degradation: log error but continue watching
                    //         eprintln!(
                    //             "[FileWatcher] Reindex failed for {}: {}",
                    //             file_path_str, e
                    //         );
                    //     }
                    // }

                    // Temporary stub: Just log the event
                    println!("[FileWatcher] Event logged (reindex temporarily disabled)")
                }
                // else: A newer event superseded this one, skip processing
            });
        });

        self.watcher_provider
            .start_watching_directory_recursively(&self.config.watch_directory_path_value, callback)
            .await?;

        self.service_running_status_flag.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Stop the file watcher integration
    ///
    /// # 4-Word Name: stop_file_watcher_service
    pub async fn stop_file_watcher_service(&self) -> IntegrationResult<()> {
        if !self.service_running_status_flag.load(Ordering::SeqCst) {
            return Err(FileWatcherIntegrationError::ServiceNotRunning);
        }

        self.watcher_provider.stop_watching_directory_now().await?;
        self.service_running_status_flag.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Check if service is running
    ///
    /// # 4-Word Name: check_service_running_status
    pub fn check_service_running_status(&self) -> bool {
        self.service_running_status_flag.load(Ordering::SeqCst)
    }

    /// Get count of events processed
    ///
    /// # 4-Word Name: get_events_processed_count
    pub fn get_events_processed_count(&self) -> usize {
        self.events_processed_count_arc.load(Ordering::SeqCst)
    }

    /// Check if file extension is watched
    ///
    /// # 4-Word Name: check_extension_is_watched
    pub fn check_extension_is_watched(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_string();
            self.config.watched_extensions_list_vec.contains(&ext_str)
        } else {
            false
        }
    }
}

/// Type alias for production service
pub type ProductionFileWatcherService =
    FileWatcherIntegrationService<NotifyFileWatcherProvider>;

/// Type alias for test service
pub type MockFileWatcherService = FileWatcherIntegrationService<MockFileWatcherProvider>;

/// Create production file watcher service
///
/// # 4-Word Name: create_production_watcher_service
pub fn create_production_watcher_service(
    application_state: SharedApplicationStateContainer,
    config: FileWatcherIntegrationConfig,
) -> ProductionFileWatcherService {
    let provider = NotifyFileWatcherProvider::create_with_debounce_duration(
        config.debounce_duration_milliseconds_value,
    );
    FileWatcherIntegrationService::create_file_watcher_service(
        Arc::new(provider),
        application_state,
        config,
    )
}

/// Create mock file watcher service for testing
///
/// # 4-Word Name: create_mock_watcher_service
pub fn create_mock_watcher_service(
    application_state: SharedApplicationStateContainer,
    config: FileWatcherIntegrationConfig,
) -> MockFileWatcherService {
    let provider = MockFileWatcherProvider::create_mock_watcher_provider();
    FileWatcherIntegrationService::create_file_watcher_service(
        Arc::new(provider),
        application_state,
        config,
    )
}

#[cfg(test)]
#[path = "file_watcher_integration_service_tests.rs"]
mod file_watcher_integration_service_tests;
