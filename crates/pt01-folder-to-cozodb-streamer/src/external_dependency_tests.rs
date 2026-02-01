//! TDD Tests for External Dependency Placeholder Node Creation (Bug #4)
//!
//! ## Problem Statement
//!
//! Blast radius fails because external dependencies don't exist in the database.
//! When code calls `build_cli()` from an external crate, we create a dependency edge
//! pointing to `rust:fn:build_cli:unknown:0-0`, but this entity doesn't exist in the
//! database, causing graph traversal to fail.
//!
//! ## Architectural Solution
//!
//! Instead of skipping external dependencies, CREATE placeholder CodeEntity nodes:
//! 1. Detect external dependency references (imports, uses, function calls to external code)
//! 2. Create placeholder CodeEntity for each external dependency
//! 3. Store with ISGL1 key like: `rust:fn:build_cli:external-dependency-crate:0-0`
//! 4. Mark with EntityClass::ExternalDependency (new enum variant needed)
//! 5. Connect via dependency edges to entities that reference them
//!
//! ## TDD Cycle: RED → GREEN → REFACTOR
//!
//! This file contains RED tests that will initially fail.
//! Implementation will make them GREEN.
//! Then we REFACTOR to clean code.

#[cfg(test)]
mod external_dependency_placeholder_tests {
    use std::path::PathBuf;
    use parseltongue_core::entities::Language;
    use crate::isgl1_generator::{Isgl1KeyGeneratorImpl, Isgl1KeyGenerator};
    use crate::external_dependency_handler::create_external_dependency_placeholder_entity_validated;

    /// RED TEST 1: Parse Rust `use` statement and detect external crate
    ///
    /// **Preconditions**:
    /// - Source contains `use clap::Parser;`
    /// - `clap` is an external dependency (not in local codebase)
    ///
    /// **Expected Behavior**:
    /// - Parser detects `clap::Parser` as external dependency
    /// - Extracts crate name: "clap"
    /// - Extracts item path: "Parser"
    ///
    /// **Postconditions**:
    /// - Function returns external dependency metadata
    /// - Metadata includes: crate name, item path, import location
    ///
    /// **Current Status**: RED (will fail - no external dependency detection yet)
    #[test]
    fn test_parse_rust_use_detects_external_crate() {
        // Arrange: Rust code with external dependency
        let source = r#"
            use clap::Parser;

            fn main() {
                Parser::parse();
            }
        "#;

        let file_path = PathBuf::from("src/main.rs");
        let generator = Isgl1KeyGeneratorImpl::new();

        // Act: Parse source and extract dependencies
        let (_entities, dependencies) = generator
            .parse_source(source, &file_path)
            .expect("Failed to parse source");

        // Assert: External dependency detected
        // We should have a dependency edge pointing to external `clap::Parser`
        let has_external_dep = dependencies.iter().any(|dep| {
            dep.to_key.as_str().contains("clap") &&
            dep.to_key.as_str().contains("Parser")
        });

        assert!(
            has_external_dep,
            "Should detect external dependency to clap::Parser, but found: {:?}",
            dependencies
        );

        // Verify the edge points to an external entity key
        let external_dep = dependencies.iter()
            .find(|dep| dep.to_key.as_str().contains("clap"))
            .expect("Should have clap dependency");

        // External dependency key should have format: rust:type:name:external-dependency-crate:0-0
        assert!(
            external_dep.to_key.as_str().contains(":external-dependency-"),
            "External dependency key should contain ':external-dependency-' marker: {}",
            external_dep.to_key.as_str()
        );

        assert!(
            external_dep.to_key.as_str().ends_with(":0-0"),
            "External dependency should have line range 0-0: {}",
            external_dep.to_key.as_str()
        );
    }

