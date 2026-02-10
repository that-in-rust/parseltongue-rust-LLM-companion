// C# Dependency Edge Tests (T200-T219)
//
// Tests for C# dependency edge detection using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;

// ============================================================================
// T200: C# Constructor Calls (new TypeName())
// ============================================================================

#[test]
fn t200_csharp_constructor_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T200-csharp-constructor-method-edges",
        "constructor_simple.cs",
    );

    println!("\n=== T200: C# Constructor Simple ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should find at least one edge for the constructor call
    let constructor_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.as_str().contains("DataModel"))
        .collect();

    assert!(
        !constructor_edges.is_empty(),
        "Expected at least one edge to DataModel constructor"
    );
}

#[test]
fn t200_csharp_constructor_generic() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T200-csharp-constructor-method-edges",
        "constructor_generic.cs",
    );

    println!("\n=== T200: C# Constructor Generic ===");
    println!("Edges found: {}", edges.len());

    // Should detect both constructors
    let has_list = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("List"));
    let has_dict = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Dictionary"));

    assert!(has_list, "Expected edge for List constructor");
    assert!(has_dict, "Expected edge for Dictionary constructor");
}

#[test]
fn t200_csharp_constructor_with_initializer() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T200-csharp-constructor-method-edges",
        "constructor_initializer.cs",
    );

    println!("\n=== T200: C# Constructor With Initializer ===");
    println!("Edges found: {}", edges.len());

    // Should detect constructor
    let person_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.as_str().contains("Person"))
        .collect();

    assert!(
        !person_edges.is_empty(),
        "Expected edge for Person constructor"
    );
}

#[test]
fn t200_csharp_constructor_fixture() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T200-csharp-constructor-method-edges",
        "constructor_fixture.cs",
    );

    println!("\n=== T200: C# Constructor Fixture ===");
    println!("Edges found: {}", edges.len());

    // Expected edges: List constructor, DataModel constructor, Add method
    let constructor_edges = edges
        .iter()
        .filter(|e| {
            e.to_key.as_str().contains("List") || e.to_key.as_str().contains("DataModel")
        })
        .count();

    assert!(
        constructor_edges >= 2,
        "Expected at least 2 constructor edges (List, DataModel), found {}",
        constructor_edges
    );
}

// ============================================================================
// T202: C# Property Access and LINQ Operations
// ============================================================================

#[test]
fn t202_csharp_property_access_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T202-csharp-property-linq-edges",
        "property_simple.cs",
    );

    println!("\n=== T202: C# Property Access Simple ===");
    println!("Edges found: {}", edges.len());

    // Should detect property access to Name and Age
    let name_edges = edges.iter().any(|e| e.to_key.as_str().contains("Name"));
    let age_edges = edges.iter().any(|e| e.to_key.as_str().contains("Age"));

    assert!(name_edges, "Expected edge for Name property access");
    assert!(age_edges, "Expected edge for Age property access");
}

#[test]
fn t202_csharp_property_assignment() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T202-csharp-property-linq-edges",
        "property_assignment.cs",
    );

    println!("\n=== T202: C# Property Assignment ===");
    println!("Edges found: {}", edges.len());

    // Should detect property assignments
    let age_edges = edges.iter().any(|e| e.to_key.as_str().contains("Age"));
    let status_edges = edges.iter().any(|e| e.to_key.as_str().contains("Status"));

    assert!(age_edges, "Expected edge for Age property assignment");
    assert!(status_edges, "Expected edge for Status property assignment");
}

#[test]
fn t202_csharp_linq_where_select() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T202-csharp-property-linq-edges",
        "linq_where_select.cs",
    );

    println!("\n=== T202: C# LINQ Where/Select ===");
    println!("Edges found: {}", edges.len());

    // Should detect LINQ methods
    let where_edges = edges.iter().any(|e| e.to_key.as_str().contains("Where"));
    let select_edges = edges.iter().any(|e| e.to_key.as_str().contains("Select"));
    let tolist_edges = edges.iter().any(|e| e.to_key.as_str().contains("ToList"));

    assert!(where_edges, "Expected edge for Where LINQ method");
    assert!(select_edges, "Expected edge for Select LINQ method");
    assert!(tolist_edges, "Expected edge for ToList LINQ method");
}

#[test]
fn t202_csharp_linq_aggregate_operations() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T202-csharp-property-linq-edges",
        "linq_aggregate.cs",
    );

    println!("\n=== T202: C# LINQ Aggregate ===");
    println!("Edges found: {}", edges.len());

    // Should detect aggregate LINQ methods
    let count_edges = edges.iter().any(|e| e.to_key.as_str().contains("Count"));
    let sum_edges = edges.iter().any(|e| e.to_key.as_str().contains("Sum"));
    let avg_edges = edges.iter().any(|e| e.to_key.as_str().contains("Average"));

    assert!(count_edges, "Expected edge for Count LINQ method");
    assert!(sum_edges, "Expected edge for Sum LINQ method");
    assert!(avg_edges, "Expected edge for Average LINQ method");
}

// ============================================================================
// T204: C# Edge Key Format (ISGL1 v2)
// ============================================================================

