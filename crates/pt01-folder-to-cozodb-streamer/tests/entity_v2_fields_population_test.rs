//! ISGL1 v2 Entity Field Population Tests
//!
//! Tests that CodeEntity instances are populated with v2 fields:
//! - birth_timestamp (deterministic from file + name)
//! - content_hash (SHA-256 of entity code)
//! - semantic_path (sanitized file path)

use pt01_folder_to_cozodb_streamer::streamer::{FileStreamerImpl, FileStreamer};
use pt01_folder_to_cozodb_streamer::StreamerConfig;
use pt01_folder_to_cozodb_streamer::isgl1_generator::Isgl1KeyGeneratorFactory;
use pt01_folder_to_cozodb_streamer::test_detector::DefaultTestDetector;
use parseltongue_core::storage::CozoDbStorage;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio;

/// Helper: Create test streamer with temporary database
async fn create_test_streamer() -> (FileStreamerImpl, TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = format!("rocksdb:{}", db_path.display());

    let config = StreamerConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: db_path_str.clone(),
        max_file_size: 10 * 1024 * 1024, // 10MB
        include_patterns: vec!["**/*.rs".to_string()],
        exclude_patterns: vec![],
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    let key_generator = Isgl1KeyGeneratorFactory::new();
    let test_detector = Arc::new(DefaultTestDetector::new());

    let streamer = FileStreamerImpl::new(config, key_generator, test_detector)
        .await
        .unwrap();

    (streamer, temp_dir, db_path_str)
}

/// Helper: Create a test Rust file
async fn create_test_rust_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
    let file_path = dir.join(name);
    tokio::fs::write(&file_path, content).await.unwrap();
    file_path
}

#[tokio::test]
async fn test_entity_has_v2_fields_populated() {
    // RED: This test should FAIL because v2 fields are currently None
    //
    // Given: A Rust file with a function
    let (streamer, temp_dir, db_path_str) = create_test_streamer().await;

    let rust_code = r#"
fn test_main() {
    println!("Hello, world!");
}
"#;

    let file_path = create_test_rust_file(temp_dir.path(), "main.rs", rust_code).await;

    // When: Stream the file (parse and convert to CodeEntity)
    let result = streamer.stream_file(&file_path).await.unwrap();

    // Then: Result should contain 1 entity with v2 fields populated
    assert_eq!(result.entities_created, 1, "Should create 1 entity");

    // Drop streamer to release database lock
    drop(streamer);

    // Query the database to get the entity
    let storage = CozoDbStorage::new(&db_path_str).await.unwrap();
    let all_entities: Vec<parseltongue_core::entities::CodeEntity> = storage.get_all_entities().await.unwrap();

    assert_eq!(all_entities.len(), 1, "Database should have 1 entity");
    let entity = &all_entities[0];

    // v2 fields must be populated (currently fails - they are None)
    assert!(
        entity.birth_timestamp.is_some(),
        "birth_timestamp must be populated, got None"
    );
    assert!(
        entity.content_hash.is_some(),
        "content_hash must be populated, got None"
    );
    assert!(
        entity.semantic_path.is_some(),
        "semantic_path must be populated, got None"
    );

    // Validate field values
    let birth_timestamp = entity.birth_timestamp.unwrap();
    assert!(
        birth_timestamp > 1577836800,
        "Timestamp should be in valid range (2020+), got {}",
        birth_timestamp
    );

    let content_hash = entity.content_hash.as_ref().unwrap();
    assert_eq!(
        content_hash.len(),
        64,
        "SHA-256 hash should be 64 hex chars, got {}",
        content_hash.len()
    );
    assert!(
        content_hash.chars().all(|c| c.is_ascii_hexdigit()),
        "Hash should be hex, got {}",
        content_hash
    );

    let semantic_path = entity.semantic_path.as_ref().unwrap();
    assert!(
        semantic_path.starts_with("__"),
        "Semantic path should start with __, got {}",
        semantic_path
    );
    assert!(
        semantic_path.contains("main"),
        "Semantic path should contain 'main', got {}",
        semantic_path
    );
}

