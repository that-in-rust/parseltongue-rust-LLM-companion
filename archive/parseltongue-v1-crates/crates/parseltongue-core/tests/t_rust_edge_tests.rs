// Rust Dependency Edge Tests (T020-T039)
//
// Tests for Rust dependency edge detection using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;

// ============================================================================
// T020: Async/Await Edges
// ============================================================================

#[test]
fn t020_rust_async_await_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T020-rust-async-await-edges",
        "async_basic.rs",
    );

    let await_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("fetch_data")
                || e.to_key.to_string().contains("get")
        })
        .collect();

    assert!(
        await_edges.len() >= 2,
        "Expected at least 2 await edges (fetch_data, get), found: {}",
        await_edges.len()
    );
}

#[test]
fn t020_rust_async_await_error() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T020-rust-async-await-edges",
        "async_error.rs",
    );

    let await_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("fetch")
                || e.to_key.to_string().contains("save_data")
        })
        .collect();

    assert!(
        await_edges.len() >= 2,
        "Expected at least 2 await edges with error handling, found: {}",
        await_edges.len()
    );
}

#[test]
fn t020_rust_async_await_chain() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T020-rust-async-await-edges",
        "async_chain.rs",
    );

    let await_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("fetch_user")
                || e.to_key.to_string().contains("process")
        })
        .collect();

    assert!(
        await_edges.len() >= 2,
        "Expected at least 2 chained await edges, found: {}",
        await_edges.len()
    );
}

// ============================================================================
// T022: Field Access Edges
// ============================================================================

#[test]
fn t022_rust_field_access_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T022-rust-field-access-edges",
        "field_simple.rs",
    );

    let field_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("name")
                || e.to_key.to_string().contains("age")
        })
        .collect();

    assert!(
        field_edges.len() >= 2,
        "Expected at least 2 field access edges (name, age), found: {}",
        field_edges.len()
    );
}

#[test]
fn t022_rust_field_access_assignment() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T022-rust-field-access-edges",
        "field_assignment.rs",
    );

    let field_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("age")
                || e.to_key.to_string().contains("name")
        })
        .collect();

    assert!(
        field_edges.len() >= 2,
        "Expected at least 2 field assignment edges, found: {}",
        field_edges.len()
    );
}

#[test]
fn t022_rust_field_access_nested() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T022-rust-field-access-edges",
        "field_nested.rs",
    );

    let field_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("settings")
                || e.to_key.to_string().contains("timeout")
                || e.to_key.to_string().contains("database")
                || e.to_key.to_string().contains("connection")
                || e.to_key.to_string().contains("host")
        })
        .collect();

    assert!(
        field_edges.len() >= 3,
        "Expected at least 3 nested field access edges, found: {}",
        field_edges.len()
    );
}

// ============================================================================
// T024: Iterator Method Edges
// ============================================================================

#[test]
fn t024_rust_iterator_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T024-rust-iterator-method-edges",
        "iter_basic.rs",
    );

    let iterator_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("iter")
                || to_key_str.contains("filter")
                || to_key_str.contains("map")
                || to_key_str.contains("collect")
        })
        .collect();

    assert!(
        iterator_edges.len() >= 4,
        "Expected at least 4 iterator method edges (iter, filter, map, collect), found: {}",
        iterator_edges.len()
    );
}

#[test]
fn t024_rust_iterator_into_iter() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T024-rust-iterator-method-edges",
        "iter_into.rs",
    );

    let iterator_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("into_iter")
                || to_key_str.contains("filter")
                || to_key_str.contains("sum")
        })
        .collect();

    assert!(
        iterator_edges.len() >= 3,
        "Expected at least 3 iterator edges (into_iter, filter, sum), found: {}",
        iterator_edges.len()
    );
}

#[test]
fn t024_rust_iterator_aggregation() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T024-rust-iterator-method-edges",
        "iter_aggregation.rs",
    );

    let iterator_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("iter")
                || to_key_str.contains("max")
                || to_key_str.contains("min")
                || to_key_str.contains("count")
        })
        .collect();

    assert!(
        iterator_edges.len() >= 6, // 3 iter() + 3 aggregation methods
        "Expected at least 6 iterator edges (3x iter, max, min, count), found: {}",
        iterator_edges.len()
    );
}

#[test]
fn t024_rust_iterator_find_any() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T024-rust-iterator-method-edges",
        "iter_find_any.rs",
    );

    let iterator_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("find")
                || to_key_str.contains("any")
                || to_key_str.contains("all")
        })
        .collect();

    assert!(
        iterator_edges.len() >= 3,
        "Expected at least 3 search iterator edges (find, any, all), found: {}",
        iterator_edges.len()
    );
}

#[test]
fn t024_rust_iterator_fold() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T024-rust-iterator-method-edges",
        "iter_fold.rs",
    );

    let iterator_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("fold")
        })
        .collect();

    assert!(
        iterator_edges.len() >= 1,
        "Expected at least 1 fold edge, found: {}",
        iterator_edges.len()
    );
}

// ============================================================================
// T026: Generic Type Edges
// ============================================================================

#[test]
fn t026_rust_generic_types_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T026-rust-generic-type-edges",
        "generic_basic.rs",
    );

    let generic_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("Vec")
                || to_key_str.contains("HashMap")
                || to_key_str.contains("String")
                || to_key_str.contains("User")
        })
        .collect();

    assert!(
        generic_edges.len() >= 2,
        "Expected at least 2 generic type edges (Vec, HashMap), found: {}",
        generic_edges.len()
    );
}

#[test]
fn t026_rust_generic_types_nested() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T026-rust-generic-type-edges",
        "generic_nested.rs",
    );

    let generic_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("Vec")
                || to_key_str.contains("HashMap")
                || to_key_str.contains("Item")
        })
        .collect();

    assert!(
        generic_edges.len() >= 3,
        "Expected at least 3 nested generic edges, found: {}",
        generic_edges.len()
    );
}

#[test]
fn t026_rust_generic_types_custom() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T026-rust-generic-type-edges",
        "generic_custom.rs",
    );

    let generic_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("Result")
                || to_key_str.contains("Option")
                || to_key_str.contains("User")
                || to_key_str.contains("Error")
                || to_key_str.contains("Data")
        })
        .collect();

    assert!(
        generic_edges.len() >= 2,
        "Expected at least 2 custom generic edges (Result, Option), found: {}",
        generic_edges.len()
    );
}

// ============================================================================
// T028: Integration Test - All Patterns
// ============================================================================

#[test]
fn t028_rust_integration_all() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T028-rust-integration-all-patterns",
        "combined.rs",
    );

    // Should detect:
    // - Generic types: Vec<User>, Vec<String>, HashMap<String, String>
    // - Iterator methods: iter, filter, map, collect
    // - Async/await: fetch_data().await
    // - Field access: u.active, u.name, data.field_value

    assert!(
        edges.len() >= 8,
        "Expected at least 8 edges from integration test, found: {}",
        edges.len()
    );
}
