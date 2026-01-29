//! Incremental reindex core logic module
//!
//! # 4-Word Naming: incremental_reindex_core_logic
//!
//! PRD-2026-01-28: Extracted core reindex logic for reuse by:
//! - HTTP handler (POST /incremental-reindex-file-update)
//! - File watcher callback (automatic reindex on file change)
//!
//! ## S06 Compliance
//! - All function names are exactly 4 words
//! - Trait-based dependency injection via SharedApplicationStateContainer
//! - RAII resource management (no manual cleanup needed)
//!
//! ## Acceptance Criteria (WHEN...THEN...SHALL)
//!
//! 1. WHEN execute_incremental_reindex_core is called with valid path
//!    THEN the system SHALL reindex the file and return diff statistics
//!
//! 2. WHEN file content is unchanged (same hash)
//!    THEN the system SHALL return early with hash_changed: false
//!
//! 3. WHEN file parsing fails
//!    THEN the system SHALL delete old entities and return deletion stats

use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use parseltongue_core::entity_conversion::convert_parsed_to_code_entity;
use parseltongue_core::storage::CozoDbStorage;

/// Error types for incremental reindex operations
///
/// # 4-Word Name: IncrementalReindexOperationError
#[derive(Error, Debug)]
pub enum IncrementalReindexOperationError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Path is not a file: {0}")]
    NotAFile(String),

    #[error("Failed to read file: {0}")]
    FileReadError(String),

    #[error("File is not valid UTF-8: {0}")]
    InvalidUtf8Error(String),

    #[error("Database not connected")]
    DatabaseNotConnected,

    #[error("Database operation failed: {0}")]
    DatabaseOperationFailed(String),
}

/// Result type for incremental reindex operations
pub type ReindexResult<T> = Result<T, IncrementalReindexOperationError>;

/// Result data from incremental reindex operation
///
/// # 4-Word Name: IncrementalReindexResultData
#[derive(Debug, Clone)]
pub struct IncrementalReindexResultData {
    pub file_path: String,
    pub entities_before: usize,
    pub entities_after: usize,
    pub entities_added: usize,
    pub entities_removed: usize,
    pub edges_added: usize,
    pub edges_removed: usize,
    pub hash_changed: bool,
    pub processing_time_ms: u64,
}

