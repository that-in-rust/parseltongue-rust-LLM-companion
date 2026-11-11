// JavaScript Dependency Edge Tests (T060-T079)
//
// Tests for JavaScript dependency edge detection using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;

// ============================================================================
// T060: Constructor New Edges
// ============================================================================

#[test]
fn t060_javascript_constructor_new_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T060-javascript-constructor-new-edges",
        "constructor_basic.js",
    );

    println!("\n=== JavaScript Constructor (new_expression) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect constructor calls
    let user_edge = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let date_edge = edges.iter().any(|e| e.to_key.as_str().contains("Date"));
    let map_edge = edges.iter().any(|e| e.to_key.as_str().contains("Map"));

    assert!(user_edge, "Expected edge for User constructor");
    assert!(date_edge, "Expected edge for Date constructor");
    assert!(map_edge, "Expected edge for Map constructor");
}

#[test]
fn t060_javascript_constructor_new_qualified() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T060-javascript-constructor-new-edges",
        "constructor_qualified.js",
    );

    println!("\n=== JavaScript Constructor (qualified) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let user_edge = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let logger_edge = edges.iter().any(|e| e.to_key.as_str().contains("Logger"));

    assert!(user_edge, "Expected edge for qualified User constructor");
    assert!(logger_edge, "Expected edge for qualified Logger constructor");
}

// ============================================================================
// T062: Property Access Edges
// ============================================================================

#[test]
fn t062_javascript_property_access_read() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T062-javascript-property-access-edges",
        "property_read.js",
    );

    println!("\n=== JavaScript Property Access (read) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let name_edge = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edge = edges.iter().any(|e| e.to_key.as_str().contains("age"));
    let email_edge = edges.iter().any(|e| e.to_key.as_str().contains("email"));

    assert!(name_edge, "Expected edge for name property access");
    assert!(age_edge, "Expected edge for age property access");
    assert!(email_edge, "Expected edge for email property access");
}

#[test]
fn t062_javascript_property_access_nested() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T062-javascript-property-access-edges",
        "property_nested.js",
    );

    println!("\n=== JavaScript Property Access (nested) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect nested property access
    let has_property_access = edges.iter().any(|e| {
        e.to_key.as_str().contains("config")
        || e.to_key.as_str().contains("database")
        || e.to_key.as_str().contains("host")
    });

    assert!(has_property_access, "Expected nested property access edges");
}

// ============================================================================
// T064: Async/Await Edges
// ============================================================================

#[test]
fn t064_javascript_async_await_basic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T064-javascript-async-await-edges",
        "async_basic.js",
    );

    println!("\n=== JavaScript Async/Await Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let fetch_edge = edges.iter().any(|e| e.to_key.as_str().contains("fetch"));
    let json_edge = edges.iter().any(|e| e.to_key.as_str().contains("json"));
    let save_edge = edges.iter().any(|e| e.to_key.as_str().contains("saveToDb"));

    assert!(fetch_edge, "Expected edge for await fetch call");
    assert!(json_edge, "Expected edge for await json call");
    assert!(save_edge, "Expected edge for await saveToDb call");
}

#[test]
fn t064_javascript_async_await_method() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T064-javascript-async-await-edges",
        "async_method.js",
    );

    println!("\n=== JavaScript Async Method Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let fetch_edge = edges.iter().any(|e| e.to_key.as_str().contains("fetchData"));
    let process_edge = edges.iter().any(|e| e.to_key.as_str().contains("processData"));

    assert!(fetch_edge, "Expected edge for await fetchData");
    assert!(process_edge, "Expected edge for await processData");
}

// ============================================================================
// T066: Array Method Edges
// ============================================================================

