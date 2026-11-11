//! Incremental reindex file update endpoint handler
//!
//! # 4-Word Naming: incremental_reindex_file_handler
//!
//! Endpoint: POST /incremental-reindex-file-update?path=/path/to/file.rs
//!
//! PRD-2026-01-28: Live Tracking / Incremental Indexing
//! Incrementally updates the dependency graph when a single file changes.
//! Uses file hash comparison to skip unchanged files.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::incremental_reindex_core_logic::{execute_incremental_reindex_core, IncrementalReindexOperationError};

/// Query parameters for incremental reindex endpoint
///
/// # 4-Word Name: IncrementalReindexQueryParams
#[derive(Debug, Deserialize)]
pub struct IncrementalReindexQueryParams {
    /// Absolute file path to reindex (required)
    pub path: String,
}

/// Incremental reindex response data
///
/// # 4-Word Name: IncrementalReindexDataPayload
#[derive(Debug, Serialize)]
pub struct IncrementalReindexDataPayload {
    pub file_path: String,
    pub entities_before: usize,
    pub entities_after: usize,
    pub entities_added: usize,
    pub entities_removed: usize,
    pub entities_modified: usize,
    pub edges_added: usize,
    pub edges_removed: usize,
    pub hash_changed: bool,
    pub processing_time_ms: u64,
}

/// Success response for incremental reindex
///
/// # 4-Word Name: IncrementalReindexSuccessResponse
#[derive(Debug, Serialize)]
pub struct IncrementalReindexSuccessResponse {
    pub success: bool,
    pub endpoint: String,
    pub data: IncrementalReindexDataPayload,
}

/// Error response for incremental reindex
///
/// # 4-Word Name: IncrementalReindexErrorResponse
#[derive(Debug, Serialize)]
pub struct IncrementalReindexErrorResponse {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
}

/// Handle incremental reindex file update request
///
/// # 4-Word Name: handle_incremental_reindex_file_request
///
/// # Contract
/// - Precondition: Database connected with CodeGraph and DependencyEdges schemas
/// - Precondition: File path is absolute and exists (or returns error)
/// - Postcondition: Returns diff statistics of what changed
/// - Performance: <500ms for typical file with <100 entities
///
/// # Algorithm (ISGL1 v2 with Entity Matching)
/// 1. Validate path parameter and file existence
/// 2. Delegate to execute_incremental_reindex_core() which:
///    - Reads file content, computes SHA-256 hash
///    - Compares hash against cached value (early return if unchanged)
///    - Re-parses file using FileParser
///    - Matches new entities against old entities (ISGL1 v2)
///    - Preserves keys for matched entities (ContentMatch/PositionMatch)
///    - Assigns new keys only for truly new entities
///    - Deletes only unmatched entities and their edges
///    - Upserts matched/new entities
///    - Updates hash cache
/// 3. Map result to HTTP response with diff statistics
///
/// # URL Pattern
/// - Endpoint: POST /incremental-reindex-file-update?path=/path/to/file.rs
pub async fn handle_incremental_reindex_file_request(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<IncrementalReindexQueryParams>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    let endpoint = "/incremental-reindex-file-update".to_string();

    // Validate path parameter
    if params.path.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(IncrementalReindexErrorResponse {
                success: false,
                error: "Path parameter is required".to_string(),
                endpoint,
            }),
        ).into_response();
    }

    let file_path = Path::new(&params.path);

    // Check if file exists
    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            Json(IncrementalReindexErrorResponse {
                success: false,
                error: format!("File not found: {}", params.path),
                endpoint,
            }),
        ).into_response();
    }

    // Check if it's a file (not directory)
    if !file_path.is_file() {
        return (
            StatusCode::BAD_REQUEST,
            Json(IncrementalReindexErrorResponse {
                success: false,
                error: format!("Path is not a file: {}", params.path),
                endpoint,
            }),
        ).into_response();
    }

    // Execute core incremental reindex logic
    match execute_incremental_reindex_core(&params.path, &state).await {
        Ok(result) => {
            (
                StatusCode::OK,
                Json(IncrementalReindexSuccessResponse {
                    success: true,
                    endpoint,
                    data: IncrementalReindexDataPayload {
                        file_path: result.file_path,
                        entities_before: result.entities_before,
                        entities_after: result.entities_after,
                        entities_added: result.entities_added,
                        entities_removed: result.entities_removed,
                        entities_modified: 0, // Not tracked - would need entity matching to detect
                        edges_added: result.edges_added,
                        edges_removed: result.edges_removed,
                        hash_changed: result.hash_changed,
                        processing_time_ms: result.processing_time_ms,
                    },
                }),
            ).into_response()
        }
        Err(e) => {
            let (status_code, error_message) = match e {
                IncrementalReindexOperationError::FileNotFound(msg) => (StatusCode::NOT_FOUND, msg),
                IncrementalReindexOperationError::NotAFile(msg) => (StatusCode::BAD_REQUEST, msg),
                IncrementalReindexOperationError::FileReadError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read file: {}", msg)),
                IncrementalReindexOperationError::InvalidUtf8Error(msg) => (StatusCode::BAD_REQUEST, format!("File is not valid UTF-8: {}", msg)),
                IncrementalReindexOperationError::DatabaseNotConnected => (StatusCode::INTERNAL_SERVER_ERROR, "Database not connected".to_string()),
                IncrementalReindexOperationError::DatabaseOperationFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Database operation failed: {}", msg)),
            };
            (
                status_code,
                Json(IncrementalReindexErrorResponse {
                    success: false,
                    error: error_message,
                    endpoint,
                }),
            ).into_response()
        }
    }
}
