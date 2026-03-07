# DECISION: v1.7.3 — Which Endpoints Should pt02/pt03 Support?

**Date**: 2026-02-12
**Target Scale**: Rust compiler / Linux kernel class codebases (~400K entities, ~1.6M edges)

---

## The Real Numbers: 1.6M Edges

Previous analysis used 50K entities / 100K edges. The actual target is **100-1000x larger**.

### Per-Unit RAM Cost

| Data Type | Avg Size | Source |
|-----------|:--------:|--------|
| `CodeEntity` (with code body) | ~3 KB | ISGL1 key (~50B) + code body (~1-2KB) + interface sig (~200B) + metadata (~500B) |
| `DependencyEdge` | ~250 bytes | Two ISGL1 keys (~100B) + edge_type (~20B) + source_location (~30B) + overhead |
| CozoDB BTreeMap node overhead | ~1.5-2x raw | Per-row indexing, string interning, internal pointers |

### Base Cost: Just Loading Data Into CozoDB `mem`

| Scale | Entities | Edges | Raw Data | CozoDB Overhead | **Total Base** |
|-------|----------|-------|----------|:--------------:|:--------------:|
| Parseltongue self | 1,600 | 10K | 7 MB | 13 MB | **~20 MB** |
| Medium project | 10,000 | 50K | 43 MB | 37 MB | **~80 MB** |
| Large monorepo | 50,000 | 100K | 175 MB | 175 MB | **~350 MB** |
| **Rust compiler** | **200,000** | **800K** | **800 MB** | **800 MB** | **~1.6 GB** |
| **Linux kernel** | **400,000** | **1.6M** | **1.6 GB** | **1.6 GB** | **~3.2 GB** |
| Extreme (kernel + drivers) | 500,000 | 2M | 2 GB | 2 GB | **~4 GB** |

**At 1.6M edges, just LOADING the data into CozoDB `mem` costs ~3.2 GB.**

---

## What Each Handler Actually Loads (Exact CozoScript Queries)

The explore agent extracted the exact query strings. Here's what actually happens at 1.6M edge scale.

### TRULY SAFE — Targeted Queries Only (7 endpoints)

These endpoints query by key, use `:limit`, or return static data. They never touch full tables.

| # | Endpoint | Exact Query Pattern | Per-Request RAM |
|---|----------|-------------------|:--------------:|
| 1 | `/server-health-check-status` | No DB query (uptime only) | <100 KB |
| 2 | `/codebase-statistics-overview-summary` | `COUNT(*)` aggregate | <100 KB |
| 3 | `/api-reference-documentation-help` | No DB query (static docs) | <500 KB |
| 4 | `/code-entity-detail-view?key=X` | `*CodeGraph{ISGL1_key: key} WHERE key == "X"` | <500 KB |
| 5 | `/reverse-callers-query-graph?entity=X` | `*DependencyEdges{...} WHERE to_key == "X"` | <10 MB |
| 6 | `/forward-callees-query-graph?entity=X` | `*DependencyEdges{...} WHERE from_key == "X"` | <10 MB |
| 7 | `/dependency-edges-list-all?limit=100` | `*DependencyEdges{...} :limit 100` | <2 MB |

**These 7 work at ANY scale, on ANY machine.** They do key lookups or paginated reads.

### LOADS ALL ENTITIES — Full CodeGraph Scan (3 endpoints)

These query `SELECT * FROM CodeGraph` with no `:limit` clause. At 400K entities, that's **1.2 GB loaded into a Vec in Rust**.

| # | Endpoint | Exact Query | Per-Request RAM @ 1.6M edges |
|---|----------|-------------|:----------------------------:|
| 8 | `/code-entities-list-all` | `?[key, file_path, entity_type, ...] := *CodeGraph{...}` (no limit) | **+1.2 GB** |
| 9 | `/code-entities-search-fuzzy?q=X` | `?[key, file_path, ...] := *CodeGraph{...}` then substring filter in Rust | **+1.2 GB** |
| 10 | `/folder-structure-discovery-tree` | `?[...] := *CodeGraph{...}` deduplicate L1/L2 | **+1.2 GB** (but result is small) |

