//! C Dependency Pattern Tests (v1.4.9)

use parseltongue_core::entities::Language;
use parseltongue_core::query_extractor::QueryBasedExtractor;
use std::path::Path;

fn extract_c_dependencies(code: &str) -> Vec<String> {
    let mut extractor = QueryBasedExtractor::new().expect("Failed to create extractor");
    let path = Path::new("test.c");

    match extractor.parse_source(code, path, Language::C) {
        Ok((_, edges)) => edges.iter().map(|e| {
            e.to_key.to_string().split(':').nth(2).unwrap_or("").to_string()
        }).collect(),
        Err(_) => vec![],
    }
}

#[test]
fn test_c_struct_field_arrow() {
    let code = r#"
void process(struct User* user) {
    char* name = user->name;
    int age = user->age;
}
    "#;

    let edges = extract_c_dependencies(code);
    assert!(edges.iter().any(|e| e == "name"), "Should detect name field");
    assert!(edges.iter().any(|e| e == "age"), "Should detect age field");
}

#[test]
fn test_c_struct_field_dot() {
    let code = r#"
void setup(struct Config config) {
    int timeout = config.timeout;
    int port = config.port;
}
    "#;

    let edges = extract_c_dependencies(code);
    assert!(edges.iter().any(|e| e == "timeout"), "Should detect timeout field");
    assert!(edges.iter().any(|e| e == "port"), "Should detect port field");
}

#[test]
fn test_c_existing_function_calls() {
    // Regression test - ensure existing patterns work
    let code = r#"
void main() {
    printf("hello");
    process_data(x);
}
    "#;

    let edges = extract_c_dependencies(code);
    assert!(edges.iter().any(|e| e == "printf"), "Should detect printf");
    assert!(edges.iter().any(|e| e == "process_data"), "Should detect process_data");
}
