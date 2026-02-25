# ZAI-REUSABLE-Patterns-01.md: V200 Pattern Extraction from v173

**Generated**: 2026-02-18
**Source**: parseltongue v1.7.2 codebase analysis via API (1021 entities, 9836 edges)
**Purpose**: Identify reusable patterns for V200 clean-room rewrite

---

## 1. Executive Summary

### 1.1 What Was Analyzed

- **Database**: 1021 entities, 9836 dependency edges (Rust only from `crates/` directory)
- **API Endpoints Queried**: 9 graph analysis endpoints + 4 search queries
- **Key Files Read**:
  - `crates/parseltongue-core/src/lib.rs` - Core module structure
  - `crates/parseltongue-core/src/isgl1_v2.rs` - EntityKey implementation
  - `crates/parseltongue-core/src/query_extractor.rs` - Tree-sitter extraction
  - `crates/parseltongue-core/src/storage/cozo_client.rs` - Storage patterns
  - `crates/pt08-http-code-query-server/src/lib.rs` - HTTP handler patterns
  - `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/` - 27 handlers

### 1.2 Key Findings

| Pattern Category | Reusability | Notes |
|------------------|-------------|-------|
| **Entity Key Format** | PARTIAL | ISGL1 v2 good foundation, but V200 uses `|||` delimiter |
| **HTTP Handler Pattern** | HIGH | 4-word naming + consistent response structure reusable |
| **Storage Interface** | HIGH | Single getter contract matches V200 G2 gate |
| **Tree-Sitter Extraction** | HIGH | Declarative query pattern is industry standard |
| **Graph Algorithms** | HIGH | SCC, Leiden, PageRank implementations portable |
| **Error Handling** | HIGH | thiserror pattern matches V200 contract design |
| **Response Format** | HIGH | JSON with success/endpoint/data/tokens reusable |

### 1.3 What Needs Redesign

| Component | Issue | V200 Solution |
|-----------|-------|---------------|
| Entity Key Delimiter | `:` causes parsing issues | `|||` delimiter (ZAI-PRD §6) |
| Scope Encoding | Path mangling with underscores | Hierarchical scope as Vec<String> |
| Unresolved References | `unresolved-reference:0-0` keys | Explicit EXTERNAL entity handling |
| Batch Insert Performance | Inline data string building | Prepared statements + parameterized queries |

---

## 2. Entity Key Patterns (for rust-llm-core-foundation)

### 2.1 Current ISGL1 v2 Implementation

**File**: `crates/parseltongue-core/src/isgl1_v2.rs`

```rust
// Current format: rust:fn:handle_auth:__src_auth_rs:T1706284800
pub fn format_key_v2(
    entity_type: EntityType,
    name: &str,
    language: &str,
    semantic_path: &str,
    birth_timestamp: i64,
) -> String {
    let type_str = match entity_type {
        EntityType::Function => "fn",
        EntityType::Method => "method",
        // ... etc
    };
    format!("{}:{}:{}:{}:T{}", language, type_str, name, semantic_path, birth_timestamp)
}
```

### 2.2 Key Issues Identified

1. **Delimiter Collision**: `:` used both as delimiter AND in generic types (e.g., `std::collections`)
2. **Path Sanitization**: File paths mangled (`src/auth.rs` → `__src_auth_rs`)
3. **No Overload Support**: Same name + file = same key (breaks for overloaded methods)

### 2.3 V200 Migration Path

**From** (ISGL1 v2):
```
rust:fn:handle_auth:__src_auth_rs:T1706284800
```

**To** (V200 `|||` format):
```
rust|||fn|||my_crate::auth::handlers|||login|||src/auth.rs|||d0
```

### 2.4 Reusable Components

| Component | Status | V200 Crate |
|-----------|--------|------------|
| `compute_birth_timestamp()` | REUSE | rust-llm-core-foundation |
| `compute_content_hash()` | REUSE | rust-llm-core-foundation |
| `extract_semantic_path()` | REDESIGN | Use hierarchical scope instead |
| `sanitize_entity_name_for_isgl1()` | REMOVE | `|||` delimiter eliminates need |
| `match_entity_with_old_index()` | REUSE | Incremental reindex logic |

### 2.5 Key Code Snippet: Birth Timestamp

```rust
/// Compute deterministic birth timestamp for entity
pub fn compute_birth_timestamp(file_path: &str, entity_name: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    file_path.hash(&mut hasher);
    entity_name.hash(&mut hasher);
    let hash = hasher.finish();
    
    let base_timestamp = 1577836800; // 2020-01-01 00:00:00 UTC
    let range = 315360000; // ~10 years
    let offset = (hash % range as u64) as i64;
    base_timestamp + offset
}
```

