// Rust Dependency Pattern Tests (v1.4.10)
//
// Tests for comprehensive Rust dependency detection:
// 1. Async/await expressions (await operations)
// 2. Field access (non-call context like property reads)
// 3. Iterator/collection methods (iter, map, filter, collect)
// 4. Generic type usage (Vec<T>, HashMap<K, V>)
//
// TDD Phase: RED (tests written first, should fail)

use parseltongue_core::entities::{DependencyEdge, Language};
use parseltongue_core::query_extractor::{ParsedEntity, QueryBasedExtractor};
use std::path::Path;

/// Helper: Parse Rust code and extract entities + edges
fn extract_rust_dependencies(
    code: &str,
) -> (Vec<ParsedEntity>, Vec<DependencyEdge>) {
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    extractor
        .parse_source(code, Path::new("test.rs"), Language::Rust)
        .expect("Failed to parse Rust code")
}

// ============================================================================
// PATTERN 1: Async/Await Expressions
// ============================================================================

#[test]
fn test_rust_async_await_basic() {
    let code = r#"
async fn load_data() {
    let result = fetch_data().await;
    let json = client.get().await;
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

    println!("=== Async/Await Basic Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect await on both fetch_data() and client.get()
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
fn test_rust_async_await_with_error() {
    let code = r#"
async fn process() -> Result<(), Error> {
    let data = client.fetch().await?;
    let result = save_data(data).await?;
    Ok(())
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

    println!("=== Async/Await with Error Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

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
fn test_rust_async_method_chain() {
    let code = r#"
async fn complex_operation() {
    let value = api.fetch_user().await.process().await;
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

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
// PATTERN 2: Field Access (Non-call Context)
// ============================================================================

#[test]
fn test_rust_field_access_simple() {
    let code = r#"
fn process(user: &User) {
    let name = user.name;
    let age = user.age;
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

    println!("=== Field Access Simple Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

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
fn test_rust_field_access_assignment() {
    let code = r#"
fn update(user: &mut User) {
    user.age = 30;
    user.name = String::from("Alice");
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

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
fn test_rust_field_access_nested() {
    let code = r#"
fn get_value(config: &Config) {
    let timeout = config.settings.timeout;
    let host = config.database.connection.host;
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

    println!("=== Field Access Nested Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

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
// PATTERN 3: Iterator/Collection Methods
// ============================================================================

#[test]
fn test_rust_iterator_methods_basic() {
    let code = r#"
fn transform(items: Vec<Item>) -> Vec<String> {
    items.iter()
        .filter(|x| x.active)
        .map(|x| x.name.clone())
        .collect()
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

    println!("=== Iterator Methods Basic Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

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
fn test_rust_iterator_methods_into_iter() {
    let code = r#"
fn process(vec: Vec<u32>) -> u32 {
    vec.into_iter()
        .filter(|&x| x > 0)
        .sum()
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

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
fn test_rust_iterator_methods_aggregation() {
    let code = r#"
fn analyze(data: Vec<i32>) -> (i32, i32, usize) {
    let max = data.iter().max();
    let min = data.iter().min();
    let count = data.iter().count();
    (max.unwrap(), min.unwrap(), count)
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

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
fn test_rust_iterator_methods_find_any() {
    let code = r#"
fn search(items: &[Item]) -> bool {
    items.iter().find(|x| x.id == 10);
    items.iter().any(|x| x.active);
    items.iter().all(|x| x.valid);
    true
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

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
fn test_rust_iterator_methods_fold() {
    let code = r#"
fn calculate(values: Vec<i32>) -> i32 {
    values.iter().fold(0, |acc, x| acc + x)
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

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
// PATTERN 4: Generic Type Usage
// ============================================================================

#[test]
fn test_rust_generic_types_basic() {
    let code = r#"
fn setup() {
    let items: Vec<String> = Vec::new();
    let map: HashMap<String, User> = HashMap::new();
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

    println!("=== Generic Types Basic Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

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
fn test_rust_generic_types_nested() {
    let code = r#"
fn build() {
    let data: Vec<Vec<u32>> = vec![];
    let mapping: HashMap<String, Vec<Item>> = HashMap::new();
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

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
fn test_rust_generic_types_custom() {
    let code = r#"
fn process() {
    let result: Result<User, Error> = Ok(user);
    let option: Option<Data> = Some(data);
}
    "#;

    let (_entities, edges) = extract_rust_dependencies(code);

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
// INTEGRATION TEST: Multiple Patterns Together
// ============================================================================

#[test]
fn test_rust_patterns_integration() {
    let code = r#"
use std::collections::HashMap;

async fn process_users(users: Vec<User>) -> Result<HashMap<String, String>, Error> {
    let names: Vec<String> = users
        .iter()
        .filter(|u| u.active)
        .map(|u| u.name.clone())
        .collect();

    let data = fetch_data().await?;
    let result = data.field_value;

    Ok(HashMap::new())
}
    "#;

    let (entities, edges) = extract_rust_dependencies(code);

    println!("=== Integration Test ===");
    println!("Entities: {}", entities.len());
    println!("Edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

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
