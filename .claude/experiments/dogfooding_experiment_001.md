# Dogfooding Experiment #001

**Date**: 2025-12-01
**Version**: v1.0.3
**Target Codebase**: parseltongue-dependency-graph-generator (self-analysis)

---

## Executive Summary

Comprehensive end-to-end test of Parseltongue analyzing its own codebase. Tests all 13 HTTP API endpoints. Identifies 3 critical issues and 7 improvement opportunities.

**Overall Score**: 10/13 endpoints working correctly (77%)

---

## Phase 1: Ingestion (pt01-folder-to-cozodb-streamer)

### Command Executed
```bash
./target/release/parseltongue pt01-folder-to-cozodb-streamer .
```

### Results
| Metric | Value |
|--------|-------|
| Workspace Created | `parseltongue20251201170547/` |
| Database | `rocksdb:parseltongue20251201170547/analysis.db` |
| Files Processed | ~50 Rust files |
| Entities Created | 214 |
| Dependency Edges | 3006 |
| Languages Detected | Rust |

### Observations
- Auto-timestamped workspace creation: **Working**
- RocksDB initialization: **Working**
- Tree-sitter parsing: **Working**
- Entity extraction includes: functions, structs, enums, impls, methods, modules

---

## Phase 2: HTTP Server (pt08-http-code-query-server)

### Command Executed
```bash
./target/release/parseltongue pt08-http-code-query-server \
  --db "rocksdb:parseltongue20251201170547/analysis.db"
```

### Server Startup
- Default Port: **7777**
- Status: **Running**
- Database Connection: **Successful**

---

## Phase 3: Endpoint Testing Results

### Category: Core Endpoints

#### 1. `/server-health-check-status`
**Status**: PASS
```json
{
  "success": true,
  "status": "ok",
  "server_uptime_seconds_count": 116,
  "endpoint": "/server-health-check-status"
}
```

#### 2. `/codebase-statistics-overview-summary`
**Status**: PASS
```json
{
  "success": true,
  "data": {
    "code_entities_total_count": 214,
    "test_entities_total_count": 0,
    "dependency_edges_total_count": 3006,
    "languages_detected_list": [],
    "database_file_path": "rocksdb:parseltongue20251201170547/analysis.db"
  },
  "tokens": 50
}
```
**Issue Found**: `languages_detected_list` is empty despite Rust being parsed.

#### 3. `/api-reference-documentation-help`
**Status**: PASS
- Returns 13 endpoints in 4 categories
- Documentation structure: Complete

---

### Category: Entity Endpoints

#### 4. `/code-entities-list-all`
**Status**: PASS
- Total Count: 214 entities
- Entity Types: enum, function, impl, method, module, struct
- Entity Class: All marked as "CODE"

#### 5. `/code-entity-detail-view/{key}`
**Status**: FAIL
- URL-encoded key returns empty response
- Tested: `rust%3Afn%3Amain%3A__crates_parseltongue_src_main_rs%3A18-40`
- Result: Empty response (no error, no data)
- **Bug**: Path parameter parsing may have issue with URL decoding

#### 6. `/code-entities-search-fuzzy?q=pattern`
**Status**: PASS
```
Query: "handle"
Results: 121 entities matching
```
- Fast substring matching
- Returns full entity metadata

---

### Category: Edge Endpoints

#### 7. `/dependency-edges-list-all`
**Status**: PASS
- Total Count: 3006 edges
- Paginated: 100 per request
- Edge Types: Uses, Calls

#### 8. `/reverse-callers-query-graph?entity=X`
**Status**: FAIL (Potential Bug)
```json
{
  "success": false,
  "error": "No callers found for entity: rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs:38-54"
}
```
**Issue**: `CozoDbStorage::new()` is called 215 times per complexity hotspots, but reverse callers returns 0.

#### 9. `/forward-callees-query-graph?entity=X`
**Status**: PASS
```json
{
  "total_count": 8,
  "callees": [
    {"to_key": "rust:fn:Ok:unknown:0-0", "edge_type": "Calls"},
    {"to_key": "rust:fn:collect:unknown:0-0", "edge_type": "Calls"},
    ...
  ]
}
```

---

### Category: Analysis Endpoints

#### 10. `/blast-radius-impact-analysis?entity=X&hops=N`
**Status**: FAIL (Bug)
```json
{
  "success": false,
  "error": "No affected entities found for: rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs:38-54"
}
```
**Issue**: Blast radius should use REVERSE dependencies (who calls me) but appears to use forward deps.

#### 11. `/circular-dependency-detection-scan`
**Status**: PASS
```json
{
  "success": true,
  "data": {
    "has_cycles": false,
    "cycle_count": 0,
    "cycles": []
  }
}
```
- Clean codebase with no circular dependencies

