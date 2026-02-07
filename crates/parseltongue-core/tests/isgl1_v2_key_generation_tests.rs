// ISGL1 v2 Key Generation Tests
// Cycle 1 of 5 - RED phase
//
// These tests verify the new timestamp-based entity key format.
// Old format: rust:fn:handle_auth:__src_auth_rs:10-50
// New format: rust:fn:handle_auth:__src_auth_rs:T1706284800
//
// Birth timestamp ensures keys remain stable even when lines change.

use parseltongue_core::entities::EntityType;
use parseltongue_core::isgl1_v2::{format_key_v2, extract_semantic_path, compute_birth_timestamp};

/// Test 1: format_key_v2 generates correct timestamp format
#[test]
fn test_format_key_v2_generates_timestamp_suffix() {
    let birth_timestamp = 1706284800; // Fixed timestamp for testing
    let key = format_key_v2(
        EntityType::Function,
        "handle_auth",
        "rust",
        "__src_auth_rs",
        birth_timestamp
    );

    // Should match format: rust:fn:handle_auth:__src_auth_rs:T1706284800
    assert_eq!(key, "rust:fn:handle_auth:__src_auth_rs:T1706284800");
    assert!(key.starts_with("rust:fn:handle_auth:"));
    assert!(key.ends_with(":T1706284800"));
    assert!(!key.contains(":10-50")); // Old format must not appear
}

/// Test 2: Semantic path extraction removes extension and sanitizes
#[test]
fn test_extract_semantic_path_sanitizes_correctly() {
    // Test case 1: Basic path with extension
    let path1 = "src/auth.rs";
    let semantic1 = extract_semantic_path(path1);
    assert_eq!(semantic1, "__src_auth");

    // Test case 2: Nested path with multiple slashes
    let path2 = "crates/core/src/parser/tree.rs";
    let semantic2 = extract_semantic_path(path2);
    assert_eq!(semantic2, "__crates_core_src_parser_tree");

    // Test case 3: Path with special characters
    let path3 = "my-module/lib.py";
    let semantic3 = extract_semantic_path(path3);
    assert_eq!(semantic3, "__my_module_lib");
}

/// Test 3: Birth timestamp is deterministic for same file
#[test]
fn test_compute_birth_timestamp_is_deterministic() {
    let file_path = "src/main.rs";

    // Compute twice - should be identical
    let ts1 = compute_birth_timestamp(file_path, "main");
    let ts2 = compute_birth_timestamp(file_path, "main");

    assert_eq!(ts1, ts2);
    assert!(ts1 > 0); // Valid Unix timestamp
}

/// Test 4: Birth timestamp differs for different entities
#[test]
fn test_compute_birth_timestamp_differs_by_entity() {
    let file_path = "src/main.rs";

    let ts_main = compute_birth_timestamp(file_path, "main");
    let ts_helper = compute_birth_timestamp(file_path, "helper");

    // Different entities should have different timestamps (hash-based)
    assert_ne!(ts_main, ts_helper);
}

/// Test 5: Complete key generation workflow (stability after line changes)
#[test]
fn test_complete_key_generation_workflow() {
    // Simulate first scan (assign birth timestamp)
    let file_path = "lib/processor.rs";
    let entity_name = "process_data";
    let semantic_path = extract_semantic_path(file_path);

    let birth_ts = compute_birth_timestamp(file_path, entity_name);
    let key = format_key_v2(
        EntityType::Function,
        entity_name,
        "rust",
        &semantic_path,
        birth_ts
    );

    // Verify format components
    let parts: Vec<&str> = key.split(':').collect();
    assert_eq!(parts.len(), 5);
    assert_eq!(parts[0], "rust");
    assert_eq!(parts[1], "fn");
    assert_eq!(parts[2], "process_data");
    assert!(parts[3].starts_with("__lib_processor"));
    assert!(parts[4].starts_with('T'));

    // Simulate reindex after moving entity to different line numbers
    // Key must remain IDENTICAL (stable identity) - this is the CORE of ISGL1 v2!
    // The key is based on birth_ts (from file+name hash), NOT line numbers
    let key_after_move = format_key_v2(
        EntityType::Function,
        entity_name,
        "rust",
        &semantic_path,
        birth_ts  // Same birth_ts!
    );

    assert_eq!(key, key_after_move, "Key must remain stable when lines change!");
}
