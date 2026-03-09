# THESIS: Parseltongue Ingestion Speed Optimization

**Version**: v1.5.0 Proposal
**Date**: 2026-02-06
**Analysis Method**: Self-referential parseltongue API exploration (no grep/glob)
**Problem**: 1-2GB codebases take 20-30 minutes to ingest

---

## Executive Summary

Using parseltongue's own HTTP API to analyze its codebase, I identified **7 major bottlenecks** in the ingestion pipeline. The current architecture is **fundamentally sequential** with **N database round-trips for N entities**. With the proposed optimizations, we can achieve **10-50x speedup**.

---

## Discovery Method

All findings derived exclusively from parseltongue endpoints:
- `/code-entities-search-fuzzy` - Entity discovery
- `/forward-callees-query-graph` - Call chain analysis
- `/reverse-callers-query-graph` - Caller identification
- `/blast-radius-impact-analysis` - Impact mapping
- `/complexity-hotspots-ranking-view` - Bottleneck identification

---

## Architecture Analysis

### Current Pipeline Flow

```
stream_directory (streamer.rs)
    │
    ├─► WalkDir.into_iter()     [SEQUENTIAL - Bottleneck #1]
    │       │
    │       ▼
    │   for each file:
    │       │
    │       ├─► read_file_content()      [SYNC I/O - Bottleneck #2]
    │       │
    │       ├─► parse_source()           [Sequential parsing - Bottleneck #3]
    │       │       │
    │       │       ├─► tree-sitter parse()
    │       │       ├─► execute_query()
    │       │       └─► execute_dependency_query()
    │       │
    │       ├─► for each entity:         [N iterations - Bottleneck #4]
    │       │       │
    │       │       ├─► generate_key()
    │       │       ├─► fetch_lsp_metadata_for_entity()  [LSP RPC - Bottleneck #5]
    │       │       ├─► parsed_entity_to_code_entity()
    │       │       └─► insert_entity()   [DB round-trip - Bottleneck #6]
    │       │               │
    │       │               └─► run_script()  [CozoDB execution]
    │       │
    │       └─► insert_edges_batch()     [BATCHED - Good pattern!]
    │               │
    │               └─► run_script()      [Single CozoDB call]
    │
    └─► get_stats()
```

### Complexity Hotspots (from `/complexity-hotspots-ranking-view`)

| Rank | File | Outbound Calls | Role |
|------|------|----------------|------|
| 1 | streamer.rs | 130 | Main ingestion loop |
| 2 | entities.rs | 94 | Entity definitions |
| 3 | incremental_reindex_core_logic.rs | 82 | Reindexing |
| 4 | cozo_client.rs | 74 | Database operations |
| 5 | query_extractor.rs | 70 | Tree-sitter parsing |

---

## The 7 Bottlenecks

### Bottleneck #1: Sequential File Processing

**Evidence from API:**
```
rust:method:stream_directory calls:
  - rust:fn:into_iter (WalkDir iteration)
  - rust:fn:stream_file (per-file, in sequence)
```

**Problem**: Files are processed one at a time. A 2GB codebase with 10,000 files processes them serially.

**Impact**:
- With 100ms average per file: 10,000 files × 100ms = **16.7 minutes**
- CPU utilization: ~12% (1 core of 8)

---

### Bottleneck #2: Synchronous File I/O

**Evidence from API:**
```
rust:fn:read_file_content (unresolved-reference)
tokio::fs NOT used in pt01 (only in pt08 HTTP server)
```

**Problem**: File reads block the thread. No async I/O for ingestion.

**Impact**: On spinning disks or network mounts, I/O wait dominates.

---

### Bottleneck #3: Sequential Tree-Sitter Parsing

**Evidence from API:**
```
rust:method:parse_source:____crates_parseltongue_core_src_query_extractor
  calls:
    - rust:fn:parse (tree-sitter)
    - rust:fn:execute_query
    - rust:fn:execute_dependency_query
```

**Problem**: Parser is single-threaded. No parallel parsing across files.

**Impact**: Parsing 1MB files takes ~50-200ms. For 1,000 large files: ~50-200 seconds.

