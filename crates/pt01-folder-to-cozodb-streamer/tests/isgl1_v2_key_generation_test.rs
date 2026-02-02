//! ISGL1 v2 Key Generation Integration Tests
//!
//! Tests that pt01 uses timestamp-based keys (v2) instead of line-number based keys (v1).
//! This is THE KEY TEST for incremental indexing stability.

use pt01_folder_to_cozodb_streamer::isgl1_generator::{
    Isgl1KeyGenerator, Isgl1KeyGeneratorImpl, ParsedEntity, EntityType,
};
use parseltongue_core::entities::Language;
use std::collections::HashMap;

/// Helper: Create test entity for key generation
fn create_test_entity(
    name: &str,
    language: Language,
    file_path: &str,
    start_line: usize,
    end_line: usize,
) -> ParsedEntity {
    ParsedEntity {
        entity_type: EntityType::Function,
        name: name.to_string(),
        language,
        line_range: (start_line, end_line),
        file_path: file_path.to_string(),
        metadata: HashMap::new(),
    }
}

#[test]
fn test_format_key_uses_timestamp_not_lines() {
    // RED: This test should FAIL because current implementation uses line numbers
    //
    // Given: A parsed entity with line range 10-50
    let entity = create_test_entity("main", Language::Rust, "src/main.rs", 10, 50);

    // When: Generate key
    let generator = Isgl1KeyGeneratorImpl::new();
    let key = generator.generate_key(&entity).unwrap();

    // Then: Key should have :T prefix (v2 format)
    assert!(
        key.contains(":T"),
        "Key should use timestamp format (v2), got: {}",
        key
    );
    assert!(
        !key.contains("-"),
        "Key should not contain line range hyphen, got: {}",
        key
    );
    assert!(
        key.starts_with("rust:fn:main"),
        "Key should start with language:type:name, got: {}",
        key
    );
}

#[test]
fn test_birth_timestamp_deterministic() {
    // RED: This is THE KEY TEST - keys must be stable across line shifts
    //
    // Given: Same entity at different line positions
    let entity1 = create_test_entity("main", Language::Rust, "src/main.rs", 10, 50);
    let entity2 = create_test_entity("main", Language::Rust, "src/main.rs", 110, 150); // Shifted +100 lines!

    let generator = Isgl1KeyGeneratorImpl::new();

    // When: Generate keys for both
    let key1 = generator.generate_key(&entity1).unwrap();
    let key2 = generator.generate_key(&entity2).unwrap();

    // Then: Keys MUST be identical (same file + entity name = same birth timestamp)
    assert_eq!(
        key1, key2,
        "Keys must be stable across line shifts!\n  Key at line 10: {}\n  Key at line 110: {}",
        key1, key2
    );
}

#[test]
fn test_different_files_different_keys() {
    // Given: Same entity name in different files
    let entity1 = create_test_entity("main", Language::Rust, "src/main.rs", 10, 50);
    let entity2 = create_test_entity("main", Language::Rust, "src/other.rs", 10, 50);

    let generator = Isgl1KeyGeneratorImpl::new();

    // When: Generate keys
    let key1 = generator.generate_key(&entity1).unwrap();
    let key2 = generator.generate_key(&entity2).unwrap();

    // Then: Keys should be different (different file path = different birth timestamp)
    assert_ne!(
        key1, key2,
        "Different files should have different keys"
    );
}

#[test]
fn test_different_entities_same_file_different_keys() {
    // Given: Different entities in same file
    let entity1 = create_test_entity("alpha", Language::Rust, "src/main.rs", 10, 50);
    let entity2 = create_test_entity("beta", Language::Rust, "src/main.rs", 60, 100);

    let generator = Isgl1KeyGeneratorImpl::new();

    // When: Generate keys
    let key1 = generator.generate_key(&entity1).unwrap();
    let key2 = generator.generate_key(&entity2).unwrap();

    // Then: Keys should be different (different entity name = different birth timestamp)
    assert_ne!(
        key1, key2,
        "Different entities should have different keys"
    );
}

#[test]
fn test_key_format_structure() {
    // Given: A test entity
    let entity = create_test_entity("handle_request", Language::Python, "api/server.py", 42, 99);

    let generator = Isgl1KeyGeneratorImpl::new();

    // When: Generate key
    let key = generator.generate_key(&entity).unwrap();

    // Then: Key should match v2 format: language:type:name:semantic_path:Ttimestamp
    let parts: Vec<&str> = key.split(':').collect();
    assert_eq!(parts.len(), 5, "Key should have 5 parts, got: {}", key);
    assert_eq!(parts[0], "python", "First part should be language");
    assert_eq!(parts[1], "fn", "Second part should be entity type");
    assert_eq!(parts[2], "handle_request", "Third part should be entity name");
    assert!(parts[3].starts_with("__"), "Fourth part should be semantic path (starts with __)");
    assert!(parts[4].starts_with("T"), "Fifth part should be timestamp (starts with T)");

    // Timestamp should be numeric after 'T'
    let timestamp_str = &parts[4][1..]; // Remove 'T' prefix
    let timestamp: i64 = timestamp_str.parse().expect("Timestamp should be numeric");
    assert!(timestamp > 1577836800, "Timestamp should be in valid range (2020+)");
}
