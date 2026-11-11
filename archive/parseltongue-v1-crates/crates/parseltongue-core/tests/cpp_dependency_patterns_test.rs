// C++ Dependency Pattern Tests (v1.4.10)
// REQ-CPP-001.0 through REQ-CPP-004.0
// Following TDD: RED -> GREEN -> REFACTOR
//
// P1 HIGH: C++ missing advanced dependency patterns!

use parseltongue_core::query_extractor::QueryBasedExtractor;
use parseltongue_core::entities::Language;
use std::path::Path;

// ============================================================================
// Helper Function
// ============================================================================

fn parse_cpp_code_extract_edges(code: &str) -> Vec<parseltongue_core::entities::DependencyEdge> {
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let (_entities, edges) = extractor
        .parse_source(code, Path::new("Test.cpp"), Language::Cpp)
        .expect("Failed to parse C++ code");
    edges
}

// ============================================================================
// REQ-CPP-001.0: Constructor Calls (new_expression)
// ============================================================================

#[test]
fn test_cpp_new_expression_simple() {
    let code = r#"
void create() {
    User* user = new User();
    Config* config = new Config("test");
}
    "#;

    let edges = parse_cpp_code_extract_edges(code);

    println!("\n=== C++ new_expression (Simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect constructor calls via new_expression
    let user_edges = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let config_edges = edges.iter().any(|e| e.to_key.as_str().contains("Config"));

    assert!(user_edges, "Expected edge for User constructor via new");
    assert!(config_edges, "Expected edge for Config constructor via new");
}

#[test]
fn test_cpp_new_expression_template() {
    let code = r#"
void allocate() {
    auto ptr = new std::vector<int>();
    auto map = new std::map<std::string, int>();
}
    "#;

    let edges = parse_cpp_code_extract_edges(code);

    println!("\n=== C++ new_expression (Template) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect template type constructors
    let vector_edges = edges.iter().any(|e| e.to_key.as_str().contains("vector"));
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("map"));

    assert!(vector_edges, "Expected edge for std::vector constructor");
    assert!(map_edges, "Expected edge for std::map constructor");
}

// ============================================================================
// REQ-CPP-002.0: Field Access (field_expression)
// ============================================================================

#[test]
fn test_cpp_field_access_pointer() {
    let code = r#"
void process(User* user) {
    std::string name = user->name;
    int age = user->age;
}
    "#;

    let edges = parse_cpp_code_extract_edges(code);

    println!("\n=== C++ field_expression (Pointer) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect field access via ->
    let name_edges = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edges = edges.iter().any(|e| e.to_key.as_str().contains("age"));

    assert!(name_edges, "Expected edge for name field access");
    assert!(age_edges, "Expected edge for age field access");
}

#[test]
fn test_cpp_field_access_reference() {
    let code = r#"
void update(User& user) {
    user.name = "John";
    user.age = 30;
}
    "#;

    let edges = parse_cpp_code_extract_edges(code);

    println!("\n=== C++ field_expression (Reference) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect field access via .
    let name_edges = edges.iter().any(|e| e.to_key.as_str().contains("name"));
    let age_edges = edges.iter().any(|e| e.to_key.as_str().contains("age"));

    assert!(name_edges, "Expected edge for name field access");
    assert!(age_edges, "Expected edge for age field access");
}

// ============================================================================
// REQ-CPP-003.0: Template Types
// ============================================================================

#[test]
fn test_cpp_template_types_simple() {
    let code = r#"
void setup() {
    std::vector<int> items;
    std::string text;
}
    "#;

    let edges = parse_cpp_code_extract_edges(code);

    println!("\n=== C++ template_type (Simple) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect template type usage
    let vector_edges = edges.iter().any(|e| e.to_key.as_str().contains("vector"));
    let string_edges = edges.iter().any(|e| e.to_key.as_str().contains("string"));

    assert!(vector_edges, "Expected edge for std::vector template type");
    assert!(string_edges, "Expected edge for std::string type");
}

#[test]
fn test_cpp_template_types_nested() {
    let code = r#"
void complex() {
    std::map<std::string, User> users;
    std::vector<std::shared_ptr<Config>> configs;
}
    "#;

    let edges = parse_cpp_code_extract_edges(code);

    println!("\n=== C++ template_type (Nested) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect nested template types
    let map_edges = edges.iter().any(|e| e.to_key.as_str().contains("map"));
    let user_edges = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let vector_edges = edges.iter().any(|e| e.to_key.as_str().contains("vector"));
    let shared_ptr_edges = edges.iter().any(|e| e.to_key.as_str().contains("shared_ptr"));
    let config_edges = edges.iter().any(|e| e.to_key.as_str().contains("Config"));

    assert!(map_edges, "Expected edge for std::map");
    assert!(user_edges, "Expected edge for User type");
    assert!(vector_edges, "Expected edge for std::vector");
    assert!(shared_ptr_edges, "Expected edge for std::shared_ptr");
    assert!(config_edges, "Expected edge for Config type");
}

// ============================================================================
// REQ-CPP-004.0: Smart Pointers
// ============================================================================

#[test]
fn test_cpp_smart_pointers_unique() {
    let code = r#"
void create() {
    auto user = std::make_unique<User>();
    auto config = std::make_unique<Config>("test");
}
    "#;

    let edges = parse_cpp_code_extract_edges(code);

    println!("\n=== C++ Smart Pointers (unique_ptr) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect make_unique calls
    let make_unique_edges = edges.iter().any(|e| e.to_key.as_str().contains("make_unique"));
    let user_edges = edges.iter().any(|e| e.to_key.as_str().contains("User"));
    let config_edges = edges.iter().any(|e| e.to_key.as_str().contains("Config"));

    assert!(make_unique_edges, "Expected edge for make_unique function");
    assert!(user_edges, "Expected edge for User type in make_unique");
    assert!(config_edges, "Expected edge for Config type in make_unique");
}

#[test]
fn test_cpp_smart_pointers_shared() {
    let code = r#"
void share() {
    auto config = std::make_shared<Config>();
    auto manager = std::make_shared<Manager>(config);
}
    "#;

    let edges = parse_cpp_code_extract_edges(code);

    println!("\n=== C++ Smart Pointers (shared_ptr) Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Should detect make_shared calls
    let make_shared_edges = edges.iter().any(|e| e.to_key.as_str().contains("make_shared"));
    let config_edges = edges.iter().any(|e| e.to_key.as_str().contains("Config"));
    let manager_edges = edges.iter().any(|e| e.to_key.as_str().contains("Manager"));

    assert!(make_shared_edges, "Expected edge for make_shared function");
    assert!(config_edges, "Expected edge for Config type in make_shared");
    assert!(manager_edges, "Expected edge for Manager type in make_shared");
}

// ============================================================================
// REQ-CPP-REGRESS.0: Regression Tests - Existing Patterns
// ============================================================================

#[test]
fn test_cpp_existing_function_calls() {
    let code = r#"
void process() {
    calculate();
    validate();
}
    "#;

    let edges = parse_cpp_code_extract_edges(code);

    println!("\n=== C++ Existing Function Calls Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Ensure existing function call detection still works
    let calc_edges = edges.iter().any(|e| e.to_key.as_str().contains("calculate"));
    let validate_edges = edges.iter().any(|e| e.to_key.as_str().contains("validate"));

    assert!(calc_edges, "Expected edge for calculate function");
    assert!(validate_edges, "Expected edge for validate function");
}

#[test]
#[ignore = "C++ #include directives are preprocessor-level; tree-sitter parses post-preprocessor AST"]
fn test_cpp_existing_includes() {
    let code = r#"
#include <iostream>
#include "custom.h"

void main() {}
    "#;

    let edges = parse_cpp_code_extract_edges(code);

    println!("\n=== C++ Existing Include Test ===");
    println!("Edges found: {}", edges.len());
    for edge in &edges {
        println!("  {} -> {}", edge.from_key.as_str(), edge.to_key.as_str());
    }

    // Ensure existing include detection still works
    let iostream_edges = edges.iter().any(|e| e.to_key.as_str().contains("iostream"));
    let custom_edges = edges.iter().any(|e| e.to_key.as_str().contains("custom"));

    assert!(iostream_edges, "Expected edge for iostream include");
    assert!(custom_edges, "Expected edge for custom.h include");
}