**Reuse in V200**: Directly portable to `rust-llm-core-foundation`.

---

## 3. HTTP Handler Patterns (for rust-llm-interface-gateway)

### 3.1 Handler File Structure

Each handler follows consistent 4-word naming:

```
blast_radius_impact_handler.rs
code_entities_list_all_handler.rs
server_health_check_handler.rs
dependency_edges_list_handler.rs
```

### 3.2 Handler Pattern Template

**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs`

```rust
// 1. Query parameters struct (4-word name)
#[derive(Debug, Deserialize)]
pub struct BlastRadiusQueryParamsStruct {
    pub entity: String,
    #[serde(default = "default_hops")]
    pub hops: usize,
    pub scope: Option<String>,
}

// 2. Response payload struct (4-word name)
#[derive(Debug, Serialize)]
pub struct BlastRadiusResponsePayloadStruct {
    pub success: bool,
    pub endpoint: String,
    pub data: BlastRadiusDataPayloadStruct,
    pub tokens: usize,
}

// 3. Error response struct (4-word name)
#[derive(Debug, Serialize)]
pub struct BlastRadiusErrorResponseStruct {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}

// 4. Handler function (4-word name)
pub async fn handle_blast_radius_impact_analysis(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<BlastRadiusQueryParamsStruct>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;
    
    // Validate parameters
    if params.entity.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(error_response)).into_response();
    }
    
    // Execute query via storage
    let result = compute_blast_radius_by_hops(&state, &params).await;
    
    // Return success response
    (StatusCode::OK, Json(success_response)).into_response()
}
```

### 3.3 V200 Adaptations

| Pattern | Status | Notes |
|---------|--------|-------|
| 4-word naming | KEEP | Matches V200 LLM tokenization goals |
| `State(state)` extraction | KEEP | Axum pattern is clean |
| `update_last_request_timestamp()` | KEEP | For health monitoring |
| Response structure | ADAPT | Add XML-tagged responses (R4) |
| Token estimation | KEEP | Required for context budgeting |
| Scope filtering | ENHANCE | Add project slug (R5) |

### 3.4 V200 Response Format (R4 - XML-Tagged)

```rust
// Current v173:
{
  "success": true,
  "endpoint": "/blast-radius-impact-analysis",
  "data": { ... },
  "tokens": 450
}

// V200 (XML-tagged per R4):
<response endpoint="/myapp/blast-radius-impact-analysis" tokens="450">
  <entities>
    <![CDATA[
    { ... JSON data ... }
    ]]>
  </entities>
</response>
```

---

## 4. Storage Patterns (for rust-llm-store-runtime)

### 4.1 CozoDbStorage Interface

**File**: `crates/parseltongue-core/src/storage/cozo_client.rs`

```rust
pub struct CozoDbStorage {
    db: DbInstance,
}

impl CozoDbStorage {
    // Connection
    pub async fn new(engine_spec: &str) -> Result<Self>;
    pub async fn is_connected(&self) -> bool;
    
    // Schema management
    pub async fn create_schema(&self) -> Result<()>;
    pub async fn create_dependency_edges_schema(&self) -> Result<()>;
    
    // Single entity operations
    pub async fn insert_entity(&self, entity: &CodeEntity) -> Result<()>;
    pub async fn get_entity(&self, isgl1_key: &str) -> Result<CodeEntity>;
    pub async fn delete_entity(&self, isgl1_key: &str) -> Result<()>;
    
    // Batch operations
    pub async fn insert_entities_batch(&self, entities: &[CodeEntity]) -> Result<()>;
    pub async fn insert_edges_batch(&self, edges: &[DependencyEdge]) -> Result<()>;
    
    // Graph queries
    pub async fn get_forward_dependencies(&self, isgl1_key: &str) -> Result<Vec<String>>;
    pub async fn get_reverse_dependencies(&self, isgl1_key: &str) -> Result<Vec<String>>;
    pub async fn calculate_blast_radius(&self, key: &str, hops: usize) -> Result<Vec<(String, usize)>>;
    pub async fn get_transitive_closure(&self, key: &str) -> Result<Vec<String>>;
    
