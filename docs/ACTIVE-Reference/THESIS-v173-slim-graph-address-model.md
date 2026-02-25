# THESIS: v1.7.3 — Slim Graph+Address Model for pt02/pt03

**Date**: 2026-02-12
**Method**: Every endpoint queried on live Parseltongue server (3,847 entities, 10,459 edges) to capture exact response shapes. Then analyzed against the slim model at 1.6M edge scale.

---

## Core Idea

**Current pt01**: Stores EVERYTHING — code bodies, signatures, metadata, diagnostics, subfolder data.
**Proposed pt02**: Stores ONLY the dependency graph + a file address per entity.

The LLM uses Parseltongue as a **map/index**, not a code store. It asks "what connects to what" and "where is it" — then reads the files directly.

---

## Slim Entity Schema

```rust
struct SlimEntity {
    isgl1_key: String,    // "rust:fn:main:____crates_parseltongue_src_main:T123"
    file_path: String,    // "./crates/parseltongue/src/main.rs"
    line_range: String,   // "17-40"
    entity_type: String,  // "function"
    language: String,     // "rust"
    // DROPPED: Current_Code, Future_Code, interface_signature, lsp_meta_data,
    //          TDD_Classification, metadata, content_hash, birth_timestamp,
    //          semantic_path, entity_class
}
// ~114 bytes per entity (vs ~3,000 bytes with full model)
```

```rust
struct SlimEdge {
    from_key: String,     // "rust:fn:main:____crates_parseltongue_src_main:T123"
    to_key: String,       // "rust:fn:build_cli:____crates_parseltongue_src_main:T456"
    edge_type: String,    // "Calls"
    // DROPPED: source_location (redundant — from_key contains file+line info)
}
// ~120 bytes per edge
```

---

## RAM Comparison at 1.6M Edges (400K Entities)

| | Full Model | Slim Model |
|--|:-:|:-:|
| Per entity | ~3,000 B | **~114 B** |
| Per edge | ~250 B | **~120 B** |
| 400K entities | 1,200 MB | **46 MB** |
| 1.6M edges | 400 MB | **192 MB** |
| CozoDB mem overhead | 1,600 MB | **238 MB** |
| **Total base RAM** | **3,200 MB** | **~476 MB** |
| `.ptgraph` file size | ~1 GB | **~200 MB** |
| `.json` file size | ~4 GB | **~600 MB** |

---

## Every Endpoint Rated: Build or Drop?

### Rating Scale
- **10/10**: Behavior identical in pt01 and pt02. Pure graph operation. Must build.
- **7-9/10**: Behavior similar but response shape differs (no code body). Still useful.
- **4-6/10**: Behavior partially degraded. Some data missing. Questionable value.
- **1-3/10**: Behavior fundamentally different. Useless or misleading without full data. Drop.

### What I Found Querying Each Endpoint

---

#### 1. `/server-health-check-status` — Rating: **10/10 BUILD**

**pt01 response**: `{ "success": true, "status": "ok", "server_uptime_seconds_count": 7 }`
**pt02 response**: Identical. No data dependency.
**Verdict**: Pure server status. Zero difference.

---

#### 2. `/codebase-statistics-overview-summary` — Rating: **10/10 BUILD**

**pt01 response**: `{ code_entities_total_count: 1597, dependency_edges_total_count: 10459, languages_detected_list: [...] }`
**pt02 response**: Identical. COUNT aggregates work on slim schema.
**Verdict**: Counts and language list are the same regardless of whether code bodies exist.

---

#### 3. `/api-reference-documentation-help` — Rating: **10/10 BUILD**

Static hardcoded documentation. No database dependency.

---

#### 4. `/code-entity-detail-view?key=X` — Rating: **8/10 BUILD**

**pt01 response** (observed): Returns `Current_Code`, `interface_signature`, `TDD_Classification`, etc.
**pt02 response**: Returns `{ key, file_path, line_range, entity_type, language }`.

**Difference**: No code body. But pt02 returns the **address** — the LLM reads the file directly. This is actually MORE useful because:
- Code is always fresh (not a stale snapshot)
- LLM context tools already support file reading
- Less tokens consumed in the API response

---

#### 5. `/reverse-callers-query-graph?entity=X` — Rating: **10/10 BUILD**

**pt01 response**: `{ callees: [{ from_key, to_key, edge_type, source_location }] }`
**pt02 response**: Identical except no `source_location` field. But `from_key` already encodes the file.

