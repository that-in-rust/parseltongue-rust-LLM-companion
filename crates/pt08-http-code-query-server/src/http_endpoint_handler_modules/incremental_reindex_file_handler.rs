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
use sha2::{Sha256, Digest};
use std::path::Path;
use std::time::Instant;

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use parseltongue_core::entity_conversion::convert_parsed_to_code_entity;

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

/// Compute SHA-256 hash of file content
///
/// # 4-Word Name: compute_file_content_hash
fn compute_file_content_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    hex::encode(result)
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
/// # Algorithm
/// 1. Read file content, compute SHA-256 hash
/// 2. Compare hash against cached value (if exists)
/// 3. If unchanged, return early with hash_changed: false
/// 4. Query existing entities WHERE file_path == requested_path
/// 5. Delete those entities and their outgoing edges
/// 6. Re-parse file (not implemented in this MVP - placeholder)
/// 7. Insert new entities and edges (not implemented in this MVP)
/// 8. Update hash cache
/// 9. Return diff statistics
///
/// # URL Pattern
/// - Endpoint: POST /incremental-reindex-file-update?path=/path/to/file.rs
pub async fn handle_incremental_reindex_file_request(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<IncrementalReindexQueryParams>,
) -> impl IntoResponse {
    let start_time = Instant::now();

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

    // Read file content
    let file_content = match std::fs::read(&params.path) {
        Ok(content) => content,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(IncrementalReindexErrorResponse {
                    success: false,
                    error: format!("Failed to read file: {}", e),
                    endpoint,
                }),
            ).into_response();
        }
    };

    // Compute hash
    let current_hash = compute_file_content_hash(&file_content);

    // Get database connection
    let db_guard = state.database_storage_connection_arc.read().await;
    let storage = match db_guard.as_ref() {
        Some(s) => s,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(IncrementalReindexErrorResponse {
                    success: false,
                    error: "Database not connected".to_string(),
                    endpoint,
                }),
            ).into_response();
        }
    };

    // Ensure FileHashCache schema exists (lazy creation, ignore if already exists)
    let _ = storage.create_file_hash_cache_schema().await;

    // Check cached hash
    let cached_hash = storage.get_cached_file_hash_value(&params.path).await.ok().flatten();

    // If hash unchanged, return early
    if cached_hash.as_ref() == Some(&current_hash) {
        let processing_time_ms = start_time.elapsed().as_millis() as u64;
        return (
            StatusCode::OK,
            Json(IncrementalReindexSuccessResponse {
                success: true,
                endpoint,
                data: IncrementalReindexDataPayload {
                    file_path: params.path,
                    entities_before: 0,
                    entities_after: 0,
                    entities_added: 0,
                    entities_removed: 0,
                    entities_modified: 0,
                    edges_added: 0,
                    edges_removed: 0,
                    hash_changed: false,
                    processing_time_ms,
                },
            }),
        ).into_response();
    }

    // Get existing entities for this file
    let existing_entities = match storage.get_entities_by_file_path(&params.path).await {
        Ok(entities) => entities,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(IncrementalReindexErrorResponse {
                    success: false,
                    error: format!("Failed to query existing entities: {}", e),
                    endpoint,
                }),
            ).into_response();
        }
    };

    let entities_before = existing_entities.len();

    // Collect entity keys for deletion
    let entity_keys: Vec<String> = existing_entities
        .iter()
        .map(|e| e.isgl1_key.clone())
        .collect();

    // Delete existing edges from these entities
    let edges_removed = if !entity_keys.is_empty() {
        match storage.delete_edges_by_from_keys(&entity_keys).await {
            Ok(count) => count,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(IncrementalReindexErrorResponse {
                        success: false,
                        error: format!("Failed to delete edges: {}", e),
                        endpoint,
                    }),
                ).into_response();
            }
        }
    } else {
        0
    };

    // Delete existing entities
    let entities_removed = if !entity_keys.is_empty() {
        match storage.delete_entities_batch_by_keys(&entity_keys).await {
            Ok(count) => count,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(IncrementalReindexErrorResponse {
                        success: false,
                        error: format!("Failed to delete entities: {}", e),
                        endpoint,
                    }),
                ).into_response();
            }
        }
    } else {
        0
    };

    // PRD-2026-01-28: Re-parse file and insert new entities
    // Uses FileParser from state and entity_conversion module
    let file_content_str = match String::from_utf8(file_content.clone()) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(IncrementalReindexErrorResponse {
                    success: false,
                    error: format!("File is not valid UTF-8: {}", e),
                    endpoint,
                }),
            ).into_response();
        }
    };

    // Parse file using FileParser from state
    let (parsed_entities, dependencies) = match state
        .file_parser_instance_arc
        .parse_file_to_entities(file_path, &file_content_str)
    {
        Ok(result) => result,
        Err(e) => {
            // If parsing fails (unsupported language, etc.), just report deletion stats
            eprintln!("Warning: Failed to re-parse {}: {}", params.path, e);
            // Return with deletion-only stats
            let processing_time_ms = start_time.elapsed().as_millis() as u64;
            return (
                StatusCode::OK,
                Json(IncrementalReindexSuccessResponse {
                    success: true,
                    endpoint,
                    data: IncrementalReindexDataPayload {
                        file_path: params.path,
                        entities_before,
                        entities_after: 0,
                        entities_added: 0,
                        entities_removed,
                        entities_modified: 0,
                        edges_added: 0,
                        edges_removed,
                        hash_changed: true,
                        processing_time_ms,
                    },
                }),
            ).into_response();
        }
    };

    // Convert ParsedEntity to CodeEntity and insert
    let mut entities_added = 0usize;
    for parsed in &parsed_entities {
        match convert_parsed_to_code_entity(parsed, &file_content_str) {
            Ok(code_entity) => {
                if let Err(e) = storage.insert_entity(&code_entity).await {
                    eprintln!(
                        "Warning: Failed to insert entity '{}': {}",
                        parsed.name, e
                    );
                } else {
                    entities_added += 1;
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Entity conversion failed for '{}': {}",
                    parsed.name, e
                );
            }
        }
    }

    // Insert new edges
    let edges_added = if !dependencies.is_empty() {
        match storage.insert_edges_batch(&dependencies).await {
            Ok(()) => dependencies.len(),
            Err(e) => {
                eprintln!("Warning: Failed to insert edges: {}", e);
                0
            }
        }
    } else {
        0
    };

    let entities_after = entities_added;

    // Update hash cache
    if let Err(e) = storage.set_cached_file_hash_value(&params.path, &current_hash).await {
        // Log error but don't fail the request
        eprintln!("Warning: Failed to update hash cache: {}", e);
    }

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    (
        StatusCode::OK,
        Json(IncrementalReindexSuccessResponse {
            success: true,
            endpoint,
            data: IncrementalReindexDataPayload {
                file_path: params.path,
                entities_before,
                entities_after,
                entities_added,
                entities_removed,
                entities_modified: 0, // Not tracked in MVP
                edges_added,
                edges_removed,
                hash_changed: true,
                processing_time_ms,
            },
        }),
    ).into_response()
}
