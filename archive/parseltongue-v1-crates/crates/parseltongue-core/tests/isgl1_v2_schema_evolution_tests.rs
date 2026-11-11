use parseltongue_core::entities::{CodeEntity, EntityType, EntityClass, InterfaceSignature, Visibility, LanguageSpecificSignature, RustSignature, LineRange};
use std::path::PathBuf;

/// Test 1: Create entity with v2 fields
#[test]
fn test_code_entity_with_v2_fields() {
    let signature = InterfaceSignature {
        entity_type: EntityType::Function,
        name: "test_func".to_string(),
        visibility: Visibility::Public,
        file_path: PathBuf::from("src/test.rs"),
        line_range: LineRange { start: 10, end: 20 },
        module_path: vec![],
        documentation: None,
        language_specific: LanguageSpecificSignature::Rust(RustSignature {
            generics: vec![],
            lifetimes: vec![],
            where_clauses: vec![],
            attributes: vec![],
            trait_impl: None,
        }),
    };

    let entity = CodeEntity::new_with_v2_fields(
        "rust:fn:test_func:__src_test_rs:T1706284800".to_string(),
        signature,
        EntityClass::CodeImplementation,
        1706284800,
        "abc123".to_string(),
        "__src_test_rs".to_string(),
    ).unwrap();

    assert_eq!(entity.birth_timestamp, Some(1706284800));
    assert_eq!(entity.content_hash, Some("abc123".to_string()));
    assert_eq!(entity.semantic_path, Some("__src_test_rs".to_string()));
}

/// Test 2: Backward compatibility (v1 entities without v2 fields)
#[test]
fn test_code_entity_backward_compatible() {
    let signature = InterfaceSignature {
        entity_type: EntityType::Function,
        name: "old_func".to_string(),
        visibility: Visibility::Public,
        file_path: PathBuf::from("src/old.rs"),
        line_range: LineRange { start: 5, end: 15 },
        module_path: vec![],
        documentation: None,
        language_specific: LanguageSpecificSignature::Rust(RustSignature {
            generics: vec![],
            lifetimes: vec![],
            where_clauses: vec![],
            attributes: vec![],
            trait_impl: None,
        }),
    };

    let entity = CodeEntity::new(
        "rust:fn:old_func:__src_old_rs:5-15".to_string(),  // v1 key format
        signature,
        EntityClass::CodeImplementation,
    ).unwrap();

    // v2 fields should be None (backward compatible)
    assert_eq!(entity.birth_timestamp, None);
    assert_eq!(entity.content_hash, None);
    assert_eq!(entity.semantic_path, None);

    // v1 key should still work
    assert!(entity.isgl1_key.contains("5-15"));
}

/// Test 3: Serialize/deserialize with v2 fields
#[test]
fn test_code_entity_serialization_with_v2() {
    let signature = InterfaceSignature {
        entity_type: EntityType::Function,
        name: "serialize_test".to_string(),
        visibility: Visibility::Public,
        file_path: PathBuf::from("src/ser.rs"),
        line_range: LineRange { start: 1, end: 10 },
        module_path: vec![],
        documentation: None,
        language_specific: LanguageSpecificSignature::Rust(RustSignature {
            generics: vec![],
            lifetimes: vec![],
            where_clauses: vec![],
            attributes: vec![],
            trait_impl: None,
        }),
    };

    let entity = CodeEntity::new_with_v2_fields(
        "rust:fn:serialize_test:__src_ser_rs:T1706284900".to_string(),
        signature,
        EntityClass::CodeImplementation,
        1706284900,
        "def456".to_string(),
        "__src_ser_rs".to_string(),
    ).unwrap();

    // Serialize to JSON
    let json = serde_json::to_string(&entity).unwrap();

    // Deserialize back
    let deserialized: CodeEntity = serde_json::from_str(&json).unwrap();

    // Verify v2 fields preserved
    assert_eq!(deserialized.birth_timestamp, Some(1706284900));
    assert_eq!(deserialized.content_hash, Some("def456".to_string()));
    assert_eq!(deserialized.semantic_path, Some("__src_ser_rs".to_string()));
}

/// Test 4: Mixed v1/v2 entities coexist
#[test]
fn test_mixed_v1_v2_entities() {
    let signature1 = InterfaceSignature {
        entity_type: EntityType::Function,
        name: "v1_func".to_string(),
        visibility: Visibility::Public,
        file_path: PathBuf::from("src/v1.rs"),
        line_range: LineRange { start: 10, end: 20 },
        module_path: vec![],
        documentation: None,
        language_specific: LanguageSpecificSignature::Rust(RustSignature {
            generics: vec![],
            lifetimes: vec![],
            where_clauses: vec![],
            attributes: vec![],
            trait_impl: None,
        }),
    };

    let signature2 = InterfaceSignature {
        entity_type: EntityType::Function,
        name: "v2_func".to_string(),
        visibility: Visibility::Public,
        file_path: PathBuf::from("src/v2.rs"),
        line_range: LineRange { start: 30, end: 40 },
        module_path: vec![],
        documentation: None,
        language_specific: LanguageSpecificSignature::Rust(RustSignature {
            generics: vec![],
            lifetimes: vec![],
            where_clauses: vec![],
            attributes: vec![],
            trait_impl: None,
        }),
    };

    let v1_entity = CodeEntity::new(
        "rust:fn:v1_func:__src_v1_rs:10-20".to_string(),
        signature1,
        EntityClass::CodeImplementation,
    ).unwrap();

    let v2_entity = CodeEntity::new_with_v2_fields(
        "rust:fn:v2_func:__src_v2_rs:T1706285000".to_string(),
        signature2,
        EntityClass::CodeImplementation,
        1706285000,
        "mixed123".to_string(),
        "__src_v2_rs".to_string(),
    ).unwrap();

    // Both should be valid
    assert!(v1_entity.birth_timestamp.is_none());
    assert!(v2_entity.birth_timestamp.is_some());

    // Can coexist in same collection
    let entities = vec![v1_entity, v2_entity];
    assert_eq!(entities.len(), 2);
}
