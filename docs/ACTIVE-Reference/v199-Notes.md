# Prep Doc: Parseltongue v2.0.0

**Date**: 2026-02-16
**Context**: Research, architectural decisions, and competitive analysis leading to the v2.0.0 clean break. This document captures everything discovered in the pre-v200 research sessions.

---

## Addendum: Companion app boundary for promoted V200 requirements
- Tauri desktop work is accepted as a companion consumer track, not as a new core crate in the V200 dependency graph.
- Promoted lifecycle requirements (`#7`, `#8`, `#10`, `#27`, `#28`) are implemented as gateway/runtime contracts that both CLI and companion app consume.
- Promoted response and ingest requirements (`#25`, `#29`) are contract obligations on existing interfaces, not architecture expansion.
- Promoted extraction requirement (`#35`) is parser/extractor enrichment feeding existing graph and reasoning layers.
- Any move from companion boundary to core-topology change requires explicit evidence that current contracts cannot satisfy reliability or discoverability goals.

---

## 1. CozoDB Architecture via Parseltongue Dogfooding

We used parseltongue v1.7.2 (release binary) to ingest and analyze the CozoDB source code itself. Key findings:

**Codebase Stats**: 3,105 code entities, 15,547 dependency edges, 149 files, 7 languages (Rust + C/C++ + Java + JS + Swift + TS — multi-language bindings)

### Core Files by Coupling

| Rank | File | Outbound Edges | Role |
|------|------|---------------|------|
| 1 | `cozo-core/src/runtime/db.rs` | 236 | God file — Db struct, run_script, all runtime ops |
| 2 | `cozo-core/src/runtime/relation.rs` | 172 | Stored relation management |
| 3 | `cozo-core/src/query/ra.rs` | 169 | Relational algebra execution |
| 4 | `cozo-core/src/data/functions.rs` | 167 | Built-in scalar functions |
| 5 | `cozo-core/src/data/program.rs` | 158 | AST/program representation |
| 6 | `cozo-core/src/lib.rs` | 118 | Public API surface (DbInstance enum) |

### Key Architectural Findings

- **`DbInstance` is an ENUM, not a struct** — wraps different storage backends
- **`StoreTx` is the storage abstraction trait** — at `storage/mod.rs`, 4 backends: mem, sled, sqlite, tikv + cozorocks (C++ bridge)
- **Storage layer is just KV pairs** — `range_scan`, `batch_put`, `del`, `par_put`. No typed columns.
- **`DataValue` is the core type enum** — everything stored as untyped `DataValue` tuples with `memcmp` byte encoding
- **`ColumnDef` struct** — very basic relation schema definition
- **`FixedRule` extension mechanism** — 23 entities, how CozoDB's built-in graph algorithms work
- **SQALE Tech Debt**: `runtime/db.rs` = 14 hours debt, CBO=236 (23.6x over threshold), health grade F
- **Zero circular dependencies** — clean acyclic architecture
- **K-core max coreness = 20** — innermost core: `parse/expr.rs`, `runtime/hnsw.rs`, `query/compile.rs`, `fts/indexing.rs`, `storage/mod.rs`
- **Leiden modularity = 0.163** — low, monolithic (one community of 617 entities dominates)

### Why Forking CozoDB Is Not Worth It

- Storage layer is just KV pairs — no typed column storage to improve
- God file (`runtime/db.rs`) at 236 outbound edges would need complete refactor
- CozoScript parser tightly coupled to CozoDB's Datalog AST
- Monolithic community structure means changes ripple everywhere
- Adding typed columns means essentially rewriting the database

---

## 2. CozoDB Is a Pass-Through in Our Architecture

The critical discovery: CozoDB's actual role in parseltongue is **serialization**, not computation.

```
WHAT ACTUALLY HAPPENS:

  pt01 ──→ CozoDB ──→ pt08 ──→ HashMap ──→ algorithms ──→ JSON
           │              │
           write          read
           (serialize)    (deserialize)

  No Datalog rules run between write and read.
  All 7 graph algorithms run on a pure Rust HashMap
  (AdjacencyListGraphRepresentation), NOT on CozoDB.
  CozoDB is a filing cabinet.
```

