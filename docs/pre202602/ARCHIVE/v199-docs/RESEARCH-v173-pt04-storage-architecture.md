# RESEARCH: pt04 Storage Architecture — What Goes Beyond CozoDB

**Date**: 2026-02-15
**Status**: Research (no code written)
**Question**: Does pt04 (rust-analyzer semantic analysis) need a different storage approach than CozoDB? If so, what?

---

## The Problem

CozoDB's current schema stores 5 relations. The core two:

```
CodeGraph {
    ISGL1_key: String =>
    Current_Code: String?,
    interface_signature: String,   # JSON blob — NOT queryable inside CozoDB
    TDD_Classification: String,    # JSON blob — NOT queryable
    lsp_meta_data: String?,        # JSON blob — NOT queryable
    entity_type: String,
    entity_class: String,
    file_path: String,
    language: String,
    ...19 columns total
}

DependencyEdges {
    from_key: String,
    to_key: String,
    edge_type: String              # "Calls" | "Uses" | "Implements"
    =>
    source_location: String?
}
```

Three columns (`interface_signature`, `TDD_Classification`, `lsp_meta_data`) are already JSON blobs stored as opaque strings. CozoDB cannot index or query inside them. This is a sign the schema is hitting limits.

rust-analyzer produces data that's fundamentally different from tree-sitter output:

| rust-analyzer output | Shape | CozoDB fit |
|---|---|---|
| Resolved return types, param types | Per-entity metadata | Another JSON blob column? |
| `async`, `unsafe`, visibility flags | Per-entity boolean/enum | Columns, but ONLY for Rust entities |
| Dispatch kind (Direct/TraitMethod/DynDispatch) | Per-EDGE metadata | DependencyEdges has no column |
| `via_trait`, `receiver_type` | Per-edge metadata | DependencyEdges has no column |
| Trait hierarchy (supertrait chains) | Tree/DAG structure | Needs recursive self-referencing relation |
| Closure captures with kinds | Per-closure nested list | Doesn't fit any existing relation |
| Type layouts (size, padding) | Per-struct metadata | Doesn't fit CodeGraph |
| Generic bounds, monomorphization | Per-generic nested structure | Deeply hierarchical |

**Core tension**: CozoDB is a graph database with Datalog. It excels at recursive traversal, joins, and filtering over flat relations. rust-analyzer data is hierarchical, typed, and per-entity-rich in ways flat relations struggle with.

---

## Current Architecture (What We Have)

### Storage Layer

`CozoDbStorage` struct in `crates/parseltongue-core/src/storage/cozo_client.rs`:
- Holds raw `DbInstance` from the cozo crate
- No trait abstraction — concrete struct used directly
- Wrapped in `Arc<RwLock<Option<Arc<CozoDbStorage>>>>` in shared state
- All queries via `raw_query(&str)` returning `cozo::NamedRows`
- String escaping via `escape_for_cozo_string()`

5 CozoDB relations: `CodeGraph`, `DependencyEdges`, `TestEntitiesExcluded`, `FileWordCoverage`, `IgnoredFiles`.

### Graph Algorithms

7 algorithms in `crates/parseltongue-core/src/graph_analysis/`:
- Tarjan SCC, K-Core, PageRank, Betweenness, Shannon Entropy, CK Metrics, Leiden

All operate on `AdjacencyListGraphRepresentation` (pure Rust, NOT CozoDB):

```rust
pub struct AdjacencyListGraphRepresentation {
    forward: HashMap<String, Vec<String>>,
    reverse: HashMap<String, Vec<String>>,
    edge_types: HashMap<(String, String), String>,
    nodes: HashSet<String>,
    edge_count: usize,
}
```

**Data flow**: HTTP handler → `raw_query()` CozoDB → parse to `Vec<(String, String, String)>` → `build_from_dependency_edges()` → algorithm → JSON response.

### HTTP Server

`SharedApplicationStateContainer` in pt08 holds `Arc<RwLock<Option<Arc<CozoDbStorage>>>>`. Handlers clone the Arc, run Datalog queries, parse results. 22 endpoints total.

### Key Insight

Graph algorithms already run on in-memory Rust data structures, not CozoDB. CozoDB is essentially a persistence + query layer that feeds the adjacency list. This means a new storage layer doesn't need to replace CozoDB for graph algorithms — it just needs to be accessible alongside.

---

## 15+ Options Evaluated Across 6 Categories

### Category 1: Graph Databases Embeddable in Rust

#### CozoDB (current) — KEEP for graph traversal, DON'T expand