#[tokio::test]
async fn test_content_hash_reflects_code_changes() {
    // Given: Two files with different code
    let (streamer, temp_dir, db_path_str) = create_test_streamer().await;

    let code1 = "fn alpha() { println!(\"v1\"); }";
    let code2 = "fn alpha() { println!(\"v2\"); }"; // Different content!

    let file1 = create_test_rust_file(temp_dir.path(), "file1.rs", code1).await;
    let file2 = create_test_rust_file(temp_dir.path(), "file2.rs", code2).await;

    // When: Stream both files
    streamer.stream_file(&file1).await.unwrap();
    streamer.stream_file(&file2).await.unwrap();

    // Drop streamer to release database lock
    drop(streamer);

    // Then: Entities should have different content hashes
    let storage = CozoDbStorage::new(&db_path_str).await.unwrap();
    let all_entities: Vec<parseltongue_core::entities::CodeEntity> = storage.get_all_entities().await.unwrap();

    assert_eq!(all_entities.len(), 2);

    let hash1 = all_entities[0].content_hash.as_ref().unwrap();
    let hash2 = all_entities[1].content_hash.as_ref().unwrap();

    assert_ne!(
        hash1, hash2,
        "Different code should produce different hashes"
    );
}

#[tokio::test]
async fn test_birth_timestamp_deterministic_for_same_entity() {
    // Given: Same entity name in same file (simulated by creating file twice)
    let (streamer, temp_dir, db_path_str) = create_test_streamer().await;

    let code = "fn deterministic_func() { }";
    let file_path = create_test_rust_file(temp_dir.path(), "test.rs", code).await;

    // When: Stream the same file (simulate re-indexing)
    streamer.stream_file(&file_path).await.unwrap();

    // Drop streamer to release database lock
    drop(streamer);

    // Then: Entity should have consistent birth timestamp
    let storage = CozoDbStorage::new(&db_path_str).await.unwrap();
    let entities: Vec<parseltongue_core::entities::CodeEntity> = storage.get_all_entities().await.unwrap();

    assert_eq!(entities.len(), 1);
    let ts1 = entities[0].birth_timestamp.unwrap();

    // Expected: Same file + entity name = same birth timestamp
    // (This is deterministic based on hash(file_path + entity_name))
    use parseltongue_core::isgl1_v2::compute_birth_timestamp;
    let file_path_str = file_path.to_string_lossy();
    let expected_ts = compute_birth_timestamp(&file_path_str, "deterministic_func");

    assert_eq!(
        ts1, expected_ts,
        "Birth timestamp should be deterministic"
    );
}

#[tokio::test]
async fn test_semantic_path_sanitization() {
    // Given: File with complex path
    let (streamer, temp_dir, db_path_str) = create_test_streamer().await;

    let code = "fn test_func() {}";
    let subdir = temp_dir.path().join("crates").join("core");
    tokio::fs::create_dir_all(&subdir).await.unwrap();

    let file_path = subdir.join("lib.rs");
    tokio::fs::write(&file_path, code).await.unwrap();

    // When: Stream the file
    streamer.stream_file(&file_path).await.unwrap();

    // Drop streamer to release database lock
    drop(streamer);

    // Then: Semantic path should be sanitized (no slashes, no extension)
    let storage = CozoDbStorage::new(&db_path_str).await.unwrap();
    let entities: Vec<parseltongue_core::entities::CodeEntity> = storage.get_all_entities().await.unwrap();

    assert_eq!(entities.len(), 1);
    let semantic_path = entities[0].semantic_path.as_ref().unwrap();

    // Should have underscores instead of slashes, no .rs extension
    assert!(semantic_path.starts_with("__"), "Should start with __");
    assert!(!semantic_path.contains('/'), "Should not contain /");
    assert!(!semantic_path.contains('\\'), "Should not contain \\");
    assert!(!semantic_path.ends_with(".rs"), "Should not have extension");
    assert!(semantic_path.contains("crates"), "Should contain 'crates'");
    assert!(semantic_path.contains("core"), "Should contain 'core'");
    assert!(semantic_path.contains("lib"), "Should contain 'lib'");
}
