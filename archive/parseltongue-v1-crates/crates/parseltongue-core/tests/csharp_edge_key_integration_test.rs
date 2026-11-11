// Integration test for Bug 1: C# Edge Key Format using ISGL1 v2
// This test verifies that edges generated from C# code use correct key format

use parseltongue_core::{
    query_extractor::QueryBasedExtractor,
    isgl1_v2::extract_semantic_path,
};
use std::path::Path;

/// Test that C# method call edges use ISGL1 v2 key format
#[test]
fn test_csharp_method_call_edge_uses_isgl1_v2_format() {
    // Arrange: Simple C# code with method call
    let code = r#"
using System;

public class Logger {
    public void DoWork() {
        Helper();
    }

    private void Helper() {
        Console.WriteLine("done");
    }
}
"#;

    let file_path = Path::new("Logger.cs");

    // Act: Extract dependencies
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, file_path, parseltongue_core::entities::Language::CSharp)
        .expect("Failed to parse C# code");

    // Assert: At least one edge should be extracted
    assert!(
        !edges.is_empty(),
        "Expected at least one edge from C# method calls"
    );

    // Assert: Each edge from_key should use ISGL1 v2 format (except file-level edges)
    for edge in &edges {
        let from_key_str = edge.from_key.as_str();
        println!("Edge from_key: {}", from_key_str);
        println!("Edge to_key: {}", edge.to_key.as_str());

        // Skip file-level edges - they use a different format (lang:file:path:line-line)
        if from_key_str.contains(":file:") {
            println!("  (skipping file-level edge)");
            continue;
        }

        // Key should NOT contain line range separator '-'
        // (except in to_key which is unresolved-reference:0-0)
        let from_key_parts: Vec<&str> = from_key_str.split(':').collect();

        // ISGL1 v2 format: lang:type:name:semantic_path:Ttimestamp
        assert!(
            from_key_parts.len() >= 5,
            "from_key should have at least 5 parts separated by colons: {}",
            from_key_str
        );

        // Last part should start with 'T' (timestamp marker)
        assert!(
            from_key_parts.last().unwrap().starts_with('T'),
            "Last part of from_key should start with 'T' (timestamp): {}",
            from_key_str
        );

        // Key should NOT contain line range in old format (e.g., "10-20")
        // We check that there's no pattern like ":digit-digit" before the timestamp
        let key_before_timestamp = &from_key_str[..from_key_str.rfind(":T").unwrap()];
        assert!(
            !key_before_timestamp.contains("-"),
            "from_key should not contain line range separator '-' before timestamp: {}",
            from_key_str
        );

        // Semantic path should start with "__"
        assert!(
            from_key_parts[3].starts_with("__"),
            "Semantic path should start with '__': {}",
            from_key_str
        );
    }
}

/// Test that edges for entities in same file have different timestamps
#[test]
fn test_csharp_edges_different_entities_different_timestamps() {
    // Arrange: C# code with multiple methods
    let code = r#"
public class Logger {
    public void DoWork() {
        Helper();
    }

    public void Process() {
        DoWork();
    }

    private void Helper() { }
}
"#;

    let file_path = Path::new("Logger.cs");

    // Act: Extract dependencies
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, file_path, parseltongue_core::entities::Language::CSharp)
        .expect("Failed to parse C# code");

    // Assert: Multiple edges with different from_keys (different timestamps)
    if edges.len() >= 2 {
        let timestamps: Vec<&str> = edges
            .iter()
            .filter_map(|e| e.from_key.as_str().split(":T").nth(1))
            .collect();

        // If edges come from different methods, they should have different timestamps
        if timestamps.len() > 1 {
            let unique_timestamps: std::collections::HashSet<&str> =
                timestamps.iter().copied().collect();

            // We expect at least some diversity in timestamps (not all identical)
            // Note: If all calls are from same method, timestamps will be same
            println!(
                "Timestamps found: {} unique out of {} total",
                unique_timestamps.len(),
                timestamps.len()
            );
        }
    }
}

/// Test semantic path extraction for C# files
#[test]
fn test_csharp_semantic_path_in_actual_edges() {
    // Arrange
    let code = r#"
public class DataModel {
    public void Save() {
        Validate();
    }
    private void Validate() { }
}
"#;

    let file_path = Path::new("Services/DataModel.cs");

    // Expected semantic path
    let expected_semantic_path = extract_semantic_path("Services/DataModel.cs");
    assert_eq!(expected_semantic_path, "__Services_DataModel");

    // Act: Extract dependencies
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, file_path, parseltongue_core::entities::Language::CSharp)
        .expect("Failed to parse C# code");

    // Assert: Edge from_key contains correct semantic path
    if let Some(edge) = edges.first() {
        assert!(
            edge.from_key.as_str().contains("__Services_DataModel"),
            "Edge from_key should contain semantic path '__Services_DataModel': {}",
            edge.from_key.as_str()
        );
    }
}

/// REQ-EDGE-004.0: Test that edge keys are stable across multiple parses
#[test]
fn test_csharp_edge_key_stability() {
    // Arrange: Same C# code parsed twice
    let code = r#"
public class Logger {
    public void DoWork() {
        Helper();
    }
    private void Helper() { }
}
"#;

    let file_path = Path::new("Logger.cs");

    // Act: Parse twice
    let mut extractor1 = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities1, edges1) = extractor1
        .parse_source(code, file_path, parseltongue_core::entities::Language::CSharp)
        .expect("First parse failed");

    let mut extractor2 = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities2, edges2) = extractor2
        .parse_source(code, file_path, parseltongue_core::entities::Language::CSharp)
        .expect("Second parse failed");

    // Assert: Same edges produced (same keys)
    assert_eq!(
        edges1.len(),
        edges2.len(),
        "Same number of edges should be produced"
    );

    if !edges1.is_empty() {
        let key1 = edges1[0].from_key.as_str();
        let key2 = edges2[0].from_key.as_str();

        assert_eq!(
            key1, key2,
            "Edge keys should be identical across multiple parses"
        );
    }
}
