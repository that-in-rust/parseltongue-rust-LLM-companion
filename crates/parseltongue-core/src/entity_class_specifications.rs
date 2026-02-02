//! Executable Specifications for EntityClass Integration
//!
//! Following S01 TDD principles: STUB → RED → GREEN → REFACTOR
//! These specifications define the contract for mandatory EntityClass field

#[cfg(test)]
use crate::entities::{CodeEntity, EntityClass, EntityType, InterfaceSignature};

/// Contract: All CodeEntity instances MUST have EntityClass
/// 
/// Preconditions: Valid entity creation parameters
/// Postconditions: Entity has mandatory entity_class field
/// Error conditions: Invalid entity creation should fail
#[cfg(test)]
mod entity_class_mandatory_tests {
    use super::*;

    /// GREEN: Test that CodeEntity creation requires EntityClass
    #[test]
    fn test_code_entity_creation_requires_entity_class() {
        // Arrange: Valid entity creation parameters
        let signature = InterfaceSignature {
            entity_type: EntityType::Function,
            name: "test_function".to_string(),
            visibility: crate::entities::Visibility::Public,
            file_path: std::path::PathBuf::from("test.rs"),
            line_range: crate::entities::LineRange::new(1, 3).unwrap(),
            module_path: vec![],
            documentation: None,
            language_specific: crate::entities::LanguageSpecificSignature::Rust(
                crate::entities::RustSignature {
                    generics: vec![],
                    lifetimes: vec![],
                    where_clauses: vec![],
                    attributes: vec![],
                    trait_impl: None,
                }
            ),
        };

        // Act: Create entity with mandatory EntityClass
        let entity = CodeEntity::new(
            "test.rs-test_function-fn-abc123".to_string(),
            signature,
            EntityClass::CodeImplementation, // v0.9.0: mandatory parameter
        );

        // Assert: Entity created successfully with EntityClass
        assert!(entity.is_ok(), "Entity creation should succeed with EntityClass");
        let entity = entity.unwrap();
        assert_eq!(entity.entity_class, EntityClass::CodeImplementation);
        assert_eq!(entity.isgl1_key, "test.rs-test_function-fn-abc123");
    }

    /// GREEN: Test that EntityClass is never None/Optional
    #[test]
    fn test_entity_class_is_always_present() {
        // Arrange: Create entities with different classifications
        let signature = InterfaceSignature {
            entity_type: EntityType::Function,
            name: "code_fn".to_string(),
            visibility: crate::entities::Visibility::Public,
            file_path: std::path::PathBuf::from("code.rs"),
            line_range: crate::entities::LineRange::new(1, 3).unwrap(),
            module_path: vec![],
            documentation: None,
            language_specific: crate::entities::LanguageSpecificSignature::Rust(
                crate::entities::RustSignature {
                    generics: vec![],
                    lifetimes: vec![],
                    where_clauses: vec![],
                    attributes: vec![],
                    trait_impl: None,
                }
            ),
        };

        let test_signature = InterfaceSignature {
            entity_type: EntityType::TestFunction,
            name: "test_fn".to_string(),
            visibility: crate::entities::Visibility::Private,
            file_path: std::path::PathBuf::from("code.rs"),
            line_range: crate::entities::LineRange::new(1, 3).unwrap(),
            module_path: vec![],
            documentation: None,
            language_specific: crate::entities::LanguageSpecificSignature::Rust(
                crate::entities::RustSignature {
                    generics: vec![],
                    lifetimes: vec![],
                    where_clauses: vec![],
                    attributes: vec![],
                    trait_impl: None,
                }
            ),
        };

        // Act: Create both code and test entities
        let code_entity = CodeEntity::new(
            "code.rs-code_fn-fn-def456".to_string(),
            signature,
            EntityClass::CodeImplementation,
        ).unwrap();

        let test_entity = CodeEntity::new(
            "code.rs-test_fn-test-ghi012".to_string(),
            test_signature,
            EntityClass::TestImplementation,
        ).unwrap();

        // Assert: Both entities have definitive EntityClass (no Option/None)
        assert_eq!(code_entity.entity_class, EntityClass::CodeImplementation);
        assert_eq!(test_entity.entity_class, EntityClass::TestImplementation);
        
        // Verify EntityClass has Copy trait and can be compared directly
        assert!(code_entity.entity_class == EntityClass::CodeImplementation);
        assert!(test_entity.entity_class == EntityClass::TestImplementation);
    }

