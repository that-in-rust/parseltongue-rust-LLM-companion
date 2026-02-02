//! E2E tests for incremental reindex with ISGL1 v2 entity matching
//!
//! # 4-Word Naming: e2e_incremental_reindex_isgl1v2_tests
//!
//! Tests verify:
//! - Key stability when file content changes (0% churn)
//! - Entity matching algorithm (ContentMatch > PositionMatch > NewEntity)
//! - Hash caching for unchanged files
//! - Statistics accuracy

use axum::{body::Body, http::Request};
use tower::ServiceExt;
use std::sync::Arc;
use parseltongue_core::storage::CozoDbStorage;
use pt08_http_code_query_server::{
    build_complete_router_instance,
    SharedApplicationStateContainer
};

// Test fixture: Initial Rust file with 3 functions
const INITIAL_FIXTURE_CONTENT: &str = r#"pub fn alpha_function() {
    println!("Alpha");
}

pub fn beta_function() {
    println!("Beta");
}

pub fn gamma_function() {
    println!("Gamma");
}
"#;

/// Setup test database with temporary directory
///
/// # 4-Word Name: setup_test_database_with_fixture
async fn setup_test_database_with_fixture() -> (SharedApplicationStateContainer, Arc<CozoDbStorage>, tempfile::TempDir) {
    // Create in-memory database
    let storage_raw = CozoDbStorage::new("mem").await.unwrap();
    storage_raw.create_schema().await.unwrap();
    storage_raw.create_dependency_edges_schema().await.unwrap();
    storage_raw.create_file_hash_cache_schema().await.unwrap();

    let state = SharedApplicationStateContainer::create_with_database_storage(storage_raw);

    // Get Arc reference from state
    let storage = state.database_storage_connection_arc.read().await
        .as_ref()
        .expect("Database should be connected")
        .clone();

    // Create temp directory for test files
    let temp_dir = tempfile::TempDir::new().unwrap();

    (state, storage, temp_dir)
}

/// Send reindex request to endpoint
///
/// # 4-Word Name: send_reindex_request_http
async fn send_reindex_request_http(
    app: &axum::Router,
    file_path: &str
) -> serde_json::Value {
    let uri = format!("/incremental-reindex-file-update?path={}",
        urlencoding::encode(file_path));

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    assert_eq!(status, 200, "Request should succeed. Body: {}",
        String::from_utf8_lossy(&body));

    serde_json::from_slice(&body).unwrap()
}

/// Get all entity keys for a file
///
/// # 4-Word Name: get_all_entity_keys
async fn get_all_entity_keys(storage: &CozoDbStorage, file_path: &str) -> Vec<String> {
    let entities = storage.get_entities_by_file_path(file_path)
        .await
        .unwrap_or_default();

    entities.iter()
        .map(|e| e.isgl1_key.clone())
        .collect()
}

/// Get all keys and filter by name (simplified)
///
/// # 4-Word Name: get_keys_filtered_by_name
#[allow(dead_code)]
async fn get_keys_filtered_by_name(storage: &CozoDbStorage, name_filter: &str) -> Vec<String> {
    // Get all entities and filter by name in the key
    let all_files = vec![""];  // We don't filter by file in this helper
    let mut all_keys = Vec::new();

    for file in all_files {
        let entities = storage.get_entities_by_file_path(file)
            .await
            .unwrap_or_default();

        for entity in entities {
            if entity.isgl1_key.contains(name_filter) {
                all_keys.push(entity.isgl1_key.clone());
            }
        }
    }

    all_keys
}

/// Extract timestamp portion from ISGL1 v2 key
///
/// # 4-Word Name: extract_timestamp_from_key
fn extract_timestamp_from_key(key: &str) -> String {
    // Key format: rust:fn:alpha_function:__test_module:T1706284800
    key.split(':')
        .last()
        .unwrap_or("")
        .to_string()
}

/// Calculate key preservation rate
///
/// # 4-Word Name: calculate_key_preservation_rate
fn calculate_key_preservation_rate(
    keys_before: &[String],
    keys_after: &[String]
) -> f64 {
    if keys_before.is_empty() {
        return 100.0;
    }

    let preserved_count = keys_before.iter()
        .filter(|k| keys_after.contains(k))
        .count();

    (preserved_count as f64 / keys_before.len() as f64) * 100.0
}

