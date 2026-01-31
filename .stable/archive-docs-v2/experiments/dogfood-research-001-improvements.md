# Research Document: Parseltongue Improvement Analysis

**Experiment ID**: `dogfood-001`
**Date**: 2025-12-01
**Source**: Dogfooding Experiment Results + Claude Code Visual Patterns Analysis
**Author**: Claude Code Analysis Agent

---

## Executive Summary

Based on comprehensive dogfooding of parseltongue on its own codebase, this document presents:
1. **Root Cause Analysis** of critical issues
2. **Visual/UX Improvements** inspired by Claude Code patterns
3. **Prioritized Roadmap** for fixes and enhancements

**Key Finding**: 4 of 15 endpoints (27%) are broken due to a single root cause - key format mismatch during ingestion.

---

## Part 1: Critical Bug Analysis

### 1.1 Key Format Mismatch (CRITICAL)

**Symptom**: Graph query endpoints return "No callers/callees found"

**Root Cause Investigation**:

```
Entity Keys (CodeGraph):
  rust:fn:new:__crates_parseltongue-core_src_storage_rs:45-72
                ↑
         Uses: __crates_ (underscores)

Edge Keys (DependencyEdges):
  rust:file:./crates/parseltongue-core/src/entities.rs:1-1
            ↑
         Uses: ./crates/ (slashes)
```

**Why This Happens**:
- Entity creation normalizes paths: `./` → ``, `/` → `_`
- Edge creation preserves original file paths
- These transformations happen in different code paths

**Affected Endpoints**:
| Endpoint | Query Pattern | Fails Because |
|----------|--------------|---------------|
| `/reverse-callers-query-graph` | `?[from] := *DependencyEdges{from, to: $entity_key}` | Entity key format doesn't match `to` field |
| `/forward-callees-query-graph` | `?[to] := *DependencyEdges{from: $entity_key, to}` | Entity key format doesn't match `from` field |
| `/blast-radius-impact-analysis` | Recursive edge traversal | Same mismatch |
| `/smart-context-token-budget` | Builds context from edges | Same mismatch |

**Fix Strategy**:
```rust
// Option A: Normalize at query time (quick fix)
fn normalize_entity_key_for_edge_query(key: &str) -> String {
    key.replace("__", "./").replace("_", "/")
}

// Option B: Normalize at ingestion time (proper fix)
// In pt01: Ensure edges use same format as entities
```

**Recommendation**: Option B - Fix at ingestion. One-time cost, prevents future issues.

---

### 1.2 Ingestion Format Fix

**Location**: `crates/pt01-folder-to-cozodb-streamer/src/`

**Current Behavior**:
```rust
// Entity path normalization (happens)
let normalized_path = path.replace("./", "").replace("/", "_");

// Edge creation (doesn't normalize)
let edge_from = format!("rust:file:{}", file_path); // Uses original path
```

**Proposed Fix**:
```rust
// Shared normalization function
fn normalize_file_path_format(path: &str) -> String {
    path.replace("./", "__")
        .replace("/", "_")
        .replace("-", "_")
}

// Use in both entity and edge creation
```

---

## Part 2: Visual/UX Improvements (Claude Code Patterns)

### 2.1 Current State Assessment

| Aspect | Current | Claude Code Pattern | Gap |
|--------|---------|---------------------|-----|
| **Box Drawing** | Server startup only | Pervasive | Low usage |
| **Progressive Disclosure** | None | Summary → Detail | Missing |
| **Confidence Scores** | None | 0.0-1.0 visual | Missing |
| **Token Budgeting** | Shows count | Visual bar | Minimal |
| **Color Coding** | None | Severity/Type | Missing |

### 2.2 Proposed Visual Enhancements

#### 2.2.1 Box Drawing for Responses (Terminal Mode)

**Current**:
```json
{"success": true, "data": {"total": 227}}
```

