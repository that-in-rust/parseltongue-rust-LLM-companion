//! External Dependency Placeholder Handler
//!
//! Creates placeholder CodeEntity nodes for external dependencies detected in dependency edges.
//!
//! ## Problem
//!
//! When code imports external libraries (e.g., `use clap::Parser`), the ISGL1 generator creates
//! dependency edges pointing to keys like `rust:module:Parser:external-dependency-clap:0-0`.
//! However, these entities don't exist in the database, causing:
//! - Orphaned edges (edges with no target node)
//! - Blast radius query failures ("No affected entities found")
//! - Incomplete dependency graphs
//!
//! ## Solution
//!
//! This module extracts external dependency references from edges and creates placeholder
//! CodeEntity nodes with:
//! - ISGL1 key format: `{lang}:{type}:{name}:external-dependency-{crate}:0-0`
//! - Line range: 0-0 (special marker for external dependencies)
//! - EntityClass: CodeImplementation (external dependencies are still code)
//!
//! ## Integration
//!
//! Called from `streamer.rs::stream_file()` after parsing:
//! ```ignore
//! let (entities, edges) = parser.parse_source(...)?;
//! let external_placeholders = extract_placeholders_from_edges_deduplicated(&edges);
//! let all_entities = [entities, external_placeholders].concat();
//! // ... store all_entities ...
//! ```

use std::collections::HashSet;
use std::path::PathBuf;
use parseltongue_core::entities::*;
use crate::errors::*;

/// Extract external dependency placeholders from dependency edges
///
/// **Four-Word Name**: extract_placeholders_from_edges_deduplicated
/// - extract: verb (action)
/// - placeholders: target (what we're extracting)
/// - from: preposition
/// - edges: source
/// - deduplicated: qualifier (ensures uniqueness)
///
/// ## Algorithm
///
/// 1. Scan all edges for `to_key` containing `:external-dependency-` OR `:unknown:0-0`
/// 2. Parse each external/unknown key to extract: language, type, name, crate
/// 3. Create placeholder entity for each unique external dependency or unresolved reference
/// 4. Deduplicate using HashSet (same dependency may be referenced by multiple files)
///
/// ## Supported Patterns
///
/// - **External Dependencies**: `{lang}:{type}:{name}:external-dependency-{crate}:0-0`
///   - Created for imported items from external crates (e.g., `use clap::Parser`)
/// - **Unresolved References**: `{lang}:{type}:{name}:unknown:0-0`
///   - Created for calls/references where target location is unknown
///   - Covers 6 scenarios: external deps, local functions, trait implementations, macros, generics, dynamic dispatch
///
/// ## Preconditions
///
/// - `edges` contains dependency edges from ISGL1 generator
/// - Keys follow ISGL1 format with 5 colon-separated parts
///
/// ## Postconditions
///
/// - Returns Vec<CodeEntity> with placeholder entities
/// - Each placeholder has unique ISGL1 key
/// - Line range = 0-0 for all placeholders (marker for external/unresolved)
/// - No duplicate placeholders (deduplication applied)
///
/// ## Error Handling
///
/// - Invalid keys are silently skipped (graceful degradation)
/// - Parse errors logged but don't fail the entire operation
///
/// ## Example
///
/// ```ignore
/// let edges = vec![
///     DependencyEdge {
///         from_key: "rust:fn:main:src/main.rs:10-15".into(),
///         to_key: "rust:module:Parser:external-dependency-clap:0-0".into(),  // Known external
///         edge_type: EdgeType::Uses,
///     },
///     DependencyEdge {
///         from_key: "rust:fn:build_cli:src/cli.rs:5-10".into(),
///         to_key: "rust:fn:helper:unknown:0-0".into(),  // Unresolved reference
///         edge_type: EdgeType::Calls,
///     },
/// ];
///
/// let placeholders = extract_placeholders_from_edges_deduplicated(&edges);
/// assert_eq!(placeholders.len(), 2); // Both patterns detected
/// ```
pub fn extract_placeholders_from_edges_deduplicated(
    edges: &[DependencyEdge],
) -> Vec<CodeEntity> {
    let mut seen_keys = HashSet::new();
    let mut placeholders = Vec::new();

    for edge in edges {
        // Check if target is an external dependency OR unknown reference
        let to_key_str: &str = edge.to_key.as_ref();
        if to_key_str.contains(":external-dependency-") || to_key_str.contains(":unknown:0-0") {
            // Deduplicate: only process first occurrence of each key
            if seen_keys.insert(edge.to_key.clone()) {
                // Parse external dependency key
                if let Ok((language, entity_type, item_name, crate_name)) =
                    parse_external_key_parts_validated(to_key_str)
                {
                    // Create placeholder entity
                    match create_external_dependency_placeholder_entity_validated(
                        &crate_name,
                        &item_name,
                        &entity_type,
                        language,
                    ) {
                        Ok(placeholder) => {
                            placeholders.push(placeholder);
                        }
                        Err(e) => {
                            // Graceful degradation: log error but continue processing
                            eprintln!(
                                "Warning: Failed to create placeholder for {}: {}",
                                edge.to_key, e
                            );
                        }
                    }
                } else {
                    // Invalid key format - skip silently
                    eprintln!(
                        "Warning: Invalid external dependency key format: {}",
                        edge.to_key
                    );
                }
            }
        }
    }

    placeholders
}

