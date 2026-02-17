# Prep-V200-Compiled-Research-Best-Ideas

## Context

This document compiles the best actionable ideas from 8 research documents into a single reference for v2.0.0 planning. Parseltongue v2.0.0 is a clean-room rewrite with 8 `rust-llm-*` crates. Every idea traces back to its source document.

**Source documents:**
- [CR-01] `docs/CR-v173-01.md` -- Low coverage repos parse failure analysis
- [CR-02] `docs/CR-v173-02.md` -- Deep competitive analysis of 9 repos (19,431 entities, 144,137 edges)
- [CR-03] `docs/CR-v173-03.md` -- Competitor feature deep-dive with implementation specs
- [CR-04] `docs/CR-v173-04-oh-my-pi.md` -- oh-my-pi deep dive (MCP client, agent runtime, LSP)
- [RA-RESEARCH] `docs/RESEARCH-v173-rustanalyzer-semantic-supergraph.md` -- rust-analyzer API surface
- [SHREYAS] `docs/v173-CR-Shreyas-Feedback-01.md` -- Shreyas Doshi LNO evaluation
- [PT04-WORKFLOW] `docs/v173-pt04-bidirectional-workflow.md` -- Three-layer architecture thesis
- [THESIS] `docs/pre175/THESIS-v173-slim-graph-address-model.md` -- Slim graph model (114-151 bytes/entity)

---

## Part 1: Competitive Landscape Insights

### Who exists and where they fall short

```
Competitor         Type              Entities   Key Weakness
-----------        ----              --------   ------------
code-scalpel       Full analysis     6,521      Monolithic server.py (CBO=483, grade F).
                                                Re-parses on demand. No persistent graph.
AiDex              Persistent index  159        SQLite flat tables. No graph algorithms.
                                                No PageRank, Leiden, K-core, SCC, or
                                                blast radius possible.
gemini-cli         MCP client        3,410      Not an analysis tool. MCP client infra only.
mcp-grep-servers   Thin wrappers     234        Stateless. No storage. Wrap ripgrep/grep.
oh-my-pi           Agent runtime     831 files  MCP consumer, not producer. Orchestrator
                                                not analysis engine.
```

[CR-02] [CR-03] [CR-04]

### Gaps parseltongue uniquely fills

1. **Graph-native storage**: No competitor uses a graph database. CozoDB with RocksDB is architecturally unique. Enables Leiden, PageRank, K-core, SCC, SQALE, entropy -- algorithms impossible on SQLite or stateless wrappers. [CR-02]

2. **Cross-repo unified analysis**: Single CozoDB instance ingested 9 repos (19,431 entities, 144,137 edges). No competitor demonstrated cross-repo graph analysis. [CR-02]

3. **Pre-computed graph queries**: code-scalpel re-parses on demand (CBO=483 god object). Parseltongue pre-computes the graph once, queries are instant. [CR-02]

4. **12-language unified graph**: Equal to code-scalpel's language count but with typed edges and graph algorithms layered on top. AiDex indexes 11 languages but at token/identifier level only. [CR-02]

### Market positioning

Three product archetypes exist in the space [SHREYAS]:

1. **Indexers** (AiDex, ripgrep wrappers) -- parse and store structured data, serve low-token responses. PT competes on depth: they index identifiers, PT indexes relationships.

2. **Analyzers** (code-scalpel, Semgrep MCP) -- active analysis like security scanning, taint tracking. PT competes on graph algorithms: nobody else has Tarjan SCC, PageRank, K-core, Leiden, SQALE, or blast radius.

3. **Agents** (Gemini CLI, oh-my-pi, CodeCompanion) -- orchestrate AI coding workflows. PT does NOT compete here. PT is a tool that agents consume.

**Strategic position**: The graph analysis engine that indexers wish they had and analyzers have not built. [SHREYAS]

---

## Part 2: Architecture and Storage Ideas

### Slim graph model

The slim model thesis [THESIS] proved that Parseltongue's unique value is the dependency GRAPH, not code STORAGE. Code storage is what cat, IDEs, and LLM file-reading tools already do.