**Proposed**:
```
┌─────────────────────────────────────────────┐
│  Code Entities: 227                         │
│  Dependencies:  4220                        │
│  Languages:     rust                        │
└─────────────────────────────────────────────┘
```

**Implementation**: Add `?format=terminal` query param

#### 2.2.2 Progressive Disclosure

**Current**: All data returned at once

**Proposed** (3-Level System):
```
Level 0: Summary only (35 tokens)
  curl /blast-radius-impact-analysis?entity=X

Level 1: + Entity keys (500 tokens)
  curl /blast-radius-impact-analysis?entity=X&detail=keys

Level 2: + Full context (2000 tokens)
  curl /blast-radius-impact-analysis?entity=X&detail=full
```

#### 2.2.3 Confidence Scoring Visual

**For Complexity Hotspots**:
```json
{
  "hotspots": [
    {
      "entity": "rust:fn:process_file",
      "coupling": 45,
      "confidence": 0.92,
      "bar": "████████████████████░░░░"  // 20-char visual
    }
  ]
}
```

#### 2.2.4 Token Budget Visual

**Current**:
```json
{"tokens_used": 3850, "token_budget": 4000}
```

**Proposed**:
```json
{
  "token_budget": 4000,
  "tokens_used": 3850,
  "budget_visual": "[████████████████████░] 96%",
  "remaining": 150
}
```

---

## Part 3: Endpoint-Specific Improvements

### 3.1 `/codebase-statistics-overview-summary`

**Issue**: `languages_detected_list: []`

**Fix**: Track language during entity creation:
```rust
// In entity creation loop
languages_seen.insert(entity.language.clone());

// After ingestion
db.put_relation("LanguagesDetected", languages_seen)?;
```

### 3.2 `/api-reference-documentation-help`

**Issue**: Shows 13 endpoints, should show 15

**Fix**: Add missing endpoints to documentation handler:
```rust
endpoints.push(EndpointDoc {
    path: "/temporal-coupling-hidden-deps",
    method: "GET",
    description: "Hidden temporal dependencies",
});
endpoints.push(EndpointDoc {
    path: "/smart-context-token-budget",
    method: "GET",
    description: "Optimal context selection within token budget",
});
```

### 3.3 `/complexity-hotspots-ranking-view`

**Issue**: Shows stdlib functions (`new`, `unwrap`, `Ok`)

**Fix**: Filter entities with unknown file path:
```rust
// In Datalog query
?[entity_key, coupling] :=
    *CodeGraph{Key: entity_key, file_path},
    file_path != "unknown",  // Filter stdlib
    // ... rest of query
```

### 3.4 `/semantic-cluster-grouping-list`

**Issue**: Returns `total_clusters: null`

**Fix**: Return actual count:
```rust
let cluster_count = clusters.len();
json!({
    "total_clusters": cluster_count,  // Not null
    "clusters": clusters
})
```

---

## Part 4: pt01 Output Modernization

### 4.1 Current (Legacy JSON Focus)

```
Next steps:
  Export edges:    parseltongue pt02-level00 --where-clause "ALL" ...
  Export entities: parseltongue pt02-level01 --include-code 0 ...
```

### 4.2 Proposed (HTTP Server Focus)

```
┌─────────────────────────────────────────────────────────────────┐
│  Ingestion Complete                                             │
├─────────────────────────────────────────────────────────────────┤
│  Entities: 227    Edges: 4220    Duration: 1.8s                 │
└─────────────────────────────────────────────────────────────────┘

Next step:
  parseltongue serve-http-code-backend \
    --db "rocksdb:myproject/analysis.db" --port 8080

Quick test:
  curl http://localhost:8080/server-health-check-status
  curl http://localhost:8080/codebase-statistics-overview-summary
```

---

## Part 5: Prioritized Roadmap

### P0 - Critical (Blocking Release)

| # | Task | Impact | Effort |
|---|------|--------|--------|
| 1 | Fix key format mismatch | Unblocks 4 endpoints | Medium |
| 2 | Update pt01 output | User experience | Low |