**Verdict**: The graph query is purely on DependencyEdges. Slim edges have from_key, to_key, edge_type — everything needed. The `source_location` field is redundant (it's derivable from the ISGL1 key which contains file + line info).

---

#### 6. `/forward-callees-query-graph?entity=X` — Rating: **10/10 BUILD**

Same as #5. Pure edge query. Identical behavior.

---

#### 7. `/dependency-edges-list-all?limit=N` — Rating: **10/10 BUILD**

**pt01 response**: `{ edges: [{ from_key, to_key, edge_type, source_location }], total_count: 10459 }`
**pt02 response**: Same minus `source_location`. Count and pagination identical.

---

#### 8. `/code-entities-list-all` — Rating: **7/10 BUILD**

**pt01 response**: `{ entities: [{ key, file_path, entity_type, entity_class, language }] }`
**pt02 response**: Same minus `entity_class` (CODE vs TEST distinction). Adds `line_range`.

**Issue**: `entity_class` is used to separate code entities from test entities. In slim model we could keep it (it's just one more string) OR drop it and let the LLM decide based on file path patterns (test files are obvious from path).

**At 1.6M scale**: Loads all 400K entities into Vec. Slim = 46 MB. Full = 1.2 GB. The slim model makes this endpoint viable on 8GB.

**RAM**: 476 MB base + 46 MB = **522 MB total. Fits 8GB easily.**

---

#### 9. `/code-entities-search-fuzzy?q=X` — Rating: **6/10 CONDITIONAL BUILD**

**pt01 behavior**: Loads ALL entities, substring-matches on `key` in Rust.
**pt02 behavior**: Same — searches entity key strings.

**But**: The user said "no fuzzy search beyond entities." At slim scale, searching ISGL1 keys is still useful (search by function name). The ISGL1 key contains: `language:type:name:file_hash:timestamp`. Searching "main" would find `rust:fn:main:...`.

**Issue at 1.6M scale**: Still loads all entities to filter. Slim = 46 MB (OK). Full = 1.2 GB (not OK).

**Decision**: Build it but note it searches keys only, not code bodies.

---

#### 10. `/blast-radius-impact-analysis?entity=X&hops=N` — Rating: **10/10 BUILD**

**pt01 response**: BFS traversal, returns entities per hop.
**pt02 response**: Identical. BFS only needs edges. Returns entity keys + addresses per hop.

**RAM**: Per-hop filtered queries. Each hop scans edges but doesn't load ALL into memory at once. ~200-500 MB peak. **Fits 8GB.**

---

#### 11. `/complexity-hotspots-ranking-view?top=N` — Rating: **10/10 BUILD**

**pt01 response**: `{ hotspots: [{ entity_key, inbound_count, outbound_count, total_coupling }] }`
**pt02 response**: Identical. Counts edges per entity — pure graph metric.

**The response only contains counts**, not code. Zero behavioral difference.

**RAM**: Loads all edges into count HashMaps. Slim edges = ~192 MB. HashMap overhead ~100 MB. **Total: ~770 MB. Fits 8GB.**

---

#### 12. `/circular-dependency-detection-scan` — Rating: **10/10 BUILD**

**pt01 response**: `{ has_cycles: false, cycle_count: 0, cycles: [...] }`
**pt02 response**: Identical. DFS cycle detection is a pure graph algorithm.

**RAM**: Full adjacency HashMap. Slim = ~300 MB. **Total: ~780 MB. Fits 8GB.**

---

#### 13. `/semantic-cluster-grouping-list` — Rating: **9/10 BUILD**

**pt01 response**: `{ clusters: [{ entities: [...], size }], cluster_count: 158 }`
**pt02 response**: Identical. Label propagation works on edges only.

**Minus 1**: Cluster names are derived from entity keys. In pt01, richer metadata could help label clusters. But the algorithm itself is graph-only.

**RAM**: Bidirectional graph from edges. Slim = ~400 MB. **Total: ~880 MB. Fits 8GB.**

---

#### 14. `/strongly-connected-components-analysis` — Rating: **10/10 BUILD**

**pt01 response**: `{ scc_count: 0, sccs: [...] }`
**pt02 response**: Identical. Tarjan's algorithm is pure graph.

**RAM**: AdjacencyListGraphRepresentation from slim edges. ~400 MB peak. **Total: ~880 MB. Fits 8GB.**

---

#### 15. `/kcore-decomposition-layering-analysis` — Rating: **10/10 BUILD**

Pure graph algorithm. K-core peeling only needs degree counts.

**RAM**: Similar to SCC. ~400 MB peak. **Total: ~880 MB. Fits 8GB.**

---

#### 16. `/centrality-measures-entity-ranking?method=pagerank` — Rating: **10/10 BUILD**

**pt01 response**: `{ method: "pagerank", entities: [{ entity, score }] }`
**pt02 response**: Identical. PageRank is pure graph.

**RAM**: Adjacency list + score vector (400K × 8 bytes = 3.2 MB). **Total: ~880 MB. Fits 8GB.**

**Note**: Betweenness centrality mode is O(V*E) computation — slow at 1.6M edges but memory is similar. CPU time may be 30-60+ seconds.

---

#### 17. `/entropy-complexity-measurement-scores` — Rating: **10/10 BUILD**

**pt01 response**: `{ entities: [{ entity, entropy, complexity: "LOW" }] }` (observed 3,432 entities)
**pt02 response**: Identical. Shannon entropy computed from edge type distribution — pure graph.

**RAM**: ~400 MB. **Fits 8GB.**

---

#### 18. `/coupling-cohesion-metrics-suite` — Rating: **10/10 BUILD**

**pt01 response**: `{ entities: [{ entity, cbo, lcom, rfc, wmc, health_grade }] }` (3,432 entities)
**pt02 response**: Identical. CBO/LCOM/RFC/WMC are computed from edge adjacency — pure graph metrics.

**RAM**: ~500 MB (iterates all entities, computing per-entity graph metrics). **Total: ~980 MB. Fits 8GB.**

---

#### 19. `/leiden-community-detection-clusters` — Rating: **10/10 BUILD**

**pt01 response**: `{ community_count: 954, modularity: X, communities: [{ id, size, members }] }`
**pt02 response**: Identical. Leiden operates on edge graph only.

**RAM**: ~400 MB. 10 iterations of graph mutations. **Total: ~900 MB. Fits 8GB.**

---

#### 20. `/technical-debt-sqale-scoring` — Rating: **10/10 BUILD**

**pt01 response**: `{ entities: [{ entity, total_debt_hours, violations: [{ type, metric, value, threshold }] }] }`
**pt02 response**: Identical. SQALE scores are computed from CK graph metrics (CBO, LCOM, RFC, WMC). All graph-based.

**RAM**: ~500 MB. **Total: ~980 MB. Fits 8GB.**

---

#### 21. `/folder-structure-discovery-tree` — Rating: **9/10 BUILD**

**pt01 response**: `{ folders: [{ l1: "crates", l2_children: ["parseltongue-core", ...], entity_count: 357 }] }`
**pt02 response**: Needs `root_subfolder_L1` and `root_subfolder_L2` in slim schema. These are derivable from `file_path`.

**Issue**: pt01 stores L1/L2 as separate columns in CodeGraph. pt02 would need to extract them from `file_path` at query time, OR store them as extra fields in SlimEntity.

**Decision**: Add `root_subfolder_L1` and `root_subfolder_L2` to SlimEntity (+~30 bytes per entity = +12 MB at 400K scale). This preserves the `?scope=` filtering capability.

**RAM**: Loads all entities to group by L1/L2. Slim = 58 MB. **Total: ~534 MB. Fits 8GB.**

---

#### 22. `/smart-context-token-budget?focus=X&tokens=N` — Rating: **2/10 DROP**

**pt01 response**: `{ focus_entity, token_budget, tokens_used, context: [{ entity_key, code_snippet, token_count }] }`
**pt02 response**: Would return `{ entity_key, file_path, line_range }` — no code, no tokens to count.

**Why DROP**: The entire point of this endpoint is to assemble code snippets within a token budget. Without code bodies, there are no tokens to count. The LLM already knows how to read files — it doesn't need Parseltongue to assemble context for it.

**Alternative**: Return the dependency subgraph as a list of addresses. But that's just `/forward-callees-query-graph` + `/reverse-callers-query-graph` composed. No unique value.

---

#### 23. `/ingestion-diagnostics-coverage-report` — Rating: **1/10 DROP**

**pt01 response**: `{ test_entities_excluded: { total_count, entities }, word_count_coverage: { avg_pct, files: [...] }, ignored_files: { total_count, files } }`
**pt02 response**: None of this data exists in slim model.

**Why DROP**: This endpoint reports on:
- `TestEntitiesExcluded` (TDD classification — dropped in slim)
- `FileWordCoverage` (import word counts per file — dropped)
- `IgnoredFiles` (files that weren't parsed — dropped)

All three relations are diagnostic metadata from the full ingestion pipeline. pt02's slim model doesn't compute any of this. Building it would mean duplicating pt01's diagnostic pipeline — defeating the purpose of being "slim."

---

#### 24. `/ingestion-coverage-folder-report?depth=N` — Rating: **3/10 DROP**

**pt01 response**: `{ folders: [{ folder_path, total_files, eligible_files, parsed_files, coverage_pct }] }`
**pt02 response**: Would need filesystem walk data and comparison with parsed files.

**Why DROP**: This endpoint walks the actual filesystem, counts eligible vs parsed files, and reports coverage percentage. It needs:
1. Access to the original codebase directory (filesystem walk)
2. The set of all parsed file paths

In pt02's model, the HTTP server opens a `.ptgraph` file — it may not have access to the original codebase directory. The coverage comparison requires both sides (disk files vs parsed files), which is an ingestion-time concern, not a serving-time concern.

**If needed**: Could be added later as a separate CLI command, not an HTTP endpoint.

---

## Features I Was Missing

### The `?scope=` Filter System

pt01's v1.6.5 added scope filtering across all query endpoints:
```
?scope=crates                          → L1 filter (357 entities)
?scope=crates||parseltongue-core       → L1+L2 filter (79 entities)
```

This uses `root_subfolder_L1` and `root_subfolder_L2` columns in CodeGraph. At Rust compiler / Linux kernel scale, scope filtering is **critical** — you don't want to run PageRank on the entire 1.6M edge graph, you want to scope it to `src/librustc_hir` or `drivers/gpu`.

**Decision**: SlimEntity MUST include `root_subfolder_L1` and `root_subfolder_L2`. These are derivable from `file_path` but should be pre-computed for CozoScript filter efficiency. Adds ~30 bytes per entity = negligible.

### Updated Slim Schema (with scope support)

```rust
struct SlimEntity {
    isgl1_key: String,          // 50 B
    file_path: String,          // 40 B
    line_range: String,         // 10 B  (e.g., "17-40")
    entity_type: String,        // 10 B
    language: String,           // 6 B
    root_subfolder_L1: String,  // 15 B  (e.g., "crates", "src", "drivers")
    root_subfolder_L2: String,  // 20 B  (e.g., "parseltongue-core", "librustc_hir")
}
// ~151 bytes per entity (was 114, still 20x smaller than full 3,000)
```

### At 1.6M Scale with L1/L2

| Component | Size |
|-----------|:-:|
| 400K entities × 151 B | 60 MB |
| 1.6M edges × 120 B | 192 MB |
| CozoDB overhead | 252 MB |
| **Total base** | **~504 MB** |

Still fits comfortably on 8GB.

### Scoped Graph Algorithm RAM

With `?scope=crates||parseltongue-core`, you filter from 400K → ~20K entities and ~100K → ~5K edges. Graph algorithms on 5K edges = ~5 MB peak. Even HEAVY endpoints become trivial when scoped.

---

## Final Verdict: 21 Endpoints

| # | Endpoint | Rating | Build? | Slim RAM @ 1.6M |
|---|----------|:------:|:------:|:---------------:|
| 1 | `/server-health-check-status` | 10 | **BUILD** | <1 MB |
| 2 | `/codebase-statistics-overview-summary` | 10 | **BUILD** | <1 MB |
| 3 | `/api-reference-documentation-help` | 10 | **BUILD** | <1 MB |
| 4 | `/code-entity-detail-view?key=X` | 8 | **BUILD** | <1 MB |
| 5 | `/reverse-callers-query-graph?entity=X` | 10 | **BUILD** | <10 MB |
| 6 | `/forward-callees-query-graph?entity=X` | 10 | **BUILD** | <10 MB |
| 7 | `/dependency-edges-list-all?limit=N` | 10 | **BUILD** | <2 MB |
| 8 | `/code-entities-list-all` | 7 | **BUILD** | 60 MB |
| 9 | `/code-entities-search-fuzzy?q=X` | 6 | **BUILD** (key search only) | 60 MB |
| 10 | `/blast-radius-impact-analysis?entity=X` | 10 | **BUILD** | 200-500 MB |
| 11 | `/complexity-hotspots-ranking-view?top=N` | 10 | **BUILD** | 300 MB |
| 12 | `/circular-dependency-detection-scan` | 10 | **BUILD** | 300 MB |
| 13 | `/semantic-cluster-grouping-list` | 9 | **BUILD** | 400 MB |
| 14 | `/strongly-connected-components-analysis` | 10 | **BUILD** | 400 MB |
| 15 | `/kcore-decomposition-layering-analysis` | 10 | **BUILD** | 400 MB |
| 16 | `/centrality-measures-entity-ranking` | 10 | **BUILD** | 400 MB |
| 17 | `/entropy-complexity-measurement-scores` | 10 | **BUILD** | 400 MB |
| 18 | `/coupling-cohesion-metrics-suite` | 10 | **BUILD** | 500 MB |
| 19 | `/leiden-community-detection-clusters` | 10 | **BUILD** | 400 MB |
| 20 | `/technical-debt-sqale-scoring` | 10 | **BUILD** | 500 MB |
| 21 | `/folder-structure-discovery-tree` | 9 | **BUILD** | 60 MB |
| 22 | `/smart-context-token-budget` | 2 | **DROP** | N/A |
| 23 | `/ingestion-diagnostics-coverage-report` | 1 | **DROP** | N/A |
| 24 | `/ingestion-coverage-folder-report` | 3 | **DROP** | N/A |

### Score: 21 BUILD / 3 DROP

---

## The 3 Drops Explained

| Dropped Endpoint | Why | What Replaces It |
|-----------------|-----|-----------------|
| `/smart-context-token-budget` | Returns code snippets within token budget. No code = no tokens to count. | LLM uses `/forward-callees` + `/reverse-callers` to get the graph, then reads files directly using addresses. |
| `/ingestion-diagnostics-coverage-report` | Needs 3 diagnostic relations (test entities, word coverage, ignored files) that slim model doesn't compute. | Run pt01 on Mac/Linux for full diagnostics. |
| `/ingestion-coverage-folder-report` | Needs filesystem walk comparison. pt08 serving from `.ptgraph` may not have access to original directory. | Run pt01 on Mac/Linux. Or add as pt02 CLI output (not HTTP endpoint). |

---

## Peak RAM Summary (All 21 Endpoints on 8GB)

**Base cost**: ~504 MB
**OS + other**: ~4 GB
**Free for queries**: ~3.5 GB
**Heaviest single endpoint**: CK metrics suite / SQALE = ~500 MB above base

**Total peak**: 504 + 500 = **~1 GB**. Leaves 2.5 GB headroom on 8GB machine.

**With scope filtering** (`?scope=crates||parseltongue-core`): Peak drops to ~550 MB total. Even heavy graph algorithms use <50 MB when scoped.

---

## What The Slim Model Means For Users

### Mac/Linux User (unchanged)
```bash
parseltongue pt01-folder-to-cozodb-streamer .           # Full ingestion
parseltongue pt08 --db "rocksdb:.../analysis.db"        # All 24 endpoints
```

### Windows User (new)
```bash
parseltongue pt02-folder-to-ram-snapshot .              # Slim ingestion → .ptgraph (200 MB)
parseltongue pt08 --db "ptgraph:.../analysis.ptgraph"   # 21 endpoints, <1 GB RAM
```

### JSON Export (any OS)
```bash
parseltongue pt03-folder-to-json-exporter .             # → .json (600 MB, human-readable)
parseltongue pt08 --db "json:.../analysis.json"         # 21 endpoints
cat analysis.json | jq '.entities | length'             # Inspect with standard tools
```

### LLM Workflow
```
1. LLM: "What calls the login function?"
   → GET /reverse-callers-query-graph?entity=rust:fn:login
   → Response: [{from_key: "rust:fn:handle_auth", ...}]

2. LLM: "Show me handle_auth"
   → GET /code-entity-detail-view?key=rust:fn:handle_auth
   → Response: {file_path: "src/auth.rs", line_range: "45-92"}

3. LLM reads src/auth.rs:45-92 directly (fresh code, not stale snapshot)
```

---

## Implementation Estimate

| Component | Lines | What |
|-----------|:-----:|------|
| `SlimEntity` + `SlimEdge` types | ~30 | New types in parseltongue-core/entities.rs |
| pt02 crate (ingestion → .ptgraph) | ~150 | Parse with tree-sitter → collect slim entities/edges → MessagePack serialize |
| pt03 crate (ingestion → .json) | ~80 | Same as pt02 but serde_json |
| pt08 ptgraph/json loader | ~120 | Deserialize → populate CozoDB mem with slim schema |
| pt08 handler guards (3 drops) | ~15 | Return error for dropped endpoints when backend is ptgraph/json |
| main.rs subcommands | ~80 | Add pt02/pt03 CLI commands |
| Slim CozoDB schema | ~20 | New slim CodeGraph relation definition |
| **Total** | **~495** | |

---

## Key Insight

**Parseltongue's unique value is the dependency GRAPH, not the code STORAGE.**

Code storage is what `cat`, `bat`, `less`, IDEs, and LLM file-reading tools already do perfectly. Parseltongue's value is answering: "what depends on what?" "what's the blast radius?" "where are the cycles?" "what are the community structures?"

All of those are pure graph operations. The code body was always ballast.
