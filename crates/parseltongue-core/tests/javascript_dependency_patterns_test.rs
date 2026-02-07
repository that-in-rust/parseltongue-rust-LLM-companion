// JavaScript Dependency Pattern Tests (v1.4.9)
// REQ-JAVASCRIPT-001.0 through REQ-JAVASCRIPT-005.0
// Following TDD: RED -> GREEN -> REFACTOR
//
// P1 HIGH: JavaScript missing constructor, property access, async, array methods, promises

use parseltongue_core::query_extractor::QueryBasedExtractor;
use parseltongue_core::entities::Language;
use std::path::Path;

// ============================================================================
// Helper Function
// ============================================================================

fn parse_javascript_code_extract_edges(code: &str) -> Vec<parseltongue_core::entities::DependencyEdge> {
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, Path::new("test.js"), Language::JavaScript)
        .expect("Failed to parse JavaScript code");
    edges
}

// ============================================================================
// REQ-JAVASCRIPT-001.0: Constructor Calls (new_expression)
// ============================================================================

#[test]
fn test_javascript_constructor_new_basic() {
    let code = r#"
function create() {
    const user = new User();
    const date = new Date();
    const map = new Map();
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
fn test_javascript_constructor_new_qualified() {
    let code = r#"
function build() {
    const user = new Models.User();
    const logger = new Utils.Logger();
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
// REQ-JAVASCRIPT-002.0: Property Access (member_expression non-call)
// ============================================================================

#[test]
fn test_javascript_property_access_read() {
    let code = r#"
function process(user) {
    const name = user.name;
    const age = user.age;
    const email = user.email;
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
fn test_javascript_property_access_nested() {
    let code = r#"
function extract(obj) {
    const value = obj.config.database.host;
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
// REQ-JAVASCRIPT-003.0: Async/Await
// ============================================================================

#[test]
fn test_javascript_async_await_basic() {
    let code = r#"
async function loadData() {
    const response = await fetch('/api/data');
    const json = await response.json();
    await saveToDb(json);
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
fn test_javascript_async_await_method() {
    let code = r#"
class Fetcher {
    async load() {
        const data = await this.fetchData();
        return await this.processData(data);
    }
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
// REQ-JAVASCRIPT-004.0: Array Methods (Collection Operations)
// ============================================================================

#[test]
fn test_javascript_array_methods_map_filter() {
    let code = r#"
function processItems(items) {
    return items
        .filter(x => x.active)
        .map(x => x.name);
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
fn test_javascript_array_methods_reduce() {
    let code = r#"
function sum(numbers) {
    return numbers.reduce((acc, num) => acc + num, 0);
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

    println!("\n=== JavaScript Array Methods (reduce) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    let reduce_edge = edges.iter().any(|e| e.to_key.as_str().contains("reduce"));

    assert!(reduce_edge, "Expected edge for reduce method");
}

#[test]
fn test_javascript_array_methods_find_some_every() {
    let code = r#"
function check(items) {
    const found = items.find(x => x.id === 1);
    const hasSome = items.some(x => x.active);
    const allValid = items.every(x => x.valid);
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
// REQ-JAVASCRIPT-005.0: Promise Chains
// ============================================================================

#[test]
fn test_javascript_promise_then_catch() {
    let code = r#"
function fetchUser(id) {
    return fetch('/api/user/' + id)
        .then(res => res.json())
        .catch(err => console.error(err));
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
fn test_javascript_promise_finally() {
    let code = r#"
function request() {
    return api.call()
        .then(handleSuccess)
        .catch(handleError)
        .finally(cleanup);
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
// Integration Test: Full Dependency Graph
// ============================================================================

#[test]
fn test_javascript_edge_integration_comprehensive() {
    let code = r#"
class DataService {
    async fetchAndProcess(id) {
        // Constructor call
        const logger = new Logger();

        // Async/await
        const response = await fetch('/api/data/' + id);
        const data = await response.json();

        // Property access
        const items = data.items;

        // Array methods
        const processed = items
            .filter(x => x.active)
            .map(x => x.value);

        // Promise chain
        return this.save(processed)
            .then(result => result.id)
            .catch(err => console.error(err));
    }
}
"#;

    let edges = parse_javascript_code_extract_edges(code);

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
