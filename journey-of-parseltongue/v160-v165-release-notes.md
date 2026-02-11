# Release Notes: v1.6.0 through v1.6.5

From v1.5.6 to v1.6.5, Parseltongue gained 7 graph analysis algorithms, folder-scoped queries, ingestion diagnostics, and a 2.92x parallelism speedup.

## Summary

| Metric | v1.5.6 | v1.6.5 |
|--------|--------|--------|
| HTTP endpoints | 15 | 26 (+11) |
| Tests | ~200 | 670 |
| Languages | 12 | 12 |
| CozoDB relations | 2 (CodeGraph, DependencyEdges) | 5 (+TestEntitiesExcluded, FileWordCoverage, IgnoredFiles) |
| Ingestion speed (302 files) | 5.4s | 1.8s (2.92x faster) |
| CPU utilization | 98% (single-threaded parser) | 364% (thread-local parsers) |
| Graph analysis algorithms | 0 | 7 |

---

## v1.6.0 — 7 Mathematical Graph Analysis Algorithms

### New Endpoints (7)

| Endpoint | Algorithm | What It Answers |
|----------|-----------|-----------------|
| `/strongly-connected-components-analysis` | Tarjan SCC | Which entities form dependency cycles? |
| `/technical-debt-sqale-scoring` | SQALE (ISO 25010) | Where is the highest maintainability debt? |
| `/kcore-decomposition-layering-analysis` | K-Core | Which entities are core vs peripheral? |
| `/centrality-measures-entity-ranking?method=pagerank` | PageRank / Betweenness | Which entities are most important? |
| `/entropy-complexity-measurement-scores` | Shannon Entropy | Which entities have the most information complexity? |
| `/coupling-cohesion-metrics-suite` | CK Metrics (CBO, LCOM, RFC, WMC) | Where is coupling high and cohesion low? |
| `/leiden-community-detection-clusters` | Leiden | What are the natural module boundaries? |

### Architecture

All 7 algorithms live in `parseltongue-core/src/graph_analysis/` with a shared adjacency list representation (`AdjacencyListGraph`). Each algorithm module is independently testable. HTTP handlers in `pt08` call into the core algorithms.

### Key Files Added

```
crates/parseltongue-core/src/graph_analysis/
  mod.rs                                    — Public API + AdjacencyListGraph
  adjacency_list_graph_representation.rs    — O(1) lookup directed graph
  tarjan_scc_algorithm.rs                   — Tarjan's SCC with iterative stack
  kcore_decomposition_algorithm.rs          — Iterative k-core peeling
  centrality_measures_algorithm.rs          — PageRank + Betweenness
  entropy_complexity_algorithm.rs           — Shannon entropy H(X)
  ck_metrics_suite_algorithm.rs             — CBO, LCOM, RFC, WMC
  sqale_debt_scoring_algorithm.rs           — ISO 25010 SQALE model
  leiden_community_algorithm.rs             — Leiden community detection
  integration_cross_algorithm_tests.rs      — Cross-algorithm consistency tests
  test_fixture_reference_graphs.rs          — Shared test graphs
```

---

## v1.6.5 — Folder-Scoped Queries + Diagnostics + Parallelism

### Feature 1: Folder-Scoped Queries

All 18 query endpoints accept `?scope=L1||L2` to restrict results to a specific folder.

```bash
# Discover available folders
curl http://localhost:7777/folder-structure-discovery-tree

# Scope any query
curl "http://localhost:7777/code-entities-list-all?scope=crates||parseltongue-core"
curl "http://localhost:7777/blast-radius-impact-analysis?entity=KEY&hops=2&scope=crates||parseltongue-core"
```

**Implementation**: `scope_filter_utilities_module.rs` provides `parse_scope_build_filter_clause()` which injects CozoDB Datalog filter predicates into every query. Invalid scopes return did-you-mean suggestions.

### Feature 2: Ingestion Diagnostics

New endpoint `/ingestion-diagnostics-coverage-report` with 4 sections accessible via `?section=`:

