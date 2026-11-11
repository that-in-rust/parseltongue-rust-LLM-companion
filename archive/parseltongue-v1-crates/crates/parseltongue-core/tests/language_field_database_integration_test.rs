//! Language Field Database Integration Test
//!
//! ## Test Contract
//!
//! ### Preconditions
//! - ISGL1 key contains language identifier (e.g., "javascript:fn:greet")
//! - CodeEntity has `extract_language_from_key_validated()` method
//! - Database storage uses this method to set `language` field
//!
//! ### Postconditions
//! - WHEN JavaScript file is ingested THEN database `language` field SHALL be "javascript"
//! - WHEN Python file is ingested THEN database `language` field SHALL be "python"
//! - WHEN TypeScript file is ingested THEN database `language` field SHALL be "typescript"
//! - NO hardcoded fallbacks to "rust" for non-Rust languages
//!
//! ### Error Conditions
//! - If language field = "rust" for JavaScript code → FAIL (indicates hardcoded fallback)
//! - If language field != ISGL1 key prefix → FAIL (indicates extraction bug)

use parseltongue_core::*;
use std::path::PathBuf;

/// Integration Test: Verify JavaScript entities store language="javascript" in database
///
/// **Acceptance Criteria**:
/// WHEN ingesting JavaScript code
/// THEN database SHALL extract language from ISGL1 key
/// AND language field SHALL equal "javascript" (not "rust")
#[tokio::test]
async fn test_javascript_language_field_stored_correctly() {
    // Setup: Create in-memory database
    let db = CozoDbStorage::new("mem").await.unwrap();
    db.create_schema().await.unwrap();

    // Create JavaScript entity with ISGL1 key
    let js_entity = create_javascript_test_entity();

    // Verify ISGL1 key is correct before storage
    assert!(
        js_entity.isgl1_key.starts_with("javascript:"),
        "Test setup error: ISGL1 key should start with 'javascript:', got: {}",
        js_entity.isgl1_key
    );

    // Store entity
    db.insert_entity(&js_entity).await.unwrap();

    // Retrieve entity from database
    let stored_entity = db.get_entity(&js_entity.isgl1_key).await.unwrap();

    // Verify language field is extracted from ISGL1 key
    let extracted_language = stored_entity.extract_language_from_key_validated();

    assert_eq!(
        extracted_language, "javascript",
        "FAILURE: Language field should be 'javascript' (from ISGL1 key), got: '{}'",
        extracted_language
    );

    // Verify no hardcoded fallback to "rust"
    assert_ne!(
        extracted_language, "rust",
        "FAILURE: Language field incorrectly defaulted to 'rust' for JavaScript code"
    );
}

/// Integration Test: Verify Python entities store language="python"
#[tokio::test]
async fn test_python_language_field_stored_correctly() {
    let db = CozoDbStorage::new("mem").await.unwrap();
    db.create_schema().await.unwrap();

    let python_entity = create_python_test_entity();

    assert!(
        python_entity.isgl1_key.starts_with("python:"),
        "Test setup error: ISGL1 key should start with 'python:', got: {}",
        python_entity.isgl1_key
    );

    db.insert_entity(&python_entity).await.unwrap();

    let stored_entity = db.get_entity(&python_entity.isgl1_key).await.unwrap();
    let extracted_language = stored_entity.extract_language_from_key_validated();

    assert_eq!(
        extracted_language, "python",
        "Language field should be 'python', got: '{}'",
        extracted_language
    );
}

/// Integration Test: Verify TypeScript entities store language="typescript"
#[tokio::test]
async fn test_typescript_language_field_stored_correctly() {
    let db = CozoDbStorage::new("mem").await.unwrap();
    db.create_schema().await.unwrap();

    let ts_entity = create_typescript_test_entity();

    assert!(
        ts_entity.isgl1_key.starts_with("typescript:"),
        "Test setup error: ISGL1 key should start with 'typescript:', got: {}",
        ts_entity.isgl1_key
    );

    db.insert_entity(&ts_entity).await.unwrap();

    let stored_entity = db.get_entity(&ts_entity.isgl1_key).await.unwrap();
    let extracted_language = stored_entity.extract_language_from_key_validated();

    assert_eq!(
        extracted_language, "typescript",
        "Language field should be 'typescript', got: '{}'",
        extracted_language
    );
}

