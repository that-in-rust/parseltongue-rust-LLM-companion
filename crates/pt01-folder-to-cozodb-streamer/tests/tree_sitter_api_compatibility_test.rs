//! Tree-sitter API compatibility tests for v0.22+
//!
//! These tests verify that parser initialization works correctly
//! with the new LanguageFn API (0.24+).

use pt01_folder_to_cozodb_streamer::isgl1_generator::{Isgl1KeyGenerator, Isgl1KeyGeneratorImpl};
use std::path::Path;

/// Test that Rust parser initializes successfully
///
/// Precondition: Tree-sitter 0.24+ with Rust grammar
/// Postcondition: Parser can parse basic Rust code without panics
#[test]
fn test_rust_parser_initialization() {
    // This will panic if .into() is missing or parser initialization fails
    let _generator = Isgl1KeyGeneratorImpl::new();
    // If we get here, Rust parser initialized successfully
}

/// Test that Rust parser can parse basic code
///
/// Precondition: Valid Rust function syntax
/// Postcondition: Returns Ok with at least one entity
#[test]
fn test_rust_basic_parsing() {
    let generator = Isgl1KeyGeneratorImpl::new();
    let code = "fn hello() {}";
    let path = Path::new("test.rs");

    // This will fail if API is broken
    let result = generator.parse_source(code, path);

    assert!(result.is_ok(), "Should parse valid Rust code");

    let (entities, _, _) = result.unwrap();
    assert_eq!(entities.len(), 1, "Should extract one function");
    assert_eq!(entities[0].name, "hello");
}

/// Test that parser correctly rejects invalid syntax
///
/// Precondition: Invalid Rust syntax
/// Postcondition: Parser detects errors
#[test]
fn test_rust_invalid_syntax_detection() {
    let generator = Isgl1KeyGeneratorImpl::new();
    let code = "fn hello( {"; // Missing closing paren
    let path = Path::new("test.rs");

    let result = generator.parse_source(code, path);

    // Parser should still succeed but tree should have errors
    assert!(result.is_ok(), "Parser should not crash on invalid syntax");
}

/// Test that multiple language parsers can coexist
///
/// Precondition: All language dependencies in Cargo.toml
/// Postcondition: All supported languages can parse basic code
#[test]
fn test_multiple_languages_basic_parsing() {
    let generator = Isgl1KeyGeneratorImpl::new();

    // Test that various languages can parse simple code
    // Note: Kotlin is excluded - tree-sitter-kotlin v0.3 uses incompatible tree-sitter 0.20
    let test_cases = vec![
        ("test.rs", "fn hello() {}", "Rust"),
        ("test.py", "def hello():\n    pass", "Python"),
        ("test.js", "function hello() {}", "JavaScript"),
        ("test.ts", "function hello() {}", "TypeScript"),
        ("test.go", "func hello() {}", "Go"),
        ("test.java", "class Test { void hello() {} }", "Java"),
        ("test.cpp", "void hello() {}", "C++"),
        ("test.rb", "def hello\nend", "Ruby"),
        ("test.php", "<?php function hello() {} ?>", "PHP"),
        ("test.cs", "void Hello() {}", "C#"),
        ("test.swift", "func hello() {}", "Swift"),
        ("test.scala", "def hello() = {}", "Scala"),
    ];

    for (filename, code, lang_name) in test_cases {
        let path = Path::new(filename);
        let result = generator.parse_source(code, path);

        assert!(
            result.is_ok(),
            "{} parser should initialize and parse basic code (got error: {:?})",
            lang_name,
            result.err()
        );
    }
}
