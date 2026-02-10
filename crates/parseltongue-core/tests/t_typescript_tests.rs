// TypeScript Dependency Edge Tests (T080-T099)
//
// Tests for TypeScript dependency edge detection using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;

// ============================================================================
// T080: Constructor New Edges
// ============================================================================

#[test]
fn t080_typescript_constructor_new_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T080-typescript-constructor-new-edges",
        "constructor_simple.ts",
    );

    println!("\n=== Constructor (Simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect constructor calls to Person and DataModel
    let person_edges = edges.iter().any(|e| e.to_key.as_str().contains("Person"));
    let model_edges = edges.iter().any(|e| e.to_key.as_str().contains("DataModel"));

    assert!(person_edges, "Expected edge for Person constructor");
    assert!(model_edges, "Expected edge for DataModel constructor");
}

#[test]
fn t080_typescript_constructor_new_generic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T080-typescript-constructor-new-edges",
        "constructor_generic.ts",
    );

    println!("\n=== Constructor (Generic) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect generic constructor calls
    let array_edges = edges.iter().any(|e| e.to_key.as_str().contains("Array"));
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("Map"));

    assert!(array_edges, "Expected edge for Array constructor");
    assert!(map_edges, "Expected edge for Map constructor");
}

#[test]
fn t080_typescript_constructor_new_qualified() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T080-typescript-constructor-new-edges",
        "constructor_qualified.ts",
    );

    println!("\n=== Constructor (Qualified) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect qualified constructor call
    let user_edges = edges.iter().any(|e| e.to_key.as_str().contains("User"));

    assert!(user_edges, "Expected edge for User constructor");
}

// ============================================================================
// T082: Method Call Edges
// ============================================================================

#[test]
fn t082_typescript_method_calls_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T082-typescript-method-call-edges",
        "method_calls.ts",
    );

    println!("\n=== Method Calls Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect method calls
    let helper_edges = edges.iter().any(|e| e.to_key.as_str().contains("helper"));
    let work_edges = edges.iter().any(|e| e.to_key.as_str().contains("doWork"));

    assert!(helper_edges, "Expected edge for helper method call");
    assert!(work_edges, "Expected edge for doWork method call");
}

// ============================================================================
// T084: Property Access Edges
// ============================================================================

#[test]
fn t084_typescript_property_access_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T084-typescript-property-access-edges",
        "property_simple.ts",
    );

    println!("\n=== Property Access (Simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect property access
    let name_edges = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edges = edges.iter().any(|e| e.to_key.as_str().contains("age"));

    assert!(name_edges, "Expected edge for name property access");
    assert!(age_edges, "Expected edge for age property access");
}

#[test]
fn t084_typescript_property_access_chained() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T084-typescript-property-access-edges",
        "property_chained.ts",
    );

    println!("\n=== Property Access (Chained) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect chained property access
    let config_edges = edges.iter().any(|e| e.to_key.as_str().contains("config"));
    let value_edges = edges.iter().any(|e| e.to_key.as_str().contains("value"));

    assert!(config_edges || value_edges, "Expected edge for chained property access");
}

// ============================================================================
// T086: Collection Operation Edges
// ============================================================================

#[test]
fn t086_typescript_collection_operations_map_filter() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T086-typescript-collection-operation-edges",
        "collection_map_filter.ts",
    );

    println!("\n=== Collection Operations (map/filter) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect collection operations
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("map"));
    let filter_edges = edges.iter().any(|e| e.to_key.as_str().contains("filter"));

    assert!(map_edges, "Expected edge for map operation");
    assert!(filter_edges, "Expected edge for filter operation");
}

#[test]
fn t086_typescript_collection_operations_chained() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T086-typescript-collection-operation-edges",
        "collection_chained.ts",
    );

    println!("\n=== Collection Operations (Chained) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect chained collection operations
    let has_filter = edges.iter().any(|e| e.to_key.as_str().contains("filter"));
    let has_map = edges.iter().any(|e| e.to_key.as_str().contains("map"));
    let has_reduce = edges.iter().any(|e| e.to_key.as_str().contains("reduce"));

    assert!(has_filter || has_map || has_reduce, "Expected edges for chained operations");
}

// ============================================================================
// T088: Async/Await Edges
// ============================================================================

#[test]
fn t088_typescript_async_await_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T088-typescript-async-await-edges",
        "async_basic.ts",
    );

    println!("\n=== Async/Await (Basic) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect async function calls
    let fetch_edges = edges.iter().any(|e| e.to_key.as_str().contains("fetchData"));
    let get_edges = edges.iter().any(|e| e.to_key.as_str().contains("getUser"));

    assert!(fetch_edges, "Expected edge for fetchData async call");
    assert!(get_edges, "Expected edge for getUser async call");
}

#[test]
fn t088_typescript_promise_operations() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T088-typescript-async-await-edges",
        "promise_operations.ts",
    );

    println!("\n=== Promise Operations Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect Promise chain operations
    let fetch_edges = edges.iter().any(|e| e.to_key.as_str().contains("fetchData"));
    let then_edges = edges.iter().any(|e| e.to_key.as_str().contains("handleSuccess") || e.to_key.as_str().contains("then"));
    let catch_edges = edges.iter().any(|e| e.to_key.as_str().contains("handleError") || e.to_key.as_str().contains("catch"));

    assert!(fetch_edges, "Expected edge for fetchData call");
    assert!(then_edges || catch_edges, "Expected edges for Promise operations");
}

// ============================================================================
// T090: Generic Type Edges
// ============================================================================

#[test]
fn t090_typescript_generic_types_annotations() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T090-typescript-generic-type-edges",
        "generic_annotations.ts",
    );

    println!("\n=== Generic Types (Annotations) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect generic type references
    let array_edges = edges.iter().any(|e| e.to_key.as_str().contains("Array"));
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("Map"));

    assert!(array_edges, "Expected edge for Array generic type");
    assert!(map_edges, "Expected edge for Map generic type");
}

#[test]
fn t090_typescript_generic_function_return() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T090-typescript-generic-type-edges",
        "generic_function.ts",
    );

    println!("\n=== Generic Function Return Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect generic type references in function signature
    let array_edges = edges.iter().any(|e| e.to_key.as_str().contains("Array"));
    let set_edges = edges.iter().any(|e| e.to_key.as_str().contains("Set"));

    assert!(array_edges || set_edges, "Expected edges for generic types in function signature");
}

// ============================================================================
// T092: Integration (All Patterns)
// ============================================================================

#[test]
fn t092_typescript_comprehensive_integration() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T092-typescript-integration-all-patterns",
        "integration.ts",
    );

    println!("\n=== Comprehensive Integration Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should have edges for multiple patterns
    assert!(edges.len() >= 5, "Expected at least 5 edges from various patterns, found {}", edges.len());
}