```
Slim Entity: ~151 bytes per entity (vs ~3,000 bytes full model)
  isgl1_key     50 B
  file_path     40 B
  line_range    10 B
  entity_type   10 B
  language       6 B
  subfolder_L1  15 B
  subfolder_L2  20 B

Slim Edge: ~120 bytes per edge
  from_key, to_key, edge_type
```

**RAM at 1.6M-edge scale:**

```
Component                    Full Model    Slim Model
---------                    ----------    ----------
400K entities                1,200 MB      60 MB
1.6M edges                   400 MB        192 MB
CozoDB overhead              1,600 MB      252 MB
Total base RAM               3,200 MB      ~504 MB
```

21 of 24 endpoints scored 8-10/10 on the slim model. Three dropped endpoints: /smart-context-token-budget (no code to budget), /ingestion-diagnostics-coverage-report (needs full diagnostic data), /ingestion-coverage-folder-report (needs filesystem walk). [THESIS]

### MessagePack over CozoDB

For pt02/pt03 portable formats, MessagePack serialization of slim entities and edges produces a ~200 MB .ptgraph file at 1.6M-edge scale (vs ~1 GB full CozoDB, ~4 GB JSON). pt08 loads either rocksdb: or ptgraph: or json: prefixed databases. [THESIS]

### Key format design: ISGL1

The ISGL1 key format encodes language:type:name:file_hash:timestamp. This is the universal address for any code entity. At slim scale, searching ISGL1 keys replaces full-text search -- ?q=main finds rust:fn:main:.... The key IS the index. [THESIS]

### Three-layer architecture (pt04 + LLM + CPU)

The central architectural thesis for v2.0.0 [PT04-WORKFLOW]:

```
Layer 1: pt04 (rust-analyzer)     -> GROUND TRUTH about types, traits, dispatch
Layer 2: LLM                       -> JUDGMENT about business context, naming, priorities
Layer 3: CPU graph algorithms      -> FAST computation over the enriched graph
```

**Performance comparison:**

```
Operation                  CPU Only    Bidirectional    Three-Layer
                                       (LLM+CPU)       (pt04+LLM+CPU)
---------                  --------    -------------    ---------------
Module Detection           0.3s/67%    2.1s/91%         0.8s/~96%
Cycle Classification       0.1s/0%     1.3s/95%         0.4s/~99%
Complexity Analysis        0.2s/0%     2.8s/93%         0.3s/~95%
Tech Debt Scoring          0.8s/64%    4.2s/89%         2.5s/~92%
```

The LLM stops guessing at things the compiler already knows. LLM calls drop from 50-200 per analysis to 3-5. [PT04-WORKFLOW]

---

## Part 3: Rust-Analyzer Semantic Supergraph

### Highest-value API methods (priority order)

**Phase 1 -- TypedCallEdges (80% of value):** [RA-RESEARCH] [PT04-WORKFLOW]

```
API                              What it gives us
---                              ----------------
incoming_calls/outgoing_calls    Typed edges: Direct, TraitMethod, DynDispatch, ClosureInvoke
Type::as_callable()              Resolved receiver types for each call
Semantics::resolve_method_call() Which impl block a method call resolves to
```

One CozoDB relation (TypedCallEdges) enriches all 26 existing endpoints. This is the single highest-leverage feature for pt04.

**Phase 2 -- TraitImpls + SupertraitEdges:**

```
Trait::direct_supertraits()      Supertrait hierarchy
Trait::all_supertraits()         Full trait chain
Impl::all_for_trait()            All types implementing a trait
Impl::all_for_type()             All traits a type implements
```

Enables /trait-hierarchy-graph-view and better cycle classification.

**Phase 3 -- SemanticTypes:**

```
Function::ret_type()             Fully resolved return type
Function::params_without_self()  Param types (not just names)
Module::scope()                  Visibility analysis
Type::layout()                   Memory layout (size, padding)
```

Enables visibility audit, unsafe audit, type-based search.

### What tree-sitter cannot do that rust-analyzer can

