//! CozoDB storage client implementation.
//!
//! Real database implementation following the ultra-minimalist architecture
//! and TDD-first principles. No mocks, no placeholders - this is the real deal.

use crate::entities::*;
use crate::error::{ParseltongError, Result};
use crate::interfaces::*;
use async_trait::async_trait;
use cozo::{DataValue, DbInstance, ScriptMutability};
use std::collections::BTreeMap;

/// Escape string for safe use in CozoDB query strings
///
/// Escapes backslashes and single quotes to prevent CozoDB query parsing errors.
/// Critical for Windows file paths, PHP namespaces, and strings containing quotes.
///
/// # Escaping Order (CRITICAL)
/// Must escape backslash BEFORE single quote to avoid double-escaping:
/// 1. `\` → `\\` (escape backslashes first)
/// 2. `'` → `\'` (then escape quotes)
///
/// # Examples
/// ```
/// use parseltongue_core::storage::cozo_client::escape_for_cozo_string;
/// assert_eq!(escape_for_cozo_string(r"C:\Users"), r"C:\\Users");
/// assert_eq!(escape_for_cozo_string("User's"), r"User\'s");
/// assert_eq!(escape_for_cozo_string(r"C:\User's\Path"), r"C:\\User\'s\\Path");
/// ```
pub fn escape_for_cozo_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Write tuned RocksDB options file to prevent Windows 75MB write stall
///
/// Creates an options file in the RocksDB database directory with tuned settings
/// that prevent write stalls on Windows when ingesting large codebases (>75MB).
/// The tuning increases buffer sizes and background jobs to handle burst writes.
///
/// If the options file already exists, this function does nothing (preserves user customizations).
/// If writing fails, the error is ignored - the database will still work with defaults.
///
/// # Arguments
/// * `path` - RocksDB database directory path (e.g., "parseltongue20251201/analysis.db")
fn write_rocksdb_options_file_tuned(path: &str) {
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    let options_path = Path::new(path).join("options");

    // Don't overwrite existing options file (preserve user customizations)
    if options_path.exists() {
        return;
    }

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(path) {
        eprintln!("Warning: failed to create RocksDB directory {}: {}", path, e);
        return;
    }

    // Write tuned OPTIONS file
    let options_content = r#"[DBOptions]
max_background_jobs=4
create_if_missing=true

[CFOptions "default"]
write_buffer_size=134217728
max_write_buffer_number=4
level0_slowdown_writes_trigger=40
level0_stop_writes_trigger=56
target_file_size_base=67108864
max_bytes_for_level_base=268435456
"#;

    if let Err(e) = fs::File::create(&options_path)
        .and_then(|mut f| f.write_all(options_content.as_bytes()))
    {
        eprintln!("Warning: failed to write RocksDB options file: {}", e);
        // Continue anyway - database will use defaults
    }
}

/// CozoDB storage client
///
/// Provides real database storage with SQLite backend, supporting:
/// - Temporal versioning (current_ind, future_ind, future_action)
/// - ISGL1 key-based entity storage
/// - Full CodeGraph schema from technical specifications
pub struct CozoDbStorage {
    db: DbInstance,
}

