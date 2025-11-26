# Parseltongue HTTP Server PRD

**Author**: Product
**Status**: Draft
**Version**: 1.4.0
**Last Updated**: 2025-11-27

---

## The One-Liner

Parseltongue ingests a codebase into a queryable database and spins up an HTTP server that any LLM agent can query.

---

## The Problem We're Solving

**Current pain**: Developers and LLM agents cannot easily understand codebases. They resort to grep, which:
- Returns raw text (no semantic understanding)
- Uses 100x more tokens than needed
- Misses relationships between code entities
- Requires re-parsing on every query

**The insight**: Code is a graph, not text. Parse it once, store it in a database, query it forever.

---

## What We're Building

A single binary that:
1. **Ingests** a folder into a database (entities + dependencies)
2. **Spins up** an HTTP server to query that database
3. **Returns** structured JSON that any LLM can understand

```bash
parseltongue serve-http-code-backend ./my-project
# Ingesting...done. 156 entities, 870 edges.
# Server running at http://localhost:3847
#
# Add this to your agent: http://localhost:3847
# Try: curl http://localhost:3847/stats
```

That's it. No JSON exports. No file management. Just ingest → serve → query.

---

## What We're NOT Building

- ❌ JSON file exports (removed - the HTTP server replaces this)
- ❌ GUI or web interface (terminal-first)
- ❌ Code editing/modification tools (analysis only)
- ❌ Cloud/hosted service (local backend only)
- ❌ MCP protocol (HTTP is more universal)

---

## Command Specification (4-Word Naming Convention)

Per project rules: ALL commands use EXACTLY 4 words.

### Primary Command

```bash
parseltongue serve-http-code-backend <DIRECTORY> [OPTIONS]
```

**4-Word Breakdown**: `serve` + `http` + `code` + `backend`

**Options**:
```bash
--port <PORT>        # HTTP port (default: auto-detect from 3333)
--reindex            # Force fresh ingestion even if DB exists
--daemon             # Run in background mode
--timeout <MINUTES>  # Auto-shutdown after idle period
--verbose            # Show query logs
--quiet              # Minimal output
```

**Examples**:
```bash
# Basic usage (current directory)
parseltongue serve-http-code-backend .

# Specific directory with custom port
parseltongue serve-http-code-backend /path/to/project --port 8080

# Force re-ingestion
parseltongue serve-http-code-backend . --reindex

# Background mode with auto-shutdown
parseltongue serve-http-code-backend . --daemon --timeout 60
```

---

## Languages Supported

**12 Languages via Tree-Sitter**:

| Language | File Extensions | Entity Types Extracted |
|----------|-----------------|----------------------|
| **Rust** | `.rs` | fn, struct, enum, trait, impl, mod |
| **Python** | `.py` | def, class, async def |
| **JavaScript** | `.js`, `.jsx` | function, class, arrow functions |
| **TypeScript** | `.ts`, `.tsx` | function, class, interface, type |
| **Go** | `.go` | func, type, struct, interface |
| **Java** | `.java` | class, interface, method, enum |
| **C** | `.c`, `.h` | function, struct, typedef |
| **C++** | `.cpp`, `.cc`, `.hpp` | function, class, struct, template |
| **Ruby** | `.rb` | def, class, module |
| **PHP** | `.php` | function, class, trait |
| **C#** | `.cs` | class, struct, interface, method |
| **Swift** | `.swift` | func, class, struct, protocol |

---

## ISGL Level Taxonomy (4-Word Hyphenated Names)

**ISGL** = Information Semantic Graph Level. Each level provides progressively more detail about the codebase.

| ISGL | 4-Word Name (Hyphenated) | What It Answers | Token Budget |
|------|--------------------------|-----------------|--------------|
| `ISGL0` | `Who-Calls-Who-Graph` | "What connects to what?" | ~3K tokens |
| `ISGL0.5` | `Smart-Module-Grouping-Level` | "What logical groups exist?" | ~5K tokens |
| `ISGL1` | `Function-Signature-Overview` | "What functions exist?" | ~30K tokens |
| `ISGL2` | `Complete-Type-Detail-Level` | "What are all the types?" | ~60K tokens |
| `ISGL3` | `Full-Source-Code-Level` | "What is the actual code?" | ~500K tokens |
| `ISGL4` | `Folder-File-Organization` | "How is code organized?" | ~10K tokens |

### Naming Convention by Context

| Context | Convention | Example |
|---------|------------|---------|
| Documentation/ISGL names | Hyphenated | `Who-Calls-Who-Graph` |
| HTTP URLs | Hyphenated | `/who-calls-who-graph` |
| Rust files | snake_case | `who_calls_who_graph.rs` |
| Rust modules | snake_case | `mod who_calls_who_graph;` |
| DB tables | PascalCase | `CodeProductionEntityStore` |
| Handler functions | snake_case | `handle_who_calls_who_graph()` |

### Why Hyphenated Names

1. **Readable** - `Who-Calls-Who-Graph` vs `WhoCallsWhoGraph`
2. **Self-documenting** - Each word clearly separated
3. **LLM-friendly** - Better tokenization with explicit boundaries
4. **URL-compatible** - Already standard for REST endpoints

---

## User Jobs To Be Done

| Job | How Parseltongue Helps |
|-----|------------------------|
| "I want to understand this codebase" | Ingest → query `/stats`, `/hotspots` |
| "What calls this function?" | Query `/callers/{entity}` |
| "What breaks if I change X?" | Query `/blast/{entity}?hops=3` |
| "Are there circular dependencies?" | Query `/cycles` |
| "Find functions related to payment" | Query `/search?q=payment` |

---

## User Journey: Analyzing an Open Source Codebase

### Step 1: Run Parseltongue

```bash
$ cd ~/code/some-open-source-project
$ parseltongue serve-http-code-backend .
```

### Step 2: Watch Ingestion Happen

```
Parseltongue v1.1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Ingesting: /Users/dev/code/some-open-source-project

Phase 1: Scanning
  ✓ 234 files found
  ✗ 12 skipped (target/, node_modules/, .git/, __pycache__/)

Phase 2: Parsing (12 languages supported)
  ✓ Rust:       89 files  →  456 entities
  ✓ Python:     45 files  →  123 entities
  ✓ TypeScript: 100 files →  234 entities

Phase 3: Dependency Extraction (CPG-Inspired Edge Types)
  ✓ 2,541 edges
    Calls: 1,890 | Uses: 345 | Implements: 106 | Extends: 45 | Contains: 155

Phase 4: Classification
  ✓ CODE entities: 591
  ✓ TEST entities: 222 (excluded from default queries)

Database saved to:
  ./parseltongue_20251126193045/analysis.db
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Step 3: Server Starts Automatically

```
HTTP Server running at: http://localhost:3847