- **Crate**: `cozo` v0.7.6
- **Maintenance**: Effectively dormant. Last release v0.7.2 (mid-2023). 104 unreleased commits. Unanswered issues accumulating.
- **Strengths**: Built-in graph algorithms (PageRank, SCC, shortest path, community detection). Datalog recursion. Already integrated.
- **Weaknesses**: JSON as opaque strings. No structured property indexing. String-based query API (escape-injection-adjacent).
- **Type data fitness**: 2/10. Can store but cannot query type-level data efficiently.
- **Recursive queries**: 10/10. Native Datalog recursion.
- **Verdict**: Keep for graph traversal. Do not expand its role to type queries.

#### IndraDB

- **Crate**: `indradb-lib`
- **Maintenance**: Active (issues from Sep 2025).
- **Data model**: Directed, typed vertices/edges with JSON properties. Pluggable datastores (in-memory, RocksDB, Sled, PostgreSQL).
- **Strengths**: Property graph model. Embeddable. gRPC support.
- **Weaknesses**: JSON properties same opaque-string problem as CozoDB. No query language — Rust builder API only. No recursive queries. No graph algorithms built in.
- **Type data fitness**: 3/10.
- **Verdict**: Not an improvement for type queries. Weaker than CozoDB for graph algorithms.

#### Kuzu — ELIMINATED (Apple acquisition)

- **Crate**: `kuzu`
- **Status**: Acquired by Apple in October 2025. GitHub repo archived. Last public release v0.11.3.
- **What it was**: Excellent in-process property graph DB with Cypher queries, columnar storage, competitive performance. Would have been strong.
- **Verdict**: Cannot recommend. Rust bindings will bitrot without upstream.

#### agdb (Agnesoft Graph Database)

- **Crate**: `agdb` v0.11.2
- **Maintenance**: Actively maintained.
- **Data model**: Typed, schema-less. Builder-pattern queries. Memory-mapped file with WAL.
- **Strengths**: Rust-native. `UserValue` derive macro. No query parsing overhead.
- **Weaknesses**: No recursive queries. No graph algorithms. Small community.
- **Type data fitness**: 5/10.
- **Verdict**: Interesting but insufficient for compiler-level type data.

#### Oxigraph

- **Crate**: `oxigraph` v0.5.4
- **Maintenance**: Actively maintained (release 20 days ago as of Feb 2026).
- **Data model**: RDF triples with SPARQL 1.1. In-memory or RocksDB backend.
- **Strengths**: SPARQL property paths for transitive/recursive traversal. Full SPARQL 1.1.
- **Weaknesses**: RDF triple model is verbose for typed data. Awkward mapping for complex type signatures.
- **Type data fitness**: 4/10.
- **Verdict**: Good for knowledge graphs, poor fit for compiler type data.

---

### Category 2: Datalog Engines in Rust

#### Ascent — BEST for recursive type queries

- **Crate**: `ascent` v0.8.0
- **Maintenance**: Most active Rust Datalog engine (~8 months since last update, 49K downloads).
- **Features**: Proc-macro Datalog DSL. Stratified negation + aggregation. Lattice support. Parallel (rayon). BYODS.
- **API**:

```rust
ascent! {
    struct TraitHierarchyProgram;
    relation supertrait(String, String);       // (trait, direct_supertrait)
    relation implements(String, String);        // (type, trait)
    relation all_supertraits(String, String);   // transitive closure
    relation all_implementors(String, String);

    all_supertraits(t, s) <-- supertrait(t, s);
    all_supertraits(t, s2) <-- all_supertraits(t, s1), supertrait(s1, s2);
    all_implementors(tr, ty) <-- implements(ty, tr);
    all_implementors(tr, ty) <-- all_supertraits(sub, tr), implements(ty, sub);
}

// Usage:
let mut prog = TraitHierarchyProgram::default();
prog.supertrait = vec![("Handler".into(), "Send".into()), ("Handler".into(), "Sync".into())];
prog.implements = vec![("MyHandler".into(), "Handler".into())];
prog.run();
// prog.all_supertraits and prog.all_implementors now populated
```

- **Strengths**: Natural recursive queries. Compiles to native Rust (zero runtime overhead). BYODS for custom backing stores. Parallel via rayon. Works with String keys (Clone + Eq + Hash).
- **Weaknesses**: Not a database — in-memory `Vec<Tuple>` only. No persistence. Batch-only (no incremental updates). Compile-time code gen can slow builds.
- **Performance note**: For 100K+ tuples, intern Strings to u32 for faster hashing. Use `ascent_par!` for parallel.
- **Type data fitness**: 7/10. Can express rules like "find all functions bounded by trait X" in Datalog.
- **Recursive queries**: 10/10. Primary purpose.
- **Verdict**: Use as query engine over in-memory data for trait hierarchy. Not a storage layer itself.