```
Capability                    tree-sitter    rust-analyzer
----------                    -----------    -------------
Resolve method to impl block  No             Yes
Trait hierarchy traversal      No             Yes
Closure capture analysis       No             Yes (kind + type)
Macro expansion                No             Yes
Generic bound resolution       No             Yes
Visibility (effective)         Partial        Full (incl. re-exports)
Memory layout                  No             Yes
Async/Send/Sync analysis       No             Yes
```

Key insight: tree-sitter gives you the skeleton. rust-analyzer gives you the nervous system. [RA-RESEARCH]

### Ship order (Shreyas LNO applied to pt04 features)

```
LEVERAGE:    TypedCallEdges only (~300 lines of new Rust)
NEUTRAL:     TraitImpls + SupertraitEdges (Phase 2)
NEUTRAL:     SemanticTypes (Phase 3)
OVERHEAD:    TypeLayouts, ClosureCaptures standalone, generic maps (Phase 4 -- build on demand)
```

[PT04-WORKFLOW]

---

## Part 4: Tree-Sitter Query Improvements

### Specific .scm query gaps (from CR-v173-01)

**Priority 1 -- Shebang lines (CRITICAL):** [CR-01]
- Affects 6 files across 3 repos (CodeSeeker-MCP, mcp-ripgrep, mcp-server-semgrep)
- #!/usr/bin/env node creates ERROR node in tree-sitter-typescript/javascript
- Fix: Strip shebang before parsing, replace with whitespace to preserve byte offsets
- Impact: Fixes 0% coverage for 3 repos

**Priority 2 -- Const object export pattern (HIGH):** [CR-01]
- Dominant MCP tool pattern: export const myTool = { name: "...", execute: async () => {...} }
- Not captured by any .scm query (object literal, not function/class)
- Fix: Add to typescript.scm
- Impact: +20-40% coverage per repo for 4 TypeScript repos

**Priority 3 -- Decorated Python functions (MEDIUM):** [CR-01]
- @mcp.tool() decorated functions not captured
- tree-sitter wraps as decorated_definition > function_definition
- Fix: Add to python.scm
- Impact: 25% -> 60-80% for ast-grep-mcp

**Priority 4 -- CRLF normalization (MEDIUM):** [CR-01]
- CodeSeeker-MCP uses CRLF line endings
- Fix: Normalize CRLF to LF before parsing

**Priority 5 -- Variable declarations (LOW):** [CR-01]
- Zod schemas, config constants, significant const/let declarations
- Add pattern for typed const declarations

### Missing entity types

```
Entity Type                Source       Impact
-----------                ------       ------
MCP tool objects           [CR-01]      All MCP server repos
Decorated functions        [CR-01]      Python MCP repos
Shebang-gated files        [CR-01]      All Node.js CLI tools
Test callbacks (describe)  [CR-01]      All Vitest/Jest test files
Config/re-export files     [CR-01]      Module index files
Enum members               [CR-01]      TypeScript enum values
```

### Cross-language detection patterns

No cross-language detection gaps were found in the research. The 12-language parser handles all syntax correctly. The gap is in ENTITY EXTRACTION queries (.scm files), not in grammar/parsing. [CR-01]

---

## Part 5: MCP and LLM Integration

### MCP server positioning

Parseltongue should be an MCP **server** (provider), not an MCP **client** (consumer). oh-my-pi, Gemini CLI, and Claude Code are MCP clients that would connect TO parseltongue. [CR-04]

```
Protocol stack:  Editor <-> ACP <-> Agent <-> MCP <-> Tool
Parseltongue position:                                 ^^^^
```

Implementation: pt09-mcp-protocol-bridge-server wraps pt08 HTTP handlers as MCP tools via stdio + streamable-HTTP transports. Each of 24 endpoints becomes an MCP tool. Follow Semgrep: parseltongue mcp as a subcommand, not a separate binary. [CR-03] [SHREYAS]

### Token-budgeted output strategies

1. **Per-query token estimation**: Add token_estimate field to every JSON response. Calculate as response_bytes / 4. AiDex "50 tokens per query" is their most effective marketing claim. [SHREYAS]

