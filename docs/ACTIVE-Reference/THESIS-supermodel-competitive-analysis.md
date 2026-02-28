# THESIS: Supermodel Competitive Analysis
## What Parseltongue v200 Can Learn from the Supermodel Ecosystem

*Research date: 2026-02-22*
*Sources: VSIX v0.1.162 (decompiled), GitHub repos: openapi-spec, mcp, arch-docs, dead-code-hunter, typescript-sdk*
*Gitignored extraction: competitor-research/supermodel/extracted/*

---

## 0. Executive Summary

Supermodel is a **cloud-first, AI-era codebase governance platform** built around a single powerful primitive: a hosted graph database (Neo4j) that understands your code. Their moat is that every developer tool they ship — VS Code extension, MCP server, GitHub Actions, static site generator — is a thin client over that central graph.

Parseltongue v200 is building the same primitive **locally, in Rust, as a CLI-first tool**. The comparison is instructive precisely because they made almost every opposite design choice, and some of those choices are objectively better.

```text
┌─────────────────────────────────────────────────────────────────────────┐
│              SUPERMODEL vs PARSELTONGUE v200                            │
├──────────────────────────┬──────────────────────────────────────────────┤
│  SUPERMODEL              │  PARSELTONGUE v200                           │
├──────────────────────────┼──────────────────────────────────────────────┤
│  Cloud Neo4j             │  Local CozoDB / RocksDB                      │
│  SaaS subscription       │  OSS CLI                                     │
│  VS Code primary UI      │  HTTP API primary UI                         │
│  MCP = cloud proxy       │  MCP = local proxy                           │
│  GitHub Action delivery  │  cargo install delivery                      │
│  JS/TS codebase          │  Rust codebase                               │
│  API v0.9.6              │  v200 (in design)                            │
│  5-15 min analysis       │  <60s target (local)                         │
│  $$/month                │  Free                                        │
│  GitHub OAuth            │  No auth needed                              │
└──────────────────────────┴──────────────────────────────────────────────┘
```

---

## 1. Objective Analysis: The Supermodel Ecosystem

### 1.1 Product Architecture

Supermodel is five loosely-coupled products sharing one backend API:

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                    SUPERMODEL ECOSYSTEM MAP                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │  VS Code Ext │  │  MCP Server  │  │  GitHub      │                  │
│  │  (React +D3) │  │  (stdio, TS) │  │  Action      │                  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                  │
│         │                 │                  │                          │
│         └─────────────────┼──────────────────┘                          │
│                           │ HTTPS + JSON-RPC 2.0                        │
│                           ▼                                             │
│         ┌─────────────────────────────────────┐                         │
│         │  api.supermodeltools.com             │                         │
│         │  OpenAPI v0.9.6                      │                         │
│         │  • Code graph generation             │                         │
│         │  • Dead code analysis (async)        │                         │
│         │  • Domain classification             │                         │
│         │  • Impact analysis                   │                         │
│         │  • Circular dependency detection     │                         │
│         │  • Test coverage mapping (static)    │                         │
│         │  • Supermodel IR (unified graph)     │                         │
│         └─────────────────┬───────────────────┘                         │
│                           │                                             │
│         ┌─────────────────▼───────────────────┐                         │
│         │  Neo4j + PostgreSQL                  │                         │
│         │  (hosted, managed by Supermodel)     │                         │
│         └─────────────────────────────────────┘                         │
│                                                                         │
│  ┌──────────────┐                                                        │
│  │  arch-docs   │  GitHub Action → Go SSG → Static site                 │
│  │  (Go + D3)   │  entity pages, llms.txt, sitemaps, RSS                │
│  └──────────────┘                                                        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 The SupermodelIR: Their Unified Intermediate Representation

The most important technical insight is their **SupermodelIR** — a unified graph schema that all analysis tools produce and consume:

```typescript
interface SupermodelIR {
  repo: string;
  version: string;
  schemaVersion: string;
  generatedAt: DateTime;
  summary: {
    filesProcessed, classes, functions, types,
    primaryLanguage, domains: DomainSummary[]
  };
  stats: {
    nodeCount, relationshipCount,
    nodeTypes: Record<string, number>,
    relationshipTypes: Record<string, number>
  };
  metadata: { analysisStartTime, analysisEndTime, fileCount, languages[] };
  domains: DomainSummary[];
  graph: {
    nodes: CodeGraphNode[];         // parse graph + call graph + domain
    relationships: CodeGraphRelationship[];  // contains, imports, calls, belongsTo
  };
  artifacts: SupermodelArtifact[];  // per-source stats
}
```

All five products consume this IR. Dead code hunter sends a ZIP and gets an IR back. The MCP server caches an IR in LRU + disk. arch-docs renders an IR into HTML. This is their **single source of truth** pattern.

### 1.3 MCP Server: The Most Interesting Tool

The MCP server (`@supermodeltools/mcp-server`) is technically the most sophisticated piece. It exposes exactly **two MCP tools**:

| Tool | What it does |
|------|-------------|
| `symbol_context` | Source + callers + callees + domain for any function/class/method. Supports `Class.method`, partial matching, batch lookup (max 10). `brief: true` for compact mode. |
| `overview` | Architectural map: domains, hub functions (≥3 callers by in-degree), file/function/class counts, primary language. Sub-second from cache. |

**Three-tier graph resolution** (most important implementation pattern):
```text
Request comes in
  │
  ▼
1. Pre-computed cache (SUPERMODEL_CACHE_DIR) — loaded at startup
   → Matches by commit hash, dir name, or git remote
  │ miss
  ▼
2. LRU memory cache — keyed by idempotency key (SHA1 of path + git status)
  │ miss
  ▼
3. API call → generateSupermodelGraph() → ZIP upload → 5-15 min analysis
   (blocked if SUPERMODEL_NO_API_FALLBACK is set)
```

**Key startup trick**: Connect transport FIRST (so MCP handshake completes within 60s timeout), THEN do precaching. The graph analysis is fire-and-forget from the client's perspective.

**`IndexedGraph` structure** (their in-memory representation, most complete I've seen):
```typescript
interface IndexedGraph {
  raw: SupermodelIR;
  nodeById: Map<string, CodeGraphNode>;
  labelIndex: Map<string, string[]>;       // label → [nodeId]
  pathIndex: Map<string, PathIndexEntry>;  // filepath → entry
  dirIndex: Map<string, string[]>;         // dir → [nodeId]
  nameIndex: Map<string, string[]>;        // lowercase name → [nodeId]
  callAdj: Map<string, AdjacencyList>;     // call graph adjacency
  importAdj: Map<string, AdjacencyList>;   // import graph adjacency
  domainIndex: Map<string, {
    memberIds: string[],
    relationships: CodeGraphRelationship[]
  }>;
  summary: { filesProcessed, classes, functions, types,
             domains, primaryLanguage, nodeCount, relationshipCount };
  cachedAt: string;
  cacheKey: string;
}
```

Eight separate indexes built from the same raw IR. This is what makes sub-second lookup possible.

### 1.4 Dead Code Hunter: Surprising Depth

The dead code GitHub Action is simple on the surface but reveals sophisticated backend analysis:

**Output types** (things supermodel tracks that parseltongue currently does not):
- `callerCount` — how many unique callers
- `transitiveDeadCount` — dead code induced transitively (a calls b, b is dead → a's transitive impact)
- `symbolLevelDeadCount` — separate count at symbol level vs file level
- `nearestTestedCaller` — the nearest tested function in the call graph (by hops)
- `testedSiblings` — other functions in the same file that ARE tested (with which test file)
- `suggestedTestFile` — which test file to add a test to

**Confidence levels**: `high`, `medium`, `low` — with emoji badges in PR comments (🔴, 🟠, 🟡)

**Integration tests reveal the async pattern**:
```text
1. Submit ZIP + idempotency key → 202 Accepted + jobId
2. Poll GET with same key → pending/processing/completed/failed
3. retryAfter field controls backoff (up to 90 iterations, 120s timeout)
4. On completed → return result.deadCodeCandidates[]
```

### 1.5 arch-docs: Underrated Static Site Generator

The `arch-docs` tool (Go, v1.0.0, Feb 2026) generates full architecture documentation websites from any codebase using the Supermodel API. Key features:

- **Entity pages**: Each function/class gets its own page with dependency diagram (Mermaid), force graph (D3), and arch map
- **llms.txt generation**: Markdown file listing all entities with URLs for LLM consumption
- **Taxonomy navigation**: Browse by node type, language, domain, file extension
- **Treemap + circular packing charts**: D3.js visualizations of codebase composition
- **Hub function detection**: Functions with 3+ callers ranked by in-degree centrality
- **GitHub Pages ready**: Automatic path prefix adjustment, sitemaps, RSS, Open Graph

This is a complete **public-facing documentation pipeline** for any codebase. Single GitHub Action.

### 1.6 TypeScript SDK: API v0.9.6 Capabilities

The SDK reveals analysis capabilities Supermodel has built that Parseltongue doesn't have:

| Analysis | SupermodelIR type | Parseltongue equivalent |
|----------|-------------------|------------------------|
| Impact analysis | `AffectedFunction` (file, name, distance hops, relationship) | blast-radius (partial) |
| Circular deps | `CircularDependencyCycle` (severity, breakingSuggestion) | circular-dependency-detection-scan |
| Test coverage | `TestedFunction` / `UntestedFunction` (with test gap analysis) | NONE |
| Domain classification | `DomainClassificationResponse` (full domain→subdomain→function/class mapping) | semantic-cluster-grouping-list (basic) |
| Dead code (async) | `CodeGraphEnvelopeAsync` (pending/processing/completed/failed) | NONE |

**New to parseltongue from this**: `breakingSuggestion` per cycle — supermodel tells you *which import edge to remove* to break a cycle. Parseltongue v1.6 only lists the cycles, not how to fix them.

### 1.7 VS Code Extension: Scale and Patterns

The extension (9.4 MB React bundle) is large but well-architected:

- **XState state machines** for all multi-step workflows — this is the right choice for complex async flows
- **Physics web worker** (`force-worker.js`, 488 KB) — D3 force simulation off the main thread
- **R-tree spatial indexing** (`rbush`) — efficient hit detection on the canvas
- **Hybrid keyboard shortcuts**: hover takes priority over selection (150ms delay)
- **Dual parser modes**: tree-sitter (accurate) with grep fallback (degraded)
- **Delta reconciliation**: `classifyCodeGraphDelta` reuses domain assignments incrementally
- **ZIP batching**: configurable batch size (10-500 files) to avoid OOM

---

## 2. What v200 Can Learn

### 2.1 The SupermodelIR Pattern → v200 Entity Store Design

**Lesson**: Build one unified IR that all tools produce and consume.

Supermodel's IR is their most powerful design decision. Every analysis — dead code, domains, impact, coverage — emits into the same graph schema. This means you can run dead code analysis AND domain classification AND impact analysis on the same graph without re-parsing.

**v200 application**: The `rust-llm-store-runtime` crate should define a `ParseltongueSuperIR` equivalent — the shape that every analysis emits into and every query reads from. Currently v200 design has separate entity/edge types but no unified "IR envelope" that stamps metadata (analysisStartTime, fileCount, languages[], schemaVersion).

**Concrete addition**:
```rust
struct ParseltongueIR {
    repo: String,
    schema_version: String,
    generated_at: DateTime<Utc>,
    summary: IRSummary,          // filesProcessed, classes, functions, types
    stats: IRStats,              // nodeCount, relationshipCount, nodeTypes, relTypes
    metadata: IRMetadata,        // analysisStartTime, analysisEndTime, languages
    domains: Vec<DomainSummary>, // domain classification result
    // graph lives in CozoDB — this is the envelope
}
```

### 2.2 IndexedGraph Multi-Index Pattern → Query Performance

**Lesson**: Pre-build 8 indexes at load time; queries are then O(1).

The MCP server's `IndexedGraph` maintains eight concurrent indexes over the same raw data. This is why `symbol_context` responds in sub-milliseconds even on large codebases. Parseltongue currently relies on CozoDB queries for everything — correct, but not always as fast.

**v200 application**: For the MCP server path specifically, consider building an in-memory `IndexedGraph`-equivalent from the CozoDB data at startup. For the common query patterns (fuzzy name lookup, caller/callee adjacency, domain membership), hot-loading these into Rust `HashMap`s would yield dramatically faster MCP tool responses.

**Priority index to add first**:
```rust
struct HotCache {
    name_index: HashMap<String, Vec<EntityKey>>,  // lowercase name → keys
    call_adj:   HashMap<EntityKey, Vec<EntityKey>>, // call graph outgoing
    call_radj:  HashMap<EntityKey, Vec<EntityKey>>, // call graph incoming (callers)
    domain_index: HashMap<String, Vec<EntityKey>>,  // domain → members
}
```

### 2.3 Three-Tier Cache Pattern → MCP Server Startup

**Lesson**: Connect transport immediately; do expensive work after handshake.

Supermodel's MCP server hits `StdioServerTransport.connect()` BEFORE starting precache. This ensures the 60-second MCP timeout is never triggered. Only after the handshake is complete does it start loading the graph.

**v200 application** (critical): The v200 MCP server must do the same. If graph loading from CozoDB takes >10 seconds on a large repo, the MCP client will time out. The v200 MCP server should:
1. Start, connect transport immediately (< 1 second)
2. Respond with "warming up" on first tool call if cache not ready
3. Load graph index into memory asynchronously
4. Subsequent calls hit the in-memory cache

### 2.4 Dead Code Hunter → v200 `find_dead_code_analysis` Endpoint

**Lesson**: Dead code analysis is a **first-class analysis type**, not a query.

Supermodel has a separate async analysis flow: upload ZIP → job queued → poll for results → get candidates with callerCount, confidence, reason, nearestTestedCaller. This is much more sophisticated than "entity with 0 reverse-callers."

**v200 application**: The HTTP API already has `reverse-callers-query-graph`. Extend it with:

```
GET /dead-code-candidates-list?min_confidence=high&lang=rust
```

Response includes:
- `callerCount: 0` (confirmed dead)
- `confidence: high|medium|low` based on whether the entity is exported/public (public = medium, private = high confidence dead)
- `reason: "No callers found in call graph"` or `"Only called from test files"`
- `nearestTestedCaller` if applicable
- `suggestedAction: "Remove function" | "Add test" | "Mark as entry point"`

The `high/medium/low` confidence split is the key insight: **public functions that appear unused might be library API** (medium confidence dead). Private functions with 0 callers are high confidence dead.

### 2.5 breakingSuggestion for Cycles → HTTP Endpoint Enhancement

**Lesson**: Don't just detect cycles — suggest how to fix them.

Supermodel's `CircularDependencyCycle.breakingSuggestion` tells you which specific import to remove. This is the difference between a report and a tool.

**v200 application**: The `/circular-dependency-detection-scan` endpoint currently returns cycles. Extend it with:
```json
{
  "cycle_id": "cycle-1",
  "files": ["a.rs", "b.rs"],
  "severity": "high",
  "breaking_suggestion": {
    "remove_edge": { "from": "a.rs", "to": "b.rs" },
    "import_to_remove": "use crate::b::SomeStruct;",
    "reason": "Removing this import breaks the cycle with minimum impact (1 callee)"
  }
}
```

The algorithm: for each cycle, find the edge whose callee has the fewest other callers — that's the lowest-impact cut.

### 2.6 Domain Classification → v200 Subdomain Taxonomy

**Lesson**: `semantic-cluster-grouping-list` is coarse. Domain + subdomain classification is the right granularity.

Supermodel's `DomainClassificationResponse` produces:
- Domain (e.g., "Billing")
- Subdomain (e.g., "Stripe Integration")
- Files assigned to subdomain
- Functions assigned to subdomain
- Classes assigned to subdomain
- Inter-domain relationships with strength and reason

**v200 application**: Upgrade `semantic-cluster-grouping-list` to emit a two-level taxonomy:
```
GET /domain-classification-full-taxonomy
```
This requires LLM calls or unsupervised clustering at build time, not at query time. Supermodel does this server-side with OpenRouter. v200 could do this in `rust-llm-graph-reasoning` using Ascent Datalog to propagate domain assignments via import graph.

### 2.7 Test Coverage Mapping → New v200 Analysis

**Lesson**: Static call graph reachability from test files → test coverage without instrumentation.

Supermodel's `TestCoverageMapResponse` computes which production functions are reachable from test functions using the static call graph. No `cargo test --coverage` needed. This is purely structural analysis.

**v200 application** — this is achievable in v200 with existing data:
```
GET /test-coverage-static-reachability
```
Algorithm using existing CozoDB data:
1. Find all entities where `file_path LIKE '%test%' OR name LIKE 'test_%'` → test functions
2. BFS/DFS through `calls` edges from test functions → set of reachable production functions
3. `testedFunctions` = reachable set, `untestedFunctions` = production - reachable
4. Coverage % = |tested| / |production|

This is pure graph traversal — no new parsing needed. Could be added to v200 in one sprint.

### 2.8 arch-docs → v200 Documentation Export

**Lesson**: Generate a complete static documentation site as a v200 output format.

Supermodel's `arch-docs` generates entity pages, navigation taxonomy, search, sitemaps, RSS, and `llms.txt` from the graph. The `llms.txt` file (markdown listing all entities with URLs) is particularly clever — it's a machine-readable index for LLMs.

**v200 application**: The v200 HTTP API already has all the data. A lightweight `arch-docs`-equivalent would:
1. Call `GET /code-entities-list-all` and `GET /dependency-edges-list-all`
2. For each entity, generate a markdown file
3. Generate `llms.txt` (entity list with key facts)
4. Generate a `mkdocs.yml` or Docusaurus config
5. Emit as `parseltongue export-docs --format mkdocs`

The `llms.txt` standard is growing — being able to emit it is a near-zero-cost addition to the HTTP API.

### 2.9 Async Job Pattern → v200 Large Repo Strategy

**Lesson**: For repos >50MB, synchronous analysis fails. Async job pattern is mandatory.

Supermodel's API uses `CodeGraphEnvelopeAsync` with polling for all analysis. The pattern:
- POST → 202 Accepted + `{ jobId, status: "pending" }`
- GET (same key) → `{ status: "processing", retryAfter: 30 }`
- GET → `{ status: "completed", result: SupermodelIR }`

**v200 application**: The current v200 design does synchronous ingest. For repos >10k files this will block for minutes. Consider adding an optional async mode:
```bash
parseltongue pt01-folder-to-cozodb-streamer . --async
# Returns immediately with: Workspace ID: parseltongue20260222143022
# Background process continues ingesting
# HTTP API returns { "status": "ingesting", "progress": { "files_done": 234, "files_total": 3412 } }
```

---

## 3. TUI: Minimal Effort, Maximum Gain

The question is: **can a TUI do more than CLI output, with very limited effort?**

The answer is yes — and Supermodel's architecture reveals exactly what "limited effort" means.

### 3.1 What Supermodel Reveals About TUI Appetite

Supermodel chose VS Code extension (full webview, React, D3, 9.4 MB bundle) as their primary interactive surface. This is maximum effort. Their secondary surface is MCP (zero UI). There is no middle ground — no TUI.

This is a gap. Terminal-first developers (the exact Parseltongue target user) often prefer `ratatui`/`tui-rs` UIs over webviews or CLIs. The sweet spot is:

```text
CLI output     TUI               VS Code webview
    │           │                      │
  zero UI    medium UI            full UI
  zero cost   medium cost         high cost
    │           │                      │
  grep-like  interactive         D3 graphs
  no state   stateful, fast      heavy, slow startup
```

### 3.2 What a v200 TUI Could Be (Very Limited Effort)

A v200 TUI is **not** a graph visualizer. It is a **query navigator** — a thin interactive wrapper over the existing HTTP API.

```text
┌─────────────────────────────────────────────────────────────────────┐
│  parseltongue tui  --db "rocksdb:parseltongue20260222/analysis.db"  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  [ Search: auth_____________________________] [lang: All ▼]        │
│                                                                     │
│  ┌── Results ─────────────────────────────────────────────────┐    │
│  │ ▶ fn   handle_auth_request    backend/api/auth.rs:45       │    │
│  │   fn   auth_middleware        backend/middleware.rs:12     │    │
│  │   fn   validate_user          backend/auth/logic.rs:34     │    │
│  │   struct LoginCredentials     backend/auth/types.rs:8      │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                     │
│  ┌── Detail ──────────────────────────────────────────────────┐    │
│  │  handle_auth_request  •  fn  •  rust  •  89 tokens         │    │
│  │  Callers: auth_middleware                                   │    │
│  │  Callees: validate_user, generate_token                     │    │
│  │  [b] blast radius  [c] copy key  [e] open editor  [q] quit │    │
│  └────────────────────────────────────────────────────────────┘    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Implementation cost estimate**: 2-3 sprints using `ratatui` (maintained, actively used by Cargo, Lazygit, etc.)

**The full TUI feature set at "limited effort" level**:

| Feature | HTTP API call | Effort |
|---------|---------------|--------|
| Fuzzy search box | `GET /code-entities-search-fuzzy?q=` | 1 day |
| Entity detail panel | `GET /code-entity-detail-view/{key}` | 1 day |
| Callers list | `GET /reverse-callers-query-graph?entity=` | 0.5 day |
| Callees list | `GET /forward-callees-query-graph?entity=` | 0.5 day |
| Copy key to clipboard | local | 0.5 day |
| Open in $EDITOR | local | 0.5 day |
| Blast radius (text tree) | `GET /blast-radius-impact-analysis?entity=` | 1 day |
| Stats overview | `GET /codebase-statistics-overview-summary` | 0.5 day |
| Cycles list | `GET /circular-dependency-detection-scan` | 0.5 day |

**Total: ~6 days for a genuinely useful TUI that terminal users will prefer over the web.**

### 3.3 TUI as "The CLI That Teaches Itself"

Supermodel's philosophy is "teach the user the CLI by showing the command being run." A v200 TUI can do the same:

```text
  Press [e] to open in editor:
  $ code backend/api/auth.rs:45
  ──────────────────────────────
  Press [b] for blast radius:
  $ curl "http://localhost:7777/blast-radius-impact-analysis?entity=rust%7C%7C%7Cfn%7C%7C%7C...&hops=2"
```

Every TUI action shows the equivalent CLI command. This turns the TUI into a CLI teacher, not a replacement.

### 3.4 TUI as the Instance Manager (Tauri Replacement?)

The v200 Tauri scope (decided: process manager only) is actually a natural fit for a TUI, not a GUI app:

```text
parseltongue tui instances
┌──────────────────────────────────────────────────────────────────────┐
│  PARSELTONGUE INSTANCES                                              │
├──────────────────────────────────────────────────────────────────────┤
│  myapp-20260221      ● HTTP :7777    [s] stop   [l] logs   [m] mcp  │
│  myapp-20260115      ○ idle          [S] start   ──────────────────  │
│  other-20260110      ○ idle          [S] start   ──────────────────  │
│                                                                      │
│  [+] add workspace    [q] quit                                       │
│                                                                      │
│  ┌── Log tail: myapp-20260221 ─────────────────────────────────────┐ │
│  │ [14:30:25] GET /code-entities-list-all → 200 (43ms)            │ │
│  │ [14:31:01] GET /blast-radius-impact-analysis → 200 (12ms)      │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

This replaces Tauri entirely for the instance manager use case. Terminal users prefer this over a native app. The Tauri app stays on the roadmap but is no longer the only non-CLI interface.

**Implementation**: `ratatui` + `tokio::process::Command` for child process management. ~3 days.

---

## 4. Competitive Gap Analysis

### What Supermodel Has That Parseltongue v200 Does Not (Yet)

| Supermodel Capability | Parseltongue v200 Plan | Gap |
|----------------------|----------------------|-----|
| Dead code analysis with confidence | Not in v200 plan | ★★★ High value, achievable |
| `breakingSuggestion` for cycles | HTTP returns cycles only | ★★ Medium effort |
| Test coverage (static call graph) | Not in v200 plan | ★★★ High value, pure graph traversal |
| Domain + subdomain classification | `semantic-cluster-grouping-list` | ★★ Needs two-level taxonomy |
| `llms.txt` export | Not in v200 plan | ★ Low effort, high value |
| `AffectedFunction.distance` hops | blast-radius has hops | ✓ Equivalent |
| Async analysis for large repos | Synchronous only | ★★ Needed for 10k+ file repos |
| `architecture.md` auto-generation | Not in v200 plan | ★ Low effort (single endpoint) |
| GitHub Action delivery | cargo install only | ✓ Different market (local-first) |
| Subscription billing | OSS / free | ✓ Intentional difference |

### What Parseltongue v200 Has That Supermodel Does Not

| Parseltongue Capability | Supermodel Status | Strategic Value |
|------------------------|------------------|----------------|
| Fully local (no cloud dependency) | Cloud-only | Privacy, latency |
| Rust implementation (embeddable) | JS/TS | In-process use |
| Rust-analyzer semantic enrichment | Grep/tree-sitter only | Exact type-level analysis |
| Cross-language boundary detection | Not advertised | Polyglot key feature |
| MCP server from local store | Remote API proxy | Privacy, speed |
| Ascent Datalog for graph reasoning | Not present | Compositional analysis |
| CozoDB + RocksDB (structured queries) | Neo4j (graph only) | Relational + graph |
| SQALE tech debt scoring | Not present | Code health metric |
| K-core decomposition | Not present | Architectural layering |
| Leiden community detection | Not present | Better clustering |

---

## 5. Three Recommendations

### Recommendation 1 (Immediate): Add `llms.txt` Export — 1 Day

Add `GET /llms-txt-export-format` to the HTTP API. Returns markdown-formatted entity index:
```markdown
# myapp — Parseltongue v200 Graph

> 846 entities, 5,418 edges, 3 languages

## Functions
- [handle_auth_request](http://localhost:7777/code-entity-detail-view/rust%7C%7C%7Cfn%7C%7C%7C...): Handles POST /api/auth, returns AuthToken
- [auth_middleware](http://localhost:7777/code-entity-detail-view/...): Axum middleware, validates Bearer token
...
```

This is 1 day of work and immediately differentiates v200 as LLM-friendly by a widely-understood standard.

### Recommendation 2 (Sprint): Add Dead Code Candidates Endpoint — 3 Days

```
GET /dead-code-candidates-list?min_confidence=high&lang=rust&max_results=50
```

Algorithm:
1. Query entities where `reverse_caller_count = 0`
2. Cross-reference with `is_public` flag → confidence
3. Filter test/generated files
4. Return sorted by confidence DESC, callerCount ASC

This is pure CozoDB query work on existing data — no new parsing.

### Recommendation 3 (Quarter): Build the TUI — 6 Days

Build `parseltongue tui` using `ratatui`. Start with:
1. Query navigator (search + entity detail)
2. Instance manager (process start/stop + log tail)

This gives parseltongue an interactive interface that terminal-first developers (the exact target user) prefer over VS Code webviews or Tauri apps. It also validates the HTTP API as the primary interface layer.

---

## 6. Summary

Supermodel is the best publicly-visible implementation of "code graph as a platform." Their SupermodelIR pattern, IndexedGraph multi-index, and three-tier MCP cache are directly applicable to v200. Their GitHub Action delivery, arch-docs SSG, and `llms.txt` output are low-hanging fruit.

Parseltongue v200's advantages — local-first, Rust-analyzer enrichment, cross-language boundaries, Ascent Datalog reasoning — are real and defensible. The TUI is an achievable 6-day project that fills the UX gap between CLI and Tauri without the complexity of a native app.

The single most important lesson: **build one IR, index it eight ways, cache it three tiers**.

---

## Appendix A: OpenAPI Spec Deep-Dive (v0.9.6, Agent-Complete)

*From the completed openapi-spec agent — key additions not covered above.*

### A1. All 9 API Endpoints (all POST, URL-versioned `/v1/`)

| Path | operationId | Summary |
|------|-------------|---------|
| `POST /v1/graphs/dependency` | `generateDependencyGraph` | File-level import graph |
| `POST /v1/graphs/call` | `generateCallGraph` | Function-level call graph |
| `POST /v1/graphs/domain` | `generateDomainGraph` | LLM-powered domain classification |
| `POST /v1/graphs/parse` | — | AST parse-tree relationships |
| `POST /v1/graphs/supermodel` | — | Full SupermodelIR (all graphs bundled + embeddings) |
| `POST /v1/analysis/dead-code` | — | Unreachable code detection |
| `POST /v1/analysis/test-coverage-map` | — | Static call-graph test coverage |
| `POST /v1/analysis/circular-dependencies` | — | Tarjan SCC cycle detection |
| `POST /v1/analysis/impact` | `generateImpactAnalysis` | Blast radius with LLM domain enrichment |

**Idempotency-Key is a required header on every request** — echoed back as `X-Request-Id`.

### A2. Metered Billing via Response Header

```
X-Usage-Units: <float>   # billing units consumed by this call
```

This is how Supermodel monetises per-analysis rather than per-seat. v200 doesn't need this (OSS) but it reveals their cost model: domain classification and LLM function descriptions cost more units than raw graph generation.

### A3. `BlastRadius.riskScore` — 4-Level Severity (v200 Has None)

Supermodel's impact analysis returns a **risk score per blast radius**, not just hop counts:

```typescript
interface BlastRadius {
  directDependents: number;
  transitiveDependents: number;
  affectedFiles: AffectedFile[];
  affectedDomains: string[];
  riskScore: 'low' | 'medium' | 'high' | 'critical';  // ← v200 missing this
  riskFactors: string[];   // e.g. ["cross-domain boundary", "entry point affected"]
}
```

**v200 application**: Add `risk_score` to `/blast-radius-impact-analysis` response. Algorithm:
- `critical` = transitive impact crosses a domain boundary AND hits an entry point
- `high` = crosses domain boundary OR hits >20 unique files
- `medium` = stays within domain, hits >5 files
- `low` = stays within domain, <5 files

### A4. `AffectedEntryPoint` Classification (PR Gate Signal)

```typescript
interface AffectedEntryPoint {
  file: string;
  name: string;
  type: 'route_handler' | 'module_export' | 'main_function' | 'event_handler';
}
```

If a change's blast radius reaches an entry point, it's automatically `high` or `critical`. This is the PR-gate signal: "this PR touches code that routes an HTTP request." v200's blast-radius currently returns all reachable entities; adding entry-point detection (route_handler = axum/actix handler, module_export = `pub` fn at crate root) would make the endpoint CI-ready.

### A5. `SupermodelArtifact.kind = 'embedding'` — Vector Search Hidden Feature

```typescript
interface SupermodelArtifact {
  id: string;
  kind: 'graph' | 'summary' | 'embedding';  // ← embedding here
  label: string;
  metadata: Record<string, any>;
}
```

The SupermodelIR bundle can include vector embeddings of the graph. This means `/v1/graphs/supermodel` is not just a graph dump — it's a retrieval-augmented search index. v200 doesn't do this today, but CozoDB supports vector similarity search; a future v210 could generate embeddings per entity using a local model (e.g., `nomic-embed-code`).

### A5b. Polling-via-Resubmission Pattern (No Job-ID Endpoint Needed)

The dead-code-hunter's polling loop reveals a surprisingly clean API design:

```typescript
// Loop up to 90x (15 min window), always re-POST the same ZIP + idempotency key
const response = await api.generateDeadCodeAnalysis({ idempotencyKey, file: zipBlob });
if (response.status === 'completed') return response.result;
if (response.status === 'failed') throw new Error(response.error);
await sleep(response.retryAfter * 1000);
```

There is **no `GET /jobs/{id}` endpoint**. The client re-POSTs the full request (same blob, same key) and the server deduplicates via idempotency key, returning the current job state. This avoids a separate status endpoint and simplifies both client and server.

**v200 application**: If v200 adds async ingest, adopt the same pattern. The `pt01-folder-to-cozodb-streamer` idempotency key can be `{workspace_id}:{git_commit_sha}`. Re-running the same ingest command returns existing progress rather than re-starting.

### A5c. Three More MCP Implementation Patterns (MCP Agent Complete)

**Pattern: Overview-as-system-prompt (not a registered tool)**

The `overview` tool is NOT callable by the AI client. Instead, its rendered output is injected directly into `McpServer._instructions` (via a private field hack) at startup:

```typescript
(this.server.server as any)._instructions =
  (current || '') + '\n\n' + renderOverview(graph) + testHint;
```

The AI receives the full architectural map — domains, hub functions, file/function counts — in the system prompt before the conversation starts. Zero tool-call turns consumed. This is the most elegant optimization in their stack.

**v200 application**: When the v200 MCP server starts, call `GET /codebase-statistics-overview-summary` and inject that summary into the MCP server's system instructions. Every agent session begins with a 2K-token codebase overview automatically.

**Pattern: Hardcoded "max 3 MCP calls" behavioral instruction**

The server instructions string contains explicit behavioral guidance embedded for the AI:
> "Stop calling MCP tools. Start editing by turn 3. Max 3 MCP calls total."

This is a meta-instruction that shapes AI behavior, not enforced programmatically. The reasoning: if the AI can't figure out what to edit after 3 `symbol_context` lookups, additional lookups rarely help.

**v200 application**: Add a similar efficiency constraint to the v200 MCP server instructions. Something like: "Use `fuzzy_search` once to orient, then `entity_detail` for specifics. Start coding by turn 2. Do NOT call `blast_radius` during exploration — use it only before committing changes."

**Pattern: Precise LRU limits (from constants.ts)**

- `MAX_GRAPHS = 20` — evict LRU graph when 21st is loaded
- `MAX_NODES = 1_000_000` — total node capacity across all cached graphs
- `TTL = 3600s` (1 hour) — evict stale entries on next access
- `MAX_OVERVIEW_DOMAINS = 10`
- `MAX_OVERVIEW_HUB_FUNCTIONS = 10` (functions with ≥3 callers, by in-degree)
- `MAX_SYMBOL_MATCHES = 3` — return top 3 matches per symbol name
- `MAX_SOURCE_LINES = 80` — source code excerpt truncated at 80 lines

**v200 application**: Define explicit constants in `rust-llm-store-runtime` for the hot cache:
- `HOT_CACHE_MAX_ENTITIES: usize = 500_000` (larger repos need bigger cache)
- `HOT_CACHE_TTL: Duration = Duration::from_secs(7200)` (2hr — matches typical work session)

### A6. Impact Analysis Designed for PR Gates (CI/CD)

The `/v1/analysis/impact` endpoint takes two inputs:
```bash
curl -X POST /v1/analysis/impact \
  -F "file=@repo.zip" \          # full repo ZIP
  -F "diff=@changes.diff" \      # git diff output (unified format)
  -H "X-Targets: ..." \          # optional: specific files/functions to analyze
```

v200 could add a `--diff` flag to the blast-radius endpoint:
```bash
curl "http://localhost:7777/blast-radius-impact-analysis?\
  diff=$(git diff origin/main...HEAD | base64)&hops=3"
```
This turns parseltongue into a local PR impact gate — no cloud needed.

### A7. `ImpactGlobalMetrics` — Repo-Level Criticality Map

```typescript
interface ImpactGlobalMetrics {
  mostCriticalFiles: CriticalFile[];       // [{ file, dependentCount, riskScore }]
  crossDomainDependencies: CrossDomainDependency[];  // [{ from, to, edgeCount }]
}
```

This is a **pre-computed criticality map** — independent of any specific change. v200 equivalent would be a new endpoint:
```
GET /critical-files-ranking-view?top=20
```
Returns files sorted by `transitive_dependent_count DESC` — the files where any change has the highest blast radius. This is pure CozoDB query on existing data.

---

*Sources: Decompiled from competitor-research/supermodel/extracted/ (gitignored) + GitHub repos supermodeltools/{openapi-spec,mcp,arch-docs,dead-code-hunter,typescript-sdk}. OpenAPI spec agent confirmed API v0.9.6 with 9 endpoints, 28 schemas, idempotency key requirement, and metered billing via X-Usage-Units header.*

---

## Appendix B: arch-docs Deep-Dive (Go SSG, Build Pipeline, ADRs)

*From the completed arch-docs agent — key additions covering the Go static site generator internals.*

### B1. Go Module Architecture — Internalized graph2md and pssg

The arch-docs tool is a single Go 1.25 binary with the following module structure:

```text
supermodeltools/arch-docs/
├── main.go                          # Entry point, CLI flag parsing, orchestration
└── internal/
    ├── graph2md/                    # Graph-to-Markdown pipeline (5-pass, 8 indices)
    │   └── ...
    └── pssg/                        # Parseltongue Static Site Generator (21-step build)
        └── ...
└── templates/                       # Go html/template files for entity pages
```

The critical architectural decision here is **internalization** (captured in ADR-1, Feb 17 2026): both `graph2md` and `pssg` were originally separate packages but pulled into `internal/` to enforce encapsulation. Nothing outside the `arch-docs` binary can import them. This is the idiomatic Go answer to "how do you prevent your pipeline internals from becoming a public API by accident."

**v200 application**: Parseltongue's `parseltongue-core` crate is the Rust equivalent, but it is currently a _public_ crate depended on by `pt01` and `pt08`. As long as we do not publish to crates.io, this is fine. However, if Parseltongue ever ships as a library, the `internal/` pattern translates to Rust `pub(crate)` visibility on all pipeline modules. The graph2md analogue would be `parseltongue-core::graph_pipeline` gated at `pub(crate)`.

### B2. The 21-Step pssg Build Pipeline

The `pssg` module runs a deterministic 21-step build sequence. Each step is a pure function over a shared `BuildContext` struct, making the pipeline trivially testable and resumable:

```text
Step  1: Load and validate SiteConfig (16 top-level sections)
Step  2: Create output directory tree
Step  3: Copy static assets (CSS, JS, favicon)
Step  4: Run graph2md pipeline → produce MarkdownEntity[] for all nodes
Step  5: Apply auto-tagging rules to every entity
Step  6: Build TaxonomyConfig index (domain → entity[] map)
Step  7: Generate search index JSON (tokenized, pre-compiled)
Step  8: Render entity detail pages (32-goroutine concurrent renderer)
Step  9: Render domain index pages
Step 10: Render subdomain index pages
Step 11: Render home page (top-level index.html)
Step 12: Render API reference page
Step 13: Render 404 page
Step 14: Inject D3.js v7 force-graph JSON blobs (31-node cap enforced here)
Step 15: Inject Mermaid diagram stubs (15-node cap enforced here)
Step 16: Render metric bar chart data (per-entity coupling scores)
Step 17: Render file position indicator SVG (line-range in source file)
Step 18: Generate arch-map SVG (full architectural overview, separate from D3)
Step 19: Generate llms.txt (LlmsTxtConfig section drives content)
Step 20: Generate sitemap.xml (chunked at 50K URLs per file)
Step 21: Generate RSS feed
```

**The 32-goroutine concurrent entity renderer (Step 8)** is the performance-critical path. Each goroutine processes one entity page end-to-end: template execution, D3 JSON embedding, Mermaid stub injection, and file write. The goroutine pool is sized at `min(32, runtime.NumCPU()*4)`.

**Node caps are enforced at injection time** (Steps 14-15), not at graph construction time. The full graph index is built with no cap; only the per-page visualization payload is truncated. This means the search index and sitemap are always complete — only the interactive widget on each page is limited.

**v200 application**: The 21-step pattern maps directly to how parseltongue's pt01 ingest pipeline should be structured. Today pt01 is a monolithic function. Splitting it into numbered steps over a shared `IngestContext` struct would make each step independently testable (TDD red-green per step) and enable resumable ingests — skip steps whose output files already exist on disk.

### B3. The 7 Architecture Decision Records and Their v200 Implications

arch-docs ships 7 ADRs in its repository. Each is a short dated document with Status, Context, Decision, and Consequences sections.

**ADR-1: Internalize graph2md and pssg into internal/ (Feb 17 2026)**
- Decision: Move both packages from top-level to `internal/` to prevent accidental public API surface.
- Status: Accepted.
- v200 implication: Validate that `parseltongue-core` pub items are intentional exports. Any type only used within a single crate should be `pub(crate)`.

**ADR-2: 202 polling + idempotency key (not webhooks)**
- Decision: Long-running analyses (dead-code, domain classification) return HTTP 202 immediately. Client re-POSTs with the same idempotency key to poll. No webhooks, no separate `/jobs/{id}` endpoint.
- Status: Accepted.
- v200 implication: If pt01 ingest ever becomes async (large repos), adopt this exact pattern. The idempotency key is `{workspace_slug}:{git_sha}`. Re-running `pt01` on an unchanged repo returns cached status rather than re-ingesting.

**ADR-3: Single /v1/graphs/supermodel call (not 5 separate)**
- Decision: Rather than requiring clients to call dependency, call, domain, parse, and embedding endpoints separately and stitch results, expose one "supermodel" endpoint that returns all five as a bundle.
- Status: Accepted.
- v200 implication: Add a `/full-analysis-bundle-export` endpoint to pt08 that returns entity list + edges + clusters + SCC + centrality in one JSON payload. Eliminates 6+ sequential API calls from MCP tool implementations.

**ADR-4: overview as instructions injection, not registered tool**
- Decision: The architectural overview (domain map, hub functions, entity counts) is injected into MCP server system instructions at startup, not exposed as a callable tool.
- Status: Accepted.
- v200 implication: This is the single highest-leverage MCP optimization available to v200. Inject the output of `/codebase-statistics-overview-summary` into MCP `_instructions` at server boot. Every AI session begins pre-oriented with zero tool-call turns consumed.

**ADR-5: Three-tier cache (disk precache -> LRU -> API)**
- Decision: Hot data lives in a bounded LRU (20 graphs, 1M nodes, 1hr TTL). Warm data lives in a disk precache directory. Cold data fetches from the API and populates both tiers.
- Status: Accepted.
- v200 implication: Map to Parseltongue tiers: hot = in-process `DashMap` of parsed entity structs; warm = the RocksDB-backed CozoDB file (already exists); cold = re-run `pt01` ingest. Define explicit constants: `HOT_CACHE_MAX: usize = 500_000`, `HOT_CACHE_TTL: Duration = Duration::from_secs(7200)`.

**ADR-6: Docker multi-stage, CGO disabled, static binary**
- Decision: The arch-docs Docker image uses a multi-stage build. Stage 1 compiles with `CGO_ENABLED=0 GOFLAGS=-trimpath`. Stage 2 is `FROM scratch`. Final image is a single static binary with no libc dependency.
- Status: Accepted.
- v200 implication: Parseltongue already builds a static Rust binary but the Dockerfile (if added in v200) should follow the same multi-stage `FROM scratch` pattern. The CozoDB RocksDB backend is the only FFI dependency — verify `RUSTFLAGS="-C target-feature=+crt-static"` produces a fully static binary on Linux before adding Docker support.

**ADR-7: llms.txt generation**
- Decision: Every arch-docs site build produces a `/llms.txt` file following the emerging `llms.txt` standard. It lists all entities with their type, path, and a one-sentence description, structured for direct inclusion in an AI context window.
- Status: Accepted.
- v200 implication: This is a concrete, low-effort v200 deliverable. Add a `/llms-txt-export-generate` endpoint to pt08 that produces a standards-compliant `llms.txt` document from the CozoDB entity store. Format per the spec: `# Title`, then `## Section` headings per domain, then `- [Entity](url): description` lines.

### B4. Visualization Architecture (D3.js v7, Mermaid, Search Index)

**Frontend stack**: No bundler, no build step. D3.js v7 and Mermaid are loaded from CDN. All interactivity is vanilla JS embedded in Go templates.

**Five visualizations per entity page**:

| Visualization | Technology | Node cap | Data source |
|---------------|-----------|----------|-------------|
| Force graph (interactive) | D3.js v7, CDN | 31 nodes | Pre-serialized JSON blob per page |
| Arch-map (static overview) | SVG, generated | None | Full graph, rendered at build time |
| Mermaid diagram (flow) | Mermaid, CDN | 15 nodes | Pre-serialized Mermaid DSL per page |
| Metric bar chart | D3.js v7 inline | N/A | Coupling/cohesion scores per entity |
| File position indicator | SVG, generated | N/A | Line-range within source file |

**Keyboard shortcut**: pressing `/` or `Ctrl+K` on any page opens a search modal. The search modal queries the pre-compiled JSON search index — no server round-trip.

**Search index scoring** (tokenized, pre-compiled at build time):
- Exact match on full entity name: score 100
- Prefix match on name token: score 50
- Substring match anywhere: score 10
- Results capped at 20 entries
- Index built as a flat JSON array, loaded once per page session

**v200 application**: Parseltongue's current `/code-entities-search-fuzzy` endpoint does live CozoDB queries. For the planned TUI or any future static export, pre-compiling a search index JSON at ingest time (Step 7 equivalent) would allow instant search in offline or read-only contexts. The scoring tiers (exact > prefix > substring) are a clean, implementable spec.

### B5. What v200 Can Learn from arch-docs Patterns

Five concrete, high-value lessons from the arch-docs deep-dive:

**Lesson 1: llms.txt as a first-class output (ADR-7)**

The `llms.txt` standard is emerging as the canonical way to expose a codebase to AI tooling. arch-docs generates one automatically on every build. v200 should treat `llms.txt` generation as a core feature of pt01 ingest — not a bolt-on. When `pt01` finishes ingesting, it should write a `llms.txt` file into the workspace directory alongside `analysis.db`. Any AI agent running in the repo can then do `read(llms.txt)` and be immediately oriented without calling the MCP server.

**Lesson 2: Concurrent entity rendering as a structural pattern**

The 32-goroutine pool for entity page rendering (Step 8) is not a micro-optimization — it is a structural statement that entity processing is embarrassingly parallel. In Rust terms, this is `rayon::par_iter()` over the entity list. Parseltongue's current pt01 ingest processes entities sequentially in the tree-sitter parse loop. Restructuring parse output collection as `Vec<EntityResult>` and then running enrichment (relationship extraction, tag application, CozoDB write batching) as a `par_iter()` would reduce ingest time proportionally to core count.

**Lesson 3: Auto-tagging rules as a queryable first-class attribute**

arch-docs applies four auto-tags to entities at build time:
- `High-Dependency`: entity has 5 or more incoming dependency edges
- `Many-Imports`: entity imports 5 or more other modules
- `Complex`: entity defines 10 or more functions
- `Isolated`: entity has zero dependency edges in either direction

These tags are stored on the entity node and exposed in search facets and the force-graph UI. v200 should compute equivalent tags at ingest time and store them as entity attributes in CozoDB. This enables CozoDB queries like `?[name, tags] := *entities{name, tags}, "High-Dependency" in tags` — which is not currently possible because tags do not exist as stored data.

**Lesson 4: Build pipeline as numbered steps over shared context**

The 21-step sequential pipeline with a shared `BuildContext` struct is a proven pattern for complex multi-output build processes. Every step is unit-testable in isolation. The context struct carries all mutable state, making it easy to inspect intermediate build state during debugging. v200's pt01 ingest should adopt this pattern: define an `IngestContext` struct carrying the parsed entity list, the CozoDB handle, the workspace path, and timing metadata. Each ingest phase (parse, enrich, tag, write, report) becomes a function `fn run_step_N(ctx: &mut IngestContext) -> Result<()>`.

**Lesson 5: No VS Code extension — AI integration is MCP-only**

A frequently cited assumption is that arch-docs will add a VS Code extension. The deep-dive confirms this is false. The AI integration surface is exclusively MCP (Claude Code, Cursor, Codex). No VS Code extension exists or is planned. This validates Parseltongue's v200 direction: build the best MCP server, not a VS Code plugin. The VS Code GUI surface is a distraction at this stage of the product; the MCP protocol is the correct abstraction layer for AI-native tooling.

---

*Appendix B sources: arch-docs agent deep-dive (2026-02-22), supermodeltools/arch-docs repository, 7 ADR documents, Go 1.25 module structure analysis.*