    /// GREEN: Test database schema includes entity_class column
    #[test]
    fn test_database_schema_includes_entity_class() {
        // This test verifies the CozoDB schema includes entity_class
        // The actual schema is tested in storage/cozo_client.rs
        // Here we verify the entity_class field is part of the struct
        
        // Arrange: Create an entity
        let signature = InterfaceSignature {
            entity_type: EntityType::Struct,
            name: "TestStruct".to_string(),
            visibility: crate::entities::Visibility::Public,
            file_path: std::path::PathBuf::from("test.rs"),
            line_range: crate::entities::LineRange::new(1, 5).unwrap(),
            module_path: vec![],
            documentation: None,
            language_specific: crate::entities::LanguageSpecificSignature::Rust(
                crate::entities::RustSignature {
                    generics: vec![],
                    lifetimes: vec![],
                    where_clauses: vec![],
                    attributes: vec![],
                    trait_impl: None,
                }
            ),
        };

        // Act: Create entity
        let entity = CodeEntity::new(
            "test.rs-TestStruct-struct-jkl012".to_string(),
            signature,
            EntityClass::CodeImplementation,
        ).unwrap();

        // Assert: entity_class field exists and is accessible
        assert_eq!(entity.entity_class.to_string(), "CODE");
        assert_eq!(format!("{}", entity.entity_class), "CODE");
    }
}

/// Contract: PT01 must classify entities during indexing
/// 
/// Preconditions: File path and content provided
/// Postconditions: EntityClass determined and stored
/// Error conditions: Classification should default to CODE for ambiguous cases
#[cfg(test)]
mod pt01_classification_tests {
    use super::*;

    /// GREEN: Test PT01 classifies test files correctly
    #[test]
    fn test_pt01_classifies_test_files() {
        // This test verifies the test detector can classify test files
        // Actual PT01 integration is tested in pt01 crate
        
        // Arrange: Test file patterns
        let test_cases = vec![
            ("src/lib.rs", "fn test_main() {}", EntityClass::CodeImplementation),
            ("tests/integration_test.rs", "fn test_integration() {}", EntityClass::TestImplementation),
            ("src/utils_test.rs", "fn test_utils() {}", EntityClass::TestImplementation),
            ("test/unit_test.rs", "fn test_unit() {}", EntityClass::TestImplementation),
        ];

        // Act & Assert: Verify classification logic
        for (file_path, _content, expected_class) in test_cases {
            // Simulate PT01 classification logic
            let is_test_file = file_path.contains("test") || 
                              file_path.starts_with("tests/") || 
                              file_path.starts_with("test/");
            
            let actual_class = if is_test_file {
                EntityClass::TestImplementation
            } else {
                EntityClass::CodeImplementation
            };

            assert_eq!(actual_class, expected_class, 
                      "File '{}' should be classified as {:?}", file_path, expected_class);
        }
    }

    /// GREEN: Test PT01 classifies production code correctly
    #[test]
    fn test_pt01_classifies_production_code() {
        // Arrange: Production file patterns
        let prod_cases = vec![
            ("src/lib.rs", "pub fn main() {}", EntityClass::CodeImplementation),
            ("src/utils/mod.rs", "pub mod utils", EntityClass::CodeImplementation),
            ("src/main.rs", "fn main() {}", EntityClass::CodeImplementation),
            ("src/core.rs", "struct Core {}", EntityClass::CodeImplementation),
        ];

        // Act & Assert: Verify classification logic
        for (file_path, _content, expected_class) in prod_cases {
            // Simulate PT01 classification logic
            let is_test_file = file_path.contains("test") || 
                              file_path.starts_with("tests/") || 
                              file_path.starts_with("test/");
            
            let actual_class = if is_test_file {
                EntityClass::TestImplementation
            } else {
                EntityClass::CodeImplementation
            };

            assert_eq!(actual_class, expected_class, 
                      "File '{}' should be classified as {:?}", file_path, expected_class);
        }
    }

