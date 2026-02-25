# PRD Contracts: Parseltongue v2.0.0

**Generated**: 2026-02-17
**Sources**: v200-docs directory (18 documents)
**Purpose**: Comprehensive contracts document defining v2.0.0 specifications, decisions, user journeys, technical specs, risks, and implementation priorities

---

## Table of Contents

1. [v200 Vision Summary](#1-v200-vision-summary)
2. [Core Requirements](#2-core-requirements)
3. [Architecture Contracts](#3-architecture-contracts)
4. [Data Contracts](#4-data-contracts)
5. [API Contracts](#5-api-contracts)
6. [Behavior Contracts](#6-behavior-contracts)
7. [User Journeys](#7-user-journeys)
8. [Technical Specifications](#8-technical-specifications)
9. [Risks and Mitigations](#9-risks-and-mitigations)
10. [Implementation Priorities](#10-implementation-priorities)

---

## 1. v200 Vision Summary

### 1.1 What v200 Is Trying to Achieve

Parseltongue v2.0.0 is a **clean-room rewrite** of the code analysis toolkit with a fundamental reframe: **The primary user is an LLM, not a human.**

**Core Value Proposition**: 99% token reduction (2-5K tokens vs 500K raw dumps), 31x faster than grep, with architectural understanding no other tool provides.

**Key Differentiators**:
1. **Graph-native analysis** - 7+ algorithms (SCC, PageRank, k-core, Leiden, SQALE, entropy, CK metrics)
2. **Cross-language edge detection** - FFI, WASM, PyO3, gRPC, message queue boundaries
3. **Token-budgeted LLM context** - Architecturally-ranked context selection
4. **Datalog reasoning** - Ascent-based custom rules (embeddable CodeQL alternative)
5. **MCP-first interface** - Native integration with Claude Desktop, Cursor, VS Code

**Source**: `PRD-v200.md`, `Prep-V200-Competitive-Deep-Dive.md`

### 1.2 The Clean Break Principle

**Requirement #1**: No backward compatibility needed. v2.0.0 is a clean break with new ingestion pipeline, storage, and server.

**Requirement #2**: No old code will be deleted. v1.x stays in repo, compiles, and works. v2.0.0 is additive.

**Source**: `PRD-v200.md`

### 1.3 Positioning Statement

> **rust-llm: The code intelligence layer that AI coding tools are missing.**

Every AI coding tool (Cursor, Aider, Continue, Cody, Claude Code, Copilot) needs to understand code architecture to give better answers. None of them have it. rust-llm provides it as a library, MCP server, or HTTP API.

**Source**: `Prep-V200-Max-Adoption-Architecture-Strategy.md`

---

## 2. Core Requirements

### 2.1 Lifecycle + Companion Bundle (Promoted from PRD v173)

| ID | Requirement | Owner Crate | Description |
|----|-------------|-------------|-------------|
| R1 | Route prefix nesting | rust-llm-interface-gateway | Stable namespaced routing by active mode |
| R2 | Auto port + port file | rust-llm-interface-gateway | Deterministic startup/discovery lifecycle |
| R3 | Shutdown CLI command | rust-llm-interface-gateway | Graceful stop contract from CLI to server |
| R4 | XML-tagged responses | rust-llm-interface-gateway | Semantic response grouping for LLM consumption |
| R5 | Project slug in URL | rust-llm-interface-gateway | Self-describing multi-project endpoint identity |
| R6 | Slug in port file | rust-llm-interface-gateway | Slug-aware server discovery path |
| R7 | Token count at ingest | rust-llm-store-runtime | Persist real token_count for deterministic use |
| R8 | Data-flow tree-sitter queries | rust-llm-tree-extractor | Extract assign/param/return flow edges |

**Source**: `ES-V200-Decision-log-01.md`

### 2.2 Non-Negotiable Hardening Gates

| ID | Gate | Meaning | Pass Link |
|----|------|---------|-----------|
| G1 | Slim types gate | Entity/storage schema stays canonical, minimal, deterministic | CF-P1-F |
| G2 | Single getter contract | All read paths go through one storage getter contract | SR-P2-F |
| G3 | Filesystem source-read | Detail view returns current disk lines with explicit error contract | GW-P7-F |
| G4 | Path normalization coverage | Coverage treats ./path, path, absolute path as one file | TH-P8-F |

**Source**: `ES-V200-Hashing-Risks-v01.md`

---

## 3. Architecture Contracts

### 3.1 Crate Architecture (8 Crates)

```
rust-llm-core           -- Foundation types, traits, key format
rust-llm-tree-extractor -- Tree-sitter parsing, entity extraction
rust-llm-rust-semantics -- rust-analyzer bridge for deep Rust analysis
rust-llm-cross-boundaries -- Cross-language edge detection
rust-llm-graph-reasoning -- Ascent Datalog reasoning engine
rust-llm-store-runtime  -- HashMap-based typed analysis store
rust-llm-interface-gateway -- HTTP server, MCP server
rust-llm-test-harness   -- Contract gates, fixture testing
```

**Source**: `ES-V200-Hashing-Risks-v01.md`

### 3.2 Dependency Flow

```
rust-llm-interface-gateway (ingest/query)
    ├── rust-llm-tree-extractor (entities)
    ├── rust-llm-rust-semantics (Rust enrichment)
    ├── rust-llm-cross-boundaries (cross-lang edges)
    ├── rust-llm-graph-reasoning (derived facts)
    └── rust-llm-store-runtime (persistence)

rust-llm-core (shared types) ─── all crates depend on this
```

**Source**: `ES-V200-Hashing-Risks-v01.md`

### 3.3 Problem-Shaped Crates Strategy

For maximum adoption, organize by problem solved:

| Crate | Problem Solved | Pitch |
|-------|----------------|-------|
| rust-llm-context | "I need the right code context for my LLM" | Token-budgeted, ranked context |
| rust-llm-graph | "I need architectural understanding" | 7 graph algorithms |
| rust-llm-crosslang | "What connects my languages?" | FFI/WASM/PyO3/gRPC detection |
| rust-llm-safety | "Show me unsafe paths" | Taint analysis, unsafe chains |
| rust-llm-rules | "I want custom analysis rules" | Datalog rules as moat |

**Source**: `Prep-V200-Max-Adoption-Architecture-Strategy.md`

---

## 4. Data Contracts

### 4.1 EntityKey Format

**Contract**: All entity keys follow this serialization format:

```
{language}|||{entity_kind}|||{scope}|||{name}|||{file_path}|||{discriminator}
```

**Example**:
```
rust|||fn|||my_crate::auth::handlers|||login|||src/auth.rs|||d0
python|||class|||services.auth_service|||AuthService|||lib/services/auth_service.py|||d0
java|||method|||com.example.parser.Parser|||parse|||src/Parser.java|||String_int
```

**Rust Struct**:
```rust
pub struct EntityKey {
    pub language: Language,
    pub kind: EntityKind,
    pub scope: Vec<String>,
    pub name: String,
    pub file_path: String,
    pub discriminator: String,
}
```

**Source**: `Prep-V200-Key-Format-Design.md`, `Prep-V200-isgl1-ambiguity-risk-table.md`

### 4.2 EntityKey Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| M1 | Unique across 12+ languages | Same name + same file = different keys |
| M2 | Human-readable | LLMs must understand keys without lookup |
| M3 | Stable across minor edits | Comments/blank lines don't change keys |
| M4 | Hierarchical | Encode containment: file > module > class > method |
| M5 | Hashable | HashMap key efficiency |
| M6 | Serializable | JSON/MessagePack/MCP support |
| M7 | Delimiter-safe | `|||` has zero collisions across 15 languages |
| M8 | No sanitization required | Raw generic types, module paths |
| M9 | Deterministic | Same input = same key, every time |
| M10 | Overload-distinguishable | Same name + different params = different keys |

**Source**: `Prep-V200-Key-Format-Design.md`

### 4.3 CrossLangEdge Contract

```rust
pub struct CrossLangEdge {
    pub source: BoundaryEndpoint,
    pub target: BoundaryEndpoint,
    pub pattern: CrossLangPattern,
    pub confidence: f64,  // [0.0, 1.0]
    pub signals: Vec<DetectionSignal>,
}

pub enum CrossLangPattern {
    FfiRaw,         // extern "C" + #[no_mangle]
    FfiCxx,         // #[cxx::bridge]
    WasmBindgen,    // #[wasm_bindgen]
    PyO3,           // #[pyfunction] / #[pyclass]
    Jni,            // Java_* naming / robusta
    RubyFfi,        // magnus / rutie
    Http,           // Route + request matching
    Grpc,           // Proto service + impl
    MessageQueue,   // Topic-based pub/sub
}
```

**Confidence Thresholds**:
- >= 0.80: High confidence, include in graph by default
- 0.60-0.79: Medium confidence, include with "uncertain" flag
- 0.40-0.59: Low confidence, include only if user opts in
- < 0.40: Rejected, do not include

**Source**: `Prep-V200-Cross-Language-Detection-Heuristics.md`

### 4.4 FactSet Protocol (for Ecosystem)

```rust
pub struct FactSet {
    pub version: u32,
    pub entities: Vec<EntityFact>,
    pub edges: Vec<EdgeFact>,
    pub attributes: Vec<AttributeFact>,
    pub cross_lang_edges: Vec<CrossLangEdge>,
}
```

**Purpose**: Decouple producers (tree-sitter, rust-analyzer, LSP) from consumers (graph algorithms, HTTP server, MCP server).

**Source**: `Prep-V200-Max-Adoption-Architecture-Strategy.md`

---

## 5. API Contracts

### 5.1 HTTP REST Endpoints (22 Total)

| Category | Endpoint | Description |
|----------|----------|-------------|
| Core | `/server-health-check-status` | Health check |
| Core | `/codebase-statistics-overview-summary` | Stats summary |
| Core | `/api-reference-documentation-help` | API docs |
| Entity | `/code-entities-list-all` | All entities |
| Entity | `/code-entity-detail-view/{key}` | Entity detail |
| Entity | `/code-entities-search-fuzzy?q=pattern` | Fuzzy search |
| Graph | `/dependency-edges-list-all` | All edges |
| Graph | `/reverse-callers-query-graph?entity=X` | Who calls X? |
| Graph | `/forward-callees-query-graph?entity=X` | What does X call? |
| Analysis | `/blast-radius-impact-analysis?entity=X&hops=N` | Impact analysis |
| Analysis | `/circular-dependency-detection-scan` | Cycle detection |
| Analysis | `/complexity-hotspots-ranking-view?top=N` | Coupling hotspots |
| Analysis | `/semantic-cluster-grouping-list` | Module clusters |
| Advanced | `/smart-context-token-budget?focus=X&tokens=N` | LLM context |
| Graph v1.6 | `/strongly-connected-components-analysis` | Tarjan SCC |
| Graph v1.6 | `/technical-debt-sqale-scoring?entity=X` | SQALE tech debt |
| Graph v1.6 | `/kcore-decomposition-layering-analysis?k=N` | K-core layering |
| Graph v1.6 | `/centrality-measures-entity-ranking?method=pagerank` | PageRank/Betweenness |
| Graph v1.6 | `/entropy-complexity-measurement-scores?entity=X` | Shannon entropy |
| Graph v1.6 | `/coupling-cohesion-metrics-suite?entity=X` | CK metrics |
| Graph v1.6 | `/leiden-community-detection-clusters` | Leiden clustering |
| Coverage | `/ingestion-coverage-folder-report?depth=N` | Ingestion coverage |

**Source**: `CLAUDE.md` (project instructions)

### 5.2 MCP Tools (20+ Mapped)

| Tool | HTTP Equivalent | Description |
|------|-----------------|-------------|
| `analyze_codebase` | POST /analyze | Ingest and analyze codebase |
| `get_context` | GET /context | Token-budgeted LLM context |
| `blast_radius` | GET /blast-radius | Impact analysis |
| `search_entities` | GET /search | Fuzzy entity search |
| `get_entity_detail` | GET /entities/{key} | Entity detail view |
| `get_architecture` | GET /architecture | SCC + communities + hotspots |
| `find_cycles` | GET /cycles | Circular dependency detection |
| `find_unsafe_chains` | GET /unsafe-chains | Unsafe code path tracing |
| `get_tech_debt` | GET /tech-debt | SQALE tech debt scores |
| `get_centrality` | GET /centrality | PageRank/betweenness rankings |
| `get_coupling_metrics` | GET /coupling | CBO/LCOM/RFC/WMC metrics |
| `detect_cross_lang` | GET /cross-lang-edges | Cross-language boundaries |
| `get_callers` | GET /reverse-callers | Who calls this entity? |
| `get_callees` | GET /forward-callees | What does this call? |
| `get_statistics` | GET /statistics | Codebase overview stats |
| `run_rule` | POST /rules/run | Execute custom Ascent rule |

**Source**: `Prep-V200-MCP-Protocol-Integration.md`

### 5.3 MCP Tool Schema: get_context (The Killer Tool)

```json
{
  "name": "get_context",
  "description": "Get the most architecturally relevant code context for a given entity, optimized for your token budget. Uses PageRank, blast radius, SCC membership, Leiden community clustering, and coupling metrics to rank what to include.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "entity": {
        "type": "string",
        "description": "Entity key (e.g., 'rust:fn:handle_request')"
      },
      "token_budget": {
        "type": "integer",
        "description": "Maximum tokens to include (default: 4096)",
        "default": 4096
      },
      "include_callers": {
        "type": "boolean",
        "description": "Include entities that call this entity",
        "default": true
      },
      "include_callees": {
        "type": "boolean",
        "description": "Include entities this entity calls",
        "default": true
      }
    },
    "required": ["entity"]
  }
}
```

**Source**: `Prep-V200-MCP-Protocol-Integration.md`

### 5.4 MCP Resources (Application-Controlled)

| URI | Description | Content Type |
|-----|-------------|--------------|
| `codebase://entities` | All code entities (summary) | application/json |
| `codebase://entities/{key}` | Single entity detail | application/json |
| `codebase://graph` | Full dependency edge list | application/json |
| `codebase://metrics` | Analysis metrics summary | application/json |
| `codebase://architecture/scc` | Strongly connected components | application/json |
| `codebase://architecture/communities` | Leiden community clusters | application/json |
| `codebase://cross-lang-edges` | Cross-language boundaries | application/json |

**Source**: `Prep-V200-MCP-Protocol-Integration.md`

### 5.5 MCP Prompts (User-Initiated)

| Prompt | Purpose | Arguments |
|--------|---------|-----------|
| `analyze_architecture` | Comprehensive architecture review | focus?: string |
| `find_security_concerns` | Security audit workflow | scope?: string |
| `understand_entity` | Deep-dive on specific entity | entity: string |
| `review_change_impact` | Blast radius analysis | entity: string |
| `find_tech_debt` | Technical debt assessment | top?: integer |
| `onboard_to_codebase` | New developer onboarding | none |

**Source**: `Prep-V200-MCP-Protocol-Integration.md`

---

## 6. Behavior Contracts

### 6.1 Dependency Graph Contract Hardening Method

**Purpose**: Reduce architecture risk through interface-level evidence using the dependency graph.

**Pass Loop** (repeat for each crate):
1. Select one crate (highest dependency impact and/or highest Risk/Unclear)
2. Freeze public interface contract for that crate (input, output, task)
3. Rubber-duck dependency walk:
   - Who calls this interface?
   - What assumptions do callers make?
   - What breaks if this contract fails or changes?
4. Enumerate top failure modes (at least 3)
5. Define minimal probes that can falsify assumptions
6. Record evidence
7. Update risk scores only if evidence supports change
8. Update graph/interfaces if coupling changed
9. Log unresolved questions to next pass queue

**Source**: `ES-V200-Dependency-Graph-Contract-Hardening.md`

### 6.2 Pass Order Policy

Default order for V200:
1. `rust-llm-core-foundation`
2. `rust-llm-store-runtime`
3. `rust-llm-rust-semantics`
4. `rust-llm-tree-extractor`
5. `rust-llm-cross-boundaries`
6. `rust-llm-graph-reasoning`
7. `rust-llm-interface-gateway`
8. `rust-llm-test-harness`

**Source**: `ES-V200-Dependency-Graph-Contract-Hardening.md`

### 6.3 TDD Workflow

Follow STUB → RED → GREEN → REFACTOR cycle:
1. Write failing test first
2. Run test, verify failure
3. Minimal implementation to pass
4. Refactor without breaking tests

**Source**: `CLAUDE.md` (project instructions)

---

## 7. User Journeys

### 7.1 Three User Modes

```
MODE 1: QUICK LOOKUP (Tauri app sufficient)
- "Show me codebase stats"
- "Find function X"
- "What are the top 10 hotspots?"
- User clicks, sees answer, done in <30 seconds

MODE 2: EXPLORATORY ANALYSIS (Tauri → Terminal handoff)
- "I need to understand this module"
- Tauri shows overview + top entities
- User clicks "Terminal Workflow" button
- Copies 5 curl commands to terminal
- Pipes through jq, saves outputs, composes context

MODE 3: AUTOMATED/LLM WORKFLOWS (Pure CLI)
- "Run this analysis in CI on every PR"
- "Feed blast radius into Claude for review"
- "Generate architecture report weekly"
- No GUI involved — scripts + cron + LLM APIs
```

**Source**: `ES-V200-User-Journey-Addendum-Tauri-CLI-Philosophy.md`

### 7.2 Tauri App Philosophy

**The Tauri app is a visual launcher for CLI commands, not a replacement for terminal workflows.**

Think of it as:
- **VSCode's Command Palette** for Parseltongue
- **GitHub Desktop** for git (shows status, but power users live in terminal)
- **Postman Collections** (generates curl commands you copy-paste)

**Key Design Principle**: The Tauri app's job is to get users from Day 1 to Day 7. By Day 30, they don't need it.

**Source**: `ES-V200-User-Journey-Addendum-Tauri-CLI-Philosophy.md`

### 7.3 Acceptance Criteria (21 Total for Tauri)

| Journey Step | Criteria |
|--------------|----------|
| Discovery | Install < 2 min, README shows 15-sec demo |
| First Ingest | Server auto-starts, progress updates 4x/sec, <5 sec for 5K entities |
| First Query | Overview loads <200ms, shows entity/edge/token counts |
| Search | Results in <100ms, show type/path/caller count |
| Blast Radius | Compute <500ms for 2-hop queries |
| Multi-Project | Unique slug per project, unique auto-assigned port |
| Shutdown | Clean shutdown within 2 seconds, port files deleted |

**Source**: `ES-V200-User-Journey-01.md`

---

## 8. Technical Specifications

### 8.1 Key Format Implementation

**Decision**: Candidate D (Typed Rust Struct) + Candidate B (`|||` serialization)

```rust
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntityKey {
    pub language: Language,
    pub kind: EntityKind,
    pub scope: Vec<String>,
    pub name: String,
    pub file_path: String,
    pub discriminator: String,
}
```

**Display Implementation**: `rust|||fn|||my_crate.auth.handlers|||login|||src/auth.rs|||d0`

**Source**: `Prep-V200-Key-Format-Design.md`

### 8.2 Cross-Language Detection Patterns

| Pattern | Base Confidence | False Positive Rate |
|---------|-----------------|---------------------|
| FFI (raw extern) | 0.90 | 3-8% |
| FFI (CXX bridge) | 0.95 | 1-3% |
| WASM (wasm_bindgen) | 0.88 | 5-10% |
| PyO3 | 0.85 | ~12% |
| JNI | 0.88 | Varies |
| gRPC | 0.85 | 5-10% |
| HTTP | 0.65 | 10-20% |
| Message Queue | 0.70 | 15-25% |

**Source**: `Prep-V200-Cross-Language-Detection-Heuristics.md`

### 8.3 Ascent Datalog Base Relations

```rust
// Entity facts
relation entity(String, String, EntityKind, String, u32, u32);
relation edge(String, String, EdgeKind);
relation in_module(String, String);
relation is_pub(String);
relation is_async(String);
relation is_unsafe(String);
relation trait_impl(String, String);
relation supertrait(String, String);

// Cross-language facts
relation cross_lang_edge(String, String, CrossLangMechanism);
relation ffi_export(String, String, String);
relation string_literal(String, String);
```

**Source**: `Prep-V200-Datalog-Ascent-Rule-Patterns.md`

### 8.4 Derived Rules (18 Total)

| Rule | Purpose |
|------|---------|
| Transitive Trait Hierarchy | `all_supers(t, s)` via supertrait recursion |
| Unsafe Call Chain | `unsafe_chain(f)` - functions transitively reaching unsafe |
| Taint Analysis | `tainted(f)` - data flow from sources to sinks |
| Reachability | `reachable(a, b)` - full transitive closure |
| Dead Code Detection | `dead_code(f)` - unreachable from entry points |
| Layer Violation | `layer_violation(f, g)` - architectural boundary violations |
| Async Boundary | `async_boundary(f, g)` - async calling sync (blocking) |
| Circular Deps | `circular_dep(a, b)` - mutual module reachability |
| Coupling Metrics | `cbo(m)` - Coupling Between Objects via Datalog |
| API Surface | `api_surface(e)` - public entities and dependencies |
| Closure Captures | `closure_captures_unsafe(c, p)` - unsafe context captures |
| Error Propagation | `error_chain(a, b)` - Result/Option flow chains |
| Module Cohesion | `same_module_edge(e1, e2)` - internal module connections |
| Test Coverage Gap | `untested_pub_fn(f)` - public functions without tests |
| Derive Macro Inference | `derive_impl(key, trait)` - #[derive(...)] trait impls |
| God Object | `god_object(f)` - high fan-in + fan-out |
| SCC Membership | `in_scc(a, rep)` - entity group membership |
| Cross-Language Edge Join | `ffi_match(rust_key, c_key)` - FFI name matching |

**Source**: `Prep-V200-Datalog-Ascent-Rule-Patterns.md`

### 8.5 LLM Context Ranking Signals

| Signal | Weight | Purpose |
|--------|--------|---------|
| Blast Radius | 0.30 | Local relevance - distance from focus entity |
| SCC Membership | 0.20 | Mandatory co-inclusion - tightly coupled group |
| Leiden Community | 0.10 | Module cohesion - same community bonus |
| PageRank | 0.10 | Global importance - architectural pillars |
| K-Core | 0.05 | Structural depth - core vs periphery |
| CK Metrics | 0.10 | Complexity-driven need - high coupling = more context |
| Cross-Language | 0.10 | API contract relevance - FFI/WASM boundaries |
| Entropy | 0.05 | Complexity amplifier - high entropy = more context |

**Source**: `Prep-V200-LLM-Context-Optimization-Research.md`

### 8.6 Token Budgeting Algorithm

**Recommended Approach**: Hybrid (Hierarchical + Proportional)

```
1. Compute unified ranking scores for all entities
2. Identify Leiden communities; allocate budget proportionally:
   - Community with focus entity: 50%
   - Adjacent communities (1-hop): 30%
   - High-PageRank pillars: 15%
   - Reserve: 5% for cross-language + SCC completions
3. Within each community, use hierarchical phases:
   - Phase 1 (40%): Signatures of all relevant entities
   - Phase 2 (40%): Bodies of highest-ranked entities
   - Phase 3 (20%): Relationship annotations
4. Cap at 80% of stated budget (Context Rot research)
```

**Source**: `Prep-V200-LLM-Context-Optimization-Research.md`

### 8.7 rust-analyzer Bridge API (20 Methods)

| Type | Methods |
|------|---------|
| Function | `name()`, `module()`, `ret_type()`, `params_without_self()`, `is_async()`, `is_unsafe()`, `generic_params()` |
| Impl | `self_ty()`, `trait_()`, `items()`, `all_for_type()`, `all_for_trait()` |
| Type | `as_adt()`, `layout()`, `display()`, `impls_trait()` |
| Trait | `super_traits()`, `all_super_traits()`, `items()` |
| Closure | `captured_items()`, `fn_trait()` |

**Source**: `Prep-V200-Rust-Analyzer-API-Surface.md`

### 8.8 Tree-Sitter Query Extraction Targets

| Category | Languages | Captures |
|----------|-----------|----------|
| Function declarations | All 12 | @name, @visibility, @modifiers, @return_type, @params, @body |
| Class/struct/trait | All 12 | @name, @generics, @fields, @supertraits |
| Import statements | All 12 | @module_path, @imported_name, @alias |
| Attributes/decorators | Rust, Python, Java, C# | @attr_name, @attr_args |
| String literals | All 12 | @string.content (for cross-lang detection) |
| Closure/lambda | 8 langs | @param_name, @body |

**Source**: `Prep-V200-Tree-Sitter-Query-Patterns.md`

---

## 9. Risks and Mitigations

### 9.1 Risk/Unclear Matrix (Baseline)

| Crate | Risk / 5 | Unclear / 5 | Why Not Low |
|-------|----------|-------------|-------------|
| rust-llm-interface-gateway | 3 | 2 | Unified behavior across CLI/HTTP/MCP |
| rust-llm-core-foundation | 4 | 4 | Key model affects all crates |
| rust-llm-tree-extractor | 4 | 3 | 12-language query correctness |
| rust-llm-rust-semantics | 5 | 4 | RA/proc-macro/build-script reliability |
| rust-llm-cross-boundaries | 4 | 4 | Heuristic linking quality |
| rust-llm-graph-reasoning | 4 | 3 | Rule correctness/scale tradeoffs |
| rust-llm-store-runtime | 5 | 4 | Delta consistency, indexing |
| rust-llm-test-harness | 3 | 3 | Fixture breadth vs CI time |

**Source**: `ES-V200-Hashing-Risks-v01.md`

### 9.2 Key Format Risks (18 Questions)

| Area | Risk Level | Open Questions |
|------|------------|----------------|
| Scope representation | LOW | Q1: Vec<String> vs Vec<ScopeSegment> |
| Scope extraction | MEDIUM | Q12: Per-language extraction depth |
| String interning | LOW | Q2: Memory vs complexity tradeoff |
| Path normalization | LOW | Q3: One normalize_file_path() function |
| External entities | LOW-MEDIUM | Q5: EXTERNAL: prefix format |
| rust-analyzer bridge | MEDIUM | Q6: DefId-to-EntityKey mapping |
| Discriminator | LOW | Q7, Q13: Format rules defined |
| Taxonomy alignment | LOW | Q8-Q11, Q15-Q17: Documentation alignment |

**Verdict**: Key format is LOW risk overall. RFC exists, questions are bounded.

**Source**: `Prep-V200-isgl1-ambiguity-risk-table.md`

### 9.3 Competitive Risks

| Competitor | Risk | Mitigation |
|------------|------|------------|
| CodeQL | Security depth, QL expressiveness | Focus on architecture + LLM output + cross-language |
| Semgrep | Speed, OSS rule ecosystem | Graph algorithms are unique differentiator |
| Sourcegraph/SCIP | Navigation scale | Add architectural understanding on top |
| Cursor/Aider | LLM integration, embeddings | Offer as library + MCP server |
| Augment Code | Context engine | Open source + embeddable + transparent |

**Source**: `Prep-V200-Competitive-Deep-Dive.md`

---

## 10. Implementation Priorities

### 10.1 HIGH Priority (Ship in v2.0.0 Core)

| ID | Item | Crate | Effort | Depends On |
|----|------|-------|--------|------------|
| H1 | Slim graph model (151 B/entity) | rust-llm-core | 2 weeks | None |
| H2 | TypedCallEdges (pt04 Phase 1) | rust-llm-semantic-engine | 3 weeks | H1 |
| H3 | MCP subcommand (pt09) | rust-llm-mcp | 2 weeks | H1 |
| H4 | Progressive disclosure (?detail) | rust-llm-http | 2 days | H1 |
| H5 | Per-query token estimation | rust-llm-http | 2 hours | H1 |
| H6 | Shebang line handling | rust-llm-core | 1 day | None |
| H7 | Const object export .scm query | rust-llm-core | 1 day | None |
| H8 | Decorated Python fn .scm query | rust-llm-core | 1 day | None |
| H9 | Orphaned entity detection | rust-llm-graph | 1 day | H1 |
| H10 | Graph neighborhood alias | rust-llm-http | 30 min | H1 |
| H11 | MessagePack export (pt02/pt03) | rust-llm-export | 1 week | H1 |

**Source**: `Prep-V200-Compiled-Research-Best-Ideas.md`

### 10.2 MEDIUM Priority (v2.0.x Follow-ups)

| ID | Item | Crate | Effort |
|----|------|-------|--------|
| M1 | TraitImpls + SupertraitEdges | rust-llm-semantic-engine | 1 week |
| M2 | SemanticTypes (resolved sigs) | rust-llm-semantic-engine | 1 week |
| M3 | Time-based entity filtering | rust-llm-core + http | 2 days |
| M4 | Session persistence layer | rust-llm-core + http | 3 days |
| M5 | OpenTelemetry tracing | rust-llm-http | 3 days |
| M6 | CRLF normalization | rust-llm-core | 2 hours |
| M7 | Trait hierarchy endpoint | rust-llm-http | 2 days |
| M8 | Async call chain trace endpoint | rust-llm-http | 3 days |
| M9 | Visibility audit endpoint | rust-llm-http | 2 days |
| M10 | Datalog policy engine | rust-llm-core + http | 3 weeks |

**Source**: `Prep-V200-Compiled-Research-Best-Ideas.md`

### 10.3 Build Order (5 Weeks)

```
Week 1: EntityKey struct, EntityKind/EntityClass enums, Display impl
Week 2: Rust scope extractor, discriminator format rules
Week 3: Scope extraction for Python, TypeScript, Java
Week 4: rust-analyzer bridge mapping (Solution C)
Week 5: External entities, cross-lang resolver, memory measurement
```

**Source**: `Prep-V200-isgl1-ambiguity-risk-table.md`

### 10.4 Deferred to v210

| Item | Reason |
|------|--------|
| rust-llm-context-packer | V200 focuses on graph-grounded context adequacy, not token minimization |
| Token-minimization strategies | Optimization layer that can mask weak retrieval/fidelity |

**Source**: `ES-V200-Decision-log-01.md`

---

## Appendix A: Source Document Index

| Document | Lines | Key Contribution |
|----------|-------|------------------|
| PRD-v200.md | 20 | Clean break requirements |
| ES-V200-Decision-log-01.md | 60 | Lifecycle bundle decisions |
| ES-V200-Dependency-Graph-Contract-Hardening.md | 120 | Pass method definition |
| ES-V200-Hashing-Risks-v01.md | 1189 | Risk matrix, pass ledger |
| ES-V200-User-Journey-01.md | 650 | Tauri journey + acceptance criteria |
| ES-V200-User-Journey-Addendum-Tauri-CLI-Philosophy.md | 400 | Tauri as CLI launcher |
| Prep-V200-Competitive-Deep-Dive.md | 1400 | Competitor analysis |
| Prep-V200-Compiled-Research-Best-Ideas.md | 800 | Priority matrix |
| Prep-V200-Cross-Language-Detection-Heuristics.md | 1822 | 5 patterns + confidence scoring |
| Prep-V200-Datalog-Ascent-Rule-Patterns.md | 1200 | 18 derived rules |
| Prep-V200-Key-Format-Design.md | 762 | 4 candidates + recommendation |
| Prep-V200-LLM-Context-Optimization-Research.md | 1071 | 8 ranking signals |
| Prep-V200-Max-Adoption-Architecture-Strategy.md | 800 | 4 architecture options |
| Prep-V200-MCP-Protocol-Integration.md | 1100 | MCP spec + tool mapping |
| Prep-V200-Rust-Analyzer-API-Surface.md | 500 | 20 API methods |
| Prep-V200-Tree-Sitter-Query-Patterns.md | 1400 | Query syntax + patterns |
| Prep-V200-isgl1-ambiguity-risk-table.md | 800 | 18 questions + reassessment |
| v200-doc-index-01.md | 30 | Curated index |

**Total Research**: ~15,000 lines across 18 documents

---

## Appendix B: Acronym Glossary

| Term | Definition |
|------|------------|
| SCC | Strongly Connected Component |
| CBO | Coupling Between Objects |
| LCOM | Lack of Cohesion in Methods |
| RFC | Response For a Class |
| WMC | Weighted Methods per Class |
| SQALE | Software Quality Assessment based on Lifecycle Expectations |
| MCP | Model Context Protocol |
| FFI | Foreign Function Interface |
| PyO3 | Python ↔ Rust bindings |
| JNI | Java Native Interface |
| WASM | WebAssembly |
| RA | rust-analyzer |
| HIR | High-level Intermediate Representation |
| AST | Abstract Syntax Tree |
| CST | Concrete Syntax Tree |
| ISGL1 | Identity String Graph Level 1 (key format) |

---

*Generated: 2026-02-17*
*Total Source Documents: 18*
*Total Lines Analyzed: ~15,000*
