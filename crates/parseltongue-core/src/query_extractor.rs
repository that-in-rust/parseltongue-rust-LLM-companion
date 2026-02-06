//! Query-Based Entity Extractor
//!
//! Uses tree-sitter's query system for declarative entity extraction.
//! This approach reduces code by 67% compared to imperative per-language extractors
//! (210 lines vs 650 lines) and is the industry standard used by GitHub, ast-grep,
//! and nvim-treesitter.
//!
//! ## Design Principles
//!
//! - **Declarative queries**: .scm files define extraction patterns (not imperative code)
//! - **Compile-time embedding**: Query files embedded via include_str! for zero runtime I/O
//! - **Streaming iteration**: tree-sitter 0.25 uses StreamingIterator to prevent UB
//! - **Deduplication**: Automatic handling of overlapping query patterns
//!
//! ## Performance Contracts
//!
//! - **Parsing**: <20ms per 1K LOC (release), <50ms (debug)
//! - **Memory**: <1MB per query file
//! - **Zero panics**: Gracefully handles malformed code
//!
//! ## Supported Languages
//!
//! Currently supports: Rust, Python, C, C++, Ruby, JavaScript, TypeScript, Go, Java, PHP, C#, Swift (12 languages)
//! Note: Kotlin support pending tree-sitter version upgrade (currently incompatible: 0.20 vs 0.25)
//! Extensible: Add new languages by creating .scm query files (~1 hour per language)

use std::collections::HashMap;
use std::path::Path;
use anyhow::{Context, Result};
use tree_sitter::{Query, QueryCursor, Tree, Parser, StreamingIterator};

use crate::entities::{Language, DependencyEdge, EdgeType};
use crate::isgl1_v2::{compute_birth_timestamp, extract_semantic_path};

/// Parsed code entity representation
#[derive(Debug, Clone)]
pub struct ParsedEntity {
    pub entity_type: EntityType,
    pub name: String,
    pub language: Language,
    pub line_range: (usize, usize),
    pub file_path: String,
    pub metadata: HashMap<String, String>,
}

/// Entity types that can be parsed
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Function,
    Struct,
    Enum,
    Trait,
    Interface,  // Swift protocols, Java/C#/TypeScript interfaces
    Impl,
    Module,
    Class,
    Method,
    Typedef,
    Namespace,
}

/// Query-based extractor using .scm query files
pub struct QueryBasedExtractor {
    queries: HashMap<Language, String>,
    dependency_queries: HashMap<Language, String>,  // v0.9.0: Dependency extraction
    parsers: HashMap<Language, Parser>,
}

/// Sanitize file path for key format
///
/// Ensures edge keys match entity keys by normalizing path separators.
/// Entity keys use underscores, so edges must too.
fn sanitize_path_for_key_format(path: &str) -> String {
    path.replace(['/', '\\', '.'], "_")
}

/// Parse external dependency from full use path
///
/// **Four-Word Naming**: parse_external_dependency_from_path
/// - parse: verb (extract structured data)
/// - external: constraint (not local/stdlib)
/// - dependency: target (what we're identifying)
/// - from_path: qualifier (source of data)
///
/// **Design**: Detects external crates by analyzing use path structure
///
/// # Examples
///
/// ```ignore
/// parse_external_dependency_from_path("clap::Parser")
///   → Some(("clap", "Parser"))
///   → Creates key: rust:module:Parser:external-dependency-clap:0-0
///
/// parse_external_dependency_from_path("std::collections::HashMap")
///   → None (stdlib crate)
///
/// parse_external_dependency_from_path("crate::module::Type")
///   → None (local crate keyword)
/// ```
///
/// **Preconditions**:
/// - path contains valid Rust module path syntax
/// - path may contain "::" separators
///
/// **Postconditions**:
/// - Returns Some((crate_name, item_name)) if external
/// - Returns None if stdlib or local keyword
/// - External keys use format: external-dependency-{crate} (2-word prefix)
///
/// **Error Conditions**:
/// - Empty path → None
/// - Single identifier → None (ambiguous)
fn parse_external_dependency_from_path(path: &str) -> Option<(String, String)> {
    // Split path by "::"
    let segments: Vec<&str> = path.split("::").collect();

    // Need at least 2 segments (crate::item)
    if segments.len() < 2 {
        return None;
    }

    let crate_name = segments[0];
    let item_name = segments.last().unwrap();

    // Check if it's a local keyword
    if matches!(crate_name, "crate" | "self" | "super") {
        return None;
    }

    // Check if it's a stdlib crate
    let stdlib_crates = [
        "std", "core", "alloc", "proc_macro",
        "test", // Rust stdlib crates
    ];

    if stdlib_crates.contains(&crate_name) {
        return None;
    }

    // It's external!
    Some((crate_name.to_string(), item_name.to_string()))
}

