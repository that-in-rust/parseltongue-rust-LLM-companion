//! Initial codebase scanning for file watcher
//!
//! # 4-Word Naming: initial_scan
//!
//! Performs initial directory scan BEFORE starting file watcher.
//! This solves the fundamental issue that file system watchers (FSEvents, inotify, etc.)
//! only emit events for changes that occur AFTER the watcher starts.
//!
//! ## Why This Exists
//!
//! File watchers are edge-triggered (detect transitions), not level-triggered (detect state).
//! Industry standard pattern (rust-analyzer, VS Code, watchman):
//! 1. Initial scan (manual directory walk) - THIS MODULE
//! 2. Event-based watching (incremental updates) - file_watcher_integration_service
//!
//! ## References
//!
//! - Research: /tmp/file_watching_research.md
//! - PRD-145: Bug #1 - File watcher reindexing
//!
//! # Implementation Strategy
//!
//! Reuses pt01's FileStreamerImpl instead of duplicating parsing logic.
//! This ensures consistency and reduces code duplication.
//!
//! # 4-Word Naming Convention
//!
//! All functions follow 4-word naming:
//! - execute_initial_codebase_scan
//! - extract_database_path_from_state

use anyhow::Result;
use std::path::Path;
use std::time::Instant;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

// Reuse pt01's streamer infrastructure
use pt01_folder_to_cozodb_streamer::{FileStreamer, ToolFactory, StreamerConfig};

/// Initial scan statistics
///
/// # 4-Word Name: InitialScanStatistics
#[derive(Debug, Clone)]
pub struct InitialScanStatistics {
    pub files_scanned_total_count: usize,
    pub files_processed_success_count: usize,
    pub files_skipped_error_count: usize,
    pub entities_created_total_count: usize,
    pub code_entities_created_count: usize,
    pub test_entities_created_count: usize,
    pub scan_duration_milliseconds: u128,
}

/// Execute initial codebase scan
///
/// # 4-Word Name: execute_initial_codebase_scan
///
/// Performs initial directory walk and indexes all supported files using pt01's streamer.
/// This ensures consistency with pt01's parsing logic and reduces code duplication.
///
/// ## Flow
/// 1. Extract database path from state
/// 2. Create pt01 StreamerConfig with same database
/// 3. Call pt01's stream_directory() method
/// 4. Return statistics
///
/// ## Supported Extensions (from pt01)
/// - Rust (.rs), Python (.py), JavaScript/TypeScript (.js, .ts)
/// - Go (.go), Java (.java)
/// - C (.c, .h), C++ (.cpp, .hpp)
/// - Ruby (.rb), PHP (.php), C# (.cs), Swift (.swift)
///
/// ## Error Handling
/// Graceful degradation: Logs errors but returns statistics.
pub async fn execute_initial_codebase_scan(
    workspace_path: &Path,
    state: &SharedApplicationStateContainer,
) -> Result<InitialScanStatistics> {
    let start_time = Instant::now();

    // Extract database path from state
    let db_path = extract_database_path_from_state(state).await?;

    println!("[InitialScan] Using database: {}", db_path);
    println!("[InitialScan] Scanning directory: {}", workspace_path.display());

    // Create StreamerConfig (reuse pt01 logic)
    let config = StreamerConfig {
        root_dir: workspace_path.to_path_buf(),
        db_path: db_path.clone(),
        max_file_size: 100 * 1024 * 1024, // 100MB (ultra-minimalist: let tree-sitter decide)
        include_patterns: vec!["*.rs".to_string(), "*.py".to_string(), "*.js".to_string(), "*.ts".to_string()],
        exclude_patterns: vec![
            "target/**".to_string(),
            "node_modules/**".to_string(),
            ".git/**".to_string(),
            "build/**".to_string(),
            "dist/**".to_string(),
        ],
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    // Create pt01 streamer (reuse ALL pt01 logic!)
    let streamer = ToolFactory::create_streamer(config).await
        .map_err(|e| anyhow::anyhow!("Failed to create streamer: {}", e))?;

    // Stream directory (this does the full indexing)
    let result = streamer.stream_directory().await
        .map_err(|e| anyhow::anyhow!("Failed to stream directory: {}", e))?;

    let duration = start_time.elapsed();

    // Get detailed stats from streamer
    let detailed_stats = streamer.get_stats();

    let stats = InitialScanStatistics {
        files_scanned_total_count: result.total_files,
        files_processed_success_count: result.processed_files,
        files_skipped_error_count: result.errors.len(),
        entities_created_total_count: result.entities_created,
        code_entities_created_count: detailed_stats.code_entities_created,
        test_entities_created_count: detailed_stats.test_entities_created,
        scan_duration_milliseconds: duration.as_millis(),
    };

    Ok(stats)
}

/// Extract database path from state
///
/// # 4-Word Name: extract_database_path_from_state
async fn extract_database_path_from_state(
    state: &SharedApplicationStateContainer,
) -> Result<String> {
    let stats = state.codebase_statistics_metadata_arc.read().await;
    let db_path = stats.database_file_path_string.clone();

    if db_path.is_empty() || db_path == "mem" {
        anyhow::bail!("No database path configured - cannot perform initial scan");
    }

    Ok(db_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_server_startup_runner::{SharedApplicationStateContainer, CodebaseStatisticsMetadata};

    #[tokio::test]
    async fn test_extract_database_path_from_state() {
        let state = SharedApplicationStateContainer::create_new_application_state();

        // Set database path
        {
            let mut stats = state.codebase_statistics_metadata_arc.write().await;
            stats.database_file_path_string = "rocksdb:test.db".to_string();
        }

        let db_path = extract_database_path_from_state(&state).await.unwrap();
        assert_eq!(db_path, "rocksdb:test.db");
    }

    #[tokio::test]
    async fn test_extract_database_path_fails_for_empty() {
        let state = SharedApplicationStateContainer::create_new_application_state();

        let result = extract_database_path_from_state(&state).await;
        assert!(result.is_err());
    }
}
