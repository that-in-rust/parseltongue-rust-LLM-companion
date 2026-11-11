//! Query-Based Entity Extraction Tests (TDD RED Phase)
//!
//! Preconditions:
//! - Query files exist in entity_queries/ directory
//! - Tree-sitter parsers initialized for all languages
//!
//! Postconditions:
//! - Extracts same entities as imperative approach
//! - <20ms per 1K LOC (performance contract)
//! - Zero panics on malformed code

use parseltongue_core::query_extractor::QueryBasedExtractor;
use parseltongue_core::entities::Language;
use std::path::Path;

/// RED TEST 1: Query-based Rust extraction
#[test]
fn test_query_rust_functions_and_structs() {
    let mut extractor = QueryBasedExtractor::new().unwrap();
    let code = r#"
        fn calculate_sum(a: i32, b: i32) -> i32 { a + b }
        struct User { name: String, age: u32 }
        enum Status { Active, Inactive }
    "#;

    let (entities, _deps) = extractor.parse_source(
        code,
        Path::new("test.rs"),
        Language::Rust
    ).unwrap();

    assert_eq!(entities.len(), 3, "Should extract function + struct + enum");

    // Verify entity names
    let names: Vec<&str> = entities.iter()
        .map(|e| e.name.as_str())
        .collect();
    assert!(names.contains(&"calculate_sum"));
    assert!(names.contains(&"User"));
    assert!(names.contains(&"Status"));
}

/// RED TEST 2: Query-based Python extraction
#[test]
fn test_query_python_classes_and_functions() {
    let mut extractor = QueryBasedExtractor::new().unwrap();
    let code = r#"
class Calculator:
    def add(self, a, b):
        return a + b

def hello_world():
    print("Hello")
    "#;

    let (entities, _deps) = extractor.parse_source(
        code,
        Path::new("test.py"),
        Language::Python
    ).unwrap();

    assert_eq!(entities.len(), 3, "Should extract class + 2 functions");
}

/// RED TEST 3: Query-based C extraction
#[test]
fn test_query_c_functions_and_structs() {
    let mut extractor = QueryBasedExtractor::new().unwrap();
    let code = r#"
int add(int a, int b) { return a + b; }

struct Node {
    int value;
    struct Node* next;
};

typedef struct {
    char* name;
    int age;
} Person;
    "#;

    let (entities, _deps) = extractor.parse_source(
        code,
        Path::new("test.c"),
        Language::C
    ).unwrap();

    assert_eq!(entities.len(), 3, "Should extract function + 2 structs");
}

/// RED TEST 5: No panic on malformed code
#[test]
fn test_malformed_code_no_panic() {
    let mut extractor = QueryBasedExtractor::new().unwrap();
    let broken_code = "fn main( { println!(\"broken\";";

    // Should not panic, may return Ok with partial entities or Err
    let result = extractor.parse_source(broken_code, Path::new("test.rs"), Language::Rust);
    // Just verify no panic - result can be Ok or Err
    let _ = result;
}

/// RED TEST 6: Query-based JavaScript extraction
#[test]
fn test_query_javascript_functions_and_classes() {
    let mut extractor = QueryBasedExtractor::new().unwrap();
    let code = r#"
function greet(name) {
    console.log("Hello " + name);
}

const add = (a, b) => a + b;

class Calculator {
    multiply(x, y) {
        return x * y;
    }
}
    "#;

    let (entities, _deps) = extractor.parse_source(
        code,
        Path::new("test.js"),
        Language::JavaScript
    ).unwrap();

    assert!(entities.len() >= 3, "Should extract function + arrow function + class (got {})", entities.len());

    // Verify entity names
    let names: Vec<&str> = entities.iter()
        .map(|e| e.name.as_str())
        .collect();
    assert!(names.contains(&"greet"), "Should find 'greet' function");
    assert!(names.contains(&"Calculator"), "Should find 'Calculator' class");
}

// Helper: Generate N lines of Rust code
fn generate_rust_code(lines: usize) -> String {
    (0..lines)
        .map(|i| format!("fn func_{}() {{ println!(\"test\"); }}", i))
        .collect::<Vec<_>>()
        .join("\n")
}