### LOADS ALL EDGES — Full DependencyEdges Scan (6 endpoints)

These query `SELECT * FROM DependencyEdges` with no `:limit`. At 1.6M edges, that's **400 MB loaded into Vec, then built into HashMap adjacency lists = 800 MB-1.6 GB more**.

| # | Endpoint | What It Builds In Memory | Per-Request RAM @ 1.6M edges |
|---|----------|-------------------------|:----------------------------:|
| 11 | `/smart-context-token-budget` | TWO HashMaps (forward + reverse adjacency) from ALL edges | **+1.2 GB** |
| 12 | `/complexity-hotspots-ranking-view` | Two count HashMaps from ALL edges, sort, take top N | **+800 MB** |
| 13 | `/circular-dependency-detection-scan` | Full adjacency HashMap + DFS color map from ALL edges | **+1 GB** |
| 14 | `/semantic-cluster-grouping-list` | Bidirectional graph HashMap + edge Vec + label maps × 10 iterations | **+1.6 GB** |
| 15 | `/blast-radius-impact-analysis` | Per-hop filtered queries (better than full scan, but each hop scans edges) | **+200-500 MB** |
| 16 | `/ingestion-diagnostics-coverage-report` | 3 diagnostic relations (TestEntities + WordCoverage + IgnoredFiles) | **+300-600 MB** |

### LOADS ALL EDGES + GRAPH ALGORITHMS (7 endpoints)

These call `build_graph_from_database_edges()` → full `AdjacencyListGraphRepresentation` → then run O(V+E) to O(V^2) algorithms on top.

| # | Endpoint | Algorithm | Per-Request RAM @ 1.6M edges |
|---|----------|-----------|:----------------------------:|
| 17 | `/strongly-connected-components-analysis` | Tarjan SCC: O(V+E) traversal | **+2 GB** |
| 18 | `/technical-debt-sqale-scoring` | SQALE + CK metrics per entity: O(V*E) | **+3 GB** |
| 19 | `/kcore-decomposition-layering-analysis` | K-core peeling: O(V+E) iterative | **+2 GB** |
| 20 | `/centrality-measures-entity-ranking` | PageRank: O(V+E)*100 iters / Betweenness: O(V*E^2) | **+2-4 GB** |
| 21 | `/entropy-complexity-measurement-scores` | Shannon entropy per entity: O(V+E) | **+2 GB** |
| 22 | `/coupling-cohesion-metrics-suite` | CBO/LCOM/RFC/WMC per entity: O(V^2) | **+3-5 GB** |
| 23 | `/leiden-community-detection-clusters` | Leiden: O(V+E)*10 iters | **+2.5 GB** |
| 24 | `/ingestion-coverage-folder-report` | Filesystem walk + all file_paths from DB | **+500 MB** |

---

## Total RAM Per Request (Base + Query)

**Base cost** (CozoDB mem loaded with 400K entities + 1.6M edges): **~3.2 GB**
**OS + other processes**: **~4 GB**

| Category | Endpoints | Base + Query RAM | 8GB Machine | 16GB Machine | 32GB Machine |
|----------|:---------:|:----------------:|:-----------:|:------------:|:------------:|
| **SAFE** (7) | 1-7 | 3.2 + 0.01 = **3.2 GB** | Yes (7.2 total) | Yes | Yes |
| **ALL-ENTITIES** (3) | 8-10 | 3.2 + 1.2 = **4.4 GB** | **No** (8.4 > 8) | Yes | Yes |
| **ALL-EDGES** (6) | 11-16 | 3.2 + 0.8-1.6 = **4-4.8 GB** | **No** (8-8.8 > 8) | Yes (12.8-13.2) | Yes |
| **GRAPH ALGOS** (7) | 17-23 | 3.2 + 2-5 = **5.2-8.2 GB** | **No** | **Tight** (12.4-13.8) | Yes |
| **Betweenness/CK** (2) | 20, 22 | 3.2 + 4-5 = **7.2-8.2 GB** | **No** | **No** (15.2-16.2 > 16) | Yes |