/// Compute SHA-256 hash of content
///
/// # 4-Word Name: compute_content_hash_sha256
pub fn compute_content_hash_sha256(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Execute core incremental reindex logic
///
/// # 4-Word Name: execute_incremental_reindex_core
///
/// This is the shared logic used by both:
/// - HTTP handler (handle_incremental_reindex_file_request)
/// - File watcher callback (trigger_reindex_for_file)
///
/// # Contract
/// - Precondition: Database connected with CodeGraph and DependencyEdges schemas
/// - Precondition: File path exists and is a regular file
/// - Postcondition: Returns diff statistics of what changed
/// - Performance: <500ms for typical file with <100 entities
///
/// # Algorithm
/// 1. Read file content, compute SHA-256 hash
/// 2. Compare hash against cached value (if exists)
/// 3. If unchanged, return early with hash_changed: false
/// 4. Delete existing entities and their outgoing edges
/// 5. Re-parse file using FileParser
/// 6. Insert new entities and edges
/// 7. Update hash cache
/// 8. Return diff statistics
pub async fn execute_incremental_reindex_core(
    file_path_string: &str,
    state: &SharedApplicationStateContainer,
) -> ReindexResult<IncrementalReindexResultData> {
    let start_time = Instant::now();
    let file_path = Path::new(file_path_string);

    // Validate file exists
    if !file_path.exists() {
        return Err(IncrementalReindexOperationError::FileNotFound(
            file_path_string.to_string(),
        ));
    }

    // Validate it's a file (not directory)
    if !file_path.is_file() {
        return Err(IncrementalReindexOperationError::NotAFile(
            file_path_string.to_string(),
        ));
    }

    // Read file content
    let file_content = std::fs::read(file_path_string).map_err(|e| {
        IncrementalReindexOperationError::FileReadError(e.to_string())
    })?;

    // Compute hash
    let current_hash = compute_content_hash_sha256(&file_content);

    // Get database connection
    let db_guard = state.database_storage_connection_arc.read().await;
    let storage = db_guard.as_ref().ok_or(
        IncrementalReindexOperationError::DatabaseNotConnected,
    )?;

    // Ensure FileHashCache schema exists
    let _ = storage.create_file_hash_cache_schema().await;

    // Check cached hash - early return if unchanged
    let cached_hash = storage
        .get_cached_file_hash_value(file_path_string)
        .await
        .ok()
        .flatten();

    if cached_hash.as_ref() == Some(&current_hash) {
        let processing_time_ms = start_time.elapsed().as_millis() as u64;
        return Ok(IncrementalReindexResultData {
            file_path: file_path_string.to_string(),
            entities_before: 0,
            entities_after: 0,
            entities_added: 0,
            entities_removed: 0,
            edges_added: 0,
            edges_removed: 0,
            hash_changed: false,
            processing_time_ms,
        });
    }

    // Execute reindex with storage
    execute_reindex_with_storage_arc(
        file_path_string,
        &file_content,
        &current_hash,
        storage,
        state,
        start_time,
    )
    .await
}

/// Execute reindex with storage reference
///
/// # 4-Word Name: execute_reindex_with_storage_arc
///
/// Internal helper that performs the actual reindex once we have storage.
async fn execute_reindex_with_storage_arc(
    file_path_string: &str,
    file_content: &[u8],
    current_hash: &str,
    storage: &Arc<CozoDbStorage>,
    state: &SharedApplicationStateContainer,
    start_time: Instant,
) -> ReindexResult<IncrementalReindexResultData> {
    let file_path = Path::new(file_path_string);

    // Get existing entities for this file
    let existing_entities = storage
        .get_entities_by_file_path(file_path_string)
        .await
        .map_err(|e| {
            IncrementalReindexOperationError::DatabaseOperationFailed(e.to_string())
        })?;

    let entities_before = existing_entities.len();

    // Collect entity keys for deletion
    let entity_keys: Vec<String> = existing_entities
        .iter()
        .map(|e| e.isgl1_key.clone())
        .collect();

    // Delete existing edges from these entities
    let edges_removed = if !entity_keys.is_empty() {
        storage
            .delete_edges_by_from_keys(&entity_keys)
            .await
            .map_err(|e| {
                IncrementalReindexOperationError::DatabaseOperationFailed(e.to_string())
            })?
    } else {
        0
    };

    // Delete existing entities
    let entities_removed = if !entity_keys.is_empty() {
        storage
            .delete_entities_batch_by_keys(&entity_keys)
            .await
            .map_err(|e| {
                IncrementalReindexOperationError::DatabaseOperationFailed(e.to_string())
            })?
    } else {
        0
    };

    // Convert to UTF-8 for parsing
    let file_content_str = String::from_utf8(file_content.to_vec()).map_err(|e| {
        IncrementalReindexOperationError::InvalidUtf8Error(e.to_string())
    })?;

    // Parse file using FileParser from state
    let (parsed_entities, dependencies) = match state
        .file_parser_instance_arc
        .parse_file_to_entities(file_path, &file_content_str)
    {
        Ok(result) => result,
        Err(e) => {
            // If parsing fails, return deletion-only stats
            eprintln!(
                "[ReindexCore] Warning: Failed to parse {}: {}",
                file_path_string, e
            );
            let processing_time_ms = start_time.elapsed().as_millis() as u64;
            return Ok(IncrementalReindexResultData {
                file_path: file_path_string.to_string(),
                entities_before,
                entities_after: 0,
                entities_added: 0,
                entities_removed,
                edges_added: 0,
                edges_removed,
                hash_changed: true,
                processing_time_ms,
            });
        }
    };

    // Convert ParsedEntity to CodeEntity and insert
    let mut entities_added = 0usize;
    for parsed in &parsed_entities {
        match convert_parsed_to_code_entity(parsed, &file_content_str) {
            Ok(code_entity) => {
                if let Err(e) = storage.insert_entity(&code_entity).await {
                    eprintln!(
                        "[ReindexCore] Warning: Failed to insert entity '{}': {}",
                        parsed.name, e
                    );
                } else {
                    entities_added += 1;
                }
            }
            Err(e) => {
                eprintln!(
                    "[ReindexCore] Warning: Entity conversion failed for '{}': {}",
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
                eprintln!("[ReindexCore] Warning: Failed to insert edges: {}", e);
                0
            }
        }
    } else {
        0
    };

    let entities_after = entities_added;

    // Update hash cache
    if let Err(e) = storage
        .set_cached_file_hash_value(file_path_string, current_hash)
        .await
    {
        eprintln!("[ReindexCore] Warning: Failed to update hash cache: {}", e);
    }

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(IncrementalReindexResultData {
        file_path: file_path_string.to_string(),
        entities_before,
        entities_after,
        entities_added,
        entities_removed,
        edges_added,
        edges_removed,
        hash_changed: true,
        processing_time_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_content_hash_sha256() {
        // GIVEN: Known content
        let content = b"fn main() {}";

        // WHEN: Computing hash
        let hash = compute_content_hash_sha256(content);

        // THEN: Hash should be 64 hex chars (SHA-256)
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_same_content_same_hash() {
        // GIVEN: Same content twice
        let content1 = b"fn foo() { bar(); }";
        let content2 = b"fn foo() { bar(); }";

        // WHEN: Computing hashes
        let hash1 = compute_content_hash_sha256(content1);
        let hash2 = compute_content_hash_sha256(content2);

        // THEN: Hashes should be equal
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_content_different_hash() {
        // GIVEN: Different content
        let content1 = b"fn foo() {}";
        let content2 = b"fn bar() {}";

        // WHEN: Computing hashes
        let hash1 = compute_content_hash_sha256(content1);
        let hash2 = compute_content_hash_sha256(content2);

        // THEN: Hashes should differ
        assert_ne!(hash1, hash2);
    }
}