---

### Bottleneck #4: Per-Entity Database Insertion (THE KILLER)

**Evidence from API:**
```
rust:method:stream_file calls:
  - rust:fn:insert_entity (line 624, INSIDE THE LOOP)

rust:method:insert_entity calls:
  - rust:fn:run_script (DB execution per entity!)

BUT:
  - rust:fn:insert_entities_batch DOES NOT EXIST!
  - rust:method:insert_edges_batch DOES EXIST (line 655)
```

**Problem**: Each entity = 1 database transaction. For a file with 50 entities, that's 50 DB round-trips.

**Impact**:
- Typical codebase: 50,000 entities
- CozoDB round-trip: ~2-5ms
- Total: 50,000 × 3ms = **150 seconds just for inserts**

**The pattern exists!** `insert_edges_batch` shows the solution is already implemented for edges, just not entities.

---

### Bottleneck #5: LSP Metadata Fetching Per Entity

**Evidence from API:**
```
rust:method:stream_file calls:
  - rust:fn:fetch_lsp_metadata_for_entity (line 604)

rust:mod:lsp_client exists in pt01
```

**Problem**: LSP calls require language server communication (JSON-RPC over stdio/socket). This is called for EVERY entity.

**Impact**:
- LSP hover request: ~10-50ms
- 50,000 entities × 25ms = **20+ minutes**
- Often the dominant bottleneck!

---

### Bottleneck #6: No Parallel Crate Usage

**Evidence from API:**
```
External dependencies found:
  - walkdir (sequential iteration)
  - tokio (only for HTTP, not ingestion)

NOT FOUND:
  - rayon (parallel iterators)
  - crossbeam (parallel data structures)
```

**Problem**: The codebase doesn't use any parallelization libraries for ingestion.

---

### Bottleneck #7: Parser Instance Management

**Evidence from API:**
```
rust:struct:QueryBasedExtractor - single instance
rust:method:parse_source - creates tree-sitter parser
```

**Problem**: Parser instantiation overhead may not be amortized. Tree-sitter parsers are reusable but may be recreated.

---

## Proposed Optimizations

### Phase 1: Batch Entity Insertion (10x speedup)
**Effort**: Low | **Impact**: Very High

```rust
// Current (N DB calls):
for entity in entities {
    storage.insert_entity(&entity)?;  // run_script each time
}

// Proposed (1 DB call):
storage.insert_entities_batch(&entities)?;  // Single run_script
```

**Implementation**: Copy `insert_edges_batch` pattern to create `insert_entities_batch`.

**Expected Result**: 50,000 entities in 1 DB call vs 50,000 calls = **~100x faster DB operations**

---

### Phase 2: Parallel File Processing (4-8x speedup)
**Effort**: Medium | **Impact**: High

```rust
// Add to Cargo.toml
rayon = "1.8"

// Current:
for entry in WalkDir::new(dir).into_iter() {
    self.stream_file(&entry.path())?;
}

// Proposed:
WalkDir::new(dir)
    .into_iter()
    .par_bridge()  // Convert to parallel iterator
    .for_each(|entry| {
        self.stream_file(&entry.path());
    });
```

**Caveat**: Requires thread-safe storage or per-thread batching.

---

### Phase 3: LSP Batching or Deferral (5x speedup)
**Effort**: Medium | **Impact**: High

**Option A: Batch LSP requests**
```rust
// Collect all positions, make single LSP request per file
let positions: Vec<Position> = entities.iter().map(|e| e.location).collect();
let metadata = lsp_client.batch_hover(file, positions)?;
```

**Option B: Defer LSP to background**
```rust
// Ingest entities immediately, fetch LSP metadata asynchronously
storage.insert_entity(&entity)?;
background_queue.push(LspTask { entity_key, file, position });
```

**Option C: Make LSP optional for initial ingestion**
```bash
parseltongue pt01 --skip-lsp ./codebase  # Fast initial scan
parseltongue pt01 --enrich-lsp ./codebase  # Background enrichment
```

---