We don't use:
- Datalog's recursive query power
- CozoDB's graph traversal
- Magic sets optimization
- Semi-naive evaluation
- Any of the 6,500 entities of general-purpose database code

We DO use:
- `insert_entities_batch()` — write
- `raw_query()` — read
- That's it.

---

## 3. What Tree-Sitter Gives Us That We Throw Away

### Currently Extracted (skeleton only)
- Entity declarations (fn, struct, trait, class, etc.)
- Call edges (A calls B)
- Uses edges (A uses B)
- Implements edges (A implements B)
- File-level word coverage

### Available But Ignored (80% of the parse tree)

```
GENERICS & BOUNDS:
  type_parameters: <T: Display + Clone, U: Into<String>>
  where_clause: where T: Send + Sync
  These ARE trait dependency edges. Free data we walk past.

VISIBILITY:
  pub, pub(crate), pub(super), pub(in path)
  Tree-sitter parses this as a node. We don't store it.

FUNCTION MODIFIERS:
  async, unsafe, const
  Markers that matter for analysis. Ignored.

ATTRIBUTES:
  #[async_trait], #[derive(...)], #[cfg(...)], #[test]
  #[wasm_bindgen], #[pyfunction], #[no_mangle]
  Metadata about entities. Ignored.

RETURN TYPES & PARAMETER TYPES:
  fn handle(req: HttpRequest) -> Result<Response, AuthError>
  Return type = dependency on Response AND AuthError.
  Parameter type = dependency on HttpRequest.
  All ignored.

CLOSURES:
  closure_expression node has body, move keyword, parameters.
  Ignored.

DOC COMMENTS:
  /// and //! are parsed as nodes.
  Contain API contracts, invariants. Ignored.

LIFETIME ANNOTATIONS:
  'a, 'static — constrain data flow. Ignored.

MACROS:
  macro_invocation nodes tell us which macros used where.
  derive macros generate trait impls = dependency edges. Ignored.

USE STATEMENTS:
  Full import paths. Module-level dependency information.
  Partially captured, could be much richer.
```

### Tree-Sitter Query Language (Not Used At All)

We walk the AST manually with cursors. Tree-sitter has a declarative query language:

```scheme
;; Find all functions with generic bounds (= trait edges for free)
(function_item
  (visibility_modifier) @vis
  (function_modifiers) @mods
  name: (identifier) @name
  type_parameters: (type_parameters
    (constrained_type_parameter
      name: (identifier) @generic_name
      bounds: (trait_bounds) @bounds))
  parameters: (parameters
    (parameter
      pattern: (identifier) @param_name
      type: (_) @param_type))
  return_type: (_) @ret_type)
```

One query, all matches across entire file. Declarative, fast, works identically across all 12 languages (different grammar, same query API).

### Other Tree-Sitter Features Not Used
- **Incremental parsing** — re-parse only changed bytes, not whole file (for file watcher)
- **Language injection** — SQL inside Rust strings, JS inside HTML
- **tree-sitter-tags** — purpose-built definition/reference extraction

---

## 4. Ascent for Typed Datalog Reasoning

Ascent is a proc-macro Rust crate that compiles Datalog rules into efficient Rust code at compile time. No query parser at runtime. No optimizer. The Rust compiler IS the optimizer.

### How It Works

```rust
ascent! {
    struct CodeAnalysis;

    // Base relations (populated from extractors)
    relation entity(String, EntityInfo);
    relation edge(String, String, EdgeKind);
    relation trait_impl(String, String);
    relation supertrait(String, String);
    relation is_async(String);
    relation is_unsafe(String);

    // Derived relations (computed by Ascent)
    relation all_supers(String, String);
    relation unsafe_chain(String);
    relation async_boundary(String, String);

    // Rules
    all_supers(T, S) :- supertrait(T, S);
    all_supers(T, S) :- all_supers(T, M), supertrait(M, S);

    unsafe_chain(F) :- is_unsafe(F);
    unsafe_chain(F) :- edge(F, G, Calls), unsafe_chain(G);

    async_boundary(F, G) :- edge(F, G, Calls), is_async(F), !is_async(G);
}
```