┌─────────────────────────────────────────────────────────────────────┐
│  Add this to your LLM agent configuration:                          │
│                                                                     │
│  PARSELTONGUE_URL=http://localhost:3847                             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘

Quick test:
  curl http://localhost:3847/stats

Available endpoints:
  GET /stats              - Codebase statistics
  GET /entities           - List all entities
  GET /callers/{name}     - Who calls this?
  GET /callees/{name}     - What does this call?
  GET /blast/{name}       - Impact analysis
  GET /cycles             - Circular dependencies
  GET /hotspots           - Complexity ranking
  GET /clusters           - Semantic module groupings (LPA)
  GET /search?q=pattern   - Search entities
  GET /help               - Full API reference

Press Ctrl+C to stop server
```

### Step 4: Agent Queries the Server

The LLM agent (Claude Code, GPT, Cursor, etc.) now knows to query `http://localhost:3847`:

```
User: "What are the most complex functions in this codebase?"

Agent internally runs:
  curl http://localhost:3847/hotspots?top=5

Server responds:
  {
    "hotspots": [
      {"name": "process_request", "deps": 34, "file": "src/handler.rs"},
      {"name": "parse_config", "deps": 28, "file": "src/config.rs"},
      ...
    ]
  }

Agent responds:
  "The most complex functions are process_request (34 dependencies)
   and parse_config (28 dependencies). Would you like me to analyze
   either of these?"
```

---

## User Journey: Returning to a Previously Analyzed Codebase

### Step 1: Run Parseltongue Again

```bash
$ cd ~/code/some-open-source-project
$ parseltongue serve-http-code-backend .
```

### Step 2: Existing Database Detected

```
Parseltongue v1.1.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Found existing database:
  ./parseltongue_20251126193045/analysis.db
  Created: 2 hours ago
  Entities: 591 | Edges: 2,341

Loading... done (0.3s)

HTTP Server running at: http://localhost:3847

(Use --reindex to force fresh ingestion)
```

No re-indexing. Instant startup. Same port management.

---

## Technical Architecture

### Folder Structure

```
./my-project/
├── src/
├── Cargo.toml
└── parseltongue_20251126193045/     ← Created by parseltongue
    ├── analysis.db                   ← RocksDB + CozoDB
    └── README.md                     ← Auto-generated API reference
```

**Why timestamped folder?**
- Multiple analysis snapshots possible
- Clear provenance (when was this indexed?)
- No conflicts with project files

### Database Schema (14 Tables - 4-Word Naming Convention)

**Key Design Decision**: CODE and TEST entities are stored in **separate tables** for:
- Cleaner queries (no `WHERE entity_class='CODE'` needed)
- 75% token savings when excluding test entities
- Separate edge graphs for CODE→CODE vs TEST→TEST vs TEST→CODE

#### Table Summary

| # | Table Name (4 words) | Purpose |
|---|---------------------|---------|
| 1 | `CodeProductionEntityStore` | Production code entities |
| 2 | `TestImplementationEntityStore` | Test code entities |
| 3 | `CodeDependencyEdgeGraph` | CODE→CODE dependencies |
| 4 | `TestDependencyEdgeGraph` | TEST→TEST dependencies |
| 5 | `TestToCodeEdgeBridge` | TEST→CODE coverage links |
| 6 | `EntityComputedMetricsCache` | Pre-computed entity stats |
| 7 | `GraphGlobalStatisticsStore` | Global codebase stats |
| 8 | `SemanticClusterDefinitionStore` | ISGL0.5 cluster definitions |
| 9 | `EntityClusterMembershipMap` | Entity→Cluster mapping |
| 10 | `FileEntityMappingIndex` | Package graph (ISGL4) |
| 11 | `ModuleCohesionMetricsCache` | Module quality metrics |
| 12 | `OrphanDeadCodeCache` | Dead code detection cache |
| 13 | `ControlFlowPathAnalysis` | Critical path analysis |
| 14 | `TemporalCouplingEdgeStore` | Git-derived hidden dependencies |

#### Core Entity Tables (1-2)

```datalog
:create CodeProductionEntityStore {
    ISGL1_key: String =>           # "rust:fn:process:src_handler_rs:45-89"
    entity_name: String,
    entity_type: String,           # "function", "struct", "trait", "impl"
    visibility: String,            # "public", "private", "crate"
    file_path: String,
    line_start: Int,
    line_end: Int,
    language: String,
    module_path: String,           # "crate::module::submodule"
    current_code: String?,
    future_code: String?,
    interface_signature: String,   # JSON
    current_ind: Bool,
    future_ind: Bool,
    future_action: String?,
    doc_comment: String?,
    tdd_classification: String,
    content_hash: String,
    created_at: String,
    modified_at: String
}

:create TestImplementationEntityStore {
    ISGL1_key: String =>
    entity_name: String,
    entity_type: String,           # "test_function", "test_module"
    test_kind: String,             # "unit", "integration", "benchmark"
    file_path: String,
    line_start: Int,
    line_end: Int,
    language: String,
    module_path: String,
    current_code: String?,
    interface_signature: String,
    current_ind: Bool,
    future_ind: Bool,
    future_action: String?,
    test_attributes: String?,      # JSON: #[test], #[tokio::test]
    tested_entity_key: String?,    # Link to CodeEntities
    assertion_count: Int?,
    content_hash: String,
    created_at: String,
    modified_at: String
}
```

#### Edge Tables (3-5)

```datalog
:create CodeDependencyEdgeGraph {
    from_key: String,
    to_key: String,
    edge_type: String              # "Calls", "Uses", "Implements", "Extends", "Contains"
    =>
    direction: String,             # "downward", "upward", "horizontal"
    source_location: String?,
    weight: Float?,
    created_at: String
}

:create TestDependencyEdgeGraph {
    from_key: String,
    to_key: String,
    edge_type: String
    =>
    source_location: String?,
    created_at: String
}

:create TestToCodeEdgeBridge {
    test_key: String,
    code_key: String,
    edge_type: String              # "Tests", "Calls", "Mocks"
    =>
    coverage_type: String?,        # "direct", "indirect"
    source_location: String?,
    created_at: String
}
```

#### Metrics Tables (6-7)

```datalog
:create EntityComputedMetricsCache {
    ISGL1_key: String =>
    entity_class: String,          # "CODE" or "TEST"
    outgoing_count: Int,
    incoming_count: Int,
    transitive_count: Int,
    cyclomatic_complexity: Float?,
    cognitive_complexity: Float?,
    loc: Int?,
    unwrap_count: Int?,
    clone_count: Int?,
    unsafe_count: Int?,
    hotspot_score: Float,
    computed_at: String
}

:create GraphGlobalStatisticsStore {
    metric_name: String =>
    metric_value: Float,
    entity_class: String?,         # null = all, "CODE", "TEST"
    computed_at: String
}
```