### Phase 4: Async I/O for File Reading (2x speedup on slow storage)
**Effort**: Low | **Impact**: Medium

```rust
// Current:
let content = std::fs::read_to_string(&path)?;

// Proposed:
let content = tokio::fs::read_to_string(&path).await?;
```

Combined with tokio's work-stealing runtime, this overlaps I/O with parsing.

---

### Phase 5: Two-Phase Ingestion Architecture (Best overall)
**Effort**: High | **Impact**: Transformative

```
Phase 1: SCAN (parallel, no DB writes)
├── Walk directory (rayon parallel)
├── Read files (async I/O)
├── Parse with tree-sitter (parallel)
└── Collect: Vec<(file, entities, edges)>

Phase 2: COMMIT (sequential, batched)
├── insert_entities_batch(all_entities)  // 1 DB call
├── insert_edges_batch(all_edges)         // 1 DB call
└── commit_file_hashes()                   // 1 DB call
```

**Benefits**:
- Phase 1 is embarrassingly parallel
- Phase 2 is 3 DB transactions total
- Memory bounded by chunking (e.g., 10,000 entities per batch)

---

## Expected Performance Gains

| Optimization | Speedup Factor | Cumulative |
|--------------|----------------|------------|
| Batch entity insertion | 10x | 10x |
| Parallel file processing (8 cores) | 6x | 60x |
| LSP deferral/skip | 5x | 300x |
| Async I/O (SSD) | 1.5x | 450x |

**Realistic expectation**: **20-50x improvement** after accounting for overhead.

| Codebase Size | Current Time | After Optimization |
|---------------|--------------|-------------------|
| 100MB | 2 min | 10-20 sec |
| 500MB | 10 min | 30-60 sec |
| 1GB | 20 min | 1-2 min |
| 2GB | 30+ min | 2-4 min |

---

## Implementation Roadmap

### v1.5.0 - Batch Insertions (Quick Win)
- [ ] Implement `insert_entities_batch` (copy edge pattern)
- [ ] Buffer entities per file, batch insert at file end
- [ ] Tests: verify 10x speedup on 10K entity benchmark

### v1.5.1 - Parallel Processing
- [ ] Add rayon dependency
- [ ] Thread-safe entity buffer (crossbeam channel)
- [ ] Parallel WalkDir iteration
- [ ] Tests: verify linear scaling with cores

### v1.5.2 - LSP Optimization
- [ ] Add `--skip-lsp` flag for fast initial scan
- [ ] Background LSP enrichment queue
- [ ] Batch hover requests per file

### v1.5.3 - Two-Phase Architecture
- [ ] Separate scan and commit phases
- [ ] Streaming memory-bounded processing
- [ ] Progress reporting per phase

---

## Verification Plan

### Benchmarks to Create
```rust
#[bench]
fn bench_ingest_1k_files() { ... }

#[bench]
fn bench_ingest_10k_entities_batch_vs_individual() { ... }

#[bench]
fn bench_parallel_vs_sequential_parsing() { ... }
```

### Real-World Test Codebases
1. Linux kernel (1.2GB) - Stress test
2. Kubernetes (500MB) - Mixed languages
3. VS Code (300MB) - TypeScript heavy
4. Parseltongue itself - Dogfooding

---

## Conclusion

The parseltongue ingestion pipeline suffers from a **fundamental architectural bottleneck**: per-entity database insertions. The solution already exists in the codebase (`insert_edges_batch`) and simply needs to be extended to entities.

Combined with parallel file processing using rayon and LSP deferral, we can achieve **20-50x speedup**, bringing 2GB codebase ingestion from 30 minutes down to **1-2 minutes**.

The path forward is clear:
1. **Quick win**: Batch entity insertions (v1.5.0)
2. **Parallel processing**: Add rayon (v1.5.1)
3. **LSP optimization**: Skip or defer (v1.5.2)
4. **Architecture overhaul**: Two-phase design (v1.5.3)

---

*Analysis performed using parseltongue v1.4.5 on its own codebase. 2,304 entities analyzed across 127 files with 6,681 dependency edges.*