### P1 - High Priority

| # | Task | Impact | Effort |
|---|------|--------|--------|
| 3 | Populate languages_detected_list | Data completeness | Low |
| 4 | Update API docs to 15 endpoints | Documentation | Low |
| 5 | Filter stdlib from hotspots | Data quality | Low |

### P2 - Medium Priority (UX Polish)

| # | Task | Impact | Effort |
|---|------|--------|--------|
| 6 | Fix semantic cluster count | Data quality | Low |
| 7 | Add terminal format option | UX | Medium |
| 8 | Add progressive disclosure | Token efficiency | Medium |
| 9 | Add visual confidence bars | UX | Low |

### P3 - Future Enhancements

| # | Task | Impact | Effort |
|---|------|--------|--------|
| 10 | Real git log parsing for temporal coupling | Feature completeness | High |
| 11 | Streaming responses for large results | Performance | High |
| 12 | Color coding in terminal output | UX | Medium |

---

## Part 6: Verification Plan

After implementing fixes:

```bash
# 1. Re-run dogfooding
./target/release/parseltongue pt01-folder-to-cozodb-streamer . --db "rocksdb:verify.db"

# 2. Start server
./target/release/parseltongue serve-http-code-backend --db "rocksdb:verify.db" --port 8080

# 3. Test previously broken endpoints
curl "http://localhost:8080/reverse-callers-query-graph?entity=rust:fn:new:__crates_parseltongue-core_src_storage_rs:45-72"
# Expected: success: true, callers list

curl "http://localhost:8080/blast-radius-impact-analysis?entity=rust:fn:new:__crates_parseltongue-core_src_storage_rs:45-72&hops=2"
# Expected: success: true, total_affected > 0

# 4. Verify pt01 output
./target/release/parseltongue pt01-folder-to-cozodb-streamer . --db "rocksdb:test.db" 2>&1 | grep -A5 "Next"
# Expected: Shows serve-http-code-backend, NOT pt02-level00
```

---

## Appendix A: Claude Code Visual Patterns Reference

Key patterns from `.claude/refDocsClaudeCode/`:

1. **Minimalist ASCII**: Use `─│┌┐└┘├┤┬┴┼` for structure
2. **Information Density**: Show most important info first
3. **Token Awareness**: Always show token cost
4. **Progressive Disclosure**: Summary → Detail → Full
5. **Confidence Visual**: `████░░░░` style bars

---

## Appendix B: Test Matrix Post-Fix

| Endpoint | Pre-Fix | Post-Fix | Test |
|----------|---------|----------|------|
| /server-health-check-status | PASS | PASS | Existing |
| /codebase-statistics-overview-summary | PARTIAL | PASS | Verify languages |
| /api-reference-documentation-help | PARTIAL | PASS | Verify 15 endpoints |
| /code-entities-list-all | PASS | PASS | Existing |
| /code-entities-search-fuzzy | PASS | PASS | Existing |
| /code-entity-detail-view/{key} | UNTESTED | PASS | Add test |
| /dependency-edges-list-all | PASS | PASS | Existing |
| /reverse-callers-query-graph | BROKEN | PASS | Verify returns callers |
| /forward-callees-query-graph | BROKEN | PASS | Verify returns callees |
| /blast-radius-impact-analysis | BROKEN | PASS | Verify affected > 0 |
| /circular-dependency-detection-scan | PASS | PASS | Existing |
| /complexity-hotspots-ranking-view | PARTIAL | PASS | Verify no stdlib |
| /semantic-cluster-grouping-list | PARTIAL | PASS | Verify count != null |
| /smart-context-token-budget | BROKEN | PASS | Verify context populated |
| /temporal-coupling-hidden-deps | PASS | PASS | Existing |

---

*Research document completed: 2025-12-01*
*Next action: Implement P0 fixes (key format mismatch)*