    /// RED TEST 2: Create placeholder CodeEntity for external dependency
    ///
    /// **Preconditions**:
    /// - External dependency detected: `tokio::runtime::Runtime`
    /// - Need to create CodeEntity placeholder
    ///
    /// **Expected Behavior**:
    /// - Create CodeEntity with ISGL1 key: `rust:struct:Runtime:external-dependency-tokio:0-0`
    /// - EntityClass = ExternalDependency (new variant)
    /// - Language extracted from importing file context
    /// - Line range = 0-0 (indicates external)
    ///
    /// **Postconditions**:
    /// - CodeEntity created and validated
    /// - Can be stored in database
    /// - Can be queried via ISGL1 key
    ///
    /// **Current Status**: RED (will fail - no placeholder creation logic yet)
    #[test]
    fn test_create_external_dependency_placeholder_entity() {
        // Arrange: External dependency metadata
        let crate_name = "tokio";
        let item_name = "Runtime";
        let item_type = "struct";
        let language = Language::Rust;

        // Act: Create placeholder entity (function to be implemented)
        // This will fail until we implement `create_external_dependency_placeholder_entity_validated()`
        let result = create_external_dependency_placeholder_entity_validated(
            crate_name,
            item_name,
            item_type,
            language,
        );

        // Assert: Placeholder entity created with correct format
        assert!(
            result.is_ok(),
            "Should create external dependency placeholder: {:?}",
            result.err()
        );

        let entity = result.unwrap();

        // Verify ISGL1 key format
        let expected_key_pattern = format!("rust:{}:{}:external-dependency-{}", item_type, item_name, crate_name);
        assert!(
            entity.isgl1_key.contains(&expected_key_pattern),
            "Key should match pattern '{}', got: {}",
            expected_key_pattern,
            entity.isgl1_key
        );

        // Verify line range is 0-0 (external marker)
        assert_eq!(
            entity.interface_signature.line_range.start, 0,
            "External dependency should have line start = 0"
        );
        assert_eq!(
            entity.interface_signature.line_range.end, 0,
            "External dependency should have line end = 0"
        );

        // Verify EntityClass (will fail until we add ExternalDependency variant)
        // TODO: Uncomment after adding EntityClass::ExternalDependency
        // assert_eq!(
        //     entity.entity_class,
        //     EntityClass::ExternalDependency,
        //     "External dependencies should have ExternalDependency class"
        // );
    }

    /// RED TEST 3: Store external dependency placeholder in database
    ///
    /// **Preconditions**:
    /// - Database initialized with schema
    /// - Placeholder entity created for `anyhow::Error`
    ///
    /// **Expected Behavior**:
    /// - Store placeholder entity in `CodeEntities` relation
    /// - Store dependency edge in `DependencyEdges` relation
    /// - Query returns external dependency
    ///
    /// **Postconditions**:
    /// - External dependency queryable by ISGL1 key
    /// - Dependency edges connect local code to external dependency
    ///
    /// **Current Status**: RED (will fail - no storage logic for external deps yet)
    #[test]
    #[ignore = "Integration test - requires database"]
    fn test_store_external_dependency_in_database() {
        // Arrange: Create test database and placeholder entity
        // (Implementation details depend on database testing infrastructure)

        // This test will be GREEN once we:
        // 1. Create placeholder entities during parsing
        // 2. Store them alongside regular entities
        // 3. Ensure they're queryable

        todo!("Implement database storage test for external dependencies");
    }

    /// RED TEST 4: Blast radius query includes external dependencies
    ///
    /// **Preconditions**:
    /// - Local function `main()` calls external `clap::Parser::parse()`
    /// - Both entities and edges stored in database
    ///
    /// **Expected Behavior**:
    /// - Blast radius query for `main()` includes `clap::Parser::parse()`
    /// - Graph traversal doesn't fail on external dependency
    /// - Returns both local and external affected entities
    ///
    /// **Postconditions**:
    /// - Query returns success (not "No affected entities found")
    /// - External dependencies visible in blast radius results
    ///
    /// **Current Status**: RED (will fail - current behavior is "No affected entities found")
    #[test]
    #[ignore = "Integration test - requires database and HTTP server"]
    fn test_blast_radius_includes_external_dependencies() {
        // Arrange: Database with local and external entities
        // (Implementation details depend on HTTP server testing infrastructure)

        // This test will be GREEN once we:
        // 1. Create external dependency placeholders
        // 2. Store them in database
        // 3. Blast radius query traverses external edges

        todo!("Implement blast radius integration test with external dependencies");
    }