#### Crepe — Skip (unmaintained, ~2 years)
#### Datafrog — Skip (unmaintained, ~7 years, used in Polonius)

---

### Category 3: In-Memory Relational/Columnar Stores

#### DuckDB

- **Crate**: `duckdb` v1.4.4 (Jan 2026)
- **Maintenance**: Extremely active. Official bindings.
- **Data model**: Full SQL RDBMS, columnar, vectorized execution.
- **Strengths**: SQL for type queries (`WHERE return_type LIKE '%AuthError%'`). Recursive CTEs with `USING KEY` (SIGMOD 2025). DuckPGQ extension for graph queries. In-memory + persistent. Parquet/JSON export.
- **Weaknesses**: C++ core via FFI. ~50MB+ binary size. Not native Rust. Recursive CTEs slower than Datalog.
- **Type data fitness**: 8/10. SQL columns represent all 9 data types naturally.
- **Recursive queries**: 7/10. CTEs work but less elegant than Datalog.
- **Verdict**: Strong candidate if we wanted a single-store solution. Too heavy for a sidecar (50MB binary bloat). Would be a v2.0 full rewrite, not an incremental addition.

#### DataFusion — Overkill (query engine, not a database, no recursive CTEs)
#### Polars — Wrong tool (DataFrame analytics, no graph support, no recursion)

---

### Category 4: Purpose-Built Rust-Native Approaches

#### Custom Store: HashMap + BTreeMap Secondary Indexes — RECOMMENDED

- **Crates**: std `HashMap`/`BTreeMap`, optionally `slotmap` for arena storage
- **Architecture**:

```rust
pub struct TypedAnalysisStore {
    // Primary storage
    entity_metadata: HashMap<String, SemanticEntityInfo>,
    typed_edges: HashMap<(String, String), TypedEdgeInfo>,
    trait_impls: HashMap<String, Vec<TraitImplInfo>>,
    supertrait_edges: Vec<(String, String)>,

    // Secondary indexes (rebuilt on deserialize)
    idx_by_return_type: HashMap<String, Vec<String>>,
    idx_by_visibility: HashMap<Visibility, Vec<String>>,
    idx_by_trait_bound: HashMap<String, Vec<String>>,
    idx_by_call_kind: HashMap<CallKind, Vec<(String, String)>>,
    idx_async_entities: Vec<String>,
    idx_unsafe_entities: Vec<String>,
}
```

- **Strengths**: Zero abstraction cost. O(1) lookups. Full type safety (Rust structs, not JSON). Trivially serializable (serde). No FFI, no external deps. Custom indexes target exactly the fields needed. Sub-microsecond for indexed lookups at 100K entities.
- **Weaknesses**: Must maintain indexes manually. No declarative query language. No query optimizer. Must serialize/deserialize entire store. Recursive queries must be hand-coded or use Ascent.
- **Type data fitness**: 9/10.
- **Recursive queries**: 4/10 alone, 10/10 when paired with Ascent.
- **Serialization**: 10/10 via serde + rmp-serde (MessagePack already in workspace).
- **Verdict**: Best option for type-level queries when combined with Ascent for recursion.

#### ECS Approach (bevy_ecs, hecs) — Wrong abstraction

- Designed for game loops, not database queries. No secondary indexes. Component queries are by type, not by value. Conceptual mismatch: code entities have fixed schemas.
- **Verdict**: Skip.

---

### Category 5: Hybrid Approaches Evaluated

#### Option A: CozoDB (graph) + DuckDB (type queries)

- Best of both worlds on paper.
- Cons: Two database dependencies. Data duplication. Large binary (DuckDB FFI ~50MB+).
- **Verdict**: Too heavy for a sidecar.

#### Option B: CozoDB (graph) + Custom Rust Store + Ascent (recursive) — RECOMMENDED

- Keep CozoDB exactly as-is for all 22 existing endpoints.
- New `TypedAnalysisStore` in pure Rust for pt04's 9 data types.
- Ascent for recursive trait hierarchy queries.
- Share ISGL1 keys between stores.
- Serialize typed store to MessagePack alongside CozoDB snapshot.
- **Pros**: Zero new external deps beyond `ascent` (~0.5MB). No binary bloat. Compile-time type safety. Existing CozoDB untouched (zero regression risk). Sub-microsecond lookups.
- **Cons**: Two data stores to keep in sync (mitigated by shared ISGL1 key). Must maintain custom indexes.