/// Test E2E-ISGL1V2-001: Adding lines at top preserves all entity keys
///
/// # 4-Word Name: test_add_lines_preserves_keys
///
/// # Acceptance Criteria
/// WHEN 10 comment lines are added at top of file
/// THEN ALL 3 existing entity keys SHALL remain unchanged
/// AND entities_added SHALL be 0 (no new entities)
/// AND entities_removed SHALL be 0 (no deletions)
/// AND hash_changed SHALL be true (content changed)
#[tokio::test]
async fn test_add_lines_preserves_keys() {
    // GIVEN: Database with initial file indexed
    let (state, storage, temp_dir) = setup_test_database_with_fixture().await;
    let test_file = temp_dir.path().join("test_module.rs");

    // Write initial version (3 functions)
    std::fs::write(&test_file, INITIAL_FIXTURE_CONTENT).unwrap();

    // Index initial version
    let app = build_complete_router_instance(state.clone());
    let _ = send_reindex_request_http(&app, test_file.to_str().unwrap()).await;

    // Capture entity keys BEFORE modification
    let keys_before = get_all_entity_keys(&storage, test_file.to_str().unwrap()).await;
    assert_eq!(keys_before.len(), 3, "Should have 3 initial entities");

    println!("Keys BEFORE: {:?}", keys_before);

    // WHEN: Add 10 comment lines at top
    let modified_content = format!("// Comment 1\n// Comment 2\n// Comment 3\n// Comment 4\n// Comment 5\n// Comment 6\n// Comment 7\n// Comment 8\n// Comment 9\n// Comment 10\n\n{}",
        INITIAL_FIXTURE_CONTENT
    );
    std::fs::write(&test_file, modified_content).unwrap();

    // Trigger reindex
    let response = send_reindex_request_http(&app, test_file.to_str().unwrap()).await;

    println!("Response: {}", serde_json::to_string_pretty(&response).unwrap());

    // THEN: Keys preserved
    let keys_after = get_all_entity_keys(&storage, test_file.to_str().unwrap()).await;

    println!("Keys AFTER: {:?}", keys_after);

    let preservation_rate = calculate_key_preservation_rate(&keys_before, &keys_after);
    assert_eq!(preservation_rate, 100.0,
        "Expected 100% key preservation, got {}%. Keys before: {:?}, after: {:?}",
        preservation_rate, keys_before, keys_after);

    assert_eq!(response["data"]["entities_added"].as_u64().unwrap(), 0);
    assert_eq!(response["data"]["entities_removed"].as_u64().unwrap(), 0);
    assert_eq!(response["data"]["hash_changed"].as_bool().unwrap(), true);

    // Verify timestamp portion is preserved
    for key in &keys_before {
        assert!(key.contains(":T"), "Key must have timestamp: {}", key);
        let timestamp = extract_timestamp_from_key(key);
        assert!(keys_after.iter().any(|k| k.contains(&timestamp)),
            "Timestamp {} must be preserved in key {}", timestamp, key);
    }
}

/// Test E2E-ISGL1V2-002: Modifying function body preserves key via PositionMatch
///
/// # 4-Word Name: test_modify_body_preserves_key
#[tokio::test]
async fn test_modify_body_preserves_key() {
    // GIVEN: Indexed file with beta_function
    let (state, storage, temp_dir) = setup_test_database_with_fixture().await;
    let test_file = temp_dir.path().join("test_module.rs");
    std::fs::write(&test_file, INITIAL_FIXTURE_CONTENT).unwrap();

    let app = build_complete_router_instance(state.clone());
    let _ = send_reindex_request_http(&app, test_file.to_str().unwrap()).await;

    let keys_before = get_all_entity_keys(&storage, test_file.to_str().unwrap()).await;
    let beta_key_before = keys_before.iter()
        .find(|k| k.contains("beta_function"))
        .expect("Should have beta_function key");

    println!("Beta key BEFORE: {}", beta_key_before);

    // WHEN: Modify beta_function body (change println message)
    let modified = INITIAL_FIXTURE_CONTENT.replace(
        r#"println!("Beta");"#,
        r#"println!("Beta modified with extra logic");"#
    );
    std::fs::write(&test_file, modified).unwrap();

    let response = send_reindex_request_http(&app, test_file.to_str().unwrap()).await;

    // THEN: Key preserved (PositionMatch - same name, file, approximate position)
    let keys_after = get_all_entity_keys(&storage, test_file.to_str().unwrap()).await;
    let beta_key_after = keys_after.iter()
        .find(|k| k.contains("beta_function"))
        .expect("Should have beta_function key after modification");

    println!("Beta key AFTER: {}", beta_key_after);

    assert_eq!(beta_key_before, beta_key_after,
        "Key must be preserved when function body changes");
    assert_eq!(response["data"]["entities_added"].as_u64().unwrap(), 0);
    assert_eq!(response["data"]["entities_removed"].as_u64().unwrap(), 0);
}