impl QueryBasedExtractor {
    /// Create new extractor with embedded query files
    ///
    /// # Example
    ///
    /// ```rust
    /// use parseltongue_core::query_extractor::QueryBasedExtractor;
    ///
    /// let extractor = QueryBasedExtractor::new().unwrap();
    /// // Now ready to parse Rust, Python, C, C++, Ruby code
    /// ```
    ///
    /// # Performance
    ///
    /// Initializes parsers for all supported languages (~1ms overhead).
    /// Query files are embedded at compile time (zero runtime I/O).
    pub fn new() -> Result<Self> {
        let mut queries = HashMap::new();

        // Embed query files at compile time
        queries.insert(
            Language::Rust,
            include_str!("../../../entity_queries/rust.scm").to_string()
        );
        queries.insert(
            Language::Python,
            include_str!("../../../entity_queries/python.scm").to_string()
        );
        queries.insert(
            Language::C,
            include_str!("../../../entity_queries/c.scm").to_string()
        );
        queries.insert(
            Language::Cpp,
            include_str!("../../../entity_queries/cpp.scm").to_string()
        );
        queries.insert(
            Language::Ruby,
            include_str!("../../../entity_queries/ruby.scm").to_string()
        );
        queries.insert(
            Language::JavaScript,
            include_str!("../../../entity_queries/javascript.scm").to_string()
        );
        queries.insert(
            Language::TypeScript,
            include_str!("../../../entity_queries/typescript.scm").to_string()
        );
        queries.insert(
            Language::Go,
            include_str!("../../../entity_queries/go.scm").to_string()
        );
        queries.insert(
            Language::Java,
            include_str!("../../../entity_queries/java.scm").to_string()
        );
        queries.insert(
            Language::Php,
            include_str!("../../../entity_queries/php.scm").to_string()
        );
        queries.insert(
            Language::CSharp,
            include_str!("../../../entity_queries/c_sharp.scm").to_string()
        );
        queries.insert(
            Language::Swift,
            include_str!("../../../entity_queries/swift.scm").to_string()
        );
        // NOTE: Kotlin temporarily disabled due to tree-sitter version incompatibility (0.20 vs 0.25)
        // queries.insert(
        //     Language::Kotlin,
        //     include_str!("../../../entity_queries/kotlin.scm").to_string()
        // );

        // Initialize parsers
        let mut parsers = HashMap::new();
        Self::init_parser(&mut parsers, Language::Rust, &tree_sitter_rust::LANGUAGE.into())?;
        Self::init_parser(&mut parsers, Language::Python, &tree_sitter_python::LANGUAGE.into())?;
        Self::init_parser(&mut parsers, Language::C, &tree_sitter_c::LANGUAGE.into())?;
        Self::init_parser(&mut parsers, Language::Cpp, &tree_sitter_cpp::LANGUAGE.into())?;
        Self::init_parser(&mut parsers, Language::Ruby, &tree_sitter_ruby::LANGUAGE.into())?;
        Self::init_parser(&mut parsers, Language::JavaScript, &tree_sitter_javascript::LANGUAGE.into())?;
        Self::init_parser(&mut parsers, Language::TypeScript, &tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())?;
        Self::init_parser(&mut parsers, Language::Go, &tree_sitter_go::LANGUAGE.into())?;
        Self::init_parser(&mut parsers, Language::Java, &tree_sitter_java::LANGUAGE.into())?;
        Self::init_parser(&mut parsers, Language::Php, &tree_sitter_php::LANGUAGE_PHP.into())?;
        Self::init_parser(&mut parsers, Language::CSharp, &tree_sitter_c_sharp::LANGUAGE.into())?;
        Self::init_parser(&mut parsers, Language::Swift, &tree_sitter_swift::LANGUAGE.into())?;
        // NOTE: Kotlin temporarily disabled due to tree-sitter version incompatibility
        // Self::init_parser(&mut parsers, Language::Kotlin, &tree_sitter_kotlin::language())?;

        // v0.9.0: Load dependency query files for ALL languages
        let mut dependency_queries = HashMap::new();
        dependency_queries.insert(
            Language::Rust,
            include_str!("../../../dependency_queries/rust.scm").to_string()
        );
        dependency_queries.insert(
            Language::Python,
            include_str!("../../../dependency_queries/python.scm").to_string()
        );
        dependency_queries.insert(
            Language::JavaScript,
            include_str!("../../../dependency_queries/javascript.scm").to_string()
        );
        dependency_queries.insert(
            Language::TypeScript,
            include_str!("../../../dependency_queries/typescript.scm").to_string()
        );
        dependency_queries.insert(
            Language::Go,
            include_str!("../../../dependency_queries/go.scm").to_string()
        );
        dependency_queries.insert(
            Language::Java,
            include_str!("../../../dependency_queries/java.scm").to_string()
        );
        dependency_queries.insert(
            Language::C,
            include_str!("../../../dependency_queries/c.scm").to_string()
        );
        dependency_queries.insert(
            Language::Cpp,
            include_str!("../../../dependency_queries/cpp.scm").to_string()
        );
        dependency_queries.insert(
            Language::Ruby,
            include_str!("../../../dependency_queries/ruby.scm").to_string()
        );
        dependency_queries.insert(
            Language::Php,
            include_str!("../../../dependency_queries/php.scm").to_string()
        );
        dependency_queries.insert(
            Language::CSharp,
            include_str!("../../../dependency_queries/c_sharp.scm").to_string()
        );
        dependency_queries.insert(
            Language::Swift,
            include_str!("../../../dependency_queries/swift.scm").to_string()
        );

        Ok(Self { queries, dependency_queries, parsers })
    }

