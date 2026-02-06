// ISGL1 v2: Stable Entity Identity with Birth Timestamps
//
// Old format: rust:fn:handle_auth:__src_auth_rs:10-50 (line-number based)
// New format: rust:fn:handle_auth:__src_auth_rs:T1706284800 (timestamp based)
//
// Birth timestamp ensures keys remain stable when line numbers change.
// This solves the incremental indexing false positive problem.

use crate::entities::EntityType;
use sha2::{Digest, Sha256};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Format ISGL1 v2 key with birth timestamp
///
/// # Arguments
/// * `entity_type` - Type of entity (Function, Class, etc.)
/// * `name` - Entity name
/// * `language` - Programming language
/// * `semantic_path` - Sanitized file path (use extract_semantic_path)
/// * `birth_timestamp` - Unix epoch timestamp (use compute_birth_timestamp)
///
/// # Returns
/// Key in format: `rust:fn:handle_auth:__src_auth_rs:T1706284800`
pub fn format_key_v2(
    entity_type: EntityType,
    name: &str,
    language: &str,
    semantic_path: &str,
    birth_timestamp: i64,
) -> String {
    let type_str = match entity_type {
        EntityType::Function => "fn",
        EntityType::Method => "method",
        EntityType::Class => "class",
        EntityType::Struct => "struct",
        EntityType::Interface => "interface",
        EntityType::Enum => "enum",
        EntityType::Trait => "trait",
        EntityType::Module => "module",
        EntityType::ImplBlock { .. } => "impl",
        EntityType::Macro => "macro",
        EntityType::ProcMacro => "proc_macro",
        EntityType::TestFunction => "test",
        EntityType::Variable => "var",
        EntityType::Constant => "const",
    };

    format!("{}:{}:{}:{}:T{}", language, type_str, name, semantic_path, birth_timestamp)
}

/// Extract semantic path from file path
///
/// Removes file extension and sanitizes path for use in ISGL1 keys.
///
/// # Example
/// ```
/// use parseltongue_core::isgl1_v2::extract_semantic_path;
/// assert_eq!(extract_semantic_path("src/auth.rs"), "__src_auth");
/// assert_eq!(extract_semantic_path("crates/core/lib.py"), "__crates_core_lib");
/// ```
pub fn extract_semantic_path(file_path: &str) -> String {
    // Remove file extension
    let without_ext = if let Some(pos) = file_path.rfind('.') {
        &file_path[..pos]
    } else {
        file_path
    };

    // Replace path separators and special chars with underscore
    let sanitized = without_ext
        .replace(['/', '\\', '-', '.'], "_");

    // Add leading underscores (ISGL1 convention)
    format!("__{}", sanitized)
}

/// Compute birth timestamp for entity
///
/// Uses deterministic hash of file_path + entity_name to generate
/// a stable timestamp. Same entity = same timestamp across runs.
///
/// # Arguments
/// * `file_path` - File containing the entity
/// * `entity_name` - Name of the entity
///
/// # Returns
/// Deterministic Unix timestamp (seconds since epoch)
///
/// # Example
/// ```
/// use parseltongue_core::isgl1_v2::compute_birth_timestamp;
/// let ts1 = compute_birth_timestamp("src/main.rs", "main");
/// let ts2 = compute_birth_timestamp("src/main.rs", "main");
/// assert_eq!(ts1, ts2); // Deterministic
/// ```
pub fn compute_birth_timestamp(file_path: &str, entity_name: &str) -> i64 {
    // Create deterministic hash from file + entity name
    let mut hasher = DefaultHasher::new();
    file_path.hash(&mut hasher);
    entity_name.hash(&mut hasher);
    let hash = hasher.finish();

    // Convert hash to reasonable timestamp range
    // Use modulo to keep it within recent years (2020-2030 range)
    let base_timestamp = 1577836800; // 2020-01-01 00:00:00 UTC
    let range = 315360000; // ~10 years in seconds
    let offset = (hash % range as u64) as i64;

    base_timestamp + offset
}

/// Compute SHA-256 content hash for entity code
///
/// Returns hex-encoded SHA-256 hash (64 characters) for change detection.
/// Whitespace-sensitive to detect formatting changes.
///
/// This enables incremental indexing: compare hashes to detect if entity code
/// has changed since last indexing.
///
/// # Arguments
/// * `code` - The source code text to hash
///
/// # Returns
/// Lowercase hex-encoded SHA-256 hash (64 characters)
///
/// # Example
/// ```
/// use parseltongue_core::isgl1_v2::compute_content_hash;
/// let code = "fn main() {}";
/// let hash = compute_content_hash(code);
/// assert_eq!(hash.len(), 64);
/// assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
/// ```
pub fn compute_content_hash(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Entity candidate from new codebase scan
///
/// Represents an entity detected during incremental reindexing
/// that needs to be matched against the old index.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityCandidate {
    pub name: String,
    pub entity_type: EntityType,
    pub file_path: String,
    pub line_range: (usize, usize),
    pub content_hash: String,
    pub code: String,
}

