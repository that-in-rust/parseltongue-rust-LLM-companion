// Java Dependency Edge Tests (T120-T139)
//
// Tests for Java dependency edge detection using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;

// ============================================================================
// T120: Constructor Call Edges
// ============================================================================

#[test]
fn t120_java_constructor_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T120-java-constructor-call-edges",
        "simple.java",
    );

    println!("\n=== Java Constructor (Simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect constructor calls to Person and DataModel
    let person_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Person"));
    let model_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("DataModel"));

    assert!(person_edges, "Expected edge for Person constructor");
    assert!(model_edges, "Expected edge for DataModel constructor");
}

#[test]
fn t120_java_constructor_generic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T120-java-constructor-call-edges",
        "generic.java",
    );

    println!("\n=== Java Constructor (Generic) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect generic constructor calls
    let arraylist_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("ArrayList"));
    let hashmap_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("HashMap"));

    assert!(arraylist_edges, "Expected edge for ArrayList constructor");
    assert!(hashmap_edges, "Expected edge for HashMap constructor");
}

// ============================================================================
// T122: Stream Operation Edges
// ============================================================================

#[test]
fn t122_java_stream_operations() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T122-java-stream-operation-edges",
        "stream.java",
    );

    println!("\n=== Java Stream Operations Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect stream, filter, map, collect operations
    let stream_edges = edges.iter().any(|e| e.to_key.as_str().contains("stream"));
    let filter_edges = edges.iter().any(|e| e.to_key.as_str().contains("filter"));
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("map"));
    let collect_edges = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("collect"));

    assert!(stream_edges, "Expected edge for stream operation");
    assert!(filter_edges, "Expected edge for filter operation");
    assert!(map_edges, "Expected edge for map operation");
    assert!(collect_edges, "Expected edge for collect operation");
}

// ============================================================================
// T124: Generic Type Edges
// ============================================================================

#[test]
fn t124_java_generic_type_variable() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T124-java-generic-type-edges",
        "types.java",
    );

    println!("\n=== Java Generic Types Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect generic type references
    let list_edges = edges.iter().any(|e| e.to_key.as_str().contains("List"));
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("Map"));
    let set_edges = edges.iter().any(|e| e.to_key.as_str().contains("Set"));

    assert!(list_edges, "Expected edge for List generic type");
    assert!(map_edges, "Expected edge for Map generic type");
    assert!(set_edges, "Expected edge for Set generic type");
}

// ============================================================================
// T126: Annotation Dependency Edges
// ============================================================================

#[test]
fn t126_java_annotations() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T126-java-annotation-dependency-edges",
        "annotations.java",
    );

    println!("\n=== Java Annotations Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect annotation usage
    let entity_edges = edges.iter().any(|e| e.to_key.as_str().contains("Entity"));
    let has_annotations = edges.iter().any(|e| {
        e.to_key.as_str().contains("Entity")
            || e.to_key.as_str().contains("Id")
            || e.to_key.as_str().contains("Override")
    });

    assert!(entity_edges, "Expected edge for Entity annotation");
    assert!(has_annotations, "Expected at least one annotation edge");
    // Note: @Override may not be captured as it's a built-in annotation without explicit import
}

// ============================================================================
// T128: Integration Test - All Patterns
// ============================================================================

#[test]
fn t128_java_integration_complex_service() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T128-java-integration-all-patterns",
        "service.java",
    );

    println!("\n=== Java Integration Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect various patterns:
    // 1. Constructor calls: new UserRepository(), new User()
    // 2. Field access: user.name
    // 3. Method calls: findAll(), stream(), filter(), map(), collect(), save()

    assert!(
        edges.len() >= 5,
        "Expected at least 5 edges from complex service"
    );

    // Check for key dependencies
    let has_constructor = edges.iter().any(|e| {
        e.to_key.as_str().contains("UserRepository") || e.to_key.as_str().contains("User")
    });
    let has_stream_ops = edges.iter().any(|e| {
        e.to_key.as_str().contains("stream")
            || e.to_key.as_str().contains("filter")
            || e.to_key.as_str().contains("map")
    });

    assert!(has_constructor, "Expected constructor call edges");
    assert!(has_stream_ops, "Expected stream operation edges");
}

// ============================================================================
// T121: Field Access Edges
// ============================================================================

#[test]
fn t121_java_field_access_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T121-java-field-access-edges",
        "field_simple.java",
    );

    println!("\n=== Java Field Access (Method Calls) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Java typically uses getters/setters, which are method calls
    // Field access nodes may not always be captured as separate dependencies
    let getter_edges = edges.iter().any(|e| {
        e.to_key.as_str().contains("getSetting") || e.to_key.as_str().contains("getPort")
    });

    assert!(
        getter_edges,
        "Expected edges for getter method calls (field access pattern)"
    );
}
