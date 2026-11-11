//! TDD Classification Tests
//!
//! Executable specification: Tool 1 MUST correctly classify test vs code entities

use pt01_folder_to_cozodb_streamer::{streamer::FileStreamer, StreamerConfig, ToolFactory};
use parseltongue_core::entities::EntityClass;
use parseltongue_core::storage::CozoDbStorage;
use tempfile::TempDir;

/// TDD RED→GREEN Test: Verify test functions are EXCLUDED from ingestion
///
/// **v0.9.6 Implementation**: Test exclusion at ingestion layer
///
/// **Architecture**: Post-classification filtering
/// - QueryBasedExtractor: Extracts entities (language-agnostic)
/// - Attribute Parser: Enriches Rust entities with #[test] metadata (Rust-specific)
/// - **NEW**: Streamer filters out TestImplementation entities before database insertion
///
/// Preconditions:
/// - Rust file with #[test] attribute and regular function
/// - File indexed by Tool 1 with test exclusion enabled
///
/// Postconditions:
/// - Test entities are detected but NOT inserted into database
/// - Only CODE entities are in database (regular_function)
/// - Database contains exactly 1 entity (not 2)
///
/// Error Conditions:
/// - Test entity found in database → FAIL (should be excluded)
#[tokio::test]
async fn test_function_with_test_attribute_classified_correctly() {
    // Setup: Create temp directory with separate test and code files
    let temp_dir = TempDir::new().unwrap();

    // File 1: Pure code file with regular function
    let code_file = temp_dir.path().join("lib.rs");
    std::fs::write(
        &code_file,
        r#"
fn regular_function() {
    println!("Not a test");
}
"#,
    )
    .unwrap();

    // File 2: Test file with test attribute
    let tests_dir = temp_dir.path().join("tests");
    std::fs::create_dir(&tests_dir).unwrap();
    let test_file = tests_dir.join("unit.rs");
    std::fs::write(
        &test_file,
        r#"
#[test]
fn test_example() {
    assert_eq!(1 + 1, 2);
}
"#,
    )
    .unwrap();

    // Setup database
    let db_path = temp_dir.path().join("test.db");
    let config = StreamerConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: format!("rocksdb:{}", db_path.display()),
        max_file_size: 1024 * 1024,
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    // Execute: Index with Tool 1
    {
        let streamer = ToolFactory::create_streamer(config.clone()).await.unwrap();
        let _result = streamer.stream_directory().await.unwrap();
    } // Drop streamer to release database lock

    // Verify: Check classifications
    let storage = CozoDbStorage::new(&config.db_path).await.unwrap();
    let entities = storage.get_all_entities().await.unwrap();

    // ✅ v0.9.6: Should have only 1 entity (test excluded, only regular_function)
    assert_eq!(entities.len(), 1, "Should have exactly 1 entity (test excluded)");

    // Verify test entity is NOT in database
    let test_entity = entities
        .iter()
        .find(|e| e.interface_signature.name == "test_example");
    assert!(test_entity.is_none(), "test_example should be EXCLUDED from database");

    // Find regular function (should be the only entity)
    let code_entity = &entities[0];
    assert_eq!(
        code_entity.interface_signature.name, "regular_function",
        "Only regular_function should be in database"
    );

    assert_eq!(
        code_entity.tdd_classification.entity_class,
        EntityClass::CodeImplementation,
        "Regular function should be classified as CODE_IMPLEMENTATION"
    );
}

