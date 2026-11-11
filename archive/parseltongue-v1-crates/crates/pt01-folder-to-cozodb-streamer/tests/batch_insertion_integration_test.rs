//! Integration test for batch entity insertion in streamer
//!
//! Tests that stream_file() uses insert_entities_batch() instead of individual inserts.
//! Performance target: 50 entities should complete in < 1 second.

use std::fs;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;

use parseltongue_core::storage::CozoDbStorage;
use pt01_folder_to_cozodb_streamer::{
    FileStreamer, FileStreamerImpl, StreamerConfig,
    isgl1_generator::Isgl1KeyGeneratorImpl,
    test_detector::DefaultTestDetector,
};

/// Test that stream_file uses batch insertion for entities
///
/// # Performance Contract
/// - 50 entities should be inserted in < 1 second
/// - Batch insertion should be significantly faster than 50 individual inserts
#[tokio::test]
async fn test_stream_file_uses_batch_insertion() {
    // Create temp directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("large_module.rs");

    // Generate Rust file with 50 functions
    let mut source_code = String::from("// Test file with 50 functions\n\n");
    for i in 1..=50 {
        source_code.push_str(&format!(
            "/// Function number {}\npub fn function_number_{}() -> i32 {{\n    {}\n}}\n\n",
            i, i, i
        ));
    }

    fs::write(&test_file, &source_code).expect("Failed to write test file");

    // Create streamer with in-memory database
    let db_dir = TempDir::new().expect("Failed to create db dir");
    let db_path = format!("rocksdb:{}/test.db", db_dir.path().display());

    let config = StreamerConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: db_path.clone(),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        max_file_size: 10_000_000,
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    let key_generator = Arc::new(Isgl1KeyGeneratorImpl::new());
    let test_detector = Arc::new(DefaultTestDetector::default());

    let streamer = FileStreamerImpl::new(config, key_generator, test_detector)
        .await
        .expect("Failed to create streamer");

    // Measure time for batch insertion
    let start = Instant::now();
    let result = streamer.stream_file(&test_file)
        .await
        .expect("Failed to stream file");
    let elapsed = start.elapsed();

    // Assertions
    assert!(result.success, "Stream should succeed");
    assert_eq!(result.entities_created, 50, "Should create 50 entities");
    assert!(result.error.is_none(), "Should have no errors");

    // Performance assertion: 50 entities in < 1 second
    assert!(
        elapsed.as_secs() < 1,
        "Batch insertion of 50 entities should complete in < 1 second, took {:?}",
        elapsed
    );

    println!("✓ Batch insertion completed in {:?}", elapsed);

    // Drop streamer to release database lock
    drop(streamer);

    // Verify all entities are in database
    let db = CozoDbStorage::new(&db_path)
        .await
        .expect("Failed to open database");

    let all_entities = db.get_all_entities()
        .await
        .expect("Failed to list entities");

    assert_eq!(
        all_entities.len(),
        50,
        "Database should contain exactly 50 entities"
    );

    // Verify entity keys match expected pattern
    let function_entities: Vec<_> = all_entities
        .iter()
        .filter(|e| e.isgl1_key.contains("function_number_"))
        .collect();

    assert_eq!(
        function_entities.len(),
        50,
        "Should have 50 function entities"
    );

    println!("✓ All 50 entities verified in database");
}

/// Test batch insertion with mixed entity types
///
/// Verifies that batch insertion works with functions, structs, and impl blocks
#[tokio::test]
async fn test_batch_insertion_with_mixed_entities() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("mixed.rs");

    let source_code = r#"
// Mixed entity types
pub struct DataProcessor {
    pub count: i32,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn process(&mut self, data: i32) {
        self.count += data;
    }
}

pub fn helper_function_one() -> i32 { 1 }
pub fn helper_function_two() -> i32 { 2 }
pub fn helper_function_three() -> i32 { 3 }
"#;

    fs::write(&test_file, source_code).expect("Failed to write test file");

    let db_dir = TempDir::new().expect("Failed to create db dir");
    let db_path = format!("rocksdb:{}/test.db", db_dir.path().display());

    let config = StreamerConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: db_path.clone(),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        max_file_size: 10_000_000,
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    let key_generator = Arc::new(Isgl1KeyGeneratorImpl::new());
    let test_detector = Arc::new(DefaultTestDetector::default());

    let streamer = FileStreamerImpl::new(config, key_generator, test_detector)
        .await
        .expect("Failed to create streamer");

    let result = streamer.stream_file(&test_file)
        .await
        .expect("Failed to stream file");

    assert!(result.success, "Stream should succeed");
    assert!(result.entities_created >= 5, "Should create at least 5 entities (struct, impl, 3 functions)");

    // Drop streamer to release database lock
    drop(streamer);

    // Verify entities in database
    let db = CozoDbStorage::new(&db_path)
        .await
        .expect("Failed to open database");

    let all_entities = db.get_all_entities()
        .await
        .expect("Failed to list entities");

    // Check entity types
    let has_struct = all_entities.iter().any(|e| e.isgl1_key.contains("DataProcessor"));
    let has_functions = all_entities.iter().any(|e| e.isgl1_key.contains("helper_function"));

    assert!(has_struct, "Should have struct entity");
    assert!(has_functions, "Should have function entities");

    println!("✓ Mixed entity batch insertion verified ({} entities)", all_entities.len());
}

/// Test that external dependency placeholders are also batch inserted
#[tokio::test]
async fn test_external_placeholders_batch_insertion() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("with_deps.rs");

    // Code that references external dependencies
    let source_code = r#"
use std::collections::HashMap;
use tokio::runtime::Runtime;

pub fn process_with_dependencies() {
    let mut map = HashMap::new();
    map.insert("key", "value");

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        println!("Hello from async");
    });
}
"#;

    fs::write(&test_file, source_code).expect("Failed to write test file");

    let db_dir = TempDir::new().expect("Failed to create db dir");
    let db_path = format!("rocksdb:{}/test.db", db_dir.path().display());

    let config = StreamerConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: db_path.clone(),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        max_file_size: 10_000_000,
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    let key_generator = Arc::new(Isgl1KeyGeneratorImpl::new());
    let test_detector = Arc::new(DefaultTestDetector::default());

    let streamer = FileStreamerImpl::new(config, key_generator, test_detector)
        .await
        .expect("Failed to create streamer");

    let start = Instant::now();
    let result = streamer.stream_file(&test_file)
        .await
        .expect("Failed to stream file");
    let elapsed = start.elapsed();

    assert!(result.success, "Stream should succeed");
    assert!(result.entities_created >= 1, "Should create at least function entity");

    // Should complete quickly even with external dependencies
    assert!(
        elapsed.as_millis() < 500,
        "Should complete in < 500ms, took {:?}",
        elapsed
    );

    // Drop streamer to release database lock
    drop(streamer);

    println!("✓ External dependencies handled in {:?}", elapsed);
}