| Section | Content |
|---------|---------|
| `summary` | Aggregate counts (total entities, test exclusions, coverage stats) |
| `word_coverage` | Per-file `raw_coverage_pct` and `effective_coverage_pct` |
| `test_entities` | All test functions that were excluded from the graph |
| `ignored_files` | Files skipped during ingestion (non-parseable extensions) |

**Dual coverage metrics**: `raw_coverage_pct` = entity words / source words. `effective_coverage_pct` = entity words / (source words - imports - comments). A file at 72% raw / 96% effective is healthy.

### Feature 3: New CozoDB Relations

| Relation | Purpose | Columns |
|----------|---------|---------|
| `TestEntitiesExcluded` | Test functions excluded from code graph | folder_path, filename, language, entity_name, entity_type, line_start, line_end |
| `FileWordCoverage` | Per-file word coverage metrics | folder_path, filename, language, source_word_count, entity_word_count, import_word_count, raw_coverage_pct, effective_coverage_pct |
| `IgnoredFiles` | Non-parseable files encountered | folder_path, filename, extension, file_size_bytes, reason |

### Feature 4: Folder Structure Discovery

New endpoint `/folder-structure-discovery-tree` returns L1/L2 folder hierarchy with entity counts per folder.

### Feature 5: Thread-Local Parser Parallelism (Phase 5)

Replaced `Arc<Mutex<Parser>>` and `Mutex<QueryBasedExtractor>` with `thread_local!` macros:

```rust
thread_local! {
    static THREAD_PARSERS: RefCell<HashMap<Language, Parser>> = RefCell::new(HashMap::new());
    static THREAD_EXTRACTOR: RefCell<Option<QueryBasedExtractor>> = RefCell::new(None);
}
```

**Before**: Each Rayon worker thread fought over a single mutex-guarded parser. 98% CPU on 10 cores.

**After**: Each thread gets its own parser instance. 364% CPU on 10 cores. Parsing is embarrassingly parallel.

**Benchmark** (Parseltongue self-ingestion, 302 files):

| Phase | Baseline | Phase 5 | Speedup |
|-------|----------|---------|---------|
| Streaming | 5.396s | 1.794s | 2.92x |
| Wall clock | 5.71s | 2.25s | 2.54x |
| CPU utilization | 98% | 364% | 3.7x |

---

## Test Corpus

v1.6.0-v1.6.5 established a comprehensive T-folder test fixture system:

- **94 T-folders** with EXPECTED.txt specifications
- **12 language-specific test files** (`t_rust_edge_tests.rs`, `t_python_tests.rs`, etc.)
- **Graph analysis tests**: 83 tests across 7 algorithms + integration
- **Storage tests**: Batch insert, CRUD, special character escaping
- **HTTP handler tests**: Scope filtering, section parameters, error cases

Total: **670 tests**, all passing.

---

## Migration from v1.5.6

No breaking changes. All v1.5.6 queries work unchanged. New features are additive:

- New endpoints return 404 only if called on older databases
- `?scope=` parameter is optional — omitting it returns full unfiltered results
- Three new CozoDB relations are created automatically during ingestion
- Thread-local parallelism requires no configuration

```bash
# Re-ingest to populate new relations
parseltongue pt01-folder-to-cozodb-streamer ./my-project

# New capabilities immediately available
curl http://localhost:7777/folder-structure-discovery-tree
curl "http://localhost:7777/code-entities-list-all?scope=src"
curl http://localhost:7777/ingestion-diagnostics-coverage-report
curl http://localhost:7777/strongly-connected-components-analysis
curl "http://localhost:7777/centrality-measures-entity-ranking?method=pagerank"
```

---

## Files Changed (from v1.5.6 baseline)

- **337 files changed**: +31,194 lines / -2,270 lines
- **67 source files** in crates/ (+11,743 / -233)
- **134 new public items** (functions, structs, enums, traits)
- **94 new test fixtures** in test-fixtures/
- **15 new documentation files** in docs/