/// Entity from previous index
///
/// Minimal representation of an existing entity from the old index,
/// used for matching during incremental reindexing.
#[derive(Debug, Clone, PartialEq)]
pub struct OldEntity {
    pub key: String,              // ISGL1 v2 key with birth timestamp
    pub name: String,
    pub file_path: String,
    pub line_range: (usize, usize),
    pub content_hash: String,
}

/// Result of matching entity with old index
///
/// Three-priority matching strategy:
/// 1. ContentMatch: Same hash → identical entity (preserve key)
/// 2. PositionMatch: Same name/file/approx position → likely same entity (preserve key)
/// 3. NewEntity: Not found in old index → assign new birth timestamp
#[derive(Debug, Clone, PartialEq)]
pub enum EntityMatchResult {
    /// Priority 1: Hash matched (content unchanged)
    ContentMatch { old_key: String },
    /// Priority 2: Name/position matched (content changed)
    PositionMatch { old_key: String },
    /// Priority 3: New entity (will get new birth timestamp)
    NewEntity,
}

/// Position tolerance for entity matching (lines)
///
/// Entities within ±10 lines of each other are considered at "approximately same position"
/// for incremental reindexing purposes. This handles nearby edits without false positives.
pub const POSITION_TOLERANCE_LINES: i32 = 10;

/// Check if two line ranges are within position tolerance
///
/// Compares start lines to determine if entities are at approximately the same position.
///
/// # Arguments
/// * `old_range` - Line range (start, end) from old entity
/// * `new_range` - Line range (start, end) from new entity
/// * `tolerance` - Maximum allowed line difference
///
/// # Returns
/// `true` if start lines differ by at most `tolerance`
#[inline]
fn is_within_position_tolerance(
    old_range: (usize, usize),
    new_range: (usize, usize),
    tolerance: i32,
) -> bool {
    let old_start = old_range.0 as i32;
    let new_start = new_range.0 as i32;
    (old_start - new_start).abs() <= tolerance
}

/// Match entity candidate with old index
///
/// Implements 3-priority matching algorithm for incremental reindexing:
/// 1. **Priority 1 (Hash Match)**: Content hash + name + file match → ContentMatch
///    - Most reliable: identical code = same entity
///    - Early return optimization: stops at first match
/// 2. **Priority 2 (Position Match)**: Name + file + approx position match → PositionMatch
///    - Handles code changes while preserving identity
///    - Uses ±10 line tolerance for nearby edits
/// 3. **Priority 3 (New Entity)**: No match found → NewEntity
///    - Will receive new birth timestamp
///
/// # Arguments
/// * `new_entity` - Entity candidate from new codebase scan
/// * `old_entities` - Entities from previous index
///
/// # Returns
/// Match result indicating whether to preserve old key or assign new one
///
/// # Performance
/// - Best case: O(1) hash match at start of list
/// - Average case: O(n) where n = number of old entities
/// - Worst case: O(2n) if no hash match and position match at end
pub fn match_entity_with_old_index(
    new_entity: &EntityCandidate,
    old_entities: &[OldEntity],
) -> EntityMatchResult {
    // Priority 1: Try hash match first (most reliable)
    // Check content hash + name + file to ensure same entity
    if let Some(matched) = old_entities.iter().find(|old| {
        old.content_hash == new_entity.content_hash
            && old.name == new_entity.name
            && old.file_path == new_entity.file_path
    }) {
        return EntityMatchResult::ContentMatch {
            old_key: matched.key.clone(),
        };
    }

    // Priority 2: Try position match (same name, file, approximate position)
    // Only executed if hash match fails (content changed)
    if let Some(matched) = old_entities.iter().find(|old| {
        old.name == new_entity.name
            && old.file_path == new_entity.file_path
            && is_within_position_tolerance(
                old.line_range,
                new_entity.line_range,
                POSITION_TOLERANCE_LINES,
            )
    }) {
        return EntityMatchResult::PositionMatch {
            old_key: matched.key.clone(),
        };
    }

    // Priority 3: No match found - treat as new entity
    EntityMatchResult::NewEntity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_key_basic() {
        let key = format_key_v2(
            EntityType::Function,
            "test_func",
            "rust",
            "__src_test",
            1706284800
        );
        assert_eq!(key, "rust:fn:test_func:__src_test:T1706284800");
    }

    #[test]
    fn test_extract_semantic_path_basic() {
        assert_eq!(extract_semantic_path("src/main.rs"), "__src_main");
        assert_eq!(extract_semantic_path("lib.py"), "__lib");
    }

    #[test]
    fn test_compute_birth_timestamp_deterministic() {
        let ts1 = compute_birth_timestamp("test.rs", "func");
        let ts2 = compute_birth_timestamp("test.rs", "func");
        assert_eq!(ts1, ts2);
    }
}