    // Raw query access
    pub async fn raw_query(&self, query: &str) -> Result<NamedRows>;
}
```

### 4.2 Single Getter Contract (G2 Gate)

V200 requires **all read paths** to go through one canonical getter:

```rust
// V200 pattern (SR-P2-F gate):
pub trait GraphReader {
    async fn get_entities(&self, filter: EntityFilter) -> Result<EntityPage>;
    async fn get_edges(&self, filter: EdgeFilter) -> Result<EdgePage>;
    async fn get_entity_by_key(&self, key: &EntityKey) -> Result<Option<Entity>>;
}
```

**Current v173 Compliance**: PARTIAL
- `get_entity()` exists but handlers use `raw_query()` directly
- V200 should wrap all `raw_query()` calls in typed getters

### 4.3 Batch Insert Pattern

```rust
pub async fn insert_entities_batch(&self, entities: &[CodeEntity]) -> Result<()> {
    if entities.is_empty() {
        return Ok(());
    }
    
    // Build inline data arrays
    let mut query_data = String::with_capacity(entities.len() * 500);
    for (idx, entity) in entities.iter().enumerate() {
        if idx > 0 { query_data.push_str(", "); }
        query_data.push('[');
        // ... serialize all fields
        query_data.push(']');
    }
    
    let query = format!(
        "?[...] <- [{}]
         :put CodeGraph {{ ... }}",
        query_data
    );
    
    self.db.run_script(&query, Default::default(), ScriptMutability::Mutable)?;
    Ok(())
}
```

**V200 Improvement**: Use parameterized queries instead of string building.

### 4.4 String Escaping Pattern

```rust
/// Escape string for safe use in CozoDB query strings
pub fn escape_for_cozo_string(s: &str) -> String {
    // ORDER CRITICAL: backslash BEFORE quote
    s.replace('\\', "\\\\").replace('\'', "\\'")
}
```

**Reuse in V200**: Directly portable.

---

## 5. Extraction Patterns (for rust-llm-tree-extractor)

### 5.1 QueryBasedExtractor Architecture

**File**: `crates/parseltongue-core/src/query_extractor.rs`

```rust
pub struct QueryBasedExtractor {
    queries: HashMap<Language, String>,        // .scm files embedded at compile time
    dependency_queries: HashMap<Language, String>,  // v0.9.0: Edge extraction
    parsers: HashMap<Language, Parser>,
}

impl QueryBasedExtractor {
    pub fn new() -> Result<Self> {
        // Embed query files at compile time
        queries.insert(Language::Rust, include_str!("../../../entity_queries/rust.scm"));
        queries.insert(Language::Python, include_str!("../../../entity_queries/python.scm"));
        // ... 12 languages total
        
        // Initialize tree-sitter parsers
        Self::init_parser(&mut parsers, Language::Rust, &tree_sitter_rust::LANGUAGE.into())?;
        // ...
    }
    
    pub fn parse_source(&mut self, source: &str, file_path: &Path, language: Language) 
        -> Result<(Vec<ParsedEntity>, Vec<DependencyEdge>)> 
    {
        let tree = parser.parse(source, None)?;
        let entities = self.execute_query(&tree, source, file_path, language, query_source)?;
        let dependencies = self.execute_dependency_query(&tree, source, ...)?;
        Ok((entities, dependencies))
    }
}
```

### 5.2 Declarative Query Pattern (.scm files)

**Entity Extraction** (`entity_queries/rust.scm`):
```scheme
(function_item name: (identifier) @name) @definition.function
(struct_item name: (type_identifier) @name) @definition.struct
(trait_item name: (type_identifier) @name) @definition.trait
```

**Dependency Extraction** (`dependency_queries/rust.scm`):
```scheme
(call_expression function: (identifier) @reference.call) @dependency.call
(use_declaration argument: (scoped_identifier) @reference.use_full_path) @dependency.use
```

### 5.3 V200 Data-Flow Edge Extraction (R8)

```rust
// R8: Extract assign/param/return flow edges
pub enum FlowEdgeType {
    Assign,   // X = func()
    Param,    // func(X)
    Return,   // return X
}