#### Option C: Replace CozoDB entirely with DuckDB + DuckPGQ

- Single source of truth. SQL everywhere.
- **Verdict**: Massive migration. DuckPGQ newer/less proven. Would need to replicate CozoDB's built-in algorithms. v2.0 rewrite, not v1.7.3 task.

---

## Evaluation Matrix

| Criterion | CozoDB (current) | DuckDB | Custom Store + Ascent | agdb | IndraDB | Oxigraph |
|---|---|---|---|---|---|---|
| Typed call edges | JSON blob | SQL columns | Native structs | Builder queries | JSON props | RDF triples |
| Trait hierarchies (recursive) | Datalog recursion | Recursive CTE | Ascent rules | Manual | Manual | SPARQL paths |
| Trait implementations | Can query | Can query | Indexed HashMap | Can query | Can query | Triple patterns |
| Type signature search | Full scan | SQL LIKE/regex | Indexed + filter | Builder filter | JSON scan | SPARQL filter |
| Visibility filtering | String compare | WHERE clause | HashMap index | Builder filter | JSON prop | Triple filter |
| Closure captures | JSON blob | JSON functions | Typed Vec | Builder | JSON prop | Triples |
| Type layouts | JSON blob | Numeric columns | BTreeMap range | Numeric | JSON prop | Numeric literal |
| Generic bounds search | JSON blob | LIKE/array | HashMap index | Builder | JSON prop | Triple pattern |
| Async/unsafe flags | Bool column | Bool column | Bool field | Bool | JSON prop | Bool literal |
| Embeddable Rust | Yes | FFI (bundled) | Pure Rust | Yes | Yes | Yes |
| Maintained | Dormant | Very active | N/A (our code) | Active | Active | Active |
| Binary size impact | ~15MB | ~50MB+ | ~0.5MB | ~5MB | ~10MB | ~15MB |
| Query latency (point) | <5ms | <1ms | <1us | <5ms | <5ms | <10ms |
| Query latency (scan) | <100ms | <50ms | <50ms | <100ms | <100ms | <200ms |
| Serializable | RocksDB/SQLite | Parquet/JSON | MessagePack | File-based | JSON | RDF formats |
| Recursive queries | Native Datalog | CTE | Ascent Datalog | No | No | SPARQL paths |

---

## Recommendation: Option B (CozoDB + Custom Store + Ascent)

```
CozoDB (KEEP — graph traversal)          TypedAnalysisStore (NEW — type queries)
-------------------------------          ----------------------------------------
CodeGraph (entities)                     entity_metadata: HashMap<Isgl1Key, SemanticEntityInfo>
DependencyEdges (syntactic edges)        typed_edges: HashMap<(Isgl1Key, Isgl1Key), TypedEdgeInfo>
TestEntitiesExcluded                     trait_impls: HashMap<String, Vec<TraitImplInfo>>
FileWordCoverage                         supertrait_edges: Vec<(String, String)>
IgnoredFiles                             + secondary indexes (by_return_type, by_trait, etc.)
                                         + Ascent program for recursive trait hierarchy

Shared key: ISGL1_key (String)
pt08 handlers query BOTH, merge at HTTP response layer
```

### What goes where:

| Data | Store | Why |
|------|-------|-----|
| Entities (name, file, line range) | CozoDB `CodeGraph` | Existing, works |
| Syntactic edges (Calls/Uses/Implements) | CozoDB `DependencyEdges` | Existing, graph algos use it |
| Typed edge metadata (call_kind, via_trait, receiver_type) | TypedAnalysisStore | Per-edge enrichment, needs filtering by call_kind |
| Per-entity type info (return_type, params, visibility, async, unsafe) | TypedAnalysisStore | Rich nested data, needs type-based search |
| Trait implementations | TypedAnalysisStore | Join queries + recursive hierarchy |
| Supertrait relationships | TypedAnalysisStore + Ascent | Recursive traversal via Datalog |
| Closure captures | TypedAnalysisStore | Nested per-closure data |
| Type layouts (size, alignment, padding) | TypedAnalysisStore | Numeric filtering/sorting |

### Why this wins:

1. **Type queries become O(1)**: "Find all functions returning AuthError" = `store.idx_by_return_type.get("AuthError")`.
2. **Recursive queries via Ascent**: Trait hierarchy transitive closure in native Datalog.
3. **Zero regression risk**: All 22 existing HTTP endpoints continue using CozoDB unchanged.
4. **Minimal footprint**: `ascent` (proc macro, no runtime) + what's already in workspace (`serde`, `rmp-serde`).
5. **CozoDB maintenance risk mitigated**: If CozoDB becomes fully abandoned, the typed store handles all new features. The existing `graph_analysis/` module already has all 7 algorithms in pure Rust.