After `prog.run()`, relations are just `Vec<(T1, T2, ...)>`. The "database" is the Ascent program's output. No serialization round-trip. No DataValue enum. Just Rust types.

### Why Datalog Beats SQL, Graph DBs, and Custom Code for This Use Case

1. **Recursive queries are first-class and guaranteed to terminate**
2. **Rules compose naturally** — each layer builds on the previous
3. **Incremental maintenance** (the killer feature) — when a file changes, propagate only the delta

### Comparison

```
SQL recursive CTE:    15 lines, verbose, error-prone, different per DB
Custom Rust code:     12 lines, manual, imperative, no incrementality
Ascent Datalog:       2-3 lines, correct, automatic, incremental
```

---

## 5. Cross-Language Boundary Problem

Real codebases use Rust at the core with tentacles into other languages. Five patterns matter:

### Pattern 1: FFI (Rust <-> C/C++)
- Rust: `extern "C" { fn rocksdb_put(...); }`
- C: `int rocksdb_put(...) { ... }`
- Match on: function name + ABI

### Pattern 2: WASM (Rust -> JS/TS)
- Rust: `#[wasm_bindgen] pub fn process_data(...)`
- JS: `import { process_data } from './pkg/my_module'`
- Match on: attribute + import name

### Pattern 3: PyO3/JNI (Rust -> Python/Java)
- Rust: `#[pyfunction] fn analyze_code(...)`
- Python: `from parseltongue import analyze_code`
- Match on: attribute + import name

### Pattern 4: gRPC/HTTP (Rust <-> Any)
- Rust: `#[post("/api/v1/analyze")] async fn analyze(...)`
- Client: `requests.post("/api/v1/analyze", ...)`
- Match on: URL path string literal

### Pattern 5: Message Queues (Rust <-> Any via Iggy/Kafka)
- Rust: `client.send_messages("user-events", ...)`
- Java: `consumer.subscribe("user-events")`
- Match on: topic name string literal

### Key Insight

Tree-sitter sees BOTH sides of every boundary. It can extract:
- Attributes (`#[wasm_bindgen]`, `#[pyfunction]`, `extern "C"`)
- String literals in function arguments (topic names, URL paths)
- Import statements across all languages

Rust-analyzer sees ONLY the Rust side. It cannot resolve cross-language calls.

### The Architecture

```
Tree-sitter: UNIVERSAL layer (all 12 languages, syntactic)
Rust-analyzer: DEPTH layer (Rust only, semantic)
Ascent: REASONING layer (joins both fact sets, derives cross-cutting insights)
```

pt04/v2.0.0 is a MIX of tree-sitter x rust-analyzer:
- Tree-sitter for ALL languages (width)
- Rust-analyzer for Rust (depth)
- Ascent joins them with rules

---

## 6. Why pt02 and pt03 Are Skipped

### pt02 Is a Sideways Move

```
pt02 solves: "Store the same data without RocksDB"
pt04 solves: "Store RICHER data without RocksDB"

pt02 = same intelligence, different container
pt04 = more intelligence, better container

pt04 ALSO solves Windows (no RocksDB = no lock issue) as a side effect.
```

### pt02's Cost vs Value

**Adds**: Windows compat, portable files, faster cold start
**Does NOT add**: Richer extraction, typed storage, new queries, cross-language, rust-analyzer
**Costs**: New crate, .ptgraph format, /mem/ vs /db/ modes, dual handler paths, doubled testing matrix

### pt03 Was Already Killed

PRD v1.7.3: "It's a `--format` flag if ever needed. Not a crate."

---

## 7. v2.0.0 Decision: Clean Break

### Requirement #1 (from PRD-v200.md)

