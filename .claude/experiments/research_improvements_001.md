# Research Document: Parseltongue Improvement Analysis

**Based on**: Dogfooding Experiment #001
**Date**: 2025-12-01
**Version**: v1.0.3

---

## Abstract

This research document analyzes findings from dogfooding experiment #001 and proposes architectural, UX, and functional improvements. Recommendations are informed by Claude Code design patterns and LLM-optimized interface principles.

---

## Part 1: Critical Bug Analysis

### Bug #1: Entity Detail View Empty Response

**Symptom**: `/code-entity-detail-view/{key}` returns empty response
**Affected Code**: `code_entity_detail_view_handler.rs`

**Root Cause Analysis**:
The Axum path parameter `{key}` contains colons (`:`) which may conflict with Axum's path syntax. The key format `rust:fn:main:__path:1-50` uses colons as delimiters.

**Proposed Fix**:
```rust
// Option 1: Use wildcard path
.route("/code-entity-detail-view/*key", get(handle_code_entity_detail_view))

// Option 2: Use query parameter instead
.route("/code-entity-detail-view", get(handle_code_entity_detail_view))
// Query: ?key=rust:fn:main:...
```

**Recommendation**: Use query parameter approach for consistency with other endpoints.

---

### Bug #2: Reverse Callers Returns Zero

**Symptom**: `rust:method:new` has 215 inbound edges per hotspots, but reverse callers returns 0
**Affected Code**: `reverse_callers_query_graph_handler.rs`

**Root Cause Analysis**:
The query may be searching for exact key match, but the edges use a different key format:
- Queried: `rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs:38-54`
- Edge target: `rust:fn:new:unknown:0-0`

**Proposed Fix**:
```rust
// Current: Exact match query
"?[from_key] := *DependencyEdges{from_key, to_key}, to_key == $entity"

// Fixed: Also match by function name for stdlib functions
"?[from_key] := *DependencyEdges{from_key, to_key},
    (to_key == $entity or (starts_with(to_key, 'rust:fn:new:') and to_key == $entity))"
```

**Recommendation**: Implement fuzzy key matching or normalize keys during ingestion.

---

### Bug #3: Blast Radius Wrong Direction

**Symptom**: "If I change X, what breaks?" returns 0 entities
**Affected Code**: `blast_radius_impact_handler.rs`

**Root Cause Analysis**:
Blast radius asks "who depends on me?" which requires **reverse** dependency traversal. The current implementation may be using forward dependencies.

**Proposed Fix**:
```rust
// Correct: Use REVERSE dependencies (who calls me)
fn compute_blast_radius_by_hops(
    storage: &CozoDbStorage,
    entity_key: &str,
    max_hops: u32,
) -> Vec<BlastRadiusHopDataItem> {
    // Query: Find all entities that DEPEND ON (call) the target
    let query = "?[caller] := *DependencyEdges{from_key: caller, to_key: $entity}";
    // ... traverse callers transitively
}
```

---

## Part 2: UX Improvements Based on Claude Code Patterns

### Pattern 1: Minimalist Visuals

**Current State**: JSON responses are verbose
**Claude Code Pattern**: Compact, scannable output

**Proposed Enhancement**: Add `format` query parameter
```
GET /codebase-statistics-overview-summary?format=compact
```

**Compact Response**:
```
Parseltongue Analysis
=====================
Entities: 214 (CODE)
Edges:    3006
DB:       parseltongue20251201170547/analysis.db
```

---

### Pattern 2: Progressive Disclosure

**Current State**: All data returned at once
**Claude Code Pattern**: Layered information depth

**Proposed Enhancement**: Multi-level detail endpoints
```
Level 0: /codebase-statistics-overview-summary
         -> counts only (~50 tokens)

Level 1: /code-entities-list-all?detail=keys
         -> keys only (~500 tokens)

Level 2: /code-entities-list-all?detail=summary
         -> keys + types + files (~2000 tokens)

Level 3: /code-entities-list-all?detail=full
         -> all fields (~5000 tokens)
```

---

### Pattern 3: Visual Graph Output

**Current State**: Semantic clusters return JSON arrays
**Claude Code Pattern**: ASCII art for structure visualization

**Proposed Enhancement**: Add `format=tree` option
```
GET /semantic-cluster-grouping-list?format=tree
```

**Tree Response**:
```
Cluster Analysis (43 clusters, 1050 entities)
=============================================

[1] Storage Layer (622 entities)
    +-- cozo_client.rs
    |   +-- CozoDbStorage::new
    |   +-- CozoDbStorage::create_schema
    |   +-- CozoDbStorage::insert_entity
    +-- prd_schema_definition_tables.rs
        +-- create_code_dependency_edge_graph
        +-- create_entity_computed_metrics_cache

[2] HTTP Handlers (45 entities)
    +-- health_check_handler.rs
    +-- statistics_handler.rs
    ...
```

---

### Pattern 4: Token Budget Awareness

**Current State**: Returns `tokens` field with estimate
**Claude Code Pattern**: Token-aware context selection

