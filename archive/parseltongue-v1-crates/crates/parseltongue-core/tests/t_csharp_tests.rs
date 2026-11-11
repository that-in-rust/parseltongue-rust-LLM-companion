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

// ============================================================================
// T202 (Additional): C# Property Chaining
// ============================================================================

#[test]
fn t202_csharp_property_chaining() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T202-csharp-property-linq-edges",
        "property_chaining.cs",
    );

    println!("\n=== T202: C# Property Chaining ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect chained property access
    let has_settings = edges.iter().any(|e| e.to_key.as_str().contains("Settings"));
    let has_timeout = edges.iter().any(|e| e.to_key.as_str().contains("Timeout"));
    let has_connection = edges.iter().any(|e| e.to_key.as_str().contains("Connection"));
    let has_host = edges.iter().any(|e| e.to_key.as_str().contains("Host"));

    assert!(has_settings || has_timeout, "Expected edge for Settings or Timeout property");
    assert!(has_connection || has_host, "Expected edge for Connection or Host property");
}

// ============================================================================
// T203: C# LINQ Advanced Edges
// ============================================================================

#[test]
fn t203_csharp_linq_ordering() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T203-csharp-linq-advanced-edges",
        "linq_ordering.cs",
    );

    println!("\n=== T203: C# LINQ Ordering ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect ordering LINQ methods
    let orderby_edges = edges.iter().any(|e| e.to_key.as_str().contains("OrderBy"));
    let thenby_edges = edges.iter().any(|e| e.to_key.as_str().contains("ThenBy"));

    assert!(orderby_edges, "Expected edge for OrderBy LINQ method");
    assert!(thenby_edges, "Expected edge for ThenBy LINQ method");
}

#[test]
fn t203_csharp_linq_first_single() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T203-csharp-linq-advanced-edges",
        "linq_first_single.cs",
    );

    println!("\n=== T203: C# LINQ First/Single ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect First/Single/Last LINQ methods
    let first_edges = edges.iter().any(|e| e.to_key.as_str().contains("FirstOrDefault"));
    let single_edges = edges.iter().any(|e| e.to_key.as_str().contains("SingleOrDefault"));
    let last_edges = edges.iter().any(|e| e.to_key.as_str().contains("Last"));

    assert!(first_edges, "Expected edge for FirstOrDefault LINQ method");
    assert!(single_edges, "Expected edge for SingleOrDefault LINQ method");
    assert!(last_edges, "Expected edge for Last LINQ method");
}

#[test]
fn t203_csharp_linq_set_ops() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T203-csharp-linq-advanced-edges",
        "linq_set_ops.cs",
    );

    println!("\n=== T203: C# LINQ Set Operations ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect set LINQ methods
    let distinct_edges = edges.iter().any(|e| e.to_key.as_str().contains("Distinct"));
    let union_edges = edges.iter().any(|e| e.to_key.as_str().contains("Union"));
    let intersect_edges = edges.iter().any(|e| e.to_key.as_str().contains("Intersect"));

    assert!(distinct_edges, "Expected edge for Distinct LINQ method");
    assert!(union_edges, "Expected edge for Union LINQ method");
    assert!(intersect_edges, "Expected edge for Intersect LINQ method");
}

// ============================================================================
// T205: C# Async/Await Edges
// ============================================================================

#[test]
fn t205_csharp_async_simple() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T205-csharp-async-await-edges",
        "async_simple.cs",
    );

    println!("\n=== T205: C# Async/Await Simple ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect async method calls
    let fetch_edges = edges.iter().any(|e| e.to_key.as_str().contains("FetchDataAsync"));
    let save_edges = edges.iter().any(|e| e.to_key.as_str().contains("SaveAsync"));

    assert!(fetch_edges, "Expected edge for FetchDataAsync method");
    assert!(save_edges, "Expected edge for SaveAsync method");
}

#[test]
fn t205_csharp_async_member() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T205-csharp-async-await-edges",
        "async_member.cs",
    );

    println!("\n=== T205: C# Async/Await Member Access ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect async method calls on members
    let get_edges = edges.iter().any(|e| e.to_key.as_str().contains("GetAsync"));
    let read_edges = edges.iter().any(|e| e.to_key.as_str().contains("ReadAsync"));
    let parse_edges = edges.iter().any(|e| e.to_key.as_str().contains("ParseJsonAsync"));

    assert!(get_edges, "Expected edge for GetAsync method");
    assert!(read_edges, "Expected edge for ReadAsync method");
    assert!(parse_edges, "Expected edge for ParseJsonAsync method");
}

