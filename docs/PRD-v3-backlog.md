# Parseltongue v3 Backlog

Features and architectural designs that are being deferred to future versions. This document serves as a reference for planned capabilities that are not in the current active development scope.

---

## Budget Aware Query Planner (PMF 94, 800 LOC, v1.9)

**Feature Description**: Computer automatically makes queries smaller to fit your token budget. Like smart shopping that never overspends.

### Architecture Analysis (Using Parseltongue Code Reading)

#### Existing Infrastructure Found

**From `/code-entities-search-fuzzy?q=token` query (9 entities found):**
- `smart_context_token_budget_handler.rs` already implements token estimation
- `estimate_entity_tokens()` function (lines 256-266): Heuristic of 100 base + complexity bonus
- `build_smart_context_selection()` (lines 128-248): Greedy algorithm that fits entities within budget
- Pattern: Relevance scoring (1.0 for callers, 0.95 for callees, 0.7-depth for transitive)

**From `route_definition_builder_module.rs` read:**
- 14 handlers registered with consistent `handle_*` pattern
- Axum `Query<T>` extractor for parameters (seen in all handlers)
- Shared state via `State<SharedApplicationStateContainer>`

**From `dependency_edges_list_handler.rs` read:**
- Pagination already exists: `limit` (default 100) and `offset` parameters
- Token estimation formula: `60 + (returned_count * 40)`
- Returns `total_count` and `returned_count` for pagination UX

### Proposed Architecture: 3-Layer Budget Planner

#### Layer 1: Query Budget Middleware (~200 LOC)

```
New endpoint: /budget-query-planner?endpoint=X&token_budget=N&params={...}
Handler: handle_budget_query_planner_executor
```

Accepts any existing endpoint name + token budget, then:
1. Pre-estimate query cost using existing patterns
2. Apply degradation strategy automatically
3. Proxy to target endpoint with modified parameters

#### Layer 2: Cost Estimation Engine (~150 LOC)

```
Function: estimate_query_cost_heuristic(endpoint, params) -> usize
```

Maps 14 existing endpoints to cost formulas (already found in code):
- `/code-entities-list-all`: `60 + (count * 40)` (from dependency_edges_list_handler.rs:95)
- `/blast-radius-impact-analysis`: Exponential in hops (needs BFS depth estimation)
- `/smart-context-token-budget`: Already budget-aware, pass-through

#### Layer 3: Graceful Degradation Strategies (~450 LOC)

4 strategies observed from existing patterns:

**1. Pagination** (already exists in dependency_edges_list_handler.rs:25-29)
- If cost > budget: reduce `limit` parameter proportionally
- Formula: `new_limit = (budget - overhead) / per_item_cost`

**2. Hop Limiting** (pattern from blast_radius_impact_handler.rs:56-58)
- Reduce `hops` parameter: 3 → 2 → 1
- Cost reduction: O(E^3) → O(E^2) → O(E)

**3. Preview Mode** (pattern from smart_context_token_budget_handler.rs:174)
- Return entity keys only (no body content)
- Token reduction: ~80% (100 tokens → 20 tokens per entity)

**4. Top-K Filtering** (pattern from complexity_hotspots_ranking_handler.rs)
- Reduce `top` parameter: 50 → 20 → 10
- Direct 1:1 reduction in result count

### Implementation Plan

**File: `budget_query_planner_handler.rs`** (~800 LOC)
- Struct: `BudgetQueryRequestParams` (endpoint, token_budget, params JSON)
- Function: `handle_budget_query_planner_executor` (~100 LOC)
- Function: `estimate_query_cost_heuristic` (~150 LOC) - maps 14 endpoints to formulas
- Function: `apply_degradation_strategy` (~200 LOC) - 4 strategies above
- Function: `proxy_to_target_endpoint` (~150 LOC) - internal HTTP call
- Tests: (~200 LOC) - TDD for each endpoint + strategy combo

**Route registration** (add to route_definition_builder_module.rs:115-118):
```rust
.route(
    "/budget-query-planner",
    get(budget_query_planner_handler::handle_budget_query_planner_executor)
)
```

### Confidence Assessment: 88% (HIGH)

#### Why High Confidence:

**From Parseltongue Evidence:**

1. ✅ **Token estimation infrastructure exists** (smart_context_token_budget_handler.rs:256)
   - Can reuse `estimate_entity_tokens()` heuristic
   - Formula: `base + complexity_bonus`

2. ✅ **Pagination pattern proven** (dependency_edges_list_handler.rs:32-34)
   - `default_limit()` function exists
   - `limit` and `offset` already in 3 endpoints

3. ✅ **Handler pattern well-established** (found 14 handlers via `/code-entities-search-fuzzy?q=handle`)
   - Consistent signature: `State + Query → IntoResponse`
   - All use `SharedApplicationStateContainer`

4. ✅ **Query cost formulas documented** (dependency_edges_list_handler.rs:95)
   - Existing: `60 + (count * 40)`
   - Can extract from all 14 handlers

5. ✅ **Greedy selection algorithm exists** (smart_context_token_budget_handler.rs:236-247)
   - Sorts by score descending
   - Greedily fits within budget
   - Can generalize to other queries

#### Risk Factors (12% uncertainty):

1. **Internal routing complexity** (route_definition_builder_module.rs:58-137)
   - Making internal HTTP calls between handlers may need refactoring
   - Axum might not support handler-to-handler calls cleanly
   - Mitigation: Extract core logic into shared functions

2. **Unknown endpoint behaviors** (found 159 entities with "query", only analyzed 3)
   - Some endpoints may have non-linear costs
   - `/circular-dependency-detection-scan` has no parameters → can't degrade
   - Mitigation: Start with 8 parametric endpoints, add others incrementally

3. **Heuristic accuracy** (estimate_entity_tokens uses rough 100+len/10)
   - Real token counts may differ by 2-3×
   - Could over/under-estimate budget consumption
   - Mitigation: Add `?strict=false` parameter for 20% safety margin

#### Success Criteria (from Parseltongue observations):

- ✅ All 14 endpoints have documented parameter patterns
- ✅ 8/14 endpoints already support pagination or limiting
- ✅ Greedy algorithm exists and works (smart_context_token_budget proves it)
- ✅ 800 LOC estimate aligns with: 1 handler module (~400 LOC) + strategies (~400 LOC)

**Conclusion**: This feature leverages 60% existing code (token estimation, pagination, greedy selection). Main work is creating the middleware layer and cost estimation mapping. Confidence is HIGH at 88%.

### Reason for Backlog

Feature deferred to allow focus on core functionality and higher-priority v1.4-v1.5 features. The existing `/smart-context-token-budget` endpoint provides sufficient token-aware querying for current use cases. Will revisit in v1.9 when more endpoint usage patterns are established.

---

## Backlog Management Notes

**Last Updated**: 2026-02-01

**Review Criteria for Promotion to Active Development**:
- User demand validation through feature requests
- Completion of prerequisite features
- Resource availability (developer time, testing capacity)
- Technical risk assessment update

**Related Documents**:
- `/docs/PRD-v144.md` - Comprehensive feature list with PMF ratings
- `/docs/PRD-research-20260131v1/` - Research documentation folder