**NO BACKWARD COMPATIBILITY NEEDED.** New pipeline, new storage, new server.

**NO OLD CODE WILL BE DELETED.** v1.x stays in repo, compiles, works.

### The Zero-Dependency Cut

```
v1.x crates          ─── ZERO SHARED CODE ───    v2.0.0 crates
parseltongue-core         no imports              rust-llm-core
pt01                      no deps                 rust-llm-01
pt08                      no shared types         rust-llm-06
parseltongue (bin)        clean break             rust-llm (bin)
```

Both coexist in same Cargo workspace. Neither depends on the other.

---

## 8. v2.0.0 Crate Architecture

### New Crate Prefix: `rust-llm-*`

The `pt*` prefix is frozen. All new crates use `rust-llm-*`.

### Crate Map

```
crates/
│
│  ═══ v1.x (FROZEN, no changes, still compiles) ═══
├── parseltongue-core/
├── pt01-folder-to-cozodb-streamer/
├── pt02-folder-to-ram-snapshot/
├── pt08-http-code-query-server/
├── parseltongue/
│
│  ═══ v2.0.0 (NEW, zero shared code with v1.x) ═══
├── rust-llm-core/                    SHARED FOUNDATION
│   ├── Fact types (typed Rust structs, not strings)
│   ├── Key format (new, not ISGL1)
│   ├── Language enum, error types
│   └── Trait definitions (Extractor, Store, Server)
│
├── rust-llm-01-fact-extractor/       ENRICHED TREE-SITTER
│   ├── Tree-sitter QUERIES (not cursor walking)
│   ├── 12 languages
│   ├── Extracts: entities, edges, generics, bounds,
│   │   visibility, async, unsafe, attributes, return types,
│   │   params, closures, imports, string literals
│   └── Output: Vec<Fact>
│
├── rust-llm-02-cross-lang-edges/     BOUNDARY DETECTOR
│   ├── FFI, WASM, JNI, PyO3, gRPC, message queues
│   └── Output: Vec<CrossLangEdge>
│
├── rust-llm-03-rust-analyzer/        RUST DEPTH
│   ├── ra_ide / ra_hir bridge
│   ├── Resolved types, trait impls, layouts, closures
│   └── Output: Vec<SemanticFact>
│
├── rust-llm-04-reasoning-engine/     ASCENT DATALOG + ALGORITHMS
│   ├── ascent! { ... } compiled rules
│   ├── 7 graph algorithms (rewritten, typed)
│   ├── Taint analysis rules
│   ├── 30+ built-in rules, user-extensible
│   └── Output: DerivedFacts
│
├── rust-llm-05-knowledge-store/      TYPED INDEXED STORAGE
│   ├── TypedAnalysisStore (HashMaps + secondary indexes)
│   ├── MessagePack serialization
│   └── NO CozoDB. NO RocksDB. Pure Rust.
│
├── rust-llm-06-http-server/          HTTP QUERY SERVER
│   ├── Axum, new endpoint design
│   ├── Websocket live updates
│   └── LLM-optimized responses
│
├── rust-llm-07-mcp-server/           MCP TOOL SERVER
│   ├── rmcp (stdio transport)
│   └── Claude Desktop / Cursor / VS Code native
│
└── rust-llm/                         CLI BINARY
    ├── rust-llm ingest .
    ├── rust-llm serve
    └── rust-llm mcp
```

### Dependency Flow

```
rust-llm-01 (fact extractor)
    │
    ├──→ rust-llm-02 (cross-language edges)
    │        │
    │        ▼
    │    rust-llm-04 (reasoning engine / Ascent)
    │        ▲
    │        │
    └──→ rust-llm-03 (rust-analyzer bridge)
             │
             ▼
         rust-llm-04 (reasoning engine / Ascent)
             │
             ▼
         rust-llm-05 (knowledge store)
             │
             ├──→ rust-llm-06 (HTTP server)
             └──→ rust-llm-07 (MCP server)
```

---

## 9. What v1.x Features Become in v2.0.0