/// Test E2E-ISGL1V2-003: Adding new function creates new timestamp key
///
/// # 4-Word Name: test_add_function_new_key
#[tokio::test]
async fn test_add_function_new_key() {
    // GIVEN: Indexed file with 3 functions
    let (state, storage, temp_dir) = setup_test_database_with_fixture().await;
    let test_file = temp_dir.path().join("test_module.rs");
    std::fs::write(&test_file, INITIAL_FIXTURE_CONTENT).unwrap();

    let app = build_complete_router_instance(state.clone());
    let _ = send_reindex_request_http(&app, test_file.to_str().unwrap()).await;

    let keys_before = get_all_entity_keys(&storage, test_file.to_str().unwrap()).await;

    println!("Keys BEFORE adding delta: {:?}", keys_before);

    // WHEN: Add new delta_function
    let modified = format!("{}\n\npub fn delta_function() {{\n    println!(\"Delta\");\n}}",
        INITIAL_FIXTURE_CONTENT);
    std::fs::write(&test_file, modified).unwrap();

    let response = send_reindex_request_http(&app, test_file.to_str().unwrap()).await;

    // THEN: New entity with new key created
    let keys_after = get_all_entity_keys(&storage, test_file.to_str().unwrap()).await;

    println!("Keys AFTER adding delta: {:?}", keys_after);

    assert_eq!(keys_after.len(), 4, "Should have 4 entities after addition");
    assert_eq!(response["data"]["entities_added"].as_u64().unwrap(), 1);
    assert_eq!(response["data"]["entities_removed"].as_u64().unwrap(), 0);

    // Find the new key
    let new_keys: Vec<_> = keys_after.iter()
        .filter(|k| !keys_before.contains(k))
        .collect();
    assert_eq!(new_keys.len(), 1, "Should have exactly 1 new key");

    let delta_key = new_keys[0];
    assert!(delta_key.contains("delta_function"), "Key must contain function name: {}", delta_key);
    assert!(delta_key.contains(":T"), "Key must have timestamp: {}", delta_key);
}

/// Test E2E-ISGL1V2-004: Deleting function removes entity
///
/// # 4-Word Name: test_delete_function_removes_entity
#[tokio::test]
async fn test_delete_function_removes_entity() {
    // GIVEN: Indexed file with 3 functions
    let (state, storage, temp_dir) = setup_test_database_with_fixture().await;
    let test_file = temp_dir.path().join("test_module.rs");
    std::fs::write(&test_file, INITIAL_FIXTURE_CONTENT).unwrap();

    let app = build_complete_router_instance(state.clone());
    let _ = send_reindex_request_http(&app, test_file.to_str().unwrap()).await;

    let keys_before = get_all_entity_keys(&storage, test_file.to_str().unwrap()).await;
    let beta_key = keys_before.iter()
        .find(|k| k.contains("beta_function"))
        .expect("Should have beta_function key")
        .clone();

    println!("Keys BEFORE deletion: {:?}", keys_before);

    // WHEN: Delete beta_function from file
    let modified = INITIAL_FIXTURE_CONTENT
        .lines()
        .filter(|line| !line.contains("beta_function") && !line.contains("Beta"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&test_file, modified).unwrap();

    let response = send_reindex_request_http(&app, test_file.to_str().unwrap()).await;

    // THEN: Entity removed
    assert_eq!(response["data"]["entities_removed"].as_u64().unwrap(), 1);
    assert_eq!(response["data"]["entities_added"].as_u64().unwrap(), 0);

    let keys_after = get_all_entity_keys(&storage, test_file.to_str().unwrap()).await;

    println!("Keys AFTER deletion: {:?}", keys_after);

    assert_eq!(keys_after.len(), 2, "Should have 2 entities after deletion");
    assert!(!keys_after.contains(&beta_key), "Deleted entity key must be removed");
}

/// Test E2E-ISGL1V2-005: Unchanged file skips reindexing (hash cache)
///
/// # 4-Word Name: test_unchanged_file_cached_hash
#[tokio::test]
async fn test_unchanged_file_cached_hash() {
    // GIVEN: File indexed once (hash cached)
    let (state, _storage, temp_dir) = setup_test_database_with_fixture().await;
    let test_file = temp_dir.path().join("test_module.rs");
    std::fs::write(&test_file, INITIAL_FIXTURE_CONTENT).unwrap();

    let app = build_complete_router_instance(state.clone());
    let _ = send_reindex_request_http(&app, test_file.to_str().unwrap()).await;

    // WHEN: Reindex same file without changes
    let start = std::time::Instant::now();
    let response = send_reindex_request_http(&app, test_file.to_str().unwrap()).await;
    let duration = start.elapsed();

    // THEN: Early return with hash_changed: false
    assert_eq!(response["data"]["hash_changed"].as_bool().unwrap(), false);
    assert_eq!(response["data"]["entities_added"].as_u64().unwrap(), 0);
    assert_eq!(response["data"]["entities_removed"].as_u64().unwrap(), 0);

    println!("Cache hit processing time: {}ms", duration.as_millis());
    assert!(duration.as_millis() < 100,
        "Cached lookup should be fast (<100ms), took {}ms", duration.as_millis());
}