// V200 tree-sitter query for data flow:
// (assignment_expression left: (identifier) @lhs right: (_) @rhs) @flow.assign
// (call_expression arguments: (arguments (identifier) @param)) @flow.param
// (return_expression value: (identifier) @ret) @flow.return
```

### 5.4 External Dependency Detection

```rust
fn parse_external_dependency_from_path(path: &str) -> Option<(String, String)> {
    let segments: Vec<&str> = path.split("::").collect();
    if segments.len() < 2 { return None; }
    
    let crate_name = segments[0];
    let item_name = segments.last().unwrap();
    
    // Exclude local keywords
    if matches!(crate_name, "crate" | "self" | "super") { return None; }
    
    // Exclude stdlib
    let stdlib = ["std", "core", "alloc", "proc_macro", "test"];
    if stdlib.contains(&crate_name) { return None; }
    
    Some((crate_name.to_string(), item_name.to_string()))
}
```

**V200 Enhancement**: Use `EXTERNAL:` prefix for unresolved references per Q5.

---

## 6. Graph Algorithm Patterns (for rust-llm-graph-reasoning)

### 6.1 Algorithms Implemented (v1.6)

| Algorithm | Endpoint | Purpose |
|-----------|----------|---------|
| Tarjan SCC | `/strongly-connected-components-analysis` | Cycle detection |
| Leiden | `/leiden-community-detection-clusters` | Module clustering |
| PageRank | `/centrality-measures-entity-ranking?method=pagerank` | Entity importance |
| Blast Radius | `/blast-radius-impact-analysis` | Impact analysis |
| Transitive Closure | `get_transitive_closure()` | Reachability |

### 6.2 CozoDB Recursive Query Pattern

```datalog
# Blast radius (bounded BFS)
reachable[to_key, distance] := *DependencyEdges{from_key, to_key},
                               from_key == $start_key,
                               distance = 1

reachable[to_key, new_distance] := reachable[from, dist],
                                    *DependencyEdges{from_key: from, to_key},
                                    dist < $max_hops,
                                    new_distance = dist + 1

?[node, min_dist] := reachable[node, dist],
                     min_dist = min(dist)
:order min_dist
```

### 6.3 V200 Datalog Rules (18 Total)

Per ZAI-PRD §12, these are **open-sourced** rules competitors lock behind proprietary systems:

| Rule | Purpose | Phase |
|------|---------|-------|
| Transitive Trait Hierarchy | `all_supers(t, s)` | Phase 2 |
| Unsafe Call Chain | `unsafe_chain(f)` | Phase 1 |
| Taint Analysis | `tainted(f)` | Phase 2 |
| Reachability | `reachable(a, b)` | Phase 1 |
| Dead Code Detection | `dead_code(f)` | Phase 1 |
| Layer Violation | `layer_violation(f, g)` | Phase 2 |
| Async Boundary | `async_boundary(f, g)` | Phase 2 |
| Circular Deps | `circular_dep(a, b)` | Phase 1 |
| Coupling Metrics | `cbo(m)` | Phase 2 |
| API Surface | `api_surface(e)` | Phase 2 |
| Closure Captures | `closure_captures_unsafe(c, p)` | Phase 2 |
| Error Propagation | `error_chain(a, b)` | Phase 2 |
| Module Cohesion | `same_module_edge(e1, e2)` | Phase 2 |
| Test Coverage Gap | `untested_pub_fn(f)` | Phase 2 |
| Derive Macro Inference | `derive_impl(key, trait)` | Phase 2 |
| God Object | `god_object(f)` | Phase 1 |
| SCC Membership | `in_scc(a, rep)` | Phase 1 |
| Cross-Language Edge Join | `ffi_match(rust_key, c_key)` | Phase 2 |

---

## 7. Error Handling Patterns

### 7.1 Error Type Structure

**File**: `crates/parseltongue-core/src/error.rs`

```rust
#[derive(Debug, Error)]
pub enum ParseltongError {
    #[error("Database operation '{operation}' failed: {details}")]
    DatabaseError { operation: String, details: String },
    
    #[error("Entity not found: {isgl1_key}")]
    EntityNotFound { isgl1_key: String },
    
    #[error("Dependency error: {operation} - {reason}")]
    DependencyError { operation: String, reason: String },
    
    #[error("Circular dependency detected: {path}")]
    CircularDependency { path: String },
    
    #[error("Validation failed: {field} - {expected}, got {actual}")]
    ValidationError { field: String, expected: String, actual: String },
    
    // ... more variants
}
```

### 7.2 V200 Contract Mapping

| v173 Error | V200 Crate | Notes |
|------------|------------|-------|
| `DatabaseError` | rust-llm-store-runtime | Map to `StoreError` |
| `EntityNotFound` | rust-llm-core-foundation | Map to `KeyError` |
| `DependencyError` | rust-llm-graph-reasoning | Map to `EdgeError` |
| `ValidationError` | rust-llm-core-foundation | Map to `ContractError` |

### 7.3 Error Recovery Pattern

```rust
pub trait ErrorRecovery {
    fn recover(&self, error: &ParseltongError) -> Result<RecoveryAction>;
}

