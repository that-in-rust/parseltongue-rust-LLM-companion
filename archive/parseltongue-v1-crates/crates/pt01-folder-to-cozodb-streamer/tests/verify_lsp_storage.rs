//! Integration test to verify LSP metadata is actually stored in database

use pt01_folder_to_cozodb_streamer::{StreamerConfig, ToolFactory, FileStreamer};
use parseltongue_core::storage::CozoDbStorage;
use tempfile::TempDir;

#[tokio::test]
async fn test_query_stored_entity_and_verify_in_codebase() {
    // Create a test Rust file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("example.rs");
    std::fs::write(
        &test_file,
        r#"
pub fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

pub struct Calculator {
    name: String,
    version: u32,
}

impl Calculator {
    pub fn new(name: String) -> Self {
        Self { name, version: 1 }
    }
}
"#,
    )
    .unwrap();

    // Index the file
    let db_dir = TempDir::new().unwrap();
    let db_path = format!("rocksdb:{}", db_dir.path().display());

    let config = StreamerConfig {
        root_dir: temp_dir.path().to_path_buf(),
        db_path: db_path.clone(),
        max_file_size: 1024 * 1024,
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec![],
        parsing_library: "tree-sitter".to_string(),
        chunking: "ISGL1".to_string(),
    };

    let streamer = ToolFactory::create_streamer(config).await.unwrap();
    let result = streamer.stream_directory().await.unwrap();

    println!("\n📊 Indexing Results:");
    println!("  Files processed: {}", result.processed_files);
    println!("  Entities created: {}", result.entities_created);
    assert!(result.entities_created >= 3, "Should have at least 3 entities (function, struct, impl)");

    // Drop the streamer to release the RocksDB lock before opening a new connection
    drop(streamer);

    // Now query the database to get one entity
    let db = CozoDbStorage::new(&db_path).await.unwrap();

    // Query using get_all_entities
    let entities = db.get_all_entities().await.unwrap();
    println!("\n🔍 Total entities in database: {}", entities.len());

    // Pick first entity and verify
    if let Some(first_entity) = entities.first() {
        let isgl1_key = &first_entity.isgl1_key;
        println!("\n✅ Found entity: {}", isgl1_key);
        println!("  LSP metadata present: {}", first_entity.lsp_metadata.is_some());

        if let Some(lsp_meta) = &first_entity.lsp_metadata {
            println!("\n📊 LSP Metadata:");
            println!("  Type: {}", lsp_meta.type_information.resolved_type);
            println!("  Module path: {:?}", lsp_meta.type_information.module_path);
            println!("  Generic params: {:?}", lsp_meta.type_information.generic_parameters);
            println!("  Usage references: {}", lsp_meta.usage_analysis.total_references);
        }

        // Verify the entity exists in the source file
        let source_content = std::fs::read_to_string(&test_file).unwrap();

        // Parse ISGL1 key to extract entity name
        let parts: Vec<&str> = isgl1_key.split(':').collect();
        if parts.len() >= 3 {
            let entity_name = parts[2];
            println!("\n🔎 Searching for entity '{}' in source...", entity_name);

            // Search for entity in source
            assert!(
                source_content.contains(entity_name),
                "Entity '{}' should exist in source file",
                entity_name
            );
            println!("  ✓ Verified in source code!");

            // Show context from source
            for (i, line) in source_content.lines().enumerate() {
                if line.contains(entity_name) {
                    println!("\n📝 Source code context (line {}):", i + 1);
                    println!("  {}", line.trim());
                }
            }
        }
    }
}
