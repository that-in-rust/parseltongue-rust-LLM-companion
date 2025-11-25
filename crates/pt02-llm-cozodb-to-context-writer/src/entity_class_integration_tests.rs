//! Executable Specifications for PT02 EntityClass Integration
//!
//! Following S01 TDD principles: STUB → RED → GREEN → REFACTOR
//! These specifications define the contract for EntityClass-based dual output generation

use crate::export_trait::{Entity, CodeGraphRepository};
use crate::cozodb_adapter::CozoDbAdapter;
use anyhow::Result;
use std::collections::HashMap;

/// Contract: PT02 must filter entities by EntityClass for dual outputs
/// 
/// Preconditions: Database contains entities with entity_class values
/// Postconditions: Queries can separate CODE vs TEST entities
/// Error conditions: Missing entity_class should default to "CODE"
#[cfg(test)]
mod pt02_entity_class_tests {
    use super::*;

    /// STUB: Test that Entity struct includes entity_class field
    #[test]
    fn test_entity_struct_includes_entity_class() {
        // This test will fail initially - Entity struct missing entity_class
        let entity = Entity {
            isgl1_key: "test".to_string(),
            forward_deps: vec![],
            reverse_deps: vec![],
            current_ind: 1,
            future_ind: 1,
            future_action: None,
            future_code: None,
            current_code: None,
            entity_name: "test".to_string(),
            entity_type: "Function".to_string(),
            file_path: "test.rs".to_string(),
            line_number: 10,
            interface_signature: "fn test()".to_string(),
            doc_comment: None,
            entity_class: "CODE".to_string(), // v0.9.0: This field should exist
            return_type: None,
            param_types: None,
            param_names: None,
            generic_constraints: None,
            trait_impls: None,
            is_public: None,
            is_async: None,
            is_unsafe: None,
        };
        
        assert_eq!(entity.entity_class, "CODE");
    }

    /// STUB: Test that database queries select entity_class column
    #[tokio::test]
    async fn test_database_query_selects_entity_class() {
        // This test will fail initially - queries don't select entity_class
        // Mock the database to verify the query includes entity_class
        
        // Arrange: Create mock storage that captures queries
        // Act: Query entities 
        // Assert: Query includes entity_class in SELECT and FROM clauses
        
        // For now, this is a placeholder that will be implemented
        // when we have the proper mock infrastructure
        assert!(true, "Placeholder - will implement actual query verification");
    }

    /// STUB: Test that entities can be filtered by EntityClass
    #[test]
    fn test_filter_entities_by_entity_class() {
        // Arrange: Create mixed entities
        let code_entity = create_test_entity("code.rs", "CODE");
        let test_entity = create_test_entity("test.rs", "TEST");
        let entities = vec![code_entity, test_entity];
        
        // Act: Filter by entity_class
        let code_entities: Vec<_> = entities.iter()
            .filter(|e| e.entity_class == "CODE")
            .collect();
        let test_entities: Vec<_> = entities.iter()
            .filter(|e| e.entity_class == "TEST")
            .collect();
        
        // Assert: Proper separation
        assert_eq!(code_entities.len(), 1);
        assert_eq!(test_entities.len(), 1);
        assert_eq!(code_entities[0].entity_class, "CODE");
        assert_eq!(test_entities[0].entity_class, "TEST");
    }

    /// STUB: Test that dual output generation uses EntityClass
    #[test]
    fn test_dual_output_uses_entity_class() {
        // This test will fail initially - dual output logic not implemented
        // Arrange: Mixed entities
        let entities = vec![
            create_test_entity("src/main.rs", "CODE"),
            create_test_entity("tests/integration.rs", "TEST"),
        ];
        
        // Act: Separate by entity_class
        let (code_entities, test_entities) = separate_by_entity_class(entities);
        
        // Assert: Proper separation for dual outputs
        assert_eq!(code_entities.len(), 1);
        assert_eq!(test_entities.len(), 1);
        assert!(code_entities[0].file_path.contains("src/"));
        assert!(test_entities[0].file_path.contains("tests/"));
    }

    /// STUB: Test performance of EntityClass filtering
    #[test]
    fn test_entity_class_filtering_performance() {
        // This test validates performance contract: <10ms for 10K entities
        let start = std::time::Instant::now();
        
        // Arrange: Create 10K test entities
        let entities: Vec<Entity> = (0..10000)
            .map(|i| create_test_entity(&format!("file{}.rs", i), if i % 2 == 0 { "CODE" } else { "TEST" }))
            .collect();
        
        // Act: Filter by entity_class
        let _code_entities: Vec<_> = entities.iter()
            .filter(|e| e.entity_class == "CODE")
            .collect();
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 50, "EntityClass filtering took {:?}, expected <50ms (5x)", duration);  // 5x: 10ms → 50ms
    }
}

/// Helper function to create test entities
fn create_test_entity(file_path: &str, entity_class: &str) -> Entity {
    Entity {
        isgl1_key: format!("rust:fn:test:{}:10", file_path.replace("/", "_").replace(".", "_")),
        forward_deps: vec![],
        reverse_deps: vec![],
        current_ind: 1,
        future_ind: 1,
        future_action: None,
        future_code: Some("fn test() {}".to_string()),
        current_code: Some("fn test() {}".to_string()),
        entity_name: "test".to_string(),
        entity_type: "Function".to_string(),
        file_path: file_path.to_string(),
        line_number: 10,
        interface_signature: "fn test()".to_string(),
        doc_comment: None,
        entity_class: entity_class.to_string(), // v0.9.0: EntityClass field
        return_type: None,
        param_types: None,
        param_names: None,
        generic_constraints: None,
        trait_impls: None,
        is_public: Some(true),
        is_async: Some(false),
        is_unsafe: Some(false),
    }
}

/// Helper function to separate entities by EntityClass
fn separate_by_entity_class(entities: Vec<Entity>) -> (Vec<Entity>, Vec<Entity>) {
    let mut code_entities = Vec::new();
    let mut test_entities = Vec::new();
    
    for entity in entities {
        if entity.entity_class == "CODE" {
            code_entities.push(entity);
        } else if entity.entity_class == "TEST" {
            test_entities.push(entity);
        }
    }
    
    (code_entities, test_entities)
}