    /// RED TEST 5: Detect both :external-dependency- and :unknown:0-0 patterns
    ///
    /// **Phase 1 TDD Test**: Extend extract_placeholders_from_edges_deduplicated()
    ///
    /// **Preconditions**:
    /// - Edges contain both external-dependency and unknown patterns
    ///
    /// **Expected Behavior**:
    /// - Both pattern types detected and placeholders created
    /// - Deduplication works across both patterns
    ///
    /// **Postconditions**:
    /// - All unknown:0-0 references have placeholder entities
    ///
    /// **Current Status**: RED (will fail - only detects :external-dependency-)
    #[test]
    fn test_extract_placeholders_unknown_pattern_detected() {
        use parseltongue_core::entities::{DependencyEdge, EdgeType, Isgl1Key};
        use crate::external_dependency_handler::extract_placeholders_from_edges_deduplicated;

        // Arrange: Mix of external-dependency and unknown patterns
        let edges = vec![
            // Known external dependency
            DependencyEdge {
                from_key: Isgl1Key::new("rust:fn:main:src/main.rs:10-15").unwrap(),
                to_key: Isgl1Key::new("rust:module:Parser:external-dependency-clap:0-0").unwrap(),
                edge_type: EdgeType::Uses,
                source_location: None,
            },
            // Unknown pattern (unresolved reference)
            DependencyEdge {
                from_key: Isgl1Key::new("rust:fn:main:src/main.rs:20-25").unwrap(),
                to_key: Isgl1Key::new("rust:fn:build_cli:unknown:0-0").unwrap(),
                edge_type: EdgeType::Calls,
                source_location: None,
            },
            // Another unknown pattern
            DependencyEdge {
                from_key: Isgl1Key::new("rust:fn:handler:src/api.rs:30-35").unwrap(),
                to_key: Isgl1Key::new("rust:struct:Config:unknown:0-0").unwrap(),
                edge_type: EdgeType::Uses,
                source_location: None,
            },
        ];

        // Act
        let placeholders = extract_placeholders_from_edges_deduplicated(&edges);

        // Assert: Should create placeholders for BOTH patterns
        assert_eq!(
            placeholders.len(),
            3,
            "Should create 3 placeholders (1 external-dependency + 2 unknown), got: {:?}",
            placeholders.iter().map(|p| &p.isgl1_key).collect::<Vec<_>>()
        );

        // Verify external-dependency pattern still works
        let external_placeholder = placeholders.iter()
            .find(|p| p.isgl1_key.contains("external-dependency-clap"))
            .expect("Should have external-dependency placeholder");

        assert_eq!(
            external_placeholder.isgl1_key,
            "rust:module:Parser:external-dependency-clap:0-0"
        );

        // Verify unknown pattern creates placeholders
        let unknown_placeholders: Vec<_> = placeholders.iter()
            .filter(|p| p.isgl1_key.contains(":unknown:0-0"))
            .collect();

        assert_eq!(
            unknown_placeholders.len(),
            2,
            "Should have 2 unknown placeholders"
        );

        // Verify unknown placeholder keys exist
        let keys: Vec<&str> = unknown_placeholders.iter()
            .map(|p| p.isgl1_key.as_str())
            .collect();

        assert!(
            keys.contains(&"rust:fn:build_cli:unknown:0-0"),
            "Should have build_cli placeholder, got: {:?}",
            keys
        );
        assert!(
            keys.contains(&"rust:struct:Config:unknown:0-0"),
            "Should have Config placeholder, got: {:?}",
            keys
        );
    }

    // ========================================================================
    // Helper Functions
    // ========================================================================
    //
    // Note: Helper function moved to production module:
    // crate::external_dependency_handler::create_external_dependency_placeholder_entity_validated
}