/// Parse external dependency key into components
///
/// **Four-Word Name**: parse_external_key_parts_validated
/// - parse: verb (action)
/// - external: constraint (not local)
/// - key: target
/// - parts: qualifier (breaking into components)
/// - validated: ensures correctness
///
/// ## Key Formats
///
/// Supports two patterns:
///
/// **1. External Dependencies (Known Crates)**:
/// ```text
/// rust:module:Parser:external-dependency-clap:0-0
/// └─┬─┘└──┬──┘└──┬──┘└──────────┬────────────┘└┬┘
///  lang  type  name      file_marker         lines
/// ```
///
/// **2. Unresolved References (Unknown Location)**:
/// ```text
/// rust:fn:build_cli:unknown:0-0
/// └─┬─┘└┬┘└───┬───┘└──┬──┘└┬┘
///  lang type name  marker lines
/// ```
///
/// ## Preconditions
///
/// - `key` is a valid ISGL1 key string
/// - Key contains `:external-dependency-{crate}` OR `:unknown:0-0`
/// - Key has 5 colon-separated parts
///
/// ## Postconditions
///
/// - Returns Ok((Language, entity_type, name, crate_name)) on success
/// - For unknown pattern: crate_name = "unresolved-reference"
/// - For external pattern: crate_name = actual crate name
/// - Returns Err for invalid formats
///
/// ## Error Conditions
///
/// - Wrong number of parts (not 5)
/// - Missing both `:external-dependency-` and `:unknown` markers
/// - Invalid language prefix
///
/// ## Examples
///
/// ```ignore
/// // External dependency
/// let key = "rust:module:Parser:external-dependency-clap:0-0";
/// let (lang, typ, name, crate_name) = parse_external_key_parts_validated(key)?;
/// assert_eq!(crate_name, "clap");
///
/// // Unresolved reference
/// let key = "rust:fn:build_cli:unknown:0-0";
/// let (lang, typ, name, crate_name) = parse_external_key_parts_validated(key)?;
/// assert_eq!(crate_name, "unresolved-reference");
/// ```
pub fn parse_external_key_parts_validated(
    key: &str,
) -> Result<(Language, String, String, String)> {
    let parts: Vec<&str> = key.split(':').collect();

    // Validate format: lang:type:name:external-dependency-crate:0-0
    if parts.len() != 5 {
        return Err(StreamerError::ParsingError {
            file: "external_dependency".to_string(),
            reason: format!(
                "Expected 5 parts in external dependency key, got {}: {}",
                parts.len(),
                key
            ),
        });
    }

    let language_str = parts[0];
    let entity_type = parts[1].to_string();
    let item_name = parts[2].to_string();
    let file_path = parts[3];

    // Extract crate name from pattern
    // Supports two patterns:
    // 1. "external-dependency-{crate}" - Known external crate from USE statements
    // 2. "unknown" - Unresolved reference (6 scenarios: external deps, local functions, traits, macros, generics, dynamic dispatch)
    let crate_name = if file_path == "unknown" {
        // Unknown pattern: map to special "unresolved-reference" crate
        "unresolved-reference".to_string()
    } else if let Some(crate_name) = file_path.strip_prefix("external-dependency-") {
        // Known external dependency pattern
        crate_name.to_string()
    } else {
        // Invalid format - neither pattern matched
        return Err(StreamerError::ParsingError {
            file: "external_dependency".to_string(),
            reason: format!(
                "Invalid file_path format (expected 'external-dependency-{{crate}}' or 'unknown'): {}",
                key
            ),
        });
    };

    // Map language string to Language enum
    let language = match language_str {
        "rust" => Language::Rust,
        "javascript" => Language::JavaScript,
        "typescript" => Language::TypeScript,
        "python" => Language::Python,
        "java" => Language::Java,
        "c" => Language::C,
        "cpp" => Language::Cpp,
        "go" => Language::Go,
        "ruby" => Language::Ruby,
        "php" => Language::Php,
        "csharp" => Language::CSharp,
        "swift" => Language::Swift,
        "kotlin" => Language::Kotlin,
        "scala" => Language::Scala,
        _ => {
            return Err(StreamerError::ParsingError {
                file: "external_dependency".to_string(),
                reason: format!("Unknown language prefix: {}", language_str),
            })
        }
    };

    Ok((language, entity_type, item_name, crate_name))
}

