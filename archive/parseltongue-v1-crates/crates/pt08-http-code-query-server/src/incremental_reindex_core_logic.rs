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
use std::collections::HashSet;

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use parseltongue_core::storage::CozoDbStorage;
use parseltongue_core::isgl1_v2::{
    EntityCandidate, OldEntity, EntityMatchResult, match_entity_with_old_index,
    compute_birth_timestamp, compute_content_hash, extract_semantic_path, format_key_v2,
};
use parseltongue_core::entities::{CodeEntity, InterfaceSignature, LineRange, Visibility, EntityClass as CoreEntityClass, Language, LanguageSpecificSignature, EntityType as CoreEntityType, RustSignature};
use pt01_folder_to_cozodb_streamer::isgl1_generator::Isgl1KeyGeneratorFactory;
use std::path::PathBuf;

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

/// Convert Language enum to lowercase string for ISGL1 key
fn language_to_string_format(lang: &Language) -> String {
    format!("{:?}", lang).to_lowercase()
}

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
    _state: &SharedApplicationStateContainer,
    start_time: Instant,
) -> ReindexResult<IncrementalReindexResultData> {
    let file_path = Path::new(file_path_string);

    // Convert to UTF-8 for parsing (do this early for matching)
    let file_content_str = String::from_utf8(file_content.to_vec()).map_err(|e| {
        IncrementalReindexOperationError::InvalidUtf8Error(e.to_string())
    })?;

    // Get existing entities for this file
    let existing_entities = storage
        .get_entities_by_file_path(file_path_string)
        .await
        .map_err(|e| {
            IncrementalReindexOperationError::DatabaseOperationFailed(e.to_string())
        })?;

    let entities_before = existing_entities.len();

    // Parse file using pt01's Isgl1KeyGenerator
    let key_generator = Isgl1KeyGeneratorFactory::new();
    let (parsed_entities, dependencies, _warnings) = match key_generator.parse_source(&file_content_str, file_path) {
        Ok(result) => result,
        Err(e) => {
            // If parsing fails, delete all old entities and return
            eprintln!(
                "[ReindexCore] Warning: Failed to parse {}: {}",
                file_path_string, e
            );

            let entity_keys: Vec<String> = existing_entities
                .iter()
                .map(|e| e.isgl1_key.clone())
                .collect();

            let edges_removed = if !entity_keys.is_empty() {
                storage.delete_edges_by_from_keys(&entity_keys).await.unwrap_or(0)
            } else {
                0
            };

            let entities_removed = if !entity_keys.is_empty() {
                storage.delete_entities_batch_by_keys(&entity_keys).await.unwrap_or(0)
            } else {
                0
            };

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

    // ISGL1 v2: Convert existing entities to OldEntity format for matching
    let old_entities: Vec<OldEntity> = existing_entities
        .iter()
        .filter_map(|e| {
            Some(OldEntity {
                key: e.isgl1_key.clone(),
                name: e.interface_signature.name.clone(),
                file_path: e.interface_signature.file_path.to_string_lossy().to_string(),
                line_range: (
                    e.interface_signature.line_range.start as usize,
                    e.interface_signature.line_range.end as usize,
                ),
                content_hash: e.content_hash.clone()?,
            })
        })
        .collect();

    // ISGL1 v2: Match new entities against old index
    let mut matched_keys = HashSet::new();
    let mut new_entity_keys = HashSet::new();  // Track NEW entities separately
    let mut entities_to_upsert = Vec::new();

    for parsed in &parsed_entities {
        // Extract code snippet for this entity
        let code_snippet = file_content_str
            .lines()
            .skip(parsed.line_range.0.saturating_sub(1))
            .take(parsed.line_range.1 - parsed.line_range.0 + 1)
            .collect::<Vec<_>>()
            .join("\n");

        // Create candidate (MVP: simplified to Function entity type)
        let candidate = EntityCandidate {
            name: parsed.name.clone(),
            entity_type: CoreEntityType::Function,  // Simplified for MVP
            file_path: file_path_string.to_string(),
            line_range: parsed.line_range,
            content_hash: compute_content_hash(&code_snippet),
            code: code_snippet.clone(),
        };

        // Match against old entities
        let match_result = match_entity_with_old_index(&candidate, &old_entities);

        // Determine ISGL1 key based on match result
        let isgl1_key = match match_result {
            EntityMatchResult::ContentMatch { old_key } | EntityMatchResult::PositionMatch { old_key } => {
                matched_keys.insert(old_key.clone());
                old_key
            }
            EntityMatchResult::NewEntity => {
                // Assign new birth timestamp and create ISGL1 v2 key
                let birth_timestamp = compute_birth_timestamp(file_path_string, &parsed.name);
                let semantic_path = extract_semantic_path(file_path_string);
                let language_str = language_to_string_format(&parsed.language);
                let new_key = format_key_v2(
                    CoreEntityType::Function,  // Simplified for MVP
                    &parsed.name,
                    &language_str,
                    &semantic_path,
                    birth_timestamp,
                );
                new_entity_keys.insert(new_key.clone());  // Track as NEW
                new_key
            }
        };

        // Convert to CodeEntity using the determined key
        // Create InterfaceSignature with simplified LanguageSpecificSignature (MVP: use empty RustSignature)
        let interface_signature = InterfaceSignature {
            entity_type: parseltongue_core::entities::EntityType::Function, // Simplified - real impl would map entity_type
            name: parsed.name.clone(),
            visibility: Visibility::Public,
            file_path: PathBuf::from(&parsed.file_path),
            line_range: LineRange::new(parsed.line_range.0 as u32, parsed.line_range.1 as u32)
                .unwrap_or_else(|_| LineRange { start: 0, end: 0 }),
            module_path: vec![],  // Simplified for incremental reindex
            documentation: None,
            language_specific: LanguageSpecificSignature::Rust(RustSignature {
                generics: vec![],
                lifetimes: vec![],
                where_clauses: vec![],
                attributes: vec![],
                trait_impl: None,
            }),  // Simplified for MVP - TODO: map parsed.language properly
        };

        // Create CodeEntity
        let mut code_entity = CodeEntity::new(
            isgl1_key,
            interface_signature,
            CoreEntityClass::CodeImplementation, // Simplified - real impl would detect tests
        ).unwrap();

        // Populate ISGL1 v2 fields
        code_entity.birth_timestamp = Some(compute_birth_timestamp(file_path_string, &parsed.name));
        code_entity.content_hash = Some(compute_content_hash(&code_snippet));
        code_entity.semantic_path = Some(extract_semantic_path(file_path_string));
        code_entity.current_code = Some(code_snippet.clone());
        code_entity.future_code = Some(code_snippet);

        entities_to_upsert.push(code_entity);
    }

    // Delete unmatched old entities (those that no longer exist in file)
    let unmatched_keys: Vec<String> = existing_entities
        .iter()
        .filter(|e| !matched_keys.contains(&e.isgl1_key))
        .map(|e| e.isgl1_key.clone())
        .collect();

    let edges_removed = if !unmatched_keys.is_empty() {
        storage
            .delete_edges_by_from_keys(&unmatched_keys)
            .await
            .unwrap_or(0)
    } else {
        0
    };

    let entities_removed = if !unmatched_keys.is_empty() {
        storage
            .delete_entities_batch_by_keys(&unmatched_keys)
            .await
            .unwrap_or(0)
    } else {
        0
    };

    // Insert/update all entities (CozoDB :put handles upsert)
    for code_entity in &entities_to_upsert {
        if let Err(e) = storage.insert_entity(code_entity).await {
            eprintln!(
                "[ReindexCore] Warning: Failed to upsert entity '{}': {}",
                code_entity.interface_signature.name, e
            );
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

    // Calculate statistics
    let entities_added = new_entity_keys.len();
    let entities_after = entities_before - entities_removed + entities_added;

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