### Critical Finding

**On an 8GB machine with 1.6M edges**: Only the 7 SAFE endpoints work. Everything else risks OOM.

**On a 16GB machine with 1.6M edges**: SAFE + ALL-ENTITIES + most ALL-EDGES work. Graph algorithms are tight. Betweenness centrality and CK metrics will likely crash.

**BUT**: Even the SAFE endpoints require **3.2 GB base cost** just to have CozoDB `mem` loaded. On an 8GB machine, that leaves only ~800 MB headroom.

---

## The Fundamental Problem

The original plan was: "pt02/pt03 serialize data → pt08 loads into CozoDB `mem` → serves endpoints."

At 1.6M edges, CozoDB `mem` base cost alone is **3.2 GB**. This means:
- 8GB machine: only 7 endpoints safe, and barely
- An 8GB Windows machine (the target audience) has ~4GB free after OS
- 3.2GB for CozoDB + 800MB for the process itself = already at the limit

**The CozoDB `mem` approach does not scale to 1.6M edges on 8GB machines.**

---

## Revised Tier Classification (1.6M Edge Scale)

### Tier 1: SAFE on 8GB (7 endpoints)

Key lookups, aggregates, pagination. No full-table scans.

```
/server-health-check-status          → no DB query
/codebase-statistics-overview-summary → COUNT aggregate
/api-reference-documentation-help    → static
/code-entity-detail-view?key=X       → single key lookup
/reverse-callers-query-graph?entity=X → filtered edge lookup
/forward-callees-query-graph?entity=X → filtered edge lookup
/dependency-edges-list-all?limit=N    → paginated
```

### Tier 2: Needs 16GB (10 endpoints)

Load all entities OR all edges into memory. One full-table scan per request.

```
/code-entities-list-all              → loads ALL entities (1.2 GB)
/code-entities-search-fuzzy?q=X     → loads ALL entities (1.2 GB)
/folder-structure-discovery-tree     → loads ALL entities (1.2 GB, small result)
/smart-context-token-budget          → loads ALL edges (1.2 GB in two HashMaps)
/complexity-hotspots-ranking-view    → loads ALL edges (800 MB)
/blast-radius-impact-analysis        → per-hop edge scans (200-500 MB)
/circular-dependency-detection-scan  → loads ALL edges + DFS (1 GB)
/semantic-cluster-grouping-list      → loads ALL edges + label prop (1.6 GB)
/ingestion-diagnostics-coverage-report → 3 diagnostic tables (300-600 MB)
/ingestion-coverage-folder-report    → filesystem walk + all paths (500 MB)
```

### Tier 3: Needs 32GB (7 endpoints)

Full graph algorithm: loads all edges into `AdjacencyListGraphRepresentation` + runs O(V*E) or worse.

```
/strongly-connected-components-analysis  → +2 GB
/technical-debt-sqale-scoring            → +3 GB
/kcore-decomposition-layering-analysis   → +2 GB
/centrality-measures-entity-ranking      → +2-4 GB (betweenness: +4 GB)
/entropy-complexity-measurement-scores   → +2 GB
/coupling-cohesion-metrics-suite         → +3-5 GB (HIGHEST)
/leiden-community-detection-clusters     → +2.5 GB
```

---

## Endpoint Coverage Summary (1.6M Edge Scale)