#### Semantic Cluster Tables (8-9) - ISGL0.5

```datalog
:create SemanticClusterDefinitionStore {
    cluster_id: String =>
    cluster_name: String,          # "export_pipeline", "db_adapter"
    purpose: String?,              # LLM-inferred purpose
    entity_class: String,
    size: Int,
    cohesion_score: Float,         # Internal connectivity
    coupling_score: Float,         # External dependencies
    parent_cluster_id: String?,    # Hierarchy support
    depth: Int,
    algorithm: String,             # "louvain", "lpa"
    created_at: String
}

:create EntityClusterMembershipMap {
    ISGL1_key: String,
    cluster_id: String
    =>
    membership_score: Float?,
    is_bridge: Bool?,              # Connects multiple clusters
    role: String?                  # "core", "boundary", "bridge"
}
```

#### Analysis Tables (10-13)

```datalog
:create FileEntityMappingIndex {
    file_path: String =>
    entity_count: Int,
    code_entity_count: Int,
    test_entity_count: Int,
    total_loc: Int?,
    language: String,
    module_path: String?,
    last_modified: String
}

:create ModuleCohesionMetricsCache {
    scope: String,                 # file_path or cluster_id
    scope_type: String             # "file", "cluster", "module"
    =>
    internal_edges: Int,
    external_edges: Int,
    cohesion_ratio: Float,         # internal / (internal + external)
    instability: Float?,           # external_out / (external_in + external_out)
    computed_at: String
}

:create OrphanDeadCodeCache {
    ISGL1_key: String =>
    entity_type: String,
    file_path: String,
    reason: String,                # "no_callers", "unused_export", "dead_branch"
    detected_at: String
}

:create ControlFlowPathAnalysis {
    path_id: String =>
    start_entity: String,
    end_entity: String,
    path_length: Int,
    path_entities: String,         # JSON array of ISGL1 keys
    path_type: String,             # "critical", "hot", "dead"
    importance_score: Float?,
    computed_at: String
}
```

#### Temporal Coupling Table (14) - Git-Derived Hidden Dependencies

```datalog
:create TemporalCouplingEdgeStore {
    entity_a: String,
    entity_b: String =>
    co_change_count: Int,           # Times changed together
    coupling_score: Float,          # Normalized 0.0-1.0
    time_window_days: Int,          # Analysis window (default: 180)
    first_co_change: String,        # RFC3339
    last_co_change: String,         # RFC3339
    computed_at: String
}
```

**Why Temporal Coupling Matters**: Static analysis sees code dependencies. Temporal coupling reveals **invisible architecture** - files that always change together but have ZERO code dependency.

Example:
```
auth.rs ↔ config.yaml (changed together 47 times, ZERO code dependency)
```
This reveals missing abstractions, implicit contracts, and true change impact.

### Edge Types (CPG-Inspired)

| Edge Type | Direction | Meaning | Example |
|-----------|-----------|---------|---------|
| `Calls` | downward | Function invocation | `main` → `process` |
| `Uses` | downward | Type/constant reference | `handler` → `Config` |
| `Implements` | upward | Trait implementation | `MyStruct` → `Trait` |
| `Extends` | upward | Inheritance/extension | `Child` → `Parent` |
| `Contains` | downward | Structural containment | `module` → `function` |

**Direction Semantics**:
- `"downward"` - Caller→Callee, User→Provider, Container→Contained
- `"upward"` - Implementor→Trait, Child→Parent
- `"horizontal"` - Peer relationships (rare)

### Port Management

The server picks an available port automatically:

```rust
// Pseudocode
let port = find_available_port_number(starting_from: 3333);
println!("Server running at: http://localhost:{}", port);
```

**Why auto-detect port?**
- User might run multiple instances (different projects)
- No port conflicts
- Server prints exact URL to use

### The Port-Instance Association

Each `parseltongue serve-http-code-backend` session:
1. Creates/loads a database in `parseltongue_yymmddhhss/`
2. Binds to an available port
3. That port serves ONLY that database
4. Ctrl+C kills server, database persists

```
Terminal 1: parseltongue serve-http-code-backend ./project-a  → localhost:3333 → project-a DB
Terminal 2: parseltongue serve-http-code-backend ./project-b  → localhost:3334 → project-b DB
```

---

## HTTP API Specification

### Endpoint Handler Functions (4-Word Hyphenated URLs)

Per project rules: ALL endpoints use hyphenated 4-word URLs for LLM readability.

#### Core Endpoints (Hyphenated URLs)

| Endpoint | Handler Function | Description |
|----------|-----------------|-------------|
| `GET /server-health-check-status` | `handle_server_health_check_status()` | Server status |
| `GET /codebase-statistics-overview-summary` | `handle_codebase_statistics_overview_summary()` | Codebase statistics |
| `GET /code-entities-list-all` | `handle_code_entities_list_all()` | All entities |
| `GET /code-entity-detail-view/{key}` | `handle_code_entity_detail_view()` | Single entity |
| `GET /dependency-edges-list-all` | `handle_dependency_edges_list_all()` | All dependency edges |
| `GET /reverse-callers-query-graph/{entity}` | `handle_reverse_callers_query_graph()` | Reverse dependencies |
| `GET /forward-callees-query-graph/{entity}` | `handle_forward_callees_query_graph()` | Forward dependencies |
| `GET /blast-radius-impact-analysis/{entity}` | `handle_blast_radius_impact_analysis()` | Transitive impact |
| `GET /circular-dependency-detection-scan` | `handle_circular_dependency_detection_scan()` | Circular dependencies |
| `GET /complexity-hotspots-ranking-view` | `handle_complexity_hotspots_ranking_view()` | Complexity ranking |
| `GET /fuzzy-entity-search-query` | `handle_fuzzy_entity_search_query()` | Fuzzy search |
| `GET /semantic-cluster-grouping-list` | `handle_semantic_cluster_grouping_list()` | Semantic module groupings |
| `GET /api-reference-documentation-help` | `handle_api_reference_documentation_help()` | API documentation |

#### New Endpoints (v1.4.0 - High-ROI Features)

| Endpoint | Handler Function | Description | Source Table |
|----------|-----------------|-------------|--------------|
| `GET /orphan-dead-code-detection` | `handle_orphan_dead_code_detection()` | Dead code detection | `OrphanDeadCodeCache` |
| `GET /package-file-structure-graph` | `handle_package_file_structure_graph()` | Package graph (ISGL4) | `FileEntityMappingIndex` |
| `GET /entity-complexity-metrics-cache` | `handle_entity_complexity_metrics_cache()` | Pre-computed metrics | `EntityComputedMetricsCache` |
| `GET /module-cohesion-quality-score` | `handle_module_cohesion_quality_score()` | Module quality metrics | `ModuleCohesionMetricsCache` |
| `GET /critical-control-flow-paths` | `handle_critical_control_flow_paths()` | Critical path analysis | `ControlFlowPathAnalysis` |
| `GET /semantic-cluster-detail-view/{id}` | `handle_semantic_cluster_detail_view()` | Cluster with members | `SemanticClusterDefinitionStore` |

