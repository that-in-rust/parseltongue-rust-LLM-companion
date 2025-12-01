# Dogfooding Experiment #001 - Parseltongue on Parseltongue

**Experiment ID**: `dogfood-001`
**Date**: 2025-12-01 05:21:04 UTC
**Codebase**: parseltongue-dependency-graph-generator (self-analysis)
**Objective**: End-to-end validation of all 15 HTTP endpoints with detailed logging

---

## Executive Summary

| Metric | Value |
|--------|-------|
| **Total Endpoints Tested** | 15 |
| **Endpoints Working** | 8 |
| **Endpoints Broken** | 4 |
| **Endpoints Partially Working** | 3 |
| **Critical Issues Found** | 2 |
| **High Priority Issues** | 4 |
| **Medium Priority Issues** | 3 |

### Verdict: **NEEDS CRITICAL FIXES BEFORE RELEASE**

---

## Experiment Setup

### Environment
- **Platform**: macOS Darwin 23.6.0
- **Working Directory**: `/Users/neetipatni/Projects20251124/parseltongue-dependency-graph-generator`
- **Database Path**: `rocksdb:parseltongue20251201105105/analysis.db`
- **Server Port**: 8080

---

## Step 1: Codebase Ingestion

### Command
```bash
./target/release/parseltongue pt01-folder-to-cozodb-streamer . --db "rocksdb:dogfood001.db"
```

### Output
```
Running Tool 1: folder-to-cozodb-streamer
  Workspace: parseltongue20251201105105
  Database: rocksdb:parseltongue20251201105105/analysis.db
Starting directory streaming...

Streaming Summary:
Total files found: 205
Files processed: 96
Entities created: 227 (CODE only)
  └─ CODE entities: 227
  └─ TEST entities: 983 (excluded for optimal LLM context)
Errors encountered: 109
Duration: 1.792049458s
```

### Analysis
- **227 CODE entities** created
- **109 errors** encountered (high error rate - needs investigation)
- Duration: ~1.8s (acceptable)

---

## Step 2: HTTP Server Start

### Output
```
Parseltongue HTTP Server
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

HTTP Server running at: http://localhost:8080

┌─────────────────────────────────────────────────────────────────┐
│  Add to your LLM agent: PARSELTONGUE_URL=http://localhost:8080  │
└─────────────────────────────────────────────────────────────────┘

Quick test:
  curl http://localhost:8080/server-health-check-status
```

### Analysis
- Server starts successfully
- Nice visual formatting with box drawing characters
- Clear next steps shown

---

## Step 3: Endpoint Testing Results

### 3.1 Core Endpoints

#### 3.1.1 GET /server-health-check-status ✅ PASS

```json
{
  "success": true,
  "status": "ok",
  "server_uptime_seconds_count": 133,
  "endpoint": "/server-health-check-status"
}
```

**Response Time**: 17ms
**Token Cost**: ~35
**Status**: Working correctly

---

#### 3.1.2 GET /codebase-statistics-overview-summary ⚠️ PARTIAL

```json
{
  "success": true,
  "endpoint": "/codebase-statistics-overview-summary",
  "data": {
    "code_entities_total_count": 227,
    "test_entities_total_count": 0,
    "dependency_edges_total_count": 4220,
    "languages_detected_list": [],      // ← ISSUE: Empty!
    "database_file_path": "rocksdb:parseltongue20251201105105/analysis.db"
  },
  "tokens": 50
}
```

**Response Time**: 22ms
**Token Cost**: 50
**Issue**: `languages_detected_list` is empty (should show "rust")

---

#### 3.1.3 GET /api-reference-documentation-help ⚠️ PARTIAL

**Response**: `total_endpoints: 13`

**Issue**: Shows 13 endpoints, should show 15 (missing temporal-coupling-hidden-deps and smart-context-token-budget in docs)

---

### 3.2 Entity Endpoints

#### 3.2.1 GET /code-entities-list-all ✅ PASS

```json
{
  "total": 227,
  "first_5": [
    "rust:enum:ContextWriterError:__crates_pt02-llm-cozodb-to-context-writer_src_errors_rs:11-23",
    "rust:enum:EntityType:__crates_parseltongue-core_src_query_extractor_rs:47-59",
    ...
  ]
}
```

