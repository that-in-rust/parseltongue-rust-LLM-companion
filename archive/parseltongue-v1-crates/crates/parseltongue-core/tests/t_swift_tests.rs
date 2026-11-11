// Swift Entity Extraction Tests (T220-T239)
//
// Tests for Swift entity extraction using the centralized
// T-folder test fixture corpus. Each test loads source files from
// test-fixtures/T{NNN}-*/ and validates extraction results.

mod fixture_harness;

use fixture_harness::parse_fixture_extract_results;
use parseltongue_core::query_extractor::{EntityType, QueryBasedExtractor};

// ============================================================================
// T220: Swift Entity Extraction (All Types)
// ============================================================================

#[test]
fn t220_swift_entity_extraction_basic() {
    let (entities, _deps) = parse_fixture_extract_results(
        "T220-swift-entity-extraction-all",
        "basic_entities.swift",
    );

    println!("\n=== T220: Swift Basic Entities ===");
    for entity in &entities {
        println!(
            "{:?}: {} (lines {}-{})",
            entity.entity_type, entity.name, entity.line_range.0, entity.line_range.1
        );
    }

    // Verify expected entities
    assert!(
        entities.iter().any(|e| e.name == "calculateSum"),
        "Should extract function 'calculateSum'"
    );
    assert!(
        entities.iter().any(|e| e.name == "UserManager"),
        "Should extract class 'UserManager'"
    );
    assert!(
        entities.iter().any(|e| e.name == "Point"),
        "Should extract struct 'Point'"
    );
    assert!(
        entities.iter().any(|e| e.name == "Direction"),
        "Should extract enum 'Direction'"
    );
    assert!(
        entities.iter().any(|e| e.name == "Drawable"),
        "Should extract protocol 'Drawable'"
    );

    // Verify methods inside types are also extracted
    assert!(
        entities.iter().any(|e| e.name == "addUser"),
        "Should extract method 'addUser'"
    );
    assert!(
        entities.iter().any(|e| e.name == "distance"),
        "Should extract method 'distance'"
    );

    println!("\n✅ Swift basic entities test passed! Extracted {} entities", entities.len());
}

#[test]
fn t220_swift_entity_extraction_real_world() {
    let (entities, _deps) = parse_fixture_extract_results(
        "T220-swift-entity-extraction-all",
        "real_world.swift",
    );

    println!("\n=== T220: Swift Real-World Code ===");
    for entity in &entities {
        println!(
            "{:?}: {} (lines {}-{})",
            entity.entity_type, entity.name, entity.line_range.0, entity.line_range.1
        );
    }

    // Validate expected entities
    assert!(entities.iter().any(|e| e.name == "User"), "Should extract User struct");
    assert!(entities.iter().any(|e| e.name == "UserRole"), "Should extract UserRole enum");
    assert!(entities.iter().any(|e| e.name == "UserRepository"), "Should extract UserRepository protocol");
    assert!(entities.iter().any(|e| e.name == "InMemoryUserRepository"), "Should extract InMemoryUserRepository class");
    assert!(entities.iter().any(|e| e.name == "fetch"), "Should extract fetch method");
    assert!(entities.iter().any(|e| e.name == "save"), "Should extract save method");
    assert!(entities.iter().any(|e| e.name == "validateEmail"), "Should extract validateEmail function");
    assert!(entities.iter().any(|e| e.name == "generateUserKey"), "Should extract generateUserKey function");

    let entity_count = entities.len();
    assert!(
        entity_count >= 8,
        "Should extract at least 8 entities, got {}",
        entity_count
    );

    println!("\n✅ Real-world Swift code extraction successful");
    println!("   Extracted {} entities from production-like code", entity_count);
}

// ============================================================================
// T222: Swift Protocol Query Extraction
// ============================================================================

#[test]
fn t222_swift_protocol_simple() {
    let (entities, _deps) = parse_fixture_extract_results(
        "T222-swift-protocol-query-extraction",
        "protocol_simple.swift",
    );

    println!("\n=== T222: Swift Protocol Simple ===");
    for entity in &entities {
        println!("{:?}: {}", entity.entity_type, entity.name);
    }

    // Should extract protocol
    let protocol_entity = entities
        .iter()
        .find(|e| e.name == "Drawable")
        .expect("Should extract protocol 'Drawable'");

    assert_eq!(
        protocol_entity.entity_type,
        EntityType::Interface,
        "Protocol should be tagged as Interface type"
    );

    println!("✅ Protocol extraction successful");
}