#### Killer Feature Endpoints (v1.4.0)

| Endpoint | Handler Function | Description | LOC |
|----------|-----------------|-------------|-----|
| `GET /temporal-coupling-hidden-deps/{entity}` | `handle_temporal_coupling_hidden_deps()` | Git-derived hidden dependencies | ~1200 |
| `GET /smart-context-token-budget?focus=X&tokens=N` | `handle_smart_context_token_budget()` | Dynamic context selection | ~1500 |

---

## Killer Features (v1.4.0)

### The Shreyas Frame

> *"What's the job the customer is hiring you for?"*
>
> Not "parse my code." Not "build a graph." Not "detect clusters."
>
> **"Give my LLM exactly what it needs to reason about this code, and nothing else."**

---

### KILLER FEATURE #1: Temporal-Coupling-Detection (~1200 LOC)

**What it reveals**: The INVISIBLE architecture.

Static analysis sees:
```
auth.rs → session.rs (Calls edge)
```

Temporal coupling reveals:
```
auth.rs ↔ config.yaml (changed together 47 times, ZERO code dependency)
auth.rs ↔ middleware.rs (changed together 31 times, indirect edge)
```

**This is information no amount of tree-sitter parsing will ever find.**

#### Implementation (Git Log Parsing)

```bash
git log --name-only --pretty=format:"%H" --since="6 months ago" \
  | parse commit boundaries \
  | count co-occurrences \
  | normalize to coupling_score
```

#### Endpoint: `/temporal-coupling-hidden-deps/{entity}`

```json
{
  "success": true,
  "entity": "auth.rs",
  "hidden_dependencies": [
    {"file": "config.yaml", "co_changes": 47, "score": 0.92, "code_edge": false},
    {"file": "middleware.rs", "co_changes": 31, "score": 0.78, "code_edge": true}
  ],
  "insight": "config.yaml has NO code dependency but HIGH temporal coupling - missing abstraction?"
}
```

#### Compounding Effect

| What It Unlocks | Why It Matters |
|-----------------|----------------|
| **Better clusters** | Multi-signal affinity: 63% → 91% accuracy |
| **True blast radius** | `/blast/auth` now includes `config.yaml` |
| **Change prediction** | "If you touch X, you'll probably touch Y" |
| **Hidden tech debt** | High temporal + zero code edge = missing abstraction |

---

### KILLER FEATURE #2: Dynamic-Context-Token-Budget (~1500 LOC)

**The culmination of everything.**

You now have:
- Static edges (code dependencies)
- Clusters (semantic groupings)
- Temporal coupling (hidden dependencies)

**But you're still making the LLM do the work.**

Dynamic context selection flips the model:

```
Input:  focus=auth, budget=4000 tokens
Output: Exactly the 4000 most relevant tokens for reasoning about auth
```

Not "here's everything related to auth" (50K tokens).
Not "here's the auth function only" (200 tokens, missing context).
**Exactly** what the LLM needs. Nothing more.

#### The Algorithm (Greedy Knapsack)

```python
def select_context(focus_entity, token_budget):
    # 1. Start with focus entity
    context = [focus_entity]

    # 2. Score all reachable entities by:
    for entity in reachable_entities(focus_entity):
        entity.score = (
            dependency_distance_score(entity, focus_entity) * 0.3 +
            temporal_coupling_score(entity, focus_entity) * 0.25 +
            cluster_co_membership_score(entity, focus_entity) * 0.25 +
            centrality_score(entity) * 0.2  # Is this a hub?
        )

    # 3. Greedily add highest-scored until budget exhausted
    for entity in sorted_by_score(reachable_entities):
        if current_tokens + entity.tokens <= token_budget:
            context.append(entity)

    # 4. Return ordered context with reasoning hints
    return ContextResponse(
        entities=context,
        edges=relevant_edges(context),
        cluster=primary_cluster(context),
        reasoning_hints=generate_hints(context)
    )
```

#### Endpoint: `/smart-context-token-budget?focus=X&tokens=N`

```json
{
  "success": true,
  "focus": "auth",
  "token_budget": 4000,
  "tokens_used": 3850,
  "selection_strategy": "greedy_knapsack_multi_signal",
  "context": {
    "core_entities": [
      {"key": "rust:fn:authenticate", "tokens": 450, "score": 0.95, "reason": "focus"},
      {"key": "rust:fn:validate_token", "tokens": 380, "score": 0.89, "reason": "direct_caller"},
      {"key": "rust:struct:Session", "tokens": 220, "score": 0.85, "reason": "cluster_co_member"}
    ],
    "supporting_edges": [...],
    "cluster": "auth_flow",
    "temporal_hints": [
      "config.yaml changes with auth 47 times - check if config change needed"
    ],
    "blast_radius_summary": "14 entities affected by changes to auth"
  }
}
```

#### Why It's THE Feature

| Without It | With It |
|------------|---------|
| Agent queries multiple endpoints, manually assembles | Agent queries one endpoint, gets perfect context |
| Token budget guesswork | Token budget respected |
| LLM reasons over noise | LLM reasons over signal |

**Why it's the moat**: "I asked for auth context, I got exactly what an LLM needs to reason about auth, and nothing else."

---

### Core Endpoints

| Endpoint | Description | Example Response |
|----------|-------------|------------------|
| `GET /stats` | Codebase overview | `{"entities": 591, "edges": 2341, "functions": 456}` |
| `GET /entities` | All entities | `[{"key": "rust:fn:...", "name": "process", ...}]` |
| `GET /entities?name=X` | Filter by name | Entities matching pattern |
| `GET /entities?type=function` | Filter by type | Only functions |
| `GET /edges` | All dependency edges | `[{"from": "...", "to": "...", "type": "Calls", "direction": "downward"}]` |

### Graph Query Endpoints

| Endpoint | Description | Use Case |
|----------|-------------|----------|
| `GET /callers/{entity}` | Reverse dependencies | "What calls this?" |
| `GET /callees/{entity}` | Forward dependencies | "What does this call?" |
| `GET /blast/{entity}?hops=N` | Transitive impact | "What breaks if I change this?" |
| `GET /cycles` | Circular dependencies | "Are there architectural problems?" |
| `GET /hotspots?top=N` | Complexity ranking | "Where is the complexity?" |
| `GET /clusters` | Semantic module groupings | "What logical modules exist?" |
| `GET /clusters?entity=X` | Cluster for entity | "What module does X belong to?" |
| `GET /search?q=pattern` | Fuzzy search | "Find anything related to X" |