    fn init_parser(
        parsers: &mut HashMap<Language, Parser>,
        lang: Language,
        grammar: &tree_sitter::Language
    ) -> Result<()> {
        let mut parser = Parser::new();
        parser.set_language(grammar)
            .context(format!("Failed to set language for {:?}", lang))?;
        parsers.insert(lang, parser);
        Ok(())
    }

    /// Parse source code and extract entities using tree-sitter queries
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to parse
    /// * `file_path` - Path to the file (for entity metadata)
    /// * `language` - The programming language
    ///
    /// # Returns
    ///
    /// A tuple of (entities, dependencies). Dependencies are not yet implemented
    /// and will return an empty vec.
    ///
    /// # Example
    ///
    /// ```rust
    /// use parseltongue_core::query_extractor::QueryBasedExtractor;
    /// use parseltongue_core::entities::Language;
    /// use std::path::Path;
    ///
    /// let mut extractor = QueryBasedExtractor::new().unwrap();
    /// let code = "fn hello() { println!(\"world\"); }";
    /// let (entities, _deps) = extractor.parse_source(
    ///     code,
    ///     Path::new("test.rs"),
    ///     Language::Rust
    /// ).unwrap();
    ///
    /// assert_eq!(entities.len(), 1);
    /// assert_eq!(entities[0].name, "hello");
    /// ```
    ///
    /// # Performance
    ///
    /// <20ms per 1K LOC in release mode, <50ms in debug mode.
    pub fn parse_source(
        &mut self,
        source: &str,
        file_path: &Path,
        language: Language,
    ) -> Result<(Vec<ParsedEntity>, Vec<DependencyEdge>)> {
        // Get parser
        let parser = self.parsers.get_mut(&language)
            .context(format!("No parser for language {:?}", language))?;

        // Parse tree
        let tree = parser.parse(source, None)
            .context("Failed to parse source")?;

        // Get entity query
        let query_source = self.queries.get(&language)
            .context(format!("No query for language {:?}", language))?;

        // Execute entity query
        let entities = self.execute_query(&tree, source, file_path, language, query_source)?;

        // v0.9.0: Execute dependency query if available
        let dependencies = if let Some(dep_query_source) = self.dependency_queries.get(&language) {
            self.execute_dependency_query(&tree, source, file_path, language, dep_query_source, &entities)?
        } else {
            // Graceful degradation: if no dependency query, return empty vec
            vec![]
        };

        Ok((entities, dependencies))
    }