/// Create external dependency placeholder entity
///
/// **Function Name**: create_external_dependency_placeholder_entity_validated
/// (6 words - EXCEPTION: moved from test module, keeping original name for consistency)
///
/// ## Design Decisions
///
/// - Line range 0-0 indicates external (not in local codebase)
/// - ISGL1 key format: `{language}:{type}:{name}:external-dependency-{crate}:0-0`
/// - EntityClass: CodeImplementation (external deps are still code, not tests)
/// - Temporal state: initial() (exists in current codebase's imports)
///
/// ## Preconditions
///
/// - `crate_name` is non-empty external crate identifier
/// - `item_name` is non-empty entity identifier
/// - `item_type` matches EntityType variants
/// - `language` is supported Language variant
///
/// ## Postconditions
///
/// - Returns Ok(CodeEntity) with valid external dependency placeholder
/// - Entity passes validate() checks (line range = 0-0 is allowed for external)
/// - ISGL1 key uniquely identifies external dependency
///
/// ## Error Conditions
///
/// - Empty crate_name or item_name → ValidationError
/// - Invalid item_type → ValidationError
///
/// ## Example
///
/// ```ignore
/// let placeholder = create_external_dependency_placeholder_entity_validated(
///     "tokio",
///     "Runtime",
///     "struct",
///     Language::Rust,
/// )?;
///
/// assert_eq!(placeholder.isgl1_key, "rust:struct:Runtime:external-dependency-tokio:0-0");
/// assert_eq!(placeholder.interface_signature.line_range.start, 0);
/// assert_eq!(placeholder.interface_signature.line_range.end, 0);
/// ```
pub fn create_external_dependency_placeholder_entity_validated(
    crate_name: &str,
    item_name: &str,
    item_type: &str,
    language: Language,
) -> Result<CodeEntity> {
    // Validate inputs
    if crate_name.is_empty() {
        return Err(StreamerError::ParsingError {
            file: "external_dependency".to_string(),
            reason: "crate_name cannot be empty".to_string(),
        });
    }
    if item_name.is_empty() {
        return Err(StreamerError::ParsingError {
            file: "external_dependency".to_string(),
            reason: "item_name cannot be empty".to_string(),
        });
    }

    // Map string type to EntityType enum
    let entity_type = match item_type {
        "struct" => EntityType::Struct,
        "fn" | "function" => EntityType::Function,
        "enum" => EntityType::Enum,
        "trait" => EntityType::Trait,
        "module" => EntityType::Module,
        "method" => EntityType::Method,
        "class" => EntityType::Class,
        "interface" => EntityType::Interface,
        "macro" => EntityType::Macro,
        "variable" | "var" => EntityType::Variable,
        "constant" | "const" => EntityType::Constant,
        _ => {
            return Err(StreamerError::ParsingError {
                file: "external_dependency".to_string(),
                reason: format!("Unknown item_type: {}", item_type),
            })
        }
    };

    // Create ISGL1 key: rust:struct:Runtime:external-dependency-tokio:0-0
    let language_prefix = match language {
        Language::Rust => "rust",
        Language::JavaScript => "javascript",
        Language::TypeScript => "typescript",
        Language::Python => "python",
        Language::Java => "java",
        Language::C => "c",
        Language::Cpp => "cpp",
        Language::Go => "go",
        Language::Ruby => "ruby",
        Language::Php => "php",
        Language::CSharp => "csharp",
        Language::Swift => "swift",
        Language::Kotlin => "kotlin",
        Language::Scala => "scala",
    };

    // Create ISGL1 key based on crate_name type
    // - Known external dependencies: rust:struct:Runtime:external-dependency-tokio:0-0
    // - Unresolved references: rust:fn:build_cli:unknown:0-0
    let (isgl1_key, file_path) = if crate_name == "unresolved-reference" {
        // Unknown pattern: use "unknown" in key
        let key = format!(
            "{}:{}:{}:unknown:0-0",
            language_prefix, item_type, item_name
        );
        let path = PathBuf::from("unknown");
        (key, path)
    } else {
        // External dependency pattern: use "external-dependency-{crate}" in key
        let key = format!(
            "{}:{}:{}:external-dependency-{}:0-0",
            language_prefix, item_type, item_name, crate_name
        );
        let path = PathBuf::from(format!("external-dependency-{}", crate_name));
        (key, path)
    };

    // Create language-specific signature (minimal for external dependencies)
    let language_specific = match language {
        Language::Rust => LanguageSpecificSignature::Rust(RustSignature {
            generics: vec![],
            lifetimes: vec![],
            where_clauses: vec![],
            attributes: vec![],
            trait_impl: None,
        }),
        Language::JavaScript => LanguageSpecificSignature::JavaScript(JavascriptSignature {
            parameters: vec![],
            return_type: None,
            is_async: false,
            is_arrow: false,
        }),
        Language::TypeScript => LanguageSpecificSignature::TypeScript(TypeScriptSignature {
            parameters: vec![],
            return_type: None,
            generics: vec![],
            is_async: false,
        }),
        Language::Python => LanguageSpecificSignature::Python(PythonSignature {
            parameters: vec![],
            return_type: None,
            is_async: false,
            decorators: vec![],
        }),
        Language::Java => LanguageSpecificSignature::Java(JavaSignature {
            access_modifier: AccessModifier::Public,
            parameters: vec![],
            return_type: "void".to_string(),
            throws: vec![],
            is_static: false,
            generics: vec![],
        }),
        _ => {
            // For unsupported languages, default to Rust signature structure
            LanguageSpecificSignature::Rust(RustSignature {
                generics: vec![],
                lifetimes: vec![],
                where_clauses: vec![],
                attributes: vec![],
                trait_impl: None,
            })
        }
    };

    // Create documentation based on placeholder type
    let documentation = if crate_name == "unresolved-reference" {
        // Unresolved reference: Explain the 6 scenarios this covers
        Some(
            "Unresolved reference - target location unknown. May be external dependency, \
            local function, trait implementation, macro expansion, generic instantiation, \
            or dynamic dispatch target."
                .to_string(),
        )
    } else {
        // Known external dependency: Mention the crate
        Some(format!(
            "External dependency from crate '{}'. Imported via USE statement.",
            crate_name
        ))
    };

    // Create minimal InterfaceSignature
    let interface_signature = InterfaceSignature {
        entity_type,
        name: item_name.to_string(),
        visibility: Visibility::Public, // External deps are public by definition
        file_path,
        line_range: LineRange::external(), // Use external() helper for 0-0 range
        module_path: vec![crate_name.to_string()],
        documentation,
        language_specific,
    };

    // Create CodeEntity (using CodeImplementation for external dependencies)
    let entity = CodeEntity::new(
        isgl1_key,
        interface_signature,
        EntityClass::CodeImplementation, // External dependencies are code, not tests
    )
    .map_err(|e| StreamerError::ParsingError {
        file: "external_dependency".to_string(),
        reason: format!("Failed to create CodeEntity: {}", e),
    })?;

    // Note: External dependencies have no code content (current_code/future_code remain None)
    // This is handled by TemporalState::initial() which sets current_ind=true, future_ind=false

    Ok(entity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_external_key_parts_validated() {
        // Arrange
        let key = "rust:module:Parser:external-dependency-clap:0-0";

        // Act
        let result = parse_external_key_parts_validated(key);

        // Assert
        assert!(result.is_ok(), "Should parse valid key: {:?}", result.err());
        let (language, entity_type, item_name, crate_name) = result.unwrap();

        assert_eq!(language, Language::Rust);
        assert_eq!(entity_type, "module");
        assert_eq!(item_name, "Parser");
        assert_eq!(crate_name, "clap");
    }

    #[test]
    fn test_parse_external_key_invalid_format() {
        // Arrange
        let key = "rust:module:Parser:0-0"; // Missing external-dependency marker

        // Act
        let result = parse_external_key_parts_validated(key);

        // Assert
        assert!(result.is_err(), "Should fail on invalid format");
    }

    #[test]
    fn test_create_placeholder_entity() {
        // Arrange
        let crate_name = "tokio";
        let item_name = "Runtime";
        let item_type = "struct";
        let language = Language::Rust;

        // Act
        let result = create_external_dependency_placeholder_entity_validated(
            crate_name, item_name, item_type, language,
        );

        // Assert
        assert!(
            result.is_ok(),
            "Should create placeholder: {:?}",
            result.err()
        );
        let entity = result.unwrap();

        // Verify ISGL1 key format
        assert_eq!(
            entity.isgl1_key,
            "rust:struct:Runtime:external-dependency-tokio:0-0"
        );

        // Verify line range is 0-0 (external marker)
        assert_eq!(entity.interface_signature.line_range.start, 0);
        assert_eq!(entity.interface_signature.line_range.end, 0);

        // Verify EntityClass
        assert_eq!(entity.entity_class, EntityClass::CodeImplementation);
    }

    #[test]
    fn test_extract_placeholders_deduplication() {
        // Arrange: Two edges pointing to same external dependency
        let edges = vec![
            DependencyEdge {
                from_key: Isgl1Key::new("rust:fn:main:src/main.rs:10-15").unwrap(),
                to_key: Isgl1Key::new("rust:module:Parser:external-dependency-clap:0-0").unwrap(),
                edge_type: EdgeType::Uses,
                source_location: None,
            },
            DependencyEdge {
                from_key: Isgl1Key::new("rust:fn:build_cli:src/cli.rs:5-10").unwrap(),
                to_key: Isgl1Key::new("rust:module:Parser:external-dependency-clap:0-0").unwrap(), // Same external dep
                edge_type: EdgeType::Uses,
                source_location: None,
            },
        ];

        // Act
        let placeholders = extract_placeholders_from_edges_deduplicated(&edges);

        // Assert: Should only create one placeholder (deduplicated)
        assert_eq!(placeholders.len(), 1, "Should deduplicate external dependencies");
        assert_eq!(
            placeholders[0].isgl1_key,
            "rust:module:Parser:external-dependency-clap:0-0"
        );
    }

    #[test]
    fn test_extract_placeholders_mixed_edges() {
        // Arrange: Mix of local and external dependencies
        let edges = vec![
            DependencyEdge {
                from_key: Isgl1Key::new("rust:fn:main:src/main.rs:10-15").unwrap(),
                to_key: Isgl1Key::new("rust:fn:helper:src/utils.rs:5-10").unwrap(), // Local dependency
                edge_type: EdgeType::Calls,
                source_location: None,
            },
            DependencyEdge {
                from_key: Isgl1Key::new("rust:fn:main:src/main.rs:12-13").unwrap(),
                to_key: Isgl1Key::new("rust:module:Parser:external-dependency-clap:0-0").unwrap(), // External
                edge_type: EdgeType::Uses,
                source_location: None,
            },
        ];

        // Act
        let placeholders = extract_placeholders_from_edges_deduplicated(&edges);

        // Assert: Should only extract external dependencies
        assert_eq!(placeholders.len(), 1, "Should only extract external dependencies");
        assert!(placeholders[0].isgl1_key.contains("external-dependency"));
    }

    /// RED TEST: Parse unknown pattern key
    ///
    /// **Phase 2 TDD Test**: Update parse_external_key_parts_validated()
    ///
    /// **Preconditions**:
    /// - Key contains `:unknown:0-0` instead of `:external-dependency-{crate}:0-0`
    ///
    /// **Expected Behavior**:
    /// - Successfully parse unknown pattern
    /// - Return crate_name = "unresolved-reference"
    ///
    /// **Postconditions**:
    /// - Parser handles both patterns uniformly
    ///
    /// **Current Status**: GREEN (passes after Phase 2 implementation)
    #[test]
    fn test_parse_unknown_pattern_key_validated() {
        // Arrange
        let key = "rust:fn:build_cli:unknown:0-0";

        // Act
        let result = parse_external_key_parts_validated(key);

        // Assert
        assert!(
            result.is_ok(),
            "Should parse unknown pattern key, got: {:?}",
            result.err()
        );

        let (language, entity_type, item_name, crate_name) = result.unwrap();

        assert_eq!(language, Language::Rust);
        assert_eq!(entity_type, "fn");
        assert_eq!(item_name, "build_cli");
        assert_eq!(
            crate_name, "unresolved-reference",
            "Unknown pattern should map to 'unresolved-reference' crate name"
        );
    }

    /// RED TEST: Differentiated documentation for unresolved references
    ///
    /// **Phase 3 TDD Test**: Update create_external_dependency_placeholder_entity_validated()
    ///
    /// **Preconditions**:
    /// - Creating placeholder for unresolved reference (crate_name="unresolved-reference")
    /// - Creating placeholder for known external dependency (crate_name="tokio")
    ///
    /// **Expected Behavior**:
    /// - Unresolved: "Unresolved reference - target location unknown..."
    /// - External: "External dependency from crate 'tokio'..."
    ///
    /// **Postconditions**:
    /// - Documentation clearly distinguishes the two cases
    ///
    /// **Current Status**: RED (will fail - same documentation for both)
    #[test]
    fn test_differentiated_documentation_unresolved_vs_external() {
        // Arrange & Act: Create unresolved reference placeholder
        let unresolved = create_external_dependency_placeholder_entity_validated(
            "unresolved-reference",
            "build_cli",
            "fn",
            Language::Rust,
        )
        .expect("Should create unresolved placeholder");

        // Arrange & Act: Create external dependency placeholder
        let external = create_external_dependency_placeholder_entity_validated(
            "tokio",
            "Runtime",
            "struct",
            Language::Rust,
        )
        .expect("Should create external placeholder");

        // Assert: Unresolved has specific documentation
        let unresolved_doc = unresolved
            .interface_signature
            .documentation
            .as_ref()
            .expect("Should have documentation");

        assert!(
            unresolved_doc.contains("Unresolved reference"),
            "Unresolved documentation should mention 'Unresolved reference', got: {}",
            unresolved_doc
        );
        assert!(
            unresolved_doc.contains("target location unknown"),
            "Should explain unknown location, got: {}",
            unresolved_doc
        );

        // Assert: External has crate-specific documentation
        let external_doc = external
            .interface_signature
            .documentation
            .as_ref()
            .expect("Should have documentation");

        assert!(
            external_doc.contains("External dependency"),
            "External documentation should mention 'External dependency', got: {}",
            external_doc
        );
        assert!(
            external_doc.contains("tokio"),
            "Should mention crate name 'tokio', got: {}",
            external_doc
        );

        // Assert: Different documentation for different cases
        assert_ne!(
            unresolved_doc, external_doc,
            "Documentation should differ between unresolved and external"
        );
    }
}