| Endpoint | pt01 (rocksdb) 8GB | pt02/pt03 8GB | pt02/pt03 16GB | pt02/pt03 32GB |
|----------|:------------------:|:-------------:|:--------------:|:--------------:|
| `/server-health-check-status` | Yes | **Yes** | **Yes** | **Yes** |
| `/codebase-statistics-overview-summary` | Yes | **Yes** | **Yes** | **Yes** |
| `/api-reference-documentation-help` | Yes | **Yes** | **Yes** | **Yes** |
| `/code-entity-detail-view` | Yes | **Yes** | **Yes** | **Yes** |
| `/reverse-callers-query-graph` | Yes | **Yes** | **Yes** | **Yes** |
| `/forward-callees-query-graph` | Yes | **Yes** | **Yes** | **Yes** |
| `/dependency-edges-list-all` | Yes | **Yes** | **Yes** | **Yes** |
| `/code-entities-list-all` | Yes | No | **Yes** | **Yes** |
| `/code-entities-search-fuzzy` | Yes | No | **Yes** | **Yes** |
| `/folder-structure-discovery-tree` | Yes | No | **Yes** | **Yes** |
| `/smart-context-token-budget` | Yes | No | **Yes** | **Yes** |
| `/complexity-hotspots-ranking-view` | Yes | No | **Yes** | **Yes** |
| `/blast-radius-impact-analysis` | Yes | No | **Yes** | **Yes** |
| `/circular-dependency-detection-scan` | Yes | No | **Yes** | **Yes** |
| `/semantic-cluster-grouping-list` | Yes | No | Tight | **Yes** |
| `/ingestion-diagnostics-coverage-report` | Yes | No | **Yes** | **Yes** |
| `/ingestion-coverage-folder-report` | Yes | No | **Yes** | **Yes** |
| `/strongly-connected-components-analysis` | Yes | No | Tight | **Yes** |
| `/technical-debt-sqale-scoring` | Yes | No | No | **Yes** |
| `/kcore-decomposition-layering-analysis` | Yes | No | Tight | **Yes** |
| `/centrality-measures-entity-ranking` | Yes | No | No (betweenness) | **Yes** |
| `/entropy-complexity-measurement-scores` | Yes | No | Tight | **Yes** |
| `/coupling-cohesion-metrics-suite` | Yes | No | No | **Yes** |
| `/leiden-community-detection-clusters` | Yes | No | Tight | **Yes** |

### Totals

| Backend | 8GB Machine | 16GB Machine | 32GB Machine |
|---------|:-----------:|:------------:|:------------:|
| **pt01 (rocksdb)** | **24/24** | **24/24** | **24/24** |
| **pt02/pt03 (mem)** | **7/24** | **~17/24** | **24/24** |

**RocksDB wins at every RAM tier** because it keeps data on disk and only loads query results. The `mem` backend pays the full 3.2 GB upfront.

---

## Why pt01 Works on 8GB But pt02/pt03 Don't

