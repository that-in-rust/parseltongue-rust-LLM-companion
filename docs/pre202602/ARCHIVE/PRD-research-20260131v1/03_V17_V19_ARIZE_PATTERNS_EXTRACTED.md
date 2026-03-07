# Parseltongue v1.7-v1.9: Arize Agent Harness Pattern Extraction

**Analysis Date**: 2026-01-31
**Source Material**: Arize Agent Harness Architecture Article (Aparna Dhinakaran)
**Foundation**: Assumes v1.6 complete (MCP, Unix piping, streaming implemented)
**Current Version**: v1.4.2
**Target Versions**: v1.7, v1.8, v1.9

---

## Executive Summary

Extracting **7 NEW PRD ideas** for Parseltongue v1.7-v1.9 by applying Arize's agent memory architecture patterns to our deterministic code graph. These features build on v1.6's MCP foundation and focus on **agent memory patterns** that make 200K context windows feel like infinite memory.

**Key Strategic Insight**: Parseltongue has a **deterministic code graph** (not generic files). The Arize patterns of preview/pointer, dynamic self-correction, and budget management can be supercharged when applied to **structured graph data** vs. raw text.

**Differentiator**: While Cursor/Claude/Alyx apply these patterns to file systems, Parseltongue applies them to **Interface Signature Graphs** - deterministic, structured, semantically rich.

---

## Arize Patterns Applied to Parseltongue

### Pattern Mapping Table

| Arize Pattern | Generic File System | Parseltongue ISG Graph |
|---------------|---------------------|------------------------|
| **Preview/Pointer** | File name + first 5 lines + file ID | Entity signature + SigHash pointer + full body on demand |
| **Dynamic Self-Correction** | "Results too large, backtrack" | "Query returns 500 entities (12K tokens), suggest narrower scope" |
| **Paging Through Results** | Page 1/10 of file list | Entity set 1/N with graph continuity markers |
| **SQL vs File System Tradeoff** | Remote query vs. local grep | In-DB graph query vs. exported subgraph JSON |
| **Dynamic Index Generation** | `find` creates file index | Temporary entity views for agent session |
| **Composable Tool Chains** | `grep \| sort \| uniq` | `search-entities \| filter-by-module \| rank-by-coupling` |
| **Context Budget Management** | Estimate file sizes | Pre-compute token costs from entity metadata |

---

## PRD Idea #1: Entity Preview Signature Pointers