2. **Progressive disclosure**: Add ?detail=summary|standard|full parameter to all endpoints. Summary = entity keys and counts only (~50 tokens). Standard = current default. Full = raw edge data + all metadata. Three competitors independently converged on this pattern. [SHREYAS]

3. **Model-aware budgets**: Add ?model=claude-sonnet|gpt-4o|gemini-pro parameter to /smart-context-token-budget for model-specific context window limits. [CR-03]

### LLM context optimization ideas

1. **Semantic relevance prioritization** (with pt04): Instead of selecting entities by graph distance, prioritize by TRAIT_DEFINITION > TRAIT_SIBLING > ERROR_TYPE > TRAIT_DISPATCH_CALLER. Direct utility calls are nearby but architecturally irrelevant. [PT04-WORKFLOW]

2. **Two-tier response model** (from AiDex, Greptile): Overview first (entity keys + locations), detail on demand (/code-entity-detail-view/{key}). Reduces per-query context consumption. [SHREYAS]

3. **Session persistence**: Store analysis findings in CozoDB alongside the graph. Next session retrieves prior context. Turns stateless queries into stateful analysis sessions. [SHREYAS]

### Progressive disclosure patterns

```
Level        Response Shape                               Tokens
-----        --------------                               ------
summary      { entity_count: 755, edge_count: 4055 }     ~50
standard     { entities: [{key, name, type, file}...] }   ~2,000
full         { entities: [{...all metadata...}], raw: {}} ~50,000
```

[SHREYAS] [CR-03]

---

## Part 6: Shreyas Doshi LNO Leverage Items

### All 8 Leverage items

```
#   Item                              Effort     Source
--  ----                              ------     ------
L1  Orphaned entity detection         1 day      [SHREYAS] inspired by kp-ripgrep-mcp
L2  Per-query token estimation        2 hours    [SHREYAS] inspired by AiDex
L3  Time-based entity filtering       2 days     [SHREYAS] inspired by AiDex
L4  OpenTelemetry tracing             3 days     [SHREYAS] inspired by Semgrep MCP
L5  Session persistence layer         3 days     [SHREYAS] inspired by AiDex, Greptile
L6  Graph neighborhood alias          30 min     [SHREYAS] inspired by code-scalpel
L7  MCP as subcommand                 1 week     [SHREYAS] inspired by Semgrep MCP
L8  Progressive disclosure tiers      2 days     [SHREYAS] inspired by Gemini CLI, AiDex
```

### Mapping to v2.0.0 crates

```
Leverage Item                v2.0.0 Crate
-------------                ------------
L1 Orphaned entity detection rust-llm-graph-algorithms (new Datalog query)
L2 Token estimation          rust-llm-http-server (response wrapper)
L3 Time-based filtering      rust-llm-core (entity metadata) + rust-llm-http-server
L4 OpenTelemetry tracing     rust-llm-http-server (middleware)
L5 Session persistence       rust-llm-core (CozoDB relation) + rust-llm-http-server
L6 Graph neighborhood alias  rust-llm-http-server (route alias)
L7 MCP subcommand            rust-llm-mcp-bridge (new crate, wraps HTTP handlers)
L8 Progressive disclosure    rust-llm-http-server (query parameter)
```

### Priority ordering for max adoption

Ship order (highest leverage-to-effort ratio first):

1. **L6** Graph neighborhood alias (30 min) -- immediate discoverability win
2. **L2** Token estimation (2 hours) -- every response becomes marketing
3. **L8** Progressive disclosure (2 days) -- LLM context optimization
4. **L1** Orphaned entity detection (1 day) -- dead code detection, high demand
5. **L3** Time-based filtering (2 days) -- "what changed recently?" workflow
6. **L5** Session persistence (3 days) -- stateful analysis companion
7. **L4** OpenTelemetry tracing (3 days) -- production credibility
8. **L7** MCP subcommand (1 week) -- ecosystem integration unlock