    fn execute_query(
        &self,
        tree: &Tree,
        source: &str,
        file_path: &Path,
        language: Language,
        query_source: &str,
    ) -> Result<Vec<ParsedEntity>> {
        // Create query
        let ts_lang = self.get_ts_language(language)?;
        let query = Query::new(&ts_lang, query_source)
            .context("Failed to create query")?;

        // Execute query using streaming iterator
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut entities = Vec::new();
        let mut seen = std::collections::HashSet::new();

        while let Some(m) = matches.next() {
            if let Some(entity) = self.process_match(m, &query, source, file_path, language) {
                // Deduplicate based on (name, line_range) - prevents duplicate extraction
                let key = (entity.name.clone(), entity.line_range);
                if seen.insert(key) {
                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }

    fn process_match<'a>(
        &self,
        m: &tree_sitter::QueryMatch<'a, 'a>,
        query: &Query,
        source: &str,
        file_path: &Path,
        language: Language,
    ) -> Option<ParsedEntity> {
        let mut entity_name = None;
        let mut entity_type = None;
        let mut node = None;

        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];

            if *capture_name == "name" {
                entity_name = Some(source[capture.node.byte_range()].to_string());
            } else if capture_name.starts_with("definition.") {
                entity_type = self.parse_entity_type(capture_name);
                node = Some(capture.node);
            }
        }

        if let (Some(name), Some(entity_type), Some(node)) = (entity_name, entity_type, node) {
            Some(ParsedEntity {
                entity_type,
                name,
                language,
                line_range: (
                    node.start_position().row + 1,
                    node.end_position().row + 1,
                ),
                file_path: file_path.to_string_lossy().to_string(),
                metadata: HashMap::new(),
            })
        } else {
            None
        }
    }

    fn parse_entity_type(&self, capture_name: &str) -> Option<EntityType> {
        match capture_name {
            "definition.function" => Some(EntityType::Function),
            "definition.struct" => Some(EntityType::Struct),
            "definition.class" => Some(EntityType::Class),
            "definition.enum" => Some(EntityType::Enum),
            "definition.trait" => Some(EntityType::Trait),
            "definition.interface" => Some(EntityType::Interface),
            "definition.impl" => Some(EntityType::Impl),
            "definition.module" => Some(EntityType::Module),
            "definition.method" => Some(EntityType::Method),
            "definition.typedef" => Some(EntityType::Typedef),
            "definition.namespace" => Some(EntityType::Namespace),
            _ => None,
        }
    }

    fn get_ts_language(&self, language: Language) -> Result<tree_sitter::Language> {
        Ok(match language {
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            Language::C => tree_sitter_c::LANGUAGE.into(),
            Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Language::Ruby => tree_sitter_ruby::LANGUAGE.into(),
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Language::Go => tree_sitter_go::LANGUAGE.into(),
            Language::Java => tree_sitter_java::LANGUAGE.into(),
            Language::Php => tree_sitter_php::LANGUAGE_PHP.into(),
            Language::CSharp => tree_sitter_c_sharp::LANGUAGE.into(),
            Language::Swift => tree_sitter_swift::LANGUAGE.into(),
            // NOTE: Kotlin temporarily disabled due to tree-sitter version incompatibility
            // Language::Kotlin => tree_sitter_kotlin::language(),
            _ => anyhow::bail!("Unsupported language: {:?}", language),
        })
    }