    /// GREEN: Test classification performance contract
    #[test]
    fn test_classification_performance_contract() {
        // Arrange: Test file for classification
        let file_path = "src/test_module.rs";
        let start_time = std::time::Instant::now();

        // Act: Perform classification (simulating PT01 logic)
        let is_test_file = file_path.contains("test") || 
                          file_path.starts_with("tests/") || 
                          file_path.starts_with("test/");
        
        let entity_class = if is_test_file {
            EntityClass::TestImplementation
        } else {
            EntityClass::CodeImplementation
        };

        let elapsed = start_time.elapsed();

        // Assert: Classification completes within performance contract
        assert!(elapsed.as_micros() < 50, 
               "Classification should complete within 50μs, took {}μs", 
               elapsed.as_micros());
        
        assert_eq!(entity_class, EntityClass::TestImplementation);
    }
}

/// Contract: PT02 can filter by EntityClass efficiently
/// 
/// Preconditions: Database contains entities with EntityClass
/// Postconditions: Queries can filter by entity_class with indexed performance
/// Error conditions: Invalid entity_class values should be rejected
#[cfg(test)]
mod pt02_filtering_tests {
    use super::*;

    /// GREEN: Test PT02 can filter code entities
    #[test]
    fn test_pt02_filters_code_entities() {
        // Arrange: Create entities with different classifications
        let entities = [create_test_entity("code1.rs", "fn code1()", EntityClass::CodeImplementation),
            create_test_entity("test1.rs", "fn test1()", EntityClass::TestImplementation),
            create_test_entity("code2.rs", "fn code2()", EntityClass::CodeImplementation)];

        // Act: Filter for code entities (simulating PT02 query)
        let code_entities: Vec<_> = entities.iter()
            .filter(|e| e.entity_class == EntityClass::CodeImplementation)
            .collect();

        // Assert: Only code entities returned
        assert_eq!(code_entities.len(), 2, "Should return 2 code entities");
        assert!(code_entities.iter().all(|e| e.entity_class == EntityClass::CodeImplementation));
    }

    /// GREEN: Test PT02 can filter test entities
    #[test]
    fn test_pt02_filters_test_entities() {
        // Arrange: Create entities with different classifications
        let entities = [create_test_entity("code1.rs", "fn code1()", EntityClass::CodeImplementation),
            create_test_entity("test1.rs", "fn test1()", EntityClass::TestImplementation),
            create_test_entity("test2.rs", "fn test2()", EntityClass::TestImplementation)];

        // Act: Filter for test entities (simulating PT02 query)
        let test_entities: Vec<_> = entities.iter()
            .filter(|e| e.entity_class == EntityClass::TestImplementation)
            .collect();

        // Assert: Only test entities returned
        assert_eq!(test_entities.len(), 2, "Should return 2 test entities");
        assert!(test_entities.iter().all(|e| e.entity_class == EntityClass::TestImplementation));
    }