**Priority**: P1 (Foundation for tiered responses)
**Version Target**: v1.7
**Effort Estimate**: 1.5 weeks
**Arize Pattern Applied**: Preview/Pointer (Alyx's truncated table + span IDs)

### Current Gap

Parseltongue returns **full entity details** (signature + body + metadata) in every response. For large query results (500+ entities), agents receive overwhelming detail (15-20K tokens) when they only need entity signatures to make decisions.

**Example**: `/code-entities-search-fuzzy?q=handle` returns:
```json
{
  "entities": [
    {
      "entity_key": "rust:fn:handle_request:src/server.rs:45-89",
      "signature": "pub fn handle_request(req: Request) -> Response",
      "body": "pub fn handle_request(req: Request) -> Response {\n    let auth = authenticate(&req);\n    // ... 40 more lines ...\n}",
      "dependencies": [...],
      "metrics": {...}
    }
    // ... 499 more entities with full bodies
  ]
}
```

**Token cost**: ~15K tokens for 500 entities
**Agent needs**: Often just signatures to decide which entity to drill into

### Proposed Solution

Implement **tiered response levels** with `?detail=preview|pointer|full`:

1. **Preview mode** (`?detail=preview`, default): Entity signature + metadata only
2. **Pointer mode** (`?detail=pointer`): Signature + SigHash (BLAKE3) for full retrieval
3. **Full mode** (`?detail=full`): Complete entity with body + dependencies

Add new endpoint `/code-entity-full-body-retrieve/{sighash}` for on-demand full retrieval.

### Acceptance Criteria

- [ ] All entity-returning endpoints support `?detail=preview|pointer|full` parameter
- [ ] Preview mode returns: `entity_key`, `signature`, `entity_type`, `file_path`, `lines` only
- [ ] Pointer mode adds: `sighash` (BLAKE3 hash of signature for stable lookup)
- [ ] Full mode returns: All current fields (body, dependencies, metrics)
- [ ] New endpoint `/code-entity-full-body-retrieve/{sighash}` returns full entity by hash
- [ ] Token reduction measured: Preview mode ≤ 10% of full mode
- [ ] MCP tools default to preview mode
- [ ] Backward compatibility: No `?detail` defaults to full (existing behavior)
- [ ] All tests passing (TDD: RED → GREEN → REFACTOR)

### Implementation Notes

**Crates affected**:
- `parseltongue-core` (entity serialization tiers)
- `pt08-http-code-query-server` (all entity-returning handlers)

**New modules needed**:
- `parseltongue-core/src/entity_preview_signature_serializer.rs` - Preview tier serialization
- `parseltongue-core/src/sighash_stable_pointer_generator.rs` - BLAKE3 signature hashing
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/entity_full_body_retrieve_handler.rs` - SigHash lookup

**Database schema changes**:
- Add `SigHashLookup` table: `{sighash: String, entity_key: String}`
- Populated during ingestion by `pt01-folder-to-cozodb-streamer`

### Agent Workflow Example

```bash
# Before (v1.6) - Full detail always
curl "http://localhost:7777/code-entities-search-fuzzy?q=handle"
# Returns: 500 entities × 30 tokens each = 15K tokens

# After (v1.7) - Preview mode default
curl "http://localhost:7777/code-entities-search-fuzzy?q=handle&detail=preview"
# Returns: 500 entities × 2 tokens each = 1K tokens (93% reduction)

# Agent sees signatures, decides on 3 interesting entities
curl "http://localhost:7777/code-entity-full-body-retrieve/blake3_abc123"
curl "http://localhost:7777/code-entity-full-body-retrieve/blake3_def456"
curl "http://localhost:7777/code-entity-full-body-retrieve/blake3_ghi789"
# Fetches only 3 full entities = 90 tokens total

# Total: 1K + 90 = 1,090 tokens vs. 15K tokens (92.7% reduction)
```

**Agent UX Impact**:
- Cursor can show signature list, let user click for full body
- Claude Code can scan 10× more entities in same context budget
- Alyx-style table with "expand to see implementation" UX

---

## PRD Idea #2: Query Token Budget Estimator

**Priority**: P1 (Self-correction enablement)
**Version Target**: v1.7
**Effort Estimate**: 1 week
**Arize Pattern Applied**: Dynamic Self-Correction (Claude detecting context overflow)

### Current Gap

Parseltongue executes queries and returns results with **no token cost warnings**. Agents blindly invoke queries that exceed context windows, then backtrack and retry with narrower scopes. No proactive guidance on result size.

**Example**: Agent asks "show all functions in this codebase"
- Query returns 3,500 functions
- Agent's context window: 200K tokens
- Query result: 105K tokens (52% of budget consumed by one query!)
- Agent realizes mid-processing and retries with filters

### Proposed Solution

Add **pre-query estimation** endpoint `/query-token-budget-estimate` that returns projected token count **before execution**. Use entity counts + avg tokens/entity heuristics. Add `X-Estimated-Tokens` response header to all endpoints.

Implement dry-run mode: `?dry_run=true` returns only token estimate, no actual data.

### Acceptance Criteria

- [ ] New endpoint `/query-token-budget-estimate` accepts same params as any query endpoint
- [ ] Returns: `{"estimated_tokens": 105000, "entity_count": 3500, "suggestion": "Add filter by module"}`
- [ ] All query endpoints include `X-Estimated-Tokens` response header
- [ ] Dry-run mode (`?dry_run=true`) returns estimate only, skips execution
- [ ] Token estimation accuracy: ±15% of actual (measured on 100 sample queries)
- [ ] Estimation uses cached entity counts (no full query execution)
- [ ] Suggestions provided when estimate > 50K tokens
- [ ] MCP tool `estimate_query_token_budget` exposes estimation
- [ ] All tests passing

### Implementation Notes

**Crates affected**:
- `parseltongue-core` (token estimation algorithms)
- `pt08-http-code-query-server` (all query handlers add headers + dry-run)

**New modules needed**:
- `parseltongue-core/src/token_budget_estimation_calculator.rs` - Heuristic token counting
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/query_token_budget_estimate_handler.rs`
- `parseltongue-core/src/query_optimization_suggestion_generator.rs` - Suggest filters

**Database schema changes**: None (uses existing entity counts)

### Agent Workflow Example

```bash
# Before (v1.6) - Blind execution
curl "http://localhost:7777/code-entities-list-all"
# Agent: "Received 105K tokens, context overflow, retrying..."

# After (v1.7) - Proactive estimation
curl "http://localhost:7777/code-entities-list-all?dry_run=true"
# Response:
{
  "estimated_tokens": 105000,
  "entity_count": 3500,
  "warning": "Query exceeds recommended budget (50K tokens)",
  "suggestions": [
    "Filter by module: ?module=src/auth",
    "Use preview mode: ?detail=preview (reduces to 7K tokens)",
    "Paginate results: ?limit=100&offset=0"
  ]
}

# Agent adjusts strategy BEFORE executing expensive query
curl "http://localhost:7777/code-entities-list-all?detail=preview&module=src/auth"
# Response: 4K tokens (fits comfortably)
```

**Self-Correction Flow**:
1. Agent wants broad query
2. Calls dry-run estimation
3. Sees 105K token warning
4. Narrows scope or uses preview mode
5. Executes optimized query
6. **Result**: No wasted context, no backtracking

---

## PRD Idea #3: Stateful Query Pagination Bookmarks

**Priority**: P1 (Beyond SSE streaming)
**Version Target**: v1.7
**Effort Estimate**: 2 weeks
**Arize Pattern Applied**: Paging Through Results (Cursor stepping incrementally)

### Current Gap

v1.6 provides SSE streaming (incremental delivery), but **no stateful pagination**. Agents can't bookmark positions in large result sets and resume later. No "show next 100 entities" without re-executing full query.

**Example**: Agent exploring 5,000 entity codebase
- Page 1 (entities 0-99): Reviewed, some interesting
- Agent switches context to another task
- Returns later, wants entities 100-199
- Must re-query from scratch (no cursor saved)

### Proposed Solution

Implement **server-side query cursors** with session IDs. Add `/query-pagination-cursor-create` to start paginated query, `/query-pagination-cursor-next` to fetch next page. Cursors expire after 30 minutes or explicit close.

Store cursor state in CozoDB `QueryCursorState` table with session ID.

### Acceptance Criteria

- [ ] New endpoint `/query-pagination-cursor-create` creates cursor, returns `cursor_id`
- [ ] New endpoint `/query-pagination-cursor-next/{cursor_id}` fetches next page
- [ ] New endpoint `/query-pagination-cursor-close/{cursor_id}` releases cursor
- [ ] Cursors support: Offset-based pagination (LIMIT/OFFSET) and result bookmarking
- [ ] Cursor expiration: 30 minutes TTL, auto-cleanup
- [ ] All query endpoints support cursor mode: `?use_cursor=true&page_size=100`
- [ ] Cursor state persisted in CozoDB (survives server restart)
- [ ] MCP tools `create_pagination_cursor`, `fetch_next_page`, `close_cursor`
- [ ] Backward compatibility: Non-cursor mode unchanged
- [ ] All tests passing

### Implementation Notes

**Crates affected**:
- `parseltongue-core` (cursor state management)
- `pt08-http-code-query-server` (pagination handlers)

**New modules needed**:
- `parseltongue-core/src/query_pagination_cursor_manager.rs` - Cursor lifecycle
- `parseltongue-core/src/cursor_state_persistence_layer.rs` - CozoDB storage
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/pagination_cursor_create_handler.rs`
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/pagination_cursor_next_handler.rs`

**Database schema changes**:
- Add `QueryCursorState` table: `{cursor_id: String, query_params: Json, offset: Int, created_at: Timestamp}`

### Agent Workflow Example

```bash
# After (v1.7) - Stateful pagination
curl -X POST "http://localhost:7777/query-pagination-cursor-create" \
  -d '{"query": "code-entities-list-all", "params": {"detail": "preview"}, "page_size": 100}'
# Response:
{
  "cursor_id": "cursor_abc123",
  "total_entities": 5000,
  "page_size": 100,
  "total_pages": 50
}

# Fetch page 1 (entities 0-99)
curl "http://localhost:7777/query-pagination-cursor-next/cursor_abc123"
# Response: Entities 0-99

# Agent switches tasks, returns 10 minutes later
# Fetch page 2 (entities 100-199)
curl "http://localhost:7777/query-pagination-cursor-next/cursor_abc123"
# Response: Entities 100-199 (cursor remembers position)

# Agent done
curl -X DELETE "http://localhost:7777/query-pagination-cursor-close/cursor_abc123"
```

**vs. SSE Streaming (v1.6)**:
- **SSE**: Continuous stream, no pause/resume, must consume all or restart
- **Cursors**: Pause at any point, resume later, skip pages, bookmark position

Both complement each other:
- SSE for "show me results as fast as possible"
- Cursors for "let me explore incrementally, at my own pace"

---

## PRD Idea #4: Subgraph Export Local Execution

**Priority**: P2 (SQL vs File System tradeoff)
**Version Target**: v1.8
**Effort Estimate**: 2 weeks
**Arize Pattern Applied**: SQL vs File System Tradeoff (Remote query vs. local processing)

### Current Gap

Parseltongue only supports **remote queries** (agent calls HTTP endpoint, server executes in CozoDB, returns JSON). No way to **export graph subsets** for local agent manipulation. Agents can't apply custom algorithms on graph data without repeated server round-trips.

**Example**: Agent wants to find "longest dependency chain in auth module"
- Current approach: Multiple blast-radius queries, manual chain tracking
- Ideal approach: Export auth module subgraph, run local graph algorithm

### Proposed Solution

Add `/graph-subgraph-export-json` endpoint that exports ISG subgraphs as **standalone JSON files** with embedded graph structure. Format: nodes (entities) + edges (dependencies) + metadata. Agent downloads once, processes locally unlimited times.

Support export scopes: By module, by file pattern, by entity type, by custom filter.

### Acceptance Criteria

- [ ] New endpoint `/graph-subgraph-export-json` returns graph JSON
- [ ] Export scopes: `?module=src/auth`, `?file_pattern=**/test*.rs`, `?entity_type=fn`
- [ ] JSON format includes: `nodes[]` (entities), `edges[]` (dependencies), `metadata` (stats)
- [ ] Nodes include: All entity fields from preview mode + signature
- [ ] Edges include: `from_key`, `to_key`, `edge_type` (call, import, inherit)
- [ ] Export file size estimate provided before download
- [ ] Support streaming export for large subgraphs (>10K entities)
- [ ] MCP tool `export_subgraph_to_local_file` exposes export
- [ ] Documentation includes: "When to query vs. when to export" guide
- [ ] All tests passing

### Implementation Notes

**Crates affected**:
- `parseltongue-core` (subgraph extraction, JSON serialization)
- `pt08-http-code-query-server` (export handler)

**New modules needed**:
- `parseltongue-core/src/subgraph_extraction_filter_builder.rs` - Build scope filters
- `parseltongue-core/src/isg_json_export_serializer.rs` - Graph → JSON
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/subgraph_export_json_handler.rs`

**Database schema changes**: None (reads existing graph)

### Agent Workflow Example

```bash
# After (v1.8) - Local execution mode

# Step 1: Export auth module subgraph
curl "http://localhost:7777/graph-subgraph-export-json?module=src/auth" \
  -o auth_module_graph.json
# Response: 250 entities, 890 edges, 45KB JSON file

# Step 2: Agent processes locally (Python example)
import json
import networkx as nx

graph = json.load(open("auth_module_graph.json"))
G = nx.DiGraph()

for node in graph["nodes"]:
    G.add_node(node["entity_key"], **node)

for edge in graph["edges"]:
    G.add_edge(edge["from_key"], edge["to_key"])

# Find longest dependency chain (custom algorithm)
longest_path = nx.dag_longest_path(G)
print(f"Longest chain: {len(longest_path)} hops")

# Find cycles
cycles = list(nx.simple_cycles(G))
print(f"Circular dependencies: {len(cycles)}")

# Centrality analysis
centrality = nx.betweenness_centrality(G)
hotspots = sorted(centrality.items(), key=lambda x: x[1], reverse=True)[:10]
```

**Use Cases**:
1. **Custom algorithms**: Agents run graph algorithms not built into Parseltongue
2. **Offline analysis**: Work without server connection
3. **Integration**: Import into other tools (Neo4j, Gephi, Cytoscape)
4. **Reproducibility**: Snapshot graph state for later comparison

**Trade-off Guide** (documented):
- **Query in database** (HTTP endpoints): Fast, up-to-date, built-in algorithms
- **Export for local** (JSON file): Custom processing, offline, integration, snapshots

---

## PRD Idea #5: Session Hot Path Cache

**Priority**: P2 (Dynamic index generation)
**Version Target**: v1.8
**Effort Estimate**: 1.5 weeks
**Arize Pattern Applied**: Dynamic Index Generation (Unix commands creating indexes at runtime)

### Current Gap

Parseltongue queries are **stateless** - every query scans full graph even when agents repeatedly query same entities. No session-aware caching of "hot paths" (frequently accessed entities/relationships).

**Example**: Agent refactoring authentication flow
- Queries `reverse-callers` for `authenticate()` function: 15 times in 5 minutes
- Each query: Full graph scan, 200ms
- Total wasted time: 3 seconds
- Ideal: First query cached, subsequent hits in 5ms

### Proposed Solution

Implement **session-scoped hot path cache** that tracks frequently queried entities/edges per session. Cache stored in-memory (LRU eviction, 10-minute TTL). Add `?session=<id>` parameter to all endpoints.

Cache invalidates on file changes detected by file watcher (v1.4.2 already has this).

### Acceptance Criteria

- [ ] All endpoints support `?session=<session_id>` parameter
- [ ] Hot path cache tracks: Entity lookups, blast radius queries, caller/callee graphs
- [ ] Cache hit rate measured: Aim for >70% on repeated queries
- [ ] Cache size limit: 100MB per session, LRU eviction
- [ ] Cache invalidation: On file change events (integrates with file watcher)
- [ ] New endpoint `/session-hot-path-statistics` shows cache stats
- [ ] MCP tools automatically include session ID from conversation context
- [ ] Performance improvement measured: 10-50× faster for cached queries
- [ ] All tests passing

### Implementation Notes

**Crates affected**:
- `parseltongue-core` (caching layer)
- `pt08-http-code-query-server` (session management, cache integration)

**New modules needed**:
- `parseltongue-core/src/session_hot_path_cache.rs` - LRU cache with TTL
- `parseltongue-core/src/cache_invalidation_coordinator_module.rs` - File watcher → cache invalidation
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/session_statistics_handler.rs`

**Database schema changes**: None (in-memory cache only)

### Agent Workflow Example

```bash
# After (v1.8) - Session-aware caching

# Agent conversation starts, session ID assigned
SESSION="session_xyz789"

# First query (cold cache)
time curl "http://localhost:7777/reverse-callers-query-graph?entity=rust:fn:authenticate&session=$SESSION"
# Response time: 200ms
# X-Cache-Status: MISS

# Agent asks follow-up questions, same entity
time curl "http://localhost:7777/blast-radius-impact-analysis?entity=rust:fn:authenticate&hops=2&session=$SESSION"
# Response time: 8ms (cache hit on entity metadata)
# X-Cache-Status: HIT

time curl "http://localhost:7777/forward-callees-query-graph?entity=rust:fn:authenticate&session=$SESSION"
# Response time: 5ms (cache hit)
# X-Cache-Status: HIT

# Check cache statistics
curl "http://localhost:7777/session-hot-path-statistics?session=$SESSION"
# Response:
{
  "cache_hits": 2,
  "cache_misses": 1,
  "hit_rate": 0.67,
  "cached_entities": 15,
  "cached_edges": 47,
  "memory_usage_mb": 2.3
}

# File changes detected → cache invalidated automatically
# Next query: MISS (re-caches fresh data)
```

**Dynamic Index Analogy**:
- Unix `find`: Creates temporary file list index at runtime
- Parseltongue hot cache: Creates temporary entity/edge index per agent session

---

## PRD Idea #6: ISG Query Composition Pipeline

**Priority**: P2 (Composable tool chains)
**Version Target**: v1.8
**Effort Estimate**: 2 weeks
**Arize Pattern Applied**: Composable Tool Chains (grep | sort | uniq)

### Current Gap

Parseltongue endpoints are **isolated** - no native way to pipe output of one query into another. Agents must manually chain queries: fetch entities, filter in agent code, pass filtered list to next query. Not composable.

**Example**: "Find all public functions in auth module that call database functions"
```bash
# Current (v1.6): Multi-step manual process
curl ".../code-entities-list-all" | jq '.entities[] | select(.module=="auth" and .visibility=="public")' > public_auth.json
# Extract entity keys
jq -r '.entity_key' public_auth.json | while read key; do
  curl ".../forward-callees-query-graph?entity=$key"
done | jq '.callees[] | select(.entity_key | contains("db_"))'
```

### Proposed Solution

Add **query composition pipeline** endpoint `/graph-query-pipeline-compose` that accepts **sequence of operations** in single request. Operations: `search`, `filter`, `traverse-callers`, `traverse-callees`, `rank-by-coupling`, `limit`.

Pipeline executed server-side, intermediate results never leave database.

### Acceptance Criteria

- [ ] New endpoint `/graph-query-pipeline-compose` accepts pipeline JSON
- [ ] Supported operations: `search`, `filter`, `traverse_callers`, `traverse_callees`, `rank`, `limit`, `deduplicate`
- [ ] Pipeline JSON format: `{"operations": [{"type": "search", "params": {...}}, {"type": "filter", ...}]}`
- [ ] Intermediate results stay in database (memory efficient)
- [ ] Pipeline validation: Type checking, operation compatibility
- [ ] Performance: Composable query faster than separate queries (measured)
- [ ] MCP tool `execute_query_pipeline` exposes composition
- [ ] Documentation: 20+ pipeline examples
- [ ] All tests passing

### Implementation Notes

**Crates affected**:
- `parseltongue-core` (pipeline execution engine)
- `pt08-http-code-query-server` (pipeline handler)

**New modules needed**:
- `parseltongue-core/src/query_pipeline_execution_engine.rs` - Pipeline interpreter
- `parseltongue-core/src/pipeline_operation_validator_module.rs` - Type checking
- `parseltongue-core/src/pipeline_datalog_compiler_module.rs` - Pipeline → CozoDB query
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/query_pipeline_compose_handler.rs`

**Database schema changes**: None (compiles to Datalog queries)

### Agent Workflow Example

```bash
# After (v1.8) - Pipeline composition

curl -X POST "http://localhost:7777/graph-query-pipeline-compose" \
  -H "Content-Type: application/json" \
  -d '{
    "operations": [
      {
        "type": "search",
        "params": {"module": "src/auth", "entity_type": "fn"}
      },
      {
        "type": "filter",
        "params": {"visibility": "public"}
      },
      {
        "type": "traverse_callees",
        "params": {"depth": 1}
      },
      {
        "type": "filter",
        "params": {"name_pattern": "db_*"}
      },
      {
        "type": "rank",
        "params": {"by": "coupling_score", "order": "desc"}
      },
      {
        "type": "limit",
        "params": {"count": 10}
      }
    ]
  }'

