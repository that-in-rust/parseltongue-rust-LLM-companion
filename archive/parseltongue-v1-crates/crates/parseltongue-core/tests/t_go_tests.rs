// Go Dependency Edge Tests (T100-T119)
//
// Tests for Go dependency edge detection using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;

// ============================================================================
// T100: Composite Literal Edges
// ============================================================================

#[test]
fn t100_go_composite_literal_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T100-go-composite-literal-edges",
        "simple.go",
    );

    println!("=== GO COMPOSITE LITERAL SIMPLE ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect composite literals as dependencies
    let user_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("User"))
        .collect();
    let config_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("Config"))
        .collect();

    assert!(!user_edges.is_empty(), "Expected edge to User type");
    assert!(!config_edges.is_empty(), "Expected edge to Config type");
}

#[test]
fn t100_go_composite_literal_pointer() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T100-go-composite-literal-edges",
        "pointer.go",
    );

    println!("=== GO COMPOSITE LITERAL POINTER ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect pointer composite literals
    let server_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("Server"))
        .collect();

    assert!(
        !server_edges.is_empty(),
        "Expected edge to Server type (pointer composite)"
    );
}

#[test]
fn t100_go_composite_literal_qualified() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T100-go-composite-literal-edges",
        "qualified.go",
    );

    println!("=== GO COMPOSITE LITERAL QUALIFIED ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect qualified composite literals
    let user_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("User"))
        .collect();

    assert!(
        !user_edges.is_empty(),
        "Expected edge to User type (qualified)"
    );
}

// ============================================================================
// T102: Goroutine Dependency Edges
// ============================================================================

#[test]
fn t102_go_goroutines_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T102-go-goroutine-dependency-edges",
        "basic.go",
    );

    println!("=== GO GOROUTINES BASIC ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect goroutine launches as dependencies
    let process_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("processData"))
        .collect();
    let handle_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("handleRequest"))
        .collect();

    assert!(
        !process_edges.is_empty(),
        "Expected edge to processData from goroutine"
    );
    assert!(
        !handle_edges.is_empty(),
        "Expected edge to handleRequest from goroutine"
    );
}

#[test]
fn t102_go_goroutines_anonymous() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T102-go-goroutine-dependency-edges",
        "anonymous.go",
    );

    println!("=== GO GOROUTINES ANONYMOUS ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect calls inside anonymous goroutines
    let work_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("doWork"))
        .collect();

    assert!(
        !work_edges.is_empty(),
        "Expected edge to doWork from anonymous goroutine"
    );
}

// ============================================================================
// T104: Slice/Map Literal Type Edges
// ============================================================================

#[test]
fn t104_go_slice_literal_with_type() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T104-go-slice-map-edges",
        "slice.go",
    );

    println!("=== GO SLICE LITERAL ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect Item type reference in slice literal
    let item_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("Item"))
        .collect();

    assert!(
        !item_edges.is_empty(),
        "Expected edge to Item type in slice literal"
    );
}

#[test]
fn t104_go_map_literal_with_type() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T104-go-slice-map-edges",
        "map.go",
    );

    println!("=== GO MAP LITERAL ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect User type reference in map literal
    let user_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("User"))
        .collect();

    assert!(
        !user_edges.is_empty(),
        "Expected edge to User type in map literal"
    );
}

// ============================================================================
// T106: Integration Test - All Patterns
// ============================================================================

#[test]
fn t106_go_edge_integration() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T106-go-integration-all-patterns",
        "combined.go",
    );

    println!("=== GO INTEGRATION ===");
    println!("Total entities: {}", _entities.len());
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect multiple dependency types
    assert!(
        edges.len() >= 2,
        "Expected at least 2 edges (composite literal + goroutine)"
    );

    // Should have edges for:
    // 1. Config composite literal
    // 2. handleRequests goroutine
    let config_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("Config"))
        .collect();
    let goroutine_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("handleRequests"))
        .collect();

    assert!(!config_edges.is_empty(), "Expected edge to Config");
    assert!(
        !goroutine_edges.is_empty(),
        "Expected edge to handleRequests"
    );
}

// ============================================================================
// T103: Field Access Edges (Known Limitation - Ignored)
// ============================================================================

#[test]
#[ignore = "Known limitation: Go's tree-sitter grammar cannot distinguish field access from method calls without type information. Field access (user.Name) is a bare selector_expression, while method calls (user.Method()) are call_expression wrapping selector_expression. Capturing both would create duplicates for method calls. Future enhancement: add field_access pattern and mark duplicates."]
fn t103_go_field_access_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T103-go-field-access-edges",
        "field_basic.go",
    );

    println!("=== GO FIELD ACCESS BASIC ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // KNOWN LIMITATION: Pure field access (not in call context) is not captured
    // because it would duplicate method call edges. Go's selector_expression
    // is used for both user.Method() and user.Field.
    //
    // To fix this properly, we would need to:
    // 1. Add a field_access pattern for bare selector_expression
    // 2. Filter out duplicates where selector_expression is inside call_expression
    // 3. Or use a different EdgeType for field access vs method calls
    let field_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("Name") || to_key_str.contains("Age")
        })
        .collect();

    // This test is ignored because field access is a known limitation
    assert!(
        field_edges.len() >= 2,
        "Expected field access edges: got {}",
        field_edges.len()
    );
}

#[test]
#[ignore = "Known limitation: same as t103_go_field_access_basic - field access without call context is not captured to avoid duplicating method call edges"]
fn t103_go_field_access_nested() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T103-go-field-access-edges",
        "field_nested.go",
    );

    println!("=== GO FIELD ACCESS NESTED ===");
    println!("Total edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // KNOWN LIMITATION: Nested field access is not captured
    let field_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("config")
                || to_key_str.contains("Settings")
                || to_key_str.contains("Timeout")
        })
        .collect();

    // This test is ignored because nested field access is a known limitation
    assert!(
        field_edges.len() >= 3,
        "Expected nested field access edges: got {}",
        field_edges.len()
    );
}