### Response Format

All responses are JSON with consistent structure:

```json
{
  "success": true,
  "endpoint": "/callers/process",
  "count": 5,
  "data": [...],
  "tokens": 234,
  "query_time_ms": 12
}
```

The `tokens` field helps LLMs understand context budget impact.

---

## What the README Should Contain

The README (shown after ingestion) teaches the LLM how to query:

```markdown
# Parseltongue API Quick Reference

Base URL: http://localhost:{PORT}

## Common Queries

### "What functions exist?"
curl http://localhost:{PORT}/entities?type=function

### "What calls X?"
curl http://localhost:{PORT}/callers/{function_name}

### "What does X depend on?"
curl http://localhost:{PORT}/callees/{function_name}

### "What breaks if I change X?"
curl http://localhost:{PORT}/blast/{function_name}?hops=3

### "Are there circular dependencies?"
curl http://localhost:{PORT}/cycles

### "Where is the complexity?"
curl http://localhost:{PORT}/hotspots?top=10

### "Find functions related to payment"
curl http://localhost:{PORT}/search?q=payment
```

This README can be:
- Printed to terminal after startup
- Saved to `parseltongue_yymmddhhss/README.md`
- Fetched via `GET /help` endpoint

---

## What Gets Removed

### Crates/Commands to Delete

| Crate | Current Command | Reason for Removal |
|-------|-----------------|-------------------|
| `pt02-llm-cozodb-to-context-writer` | `parseltongue pt02-llm-cozodb-to-context level00/01/02` | Replaced by HTTP endpoints |

### Features to Delete

| Feature | Files Affected | Reason |
|---------|---------------|--------|
| JSON export | `pt02/src/exporters/*.rs` | HTTP responses replace file exports |
| TOON export | `parseltongue-core/src/serializers/toon.rs` | No longer needed |
| `--output` flag | CLI argument parsing | No file output |
| `--include-code` flag | Export logic | Handled by endpoint selection |
| `--where-clause` flag | Query filtering | Replaced by query parameters |

### Crates to Keep (Modified)

| Crate | Modification |
|-------|--------------|
| `pt01-folder-to-cozodb-streamer` | Logic absorbed into `serve-http-code-backend` |
| `pt07-visual-analytics-terminal` | Keep as separate utility |
| `parseltongue-core` | Keep storage, entities; remove file serializers |

---

## Implementation Plan (TDD-First)

Per project rules: STUB → RED → GREEN → REFACTOR for every feature.

### Phase 1: Core Server (Week 1)

**Test-First Specifications**:

```rust
#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    // GIVEN: Server running
    let server = create_test_server_instance().await;

    // WHEN: GET /health
    let response = server.get("/health").await;

    // THEN: Returns 200 with status "ok"
    assert_eq!(response.status(), 200);
    assert_eq!(response.json()["status"], "ok");
}

#[tokio::test]
async fn test_stats_returns_entity_count() {
    // GIVEN: Server with ingested codebase
    let server = create_test_server_with_data().await;

    // WHEN: GET /stats
    let response = server.get("/stats").await;

    // THEN: Returns entity and edge counts
    assert!(response.json()["data"]["entities"].as_u64().unwrap() > 0);
}
```

**Tasks**:
- [ ] HTTP server with axum (`create_http_server_instance()`)
- [ ] `/health` endpoint (`handle_health_check_request()`)
- [ ] `/stats` endpoint (`handle_stats_overview_request()`)
- [ ] Startup ingestion (integrate pt01 logic)
- [ ] Auto port detection (`find_available_port_number()`)

### Phase 2: Query Endpoints (Week 2)

**Test-First Specifications**:

```rust
#[tokio::test]
async fn test_callers_returns_reverse_deps() {
    // GIVEN: Entity "process" exists with callers
    let server = create_test_server_with_data().await;

    // WHEN: GET /callers/process
    let response = server.get("/callers/process").await;

    // THEN: Returns list of callers
    assert!(response.json()["success"].as_bool().unwrap());
    assert!(response.json()["count"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn test_blast_radius_bounded_by_hops() {
    // GIVEN: Entity with deep dependency tree
    let server = create_test_server_with_data().await;

    // WHEN: GET /blast/entity?hops=2
    let response = server.get("/blast/entity?hops=2").await;

    // THEN: No entity beyond depth 2
    for item in response.json()["data"].as_array().unwrap() {
        assert!(item["depth"].as_u64().unwrap() <= 2);
    }
}
```

**Tasks**:
- [ ] `/entities` endpoint (`handle_entities_list_request()`)
- [ ] `/edges` endpoint (`handle_edges_list_request()`)
- [ ] `/callers/{entity}` (`handle_callers_query_request()`)
- [ ] `/callees/{entity}` (`handle_callees_query_request()`)
- [ ] `/blast/{entity}` (`handle_blast_radius_request()`)
- [ ] `/cycles` (`handle_cycles_detection_request()`)
- [ ] `/hotspots` (`handle_hotspots_analysis_request()`)
- [ ] `/clusters` (`handle_clusters_query_request()`) - wraps pt08 LPA clustering
- [ ] `/search` (`handle_search_entities_request()`)

### Phase 3: Polish (Week 3)

- [ ] `--reindex` flag (`force_reindex_codebase_flag()`)
- [ ] `--daemon` mode (`run_background_daemon_mode()`)
- [ ] `--port` override (`override_default_port_number()`)
- [ ] `/help` endpoint with full API reference
- [ ] Error handling and edge cases
- [ ] README.md generation in database folder

### Phase 4: Remove Dead Code

- [ ] Delete `pt02-llm-cozodb-to-context-writer` crate
- [ ] Delete `serializers/toon.rs`
- [ ] Remove JSON export code from pt02
- [ ] Update all documentation

---

## Performance Contracts (Executable Specifications)

### Query Latency Contract

```rust
#[tokio::test]
async fn test_query_latency_under_100ms() {
    let server = create_server_with_1000_entities().await;

    let start = std::time::Instant::now();
    let _ = server.get("/entities").await;
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(100),
            "Query took {:?}, expected <100ms", elapsed);
}
```

### Startup Time Contract (Existing DB)

```rust
#[tokio::test]
async fn test_startup_existing_db_under_1s() {
    create_test_database_file().await;

    let start = std::time::Instant::now();
    let _ = start_server_existing_database().await;
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_secs(1),
            "Startup took {:?}, expected <1s", elapsed);
}
```

---

## Success Metrics