#### 12. `/complexity-hotspots-ranking-view?top=N`
**Status**: PASS
```json
{
  "hotspots": [
    {"rank": 1, "entity_key": "rust:fn:new:unknown:0-0", "inbound_count": 215},
    {"rank": 2, "entity_key": "rust:fn:unwrap:unknown:0-0", "inbound_count": 163},
    {"rank": 3, "entity_key": "rust:fn:to_string:unknown:0-0", "inbound_count": 138}
  ]
}
```
**Observation**: Standard library functions dominate. Filter for project-specific entities would improve usefulness.

#### 13. `/semantic-cluster-grouping-list`
**Status**: PASS
```json
{
  "total_entities": 1050,
  "cluster_count": 43,
  "clusters": [
    {"cluster_id": 1, "entity_count": 622},
    ...
  ]
}
```
- Label propagation clustering working
- 43 semantic clusters identified

---

## Critical Issues Found

### Issue #1: Entity Detail View Returns Empty
**Severity**: HIGH
**Endpoint**: `/code-entity-detail-view/{key}`
**Expected**: Entity details for given key
**Actual**: Empty response
**Root Cause**: Likely URL decoding issue with path parameters in Axum

### Issue #2: Reverse Callers Not Working
**Severity**: HIGH
**Endpoint**: `/reverse-callers-query-graph`
**Expected**: 215 callers for `new()` function
**Actual**: "No callers found"
**Root Cause**: Query direction may be inverted or key matching issue

### Issue #3: Blast Radius Uses Wrong Direction
**Severity**: HIGH
**Endpoint**: `/blast-radius-impact-analysis`
**Expected**: "If I change X, what breaks?" (uses reverse deps)
**Actual**: Returns 0 affected entities
**Root Cause**: May be querying forward dependencies instead of reverse

---

## Improvement Opportunities

### I1: Languages Detected List Empty
The `/codebase-statistics-overview-summary` returns empty `languages_detected_list` despite parsing Rust files.

### I2: Complexity Hotspots Need Project Filter
Standard library functions (`new`, `unwrap`, `to_string`) dominate hotspots. Add option to filter to project-specific entities only.

### I3: Token Estimation
All responses include `"tokens"` field. Verify accuracy and add model-specific tokenization options.

### I4: Entity Class Classification
All 214 entities marked as "CODE" with `test_entities_total_count: 0`, despite test files existing. Test detection may need review.

### I5: Unknown Location Entities
Many edges point to `rust:fn:X:unknown:0-0` entities (standard library). Consider:
- Filtering these from results
- Or marking them distinctly as "external"

### I6: Pagination UX
Edge list returns 100 items with total 3006. API returns `offset` and `limit` but no `next_page_url` convenience field.

### I7: Response Time Metrics
Add optional timing information to responses for performance monitoring.

---

## Test Matrix Summary

| Endpoint | Status | Notes |
|----------|--------|-------|
| Health Check | PASS | Working |
| Codebase Stats | PASS | languages_detected empty |
| API Reference | PASS | Complete |
| List Entities | PASS | 214 entities |
| Entity Detail | FAIL | Empty response |
| Fuzzy Search | PASS | 121 matches |
| Edges List | PASS | 3006 edges |
| Reverse Callers | FAIL | Returns 0 |
| Forward Callees | PASS | 8 callees |
| Blast Radius | FAIL | Wrong direction |
| Cycle Detection | PASS | No cycles |
| Hotspots | PASS | Top 5 working |
| Clusters | PASS | 43 clusters |

**Pass Rate**: 10/13 (77%)

---

## Appendix: Claude Code Design Pattern Analysis

Based on `.claude/refDocsClaudeCode/` reference documents:

### Applicable Patterns

1. **Minimalist Visuals**: Current JSON responses are functional but verbose. Consider adding compact summary modes.

2. **Progressive Disclosure**: Already implemented via pagination. Could extend to nested entity details.

3. **Async Generators**: Not currently used. Could improve streaming for large result sets.

4. **Context Isolation**: Each endpoint is stateless - good pattern adherence.

### Recommended Visual Improvements

1. Add ASCII graph visualization for `/blast-radius-impact-analysis`
2. Tree view format option for `/semantic-cluster-grouping-list`
3. Sparkline metrics in `/complexity-hotspots-ranking-view`

---

## Next Steps

1. Fix Issue #1: Debug Axum path parameter decoding
2. Fix Issue #2: Verify CozoDB query direction for reverse callers
3. Fix Issue #3: Correct blast radius to use reverse dependencies
4. Add project-specific filter to hotspots
5. Implement language detection population

---

*Experiment conducted using Claude Code. DO NOT DELETE - Will be used for cross-LLM improvement analysis.*
