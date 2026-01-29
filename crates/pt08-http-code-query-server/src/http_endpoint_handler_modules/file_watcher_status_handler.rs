//! File watcher status endpoint handler
//!
//! # 4-Word Naming: file_watcher_status_handler
//!
//! Endpoint: GET /file-watcher-status-check
//!
//! ## Acceptance Criteria (WHEN...THEN...SHALL)
//!
//! 1. WHEN the endpoint is called
//!    THEN the system SHALL return file watcher status information
//!
//! 2. WHEN file watching is enabled and running
//!    THEN the response SHALL include watch directory and extensions
//!
//! 3. WHEN file watching failed to start
//!    THEN the response SHALL include the error message

use axum::{
    extract::State,
    Json,
};
use serde::Serialize;
use std::sync::atomic::Ordering;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// File watcher status response payload
///
/// # 4-Word Name: FileWatcherStatusResponse
#[derive(Debug, Serialize)]
pub struct FileWatcherStatusResponse {
    /// Whether the endpoint call succeeded
    pub success: bool,

    /// Endpoint identifier
    pub endpoint: String,

    /// Detailed status data
    pub data: FileWatcherStatusData,
}

/// Detailed file watcher status data
///
/// # 4-Word Name: FileWatcherStatusData
#[derive(Debug, Serialize)]
pub struct FileWatcherStatusData {
    /// Whether file watching was enabled at startup (--watch flag)
    pub file_watching_enabled_flag: bool,

    /// Whether the file watcher is currently running
    pub watcher_currently_running_flag: bool,

    /// Directory being watched (if any)
    pub watch_directory_path_value: Option<String>,

    /// List of file extensions being monitored
    pub watched_extensions_list_vec: Vec<String>,

    /// Total number of file events processed
    pub events_processed_total_count: usize,

    /// Error message if watcher failed to start
    pub error_message_value_option: Option<String>,

    /// Human-readable status message
    pub status_message_text_value: String,
}

/// Handle file watcher status check request
///
/// # 4-Word Name: handle_file_watcher_status_check
///
/// # Contract
/// - Precondition: Server is running
/// - Postcondition: Returns 200 with file watcher status
/// - Performance: <10ms
pub async fn handle_file_watcher_status_check(
    State(state): State<SharedApplicationStateContainer>,
) -> Json<FileWatcherStatusResponse> {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Read file watcher status metadata
    let status = state.file_watcher_status_metadata_arc.read().await;

    // Get event count from atomic counter
    let events_count = status.events_processed_count_arc.load(Ordering::SeqCst);

    // Build human-readable status message
    let status_message = if !status.watcher_enabled_status_flag {
        "File watching is disabled. Use --watch flag to enable.".to_string()
    } else if status.watcher_running_status_flag {
        format!(
            "File watcher is running. Monitoring {} extensions in {}. {} events processed.",
            status.watched_extensions_list_vec.len(),
            status.watch_directory_path_option
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            events_count
        )
    } else {
        format!(
            "File watcher failed to start: {}",
            status.watcher_error_message_option
                .as_ref()
                .unwrap_or(&"unknown error".to_string())
        )
    };

    let data = FileWatcherStatusData {
        file_watching_enabled_flag: status.watcher_enabled_status_flag,
        watcher_currently_running_flag: status.watcher_running_status_flag,
        watch_directory_path_value: status.watch_directory_path_option
            .as_ref()
            .map(|p| p.display().to_string()),
        watched_extensions_list_vec: status.watched_extensions_list_vec.clone(),
        events_processed_total_count: events_count,
        error_message_value_option: status.watcher_error_message_option.clone(),
        status_message_text_value: status_message,
    };

    Json(FileWatcherStatusResponse {
        success: true,
        endpoint: "/file-watcher-status-check".to_string(),
        data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_response_serializes() {
        let response = FileWatcherStatusResponse {
            success: true,
            endpoint: "/file-watcher-status-check".to_string(),
            data: FileWatcherStatusData {
                file_watching_enabled_flag: false,
                watcher_currently_running_flag: false,
                watch_directory_path_value: None,
                watched_extensions_list_vec: vec![],
                events_processed_total_count: 0,
                error_message_value_option: None,
                status_message_text_value: "File watching is disabled.".to_string(),
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("file-watcher-status-check"));
    }
}