| Metric | Target | Why It Matters |
|--------|--------|----------------|
| Time to first query | < 30 seconds | Fast enough to not interrupt flow |
| Query latency | < 100ms | Real-time feel |
| Restart time (existing DB) | < 1 second | No re-indexing penalty |
| Token efficiency vs grep | 50x reduction | LLM context budget |
| Agent compatibility | 100% HTTP clients | Universal access |

---

## Key Decisions

### Decision 1: No JSON Exports
**Rationale**: The HTTP server IS the export. Generating files that agents then read is an unnecessary step. Query directly.

### Decision 2: One Port Per Project
**Rationale**: Simple mental model. Each terminal running `parseltongue serve-http-code-backend` is one project, one port, one database.

### Decision 3: Auto-Detect Port
**Rationale**: User shouldn't think about ports. We pick one, we tell them what it is, they use it.

### Decision 4: Timestamped Folders Stay
**Rationale**: Clear provenance. User can have multiple snapshots. Doesn't pollute project root.

### Decision 5: HTTP Over MCP
**Rationale**: HTTP works with every agent (Claude Code, GPT, Cursor, scripts). MCP only works with Claude Desktop.

### Decision 6: 4-Word Naming Convention
**Rationale**: Per project rules - LLM tokenization optimized. `serve-http-code-backend` is self-documenting.

### Decision 7: TDD-First Implementation
**Rationale**: Per project rules - STUB → RED → GREEN → REFACTOR. Every feature has tests before code.

---

## Open Questions

1. **Multiple databases?** Should one server be able to query multiple project databases?
   - Current answer: No. One server, one project. Keep it simple.

2. **Incremental re-indexing?** When files change, update only changed entities?
   - Current answer: Future enhancement. V1 does full reindex with `--reindex`.

3. **Authentication?** Should the server require auth?
   - Current answer: No. Localhost only. If you have localhost access, you own the machine.

---

## Appendix: Migration from Current CLI

### What Gets Removed

```bash
# These commands go away:
parseltongue pt02-llm-cozodb-to-context level00 --output edges.json ...
parseltongue pt02-llm-cozodb-to-context level01 --output entities.json ...
parseltongue pt02-llm-cozodb-to-context level02 --output types.json ...

# These files are no longer created:
edges.json
edges_test.json
entities.json
entities_test.json
```

### What Replaces Them

```bash
# This replaces everything:
parseltongue serve-http-code-backend ./project

# Query via HTTP instead of reading files:
curl http://localhost:3847/edges      # replaces edges.json
curl http://localhost:3847/entities   # replaces entities.json
```

---

## Appendix: Full Endpoint Examples

### GET /stats

```bash
curl http://localhost:3847/stats
```

```json
{
  "success": true,
  "data": {
    "entities": 591,
    "edges": 2541,
    "by_entity_type": {"function": 456, "struct": 78, "trait": 23},
    "by_edge_type": {"Calls": 1890, "Uses": 345, "Implements": 106, "Extends": 45, "Contains": 155},
    "by_language": {"rust": 456, "typescript": 234, "python": 123},
    "db_path": "parseltongue_20251126/analysis.db"
  },
  "tokens": 89
}
```

### GET /callers/{entity}

```bash
curl http://localhost:3847/callers/process
```

```json
{
  "success": true,
  "entity": "process",
  "count": 2,
  "data": [
    {"from": "rust:fn:main:src_main_rs:10", "edge_type": "Calls", "direction": "downward"},
    {"from": "rust:fn:handle:src_api_rs:34", "edge_type": "Calls", "direction": "downward"}
  ],
  "tokens": 120
}
```

### GET /clusters

```bash
curl http://localhost:3847/clusters
```

```json
{
  "success": true,
  "endpoint": "/clusters",
  "count": 5,
  "data": [
    {
      "cluster_id": 1,
      "name": "core-processing",
      "members": ["rust:fn:process", "rust:fn:validate", "rust:fn:transform"],
      "cohesion_score": 0.87
    },
    {
      "cluster_id": 2,
      "name": "api-handlers",
      "members": ["rust:fn:handle", "rust:fn:route", "rust:fn:respond"],
      "cohesion_score": 0.72
    }
  ],
  "algorithm": "LPA",
  "tokens": 250
}
```

### GET /blast/{entity}?hops=N

```bash
curl "http://localhost:3847/blast/process?hops=3"
```

```json
{
  "success": true,
  "entity": "process",
  "max_hops": 3,
  "total_affected": 14,
  "data": [
    {"entity": "main", "depth": 1},
    {"entity": "handle", "depth": 1},
    {"entity": "cli", "depth": 2}
  ],
  "tokens": 200
}
```

### GET /cycles

```bash
curl http://localhost:3847/cycles
```

```json
{
  "success": true,
  "cycle_count": 2,
  "data": [
    {"cycle_id": 1, "entities": ["parser", "lexer", "parser"], "length": 2}
  ],
  "tokens": 150
}
```

### GET /hotspots?top=N

```bash
curl "http://localhost:3847/hotspots?top=5"
```

```json
{
  "success": true,
  "data": [
    {"name": "process", "file": "src/handler.rs", "deps": 34, "risk": "HIGH"},
    {"name": "parse", "file": "src/parser.rs", "deps": 28, "risk": "MEDIUM"}
  ],
  "tokens": 180
}
```

---

## Appendix: New v1.3.0 Endpoint Examples

### GET /orphans (Dead Code Detection)

```bash
curl http://localhost:3847/orphans
```

```json
{
  "success": true,
  "endpoint": "/orphans",
  "count": 12,
  "data": [
    {
      "key": "rust:fn:legacy_export:src_old_rs:45",
      "reason": "no_callers",
      "type": "function",
      "file": "src/old.rs"
    },
    {
      "key": "rust:struct:OldConfig:src_config_rs:20",
      "reason": "unused_export",
      "type": "struct",
      "file": "src/config.rs"
    }
  ],
  "tokens": 200
}
```

### GET /files (Package Graph - ISGL4)

```bash
curl http://localhost:3847/files
```

```json
{
  "success": true,
  "endpoint": "/files",
  "count": 45,
  "data": [
    {
      "file_path": "src/handler.rs",
      "entity_count": 12,
      "code_entities": 10,
      "test_entities": 2,
      "loc": 450,
      "language": "rust"
    },
    {
      "file_path": "src/parser.rs",
      "entity_count": 8,
      "code_entities": 8,
      "test_entities": 0,
      "loc": 320,
      "language": "rust"
    }
  ],
  "tokens": 350
}
```

### GET /complexity?top=N (Pre-computed Metrics)

```bash
curl "http://localhost:3847/complexity?top=5"
```

