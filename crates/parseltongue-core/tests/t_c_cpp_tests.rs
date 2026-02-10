// C and C++ Dependency Edge Tests (T140-T159)
//
// Tests for C and C++ dependency edge detection using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;

// ============================================================================
// C TESTS (T140-T149)
// ============================================================================

// ============================================================================
// T140: C Struct Field Access Edges
// ============================================================================

#[test]
fn t140_c_struct_field_arrow() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T140-c-struct-field-access-edges",
        "arrow.c",
    );

    println!("=== C STRUCT FIELD ARROW ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Extract just the target names from edges
    let edge_targets: Vec<String> = edges
        .iter()
        .map(|e| {
            e.to_key
                .to_string()
                .split(':')
                .nth(2)
                .unwrap_or("")
                .to_string()
        })
        .collect();

    assert!(
        edge_targets.iter().any(|e| e == "name"),
        "Should detect name field"
    );
    assert!(
        edge_targets.iter().any(|e| e == "age"),
        "Should detect age field"
    );
}

#[test]
fn t140_c_struct_field_dot() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T140-c-struct-field-access-edges",
        "dot.c",
    );

    println!("=== C STRUCT FIELD DOT ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Extract just the target names from edges
    let edge_targets: Vec<String> = edges
        .iter()
        .map(|e| {
            e.to_key
                .to_string()
                .split(':')
                .nth(2)
                .unwrap_or("")
                .to_string()
        })
        .collect();

    assert!(
        edge_targets.iter().any(|e| e == "timeout"),
        "Should detect timeout field"
    );
    assert!(
        edge_targets.iter().any(|e| e == "port"),
        "Should detect port field"
    );
}

// ============================================================================
// T142: C Function Call Edges
// ============================================================================

#[test]
fn t142_c_existing_function_calls() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T142-c-function-call-edges",
        "calls.c",
    );

    println!("=== C FUNCTION CALLS ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Extract just the target names from edges
    let edge_targets: Vec<String> = edges
        .iter()
        .map(|e| {
            e.to_key
                .to_string()
                .split(':')
                .nth(2)
                .unwrap_or("")
                .to_string()
        })
        .collect();

    assert!(
        edge_targets.iter().any(|e| e == "printf"),
        "Should detect printf"
    );
    assert!(
        edge_targets.iter().any(|e| e == "process_data"),
        "Should detect process_data"
    );
}

// ============================================================================
// C++ TESTS (T150-T159)
// ============================================================================

// ============================================================================
// T150: C++ new_expression Edges
// ============================================================================

#[test]
fn t150_cpp_new_expression_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T150-cpp-new-expression-edges",
        "simple.cpp",
    );

    println!("\n=== C++ new_expression (Simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect constructor calls via new_expression
    let user_edges = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let config_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Config"));

    assert!(user_edges, "Expected edge for User constructor via new");
    assert!(
        config_edges,
        "Expected edge for Config constructor via new"
    );
}

#[test]
fn t150_cpp_new_expression_template() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T150-cpp-new-expression-edges",
        "template.cpp",
    );

    println!("\n=== C++ new_expression (Template) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect template type constructors
    let vector_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("vector"));
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("map"));

    assert!(vector_edges, "Expected edge for std::vector constructor");
    assert!(map_edges, "Expected edge for std::map constructor");
}

// ============================================================================
// T152: C++ Field Access Edges
// ============================================================================

#[test]
fn t152_cpp_field_access_pointer() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T152-cpp-field-access-pointer-edges",
        "pointer.cpp",
    );

    println!("\n=== C++ field_expression (Pointer) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect field access via ->
    let name_edges = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edges = edges.iter().any(|e| e.to_key.as_str().contains("age"));

    assert!(name_edges, "Expected edge for name field access");
    assert!(age_edges, "Expected edge for age field access");
}

#[test]
fn t152_cpp_field_access_reference() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T152-cpp-field-access-pointer-edges",
        "reference.cpp",
    );

    println!("\n=== C++ field_expression (Reference) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect field access via .
    let name_edges = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edges = edges.iter().any(|e| e.to_key.as_str().contains("age"));

    assert!(name_edges, "Expected edge for name field access");
    assert!(age_edges, "Expected edge for age field access");
}

// ============================================================================
// T154: C++ Template Type Edges
// ============================================================================

#[test]
fn t154_cpp_template_types_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T154-cpp-template-type-edges",
        "simple.cpp",
    );

    println!("\n=== C++ template_type (Simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect template type usage
    let vector_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("vector"));
    let string_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("string"));

    assert!(vector_edges, "Expected edge for std::vector template type");
    assert!(string_edges, "Expected edge for std::string type");
}

#[test]
fn t154_cpp_template_types_nested() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T154-cpp-template-type-edges",
        "nested.cpp",
    );

    println!("\n=== C++ template_type (Nested) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect nested template types
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("map"));
    let user_edges = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let vector_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("vector"));
    let shared_ptr_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("shared_ptr"));
    let config_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Config"));

    assert!(map_edges, "Expected edge for std::map");
    assert!(user_edges, "Expected edge for User type");
    assert!(vector_edges, "Expected edge for std::vector");
    assert!(shared_ptr_edges, "Expected edge for std::shared_ptr");
    assert!(config_edges, "Expected edge for Config type");
}

// ============================================================================
// T156: C++ Smart Pointer Edges
// ============================================================================

#[test]
fn t156_cpp_smart_pointers_unique() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T156-cpp-smart-pointer-edges",
        "unique.cpp",
    );

    println!("\n=== C++ Smart Pointers (unique_ptr) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect make_unique calls
    let make_unique_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("make_unique"));
    let user_edges = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let config_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Config"));

    assert!(make_unique_edges, "Expected edge for make_unique function");
    assert!(user_edges, "Expected edge for User type in make_unique");
    assert!(
        config_edges,
        "Expected edge for Config type in make_unique"
    );
}

#[test]
fn t156_cpp_smart_pointers_shared() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T156-cpp-smart-pointer-edges",
        "shared.cpp",
    );

    println!("\n=== C++ Smart Pointers (shared_ptr) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect make_shared calls
    let make_shared_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("make_shared"));
    let config_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Config"));
    let manager_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Manager"));

    assert!(make_shared_edges, "Expected edge for make_shared function");
    assert!(
        config_edges,
        "Expected edge for Config type in make_shared"
    );
    assert!(
        manager_edges,
        "Expected edge for Manager type in make_shared"
    );
}

// ============================================================================
// T158: C++ Regression Tests
// ============================================================================

#[test]
fn t158_cpp_existing_function_calls() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T158-cpp-regression-all-patterns",
        "calls.cpp",
    );

    println!("\n=== C++ Existing Function Calls Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Ensure existing function call detection still works
    let calc_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("calculate"));
    let validate_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("validate"));

    assert!(calc_edges, "Expected edge for calculate function");
    assert!(validate_edges, "Expected edge for validate function");
}
