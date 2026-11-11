// Test for Bug 1: ISGL1 v2 Key Format in Edges
// REQ-EDGE-001.0: Edge keys must match entity ISGL1 v2 format

use parseltongue_core::isgl1_v2::{compute_birth_timestamp, extract_semantic_path};

/// REQ-EDGE-002.0: Semantic path extraction for root-level C# class
#[test]
fn test_req_edge_002_semantic_path_root_class() {
    // Arrange: Root-level class file
    let file_path = "Logger.cs";

    // Act
    let semantic_path = extract_semantic_path(file_path);

    // Assert: Root uses double underscore prefix, stem without extension
    assert_eq!(semantic_path, "__Logger");
}

/// REQ-EDGE-002.0: Semantic path extraction for nested C# classes
#[test]
fn test_req_edge_002_semantic_path_nested_class() {
    // Arrange: Nested class file (namespace/class pattern)
    let file_path = "MyNamespace/MyClass.cs";

    // Act
    let semantic_path = extract_semantic_path(file_path);

    // Assert: Path separator replaced with underscore
    assert_eq!(semantic_path, "__MyNamespace_MyClass");
}

/// REQ-EDGE-002.0: Semantic path extraction handles complex paths
#[test]
fn test_req_edge_002_semantic_path_complex() {
    // Arrange: Complex nested path
    let file_path = "src/Services/Data/Repository.cs";

    // Act
    let semantic_path = extract_semantic_path(file_path);

    // Assert: All separators replaced
    assert_eq!(semantic_path, "__src_Services_Data_Repository");
}

/// REQ-EDGE-003.0: Birth timestamp stability
#[test]
fn test_req_edge_003_timestamp_stability() {
    // Arrange: Same file and entity name
    let file_path = "Logger.cs";
    let entity_name = "DoWork";

    // Act: Compute timestamp twice
    let ts1 = compute_birth_timestamp(file_path, entity_name);
    let ts2 = compute_birth_timestamp(file_path, entity_name);

    // Assert: Identical timestamps
    assert_eq!(ts1, ts2, "Birth timestamp must be deterministic");
}

/// REQ-EDGE-003.0: Birth timestamp uniqueness for different entities
#[test]
fn test_req_edge_003_timestamp_uniqueness() {
    // Arrange: Different entity names in same file
    let file_path = "Logger.cs";
    let entity1 = "DoWork";
    let entity2 = "Helper";

    // Act
    let ts1 = compute_birth_timestamp(file_path, entity1);
    let ts2 = compute_birth_timestamp(file_path, entity2);

    // Assert: Different timestamps
    assert_ne!(
        ts1, ts2,
        "Different entities should have different timestamps"
    );
}

/// REQ-EDGE-003.0: Birth timestamp uniqueness for different files
#[test]
fn test_req_edge_003_timestamp_uniqueness_different_files() {
    // Arrange: Same entity name in different files
    let file1 = "Logger.cs";
    let file2 = "DataModel.cs";
    let entity_name = "ToString";

    // Act
    let ts1 = compute_birth_timestamp(file1, entity_name);
    let ts2 = compute_birth_timestamp(file2, entity_name);

    // Assert: Different timestamps
    assert_ne!(
        ts1, ts2,
        "Same entity name in different files should have different timestamps"
    );
}

/// Edge case: Empty file path handling
#[test]
fn test_semantic_path_empty_file() {
    let semantic_path = extract_semantic_path("");
    assert_eq!(semantic_path, "__");
}

/// Edge case: File path with only extension
#[test]
fn test_semantic_path_only_extension() {
    let semantic_path = extract_semantic_path(".cs");
    assert_eq!(semantic_path, "__");
}

/// Edge case: File path with multiple dots
#[test]
fn test_semantic_path_multiple_dots() {
    let file_path = "My.Class.Helper.cs";
    let semantic_path = extract_semantic_path(file_path);
    assert_eq!(semantic_path, "__My_Class_Helper");
}

/// Test that ISGL1 v2 format does NOT contain line ranges
#[test]
fn test_isgl1_v2_format_no_line_ranges() {
    // Arrange
    let file_path = "Logger.cs";
    let entity_name = "DoWork";
    let semantic_path = extract_semantic_path(file_path);
    let birth_timestamp = compute_birth_timestamp(file_path, entity_name);

    // Act: Build expected ISGL1 v2 key
    let key = format!(
        "csharp:method:{}:{}:T{}",
        entity_name, semantic_path, birth_timestamp
    );

    // Assert: Key does NOT contain line range separators
    assert!(
        !key.contains('-'),
        "ISGL1 v2 keys should not contain line range separator '-'"
    );
    assert!(
        key.contains(":T"),
        "ISGL1 v2 keys should contain timestamp prefix ':T'"
    );
    assert_eq!(
        key.matches(':').count(),
        4,
        "ISGL1 v2 keys should have exactly 4 colons"
    );
}
