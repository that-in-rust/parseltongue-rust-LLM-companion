// Test for Bug 2 Step 2: Constructor Call Detection in C#
// REQ-CSHARP-001.0: Detect constructor calls (new TypeName())

use parseltongue_core::query_extractor::QueryBasedExtractor;
use std::path::Path;

/// REQ-CSHARP-001.0: Simple constructor call detection
#[test]
fn test_req_csharp_001_simple_constructor_call() {
    // Arrange: C# code with constructor
    let code = r#"
public class Caller {
    public void Create() {
        var obj = new DataModel();
    }
}
"#;

    let file_path = Path::new("Caller.cs");

    // Act: Extract dependencies
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, file_path, parseltongue_core::entities::Language::CSharp)
        .expect("Failed to parse C# code");

    // Assert: Constructor call detected
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
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

/// REQ-CSHARP-001.0: Generic constructor call detection
#[test]
fn test_req_csharp_001_generic_constructor_call() {
    // Arrange: Generic constructor
    let code = r#"
public class Setup {
    public void Initialize() {
        var list = new List<string>();
        var dict = new Dictionary<int, string>();
    }
}
"#;

    let file_path = Path::new("Setup.cs");

    // Act
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, file_path, parseltongue_core::entities::Language::CSharp)
        .expect("Failed to parse C# code");

    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Assert: Both constructors detected
    let has_list = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("List"));
    let has_dict = edges
        .iter()
        .any(|e| e.to_key.as_str().contains("Dictionary"));

    assert!(has_list, "Expected edge for List constructor");
    assert!(has_dict, "Expected edge for Dictionary constructor");
}

/// REQ-CSHARP-001.0: Constructor with object initializer
#[test]
fn test_req_csharp_001_constructor_with_initializer() {
    // Arrange: Object initializer syntax
    let code = r#"
public class PersonFactory {
    public void Create() {
        var person = new Person { Name = "Alice", Age = 30 };
    }
}
"#;

    let file_path = Path::new("PersonFactory.cs");

    // Act
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, file_path, parseltongue_core::entities::Language::CSharp)
        .expect("Failed to parse C# code");

    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  Edge: {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Assert: Constructor detected (initializer doesn't create extra edges)
    let person_edges: Vec<_> = edges
        .iter()
        .filter(|e| e.to_key.as_str().contains("Person"))
        .collect();

    assert!(
        !person_edges.is_empty(),
        "Expected edge for Person constructor"
    );
}

/// Test the ConstructorCall.cs fixture from TDD spec
#[test]
fn test_constructor_call_fixture() {
    let code = r#"
using System.Collections.Generic;

public class DataManager {
    public void Create() {
        var list = new List<string>();
        var model = new DataModel();
        list.Add("item");
    }
}

public class DataModel {
    public string Name { get; set; }
}
"#;

    let file_path = Path::new("ConstructorCall.cs");

    // Act
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, file_path, parseltongue_core::entities::Language::CSharp)
        .expect("Failed to parse C# code");

    println!("\n=== ConstructorCall.cs Fixture ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Expected edges: List constructor, DataModel constructor, Add method
    // Currently broken: Only Add method detected (1 edge)
    // After fix: Should detect 3 edges

    let constructor_edges = edges
        .iter()
        .filter(|e| {
            e.to_key.as_str().contains("List") || e.to_key.as_str().contains("DataModel")
        })
        .count();

    println!("Constructor edges: {}", constructor_edges);

    // This test will FAIL initially (RED state)
    // After adding constructor pattern to c_sharp.scm, it should PASS (GREEN state)
    assert!(
        constructor_edges >= 2,
        "Expected at least 2 constructor edges (List, DataModel), found {}",
        constructor_edges
    );
}