The single highest-leverage combination: **Ship L7 (MCP) + L8 (progressive disclosure) + L2 (token estimates)**. This makes PT graph analysis discoverable by every AI agent, consumable at any context budget, and self-marketing with every response. [SHREYAS]

---

## Part 7: New Endpoint Ideas

### pt04-enriched versions of existing 22 endpoints

Every existing endpoint gets a semantic field when pt04 data is available [PT04-WORKFLOW]:

```
Endpoint                              What pt04 adds
--------                              --------------
/code-entities-list-all               return_type, params with types, is_async,
                                      visibility, trait_impls, dispatch_kinds
/code-entity-detail-view              Fully resolved signature, closure captures,
                                      trait dispatch targets, effective visibility
/dependency-edges-list-all            call_kind (Direct/TraitMethod/DynDispatch/
                                      ClosureInvoke), via_trait, receiver_type
/reverse-callers-query-graph          Callers classified by dispatch kind
/forward-callees-query-graph          unique_traits_consumed, responsibility_score
/blast-radius-impact-analysis         Typed blast radius (trait boundary crossings,
                                      affected_crates, safety_assessment)
/circular-dependency-detection-scan   Auto-classification (INTENTIONAL_PATTERN vs
                                      LIKELY_VIOLATION vs AMBIGUOUS)
/complexity-hotspots-ranking-view     Coupling breakdown by dispatch kind
/semantic-cluster-grouping-list       Trait-anchored clusters, anchor_traits field
/technical-debt-sqale-scoring         VISIBILITY_BLOAT, PADDING_WASTE, DEAD_TRAIT_IMPL
/centrality-measures-entity-ranking   Typed edge subgraph filtering
/entropy-complexity-measurement       8 edge types (was 3), max H = 3.0
/coupling-cohesion-metrics-suite      CBO breakdown by dispatch kind, typed_diagnosis
/leiden-community-detection-clusters  Trait-seeded clustering, modularity improvement
/kcore-decomposition-layering         trait_dispatch_degree, is_trait_definition
/smart-context-token-budget           Semantic relevance prioritization
/codebase-statistics-overview-summary async_functions, unsafe_blocks, generic_functions
```

### New semantic-only endpoints (pt04 required)

```
Endpoint                              Purpose                           Source
--------                              -------                           ------
/trait-hierarchy-graph-view           Supertrait tree + implementors    [PT04-WORKFLOW]
/async-call-chain-trace               Await chain + spawn points        [PT04-WORKFLOW]
/visibility-audit-report              Unnecessary pub items             [PT04-WORKFLOW]
/unsafe-audit-report                  Unsafe blocks + blast radius      [PT04-WORKFLOW]
/type-size-layout-analysis            Struct padding optimization       [PT04-WORKFLOW]
/closure-capture-analysis             Mutable captures in async code    [PT04-WORKFLOW]
/generic-instantiation-map            Monomorphization cost             [PT04-WORKFLOW]
```

### Bidirectional workflow endpoints

```
Endpoint                                  Purpose                       Source
--------                                  -------                       ------
/orphaned-entities-isolation-detection    Dead code candidates          [SHREYAS]
/graph-neighborhood-extraction-query      Alias for blast radius        [SHREYAS]
/session-notes-persist-store              Stateful analysis notes       [SHREYAS]
/session-activity-tracking-status         Session lifecycle             [CR-03]
/policy-violation-detection-scan          Datalog-native policy engine  [CR-03]
/taint-flow-analysis-report              Graph-based taint tracking     [CR-03]
/structural-pattern-entity-search         Search by entity structure    [SHREYAS]
/observability-metrics-telemetry-export   Request latency + error rates [CR-03]
```

---

## Part 8: Moat and Differentiation Strategy

### What makes parseltongue architecturally unique

1. **Graph-native storage**: CozoDB with Datalog queries. No competitor uses a graph database for code analysis. Graph algorithms (Leiden, PageRank, K-core, SCC, betweenness) are first-class citizens, not bolt-on features. [CR-02]

2. **Pre-computed analysis**: Ingest once, query instantly. code-scalpel re-parses on demand (483 outbound dependencies from server.py). Parseltongue graph is ready at query time. [CR-02]