# Returns: Top 10 public auth functions that call database functions, ranked by coupling
# Executed in single efficient Datalog query
```

**Composability Benefits**:
1. **Performance**: Server-side execution, no intermediate JSON serialization
2. **Expressiveness**: Complex queries without custom DSL
3. **Debugging**: Pipeline steps can be inspected individually
4. **Reusability**: Common pipelines saved as templates

**Unix Analogy**:
```bash
# Unix pipes
grep "error" logs.txt | sort | uniq -c | head -10

# Parseltongue pipelines
search(module="auth") | filter(type="fn") | traverse_callees(depth=1) | rank(by="coupling") | limit(10)
```

---

## PRD Idea #7: Budget Aware Query Planner

**Priority**: P1 (Context budget management)
**Version Target**: v1.9
**Effort Estimate**: 2 weeks
**Arize Pattern Applied**: Context Budget Management (Making 200K feel like 200 trillion)

### Current Gap

Parseltongue executes queries as-is with no **query optimization** for token budgets. If agent has 20K tokens remaining in context, Parseltongue might return 50K token result. No query rewriting to fit budget.

**Example**: Agent with 20K token budget asks "analyze blast radius for authenticate()"
- Unoptimized query: 500 affected entities, 50K tokens
- Agent: Context overflow
- Ideal: Parseltongue auto-rewrites to preview mode, returns top 200 entities by relevance, 18K tokens

### Proposed Solution

Implement **budget-aware query planner** that accepts `?token_budget=<N>` parameter. Planner automatically:
1. Estimates query cost
2. If exceeds budget, applies optimizations:
   - Switch to preview mode
   - Limit result count
   - Filter by relevance
   - Suggest alternative queries
3. Returns optimized results + "optimization applied" metadata

Planner uses heuristics: Token/entity averages, preview mode ratios, relevance scoring.

### Acceptance Criteria

- [ ] All query endpoints support `?token_budget=<N>` parameter
- [ ] Query planner estimates cost BEFORE execution
- [ ] If cost > budget, auto-optimizations applied:
   - `detail=preview` if exceeds by 2×
   - `limit=N` to fit budget
   - `relevance_score` filtering (keep highest)
- [ ] Response includes `"optimization_applied": true` + explanation
- [ ] Optimization accuracy: Results fit within ±10% of budget
- [ ] No optimizations if query naturally fits budget
- [ ] MCP tools default to token_budget from agent's remaining context
- [ ] New endpoint `/query-plan-explain` shows optimization strategy without execution
- [ ] All tests passing

### Implementation Notes

**Crates affected**:
- `parseltongue-core` (query planning, optimization)
- `pt08-http-code-query-server` (all query handlers integrate planner)

**New modules needed**:
- `parseltongue-core/src/budget_aware_query_planner.rs` - Query optimization engine
- `parseltongue-core/src/query_cost_estimation_model.rs` - Token cost heuristics
- `parseltongue-core/src/optimization_strategy_selector_module.rs` - Pick best optimization
- `pt08-http-code-query-server/src/http_endpoint_handler_modules/query_plan_explain_handler.rs`

**Database schema changes**: None (uses existing statistics)

### Agent Workflow Example

```bash
# After (v1.9) - Budget-aware execution