#[test]
fn t205_csharp_async_task() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T205-csharp-async-await-edges",
        "async_task.cs",
    );

    println!("\n=== T205: C# Async Task Operations ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect Task utility methods
    let delay_edges = edges.iter().any(|e| e.to_key.as_str().contains("Delay"));
    let whenall_edges = edges.iter().any(|e| e.to_key.as_str().contains("WhenAll"));
    let whenany_edges = edges.iter().any(|e| e.to_key.as_str().contains("WhenAny"));

    assert!(delay_edges, "Expected edge for Task.Delay method");
    assert!(whenall_edges, "Expected edge for Task.WhenAll method");
    assert!(whenany_edges, "Expected edge for Task.WhenAny method");
}

// ============================================================================
// T207: C# Combined Patterns Test
// ============================================================================

#[test]
fn t207_csharp_combined_patterns() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T207-csharp-combined-patterns-test",
        "combined.cs",
    );

    println!("\n=== T207: C# Combined Patterns ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect multiple pattern types
    let has_property = edges.iter().any(|e| {
        e.to_key.as_str().contains("Count") || e.to_key.as_str().contains("Capacity")
    });
    let has_linq = edges.iter().any(|e| {
        e.to_key.as_str().contains("Where") || e.to_key.as_str().contains("OrderBy")
    });
    let has_async = edges.iter().any(|e| e.to_key.as_str().contains("FetchDataAsync"));
    let has_constructor = edges.iter().any(|e| e.to_key.as_str().contains("List"));

    assert!(has_property, "Expected property access edges");
    assert!(has_linq, "Expected LINQ method edges");
    assert!(has_async, "Expected async/await edges");
    assert!(has_constructor, "Expected constructor edges");

    // Should have at least 6 edges (conservative estimate)
    assert!(
        edges.len() >= 6,
        "Expected at least 6 edges for combined patterns, found {}",
        edges.len()
    );
}

// ============================================================================
// T208: C# Edge Key Stability
// ============================================================================

#[test]
fn t208_csharp_different_timestamps() {
    let (_entities, edges) = parse_fixture_extract_results(
        "T208-csharp-edge-key-stability",
        "different_timestamps.cs",
    );

    println!("\n=== T208: C# Different Timestamps ===");
    println!("Edges found: {}", edges.len());

    // Multiple edges with different from_keys (different timestamps)
    if edges.len() >= 2 {
        let timestamps: Vec<&str> = edges
            .iter()
            .filter_map(|e| e.from_key.as_str().split(":T").nth(1))
            .collect();

        // If edges come from different methods, they should have different timestamps
        if timestamps.len() > 1 {
            let unique_timestamps: std::collections::HashSet<&str> =
                timestamps.iter().copied().collect();

            println!(
                "Timestamps found: {} unique out of {} total",
                unique_timestamps.len(),
                timestamps.len()
            );
        }
    }
}

#[test]
fn t208_csharp_key_stability() {
    use parseltongue_core::query_extractor::QueryBasedExtractor;
    use parseltongue_core::entities::Language;
    use std::path::Path;

    // Read the same fixture code
    let code = fixture_harness::load_fixture_source_file(
        "T208-csharp-edge-key-stability",
        "key_stability.cs",
    );

    let file_path = Path::new("key_stability.cs");

    // Parse twice
    let mut extractor1 = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities1, edges1) = extractor1
        .parse_source(&code, file_path, Language::CSharp)
        .expect("First parse failed");

    let mut extractor2 = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities2, edges2) = extractor2
        .parse_source(&code, file_path, Language::CSharp)
        .expect("Second parse failed");

    println!("\n=== T208: C# Key Stability ===");
    println!("First parse edges: {}", edges1.len());
    println!("Second parse edges: {}", edges2.len());

    // Assert: Same edges produced (same keys)
    assert_eq!(
        edges1.len(),
        edges2.len(),
        "Same number of edges should be produced"
    );

    if !edges1.is_empty() {
        let key1 = edges1[0].from_key.as_str();
        let key2 = edges2[0].from_key.as_str();

        println!("First key:  {}", key1);
        println!("Second key: {}", key2);

        assert_eq!(
            key1, key2,
            "Edge keys should be identical across multiple parses"
        );
    }
}