### Key dependency: `ascent` v0.8.0

- Proc-macro Datalog DSL. Relations as `Vec<Tuple>`. Rules compile to native Rust.
- Works with String keys (Clone + Eq + Hash). For 100K+, intern to u32.
- Batch evaluation (not incremental). Each `run()` computes full fixpoint.
- Parallel via `ascent_par!` (rayon).
- ~0.5MB dependency footprint.

### CozoDB maintenance risk note

CozoDB appears effectively dormant. Last release v0.7.2 (mid-2023). Issues from 2024-2025 go unanswered. This is a risk factor for the existing graph traversal layer. However, since all 7 graph algorithms already run on `AdjacencyListGraphRepresentation` (pure Rust), CozoDB is effectively a persistence + query convenience layer. If it breaks, the migration path is: replace CozoDB with the TypedAnalysisStore for edge storage too, feeding `AdjacencyListGraphRepresentation` directly from HashMaps instead of Datalog queries.

---

## Integration Points in Current Codebase

### Where a typed store slots in:

1. **SharedApplicationStateContainer** (`pt08/src/http_server_startup_runner.rs`): Add `typed_analysis_store_arc: Arc<RwLock<Option<Arc<TypedAnalysisStore>>>>` alongside existing `database_storage_connection_arc`.

2. **Handler pattern**: Same as CozoDB access. Clone Arc from RwLock, query typed store, merge with CozoDB results at response serialization.

3. **AdjacencyListGraphRepresentation** (`parseltongue-core/src/graph_analysis/adjacency_list_graph_representation.rs`): Add optional `typed_edge_overlay: Option<HashMap<(String, String), TypedEdgeInfo>>` field. Algorithms that need typed edges (Leiden, SCC classification, Entropy) read it. Others ignore it.

4. **Serialization** (pt02): MessagePack the TypedAnalysisStore alongside CozoDB snapshot. Secondary indexes use `#[serde(skip)]` and are rebuilt on deserialize via `rebuild_all_indexes()`.

---

## Ascent API Reference (for implementation)

### Defining relations and rules:

```rust
use ascent::ascent;

ascent! {
    struct MyProgram;
    relation edge(String, String);
    relation reachable(String, String);
    reachable(x, y) <-- edge(x, y);
    reachable(x, z) <-- reachable(x, y), edge(y, z);
}
```

### Populating and running:

```rust
let mut prog = MyProgram::default();
prog.edge = vec![("A".into(), "B".into()), ("B".into(), "C".into())];
prog.run();
// prog.reachable now contains [("A","B"), ("A","C"), ("B","C")]
```

### Features:
- `if expr` guards in rules for filtering
- `!rel(x)` for stratified negation
- `agg y = count() in rel(x, _)` for aggregation (count, min, max, sum, mean)
- `ascent_par!` for parallel evaluation (types must be Send + Sync)
- `ascent_run!` for inline evaluation capturing local variables
- BYODS (`#[ds(trrel_uf)]`) for union-find backed transitive relations
- Lattice support for fixpoint aggregation

---

## Open Questions

1. **Should typed edges live in CozoDB too?** Currently planned for TypedAnalysisStore only. But if we added a `TypedCallEdges` CozoDB relation, Datalog queries could join on call_kind directly. Trade-off: data duplication vs query convenience. Current recommendation: TypedAnalysisStore only, attach to AdjacencyListGraphRepresentation via overlay.

2. **String interning**: At 100K+ entities, String cloning in Ascent is expensive. Should we intern ISGL1 keys to u32 upfront? Yes, if Ascent performance becomes a bottleneck. Not needed at <10K entities.

3. **Incremental updates**: Ascent is batch-only. When a file changes, do we re-run the full Ascent program or is the typed store updated incrementally? Answer: TypedAnalysisStore updates incrementally (HashMap insert/remove). Ascent only re-runs when trait hierarchy queries are needed (infrequent).

4. **CozoDB long-term**: If CozoDB is abandoned, should we migrate graph storage to the TypedAnalysisStore too? The existing `graph_analysis/` algorithms already work on HashMap-based adjacency lists. Replacing CozoDB with direct HashMap storage is feasible but is a separate migration decision.

---

**Last Updated**: 2026-02-15
**Sources**: crates.io, GitHub repos (CozoDB, IndraDB, Kuzu, agdb, Oxigraph, Ascent, Crepe, Datafrog, DuckDB, DataFusion, Polars, bevy_ecs, slotmap), docs.rs, web research for 2025-2026 developments.