# Agent has 20K tokens remaining in context
curl "http://localhost:7777/blast-radius-impact-analysis?entity=rust:fn:authenticate&hops=3&token_budget=20000"

# Response:
{
  "optimization_applied": true,
  "optimization_details": {
    "original_estimate": "51,234 tokens (500 entities, full detail)",
    "budget": 20000,
    "optimizations": [
      "Switched to detail=preview (reduces to 10K tokens)",
      "Limited to top 150 entities by relevance score",
      "Excluded transitive dependencies beyond depth 2"
    ],
    "final_estimate": "18,500 tokens"
  },
  "data": {
    "entities_included": 150,
    "entities_total": 500,
    "total_affected": 500,
    "context": [
      // Top 150 entities in preview mode, ranked by relevance
    ]
  },
  "suggestion": "For full results, increase budget to 52K tokens or narrow scope with ?module=src/auth"
}
```

**Explain Plan** (before execution):
```bash
curl "http://localhost:7777/query-plan-explain?query=blast-radius-impact-analysis&entity=rust:fn:authenticate&hops=3&token_budget=20000"

# Response:
{
  "original_query": {
    "entity": "rust:fn:authenticate",
    "hops": 3,
    "estimated_tokens": 51234
  },
  "budget": 20000,
  "exceeds_budget": true,
  "proposed_optimizations": [
    {"strategy": "preview_mode", "savings": 25617, "result_tokens": 25617},
    {"strategy": "limit_results_150", "savings": 6867, "result_tokens": 18750},
    {"strategy": "reduce_hops_2", "savings": 250, "result_tokens": 18500}
  ],
  "recommended_plan": "Apply all optimizations",
  "final_estimate": 18500
}
```

**Intelligence Features**:
1. **Relevance preservation**: Keeps most important entities, drops periphery
2. **Graceful degradation**: Progressive optimizations until fits
3. **Transparency**: Agent knows what was optimized and why
4. **Recovery path**: Suggestions for getting full results

**Arize Pattern Realization**:
> "Making 200K context feel like 200 trillion by intelligent summarization and pointers"

Parseltongue achieves this by:
- **Automatic preview mode**: Show signatures, not implementations
- **Relevance filtering**: Keep architectural core, drop leaves
- **Pointer system** (PRD #1): Agent can fetch full bodies on-demand
- **Transparent cost**: Agent always knows token impact

---

## Summary: v1.7-v1.9 Feature Prioritization

### v1.7 - Agent Memory Foundation (P1 Focus)
**Target**: Q2 2026, 4.5 weeks effort

| Feature | Priority | Effort | Arize Pattern |
|---------|----------|--------|---------------|
| Entity Preview Signature Pointers | P1 | 1.5 weeks | Preview/Pointer |
| Query Token Budget Estimator | P1 | 1 week | Self-Correction |
| Stateful Query Pagination Bookmarks | P1 | 2 weeks | Paging Results |

**Deliverables**:
- ✅ Tiered responses (preview/pointer/full)
- ✅ Proactive token estimation
- ✅ Server-side pagination cursors
- ✅ 90%+ token reduction on large queries
- ✅ No blind context overflow

---

### v1.8 - Advanced Memory Patterns (P2 Focus)
**Target**: Q3 2026, 5.5 weeks effort

| Feature | Priority | Effort | Arize Pattern |
|---------|----------|--------|---------------|
| Subgraph Export Local Execution | P2 | 2 weeks | SQL vs File System |
| Session Hot Path Cache | P2 | 1.5 weeks | Dynamic Indexing |
| ISG Query Composition Pipeline | P2 | 2 weeks | Composable Chains |

**Deliverables**:
- ✅ Export ISG subgraphs for local processing
- ✅ Session-aware hot path caching
- ✅ Composable query pipelines
- ✅ 10-50× speedup on repeated queries
- ✅ Custom graph algorithms enabled

---

### v1.9 - Intelligent Budget Management (P1 Priority)
**Target**: Q4 2026, 2 weeks effort

| Feature | Priority | Effort | Arize Pattern |
|---------|----------|--------|---------------|
| Budget Aware Query Planner | P1 | 2 weeks | Budget Management |

**Deliverables**:
- ✅ Automatic query optimization for token budgets
- ✅ Graceful degradation with transparency
- ✅ Query plan explanation
- ✅ 200K context → infinite memory feel

---

## Cross-Cutting Implementation Principles

### 4-Word Naming Convention Compliance

All new modules follow strict 4-word naming:

**Good Examples**:
- `entity_preview_signature_serializer.rs`
- `token_budget_estimation_calculator.rs`
- `query_pagination_cursor_manager.rs`
- `subgraph_extraction_filter_builder.rs`
- `session_hot_path_cache.rs`
- `query_pipeline_execution_engine.rs`
- `budget_aware_query_planner.rs`

**Bad Examples** (too short):
- `preview_serializer.rs` ❌
- `token_estimator.rs` ❌
- `cursor_manager.rs` ❌

---

### TDD Workflow Adherence

Every feature follows: **STUB → RED → GREEN → REFACTOR**

Example for PRD #1 (Entity Preview Pointers):

1. **STUB**: Write test for preview mode response
   ```rust
   #[tokio::test]
   async fn test_entity_preview_mode_returns_signature_only() {
       // STUB - Will fail
   }
   ```

2. **RED**: Run test, verify failure
   ```bash
   cargo test test_entity_preview_mode_returns_signature_only
   # Expected: FAILED (feature not implemented)
   ```

3. **GREEN**: Minimal implementation
   ```rust
   // Add preview mode logic to handler
   // Test passes
   ```

4. **REFACTOR**: Clean up without breaking tests
   ```rust
   // Extract serialization to dedicated module
   // Tests still pass
   ```

---

### Layered Architecture Integration

All features respect L1/L2/L3 boundaries:

- **L1 Core** (`parseltongue-core`):
  - Token estimation algorithms
  - Preview mode serialization
  - Query pipeline logic
  - No async, no I/O

- **L2 Standard** (`parseltongue-core`):
  - Cache management (LRU, TTL)
  - Session state
  - Query planning heuristics

- **L3 External** (tool crates):
  - HTTP handlers with Axum
  - MCP STDIO integration
  - CozoDB async queries
  - File I/O for exports

---

## Strategic Differentiators vs. Generic File Tools

### Why Parseltongue's ISG Beats File Systems for These Patterns

| Pattern | File System (Cursor/Claude) | Parseltongue ISG | Advantage |
|---------|------------------------------|------------------|-----------|
| **Preview/Pointer** | Show filename + first 5 lines | Show signature with type info | Signature is semantically complete |
| **Token Estimation** | Heuristic: 4 chars = 1 token | Exact: Pre-computed entity token counts | 95% accuracy vs. ±30% |
| **Pagination** | Line-based or file-based | Entity-based with graph continuity | Maintains semantic boundaries |
| **Export** | Copy directory tree | Export connected subgraph | Preserves dependencies |
| **Caching** | File content cache | Entity + relationship cache | Semantic units, not bytes |
| **Composition** | grep \| awk \| sort | Graph traversal pipelines | Type-safe, graph-aware |
| **Budget Planning** | Estimate file sizes | Optimize graph queries | Relevance-aware filtering |

**Core Insight**: Files are bags of text. ISG entities are **typed, connected, semantic units**. All Arize patterns work better on structured graphs than unstructured files.

---

## The Meta-Narrative: From v1.4.2 → v1.9

### v1.4.2 (Current)
- ✅ Deterministic ISG
- ✅ 14 HTTP endpoints
- ✅ File watching
- ❌ Agent-native protocols
- ❌ Memory hierarchy
- ❌ Budget awareness

### v1.6 (Foundation)
- ✅ MCP protocol
- ✅ Unix piping
- ✅ SSE streaming
- ✅ Multi-workspace
- **Gap**: Still no tiered responses, no budget management

### v1.7 (Memory Patterns)
- ✅ Preview/pointer system
- ✅ Token estimation
- ✅ Stateful pagination
- **Unlock**: 90% token reduction, no blind overflow

### v1.8 (Advanced Patterns)
- ✅ Local subgraph export
- ✅ Hot path caching
- ✅ Query composition
- **Unlock**: Custom algorithms, 10-50× speedups

### v1.9 (Intelligent Budgets)
- ✅ Auto-optimizing planner
- ✅ Graceful degradation
- ✅ Transparent costs
- **Unlock**: 200K context feels infinite

**Result**: Parseltongue becomes the **L2 cache for agent code memory** - deterministic, budget-aware, compositional.

---

## Recommended Implementation Order

### Phase 1: v1.7 Foundation (Q2 2026)
**Must-Have for Agent Adoption**

1. **Week 1-1.5**: Entity Preview Signature Pointers (PRD #1)
   - Biggest token reduction win
   - Enables all other optimizations
   - MCP integration straightforward

2. **Week 2-3**: Query Token Budget Estimator (PRD #2)
   - Self-correction enablement
   - Pairs with preview mode
   - Small implementation surface

3. **Week 3.5-5.5**: Stateful Query Pagination Bookmarks (PRD #3)
   - Complements SSE streaming from v1.6
   - More complex (cursor state management)
   - High agent UX value

**Deliverable**: v1.7 release with full Arize memory foundation

---

### Phase 2: v1.8 Advanced (Q3 2026)
**Power User Features**

4. **Week 6-8**: Subgraph Export Local Execution (PRD #4)
   - Enables integration ecosystem
   - Requires solid JSON schema design
   - Documentation-heavy (export format spec)

5. **Week 8.5-10**: Session Hot Path Cache (PRD #5)
   - Performance multiplier
   - Integrates with existing file watcher
   - Relatively isolated change

6. **Week 10.5-12.5**: ISG Query Composition Pipeline (PRD #6)
   - Most complex feature
   - Requires pipeline DSL design
   - High power user value

**Deliverable**: v1.8 release with advanced patterns

---

### Phase 3: v1.9 Intelligence (Q4 2026)
**Budget Management Capstone**

7. **Week 13-15**: Budget Aware Query Planner (PRD #7)
   - Synthesizes all v1.7-v1.8 features
   - Uses preview mode, estimation, pipelines
   - High impact, clean abstraction

**Deliverable**: v1.9 release - Full Arize pattern parity

---

## The Strategic Thesis

### Why This Roadmap Matters

**Problem**: Agents have 200K-1M token contexts, but codebases are 10M-1B tokens. Context windows are growing, but **semantic density** isn't.

**Generic Solution** (Cursor/Claude):
- File-level chunking
- Keyword search + embeddings
- Manual context management
- "Hope the right code is in context"

**Parseltongue Solution** (v1.9):
- **Deterministic graph**: Know all dependencies precisely
- **Tiered responses**: Show signatures, not implementations (90% reduction)
- **Budget-aware planning**: Auto-optimize queries to fit context
- **Compositional queries**: Express complex intent concisely
- **Session memory**: Cache hot paths, no repeated work
- **Local execution**: Export for custom algorithms

**Result**: Agent reads 1% of codebase, understands 100% of architecture.

### Competitive Moat

**Cursor/Claude/Alyx**: Apply memory patterns to **files** (unstructured)
**Parseltongue**: Apply memory patterns to **ISG** (structured graph)

Advantage compounds:
1. **Better previews**: Signatures > first 5 lines
2. **Better estimation**: Exact entity tokens > heuristics
3. **Better caching**: Semantic units > byte ranges
4. **Better composition**: Graph queries > text pipes
5. **Better planning**: Relevance filtering > size-based truncation

**Defensibility**: Tree-sitter + CozoDB + ISG architecture is **high-effort to replicate**. File systems are commodities; deterministic code graphs are moats.

---

## Appendix: Naming Reference

### All New Modules (4-Word Compliant)

**v1.7 Modules** (3 features):
1. `entity_preview_signature_serializer.rs`
2. `sighash_stable_pointer_generator.rs`
3. `entity_full_body_retrieve_handler.rs`
4. `token_budget_estimation_calculator.rs`
5. `query_optimization_suggestion_generator.rs`
6. `query_token_budget_estimate_handler.rs`
7. `query_pagination_cursor_manager.rs`
8. `cursor_state_persistence_layer.rs`
9. `pagination_cursor_create_handler.rs`
10. `pagination_cursor_next_handler.rs`

**v1.8 Modules** (3 features):
11. `subgraph_extraction_filter_builder.rs`
12. `isg_json_export_serializer.rs`
13. `subgraph_export_json_handler.rs`
14. `session_hot_path_cache.rs`
15. `cache_invalidation_coordinator_module.rs`
16. `session_statistics_handler.rs`
17. `query_pipeline_execution_engine.rs`
18. `pipeline_operation_validator_module.rs`
19. `pipeline_datalog_compiler_module.rs`
20. `query_pipeline_compose_handler.rs`

**v1.9 Modules** (1 feature):
21. `budget_aware_query_planner.rs`
22. `query_cost_estimation_model.rs`
23. `optimization_strategy_selector_module.rs`
24. `query_plan_explain_handler.rs`

**Total**: 24 new modules, all 4-word compliant

---

## Appendix: Database Schema Changes

### New Tables (v1.7-v1.9)

#### v1.7: SigHash Lookup
```sql
-- CozoDB relation
:create SigHashLookup {
  sighash: String,       # BLAKE3 hash of signature
  entity_key: String,    # ISGL1 entity key
  signature: String,     # Full signature text
  created_at: Int,       # Unix timestamp
}
```

#### v1.7: Query Cursor State
```sql
:create QueryCursorState {
  cursor_id: String,     # UUID
  query_type: String,    # Endpoint name
  query_params: String,  # JSON-encoded params
  offset: Int,           # Current position
  page_size: Int,        # Results per page
  created_at: Int,       # TTL tracking
  expires_at: Int,       # 30 minutes from creation
}
```

#### v1.8: No new tables
- Hot path cache: In-memory only (no persistence)
- Subgraph export: Read-only (no writes)
- Query pipeline: Compiles to Datalog (no new tables)

#### v1.9: No new tables
- Query planner: Uses existing stats (no persistence)

**Total**: 2 new CozoDB relations

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v1.0 | 2026-01-31 | Initial extraction of 7 PRD ideas for v1.7-v1.9 |

---

**Document Status**: ✅ COMPLETE
**Next Steps**:
1. Review with team
2. Prioritize v1.7 scope
3. Create TDD test plans for PRD #1 (Preview/Pointer)
4. Start STUB phase for v1.7
