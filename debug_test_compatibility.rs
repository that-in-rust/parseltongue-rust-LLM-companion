//! Debug script to understand Test 3.1 database compatibility issue

use parseltongue_core::storage::CozoDbStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create in-memory database
    let storage = CozoDbStorage::new("mem").await?;
    storage.create_schema().await?;
    storage.create_dependency_edges_schema().await?;

    // Insert test entities using same method as test
    storage.execute_query(r#"
        ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
          lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
          last_modified, entity_type, entity_class] <- [
            ["rust:fn:main:src_main_rs:1-10", "pub fn main() { process(); }", null, "{}", "{}", null, true, true, null, "src/main.rs", "rust", "2024-01-01T00:00:00Z", "function", "CODE"],
            ["rust:fn:process:src_process_rs:1-20", "pub fn process() { transform(); }", null, "{}", "{}", null, true, true, null, "src/process.rs", "rust", "2024-01-01T00:00:00Z", "function", "CODE"],
            ["rust:fn:transform:src_transform_rs:1-15", "pub fn transform() { /* logic */ }", null, "{}", "{}", null, true, true, null, "src/transform.rs", "rust", "2024-01-01T00:00:00Z", "function", "CODE"]
        ]
        :put CodeGraph {
            ISGL1_key => Current_Code, Future_Code, interface_signature, TDD_Classification,
            lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
            last_modified, entity_type, entity_class
        }
    "#).await?;

    // Insert dependency edges using same method as test
    storage.execute_query(r#"
        ?[from_key, to_key, edge_type, source_location] <- [
            ["rust:fn:main:src_main_rs:1-10", "rust:fn:process:src_process_rs:1-20", "Calls", "src/main.rs:5"],
            ["rust:fn:process:src_process_rs:1-20", "rust:fn:transform:src_transform_rs:1-15", "Calls", "src/process.rs:10"]
        ]

        :put DependencyEdges {
            from_key, to_key, edge_type => source_location
        }
    "#).await?;

    println!("=== Debug: Testing database compatibility ===");

    // Test 1: Raw query approach (what test does)
    println!("Testing raw query approach:");
    let raw_query = r#"
        ?[from_key, to_key, edge_type, source_location] := *DependencyEdges{from_key, to_key, edge_type, source_location},
            to_key == "rust:fn:process:src_process_rs:1-20"
    "#;

    let raw_result = storage.raw_query(raw_query).await?;
    println!("Raw query result: {} rows", raw_result.rows.len());
    for (i, row) in raw_result.rows.iter().enumerate() {
        println!("  Row {}: {:?}", i, row);
    }

    // Test 2: Existing pt01 method
    println!("\nTesting existing get_reverse_dependencies method:");
    let entity_key = "rust:fn:process:src_process_rs:1-20";
    let reverse_deps = storage.get_reverse_dependencies(entity_key).await?;
    println!("get_reverse_dependencies result: {} items", reverse_deps.len());
    for (i, dep) in reverse_deps.iter().enumerate() {
        println!("  Dep {}: {}", i, dep);
    }

    // Test 3: Verify the specific entity exists
    println!("\nVerifying target entity exists:");
    let entity_query = r#"
        ?[ISGL1_key, file_path] := *CodeGraph{ISGL1_key, file_path, entity_type, entity_class},
            ISGL1_key == "rust:fn:process:src_process_rs:1-20"
    "#;

    let entity_result = storage.raw_query(entity_query).await?;
    println!("Entity exists: {} rows", entity_result.rows.len());
    for (i, row) in entity_result.rows.iter().enumerate() {
        println!("  Entity Row {}: {:?}", i, row);
    }

    // Test 4: Count all edges
    println!("\nCounting all dependency edges:");
    let count_query = "?[count] := *DependencyEdges{from_key, to_key, edge_type, source_location}";
    let count_result = storage.raw_query(count_query).await?;
    println!("Total edges: {}", count_result.rows.len());
    if let Some(row) = count_result.rows.first() {
        if row.len() > 0 {
            println!("First count row: {:?}", row);
        }
    }

    Ok(())
}