/// Integration Test: Verify mixed languages in same database
#[tokio::test]
async fn test_mixed_languages_all_correct() {
    let db = CozoDbStorage::new("mem").await.unwrap();
    db.create_schema().await.unwrap();

    // Store entities from multiple languages
    let js_entity = create_javascript_test_entity();
    let python_entity = create_python_test_entity();
    let ts_entity = create_typescript_test_entity();
    let rust_entity = create_rust_test_entity();

    db.insert_entity(&js_entity).await.unwrap();
    db.insert_entity(&python_entity).await.unwrap();
    db.insert_entity(&ts_entity).await.unwrap();
    db.insert_entity(&rust_entity).await.unwrap();

    // Retrieve and verify each entity
    let stored_js = db.get_entity(&js_entity.isgl1_key).await.unwrap();
    let stored_python = db.get_entity(&python_entity.isgl1_key).await.unwrap();
    let stored_ts = db.get_entity(&ts_entity.isgl1_key).await.unwrap();
    let stored_rust = db.get_entity(&rust_entity.isgl1_key).await.unwrap();

    assert_eq!(stored_js.extract_language_from_key_validated(), "javascript");
    assert_eq!(stored_python.extract_language_from_key_validated(), "python");
    assert_eq!(stored_ts.extract_language_from_key_validated(), "typescript");
    assert_eq!(stored_rust.extract_language_from_key_validated(), "rust");
}

// ============================================================================
// Test Helper Functions
// ============================================================================

fn create_javascript_test_entity() -> CodeEntity {
    let signature = InterfaceSignature {
        entity_type: EntityType::Function,
        name: "greet".to_string(),
        visibility: Visibility::Public,
        file_path: PathBuf::from("test.js"),
        line_range: LineRange::new(1, 3).unwrap(),
        module_path: vec![],
        documentation: Some("Greet function".to_string()),
        language_specific: LanguageSpecificSignature::JavaScript(JavascriptSignature {
            parameters: vec![],
            return_type: None,
            is_async: false,
            is_arrow: false,
        }),
    };

    let mut entity = CodeEntity::new(
        "javascript:fn:greet".to_string(),
        signature,
        EntityClass::CodeImplementation,
    )
    .unwrap();

    entity.current_code = Some("function greet() { console.log('hello'); }".to_string());
    entity
}

fn create_python_test_entity() -> CodeEntity {
    let signature = InterfaceSignature {
        entity_type: EntityType::Function,
        name: "hello_world".to_string(),
        visibility: Visibility::Public,
        file_path: PathBuf::from("test.py"),
        line_range: LineRange::new(1, 2).unwrap(),
        module_path: vec![],
        documentation: Some("Hello world function".to_string()),
        language_specific: LanguageSpecificSignature::Python(PythonSignature {
            parameters: vec![],
            return_type: None,
            is_async: false,
            decorators: vec![],
        }),
    };

    let mut entity = CodeEntity::new(
        "python:fn:hello_world".to_string(),
        signature,
        EntityClass::CodeImplementation,
    )
    .unwrap();

    entity.current_code = Some("def hello_world():\n    print('hello')".to_string());
    entity
}

fn create_typescript_test_entity() -> CodeEntity {
    let signature = InterfaceSignature {
        entity_type: EntityType::Function,
        name: "calculate".to_string(),
        visibility: Visibility::Public,
        file_path: PathBuf::from("test.ts"),
        line_range: LineRange::new(1, 1).unwrap(),
        module_path: vec![],
        documentation: Some("Calculate function".to_string()),
        language_specific: LanguageSpecificSignature::TypeScript(TypeScriptSignature {
            parameters: vec![],
            return_type: Some("number".to_string()),
            generics: vec![],
            is_async: false,
        }),
    };

    let mut entity = CodeEntity::new(
        "typescript:fn:calculate".to_string(),
        signature,
        EntityClass::CodeImplementation,
    )
    .unwrap();

    entity.current_code =
        Some("function calculate(x: number): number { return x * 2; }".to_string());
    entity
}

fn create_rust_test_entity() -> CodeEntity {
    let signature = InterfaceSignature {
        entity_type: EntityType::Function,
        name: "add".to_string(),
        visibility: Visibility::Public,
        file_path: PathBuf::from("test.rs"),
        line_range: LineRange::new(1, 1).unwrap(),
        module_path: vec![],
        documentation: Some("Add function".to_string()),
        language_specific: LanguageSpecificSignature::Rust(RustSignature {
            generics: vec![],
            lifetimes: vec![],
            where_clauses: vec![],
            attributes: vec![],
            trait_impl: None,
        }),
    };

    let mut entity = CodeEntity::new(
        "rust:fn:add".to_string(),
        signature,
        EntityClass::CodeImplementation,
    )
    .unwrap();

    entity.current_code = Some("fn add(a: i32, b: i32) -> i32 { a + b }".to_string());
    entity
}
