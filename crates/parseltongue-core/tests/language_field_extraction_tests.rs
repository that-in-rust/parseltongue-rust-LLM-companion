//! Unit tests for language field extraction from entity keys (v1.4.5 Bug Fix)
//!
//! **Bug Context**: v1.4.3 had 100% language field corruption - ALL 233 entities
//! stored with `language: "rust"` regardless of actual language.
//!
//! **Root Cause**: Language field was hardcoded during ingestion instead of
//! being extracted from the correctly-generated entity key prefix.
//!
//! **Solution**: Add method to extract language from key prefix (first component
//! before ':' delimiter in ISGL1 key format).
//!
//! **TDD Approach**: RED -> GREEN -> REFACTOR
//! 1. RED: Write failing tests that expect language extraction method
//! 2. GREEN: Implement minimal method to extract language from key
//! 3. REFACTOR: Ensure idiomatic Rust and edge case handling
//!
//! **Key Format**: `language:type:name:file:lines`
//! - Example: `javascript:fn:greetUser:__tests_e2e_workspace_src_test_v141_js:4-6`
//! - Language prefix: `javascript` (first component)
//!
//! **Success Criteria**:
//! - Extract "rust" from "rust:fn:main:..."
//! - Extract "javascript" from "javascript:fn:greetUser:..."
//! - Extract "python" from "python:class:Parser:..."
//! - Handle edge cases (empty key, malformed key, unknown language)

use parseltongue_core::entities::CodeEntity;

/// RED Test: Extract language from Rust entity key
///
/// **Expected Behavior**: Method should parse key prefix and return "rust"
/// **Current State**: Method doesn't exist - TEST WILL FAIL
#[test]
fn test_extract_language_from_key_rust_function() {
    // Arrange: Create entity with Rust function key (realistic from live server data)
    let entity = create_test_entity_with_key(
        "rust:fn:main:__crates_parseltongue_src_main_rs:1-10"
    );

    // Act: Extract language from key prefix
    let extracted_language = entity.extract_language_from_key_validated();

    // Assert: Should extract "rust" from key prefix
    assert_eq!(
        extracted_language,
        "rust",
        "Failed to extract 'rust' from key prefix 'rust:fn:main:...'"
    );
}

/// RED Test: Extract language from JavaScript entity key
///
/// **Critical Test**: This validates the fix for Bug #3a (Language Field Corruption)
/// Live server showed JavaScript entities with `language: "rust"` field but
/// `key: "javascript:fn:greetUser:..."` - proving key generation works but
/// field assignment is broken.
///
/// **Expected Behavior**: Extract "javascript" from key prefix
/// **Current State**: Method doesn't exist - TEST WILL FAIL
#[test]
fn test_extract_language_from_key_javascript_function() {
    // Arrange: Use ACTUAL corrupted entity from live server (localhost:7777)
    // Key: "javascript:fn:greetUser:__tests_e2e_workspace_src_test_v141_js:4-6"
    // Bug: language field was "rust" instead of "javascript"
    let entity = create_test_entity_with_key(
        "javascript:fn:greetUser:__tests_e2e_workspace_src_test_v141_js:4-6"
    );

    // Act: Extract language from key prefix
    let extracted_language = entity.extract_language_from_key_validated();

    // Assert: Should extract "javascript" NOT "rust"
    assert_eq!(
        extracted_language,
        "javascript",
        "Failed to extract 'javascript' from key prefix - this is the Bug #3a scenario"
    );
}

/// RED Test: Extract language from Python entity key
///
/// **Expected Behavior**: Extract "python" from key prefix
/// **Current State**: Method doesn't exist - TEST WILL FAIL
#[test]
fn test_extract_language_from_key_python_class() {
    // Arrange: Python class entity (hypothetical example)
    let entity = create_test_entity_with_key(
        "python:class:Parser:__src_parser_py:10-50"
    );

    // Act: Extract language from key prefix
    let extracted_language = entity.extract_language_from_key_validated();

    // Assert: Should extract "python" from key prefix
    assert_eq!(
        extracted_language,
        "python",
        "Failed to extract 'python' from key prefix 'python:class:Parser:...'"
    );
}

/// RED Test: Extract language from Go entity key
///
/// **Expected Behavior**: Extract "go" from key prefix
/// **Current State**: Method doesn't exist - TEST WILL FAIL
#[test]
fn test_extract_language_from_key_go_function() {
    // Arrange: Go function entity
    let entity = create_test_entity_with_key(
        "go:fn:handleRequest:__internal_server_go:5-15"
    );

    // Act: Extract language from key prefix
    let extracted_language = entity.extract_language_from_key_validated();

    // Assert: Should extract "go" from key prefix
    assert_eq!(
        extracted_language,
        "go",
        "Failed to extract 'go' from key prefix 'go:fn:handleRequest:...'"
    );
}