#[test]
fn t204_csharp_edge_key_format() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T204-csharp-edge-key-format",
        "edge_key_format.cs",
    );

    println!("\n=== T204: C# Edge Key Format ===");
    println!("Edges found: {}", edges.len());

    // Assert: At least one edge should be extracted
    assert!(
        !edges.is_empty(),
        "Expected at least one edge from C# method calls"
    );

    // Assert: Each edge from_key should use ISGL1 v2 format (except file-level edges)
    for edge in &edges {
        let from_key_str = edge.from_key.as_str();

        // Skip file-level edges - they use a different format
        if from_key_str.contains(":file:") {
            continue;
        }

        // Key should contain :T followed by timestamp
        assert!(
            from_key_str.contains(":T"),
            "Key should contain timestamp: {}",
            from_key_str
        );

        // Key should NOT contain line range separator '-' before timestamp
        let key_before_timestamp = &from_key_str[..from_key_str.rfind(":T").unwrap()];
        assert!(
            !key_before_timestamp.contains("-"),
            "Key should not contain line range separator '-' before timestamp: {}",
            from_key_str
        );

        // Semantic path should start with "__"
        let from_key_parts: Vec<&str> = from_key_str.split(':').collect();
        assert!(
            from_key_parts[3].starts_with("__"),
            "Semantic path should start with '__': {}",
            from_key_str
        );
    }
}

#[test]
fn t204_csharp_semantic_path() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T204-csharp-edge-key-format",
        "semantic_path.cs",
    );

    println!("\n=== T204: C# Semantic Path ===");
    println!("Edges found: {}", edges.len());

    // Assert: Edge from_key contains semantic path
    if let Some(edge) = edges.first() {
        let from_key = edge.from_key.as_str();
        // Should contain semantic path component
        assert!(
            from_key.contains("__"),
            "Edge from_key should contain semantic path starting with '__': {}",
            from_key
        );
    }
}

// ============================================================================
// T206: C# Integration Validation
// ============================================================================

#[test]
fn t206_csharp_complete_integration() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T206-csharp-integration-validation-test",
        "complete_integration.cs",
    );

    println!("\n=== T206: C# Complete Integration ===");
    println!("Total edges found: {}", edges.len());
    for (i, edge) in edges.iter().enumerate() {
        println!("  {}. {} -> {}", i + 1, edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Assert: All edges use ISGL1 v2 format (except file-level edges)
    for edge in &edges {
        let from_key = edge.from_key.as_str();

        // Skip file-level edges
        if from_key.contains(":file:") {
            continue;
        }

        // Verify ISGL1 v2 format
        assert!(
            from_key.contains(":T"),
            "Key should contain timestamp: {}",
            from_key
        );
        assert!(
            !from_key[..from_key.rfind(":T").unwrap()].contains("-"),
            "Key should not contain line ranges: {}",
            from_key
        );
        assert!(
            from_key.split(':').nth(3).unwrap().starts_with("__"),
            "Semantic path should start with __: {}",
            from_key
        );
    }

    // Count edge types
    let constructor_edges = edges
        .iter()
        .filter(|e| {
            e.to_key.as_str().contains("List") || e.to_key.as_str().contains("DataModel")
        })
        .count();

    let method_edges = edges
        .iter()
        .filter(|e| {
            let to_key = e.to_key.as_str();
            to_key.contains("Helper")
                || to_key.contains("WriteLine")
                || to_key.contains("Add")
        })
        .count();

    // Assert: Expected edge counts
    assert!(
        constructor_edges >= 2,
        "Expected at least 2 constructor edges (List, DataModel)"
    );
    assert!(
        method_edges >= 3,
        "Expected at least 3 method call edges (Helper, WriteLine, Add)"
    );
    assert!(
        edges.len() >= 5,
        "Expected at least 5 total edges, found {}",
        edges.len()
    );
}

#[test]
fn t206_csharp_accuracy_test() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T206-csharp-integration-validation-test",
        "accuracy_test.cs",
    );

    println!("\n=== T206: C# Accuracy Test ===");

    // Expected dependencies:
    // 1. DoWork -> Helper (method call)
    // 2. Helper -> WriteLine (method call)
    // 3. Create -> List (constructor)
    // 4. Create -> DataModel (constructor)
    // 5. Create -> Add (method call)

    let expected_targets = vec!["Helper", "WriteLine", "List", "DataModel", "Add"];
    let mut found_targets = Vec::new();

    for edge in &edges {
        let to_key = edge.to_key.as_str();
        for target in &expected_targets {
            if to_key.contains(target) {
                found_targets.push(*target);
                break;
            }
        }
    }

    let accuracy = (found_targets.len() as f64 / expected_targets.len() as f64) * 100.0;

    println!("Expected targets: {:?}", expected_targets);
    println!("Found targets: {:?}", found_targets);
    println!("Accuracy: {:.0}% ({}/{})", accuracy, found_targets.len(), expected_targets.len());

    // After fixes, we should achieve high accuracy
    assert!(
        accuracy >= 80.0,
        "Expected accuracy >= 80%, got {:.0}%",
        accuracy
    );
}