#[test]
fn t222_swift_all_entity_types() {
    let (entities, _deps) = parse_fixture_extract_results(
        "T222-swift-protocol-query-extraction",
        "all_types.swift",
    );

    println!("\n=== T222: Swift All Types ===");
    for entity in &entities {
        println!("{:?}: {}", entity.entity_type, entity.name);
    }

    // Validate function extraction
    assert!(
        entities.iter().any(|e| e.name == "myFunc"),
        "Should extract global function"
    );

    // Validate class extraction
    assert!(
        entities.iter().any(|e| e.name == "MyClass"),
        "Should extract class"
    );

    // Validate struct extraction (tagged as Class)
    assert!(
        entities.iter().any(|e| e.name == "MyStruct"),
        "Should extract struct"
    );

    // Validate protocol extraction
    assert!(
        entities.iter().any(|e| e.name == "MyProtocol"),
        "Should extract protocol"
    );

    // Note: Enum is not in all_types.swift file (minimal test fixture)
    // Enum extraction is validated in other tests

    println!("✅ All Swift entity types extracted successfully");
}

// ============================================================================
// T224: Swift Fix Validation (v0.8.9 Query Compilation Fix)
// ============================================================================

#[test]
fn t224_swift_query_compiles_without_error() {
    // This was failing with: "Failed to create query" error
    // Specifically at row: 12 (struct_declaration line)
    let result = QueryBasedExtractor::new();

    assert!(
        result.is_ok(),
        "QueryBasedExtractor should initialize successfully. \
         If this fails with 'Failed to create query', check entity_queries/swift.scm \
         for invalid node types."
    );

    println!("✅ Swift query compiled successfully (no 'Failed to create query' error)");
}

#[test]
fn t224_swift_extracts_all_entity_types() {
    let (entities, _deps) = parse_fixture_extract_results(
        "T224-swift-fix-validation-queries",
        "fix_validation.swift",
    );

    println!("\n=== T224: Swift Fix Validation ===");
    for entity in &entities {
        println!(
            "{:?}: {} (line {})",
            entity.entity_type, entity.name, entity.line_range.0
        );
    }

    // Validate function extraction
    assert!(
        entities.iter().any(|e| e.name == "globalFunction"),
        "Should extract global function"
    );

    // Validate class extraction
    assert!(
        entities.iter().any(|e| e.name == "MyClass"),
        "Should extract class (via class_declaration node)"
    );

    // Validate struct extraction (NOTE: Swift uses class_declaration for structs)
    assert!(
        entities.iter().any(|e| e.name == "MyStruct"),
        "Should extract struct (via class_declaration node)"
    );

    // Validate enum extraction (NOTE: Swift uses class_declaration for enums)
    assert!(
        entities.iter().any(|e| e.name == "MyEnum"),
        "Should extract enum (via class_declaration node)"
    );

    // Validate protocol extraction
    assert!(
        entities.iter().any(|e| e.name == "MyProtocol"),
        "Should extract protocol (via protocol_declaration node)"
    );

    // Validate methods are extracted
    assert!(
        entities.iter().any(|e| e.name == "classMethod"),
        "Should extract class methods"
    );

    println!("\n✅ All Swift entity types extracted successfully");
    println!("   Total entities: {}", entities.len());
}

#[test]
fn t224_swift_protocol_uses_interface_entity_type() {
    let (entities, _deps) = parse_fixture_extract_results(
        "T224-swift-fix-validation-queries",
        "fix_validation.swift",
    );

    let protocol_entity = entities
        .iter()
        .find(|e| e.name == "MyProtocol")
        .expect("Should extract protocol");

    assert_eq!(
        protocol_entity.entity_type,
        EntityType::Interface,
        "Protocols should be tagged as Interface type (not Trait)"
    );

    println!("✅ Protocol correctly tagged as Interface entity type");
}