3. **Three-layer architecture**: Compiler truth (pt04) + LLM judgment + CPU graph algorithms. No competitor has this separation. code-scalpel relies entirely on on-demand Python AST parsing. [PT04-WORKFLOW]

4. **Rust-native performance**: 100% Rust. oh-my-pi bridges TypeScript->Rust via N-API for performance. Parseltongue gets Rust performance natively. [CR-04]

### Features no competitor has

```
Feature                        PT Status    Nearest Competitor
-------                        ---------    ------------------
Leiden community detection     Shipped      None
K-core decomposition           Shipped      None
Tarjan SCC analysis            Shipped      None
PageRank on code entities      Shipped      None
SQALE tech debt scoring        Shipped      None
Shannon entropy scoring        Shipped      None
Blast radius (graph BFS)       Shipped      code-scalpel (similar, different name)
Typed call edges (pt04)        Planned      None
Trait hierarchy traversal      Planned      None
Async call chain tracing       Planned      None
```

[CR-02] [PT04-WORKFLOW]

### The CodeQL-like rule engine idea

Policies expressed as CozoDB Datalog queries instead of OPA/Rego [CR-03]. No external OPA CLI. No Rego language. CozoDB Datalog IS the policy language. This leverages existing graph infrastructure and is architecturally unique -- no competitor has graph-native policy enforcement. [CR-03]

### Institutional knowledge accumulation

1. **Session persistence**: Analysis findings stored in CozoDB survive sessions. The graph accumulates institutional knowledge over time. [SHREYAS]

2. **Cross-repo graph**: Single database for multiple repos. Community detection across repos reveals shared patterns. [CR-02]

3. **Time-based entity tracking**: ?modified_since=2h enables "what changed?" workflows without git. The graph becomes a temporal knowledge base. [SHREYAS]

4. **Deterministic tool framing**: Deterministic graph queries, not AI guessing. Parseltongue gives LLMs compiler-verified facts, not string matching results. [SHREYAS] [CR-03]

---

## Part 9: Actionable Priority Matrix

### HIGH priority (ship in v2.0.0 core)

```
ID   Idea                              Crate                     Effort   Source      Depends On
--   ----                              -----                     ------   ------      ----------
H1   Slim graph model (151 B/entity)   rust-llm-core             2 wk    [THESIS]    None
H2   TypedCallEdges (pt04 Phase 1)     rust-llm-semantic-engine  3 wk    [PT04-WF]   H1
H3   MCP subcommand (pt09)             rust-llm-mcp-bridge       2 wk    [SHREYAS]   H1
H4   Progressive disclosure (?detail)  rust-llm-http-server      2 day   [SHREYAS]   H1
H5   Per-query token estimation        rust-llm-http-server      2 hr    [SHREYAS]   H1
H6   Shebang line handling             rust-llm-core (parser)    1 day   [CR-01]     None
H7   Const object export .scm query    rust-llm-core (parser)    1 day   [CR-01]     None
H8   Decorated Python fn .scm query    rust-llm-core (parser)    1 day   [CR-01]     None
H9   Orphaned entity detection         rust-llm-graph-algorithms 1 day   [SHREYAS]   H1
H10  Graph neighborhood alias          rust-llm-http-server      30 min  [SHREYAS]   H1
H11  MessagePack export (pt02/pt03)    rust-llm-export           1 wk    [THESIS]    H1
```

### MEDIUM priority (ship in v2.0.x follow-ups)