pub enum RecoveryAction {
    RetryWithBackoff(Duration),
    RetryWithModifiedParameters,
    UseFallback,
    SkipOperation,
    AbortWorkflow,
}
```

---

## 8. Response Format Patterns

### 8.1 Standard Response Structure

```rust
// Success response
#[derive(Serialize)]
pub struct StandardResponseStruct<T> {
    pub success: bool,          // Always true for success
    pub endpoint: String,       // Endpoint name for debugging
    pub data: T,                // Payload
    pub tokens: usize,          // Token estimate for LLM context
}

// Error response
#[derive(Serialize)]
pub struct StandardErrorStruct {
    pub success: bool,          // Always false for errors
    pub error: String,          // Human-readable error message
    pub endpoint: String,       // Endpoint name
    pub tokens: usize,          // Token estimate
}
```

### 8.2 Token Estimation Pattern

```rust
// Simple character-based estimation
let tokens = 80 + (entity_count * 30) + entity_key.len();

// V200 should use tiktoken for accurate counts (R7)
```

### 8.3 Scope Filtering Pattern

```rust
pub fn parse_scope_build_filter_clause(scope: &Option<String>) -> String {
    match scope {
        Some(s) if s.contains("||") => {
            let parts: Vec<&str> = s.split("||").collect();
            if parts.len() >= 2 {
                format!(
                    ", root_subfolder_L1 == '{}', root_subfolder_L2 == '{}'",
                    parts[0], parts[1]
                )
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}
```

---

## 9. Recommended Refactoring for V200

### 9.1 Critical Changes

| Change | Rationale | PRD Reference |
|--------|-----------|---------------|
| Entity key delimiter `:` → `|||` | Eliminates parsing ambiguity | §6.1-6.4 |
| Add project slug to routes | Multi-project support | R5 |
| Single getter contract | G2 gate requirement | §3.1 |
| XML-tagged responses | LLM consumption | R4 |
| Token count at ingest | Deterministic context | R7 |
| Data-flow edge extraction | Semantic enrichment | R8 |

### 9.2 Architecture Migration

```
v173                          V200
─────────────────────────────────────────────────────
parseltongue-core        →    rust-llm-core-foundation
  ├── entities.rs              ├── EntityKey (||| format)
  ├── isgl1_v2.rs              ├── contracts/
  ├── error.rs                 └── validation/
  ├── query_extractor.rs
  └── storage/

pt08-http-server         →    rust-llm-interface-gateway
  └── handlers/                ├── http/
                                ├── cli/
                                └── mcp/ (Phase 2)

parseltongue-core        →    rust-llm-tree-extractor
  └── query_extractor.rs       ├── queries/
                                └── languages/

(storage module)         →    rust-llm-store-runtime
  └── cozo_client.rs           ├── reader/
                                └── writer/

(graph_analysis)         →    rust-llm-graph-reasoning
  └── algorithms/              ├── datalog/
                                └── rules/
```

---

## 10. Pattern Mappings to V200 Crates

### 10.1 rust-llm-core-foundation

| Source Pattern | V200 Use |
|----------------|----------|
| `EntityKey` format | Redesign with `|||` delimiter |
| `compute_birth_timestamp()` | Direct reuse |
| `compute_content_hash()` | Direct reuse |
| `ParseltongError` | Rename to `ContractError` |
| Entity validation | Enhance for G1 gate |

### 10.2 rust-llm-interface-gateway

| Source Pattern | V200 Use |
|----------------|----------|
| Handler 4-word naming | Direct reuse |
| `State(state)` extraction | Direct reuse |
| Response structure | Add XML tagging (R4) |
| Token estimation | Enhance with tiktoken (R7) |
| Scope filtering | Add project slug (R5) |

### 10.3 rust-llm-store-runtime

| Source Pattern | V200 Use |
|----------------|----------|
| `CozoDbStorage` | Direct reuse |
| Batch insert pattern | Optimize with parameters |
| Single getter contract | Enforce G2 gate |
| Schema migrations | Add TypedCallEdges relation |

### 10.4 rust-llm-tree-extractor

| Source Pattern | V200 Use |
|----------------|----------|
| `QueryBasedExtractor` | Direct reuse |
| .scm query files | Add data-flow queries (R8) |
| 12-language support | Direct reuse |
| External detection | Add `EXTERNAL:` prefix |

### 10.5 rust-llm-graph-reasoning

| Source Pattern | V200 Use |
|----------------|----------|
| CozoDB recursive queries | Direct reuse |
| Blast radius algorithm | Direct reuse |
| SCC/Leiden/PageRank | Direct reuse |
| 18 Datalog rules | Implement Phase 1 rules |

### 10.6 rust-llm-rust-semantics

| Source Pattern | V200 Use |
|----------------|----------|
| (NEW crate) | rust-analyzer integration |
| TypedCallEdges | Primary semantic enrichment |
| Proc-macro handling | Degrade annotations |

### 10.7 rust-llm-cross-boundaries

| Source Pattern | V200 Use |
|----------------|----------|
| (NEW crate) | FFI/WASM/PyO3 detection |
| Confidence scoring | Thresholds per §13.2 |

### 10.8 rust-llm-test-harness

| Source Pattern | V200 Use |
|----------------|----------|
| (NEW crate) | Contract testing |
| G1-G4 gates | Probe definitions |
| Fixture corpus | Create NEW fixtures (Q7) |

---

## Appendix A: Key Code Snippets

### A.1 Entity Key Building (ISGL1 v2)

```rust
pub fn format_key_v2(
    entity_type: EntityType,
    name: &str,
    language: &str,
    semantic_path: &str,
    birth_timestamp: i64,
) -> String {
    let type_str = match entity_type {
        EntityType::Function => "fn",
        EntityType::Method => "method",
        EntityType::Class => "class",
        EntityType::Struct => "struct",
        EntityType::Interface => "interface",
        EntityType::Enum => "enum",
        EntityType::Trait => "trait",
        EntityType::Module => "module",
        EntityType::ImplBlock { .. } => "impl",
        // ...
    };
    format!("{}:{}:{}:{}:T{}", language, type_str, name, semantic_path, birth_timestamp)
}
```

### A.2 Content Hash (SHA-256)

```rust
pub fn compute_content_hash(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}
```

### A.3 Dependency Edge Builder

```rust
DependencyEdge::builder()
    .from_key(from_key.to_string())
    .to_key(to_key.to_string())
    .edge_type(edge_type)  // Calls, Uses, Implements
    .source_location(location.unwrap_or_default())
    .build()
```

### A.4 CozoDB String Escape

```rust
pub fn escape_for_cozo_string(s: &str) -> String {
    // CRITICAL: backslash BEFORE quote
    s.replace('\\', "\\\\").replace('\'', "\\'")
}
```

### A.5 Blast Radius Query

```datalog
reachable[to_key, distance] := *DependencyEdges{from_key, to_key},
                               from_key == $start_key,
                               distance = 1

reachable[to_key, new_distance] := reachable[from, dist],
                                    *DependencyEdges{from_key: from, to_key},
                                    dist < $max_hops,
                                    new_distance = dist + 1

?[node, min_dist] := reachable[node, dist],
                     min_dist = min(dist)
:order min_dist
```

---

## Appendix B: API Response Examples

### B.1 Blast Radius Response

```json
{
  "success": true,
  "endpoint": "/blast-radius-impact-analysis",
  "data": {
    "source_entity": "rust:fn:handle_request:__src_handlers_rs:10-50",
    "hops_requested": 3,
    "total_affected": 15,
    "by_hop": [
      {"hop": 1, "count": 5, "entities": ["rust:fn:auth:...", ...]},
      {"hop": 2, "count": 7, "entities": ["rust:fn:db:...", ...]},
      {"hop": 3, "count": 3, "entities": ["rust:fn:logger:...", ...]}
    ]
  },
  "tokens": 450
}
```

### B.2 SCC Analysis Response

```json
{
  "success": true,
  "endpoint": "/strongly-connected-components-analysis",
  "data": {
    "scc_count": 2790,
    "sccs": [
      {"id": 0, "size": 1, "members": ["rust:fn:any:..."], "risk_level": "NONE"},
      ...
    ]
  },
  "tokens": 125600
}
```

### B.3 Leiden Community Response

```json
{
  "success": true,
  "endpoint": "/leiden-community-detection-clusters",
  "data": {
    "community_count": 597,
    "modularity": 0.1686,
    "communities": [
      {"id": 0, "size": 386, "members": ["rust:module:mpsc:...", ...]},
      ...
    ]
  },
  "tokens": 56835
}
```

---

*Generated: 2026-02-18*
*Analysis Target: parseltongue v1.7.2*
*Total Entities: 1021*
*Total Edges: 9836*