```json
{
  "success": true,
  "endpoint": "/complexity",
  "data": [
    {
      "key": "rust:fn:process:src_handler_rs:45",
      "hotspot_score": 0.92,
      "incoming_count": 15,
      "outgoing_count": 8,
      "cyclomatic_complexity": 12,
      "risk_indicators": {
        "unwrap_count": 3,
        "clone_count": 2,
        "unsafe_count": 0
      }
    },
    {
      "key": "rust:fn:parse:src_parser_rs:100",
      "hotspot_score": 0.78,
      "incoming_count": 10,
      "outgoing_count": 6,
      "cyclomatic_complexity": 8,
      "risk_indicators": {
        "unwrap_count": 5,
        "clone_count": 0,
        "unsafe_count": 0
      }
    }
  ],
  "tokens": 280
}
```

### GET /cohesion?scope=file (Module Quality Metrics)

```bash
curl "http://localhost:3847/cohesion?scope=file"
```

```json
{
  "success": true,
  "endpoint": "/cohesion",
  "data": [
    {
      "scope": "src/handler.rs",
      "scope_type": "file",
      "internal_edges": 24,
      "external_edges": 8,
      "cohesion_ratio": 0.75,
      "instability": 0.35,
      "health": "good"
    },
    {
      "scope": "src/utils.rs",
      "scope_type": "file",
      "internal_edges": 4,
      "external_edges": 15,
      "cohesion_ratio": 0.21,
      "instability": 0.85,
      "health": "poor"
    }
  ],
  "tokens": 220
}
```

### GET /paths (Critical Path Analysis)

```bash
curl http://localhost:3847/paths
```

```json
{
  "success": true,
  "endpoint": "/paths",
  "count": 3,
  "data": [
    {
      "path_id": "critical_1",
      "start_entity": "rust:fn:main:src_main_rs:1",
      "end_entity": "rust:fn:write_output:src_io_rs:200",
      "path_length": 5,
      "path_entities": ["main", "process", "transform", "serialize", "write_output"],
      "path_type": "critical",
      "importance_score": 0.95
    },
    {
      "path_id": "hot_1",
      "start_entity": "rust:fn:handle:src_api_rs:50",
      "end_entity": "rust:fn:db_query:src_db_rs:100",
      "path_length": 3,
      "path_entities": ["handle", "validate", "db_query"],
      "path_type": "hot",
      "importance_score": 0.78
    }
  ],
  "tokens": 300
}
```

### GET /clusters/{id} (Cluster Detail with Members)

```bash
curl http://localhost:3847/clusters/export_pipeline
```

```json
{
  "success": true,
  "endpoint": "/clusters/export_pipeline",
  "data": {
    "cluster_id": "export_pipeline",
    "cluster_name": "Export Pipeline",
    "purpose": "Handles all code export operations including JSON and text output",
    "entity_class": "CODE",
    "size": 8,
    "cohesion_score": 0.87,
    "coupling_score": 0.15,
    "algorithm": "louvain",
    "members": [
      {
        "key": "rust:fn:export:src_export_rs:10",
        "role": "core",
        "membership_score": 0.95
      },
      {
        "key": "rust:fn:write_json:src_export_rs:100",
        "role": "core",
        "membership_score": 0.92
      },
      {
        "key": "rust:fn:format_output:src_format_rs:50",
        "role": "boundary",
        "membership_score": 0.78,
        "is_bridge": true
      }
    ],
    "parent_cluster_id": null,
    "depth": 0
  },
  "tokens": 350
}
```

---

## File Renaming Tasks (4-Word Convention)

**Decision**: Full rename of all 50 production .rs files to 4-word snake_case names.

**Constraint**: `mod.rs`, `lib.rs`, `main.rs` CANNOT be renamed (Rust convention).

### Task Checklist (Tree Structure)

```
crates/
├── parseltongue/                    (Main CLI - 1 file)
│   └── src/
│       └── [ ] main.rs              → (keep - Rust convention)
│
├── parseltongue-core/               (Foundational Library - 12 files)
│   └── src/
│       ├── [ ] lib.rs               → (keep)
│       ├── [ ] entities.rs          → core_entity_type_definitions.rs
│       ├── [ ] error.rs             → structured_error_handling_types.rs
│       ├── [ ] interfaces.rs        → trait_based_abstraction_contracts.rs
│       ├── [ ] temporal.rs          → temporal_versioning_state_manager.rs
│       ├── [ ] output_path_resolver.rs → timestamped_output_path_resolver.rs
│       ├── [ ] query_extractor.rs   → tree_sitter_query_extractor.rs
│       ├── [ ] query_json_graph_errors.rs → agent_graph_query_errors.rs
│       ├── [ ] query_json_graph_helpers.rs → agent_graph_traversal_helpers.rs
│       ├── [ ] entity_class_specifications.rs → code_test_classification_specs.rs
│       ├── storage/
│       │   └── [ ] cozo_client.rs   → cozo_database_client_wrapper.rs
│       └── serializers/
│           ├── [ ] json.rs          → json_token_serializer_format.rs
│           └── [ ] toon.rs          → toon_compact_serializer_format.rs
│
├── pt01-folder-to-cozodb-streamer/  (Ingestion Tool - 8 files)
│   └── src/
│       ├── [ ] lib.rs               → (keep)
│       ├── [ ] cli.rs               → command_line_argument_parser.rs
│       ├── [ ] errors.rs            → streaming_tool_error_types.rs
│       ├── [ ] streamer.rs          → file_streaming_processor_core.rs
│       ├── [ ] isgl1_generator.rs   → semantic_key_generation_service.rs
│       ├── [ ] lsp_client.rs        → rust_analyzer_lsp_client.rs
│       ├── [ ] test_detector.rs     → test_code_detection_classifier.rs
│       └── [ ] v090_specifications.rs → executable_specification_v090_tests.rs
│
├── pt02-llm-cozodb-to-context-writer/ (Export Tool - 11 files)
│   └── src/
│       ├── [ ] lib.rs               → (keep)
│       ├── [ ] cli.rs               → export_command_argument_parser.rs
│       ├── [ ] errors.rs            → export_tool_error_types.rs
│       ├── [ ] models.rs            → export_data_model_definitions.rs
│       ├── [ ] cozodb_adapter.rs    → cozo_repository_adapter_impl.rs
│       ├── [ ] export_trait.rs      → level_exporter_trait_contract.rs
│       ├── [ ] query_builder.rs     → datalog_query_composition_builder.rs
│       ├── [ ] entity_class_integration_tests.rs → code_test_integration_spec_tests.rs
│       └── exporters/
│           ├── [ ] level0.rs        → edge_only_export_level.rs
│           ├── [ ] level1.rs        → entity_signature_export_level.rs
│           └── [ ] level2.rs        → type_system_export_level.rs
│
└── pt07-visual-analytics-terminal/  (Visualization Tool - 10 files)
    └── src/
        ├── [ ] lib.rs               → (keep)
        ├── [ ] visualizations.rs    → terminal_visualization_renderer_core.rs
        ├── core/
        │   ├── [ ] cycle_detection.rs → circular_dependency_detection_algorithm.rs
        │   ├── [ ] filter_implementation_edges_only.rs → code_edge_filter_implementation.rs
        │   └── [ ] filter_implementation_entities_only.rs → code_entity_filter_implementation.rs
        ├── database/
        │   ├── [ ] adapter.rs       → pt07_database_adapter_bridge.rs
        │   └── [ ] conversion.rs    → type_conversion_utility_helpers.rs
        └── primitives/
            ├── [ ] render_box_drawing_unicode.rs → unicode_box_drawing_renderer.rs
            ├── [ ] render_color_emoji_terminal.rs → terminal_color_emoji_renderer.rs
            └── [ ] render_progress_bar_horizontal.rs → horizontal_progress_bar_renderer.rs
```