impl CozoDbStorage {
    /// Create new CozoDB storage instance
    ///
    /// # Arguments
    /// * `engine_spec` - Storage engine specification:
    ///   - "mem" for in-memory
    ///   - "rocksdb:path/to/db" for RocksDB persistent storage (recommended, fastest)
    ///   - "sled:path/to/db" for Sled persistent storage (v1.5.4: slower than RocksDB, uses more disk)
    ///   - "sqlite:path/to/db.sqlite" for SQLite storage
    ///
    /// # Performance Notes (v1.5.4)
    /// - RocksDB: Fastest for Cozo's workload, recommended for production
    /// - Sled: Pure Rust, simpler deployment, but 2-3x slower and higher disk usage
    /// - Use Sled only if deployment constraints require pure Rust stack
    ///
    /// # Examples
    /// ```ignore
    /// let db = CozoDbStorage::new("mem").await?;
    /// let db = CozoDbStorage::new("rocksdb:./parseltongue.db").await?;
    /// let db = CozoDbStorage::new("sled:./parseltongue.db").await?;
    /// let db = CozoDbStorage::new("sqlite:./parseltongue.sqlite").await?;
    /// ```
    pub async fn new(engine_spec: &str) -> Result<Self> {
        // Parse engine specification: "engine:path" or just "engine" (for mem)
        let (engine, path) = if engine_spec.contains(':') {
            let parts: Vec<&str> = engine_spec.splitn(2, ':').collect();
            (parts[0], parts[1])
        } else {
            (engine_spec, "")
        };

        // Write tuned RocksDB options file if using rocksdb engine (fixes Windows 75MB write stall)
        if engine == "rocksdb" && !path.is_empty() {
            write_rocksdb_options_file_tuned(path);
        }

        let db = DbInstance::new(engine, path, Default::default())
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "connection".to_string(),
                details: format!("Failed to create CozoDB instance with engine '{}' and path '{}': {}", engine, path, e),
            })?;

        Ok(Self { db })
    }

    /// Check if database connection is alive
    pub async fn is_connected(&self) -> bool {
        // Test query to verify connection - use ::relations which always works
        self.db
            .run_script("::relations", Default::default(), ScriptMutability::Immutable)
            .is_ok()
    }

    /// Create CodeGraph schema
    ///
    /// Implements schema from 01-cozodb-schema.md specification
    /// v0.9.0 Enhancement: Added entity_class column for test/code separation
    /// v1.5.0 ISGL1 v2: Added birth_timestamp, content_hash, semantic_path
    pub async fn create_schema(&self) -> Result<()> {
        let schema = r#"
            :create CodeGraph {
                ISGL1_key: String =>
                Current_Code: String?,
                Future_Code: String?,
                interface_signature: String,
                TDD_Classification: String,
                lsp_meta_data: String?,
                current_ind: Bool,
                future_ind: Bool,
                Future_Action: String?,
                file_path: String,
                language: String,
                last_modified: String,
                entity_type: String,
                entity_class: String,
                birth_timestamp: Int?,
                content_hash: String?,
                semantic_path: String?,
                root_subfolder_L1: String,
                root_subfolder_L2: String
            }
        "#;

        self.db
            .run_script(schema, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "schema_creation".to_string(),
                details: format!("Failed to create schema: {}", e),
            })?;

        // v1.6.5: Create diagnostic relations
        self.create_test_entities_excluded_schema().await?;
        self.create_file_word_coverage_schema().await?;
        self.create_ignored_files_schema().await?;

        Ok(())
    }

    /// Create DependencyEdges schema for code dependency graph
    ///
    /// Implements dependency tracking with composite key (from_key, to_key, edge_type).
    /// Indices automatically created on key fields for O(log n) query performance.
    ///
    /// # Schema
    /// - **Keys**: from_key, to_key, edge_type (composite key for uniqueness)
    /// - **Fields**: source_location (optional line/column info)
    ///
    /// # Performance Contracts
    /// - Single insert: <5ms (D10 specification)
    /// - Batch insert (100 edges): <50ms (D10 specification)
    ///
    /// # Example
    /// ```ignore
    /// let storage = CozoDbStorage::new("mem").await?;
    /// storage.create_dependency_edges_schema().await?;
    /// // Now ready to insert edges
    /// ```
    pub async fn create_dependency_edges_schema(&self) -> Result<()> {
        let schema = r#"
            :create DependencyEdges {
                from_key: String,
                to_key: String,
                edge_type: String
                =>
                source_location: String?
            }
        "#;

        self.db
            .run_script(schema, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "create_dependency_edges_schema".to_string(),
                reason: format!("Failed to create DependencyEdges schema: {}", e),
            })?;

        Ok(())
    }

    /// Create TestEntitiesExcluded schema for tracking excluded test entities (v1.6.5).
    ///
    /// Stores test entities that were intentionally excluded from the CodeGraph
    /// to optimize LLM context. Enables agents to know what was filtered out
    /// during ingestion.
    ///
    /// # Schema
    /// - **Keys**: entity_name, folder_path, filename (composite key)
    /// - **Fields**: entity_class, language, line_start, line_end, detection_reason
    ///
    /// # 4-Word Name: create_test_entities_excluded_schema
    pub async fn create_test_entities_excluded_schema(&self) -> Result<()> {
        let schema = r#"
            :create TestEntitiesExcluded {
                entity_name: String,
                folder_path: String,
                filename: String
                =>
                entity_class: String,
                language: String,
                line_start: Int,
                line_end: Int,
                detection_reason: String
            }
        "#;

        self.db
            .run_script(schema, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "create_test_entities_excluded_schema".to_string(),
                details: format!("Failed to create TestEntitiesExcluded schema: {}", e),
            })?;

        Ok(())
    }

    /// Create FileWordCoverage schema for tracking word count coverage metrics (v1.6.5).
    ///
    /// Stores per-file word count comparison between source code and extracted
    /// entities. Enables dual coverage metrics: raw (total extraction) vs
    /// effective (meaningful code only, excluding imports/comments).
    ///
    /// # Schema
    /// - **Keys**: folder_path, filename (composite key)
    /// - **Fields**: language, source_word_count, entity_word_count, import_word_count,
    ///   comment_word_count, raw_coverage_pct, effective_coverage_pct, entity_count
    ///
    /// # 4-Word Name: create_file_word_coverage_schema
    pub async fn create_file_word_coverage_schema(&self) -> Result<()> {
        let schema = r#"
            :create FileWordCoverage {
                folder_path: String,
                filename: String
                =>
                language: String,
                source_word_count: Int,
                entity_word_count: Int,
                import_word_count: Int,
                comment_word_count: Int,
                raw_coverage_pct: Float,
                effective_coverage_pct: Float,
                entity_count: Int
            }
        "#;

        self.db
            .run_script(schema, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "create_file_word_coverage_schema".to_string(),
                details: format!("Failed to create FileWordCoverage schema: {}", e),
            })?;

        Ok(())
    }

    /// Create IgnoredFiles schema for tracking files skipped during ingestion (v1.6.5 Wave 1).
    ///
    /// Stores files that were ignored during ingestion due to missing language
    /// parsers or exclusion patterns. Enables diagnostics endpoint to report
    /// what was not analyzed.
    ///
    /// # Schema
    /// - **Keys**: folder_path, filename (composite key)
    /// - **Fields**: extension, reason
    ///
    /// # 4-Word Name: create_ignored_files_schema
    pub async fn create_ignored_files_schema(&self) -> Result<()> {
        let schema = r#"
            :create IgnoredFiles {
                folder_path: String,
                filename: String
                =>
                extension: String,
                reason: String
            }
        "#;

        self.db
            .run_script(schema, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "create_ignored_files_schema".to_string(),
                details: format!("Failed to create IgnoredFiles schema: {}", e),
            })?;

        Ok(())
    }

    /// Insert a single dependency edge
    ///
    /// # Performance Contract
    /// - Single insert: <5ms (D10 specification)
    ///
    /// # Example
    /// ```ignore
    /// use parseltongue_core::entities::{DependencyEdge, EdgeType};
    ///
    /// let edge = DependencyEdge::builder()
    ///     .from_key("rust:fn:main:src_main_rs:1-10")
    ///     .to_key("rust:fn:helper:src_helper_rs:5-20")
    ///     .edge_type(EdgeType::Calls)
    ///     .build()?;
    ///
    /// storage.insert_edge(&edge).await?;
    /// ```
    pub async fn insert_edge(&self, edge: &DependencyEdge) -> Result<()> {
        let query = r#"
            ?[from_key, to_key, edge_type, source_location] <-
            [[$from_key, $to_key, $edge_type, $source_location]]

            :put DependencyEdges {
                from_key, to_key, edge_type =>
                source_location
            }
        "#;

        let mut params = BTreeMap::new();
        params.insert("from_key".to_string(), DataValue::Str(edge.from_key.as_ref().into()));
        params.insert("to_key".to_string(), DataValue::Str(edge.to_key.as_ref().into()));
        params.insert("edge_type".to_string(), DataValue::Str(edge.edge_type.as_str().into()));
        params.insert(
            "source_location".to_string(),
            edge.source_location
                .as_ref()
                .map(|s| DataValue::Str(s.as_str().into()))
                .unwrap_or(DataValue::Null),
        );

        self.db
            .run_script(query, params, ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "insert_edge".to_string(),
                reason: format!("Failed to insert dependency edge: {}", e),
            })?;

        Ok(())
    }

    /// Insert multiple dependency edges in a batch
    ///
    /// # Performance Contract
    /// - Batch insert (100 edges): <50ms (D10 specification)
    ///
    /// # Example
    /// ```ignore
    /// let edges = vec![
    ///     DependencyEdge::builder()
    ///         .from_key("A").to_key("B").edge_type(EdgeType::Calls).build()?,
    ///     DependencyEdge::builder()
    ///         .from_key("B").to_key("C").edge_type(EdgeType::Uses).build()?,
    /// ];
    /// storage.insert_edges_batch(&edges).await?;
    /// ```
    pub async fn insert_edges_batch(&self, edges: &[DependencyEdge]) -> Result<()> {
        if edges.is_empty() {
            return Ok(());
        }

        // Build query with inline data for batch insert
        let query = format!(
            r#"
            ?[from_key, to_key, edge_type, source_location] <- [{}]

            :put DependencyEdges {{
                from_key, to_key, edge_type =>
                source_location
            }}
            "#,
            edges
                .iter()
                .map(|edge| {
                    let source_loc = edge
                        .source_location
                        .as_ref()
                        .map(|s| format!("'{}'", escape_for_cozo_string(s)))
                        .unwrap_or_else(|| "null".to_string());

                    format!(
                        "['{}', '{}', '{}', {}]",
                        escape_for_cozo_string(edge.from_key.as_ref()),
                        escape_for_cozo_string(edge.to_key.as_ref()),
                        edge.edge_type.as_str(),
                        source_loc
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        );

        self.db
            .run_script(&query, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "insert_edges_batch".to_string(),
                reason: format!("Failed to batch insert {} edges: {}", edges.len(), e),
            })?;

        Ok(())
    }

    /// Calculate blast radius: Find all entities within N hops of a changed entity.
    ///
    /// Uses CozoDB recursive Datalog queries to perform bounded BFS graph traversal,
    /// returning all reachable entities with their minimum distance from the source.
    ///
    /// # Performance Contract
    /// - 5 hops on 10k node graph: <50ms (D10 PRD requirement)
    /// - Bounded traversal prevents runaway queries on cyclic graphs
    ///
    /// # Arguments
    /// * `changed_key` - ISGL1 key of the entity that changed (source node)
    /// * `max_hops` - Maximum number of hops to traverse (1-based distance limit)
    ///
    /// # Returns
    /// Vector of (ISGL1_key, distance) tuples sorted by distance.
    /// Returns empty vector if `max_hops == 0`.
    ///
    /// # Algorithm
    /// 1. **Base case**: Direct dependents at distance 1
    /// 2. **Recursive case**: Follow edges incrementing distance up to `max_hops`
    /// 3. **Aggregation**: Min distance per node (handles diamond/multi-path dependencies)
    ///
    /// # Example
    /// ```
    /// use parseltongue_core::storage::CozoDbStorage;
    /// use parseltongue_core::entities::{DependencyEdge, EdgeType};
    ///
    /// # tokio_test::block_on(async {
    /// let storage = CozoDbStorage::new("mem").await.unwrap();
    /// storage.create_dependency_edges_schema().await.unwrap();
    ///
    /// // Given: A -> B -> C -> D
    /// let ab = DependencyEdge::builder()
    ///     .from_key("rust:fn:a:test_rs:1-5")
    ///     .to_key("rust:fn:b:test_rs:6-10")
    ///     .edge_type(EdgeType::Calls)
    ///     .build().unwrap();
    /// let bc = DependencyEdge::builder()
    ///     .from_key("rust:fn:b:test_rs:6-10")
    ///     .to_key("rust:fn:c:test_rs:11-15")
    ///     .edge_type(EdgeType::Calls)
    ///     .build().unwrap();
    /// storage.insert_edge(&ab).await.unwrap();
    /// storage.insert_edge(&bc).await.unwrap();
    ///
    /// // Query: blast_radius("A", 2) returns B and C
    /// let affected = storage.calculate_blast_radius(
    ///     "rust:fn:a:test_rs:1-5",
    ///     2
    /// ).await.unwrap();
    ///
    /// assert_eq!(affected.len(), 2);
    /// assert_eq!(affected[0].0, "rust:fn:b:test_rs:6-10");
    /// assert_eq!(affected[0].1, 1);
    /// assert_eq!(affected[1].0, "rust:fn:c:test_rs:11-15");
    /// assert_eq!(affected[1].1, 2);
    /// # });
    /// ```
    pub async fn calculate_blast_radius(
        &self,
        changed_key: &str,
        max_hops: usize,
    ) -> Result<Vec<(String, usize)>> {
        // Validation
        if max_hops == 0 {
            return Ok(Vec::new());
        }

        // CozoDB recursive query for bounded BFS
        // Strategy: Iteratively hop through edges, tracking minimum distance
        let query = r#"
            # Recursive blast radius query
            # Find all nodes reachable from start_node within max_hops

            # Base case: Starting node at distance 0
            reachable[to_key, distance] := *DependencyEdges{from_key, to_key},
                                            from_key == $start_key,
                                            distance = 1

            # Recursive case: Follow edges, incrementing distance
            reachable[to_key, new_distance] := reachable[from, dist],
                                                *DependencyEdges{from_key: from, to_key},
                                                dist < $max_hops,
                                                new_distance = dist + 1

            # Aggregate to get minimum distance for each node
            ?[node, min_dist] := reachable[node, dist],
                                 min_dist = min(dist)

            :order min_dist
            "#.to_string();

        let mut params = BTreeMap::new();
        params.insert("start_key".to_string(), DataValue::Str(changed_key.into()));
        params.insert("max_hops".to_string(), DataValue::from(max_hops as i64));

        let result = self
            .db
            .run_script(&query, params, ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "calculate_blast_radius".to_string(),
                reason: format!("Failed to execute blast radius query: {}", e),
            })?;

        // Parse results into (key, distance) tuples
        let mut affected = Vec::new();
        for row in result.rows {
            if row.len() >= 2 {
                if let (Some(DataValue::Str(node)), Some(distance_val)) =
                    (row.first(), row.get(1))
                {
                    // Distance is stored as Num enum (Int or Float)
                    let distance = match distance_val {
                        DataValue::Num(n) => match n {
                            cozo::Num::Int(i) => *i as usize,
                            cozo::Num::Float(f) => *f as usize,
                        },
                        _ => continue,
                    };
                    affected.push((node.to_string(), distance));
                }
            }
        }

        Ok(affected)
    }

    /// Get forward dependencies: entities that this entity directly depends on (outgoing edges).
    ///
    /// Returns all entities reachable in exactly 1 hop following outgoing edges from this entity.
    /// This is a simple 1-hop query useful for understanding what a function/module directly uses.
    ///
    /// # Arguments
    /// * `isgl1_key` - ISGL1 key of the entity to query
    ///
    /// # Returns
    /// Vector of ISGL1 keys that this entity depends on. Returns empty vector if no dependencies exist.
    ///
    /// # Example
    /// ```
    /// use parseltongue_core::storage::CozoDbStorage;
    /// use parseltongue_core::entities::{DependencyEdge, EdgeType};
    ///
    /// # tokio_test::block_on(async {
    /// let storage = CozoDbStorage::new("mem").await.unwrap();
    /// storage.create_dependency_edges_schema().await.unwrap();
    ///
    /// // Create: A calls B and C
    /// let edges = vec![
    ///     DependencyEdge::builder()
    ///         .from_key("rust:fn:A:test_rs:1-5")
    ///         .to_key("rust:fn:B:test_rs:10-15")
    ///         .edge_type(EdgeType::Calls)
    ///         .build().unwrap(),
    ///     DependencyEdge::builder()
    ///         .from_key("rust:fn:A:test_rs:1-5")
    ///         .to_key("rust:fn:C:test_rs:20-25")
    ///         .edge_type(EdgeType::Calls)
    ///         .build().unwrap(),
    /// ];
    /// storage.insert_edges_batch(&edges).await.unwrap();
    ///
    /// // Query: What does A depend on?
    /// let deps = storage.get_forward_dependencies("rust:fn:A:test_rs:1-5").await.unwrap();
    /// assert_eq!(deps.len(), 2); // A depends on B and C
    /// assert!(deps.contains(&"rust:fn:B:test_rs:10-15".to_string()));
    /// assert!(deps.contains(&"rust:fn:C:test_rs:20-25".to_string()));
    /// # });
    /// ```
    ///
    /// # See Also
    /// - [`get_reverse_dependencies`] for finding what depends on this entity
    /// - [`calculate_blast_radius`] for multi-hop impact analysis
    pub async fn get_forward_dependencies(&self, isgl1_key: &str) -> Result<Vec<String>> {
        let query = "?[to_key] := *DependencyEdges{from_key, to_key}, from_key == $key";

        let mut params = BTreeMap::new();
        params.insert("key".to_string(), DataValue::Str(isgl1_key.into()));

        let result = self
            .db
            .run_script(query, params, ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "get_forward_dependencies".to_string(),
                reason: format!("Failed to query forward dependencies: {}", e),
            })?;

        // Extract to_key values from results
        let mut dependencies = Vec::new();
        for row in result.rows {
            if let Some(DataValue::Str(key)) = row.first() {
                dependencies.push(key.to_string());
            }
        }

        Ok(dependencies)
    }

    /// Get reverse dependencies: entities that directly depend on this entity (incoming edges).
    ///
    /// Returns all entities that have outgoing edges pointing to this entity.
    /// This is a simple 1-hop query useful for finding "who calls this function".
    ///
    /// # Arguments
    /// * `isgl1_key` - ISGL1 key of the entity to query
    ///
    /// # Returns
    /// Vector of ISGL1 keys that depend on this entity. Returns empty vector if no dependents exist.
    ///
    /// # Example
    /// ```
    /// use parseltongue_core::storage::CozoDbStorage;
    /// use parseltongue_core::entities::{DependencyEdge, EdgeType};
    ///
    /// # tokio_test::block_on(async {
    /// let storage = CozoDbStorage::new("mem").await.unwrap();
    /// storage.create_dependency_edges_schema().await.unwrap();
    ///
    /// // Create: A and B both call C
    /// let edges = vec![
    ///     DependencyEdge::builder()
    ///         .from_key("rust:fn:A:test_rs:1-5")
    ///         .to_key("rust:fn:C:test_rs:20-25")
    ///         .edge_type(EdgeType::Calls)
    ///         .build().unwrap(),
    ///     DependencyEdge::builder()
    ///         .from_key("rust:fn:B:test_rs:10-15")
    ///         .to_key("rust:fn:C:test_rs:20-25")
    ///         .edge_type(EdgeType::Calls)
    ///         .build().unwrap(),
    /// ];
    /// storage.insert_edges_batch(&edges).await.unwrap();
    ///
    /// // Query: Who depends on C?
    /// let dependents = storage.get_reverse_dependencies("rust:fn:C:test_rs:20-25").await.unwrap();
    /// assert_eq!(dependents.len(), 2); // A and B both call C
    /// assert!(dependents.contains(&"rust:fn:A:test_rs:1-5".to_string()));
    /// assert!(dependents.contains(&"rust:fn:B:test_rs:10-15".to_string()));
    /// # });
    /// ```
    ///
    /// # See Also
    /// - [`get_forward_dependencies`] for finding what this entity depends on
    /// - [`calculate_blast_radius`] for multi-hop impact analysis
    pub async fn get_reverse_dependencies(&self, isgl1_key: &str) -> Result<Vec<String>> {
        let query = "?[from_key] := *DependencyEdges{from_key, to_key}, to_key == $key";

        let mut params = BTreeMap::new();
        params.insert("key".to_string(), DataValue::Str(isgl1_key.into()));

        let result = self
            .db
            .run_script(query, params, ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "get_reverse_dependencies".to_string(),
                reason: format!("Failed to query reverse dependencies: {}", e),
            })?;

        // Extract from_key values from results
        let mut dependents = Vec::new();
        for row in result.rows {
            if let Some(DataValue::Str(key)) = row.first() {
                dependents.push(key.to_string());
            }
        }

        Ok(dependents)
    }

    /// Get all dependency edges from the database.
    ///
    /// Returns all dependency edges stored in the DependencyEdges table.
    /// This is useful for test validation and full graph analysis.
    ///
    /// # Returns
    /// Vector of all DependencyEdge records. Returns empty vector if no edges exist.
    ///
    /// # Example
    /// ```
    /// use parseltongue_core::storage::CozoDbStorage;
    /// use parseltongue_core::entities::{DependencyEdge, EdgeType};
    ///
    /// # tokio_test::block_on(async {
    /// let storage = CozoDbStorage::new("mem").await.unwrap();
    /// storage.create_dependency_edges_schema().await.unwrap();
    ///
    /// // Insert edges
    /// let edges = vec![
    ///     DependencyEdge::builder()
    ///         .from_key("rust:fn:A:test_rs:1-5")
    ///         .to_key("rust:fn:B:test_rs:10-15")
    ///         .edge_type(EdgeType::Calls)
    ///         .build().unwrap(),
    /// ];
    /// storage.insert_edges_batch(&edges).await.unwrap();
    ///
    /// // Query all edges
    /// let all_deps = storage.get_all_dependencies().await.unwrap();
    /// assert_eq!(all_deps.len(), 1);
    /// # });
    /// ```
    pub async fn get_all_dependencies(&self) -> Result<Vec<DependencyEdge>> {
        let query = "?[from_key, to_key, edge_type, source_location] := *DependencyEdges{from_key, to_key, edge_type, source_location}";

        let result = self
            .db
            .run_script(query, BTreeMap::new(), ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "get_all_dependencies".to_string(),
                reason: format!("Failed to query all dependencies: {}", e),
            })?;

        // Parse results into DependencyEdge structs
        let mut dependencies = Vec::new();
        for row in result.rows {
            if row.len() >= 3 {
                if let (Some(DataValue::Str(from_key)), Some(DataValue::Str(to_key)), Some(DataValue::Str(edge_type_str))) =
                    (row.first(), row.get(1), row.get(2))
                {
                    let edge_type = match edge_type_str.as_str() {
                        "Calls" => EdgeType::Calls,
                        "Uses" => EdgeType::Uses,
                        "Implements" => EdgeType::Implements,
                        _ => continue, // Skip unknown edge types
                    };

                    let source_location = row.get(3).and_then(|v| {
                        if let DataValue::Str(loc) = v {
                            Some(loc.to_string())
                        } else {
                            None
                        }
                    });

                    let edge = DependencyEdge::builder()
                        .from_key(from_key.to_string())
                        .to_key(to_key.to_string())
                        .edge_type(edge_type)
                        .source_location(source_location.unwrap_or_default())
                        .build()
                        .map_err(|e| ParseltongError::DependencyError {
                            operation: "get_all_dependencies".to_string(),
                            reason: format!("Failed to build DependencyEdge: {}", e),
                        })?;

                    dependencies.push(edge);
                }
            }
        }

        Ok(dependencies)
    }

    /// Get transitive closure: all entities reachable from this entity (unbounded).
    ///
    /// Returns ALL entities reachable by recursively following dependency edges,
    /// without any hop limit. Uses CozoDB's recursive Datalog for efficient graph traversal.
    /// Automatically handles cycles without infinite loops.
    ///
    /// # Use Cases
    /// - Full impact analysis: "If I change this function, what ALL code might be affected?"
    /// - Dependency tree extraction for LLM context
    /// - Reachability analysis for refactoring safety
    ///
    /// # Arguments
    /// * `isgl1_key` - ISGL1 key of the starting entity
    ///
    /// # Returns
    /// Vector of all reachable ISGL1 keys. May include the starting node if it's part of a cycle.
    /// Returns empty vector if no outgoing edges exist.
    ///
    /// # Example
    /// ```
    /// use parseltongue_core::storage::CozoDbStorage;
    /// use parseltongue_core::entities::{DependencyEdge, EdgeType};
    ///
    /// # tokio_test::block_on(async {
    /// let storage = CozoDbStorage::new("mem").await.unwrap();
    /// storage.create_dependency_edges_schema().await.unwrap();
    ///
    /// // Create chain: A -> B -> C -> D
    /// let edges = vec![
    ///     DependencyEdge::builder()
    ///         .from_key("rust:fn:A:test_rs:1-5")
    ///         .to_key("rust:fn:B:test_rs:10-15")
    ///         .edge_type(EdgeType::Calls)
    ///         .build().unwrap(),
    ///     DependencyEdge::builder()
    ///         .from_key("rust:fn:B:test_rs:10-15")
    ///         .to_key("rust:fn:C:test_rs:20-25")
    ///         .edge_type(EdgeType::Calls)
    ///         .build().unwrap(),
    ///     DependencyEdge::builder()
    ///         .from_key("rust:fn:C:test_rs:20-25")
    ///         .to_key("rust:fn:D:test_rs:30-35")
    ///         .edge_type(EdgeType::Calls)
    ///         .build().unwrap(),
    /// ];
    /// storage.insert_edges_batch(&edges).await.unwrap();
    ///
    /// // Query: What's ALL code reachable from A?
    /// let reachable = storage.get_transitive_closure("rust:fn:A:test_rs:1-5").await.unwrap();
    ///
    /// // Returns B, C, D (all transitively reachable nodes)
    /// assert_eq!(reachable.len(), 3);
    /// assert!(reachable.contains(&"rust:fn:B:test_rs:10-15".to_string()));
    /// assert!(reachable.contains(&"rust:fn:C:test_rs:20-25".to_string()));
    /// assert!(reachable.contains(&"rust:fn:D:test_rs:30-35".to_string()));
    /// # });
    /// ```
    ///
    /// # Algorithm
    /// Uses CozoDB recursive rules for unbounded graph traversal:
    /// 1. **Base case**: Direct outgoing edges from start node
    /// 2. **Recursive case**: Transitively follow all edges
    /// 3. **Termination**: CozoDB's fixed-point semantics guarantee termination (even with cycles)
    ///
    /// # Performance Notes
    /// - Result size grows with graph connectivity
    /// - For large graphs, consider [`calculate_blast_radius`] with hop limits
    /// - Cycle handling is automatic and efficient (no explicit visited set needed)
    ///
    /// # See Also
    /// - [`calculate_blast_radius`] for bounded multi-hop queries with distance tracking
    /// - [`get_forward_dependencies`] for simple 1-hop queries
    pub async fn get_transitive_closure(&self, isgl1_key: &str) -> Result<Vec<String>> {
        // CozoDB recursive query for unbounded reachability
        let query = r#"
            # Transitive closure: Find all nodes reachable from start node

            # Base case: Direct edges from start node
            reachable[to_key] := *DependencyEdges{from_key, to_key},
                                 from_key == $start_key

            # Recursive case: Follow edges transitively (CozoDB handles cycle termination)
            reachable[to_key] := reachable[from],
                                 *DependencyEdges{from_key: from, to_key}

            # Return all unique reachable nodes
            ?[node] := reachable[node]
        "#;

        let mut params = BTreeMap::new();
        params.insert("start_key".to_string(), DataValue::Str(isgl1_key.into()));

        let result = self
            .db
            .run_script(query, params, ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "get_transitive_closure".to_string(),
                reason: format!("Failed to compute transitive closure: {}", e),
            })?;

        // Extract all reachable keys
        let mut reachable = Vec::new();
        for row in result.rows {
            if let Some(DataValue::Str(key)) = row.first() {
                reachable.push(key.to_string());
            }
        }

        Ok(reachable)
    }

    /// Execute raw Datalog query (S01 ultra-minimalist - direct CozoDB access)
    ///
    /// For Tool 2 --query interface. Executes user-provided Datalog directly.
    /// NO query validation, NO safety checks - trust the user (S01 principle).
    pub async fn execute_query(&self, query: &str) -> Result<()> {
        self.db
            .run_script(query, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "execute_query".to_string(),
                details: format!("Datalog query failed: {}", e),
            })?;
        Ok(())
    }

    /// Execute raw Datalog query and return results
    ///
    /// # Arguments
    /// * `query` - Datalog query string
    ///
    /// # Returns
    /// Query results as NamedRows (headers + rows)
    ///
    /// # Example
    /// ```ignore
    /// let result = storage.raw_query("?[key, value] := *CodeGraph{key, value}").await?;
    /// for row in result.rows {
    ///     println!("{:?}", row);
    /// }
    /// ```
    pub async fn raw_query(&self, query: &str) -> Result<cozo::NamedRows> {
        let result = self
            .db
            .run_script(query, Default::default(), ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "raw_query".to_string(),
                details: format!("Datalog query failed: {}", e),
            })?;
        Ok(result)
    }

    /// List all relations in the database
    pub async fn list_relations(&self) -> Result<Vec<String>> {
        let result = self
            .db
            .run_script("::relations", Default::default(), ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "list_relations".to_string(),
                details: format!("Failed to list relations: {}", e),
            })?;

        let mut relations = Vec::new();
        for row in result.rows {
            if let Some(DataValue::Str(name)) = row.first() {
                relations.push(name.to_string());
            }
        }

        Ok(relations)
    }

    /// Insert entity into database
    pub async fn insert_entity(&self, entity: &CodeEntity) -> Result<()> {
        let query = r#"
            ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
              lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
              last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path,
              root_subfolder_L1, root_subfolder_L2] <-
            [[$ISGL1_key, $Current_Code, $Future_Code, $interface_signature, $TDD_Classification,
              $lsp_meta_data, $current_ind, $future_ind, $Future_Action, $file_path, $language,
              $last_modified, $entity_type, $entity_class, $birth_timestamp, $content_hash, $semantic_path,
              $root_subfolder_L1, $root_subfolder_L2]]

            :put CodeGraph {
                ISGL1_key =>
                Current_Code, Future_Code, interface_signature, TDD_Classification,
                lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
                last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path,
                root_subfolder_L1, root_subfolder_L2
            }
        "#;

        let mut params = self.entity_to_params(entity)?;

        // v1.6.5: Compute L1/L2 from file_path
        let file_path_str = entity.interface_signature.file_path.to_str().unwrap_or("");
        let (l1, l2) = crate::storage::path_utils::extract_subfolder_levels_from_path(file_path_str);
        params.insert("root_subfolder_L1".to_string(), DataValue::Str(l1.into()));
        params.insert("root_subfolder_L2".to_string(), DataValue::Str(l2.into()));

        self.db
            .run_script(query, params, ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "insert_entity".to_string(),
                details: format!("Failed to insert entity: {}", e),
            })?;

        Ok(())
    }

    /// Insert multiple entities in a single batch operation
    ///
    /// Implements v1.5.0 batch insertion optimization for 10-60x speedup.
    /// Follows the same pattern as `insert_edges_batch()` for consistency.
    ///
    /// # Performance Contract
    /// - 10,000 entities: < 500ms (v1.5.0 PRIMARY requirement)
    /// - 50,000 entities: < 2s
    /// - Linear scaling: O(n) where n = entity count
    ///
    /// # Implementation Notes
    /// Uses CozoDB inline data syntax: `?[col1, col2] <- [[val1, val2], ...]`
    /// Single `run_script()` call instead of N individual database round-trips.
    ///
    /// # Example
    /// ```ignore
    /// let entities = vec![entity1, entity2, entity3];
    /// storage.insert_entities_batch(&entities).await?;
    /// ```
    pub async fn insert_entities_batch(&self, entities: &[CodeEntity]) -> Result<()> {
        // Empty batch optimization - no database operation needed
        if entities.is_empty() {
            return Ok(());
        }

        // Helper function to escape and quote values for CozoDB inline syntax
        fn quote_value(val: &DataValue) -> String {
            match val {
                DataValue::Str(s) => {
                    // Escape single quotes and backslashes for CozoDB
                    let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
                    format!("'{}'", escaped)
                }
                DataValue::Bool(b) => b.to_string(),
                DataValue::Null => "null".to_string(),
                DataValue::Num(n) => match n {
                    cozo::Num::Int(i) => i.to_string(),
                    cozo::Num::Float(f) => f.to_string(),
                },
                _ => "null".to_string(),
            }
        }

        // Pre-allocate buffer for better performance
        // Estimate: ~500 chars per entity average (conservative)
        let mut query_data = String::with_capacity(entities.len() * 500);

        // Build inline data arrays for batch insert
        for (idx, entity) in entities.iter().enumerate() {
            if idx > 0 {
                query_data.push_str(", ");
            }

            // Convert entity to parameter map
            let params = self.entity_to_params(entity)?;

            // Extract all 17 fields in schema order and build array inline
            query_data.push('[');

            // Field 1: ISGL1_key
            query_data.push_str(&quote_value(params.get("ISGL1_key").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 2: Current_Code
            query_data.push_str(&quote_value(params.get("Current_Code").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 3: Future_Code
            query_data.push_str(&quote_value(params.get("Future_Code").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 4: interface_signature
            query_data.push_str(&quote_value(params.get("interface_signature").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 5: TDD_Classification
            query_data.push_str(&quote_value(params.get("TDD_Classification").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 6: lsp_meta_data
            query_data.push_str(&quote_value(params.get("lsp_meta_data").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 7: current_ind
            query_data.push_str(&quote_value(params.get("current_ind").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 8: future_ind
            query_data.push_str(&quote_value(params.get("future_ind").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 9: Future_Action
            query_data.push_str(&quote_value(params.get("Future_Action").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 10: file_path
            query_data.push_str(&quote_value(params.get("file_path").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 11: language
            query_data.push_str(&quote_value(params.get("language").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 12: last_modified
            query_data.push_str(&quote_value(params.get("last_modified").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 13: entity_type
            query_data.push_str(&quote_value(params.get("entity_type").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 14: entity_class
            query_data.push_str(&quote_value(params.get("entity_class").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 15: birth_timestamp
            query_data.push_str(&quote_value(params.get("birth_timestamp").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 16: content_hash
            query_data.push_str(&quote_value(params.get("content_hash").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 17: semantic_path
            query_data.push_str(&quote_value(params.get("semantic_path").unwrap_or(&DataValue::Null)));
            query_data.push_str(", ");

            // Field 18: root_subfolder_L1 (v1.6.5)
            // Extract from file_path
            let file_path_str = entity.interface_signature.file_path.to_str().unwrap_or("");
            let (l1, l2) = crate::storage::path_utils::extract_subfolder_levels_from_path(file_path_str);
            query_data.push_str(&quote_value(&DataValue::Str(l1.into())));
            query_data.push_str(", ");

            // Field 19: root_subfolder_L2 (v1.6.5, last field, no trailing comma)
            query_data.push_str(&quote_value(&DataValue::Str(l2.into())));

            query_data.push(']');
        }

        // Build complete batch insert query with pre-built data
        let query = format!(
            r#"
            ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
              lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
              last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path,
              root_subfolder_L1, root_subfolder_L2] <- [{}]

            :put CodeGraph {{
                ISGL1_key =>
                Current_Code, Future_Code, interface_signature, TDD_Classification,
                lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
                last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path,
                root_subfolder_L1, root_subfolder_L2
            }}
            "#,
            query_data
        );

        // Execute batch insert - single database round-trip
        self.db
            .run_script(&query, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "insert_entities_batch".to_string(),
                details: format!("Failed to batch insert {} entities: {}", entities.len(), e),
            })?;

        Ok(())
    }

    /// Batch insert excluded test entities (v1.6.5).
    ///
    /// Inserts test entities that were intentionally excluded from CodeGraph
    /// to optimize LLM context. Enables diagnostics reporting.
    ///
    /// # 4-Word Name: insert_test_entities_excluded_batch
    ///
    /// # Performance
    /// - Empty batch: immediate return (no DB operation)
    /// - Single round-trip to database
    ///
    /// # Arguments
    /// * `entities` - Slice of ExcludedTestEntity structs
    pub async fn insert_test_entities_excluded_batch(
        &self,
        entities: &[crate::entities::ExcludedTestEntity],
    ) -> Result<()> {
        // Empty batch optimization
        if entities.is_empty() {
            return Ok(());
        }

        // Helper function to escape and quote string values
        fn quote_str(s: &str) -> String {
            let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
            format!("'{}'", escaped)
        }

        // Pre-allocate buffer
        let mut query_data = String::with_capacity(entities.len() * 200);

        // Build inline data arrays
        for (idx, entity) in entities.iter().enumerate() {
            if idx > 0 {
                query_data.push_str(", ");
            }

            query_data.push('[');
            query_data.push_str(&quote_str(&entity.entity_name));
            query_data.push_str(", ");
            query_data.push_str(&quote_str(&entity.folder_path));
            query_data.push_str(", ");
            query_data.push_str(&quote_str(&entity.filename));
            query_data.push_str(", ");
            query_data.push_str(&quote_str(&entity.entity_class));
            query_data.push_str(", ");
            query_data.push_str(&quote_str(&entity.language));
            query_data.push_str(", ");
            query_data.push_str(&entity.line_start.to_string());
            query_data.push_str(", ");
            query_data.push_str(&entity.line_end.to_string());
            query_data.push_str(", ");
            query_data.push_str(&quote_str(&entity.detection_reason));
            query_data.push(']');
        }

        // Build batch insert query
        let query = format!(
            r#"
            ?[entity_name, folder_path, filename, entity_class, language, line_start, line_end, detection_reason] <- [{}]

            :put TestEntitiesExcluded {{
                entity_name, folder_path, filename =>
                entity_class, language, line_start, line_end, detection_reason
            }}
            "#,
            query_data
        );

        // Execute batch insert
        self.db
            .run_script(&query, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "insert_test_entities_excluded_batch".to_string(),
                details: format!("Failed to batch insert {} test entities: {}", entities.len(), e),
            })?;

        Ok(())
    }

    /// Batch insert file word coverage metrics (v1.6.5).
    ///
    /// Inserts per-file word count coverage comparison between source code
    /// and extracted entities. Enables dual coverage metrics reporting.
    ///
    /// # 4-Word Name: insert_file_word_coverage_batch
    ///
    /// # Performance
    /// - Empty batch: immediate return (no DB operation)
    /// - Single round-trip to database
    ///
    /// # Arguments
    /// * `coverages` - Slice of FileWordCoverageRow structs
    pub async fn insert_file_word_coverage_batch(
        &self,
        coverages: &[crate::entities::FileWordCoverageRow],
    ) -> Result<()> {
        // Empty batch optimization
        if coverages.is_empty() {
            return Ok(());
        }

        // Helper function to escape and quote string values
        fn quote_str(s: &str) -> String {
            let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
            format!("'{}'", escaped)
        }

        // Pre-allocate buffer
        let mut query_data = String::with_capacity(coverages.len() * 250);

        // Build inline data arrays
        for (idx, coverage) in coverages.iter().enumerate() {
            if idx > 0 {
                query_data.push_str(", ");
            }

            query_data.push('[');
            query_data.push_str(&quote_str(&coverage.folder_path));
            query_data.push_str(", ");
            query_data.push_str(&quote_str(&coverage.filename));
            query_data.push_str(", ");
            query_data.push_str(&quote_str(&coverage.language));
            query_data.push_str(", ");
            query_data.push_str(&coverage.source_word_count.to_string());
            query_data.push_str(", ");
            query_data.push_str(&coverage.entity_word_count.to_string());
            query_data.push_str(", ");
            query_data.push_str(&coverage.import_word_count.to_string());
            query_data.push_str(", ");
            query_data.push_str(&coverage.comment_word_count.to_string());
            query_data.push_str(", ");
            query_data.push_str(&coverage.raw_coverage_pct.to_string());
            query_data.push_str(", ");
            query_data.push_str(&coverage.effective_coverage_pct.to_string());
            query_data.push_str(", ");
            query_data.push_str(&coverage.entity_count.to_string());
            query_data.push(']');
        }

        // Build batch insert query
        let query = format!(
            r#"
            ?[folder_path, filename, language, source_word_count, entity_word_count,
              import_word_count, comment_word_count, raw_coverage_pct, effective_coverage_pct,
              entity_count] <- [{}]

            :put FileWordCoverage {{
                folder_path, filename =>
                language, source_word_count, entity_word_count, import_word_count,
                comment_word_count, raw_coverage_pct, effective_coverage_pct, entity_count
            }}
            "#,
            query_data
        );

        // Execute batch insert
        self.db
            .run_script(&query, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "insert_file_word_coverage_batch".to_string(),
                details: format!("Failed to batch insert {} word coverage rows: {}", coverages.len(), e),
            })?;

        Ok(())
    }

    /// Batch insert ignored files (v1.6.5 Wave 1).
    ///
    /// Inserts files that were skipped during ingestion due to missing
    /// language parsers or exclusion patterns.
    ///
    /// # 4-Word Name: insert_ignored_files_batch
    ///
    /// # Performance
    /// - Empty batch: immediate return (no DB operation)
    /// - Single round-trip to database
    ///
    /// # Arguments
    /// * `files` - Slice of IgnoredFileRow structs
    pub async fn insert_ignored_files_batch(
        &self,
        files: &[crate::entities::IgnoredFileRow],
    ) -> Result<()> {
        // Empty batch optimization
        if files.is_empty() {
            return Ok(());
        }

        // Helper function to escape and quote string values
        fn quote_str(s: &str) -> String {
            let escaped = s.replace('\\', "\\\\").replace('\'', "\\'");
            format!("'{}'", escaped)
        }

        // Pre-allocate buffer
        let mut query_data = String::with_capacity(files.len() * 150);

        // Build inline data arrays
        for (idx, file) in files.iter().enumerate() {
            if idx > 0 {
                query_data.push_str(", ");
            }

            query_data.push('[');
            query_data.push_str(&quote_str(&file.folder_path));
            query_data.push_str(", ");
            query_data.push_str(&quote_str(&file.filename));
            query_data.push_str(", ");
            query_data.push_str(&quote_str(&file.extension));
            query_data.push_str(", ");
            query_data.push_str(&quote_str(&file.reason));
            query_data.push(']');
        }

        // Build batch insert query
        let query = format!(
            r#"
            ?[folder_path, filename, extension, reason] <- [{}]

            :put IgnoredFiles {{
                folder_path, filename =>
                extension, reason
            }}
            "#,
            query_data
        );

        // Execute batch insert
        self.db
            .run_script(&query, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "insert_ignored_files_batch".to_string(),
                details: format!("Failed to batch insert {} ignored files: {}", files.len(), e),
            })?;

        Ok(())
    }

    /// Get entity by ISGL1 key
    pub async fn get_entity(&self, isgl1_key: &str) -> Result<CodeEntity> {
        let query = r#"
            ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
              lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
              last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path] :=
            *CodeGraph{
                ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
                lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
                last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path
            },
            ISGL1_key == $key
        "#;

        let mut params = BTreeMap::new();
        params.insert("key".to_string(), DataValue::Str(isgl1_key.into()));

        let result = self.db.run_script(query, params, ScriptMutability::Immutable).map_err(|e| {
            ParseltongError::DatabaseError {
                operation: "get_entity".to_string(),
                details: format!("Failed to get entity: {}", e),
            }
        })?;

        if result.rows.is_empty() {
            return Err(ParseltongError::EntityNotFound {
                isgl1_key: isgl1_key.to_string(),
            });
        }

        self.row_to_entity(&result.rows[0])
    }

    /// Update entity in database (internal method)
    pub async fn update_entity_internal(&self, entity: &CodeEntity) -> Result<()> {
        // Update is same as insert with :put which replaces existing
        self.insert_entity(entity).await
    }

    /// Delete entity from database
    pub async fn delete_entity(&self, isgl1_key: &str) -> Result<()> {
        let query = r#"
            ?[ISGL1_key] <- [[$key]]
            :rm CodeGraph { ISGL1_key }
        "#;

        let mut params = BTreeMap::new();
        params.insert("key".to_string(), DataValue::Str(isgl1_key.into()));

        self.db
            .run_script(query, params, ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "delete_entity".to_string(),
                details: format!("Failed to delete entity: {}", e),
            })?;

        Ok(())
    }

    /// Update temporal state of entity
    pub async fn update_temporal_state(
        &self,
        isgl1_key: &str,
        future_ind: bool,
        future_action: Option<TemporalAction>,
    ) -> Result<()> {
        // Get current entity
        let mut entity = self.get_entity(isgl1_key).await?;

        // Update temporal state
        entity.temporal_state.future_ind = future_ind;
        entity.temporal_state.future_action = future_action.clone();

        // Validate temporal state
        entity.temporal_state.validate()?;

        // Update in database
        self.update_entity_internal(&entity).await
    }

    /// Get entities with pending changes
    pub async fn get_changed_entities(&self) -> Result<Vec<CodeEntity>> {
        let query = r#"
            ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
              lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
              last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path] :=
            *CodeGraph{
                ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
                lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
                last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path
            },
            Future_Action != null
        "#;

        let result = self
            .db
            .run_script(query, Default::default(), ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "get_changed_entities".to_string(),
                details: format!("Failed to query changed entities: {}", e),
            })?;

        let mut entities = Vec::new();
        for row in result.rows {
            entities.push(self.row_to_entity(&row)?);
        }

        Ok(entities)
    }

    /// Get all entities from database
    ///
    /// Returns all entities in the CodeGraph table, regardless of temporal state.
    /// Useful for testing and diagnostic purposes.
    pub async fn get_all_entities(&self) -> Result<Vec<CodeEntity>> {
        let query = r#"
            ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
              lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
              last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path] :=
            *CodeGraph{
                ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
                lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
                last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path
            }
        "#;

        let result = self
            .db
            .run_script(query, Default::default(), ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "get_all_entities".to_string(),
                details: format!("Failed to query all entities: {}", e),
            })?;

        let mut entities = Vec::new();
        for row in result.rows {
            entities.push(self.row_to_entity(&row)?);
        }

        Ok(entities)
    }

    // Helper methods for data conversion

    /// Convert CodeEntity to CozoDB parameters
    fn entity_to_params(&self, entity: &CodeEntity) -> Result<BTreeMap<String, DataValue>> {
        let mut params = BTreeMap::new();

        params.insert(
            "ISGL1_key".to_string(),
            DataValue::Str(entity.isgl1_key.clone().into()),
        );

        params.insert(
            "Current_Code".to_string(),
            entity
                .current_code
                .as_ref()
                .map(|s| DataValue::Str(s.clone().into()))
                .unwrap_or(DataValue::Null),
        );

        params.insert(
            "Future_Code".to_string(),
            entity
                .future_code
                .as_ref()
                .map(|s| DataValue::Str(s.clone().into()))
                .unwrap_or(DataValue::Null),
        );

        // Serialize complex types as JSON
        let signature_json = serde_json::to_string(&entity.interface_signature)
            .map_err(|e| ParseltongError::SerializationError {
                details: format!("Failed to serialize interface_signature: {}", e),
            })?;
        params.insert(
            "interface_signature".to_string(),
            DataValue::Str(signature_json.into()),
        );

        let tdd_json = serde_json::to_string(&entity.tdd_classification)
            .map_err(|e| ParseltongError::SerializationError {
                details: format!("Failed to serialize TDD_Classification: {}", e),
            })?;
        params.insert(
            "TDD_Classification".to_string(),
            DataValue::Str(tdd_json.into()),
        );

        params.insert(
            "lsp_meta_data".to_string(),
            if let Some(ref lsp) = entity.lsp_metadata {
                let lsp_json = serde_json::to_string(lsp)
                    .map_err(|e| ParseltongError::SerializationError {
                        details: format!("Failed to serialize lsp_meta_data: {}", e),
                    })?;
                DataValue::Str(lsp_json.into())
            } else {
                DataValue::Null
            },
        );

        params.insert(
            "current_ind".to_string(),
            DataValue::Bool(entity.temporal_state.current_ind),
        );

        params.insert(
            "future_ind".to_string(),
            DataValue::Bool(entity.temporal_state.future_ind),
        );

        params.insert(
            "Future_Action".to_string(),
            entity
                .temporal_state
                .future_action
                .as_ref()
                .map(|action| {
                    DataValue::Str(
                        match action {
                            TemporalAction::Create => "Create",
                            TemporalAction::Edit => "Edit",
                            TemporalAction::Delete => "Delete",
                        }
                        .into(),
                    )
                })
                .unwrap_or(DataValue::Null),
        );

        params.insert(
            "file_path".to_string(),
            DataValue::Str(
                entity
                    .interface_signature
                    .file_path
                    .to_string_lossy()
                    .to_string()
                    .into(),
            ),
        );

        params.insert(
            "language".to_string(),
            DataValue::Str(entity.extract_language_from_key_validated().into()),
        );

        params.insert(
            "last_modified".to_string(),
            DataValue::Str(entity.metadata.modified_at.to_rfc3339().into()),
        );

        params.insert(
            "entity_type".to_string(),
            DataValue::Str(
                match &entity.interface_signature.entity_type {
                    EntityType::Function => "function",
                    EntityType::Method => "method",
                    EntityType::Struct => "struct",
                    EntityType::Enum => "enum",
                    EntityType::Trait => "trait",
                    EntityType::Interface => "interface",
                    EntityType::Module => "module",
                    EntityType::ImplBlock { .. } => "impl",
                    EntityType::Macro => "macro",
                    EntityType::ProcMacro => "proc_macro",
                    EntityType::TestFunction => "test",
                    EntityType::Class => "class",
                    EntityType::Variable => "variable",
                    EntityType::Constant => "constant",
                    EntityType::Table => "table",    // v1.5.6: SQL table
                    EntityType::View => "view",      // v1.5.6: SQL view
                }
                .into(),
            ),
        );

        // v0.9.3 FIX: Use actual entity_class from entity (was hardcoded to "CODE")
        params.insert(
            "entity_class".to_string(),
            DataValue::Str(
                match entity.entity_class {
                    EntityClass::TestImplementation => "TEST",
                    EntityClass::CodeImplementation => "CODE",
                }
                .into(),
            ),
        );

        // v1.5.0 ISGL1 v2: Add v2 fields
        params.insert(
            "birth_timestamp".to_string(),
            entity
                .birth_timestamp
                .map(DataValue::from)
                .unwrap_or(DataValue::Null),
        );

        params.insert(
            "content_hash".to_string(),
            entity
                .content_hash
                .as_ref()
                .map(|h| DataValue::Str(h.clone().into()))
                .unwrap_or(DataValue::Null),
        );

        params.insert(
            "semantic_path".to_string(),
            entity
                .semantic_path
                .as_ref()
                .map(|p| DataValue::Str(p.clone().into()))
                .unwrap_or(DataValue::Null),
        );

        Ok(params)
    }

    /// Convert CozoDB row to CodeEntity
    fn row_to_entity(&self, row: &[DataValue]) -> Result<CodeEntity> {
        if row.len() < 17 {
            return Err(ParseltongError::DatabaseError {
                operation: "row_to_entity".to_string(),
                details: format!("Invalid row length: expected 17 (with v2 fields), got {}", row.len()),
            });
        }

        // Extract ISGL1 key
        let isgl1_key = match &row[0] {
            DataValue::Str(s) => s.to_string(),
            _ => {
                return Err(ParseltongError::DatabaseError {
                    operation: "row_to_entity".to_string(),
                    details: "ISGL1_key is not a string".to_string(),
                })
            }
        };

        // Extract current_code
        let current_code = match &row[1] {
            DataValue::Str(s) => Some(s.to_string()),
            DataValue::Null => None,
            _ => {
                return Err(ParseltongError::DatabaseError {
                    operation: "row_to_entity".to_string(),
                    details: "Current_Code has invalid type".to_string(),
                })
            }
        };

        // Extract future_code
        let future_code = match &row[2] {
            DataValue::Str(s) => Some(s.to_string()),
            DataValue::Null => None,
            _ => {
                return Err(ParseltongError::DatabaseError {
                    operation: "row_to_entity".to_string(),
                    details: "Future_Code has invalid type".to_string(),
                })
            }
        };

        // Deserialize interface_signature
        let interface_signature: InterfaceSignature = match &row[3] {
            DataValue::Str(s) => serde_json::from_str(s).map_err(|e| {
                ParseltongError::SerializationError {
                    details: format!("Failed to deserialize interface_signature: {}", e),
                }
            })?,
            _ => {
                return Err(ParseltongError::DatabaseError {
                    operation: "row_to_entity".to_string(),
                    details: "interface_signature is not a string".to_string(),
                })
            }
        };

        // Deserialize TDD_Classification
        let tdd_classification: TddClassification = match &row[4] {
            DataValue::Str(s) => serde_json::from_str(s).map_err(|e| {
                ParseltongError::SerializationError {
                    details: format!("Failed to deserialize TDD_Classification: {}", e),
                }
            })?,
            _ => {
                return Err(ParseltongError::DatabaseError {
                    operation: "row_to_entity".to_string(),
                    details: "TDD_Classification is not a string".to_string(),
                })
            }
        };

        // Deserialize lsp_meta_data
        let lsp_metadata: Option<LspMetadata> = match &row[5] {
            DataValue::Str(s) => Some(serde_json::from_str(s).map_err(|e| {
                ParseltongError::SerializationError {
                    details: format!("Failed to deserialize lsp_meta_data: {}", e),
                }
            })?),
            DataValue::Null => None,
            _ => {
                return Err(ParseltongError::DatabaseError {
                    operation: "row_to_entity".to_string(),
                    details: "lsp_meta_data has invalid type".to_string(),
                })
            }
        };

        // Extract temporal state
        let current_ind = match &row[6] {
            DataValue::Bool(b) => *b,
            _ => {
                return Err(ParseltongError::DatabaseError {
                    operation: "row_to_entity".to_string(),
                    details: "current_ind is not a bool".to_string(),
                })
            }
        };

        let future_ind = match &row[7] {
            DataValue::Bool(b) => *b,
            _ => {
                return Err(ParseltongError::DatabaseError {
                    operation: "row_to_entity".to_string(),
                    details: "future_ind is not a bool".to_string(),
                })
            }
        };

        let future_action = match &row[8] {
            DataValue::Str(s) => Some(match s.as_ref() {
                "Create" => TemporalAction::Create,
                "Edit" => TemporalAction::Edit,
                "Delete" => TemporalAction::Delete,
                _ => {
                    return Err(ParseltongError::DatabaseError {
                        operation: "row_to_entity".to_string(),
                        details: format!("Invalid Future_Action value: {}", s),
                    })
                }
            }),
            DataValue::Null => None,
            _ => {
                return Err(ParseltongError::DatabaseError {
                    operation: "row_to_entity".to_string(),
                    details: "Future_Action has invalid type".to_string(),
                })
            }
        };

        let temporal_state = TemporalState {
            current_ind,
            future_ind,
            future_action,
        };

        // Extract entity_class (v0.9.0) - currently ignored in CodeEntity
        let _entity_class = match &row[13] {
            DataValue::Str(s) => s.to_string(),
            _ => "CODE".to_string(), // Default fallback
        };

        // Build CodeEntity
        let mut entity = CodeEntity::new(
            isgl1_key, 
            interface_signature,
            // v0.9.0: Extract entity_class from database row
            match &row[13] {
                cozo::DataValue::Str(s) => {
                    if s == "TEST" {
                        crate::entities::EntityClass::TestImplementation
                    } else {
                        crate::entities::EntityClass::CodeImplementation
                    }
                },
                _ => crate::entities::EntityClass::CodeImplementation, // Default fallback
            }
        )?;
        entity.current_code = current_code;
        entity.future_code = future_code;
        entity.temporal_state = temporal_state;
        entity.tdd_classification = tdd_classification;
        entity.lsp_metadata = lsp_metadata;

        // v1.5.0 ISGL1 v2: Extract v2 fields (indices 14, 15, 16)
        entity.birth_timestamp = match &row[14] {
            DataValue::Num(n) => Some(match n {
                cozo::Num::Int(i) => *i,
                cozo::Num::Float(f) => *f as i64,
            }),
            DataValue::Null => None,
            _ => None, // Ignore invalid types
        };

        entity.content_hash = match &row[15] {
            DataValue::Str(s) => Some(s.to_string()),
            DataValue::Null => None,
            _ => None, // Ignore invalid types
        };

        entity.semantic_path = match &row[16] {
            DataValue::Str(s) => Some(s.to_string()),
            DataValue::Null => None,
            _ => None, // Ignore invalid types
        };

        Ok(entity)
    }

    // ============================================================================
    // INCREMENTAL REINDEX METHODS (PRD-2026-01-28)
    // ============================================================================

    /// Create FileHashCache schema for storing file content hashes
    ///
    /// # 4-Word Name: create_file_hash_cache_schema
    ///
    /// # Performance Contract
    /// - Schema creation: <50ms
    pub async fn create_file_hash_cache_schema(&self) -> Result<()> {
        let schema = r#"
            :create FileHashCache {
                file_path: String =>
                content_hash: String,
                last_updated: String
            }
        "#;

        self.db
            .run_script(schema, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "create_file_hash_cache_schema".to_string(),
                details: format!("Failed to create FileHashCache schema: {}", e),
            })?;

        Ok(())
    }

    /// Get all entities from a specific file
    ///
    /// # 4-Word Name: get_entities_by_file_path
    ///
    /// # Arguments
    /// * `file_path` - The file path to query entities for
    ///
    /// # Returns
    /// Vector of all entities in the specified file
    ///
    /// # Performance Contract
    /// - Query: <10ms for typical file with <100 entities
    pub async fn get_entities_by_file_path(&self, file_path: &str) -> Result<Vec<CodeEntity>> {
        let query = r#"
            ?[ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
              lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
              last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path] :=
            *CodeGraph{
                ISGL1_key, Current_Code, Future_Code, interface_signature, TDD_Classification,
                lsp_meta_data, current_ind, future_ind, Future_Action, file_path, language,
                last_modified, entity_type, entity_class, birth_timestamp, content_hash, semantic_path
            },
            file_path == $target_path
        "#;

        let mut params = BTreeMap::new();
        params.insert("target_path".to_string(), DataValue::Str(file_path.into()));

        let result = self
            .db
            .run_script(query, params, ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "get_entities_by_file_path".to_string(),
                details: format!("Failed to query entities by file path: {}", e),
            })?;

        let mut entities = Vec::new();
        for row in result.rows {
            entities.push(self.row_to_entity(&row)?);
        }

        Ok(entities)
    }

    /// Delete multiple entities by their ISGL1 keys
    ///
    /// # 4-Word Name: delete_entities_batch_by_keys
    ///
    /// # Arguments
    /// * `keys` - Slice of ISGL1 keys to delete
    ///
    /// # Returns
    /// Number of entities deleted
    ///
    /// # Performance Contract
    /// - Batch delete 100 entities: <50ms
    pub async fn delete_entities_batch_by_keys(&self, keys: &[String]) -> Result<usize> {
        if keys.is_empty() {
            return Ok(0);
        }

        let count = keys.len();

        // Build query with inline data for batch delete
        let escaped_keys: Vec<String> = keys
            .iter()
            .map(|k| format!("['{}']", escape_for_cozo_string(k)))
            .collect();

        let query = format!(
            r#"
            ?[ISGL1_key] <- [{}]
            :rm CodeGraph {{ ISGL1_key }}
            "#,
            escaped_keys.join(", ")
        );

        self.db
            .run_script(&query, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "delete_entities_batch_by_keys".to_string(),
                details: format!("Failed to batch delete {} entities: {}", count, e),
            })?;

        Ok(count)
    }

    /// Delete edges where from_key is in the provided list
    ///
    /// # 4-Word Name: delete_edges_by_from_keys
    ///
    /// # Arguments
    /// * `from_keys` - Slice of ISGL1 keys whose outgoing edges should be deleted
    ///
    /// # Returns
    /// Number of edges deleted
    ///
    /// # Performance Contract
    /// - Delete edges for 100 keys: <100ms
    pub async fn delete_edges_by_from_keys(&self, from_keys: &[String]) -> Result<usize> {
        if from_keys.is_empty() {
            return Ok(0);
        }

        // First count existing edges for these from_keys
        let escaped_keys: Vec<String> = from_keys
            .iter()
            .map(|k| format!("'{}'", escape_for_cozo_string(k)))
            .collect();

        let count_query = format!(
            r#"
            ?[count(from_key)] := *DependencyEdges{{from_key, to_key, edge_type}},
                                  from_key in [{}]
            "#,
            escaped_keys.join(", ")
        );

        let count_result = self
            .db
            .run_script(&count_query, Default::default(), ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "delete_edges_by_from_keys".to_string(),
                reason: format!("Failed to count edges before deletion: {}", e),
            })?;

        let mut deleted_count = 0usize;
        if let Some(row) = count_result.rows.first() {
            if let Some(DataValue::Num(n)) = row.first() {
                deleted_count = match n {
                    cozo::Num::Int(i) => *i as usize,
                    cozo::Num::Float(f) => *f as usize,
                };
            }
        }

        if deleted_count == 0 {
            return Ok(0);
        }

        // Build delete query - need to select all matching edges first, then delete
        let delete_query = format!(
            r#"
            to_delete[from_key, to_key, edge_type] := *DependencyEdges{{from_key, to_key, edge_type}},
                                                       from_key in [{}]
            ?[from_key, to_key, edge_type] := to_delete[from_key, to_key, edge_type]
            :rm DependencyEdges {{ from_key, to_key, edge_type }}
            "#,
            escaped_keys.join(", ")
        );

        self.db
            .run_script(&delete_query, Default::default(), ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "delete_edges_by_from_keys".to_string(),
                reason: format!("Failed to delete edges by from_keys: {}", e),
            })?;

        Ok(deleted_count)
    }

    /// Get cached file hash value
    ///
    /// # 4-Word Name: get_cached_file_hash_value
    ///
    /// # Arguments
    /// * `file_path` - The file path to query hash for
    ///
    /// # Returns
    /// Some(hash) if cached, None if not found
    ///
    /// # Performance Contract
    /// - Query: <5ms
    pub async fn get_cached_file_hash_value(&self, file_path: &str) -> Result<Option<String>> {
        let query = r#"
            ?[content_hash] := *FileHashCache{file_path, content_hash},
                               file_path == $path
        "#;

        let mut params = BTreeMap::new();
        params.insert("path".to_string(), DataValue::Str(file_path.into()));

        let result = self
            .db
            .run_script(query, params, ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "get_cached_file_hash_value".to_string(),
                details: format!("Failed to query file hash cache: {}", e),
            })?;

        if let Some(row) = result.rows.first() {
            if let Some(DataValue::Str(hash)) = row.first() {
                return Ok(Some(hash.to_string()));
            }
        }

        Ok(None)
    }

    /// Set cached file hash value
    ///
    /// # 4-Word Name: set_cached_file_hash_value
    ///
    /// # Arguments
    /// * `file_path` - The file path to cache hash for
    /// * `hash` - The SHA-256 hash of the file content
    ///
    /// # Performance Contract
    /// - Insert/Update: <5ms
    pub async fn set_cached_file_hash_value(&self, file_path: &str, hash: &str) -> Result<()> {
        let query = r#"
            ?[file_path, content_hash, last_updated] <- [[$path, $hash, $timestamp]]
            :put FileHashCache {
                file_path =>
                content_hash,
                last_updated
            }
        "#;

        let mut params = BTreeMap::new();
        params.insert("path".to_string(), DataValue::Str(file_path.into()));
        params.insert("hash".to_string(), DataValue::Str(hash.into()));
        params.insert(
            "timestamp".to_string(),
            DataValue::Str(chrono::Utc::now().to_rfc3339().into()),
        );

        self.db
            .run_script(query, params, ScriptMutability::Mutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "set_cached_file_hash_value".to_string(),
                details: format!("Failed to set file hash cache: {}", e),
            })?;

        Ok(())
    }

    /// Count total entities in database
    ///
    /// # 4-Word Name: count_all_entities_total
    ///
    /// # Returns
    /// Total count of entities
    pub async fn count_all_entities_total(&self) -> Result<usize> {
        let query = "?[count(k)] := *CodeGraph{ISGL1_key: k}";

        let result = self
            .db
            .run_script(query, Default::default(), ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DatabaseError {
                operation: "count_all_entities_total".to_string(),
                details: format!("Failed to count entities: {}", e),
            })?;

        if let Some(row) = result.rows.first() {
            if let Some(DataValue::Num(n)) = row.first() {
                return Ok(match n {
                    cozo::Num::Int(i) => *i as usize,
                    cozo::Num::Float(f) => *f as usize,
                });
            }
        }

        Ok(0)
    }

    /// Count total edges in database
    ///
    /// # 4-Word Name: count_all_edges_total
    ///
    /// # Returns
    /// Total count of edges
    pub async fn count_all_edges_total(&self) -> Result<usize> {
        let query = "?[count(e)] := *DependencyEdges{from_key: e}";

        let result = self
            .db
            .run_script(query, Default::default(), ScriptMutability::Immutable)
            .map_err(|e| ParseltongError::DependencyError {
                operation: "count_all_edges_total".to_string(),
                reason: format!("Failed to count edges: {}", e),
            })?;

        if let Some(row) = result.rows.first() {
            if let Some(DataValue::Num(n)) = row.first() {
                return Ok(match n {
                    cozo::Num::Int(i) => *i as usize,
                    cozo::Num::Float(f) => *f as usize,
                });
            }
        }

        Ok(0)
    }
}

// Implement CodeGraphRepository trait
#[async_trait]
impl CodeGraphRepository for CozoDbStorage {
    async fn store_entity(&mut self, entity: CodeEntity) -> Result<()> {
        self.insert_entity(&entity).await
    }

    async fn get_entity(&self, isgl1_key: &str) -> Result<Option<CodeEntity>> {
        match self.get_entity(isgl1_key).await {
            Ok(entity) => Ok(Some(entity)),
            Err(ParseltongError::EntityNotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn update_entity(&mut self, entity: CodeEntity) -> Result<()> {
        self.update_entity_internal(&entity).await
    }

    async fn delete_entity(&mut self, isgl1_key: &str) -> Result<()> {
        self.delete_entity(isgl1_key).await
    }

    async fn query_entities(&self, _query: &TemporalQuery) -> Result<Vec<CodeEntity>> {
        // Simplified implementation for MVP
        // Full query support to be added later
        Ok(Vec::new())
    }

    async fn get_changed_entities(&self) -> Result<Vec<CodeEntity>> {
        self.get_changed_entities().await
    }

    async fn reset_temporal_state(&mut self) -> Result<()> {
        // Get all changed entities
        let changed = self.get_changed_entities().await?;

        for entity in changed {
            let mut updated_entity = entity.clone();

            // Apply temporal changes to current state
            match updated_entity.temporal_state.future_action {
                Some(TemporalAction::Create) => {
                    // New entity becomes current
                    updated_entity.temporal_state.current_ind = true;
                    updated_entity.current_code = updated_entity.future_code.clone();
                }
                Some(TemporalAction::Edit) => {
                    // Apply edit
                    updated_entity.current_code = updated_entity.future_code.clone();
                }
                Some(TemporalAction::Delete) => {
                    // Delete entity
                    self.delete_entity(&entity.isgl1_key).await?;
                    continue;
                }
                None => {}
            }

            // Reset temporal indicators
            updated_entity.temporal_state.future_ind = updated_entity.temporal_state.current_ind;
            updated_entity.temporal_state.future_action = None;
            updated_entity.future_code = None;

            self.update_entity_internal(&updated_entity).await?;
        }

        Ok(())
    }
}