#[test]
fn t066_javascript_array_methods_map_filter() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T066-javascript-array-method-edges",
        "array_map_filter.js",
    );

    println!("\n=== JavaScript Array Methods (map/filter) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let filter_edge = edges.iter().any(|e| e.to_key.as_str().contains("filter"));
    let map_edge = edges.iter().any(|e| e.to_key.as_str().contains("map"));

    assert!(filter_edge, "Expected edge for filter method");
    assert!(map_edge, "Expected edge for map method");
}

#[test]
fn t066_javascript_array_methods_reduce() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T066-javascript-array-method-edges",
        "array_reduce.js",
    );

    println!("\n=== JavaScript Array Methods (reduce) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let reduce_edge = edges.iter().any(|e| e.to_key.as_str().contains("reduce"));

    assert!(reduce_edge, "Expected edge for reduce method");
}

#[test]
fn t066_javascript_array_methods_find_some_every() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T066-javascript-array-method-edges",
        "array_find_some_every.js",
    );

    println!("\n=== JavaScript Array Methods (find/some/every) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let find_edge = edges.iter().any(|e| e.to_key.as_str().contains("find"));
    let some_edge = edges.iter().any(|e| e.to_key.as_str().contains("some"));
    let every_edge = edges.iter().any(|e| e.to_key.as_str().contains("every"));

    assert!(find_edge, "Expected edge for find method");
    assert!(some_edge, "Expected edge for some method");
    assert!(every_edge, "Expected edge for every method");
}

// ============================================================================
// T068: Promise Chain Edges
// ============================================================================

#[test]
fn t068_javascript_promise_then_catch() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T068-javascript-promise-chain-edges",
        "promise_then_catch.js",
    );

    println!("\n=== JavaScript Promise (then/catch) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let fetch_edge = edges.iter().any(|e| e.to_key.as_str().contains("fetch"));
    let then_edge = edges.iter().any(|e| e.to_key.as_str().contains("then"));
    let catch_edge = edges.iter().any(|e| e.to_key.as_str().contains("catch"));

    assert!(fetch_edge, "Expected edge for fetch call");
    assert!(then_edge, "Expected edge for then method");
    assert!(catch_edge, "Expected edge for catch method");
}

#[test]
fn t068_javascript_promise_finally() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T068-javascript-promise-chain-edges",
        "promise_finally.js",
    );

    println!("\n=== JavaScript Promise (finally) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let then_edge = edges.iter().any(|e| e.to_key.as_str().contains("then"));
    let catch_edge = edges.iter().any(|e| e.to_key.as_str().contains("catch"));
    let finally_edge = edges.iter().any(|e| e.to_key.as_str().contains("finally"));

    assert!(then_edge, "Expected edge for then method");
    assert!(catch_edge, "Expected edge for catch method");
    assert!(finally_edge, "Expected edge for finally method");
}

// ============================================================================
// T070: Integration Test - All Patterns
// ============================================================================

#[test]
fn t070_javascript_edge_integration_comprehensive() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T070-javascript-integration-all-patterns",
        "integration.js",
    );

    println!("\n=== JavaScript Integration Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect all pattern types
    let has_constructor = edges.iter().any(|e| e.to_key.as_str().contains("Logger"));
    let has_async = edges.iter().any(|e| e.to_key.as_str().contains("fetch"));
    let has_property = edges.iter().any(|e| e.to_key.as_str().contains("items"));
    let has_array_methods = edges.iter().any(|e| {
        e.to_key.as_str().contains("filter") || e.to_key.as_str().contains("map")
    });
    let has_promise = edges.iter().any(|e| {
        e.to_key.as_str().contains("then") || e.to_key.as_str().contains("catch")
    });

    assert!(has_constructor, "Missing constructor edge");
    assert!(has_async, "Missing async/await edge");
    assert!(has_property, "Missing property access edge");
    assert!(has_array_methods, "Missing array method edges");
    assert!(has_promise, "Missing promise chain edges");

    // Should have a reasonable number of total edges
    assert!(
        edges.len() >= 10,
        "Expected at least 10 edges for comprehensive test, found {}",
        edges.len()
    );
}