| Factor | pt01 (RocksDB) | pt02/pt03 (CozoDB mem) |
|--------|:-------------:|:---------------------:|
| Data location | Disk (mmap'd, OS pages on demand) | All in RAM |
| Base RAM cost at 1.6M edges | **~200 MB** (process + hot pages) | **~3.2 GB** (everything loaded) |
| Single entity lookup | RocksDB seeks on disk, returns 1 row | Fast (BTreeMap), but base cost already paid |
| Full table scan | Streams from disk, ~1.2 GB peak | Already in RAM, but result Vec adds +1.2 GB |
| Graph algorithm | Loads edges from disk → +2 GB peak, then releases | Edges already in RAM (3.2 GB) + algorithm adds +2 GB = 5.2 GB |

**RocksDB's advantage**: It only loads what each query needs. Most pages stay on disk. The OS's page cache handles frequently-accessed data. Total resident memory stays manageable.

**CozoDB mem's problem**: ALL data is resident ALL the time. Even if you only need 1 entity, you're paying for 400K entities and 1.6M edges sitting in BTreeMaps.

---

## Ingestion RAM (pt02/pt03 during parsing)

Separate from serving, the ingestion itself also needs RAM. The streamer collects all entities and edges in Vecs before serializing.

| Phase | RAM Usage @ 1.6M edges |
|-------|:---------------------:|
| Tree-sitter parsing (per-file) | ~50 MB (parser + AST) |
| Accumulated entities Vec | ~1.2 GB (400K × 3KB) |
| Accumulated edges Vec | ~400 MB (1.6M × 250B) |
| MessagePack serialization buffer | ~500 MB |
| **Peak during ingestion** | **~2.1 GB** |

This is fine on 16GB machines. Tight on 8GB but feasible since there's no CozoDB overhead during ingestion.

### Serialized File Sizes (Estimated)

| Format | Estimated Size @ 1.6M edges |
|--------|:---------------------------:|
| MessagePack (`.ptgraph`) | **~800 MB - 1.2 GB** |
| JSON (`.json`) | **~3-5 GB** |
| Bincode | ~700 MB - 1 GB |

---

## Architectural Options Going Forward

### Option A: CozoDB `mem` Backend (Current Plan)
- pt08 deserializes `.ptgraph` → populates CozoDB `mem` → serves endpoints
- **Pro**: Zero handler changes. All handlers still talk to CozoDbStorage.
- **Con**: 3.2 GB base cost. Only 7 endpoints safe on 8GB.

### Option B: Direct Vec/HashMap Serving (No CozoDB)
- pt08 deserializes `.ptgraph` → keeps as `HashMap<Key, Entity>` + indexed edges → custom query logic
- **Pro**: No CozoDB overhead. Base cost ~1.6 GB (raw data only). More endpoints safe on 8GB.
- **Con**: Must rewrite all 24 handlers to query from HashMap instead of CozoDB. ~2000+ lines of new code.

### Option C: CozoDB SQLite Backend for Serving (Disk-Backed)
- pt08 loads `.ptgraph` → inserts into CozoDB `sqlite:` file → serves from disk
- **Pro**: Disk-backed like RocksDB. Low base RAM. All 24 endpoints work.
- **Con**: Slow initial load (insert 1.6M rows into SQLite). SQLite single-writer for the insert phase. On Windows, SQLite writes failed before (though this is a one-time bulk insert, different from pt01's streaming inserts).

### Option D: Hybrid — Tier 1 from Vec, Tier 2+ Returns Error
- pt08 loads `.ptgraph` → deserializes into indexed Vecs → serves only Tier 1 endpoints
- **Pro**: Lowest RAM. 7 endpoints with ~1.6 GB base. Simple.
- **Con**: Only 7 endpoints. Graph algorithms and full-table scans not available.

### Recommendation

**For v1.7.3**: Start with **Option A** (CozoDB mem). Document the RAM requirements. On 8GB machines, 7 endpoints work. On 16GB, 17 work. This is honest and ships fast.

**For v1.8.0** (if needed): Consider **Option C** (load ptgraph into SQLite on disk). This would give all 24 endpoints on any machine, at the cost of a slower startup (inserting 1.6M rows into SQLite takes ~30-60 seconds).

---

## Decision: What pt02/pt03 Should Ship With

### v1.7.3 Scope

1. **pt02**: Parses codebase → serializes to `.ptgraph` (MessagePack). RAM during ingestion: ~2.1 GB for 1.6M edges.
2. **pt03**: Parses codebase → serializes to `.json`. RAM during ingestion: same.
3. **pt08**: Loads `.ptgraph` or `.json` into CozoDB `mem`. Serves:
   - **7 endpoints guaranteed** (Tier 1: targeted queries)
   - **10 endpoints on 16GB+** (Tier 2: full-table scans)
   - **7 endpoints on 32GB+** (Tier 3: graph algorithms)
   - Returns clear error with RAM requirement when endpoint exceeds available memory

### Error Response for Blocked Endpoints

```json
{
  "success": false,
  "error": "This endpoint loads all 1.6M edges into memory for graph analysis. Estimated RAM: 5.2 GB. Available: 3.8 GB.",
  "suggestion": "Use pt01 with RocksDB backend on Mac/Linux, or run on a machine with 32GB+ RAM.",
  "tier": 3,
  "estimated_ram_gb": 5.2
}
```