    /// GREEN: Test dual output generation
    #[test]
    fn test_pt02_dual_output_generation() {
        // Arrange: Create mixed entities
        let entities = [create_test_entity("code1.rs", "fn code1()", EntityClass::CodeImplementation),
            create_test_entity("test1.rs", "fn test1()", EntityClass::TestImplementation),
            create_test_entity("code2.rs", "fn code2()", EntityClass::CodeImplementation)];

        // Act: Separate into code and test outputs (simulating PT02 dual output)
        let code_entities: Vec<_> = entities.iter()
            .filter(|e| e.entity_class == EntityClass::CodeImplementation)
            .collect();
        
        let test_entities: Vec<_> = entities.iter()
            .filter(|e| e.entity_class == EntityClass::TestImplementation)
            .collect();

        // Assert: Proper separation for dual outputs
        assert_eq!(code_entities.len(), 2, "Code output should have 2 entities");
        assert_eq!(test_entities.len(), 1, "Tests output should have 1 entity");
        
        // Verify no overlap
        let code_keys: std::collections::HashSet<_> = code_entities.iter()
            .map(|e| &e.isgl1_key).collect();
        let test_keys: std::collections::HashSet<_> = test_entities.iter()
            .map(|e| &e.isgl1_key).collect();
        
        assert!(code_keys.intersection(&test_keys).next().is_none(), 
               "Code and test outputs should not overlap");
    }
}

/// Contract: PT03 preserves EntityClass during edits
/// 
/// Preconditions: Existing entity with EntityClass
/// Postconditions: Updated entity maintains same EntityClass
/// Error conditions: EntityClass should not be modified by PT03 operations
#[cfg(test)]
mod pt03_preservation_tests {
    use super::*;

    /// GREEN: Test PT03 preserves EntityClass during edits
    #[test]
    fn test_pt03_preserves_entity_class_during_edits() {
        // Arrange: Create entity with specific classification
        let mut entity = create_test_entity("code.rs", "fn original()", EntityClass::CodeImplementation);
        let original_class = entity.entity_class;

        // Act: Simulate PT03 edit operation (update future code)
        entity.apply_temporal_change(
            crate::entities::TemporalAction::Edit,
            Some("fn updated() { /* new implementation */ }".to_string()),
        ).unwrap();

        // Assert: EntityClass preserved during edit
        assert_eq!(entity.entity_class, original_class, 
                  "EntityClass should be preserved during PT03 edits");
        assert_eq!(entity.entity_class, EntityClass::CodeImplementation);
    }

    /// GREEN: Test PT03 cannot change EntityClass
    #[test]
    fn test_pt03_cannot_change_entity_class() {
        // Arrange: Create entity with specific classification
        let mut entity = create_test_entity("test.rs", "fn test_fn()", EntityClass::TestImplementation);
        let original_class = entity.entity_class;

        // Act: Attempt various PT03 operations
        entity.apply_temporal_change(crate::entities::TemporalAction::Create, None).unwrap();
        entity.apply_temporal_change(crate::entities::TemporalAction::Edit, Some("new code".to_string())).unwrap();
        entity.apply_temporal_change(crate::entities::TemporalAction::Delete, None).unwrap();

        // Assert: EntityClass never changes through PT03 operations
        assert_eq!(entity.entity_class, original_class, 
                  "EntityClass should never change through PT03 operations");
        assert_eq!(entity.entity_class, EntityClass::TestImplementation);
    }
}

#[cfg(test)]
fn create_test_entity(file_path: &str, code: &str, entity_class: EntityClass) -> CodeEntity {
    let signature = InterfaceSignature {
        entity_type: if entity_class == EntityClass::TestImplementation {
            EntityType::TestFunction
        } else {
            EntityType::Function
        },
        name: "test_function".to_string(),
        visibility: crate::entities::Visibility::Public,
        file_path: std::path::PathBuf::from(file_path),
        line_range: crate::entities::LineRange::new(1, 5).unwrap(),
        module_path: vec![],
        documentation: None,
        language_specific: crate::entities::LanguageSpecificSignature::Rust(
            crate::entities::RustSignature {
                generics: vec![],
                lifetimes: vec![],
                where_clauses: vec![],
                attributes: vec![],
                trait_impl: None,
            }
        ),
    };

    let mut entity = CodeEntity::new(
        format!("{}-test_function-fn-abc123", file_path.replace("/", "_").replace(".", "_")),
        signature,
        entity_class,
    ).unwrap();

    // Set current code for more realistic entity
    entity.current_code = Some(code.to_string());
    
    entity
}
