# Quick Reference: 7 PRD Ideas for v1.7-v1.9

**Purpose**: One-page reference for all 7 Arize-inspired PRD ideas
**Date**: 2026-01-31
**Foundation**: Assumes v1.6 complete (MCP, Unix piping, streaming, multi-workspace)

---

## v1.7 - Agent Memory Foundation (4.5 weeks, P1)

### PRD #1: Entity Preview Signature Pointers
**Effort**: 1.5 weeks | **Arize Pattern**: Preview/Pointer (Alyx)

**What**: Tiered responses - `?detail=preview|pointer|full`
- **Preview**: Signature + metadata only (2 tokens/entity)
- **Pointer**: + SigHash for full retrieval
- **Full**: Complete entity with body + dependencies (30 tokens/entity)

**Why**: 90% token reduction (500 entities: 15K → 1K tokens)

**How**: New endpoint `/code-entity-full-body-retrieve/{sighash}`

**Example**:
```bash
# Preview mode (default)
curl "http://localhost:7777/code-entities-search-fuzzy?q=handle&detail=preview"
# Returns: 500 signatures = 1K tokens

# Fetch 3 full entities by SigHash
curl "http://localhost:7777/code-entity-full-body-retrieve/blake3_abc123"
# Total: 1K + 90 = 1,090 tokens (vs. 15K)
```

---

### PRD #2: Query Token Budget Estimator
**Effort**: 1 week | **Arize Pattern**: Self-Correction (Claude)

**What**: Pre-query token estimation + warnings
- Endpoint: `/query-token-budget-estimate`
- Dry-run mode: `?dry_run=true` (estimate only, no execution)
- Response header: `X-Estimated-Tokens`

**Why**: Self-correction before wasting context

**How**: Cached entity counts + heuristic token/entity averages

**Example**:
```bash
# Dry-run estimation
curl "http://localhost:7777/code-entities-list-all?dry_run=true"
# Response:
{
  "estimated_tokens": 105000,
  "warning": "Exceeds budget",
  "suggestions": ["Use preview mode", "Filter by module"]
}

# Agent adjusts strategy BEFORE executing
```

---

### PRD #3: Stateful Query Pagination Bookmarks
**Effort**: 2 weeks | **Arize Pattern**: Paging Results (Cursor)

**What**: Server-side cursors for pause/resume queries
- Create cursor: `/query-pagination-cursor-create`
- Fetch next page: `/query-pagination-cursor-next/{cursor_id}`
- Close cursor: `/query-pagination-cursor-close/{cursor_id}`

**Why**: Beyond SSE streaming - stateful exploration

**How**: CozoDB `QueryCursorState` table, 30-minute TTL

**Example**:
```bash
# Create cursor for 5K entity query
curl -X POST ".../query-pagination-cursor-create" \
  -d '{"query": "code-entities-list-all", "page_size": 100}'
# Returns: cursor_id, total_pages

# Fetch page 1
curl ".../query-pagination-cursor-next/cursor_abc123"

# Agent switches tasks, returns 10 minutes later
# Fetch page 2 (cursor remembers position)
curl ".../query-pagination-cursor-next/cursor_abc123"
```

---

## v1.8 - Advanced Memory Patterns (5.5 weeks, P2)

### PRD #4: Subgraph Export Local Execution
**Effort**: 2 weeks | **Arize Pattern**: SQL vs File System

**What**: Export ISG subgraphs as JSON for local processing
- Endpoint: `/graph-subgraph-export-json`
- Scopes: By module, file pattern, entity type
- Format: Nodes (entities) + Edges (dependencies) + Metadata

**Why**: Custom graph algorithms, offline analysis, integration

**How**: CozoDB query → JSON serialization

**Example**:
```bash
# Export auth module subgraph
curl ".../graph-subgraph-export-json?module=src/auth" -o auth_graph.json

# Agent processes locally (Python NetworkX example)
import networkx as nx
graph = load_graph("auth_graph.json")
longest_chain = nx.dag_longest_path(graph)
cycles = list(nx.simple_cycles(graph))
```

---

### PRD #5: Session Hot Path Cache
**Effort**: 1.5 weeks | **Arize Pattern**: Dynamic Indexing (Unix `find`)

**What**: In-memory LRU cache of frequently queried entities/edges per session
- All endpoints support: `?session=<session_id>`
- Cache stats: `/session-hot-path-statistics`
- Invalidation: On file change events

**Why**: 10-50× speedup on repeated queries

**How**: LRU cache with 10-minute TTL, 100MB limit/session

**Example**:
```bash
SESSION="session_xyz789"

# First query (cold cache)
time curl ".../reverse-callers?entity=rust:fn:authenticate&session=$SESSION"
# 200ms, X-Cache-Status: MISS

# Second query (cache hit)
time curl ".../blast-radius?entity=rust:fn:authenticate&session=$SESSION"
# 8ms, X-Cache-Status: HIT (25× faster)
```

---

### PRD #6: ISG Query Composition Pipeline
**Effort**: 2 weeks | **Arize Pattern**: Composable Chains (grep | sort)

**What**: Server-side query pipelines - chain graph operations
- Endpoint: `/graph-query-pipeline-compose`
- Operations: search, filter, traverse_callers, traverse_callees, rank, limit

**Why**: Composability without HTTP round-trips

**How**: Pipeline → compiled Datalog query

**Example**:
```bash
# Pipeline: Search auth functions → filter public → traverse callees → filter db calls → rank → top 10
curl -X POST ".../graph-query-pipeline-compose" \
  -d '{
    "operations": [
      {"type": "search", "params": {"module": "src/auth"}},
      {"type": "filter", "params": {"visibility": "public"}},
      {"type": "traverse_callees", "params": {"depth": 1}},
      {"type": "filter", "params": {"name_pattern": "db_*"}},
      {"type": "rank", "params": {"by": "coupling_score"}},
      {"type": "limit", "params": {"count": 10}}
    ]
  }'

# Single efficient query (vs. 5 HTTP round-trips)
```