### Rename Execution Steps

1. **Phase 1: parseltongue-core** (highest import count)
   - [ ] Rename physical files
   - [ ] Update `mod.rs` declarations
   - [ ] Update all internal imports
   - [ ] Run `cargo check` to verify

2. **Phase 2: pt01-folder-to-cozodb-streamer**
   - [ ] Rename physical files
   - [ ] Update `mod.rs` declarations
   - [ ] Update imports from parseltongue-core
   - [ ] Run `cargo check` to verify

3. **Phase 3: pt02-llm-cozodb-to-context-writer**
   - [ ] Rename physical files
   - [ ] Update `mod.rs` declarations
   - [ ] Update imports from parseltongue-core
   - [ ] Run `cargo check` to verify

4. **Phase 4: pt07-visual-analytics-terminal**
   - [ ] Rename physical files
   - [ ] Update `mod.rs` declarations
   - [ ] Update imports from parseltongue-core
   - [ ] Run `cargo check` to verify

5. **Phase 5: Full verification**
   - [ ] Run `cargo build --release`
   - [ ] Run `cargo test`
   - [ ] Verify all 50 files renamed
   - [ ] Estimated ~200 import statement updates

---

## Changelog

### v1.4.0 (2025-11-27) - Hyphenated Naming + Killer Features

**ISGL Level Taxonomy (4-Word Hyphenated Names)**:
- `Who-Calls-Who-Graph` (ISGL0) - edges only, ~3K tokens
- `Smart-Module-Grouping-Level` (ISGL0.5) - semantic clusters, ~5K tokens
- `Function-Signature-Overview` (ISGL1) - signatures, ~30K tokens
- `Complete-Type-Detail-Level` (ISGL2) - types, ~60K tokens
- `Full-Source-Code-Level` (ISGL3) - full code, ~500K tokens
- `Folder-File-Organization` (ISGL4) - package graph, ~10K tokens

**New Table (14th)**:
- `TemporalCouplingEdgeStore` - Git-derived hidden dependencies
- Reveals invisible architecture: files that change together but have ZERO code dependency

**2 Killer Features**:
1. **Temporal Coupling Detection** (~1200 LOC)
   - `/temporal-coupling-hidden-deps/{entity}` endpoint
   - Git log parsing to find co-changed files
   - Multi-signal affinity for 63%→91% cluster accuracy
2. **Dynamic Context Selection** (~1500 LOC)
   - `/smart-context-token-budget?focus=X&tokens=N` endpoint
   - Greedy knapsack algorithm with multi-signal scoring
   - Token-budget-aware context selection

**Hyphenated Endpoint URLs**:
- All endpoints now use 4-word hyphenated URLs for LLM readability
- Example: `/server-health-check-status` instead of `/health`

**File Renaming Task List**:
- Complete checklist for renaming all 50 .rs files to 4-word names
- Tree structure with checkboxes for tracking progress

### v1.3.0 (2025-11-27) - Full Schema Redesign + High-ROI Features

**Database Schema Redesign (13 Tables)**:
- **CODE/TEST Separation**: Production code and test code now in separate tables
  - `CodeProductionEntityStore` - Production entities only
  - `TestImplementationEntityStore` - Test entities only
  - `CodeDependencyEdgeGraph` - CODE→CODE edges
  - `TestDependencyEdgeGraph` - TEST→TEST edges
  - `TestToCodeEdgeBridge` - TEST→CODE coverage links
- **All tables follow 4-word naming convention** for LLM tokenization optimization

**Semantic Clustering (ISGL0.5)**:
- `SemanticClusterDefinitionStore` - Logical boundaries beyond geographic file boundaries
- `EntityClusterMembershipMap` - Entity→Cluster mapping with roles (core/boundary/bridge)
- Supports hierarchical clusters with parent/child relationships

**Pre-computed Metrics**:
- `EntityComputedMetricsCache` - Hotspot scores, complexity, risk indicators (unwrap/clone/unsafe counts)
- `GraphGlobalStatisticsStore` - Global codebase statistics
- `ModuleCohesionMetricsCache` - Cohesion ratios, instability metrics

**Analysis Tables**:
- `FileEntityMappingIndex` - Package graph (ISGL4) with per-file stats
- `OrphanDeadCodeCache` - Dead code detection cache
- `ControlFlowPathAnalysis` - Critical path analysis

**6 New HTTP Endpoints**:
| Endpoint | Handler | Purpose |
|----------|---------|---------|
| `GET /orphans` | `handle_orphans_list_request()` | Dead code detection |
| `GET /files` | `handle_files_mapping_request()` | Package graph |
| `GET /complexity` | `handle_complexity_analysis_request()` | Pre-computed metrics |
| `GET /cohesion` | `handle_cohesion_scoring_request()` | Module quality |
| `GET /paths` | `handle_control_paths_request()` | Critical paths |
| `GET /clusters/{id}` | `handle_cluster_detail_request()` | Cluster details |

**Key User Insight Addressed**:
> "what matters at a level different than APIs or files - which are all geographical boundaries"

ISGL0.5 semantic clusters provide understanding at logical/functional boundaries, not just file boundaries.

### v1.2.0 (2025-11-27) - CPG-Inspired Enhancements

**New Edge Types** (Code Property Graph inspired):
- `Extends` - Inheritance/trait extension (upward direction)
- `Contains` - Structural containment (downward direction)

**Direction Metadata**:
- All edges now include `direction` field: `"downward"`, `"upward"`, `"horizontal"`
- Enables semantic understanding of relationship flow

**New Endpoint**:
- `GET /clusters` - Semantic module groupings via LPA (Label Propagation Algorithm)
- Wraps existing pt08 clustering capability
- Returns cohesion scores for each cluster

### v1.1.0 (2025-11-26) - Initial HTTP Server Architecture

- HTTP server replaces CLI + JSON exports
- 4-word command naming convention
- 12 languages supported via tree-sitter
- TDD-first implementation approach

---

**End of PRD v1.4.0**