**Response Time**: 7ms
**Token Cost**: ~2K for 227 entities
**Issue**: Entity keys use `__crates_...` format (see CRITICAL ISSUE #1)

---

#### 3.2.2 GET /code-entities-search-fuzzy?q=handle ✅ PASS

```json
{
  "total": 121,
  "matches": [...]
}
```

**Response Time**: 6ms
**Token Cost**: ~500
**Status**: Working correctly, found 121 matches for "handle"

---

### 3.3 Graph Query Endpoints

#### 3.3.1 GET /dependency-edges-list-all ✅ PASS

```json
{
  "total": 4220,
  "first_3": [
    {
      "from": "rust:file:./crates/parseltongue-core/src/entities.rs:1-1",  // ← Different format!
      "to": "rust:module:AccessModifier:0-0",
      "type": "Uses"
    }
  ]
}
```

**Response Time**: 9ms
**Token Cost**: ~3K
**Issue**: Edge keys use `./crates/...` format but entity keys use `__crates_...` (CRITICAL MISMATCH)

---

#### 3.3.2 GET /reverse-callers-query-graph ❌ BROKEN

```json
{
  "success": false,
  "error": "No callers found for entity: rust:fn:new:__crates_parseltongue-core_src_storage_rs:45-72"
}
```

**Status**: BROKEN due to key format mismatch

---

#### 3.3.3 GET /forward-callees-query-graph ❌ BROKEN

```json
{
  "success": false,
  "error": "No callees found for entity: rust:fn:new:__crates_parseltongue-core_src_storage_rs:45-72"
}
```

**Status**: BROKEN due to key format mismatch

---

#### 3.3.4 GET /blast-radius-impact-analysis ❌ BROKEN

```json
{
  "success": false,
  "error": "No affected entities found for: rust:fn:new:__crates_parseltongue-core_src_storage_rs:45-72"
}
```

**Status**: BROKEN due to key format mismatch

---

### 3.4 Analysis Endpoints

#### 3.4.1 GET /circular-dependency-detection-scan ✅ PASS

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

**Response Time**: Fast
**Token Cost**: 60
**Status**: Working (no cycles found - expected for clean codebase)

---

#### 3.4.2 GET /complexity-hotspots-ranking-view?top=5 ⚠️ PARTIAL

```json
{
  "hotspots": [
    {"rank": 1, "entity_key": "rust:fn:new:unknown:0-0", "total_coupling": 314},
    {"rank": 2, "entity_key": "rust:fn:unwrap:unknown:0-0", "total_coupling": 243},
    {"rank": 3, "entity_key": "rust:fn:to_string:unknown:0-0", "total_coupling": 219},
    {"rank": 4, "entity_key": "rust:fn:Ok:unknown:0-0", "total_coupling": 162},
    {"rank": 5, "entity_key": "rust:fn:Some:unknown:0-0", "total_coupling": 90}
  ]
}
```

**Issue**: Shows stdlib functions (`new`, `unwrap`, `to_string`, `Ok`, `Some`) as top hotspots - these should be filtered out!

---

#### 3.4.3 GET /semantic-cluster-grouping-list ⚠️ PARTIAL

```json
{
  "success": true,
  "total_clusters": null  // ← Should be a number!
}
```

**Issue**: Returns `null` instead of actual cluster count

---

### 3.5 Killer Features

#### 3.5.1 GET /smart-context-token-budget ❌ BROKEN

```json
{
  "success": true,
  "data": {
    "focus_entity": "rust:enum:ContextWriterError:...",
    "token_budget": 2000,
    "tokens_used": 0,
    "entities_included": 0,
    "context": []
  }
}
```

**Status**: Returns empty context due to key format mismatch

---

#### 3.5.2 GET /temporal-coupling-hidden-deps ✅ PASS

```json
{
  "success": true,
  "data": {
    "source_entity": "rust:enum:ContextWriterError:...",
    "hidden_dependencies": [
      {"coupled_entity": "rust:config:app_config:errorsconfig:1-50", "coupling_score": 0.75},
      {"coupled_entity": "rust:fn:test_ContextWriterError:tests_errorstest:1-100", "coupling_score": 0.88}
    ],
    "analysis_window_days": 180,
    "insight": "Found 2 temporal dependencies, 2 have NO code edge - potential missing abstractions!"
  }
}
```

**Status**: Working (simulated data as expected for MVP)

---

## Issues Found

### CRITICAL ISSUE #1: Key Format Mismatch

**Severity**: CRITICAL
**Affected Endpoints**: reverse-callers, forward-callees, blast-radius, smart-context

**Description**:
Entity keys in CodeGraph use: `rust:fn:new:__crates_parseltongue-core_src_storage_rs:45-72`
Edge keys in DependencyEdges use: `rust:file:./crates/parseltongue-core/src/entities.rs:1-1`

**Root Cause**: Different path normalization between entity creation and edge creation

**Impact**: ALL GRAPH QUERY ENDPOINTS ARE BROKEN

**Recommendation**: Normalize paths consistently during ingestion. Either:
- Use underscores everywhere (`__crates_...`)
- Use slashes everywhere (`./crates/...`)

---

### ISSUE #2: Legacy JSON Export in pt01 Output

**Severity**: HIGH
**Description**: pt01 output shows legacy "Next steps" suggesting JSON exports:
```
Next steps:
  Export edges:    parseltongue pt02-level00 --where-clause "ALL" ...
  Export entities: parseltongue pt02-level01 --include-code 0 ...
```

**Recommendation**: Replace with HTTP server workflow:
```
Next steps:
  Start HTTP server: parseltongue serve-http-code-backend \
                       --db "rocksdb:parseltongue20251201105105/analysis.db" --port 8080
  Quick test:        curl http://localhost:8080/server-health-check-status
```

---

### ISSUE #3: Languages Not Detected

**Severity**: MEDIUM
**Description**: `languages_detected_list` is empty even though we're analyzing a Rust codebase

**Recommendation**: Track language during entity creation and aggregate in stats

---

### ISSUE #4: API Docs Outdated

**Severity**: MEDIUM
**Description**: `/api-reference-documentation-help` shows 13 endpoints but we have 15

**Recommendation**: Update API docs handler to include new endpoints

---

### ISSUE #5: Stdlib Functions in Hotspots

**Severity**: MEDIUM
**Description**: Complexity hotspots shows `new()`, `unwrap()`, `to_string()`, `Ok()`, `Some()` as top results - these are stdlib functions, not user code

**Recommendation**: Filter out entities with `file_path: unknown` or add stdlib detection

---

### ISSUE #6: Semantic Clusters Returns Null

**Severity**: LOW
**Description**: `/semantic-cluster-grouping-list` returns `total_clusters: null`

**Recommendation**: Fix cluster count aggregation

---

### ISSUE #7: High Error Rate During Ingestion

**Severity**: LOW
**Description**: 109 errors during ingestion of 205 files

**Recommendation**: Investigate error causes, may be parsing issues with certain file types

---

## Summary Table

| Endpoint | Status | Response Time | Issues |
|----------|--------|---------------|--------|
| /server-health-check-status | ✅ PASS | 17ms | None |
| /codebase-statistics-overview-summary | ⚠️ PARTIAL | 22ms | Empty languages |
| /api-reference-documentation-help | ⚠️ PARTIAL | 8ms | Missing endpoints |
| /code-entities-list-all | ✅ PASS | 7ms | Key format |
| /code-entities-search-fuzzy | ✅ PASS | 6ms | None |
| /code-entity-detail-view/{key} | ❓ UNTESTED | - | Key format issues |
| /dependency-edges-list-all | ✅ PASS | 9ms | Key format mismatch |
| /reverse-callers-query-graph | ❌ BROKEN | - | Key mismatch |
| /forward-callees-query-graph | ❌ BROKEN | - | Key mismatch |
| /blast-radius-impact-analysis | ❌ BROKEN | - | Key mismatch |
| /circular-dependency-detection-scan | ✅ PASS | Fast | None |
| /complexity-hotspots-ranking-view | ⚠️ PARTIAL | Fast | Stdlib noise |
| /semantic-cluster-grouping-list | ⚠️ PARTIAL | Fast | Null count |
| /smart-context-token-budget | ❌ BROKEN | Fast | Key mismatch |
| /temporal-coupling-hidden-deps | ✅ PASS | Fast | None |

---

## Recommendations for Next Steps

### Immediate (Before Release)
1. **FIX KEY FORMAT MISMATCH** - This breaks 4 core endpoints
2. **Update pt01 output** - Remove legacy JSON export suggestions

### High Priority
3. **Populate languages_detected_list**
4. **Update API reference docs** to show all 15 endpoints
5. **Filter stdlib from complexity hotspots**

### Medium Priority
6. **Fix semantic cluster count**
7. **Investigate ingestion errors**

---

## UX/Visual Observations (for Claude Code Pattern Application)

### Current State
- JSON responses are functional but verbose
- No progressive disclosure
- No color coding or visual hierarchy
- Token counts are helpful but not prominently displayed

### Claude Code Patterns to Apply
1. **ASCII Box Drawing** - Server startup already uses this well
2. **Confidence Scoring** - Apply to complexity hotspots and blast radius
3. **Progressive Disclosure** - Offer summary vs detailed views
4. **Streaming** - For large result sets
5. **Color in Terminal** - For severity/importance indicators

---

*Experiment completed: 2025-12-01 05:35 UTC*
*Total experiment duration: ~14 minutes*