---

## v1.9 - Intelligent Budget Management (2 weeks, P1)

### PRD #7: Budget Aware Query Planner
**Effort**: 2 weeks | **Arize Pattern**: Budget Management (200K → ∞)

**What**: Auto-optimize queries to fit token budgets
- All endpoints support: `?token_budget=<N>`
- Auto-optimizations: preview mode, limit results, rank by relevance
- Response metadata: "optimization_applied" + explanation

**Why**: 200K context feels infinite - no manual optimization

**How**: Query cost estimation + optimization strategy selection

**Example**:
```bash
# Agent has 20K tokens remaining
curl ".../blast-radius?entity=rust:fn:authenticate&hops=3&token_budget=20000"

# Response:
{
  "optimization_applied": true,
  "optimization_details": {
    "original_estimate": "51,234 tokens (500 entities)",
    "optimizations": [
      "Switched to preview mode (reduces to 10K)",
      "Limited to top 150 by relevance",
      "Excluded depth > 2"
    ],
    "final_estimate": "18,500 tokens"
  },
  "data": {
    "entities_included": 150,  # Top 150, ranked
    "entities_total": 500       # Full set available
  },
  "suggestion": "For full results, increase budget to 52K or narrow scope"
}
```

**Query Plan Explanation** (before execution):
```bash
curl ".../query-plan-explain?query=blast-radius&entity=...&token_budget=20000"
# Shows optimization strategy without executing query
```

---

## Implementation Checklist

### v1.7 (All P1 - Do First)
- [ ] PRD #1: Preview/Pointer (1.5w) - Biggest token win
- [ ] PRD #2: Budget Estimator (1w) - Self-correction
- [ ] PRD #3: Pagination Cursors (2w) - Stateful exploration

### v1.8 (All P2 - Power Users)
- [ ] PRD #4: Subgraph Export (2w) - Local execution
- [ ] PRD #5: Hot Path Cache (1.5w) - Performance
- [ ] PRD #6: Query Pipelines (2w) - Composability

### v1.9 (P1 - Capstone)
- [ ] PRD #7: Budget Planner (2w) - Auto-optimization

**Total Effort**: 12 weeks (4.5w + 5.5w + 2w)

---

## Module Naming Reference (4-Word Compliant)

### v1.7 Modules
- `entity_preview_signature_serializer.rs`
- `sighash_stable_pointer_generator.rs`
- `entity_full_body_retrieve_handler.rs`
- `token_budget_estimation_calculator.rs`
- `query_optimization_suggestion_generator.rs`
- `query_token_budget_estimate_handler.rs`
- `query_pagination_cursor_manager.rs`
- `cursor_state_persistence_layer.rs`
- `pagination_cursor_create_handler.rs`
- `pagination_cursor_next_handler.rs`

### v1.8 Modules
- `subgraph_extraction_filter_builder.rs`
- `isg_json_export_serializer.rs`
- `subgraph_export_json_handler.rs`
- `session_hot_path_cache.rs`
- `cache_invalidation_coordinator_module.rs`
- `session_statistics_handler.rs`
- `query_pipeline_execution_engine.rs`
- `pipeline_operation_validator_module.rs`
- `pipeline_datalog_compiler_module.rs`
- `query_pipeline_compose_handler.rs`

### v1.9 Modules
- `budget_aware_query_planner.rs`
- `query_cost_estimation_model.rs`
- `optimization_strategy_selector_module.rs`
- `query_plan_explain_handler.rs`

**Total**: 24 new modules, all 4-word naming compliant

---

## Database Schema Changes

### v1.7: 2 New Tables
```sql
-- SigHash lookup for pointer retrieval
:create SigHashLookup {
  sighash: String,
  entity_key: String,
  signature: String,
  created_at: Int
}

-- Query cursor state for pagination
:create QueryCursorState {
  cursor_id: String,
  query_type: String,
  query_params: String,
  offset: Int,
  page_size: Int,
  created_at: Int,
  expires_at: Int
}
```

### v1.8: No new tables
- Hot path cache: In-memory only
- Subgraph export: Read-only
- Query pipeline: Compiles to Datalog

### v1.9: No new tables
- Query planner: Uses existing stats

---

## Success Metrics at a Glance

| Version | Key Metric | Target | Impact |
|---------|------------|--------|--------|
| v1.7 | Token reduction | 90% | Preview mode |
| v1.7 | Estimation accuracy | ±15% | Self-correction |
| v1.7 | Cursor performance | <100ms | Pause/resume |
| v1.8 | Export speed | <5s for 1K entities | Local execution |
| v1.8 | Cache hit rate | >70% | Speedup |
| v1.8 | Cache speedup | 10-50× | Repeated queries |
| v1.9 | Auto-optimization | >80% of queries | Invisible optimization |
| v1.9 | Budget compliance | ±10% of target | Context efficiency |

**Overall Goal**: Agent uses <20% of context on queries, 80% for reasoning

---

## The One-Sentence Summary

**v1.7-v1.9 applies Arize's proven agent memory patterns (preview/pointer, self-correction, budget management) to Parseltongue's deterministic code graph, achieving 90% token reduction and 100× speedups while making 200K context windows feel infinite.**

---

**For Full Details**:
- Executive summary: `00_EXECUTIVE_SUMMARY.md`
- Comprehensive PRD specs: `03_V17_V19_ARIZE_PATTERNS_EXTRACTED.md`
- Visual roadmap: `04_VISUAL_ROADMAP_V14_TO_V19.md`