**Proposed Enhancement**: Smart context endpoint improvements
```
GET /smart-context-token-budget?focus=CozoDbStorage&tokens=4000&priority=callers
```

**Response with budget breakdown**:
```json
{
  "budget_allocated": 4000,
  "budget_used": 3847,
  "budget_remaining": 153,
  "selection_strategy": "callers_first",
  "entities": [
    {"key": "...", "tokens": 450, "priority": 1},
    {"key": "...", "tokens": 320, "priority": 2}
  ],
  "truncation_applied": false
}
```

---

## Part 3: Functional Improvements

### F1: Language Detection Population

**Issue**: `languages_detected_list` always empty
**Impact**: Missing codebase language overview

**Solution**: Populate during ingestion
```rust
// In pt01 streamer, track languages
let mut languages_seen: HashSet<String> = HashSet::new();
for entity in &entities {
    languages_seen.insert(entity.language.clone());
}
// Store in metadata
```

---

### F2: Test Entity Classification

**Issue**: `test_entities_total_count: 0` despite test files
**Impact**: Missing TDD metrics

**Solution**: Verify test detection logic
```rust
// Current test detection patterns
fn classify_entity(entity: &CodeEntity) -> EntityClass {
    // Check file path patterns
    if entity.file_path.contains("/tests/")
        || entity.file_path.contains("_test.rs")
        || entity.name.starts_with("test_") {
        return EntityClass::TEST;
    }
    EntityClass::CODE
}
```

---

### F3: External vs Internal Entity Distinction

**Issue**: Many edges point to `rust:fn:X:unknown:0-0`
**Impact**: Noise in analysis results

**Solution**: Add `entity_scope` field
```json
{
  "key": "rust:fn:unwrap:unknown:0-0",
  "entity_scope": "external",  // vs "internal"
  "entity_source": "std"       // standard library
}
```

**Benefits**:
- Filter stdlib from hotspots
- Focus blast radius on project code
- Cleaner dependency graphs

---

### F4: Hotspots Filtering

**Issue**: Standard library dominates complexity hotspots
**Impact**: Project-specific hotspots hidden

**Solution**: Add `scope` filter
```
GET /complexity-hotspots-ranking-view?top=10&scope=internal
```

**Filtered Response** (internal only):
```json
{
  "hotspots": [
    {"rank": 1, "entity_key": "rust:impl:CozoDbStorage", "total_coupling": 89},
    {"rank": 2, "entity_key": "rust:method:row_to_entity", "total_coupling": 67},
    {"rank": 3, "entity_key": "rust:fn:handle_blast_radius", "total_coupling": 45}
  ]
}
```

---

## Part 4: Architecture Recommendations

### A1: Query Result Caching

**Rationale**: Complex graph queries are expensive
**Proposal**: Add optional Redis/in-memory cache

```rust
pub struct CachedQueryResult {
    query_hash: String,
    result: serde_json::Value,
    computed_at: DateTime<Utc>,
    ttl_seconds: u32,
}
```

---

### A2: Streaming for Large Results

**Rationale**: 3006 edges requires pagination
**Proposal**: Server-Sent Events for streaming

```
GET /dependency-edges-list-all?format=sse
```

```
event: edge
data: {"from_key": "...", "to_key": "...", "edge_type": "Calls"}

event: edge
data: {"from_key": "...", "to_key": "...", "edge_type": "Uses"}

event: complete
data: {"total_count": 3006}
```

---

### A3: OpenAPI Specification

**Rationale**: Better tooling integration
**Proposal**: Auto-generate OpenAPI spec

```
GET /openapi.json
```

Benefits:
- Swagger UI integration
- Client SDK generation
- Type-safe API consumers

---

## Part 5: Implementation Priority Matrix

| Improvement | Impact | Effort | Priority |
|-------------|--------|--------|----------|
| Fix Bug #1 (Entity Detail) | HIGH | LOW | P0 |
| Fix Bug #2 (Reverse Callers) | HIGH | MEDIUM | P0 |
| Fix Bug #3 (Blast Radius) | HIGH | MEDIUM | P0 |
| Language Detection | LOW | LOW | P2 |
| Test Classification | MEDIUM | MEDIUM | P1 |
| External/Internal Scope | HIGH | MEDIUM | P1 |
| Compact Format Option | MEDIUM | LOW | P1 |
| Tree Visualization | MEDIUM | MEDIUM | P2 |
| Streaming Support | LOW | HIGH | P3 |
| OpenAPI Spec | MEDIUM | LOW | P2 |

---

## Conclusion

The dogfooding experiment revealed 3 critical bugs (all in dependency direction/key matching) and 7 improvement opportunities. The core ingestion and graph storage works correctly. HTTP API coverage is comprehensive.

**Recommended Next Version Goals (v1.0.4)**:
1. Fix all 3 critical bugs
2. Add internal/external scope filtering
3. Implement compact response format
4. Populate language detection

**Estimated Effort**: 2-3 focused development sessions

---

*This research document is designed for cross-LLM analysis. Feed to other models for alternative improvement suggestions.*