```
ID   Idea                              Crate                     Effort   Source      Depends On
--   ----                              -----                     ------   ------      ----------
M1   TraitImpls + SupertraitEdges      rust-llm-semantic-engine  1 wk    [RA-RES]    H2
M2   SemanticTypes (resolved sigs)     rust-llm-semantic-engine  1 wk    [RA-RES]    M1
M3   Time-based entity filtering       rust-llm-core + http      2 day   [SHREYAS]   H1
M4   Session persistence layer         rust-llm-core + http      3 day   [SHREYAS]   H1
M5   OpenTelemetry tracing             rust-llm-http-server      3 day   [SHREYAS]   H1
M6   CRLF normalization                rust-llm-core (parser)    2 hr    [CR-01]     None
M7   Trait hierarchy endpoint          rust-llm-http-server      2 day   [PT04-WF]   M1
M8   Async call chain trace endpoint   rust-llm-http-server      3 day   [PT04-WF]   H2
M9   Visibility audit endpoint         rust-llm-http-server      2 day   [PT04-WF]   M2
M10  Datalog policy engine             rust-llm-core + http      3 wk    [CR-03]     H1
M11  Confidence scores on edges        rust-llm-core             1 wk    [SHREYAS]   H1
M12  Multi-format response adapters    rust-llm-http-server      3 day   [SHREYAS]   H1
M13  Auto-setup CLI (detect MCP)       rust-llm-cli              4 day   [SHREYAS]   H3
M14  Graph health diagnostic endpoint  rust-llm-http-server      3 day   [SHREYAS]   H9
```

### LOW priority (build when someone asks)

```
ID   Idea                              Crate                     Effort   Source      Depends On
--   ----                              -----                     ------   ------      ----------
L1   TypeLayouts (memory/padding)      rust-llm-semantic-engine  3 day   [PT04-WF]   M2
L2   ClosureCaptures standalone EP     rust-llm-http-server      2 day   [PT04-WF]   H2
L3   Generic instantiation map         rust-llm-http-server      2 day   [PT04-WF]   M2
L4   Unsafe audit endpoint             rust-llm-http-server      2 day   [PT04-WF]   M2
L5   Structural pattern search         rust-llm-http-server      2 wk    [SHREYAS]   H1
L6   Graph taint flow analysis         rust-llm-core + http      6 wk    [CR-03]     H1
L7   Agent skill file publication      rust-llm-cli              1 day   [SHREYAS]   H3
L8   Lua language support              rust-llm-core (parser)    3 day   [CR-03]     None
L9   Variable declaration .scm query   rust-llm-core (parser)    1 day   [CR-01]     None
L10  Enum member support               rust-llm-core (parser)    1 day   [CR-01]     None
```

### NOT recommended (overhead per LNO framework)

```
ID   Idea                              Rationale                          Source
--   ----                              ---------                          ------
X1   Z3 symbolic execution             12-16 weeks, wrong product scope   [CR-03]
X2   Code modification tools           Changes PT from read-only to write [CR-03]
X3   Taint analysis engine (full)      Competes with Semgrep; use interop [CR-03]
X4   Task management / backlog         PT is graph analysis, not Jira     [SHREYAS]
X5   Screenshot capture                Scope creep                        [SHREYAS]
X6   Browser viewer / frontend         Let community build viewers        [SHREYAS]
X7   Sub-agent orchestration           Wrong layer (agent, not tool)      [SHREYAS]
X8   Multi-LLM provider client         PT outputs data, not LLM calls    [CR-04]
X9   Sandbox / process isolation       PT is read-only HTTP server        [SHREYAS]
X10  Cloud-dependent semantic search   PT local-first is a feature        [SHREYAS]
```

### Dependency graph for HIGH items

```
None -----> H6 (shebang)
None -----> H7 (const export .scm)
None -----> H8 (decorated Python .scm)
None -----> H1 (slim graph) -----> H2 (TypedCallEdges)
                              |---> H3 (MCP subcommand)
                              |---> H4 (progressive disclosure)
                              |---> H5 (token estimation)
                              |---> H9 (orphaned entities)
                              |---> H10 (graph neighborhood alias)
                              |---> H11 (MessagePack export)
```

H6, H7, H8 have zero dependencies -- ship them first as parser improvements. H1 (slim model) is the foundation for everything else. H2 (TypedCallEdges) unlocks the semantic supergraph. H3 (MCP) unlocks ecosystem adoption. H4+H5 make every response optimized for LLM consumption.

---

*Compiled 2026-02-16 from 8 research documents totaling ~80,000 words. Every claim traces to a source document.*
