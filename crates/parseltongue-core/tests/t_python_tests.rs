// Python Dependency Edge Tests (T040-T059)
//
// Tests for Python dependency edge detection using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;

// ============================================================================
// T040: Constructor Call Edges
// ============================================================================

#[test]
fn t040_python_constructor_class_call() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T040-python-constructor-call-edges",
        "class_call.py",
    );

    println!("=== Constructor Call Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    // Verify constructor edges exist
    let constructor_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            let to_key_str = e.to_key.to_string();
            to_key_str.contains("User")
                || to_key_str.contains("Person")
                || to_key_str.contains("Configuration")
        })
        .collect();

    assert!(
        constructor_edges.len() >= 3,
        "Expected at least 3 constructor edges (User, Person, Configuration), found: {}",
        constructor_edges.len()
    );
}

#[test]
fn t040_python_constructor_qualified() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T040-python-constructor-call-edges",
        "qualified_constructor.py",
    );

    let qualified_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("User") || e.to_key.to_string().contains("Connection"))
        .collect();

    assert!(
        qualified_edges.len() >= 2,
        "Expected at least 2 qualified constructor edges, found: {}",
        qualified_edges.len()
    );
}

// ============================================================================
// T042: Attribute Access Edges
// ============================================================================

#[test]
fn t042_python_attribute_access() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T042-python-attribute-access-edges",
        "attribute_access.py",
    );

    println!("=== Attribute Access Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    let attribute_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("name")
                || e.to_key.to_string().contains("age")
                || e.to_key.to_string().contains("parent")
                || e.to_key.to_string().contains("child")
        })
        .collect();

    assert!(
        attribute_edges.len() >= 2,
        "Expected at least 2 attribute access edges, found: {}",
        attribute_edges.len()
    );
}

#[test]
fn t042_python_property_getter_setter() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T042-python-attribute-access-edges",
        "property_getter_setter.py",
    );

    let property_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("setting")
                || e.to_key.to_string().contains("config")
                || e.to_key.to_string().contains("data")
        })
        .collect();

    assert!(
        property_edges.len() >= 2,
        "Expected at least 2 property access edges, found: {}",
        property_edges.len()
    );
}

// ============================================================================
// T044: Async/Await Edges
// ============================================================================

#[test]
fn t044_python_async_await() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T044-python-async-await-edges",
        "async_basic.py",
    );

    println!("=== Async/Await Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    let async_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("fetch_data") || e.to_key.to_string().contains("save_async"))
        .collect();

    assert!(
        async_edges.len() >= 2,
        "Expected at least 2 async call edges, found: {}",
        async_edges.len()
    );
}

#[test]
fn t044_python_async_class_method() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T044-python-async-await-edges",
        "async_class_method.py",
    );

    let async_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("get_user") || e.to_key.to_string().contains("process_data"))
        .collect();

    assert!(
        async_edges.len() >= 2,
        "Expected at least 2 async method edges, found: {}",
        async_edges.len()
    );
}

// ============================================================================
// T045: Decorator Dependency Edges
// ============================================================================

#[test]
fn t045_python_decorators() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T045-python-decorator-dependency-edges",
        "class_decorators.py",
    );

    println!("=== Decorator Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    let decorator_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("property")
                || e.to_key.to_string().contains("staticmethod")
                || e.to_key.to_string().contains("route")
        })
        .collect();

    assert!(
        decorator_edges.len() >= 2,
        "Expected at least 2 decorator edges, found: {}",
        decorator_edges.len()
    );
}

#[test]
fn t045_python_function_decorators() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T045-python-decorator-dependency-edges",
        "function_decorators.py",
    );

    let decorator_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.to_string().contains("login_required") || e.to_key.to_string().contains("validate_input"))
        .collect();

    assert!(
        decorator_edges.len() >= 2,
        "Expected at least 2 function decorator edges, found: {}",
        decorator_edges.len()
    );
}

// ============================================================================
// T046: Type Hint Edges
// ============================================================================

#[test]
fn t046_python_type_hints() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T046-python-type-hint-edges",
        "type_hints.py",
    );

    println!("=== Type Hints Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    let type_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("List")
                || e.to_key.to_string().contains("Dict")
                || e.to_key.to_string().contains("User")
        })
        .collect();

    assert!(
        type_edges.len() >= 2,
        "Expected at least 2 type hint edges, found: {}",
        type_edges.len()
    );
}

#[test]
fn t046_python_optional_types() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T046-python-type-hint-edges",
        "optional_types.py",
    );

    let type_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("Optional")
                || e.to_key.to_string().contains("Union")
                || e.to_key.to_string().contains("User")
        })
        .collect();

    assert!(
        type_edges.len() >= 2,
        "Expected at least 2 optional type edges, found: {}",
        type_edges.len()
    );
}

// ============================================================================
// T048: Comprehension Call Edges
// ============================================================================

#[test]
fn t048_python_list_comprehensions() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T048-python-comprehension-call-edges",
        "list_comprehension.py",
    );

    println!("=== List Comprehension Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key, edge.to_key);
    }

    let comprehension_edges: Vec<_> = edges
        .iter()
        .filter(|e| {
            e.to_key.to_string().contains("process")
                || e.to_key.to_string().contains("validate")
                || e.to_key.to_string().contains("check")
        })
        .collect();

    assert!(
        comprehension_edges.len() >= 2,
        "Expected at least 2 comprehension call edges, found: {}",
        comprehension_edges.len()
    );
}

// ============================================================================
// T050: Integration Test - All Patterns
// ============================================================================

#[test]
fn t050_python_edge_integration() {
    let (entities, edges) = parse_fixture_extract_results(
        "T050-python-integration-all-patterns",
        "integration.py",
    );

    println!("=== Integration Test ===");
    println!("Entities: {}", entities.len());
    println!("Edges: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key, edge.to_key);
    }

    // Should detect:
    // - Type hints: List, User
    // - Decorators: property
    // - Async calls: save_user
    // - Attribute access: user.name
    // - Constructor calls: Logger()

    assert!(
        edges.len() >= 5,
        "Expected at least 5 edges from integration test, found: {}",
        edges.len()
    );
}