| v1.x Feature | v2.0.0 Fate |
|---|---|
| Tree-sitter cursor walking | REWRITE as tree-sitter queries in rust-llm-01 |
| Entity types + ISGL1 keys | REDESIGN as typed Rust structs in rust-llm-core |
| CozoDB storage (cozo_client.rs) | DROP — replaced by HashMaps in rust-llm-05 |
| 7 graph algorithms | REWRITE in rust-llm-04 on typed data |
| AdjacencyListGraphRepresentation | REPLACED by TypedAnalysisStore in rust-llm-05 |
| pt01 streamer | REWRITE as rust-llm-01 (ignore crate + queries) |
| pt08 HTTP server | REWRITE as rust-llm-06 (new endpoints) |
| pt02 snapshot | DROP (rust-llm-05 does this better) |
| parseltongue CLI | REWRITE as rust-llm binary |
| 22 HTTP endpoints | REDESIGN in rust-llm-06 (learned what worked) |
| Coverage bugs (5 bugs) | SOLVED BY DESIGN (ignore crate, proper paths) |
| .gitignore respect | BUILT-IN from day 1 (ignore crate) |
| Token counts (bpe-openai) | BUILT-IN from day 1 |
| Debug eprintln noise | NEVER HAPPENS (tracing crate from day 1) |
| XML-tagged responses | BUILT-IN from day 1 (typed responses) |
| Taint analysis | VIA ASCENT in rust-llm-04 (not CozoDB Datalog) |
| MCP (pt09) | OWN CRATE: rust-llm-07 |
| /db/ vs /mem/ modes | GONE (one mode: typed store) |
| CozoDB anything | GONE |

---

## 10. Product Strategy (Shreyas Doshi Lens)

### v1.x = Tool, v2.0.0 = Platform

v1.x is a pipeline: data in one end, out the other. v2.0.0 is composable libraries.

### What Compounds

1. **Facts compound** — every new tree-sitter extraction makes every Ascent rule more powerful
2. **Rules compound** — the 50th rule is more powerful than the 1st (stands on 49 others)
3. **Languages compound** — adding language #13 connects to the other 12 (network effect)

### Leverage Hierarchy

```
OVERHEAD:   Building pt02 (same data, different container)
NEUTRAL:    Bug fixes in pt08 (necessary, doesn't move needle)
LEVERAGE:   rust-llm-01 (every fact extracted makes every rule stronger)
LEVERAGE:   rust-llm-04 (every rule compounds on every other rule)
LEVERAGE:   rust-llm-02 (cross-language edges — nobody else has this)
```

### The Moat

The Ascent rules. Every team adds their own. Those rules become institutional knowledge encoded in code. Once you have 50 custom rules, you can't switch away.

### Who Else Wants These Libraries

| Crate | External Users |
|---|---|
| rust-llm-01 (fact extractor) | IDE plugins, CI pipelines, doc generators, code search |
| rust-llm-02 (cross-lang edges) | Microservice visualization, API contract verification, security audit |
| rust-llm-03 (rust-analyzer bridge) | Rust linters, perf analysis, unsafe audit tools |
| rust-llm-04 (reasoning engine) | Security teams, architecture teams, platform teams |

---

## 11. Open Questions for PRD-v200.md

1. **Key format**: Is ISGL1 the right key format for v2.0.0, or do we redesign?
2. **Incremental computation**: Ascent doesn't support incremental. Do we need Differential Datalog instead?
3. **rust-analyzer API stability**: ra_ide/ra_hir are internal APIs. How do we handle breakage?
4. **CLI naming**: Is the binary called `rust-llm` or `parseltongue` (same name, new version)?
5. **File format**: What replaces .ptgraph? Name, extension, versioning?
6. **Build order**: Which crate do we build first? (Likely rust-llm-core + rust-llm-01)
7. **Minimum viable v2.0.0**: What's the smallest v2.0.0 that ships? (Probably rust-llm-01 + 05 + 06 = enriched extraction + store + server, no rust-analyzer yet)
