// TypeScript Dependency Pattern Tests (v1.4.9)
// REQ-TYPESCRIPT-001.0 through REQ-TYPESCRIPT-005.0
// Following TDD: RED -> GREEN -> REFACTOR

use parseltongue_core::query_extractor::QueryBasedExtractor;
use parseltongue_core::entities::Language;
use std::path::Path;

// ============================================================================
// Helper Function
// ============================================================================

fn parse_typescript_code_extract_edges(code: &str) -> Vec<parseltongue_core::entities::DependencyEdge> {
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, Path::new("test.ts"), Language::TypeScript)
        .expect("Failed to parse TypeScript code");
    edges
}

// ============================================================================
// REQ-TYPESCRIPT-001.0: Constructor Calls (new expression)
// ============================================================================

#[test]
fn test_typescript_constructor_new_simple() {
    let code = r#"
class Manager {
    create() {
        const p = new Person();
        const m = new DataModel();
    }
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
fn test_typescript_constructor_new_generic() {
    let code = r#"
class Container {
    init() {
        const arr = new Array<string>();
        const map = new Map<string, number>();
    }
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
fn test_typescript_constructor_new_qualified() {
    let code = r#"
class Factory {
    build() {
        const user = new Models.User();
    }
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
// REQ-TYPESCRIPT-002.0: Method Calls
// ============================================================================

#[test]
fn test_typescript_method_calls_basic() {
    let code = r#"
class Service {
    process() {
        this.helper();
        obj.doWork();
    }
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
// REQ-TYPESCRIPT-003.0: Property Access
// ============================================================================

#[test]
fn test_typescript_property_access_simple() {
    let code = r#"
function process(user: User) {
    const name = user.name;
    user.age = 30;
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
fn test_typescript_property_access_chained() {
    let code = r#"
class Processor {
    process(sys: System) {
        const val = sys.config.value;
    }
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
// REQ-TYPESCRIPT-004.0: Collection Operations
// ============================================================================

#[test]
fn test_typescript_collection_operations_map_filter() {
    let code = r#"
class Transformer {
    transform(items: Item[]) {
        const names = items.map(x => x.name);
        const filtered = items.filter(x => x.active);
    }
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
fn test_typescript_collection_operations_chained() {
    let code = r#"
class Pipeline {
    process(data: Data[]) {
        const result = data
            .filter(x => x.valid)
            .map(x => x.value)
            .reduce((a, b) => a + b);
    }
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
// REQ-TYPESCRIPT-005.0: Async/Await
// ============================================================================

#[test]
fn test_typescript_async_await_basic() {
    let code = r#"
class Fetcher {
    async load() {
        const data = await fetchData();
        const user = await getUser();
    }
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
fn test_typescript_promise_operations() {
    let code = r#"
class AsyncProcessor {
    process() {
        fetchData()
            .then(handleSuccess)
            .catch(handleError);
    }
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
// REQ-TYPESCRIPT-006.0: Generic Types
// ============================================================================

#[test]
fn test_typescript_generic_types_annotations() {
    let code = r#"
class Container<T> {
    items: Array<T>;
    map: Map<string, T>;
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
fn test_typescript_generic_function_return() {
    let code = r#"
function process<T>(items: Array<T>): Set<T> {
    return new Set(items);
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

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
// Integration Test: Comprehensive Pattern Coverage
// ============================================================================

#[test]
fn test_typescript_comprehensive_integration() {
    let code = r#"
class DataService {
    async fetchAndProcess() {
        // Constructor calls
        const list = new Array<string>();
        const model = new DataModel();

        // Property access
        const name = model.name;

        // Method calls
        this.helper();

        // Collection operations
        const filtered = list.filter(x => x.length > 0);

        // Async/await
        const data = await fetchData();

        return filtered;
    }

    helper() { }
}
"#;

    let edges = parse_typescript_code_extract_edges(code);

    println!("\n=== Comprehensive Integration Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should have edges for multiple patterns
    assert!(edges.len() >= 5, "Expected at least 5 edges from various patterns, found {}", edges.len());
}