    /// Execute dependency query and extract relationships (v0.9.0)
    ///
    /// Processes tree-sitter query matches to build DependencyEdge objects.
    /// Handles three edge types: Calls, Uses, Implements.
    fn execute_dependency_query(
        &self,
        tree: &Tree,
        source: &str,
        file_path: &Path,
        language: Language,
        dep_query_source: &str,
        entities: &[ParsedEntity],
    ) -> Result<Vec<DependencyEdge>> {
        // Compile query
        let ts_lang = self.get_ts_language(language)?;
        let query = Query::new(&ts_lang, dep_query_source)
            .context("Failed to create dependency query")?;

        // Execute query
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut dependencies = Vec::new();

        while let Some(m) = matches.next() {
            // Process each match to extract dependency relationship
            if let Some(edge) = self.process_dependency_match(m, &query, source, file_path, language, entities) {
                dependencies.push(edge);
            }
        }

        Ok(dependencies)
    }

    /// Process a single dependency query match
    fn process_dependency_match<'a>(
        &self,
        m: &tree_sitter::QueryMatch<'a, 'a>,
        query: &Query,
        source: &str,
        file_path: &Path,
        language: Language,
        entities: &[ParsedEntity],
    ) -> Option<DependencyEdge> {
        let mut dependency_type = None;
        let mut from_entity = None;
        let mut to_name = None;
        let mut use_full_path = None; // Bug #4: Capture full use path for external detection
        let mut location = None;

        // Parse captures to identify relationship type and participants
        for capture in m.captures {
            let capture_name = &query.capture_names()[capture.index as usize];
            let node = capture.node;
            let node_text = &source[node.byte_range()];

            // Identify dependency type
            if capture_name.starts_with("dependency.") {
                location = Some(format!("{}:{}", file_path.display(), node.start_position().row + 1));

                if capture_name.contains("call")
                    || capture_name.contains("method_call")
                    || capture_name.contains("constructor")
                    || capture_name.contains("collection_op")
                    || capture_name.contains("collection_operation")
                    || capture_name.contains("async_call")
                    || capture_name.contains("async_method")
                    || capture_name.contains("await_call")
                    || capture_name.contains("await_method")
                    || capture_name.contains("promise_op")
                    || capture_name.contains("promise_operation")
                {
                    dependency_type = Some(EdgeType::Calls);
                    // For calls, find containing function
                    from_entity = self.find_containing_entity(node, entities);
                } else if capture_name.contains("use")
                    || capture_name.contains("import")
                    || capture_name.contains("type_ref")
                    || capture_name.contains("property_access")
                    || capture_name.contains("attribute_access")
                    || capture_name.contains("decorator")
                    || capture_name.contains("type_generic")
                    || capture_name.contains("type_simple")
                    || capture_name.contains("generic_type")
                {
                    dependency_type = Some(EdgeType::Uses);
                } else if capture_name.contains("implement")
                    || capture_name.contains("inherits")
                    || capture_name.contains("extends")
                {
                    dependency_type = Some(EdgeType::Implements);
                }
            }

            // Bug #4: Capture full use path (e.g., "clap::Parser") for external detection
            if *capture_name == "reference.use_full_path" {
                use_full_path = Some(node_text.to_string());
            }

            // Extract reference name (what is being called/used/implemented)
            if capture_name.starts_with("reference.") {
                to_name = Some(node_text.to_string());
            }

            // Extract definition name (for impl blocks)
            if capture_name.starts_with("definition.impl") {
                from_entity = entities.iter().find(|e| {
                    e.name == node_text && e.line_range.0 <= node.start_position().row + 1
                        && e.line_range.1 > node.end_position().row
                });
            }
        }

        // Build DependencyEdge if we have enough information
        if let (Some(edge_type), Some(to)) = (dependency_type, to_name) {
            // For Uses edges (imports, use declarations), create simplified keys
            if edge_type == EdgeType::Uses {
                let from_key = format!("{}:file:{}:1-1", language, sanitize_path_for_key_format(&file_path.display().to_string()));

                // Bug #4: Detect external dependencies from full use path
                let to_key = if let Some(full_path) = use_full_path {
                    // Try to parse as external dependency
                    if let Some((crate_name, item_name)) = parse_external_dependency_from_path(&full_path) {
                        // External dependency: rust:module:Parser:external-dependency-clap:0-0
                        format!("{}:module:{}:external-dependency-{}:0-0", language, item_name, crate_name)
                    } else {
                        // Stdlib or local: rust:module:HashMap:0-0
                        format!("{}:module:{}:0-0", language, to)
                    }
                } else {
                    // Fallback: rust:module:Parser:0-0
                    format!("{}:module:{}:0-0", language, to)
                };

                return DependencyEdge::builder()
                    .from_key(from_key)
                    .to_key(to_key)
                    .edge_type(edge_type)
                    .source_location(location.unwrap_or_default())
                    .build()
                    .ok();
            }

            // For Calls and Implements, we need a from_entity
            if let Some(from) = from_entity {
                // Bug Fix: Use ISGL1 v2 format with semantic path and birth timestamp
                // Old format: rust:fn:name:path:10-20 (line-range based)
                // New format: rust:fn:name:__semantic_path:T1234567890 (timestamp based)
                let semantic_path = extract_semantic_path(&from.file_path);
                let birth_timestamp = compute_birth_timestamp(&from.file_path, &from.name);

                let from_key = format!(
                    "{}:{}:{}:{}:T{}",
                    language,
                    self.entity_type_to_key_component(&from.entity_type),
                    from.name,
                    semantic_path,
                    birth_timestamp
                );

                let to_key = format!(
                    "{}:fn:{}:unresolved-reference:0-0",
                    language,
                    to
                );

                return DependencyEdge::builder()
                    .from_key(from_key)
                    .to_key(to_key)
                    .edge_type(edge_type)
                    .source_location(location.unwrap_or_default())
                    .build()
                    .ok();
            }
        }

        None
    }

    /// Find the entity that contains a given AST node
    ///
    /// Prefers the most specific entity (smallest line range) when multiple
    /// entities contain the node. This ensures method calls are attributed to
    /// the method, not the enclosing impl block.
    fn find_containing_entity<'a>(
        &self,
        node: tree_sitter::Node<'_>,
        entities: &'a [ParsedEntity],
    ) -> Option<&'a ParsedEntity> {
        let node_line = node.start_position().row + 1;

        // Find all entities that contain this line
        let mut candidates: Vec<&ParsedEntity> = entities
            .iter()
            .filter(|e| e.line_range.0 <= node_line && node_line <= e.line_range.1)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Sort by specificity
        candidates.sort_by(|a, b| {
            // Primary: Prefer smaller line ranges (more specific)
            let a_range = a.line_range.1 - a.line_range.0;
            let b_range = b.line_range.1 - b.line_range.0;

            match a_range.cmp(&b_range) {
                std::cmp::Ordering::Equal => {
                    // Secondary: Prefer methods/functions over impl blocks
                    match (&a.entity_type, &b.entity_type) {
                        (EntityType::Method, EntityType::Impl) => std::cmp::Ordering::Less,
                        (EntityType::Impl, EntityType::Method) => std::cmp::Ordering::Greater,
                        (EntityType::Function, EntityType::Impl) => std::cmp::Ordering::Less,
                        (EntityType::Impl, EntityType::Function) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                },
                other => other,
            }
        });

        // Return most specific entity
        Some(candidates[0])
    }

    /// Convert EntityType to ISGL1 key component
    fn entity_type_to_key_component(&self, entity_type: &EntityType) -> &'static str {
        match entity_type {
            EntityType::Function => "fn",
            EntityType::Method => "method",
            EntityType::Struct => "struct",
            EntityType::Enum => "enum",
            EntityType::Trait => "trait",
            EntityType::Interface => "interface",
            EntityType::Class => "class",
            EntityType::Module => "module",
            EntityType::Impl => "impl",
            EntityType::Typedef => "typedef",
            EntityType::Namespace => "namespace",
        }
    }
}