/// RED Test: Extract language from TypeScript entity key
///
/// **Expected Behavior**: Extract "typescript" from key prefix
/// **Current State**: Method doesn't exist - TEST WILL FAIL
#[test]
fn test_extract_language_from_key_typescript_interface() {
    // Arrange: TypeScript interface entity
    let entity = create_test_entity_with_key(
        "typescript:interface:UserProfile:__src_types_ts:1-20"
    );

    // Act: Extract language from key prefix
    let extracted_language = entity.extract_language_from_key_validated();

    // Assert: Should extract "typescript" from key prefix
    assert_eq!(
        extracted_language,
        "typescript",
        "Failed to extract 'typescript' from key prefix 'typescript:interface:...'"
    );
}

/// RED Test: Handle edge case - empty key
///
/// **Expected Behavior**: Return "unknown" for empty/invalid keys
/// **Current State**: Method doesn't exist - TEST WILL FAIL
#[test]
fn test_extract_language_from_empty_key() {
    // Arrange: Entity with empty ISGL1 key (should never happen, but defensive)
    // Note: CodeEntity::new validates non-empty keys, so this tests the extraction method directly
    let entity = create_test_entity_with_key(""); // Will use fallback in real code

    // Act: Extract language from empty key
    let extracted_language = entity.extract_language_from_key_validated();

    // Assert: Should return "unknown" as safe fallback
    assert_eq!(
        extracted_language,
        "unknown",
        "Empty key should return 'unknown' as safe fallback"
    );
}

/// RED Test: Handle edge case - malformed key (no delimiter)
///
/// **Expected Behavior**: Return entire key as language (or "unknown" if validation fails)
/// **Current State**: Method doesn't exist - TEST WILL FAIL
#[test]
fn test_extract_language_from_malformed_key() {
    // Arrange: Malformed key without colon delimiter
    let entity = create_test_entity_with_key("malformed_key_no_colons");

    // Act: Extract language from malformed key
    let extracted_language = entity.extract_language_from_key_validated();

    // Assert: Should return the whole string OR "unknown" (implementation choice)
    // For now, expect the whole string to be treated as language prefix
    assert_eq!(
        extracted_language,
        "malformed_key_no_colons",
        "Malformed key without delimiters should return entire key as language"
    );
}

/// RED Test: Handle edge case - key with only one component
///
/// **Expected Behavior**: Return that component as language
/// **Current State**: Method doesn't exist - TEST WILL FAIL
#[test]
fn test_extract_language_from_single_component_key() {
    // Arrange: Key with only language component
    let entity = create_test_entity_with_key("rust");

    // Act: Extract language from single-component key
    let extracted_language = entity.extract_language_from_key_validated();

    // Assert: Should return "rust"
    assert_eq!(
        extracted_language,
        "rust",
        "Single-component key should return that component as language"
    );
}

// ============================================================================
// Test Helpers
// ============================================================================

use parseltongue_core::entities::{
    InterfaceSignature, EntityType, Visibility, LineRange,
    LanguageSpecificSignature, RustSignature, EntityClass
};
use std::path::PathBuf;

/// Helper: Create minimal test entity with custom ISGL1 key
///
/// **Design**: Minimal entity for unit testing language extraction only.
/// Creates a valid CodeEntity but we only care about the ISGL1 key for these tests.
fn create_test_entity_with_key(isgl1_key: &str) -> CodeEntity {
    // Create minimal valid entity - we only care about the key for language extraction tests
    CodeEntity::new(
        isgl1_key.to_string(),
        InterfaceSignature {
            entity_type: EntityType::Function,
            name: "test".to_string(),
            visibility: Visibility::Public,
            file_path: PathBuf::from("test.rs"),
            line_range: LineRange::new(1, 1).unwrap(),
            module_path: vec![],
            documentation: None,
            language_specific: LanguageSpecificSignature::Rust(RustSignature {
                generics: vec![],
                lifetimes: vec![],
                where_clauses: vec![],
                attributes: vec![],
                trait_impl: None,
            }),
        },
        EntityClass::CodeImplementation,
    ).unwrap()
}

// ============================================================================
// Future Tests (After GREEN Phase)
// ============================================================================

// TODO: After GREEN phase, add integration test that verifies:
// 1. Entity stored in database has language field matching key prefix
// 2. Query localhost:7777/code-entities-list-all returns matching language fields
// 3. Fresh ingestion produces 0 language mismatches

// TODO: Add regression test that prevents hardcoding language field:
// - Parse multiple files (JS, Rust, Python)
// - Verify each entity's language field matches its key prefix
// - Ensure NO entities have mismatched language vs key prefix
