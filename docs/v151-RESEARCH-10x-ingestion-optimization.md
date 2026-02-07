# Research: 10x Ingestion Speed Optimization

## v1.5.1 Performance Analysis | February 2026

---

## Executive Summary

**Current Performance**: 42,000 files in 3,100 seconds = **13.5 files/sec**
**Target Performance**: 42,000 files in 310 seconds = **135 files/sec** (10x improvement)

This research uses Parseltongue's own API to analyze its architecture and identify optimization opportunities.

---

## Part 1: Current Architecture Analysis

### Complexity Hotspots (via `/complexity-hotspots-ranking-view`)

| Rank | File | Outbound Deps | Role |
|------|------|---------------|------|
| 6 | `streamer.rs` | 131 | Main file processing pipeline |
| 10 | `entities.rs` | 94 | Entity type definitions |
| 12 | `incremental_reindex_core_logic.rs` | 82 | Incremental processing |
| 13 | `cozo_client.rs` | 77 | Database operations |
| 17 | `query_extractor.rs` | 70 | Tree-sitter parsing |
| 22 | `isgl1_generator.rs` | 59 | Key generation |

**Insight**: `streamer.rs` is the central hub with 131 dependencies - optimizations here have maximum impact.

### Entity Distribution (via `/code-entities-list-all`)

| Entity Type | Count | % of Total |
|-------------|-------|------------|
| function | 617 | 65.5% |
| method | 119 | 12.6% |
| module | 94 | 10.0% |
| struct | 64 | 6.8% |
| class | 29 | 3.1% |
| impl | 7 | 0.7% |
| trait | 6 | 0.6% |
| enum | 4 | 0.4% |

### Edge Distribution (via `/dependency-edges-list-all`)

| Edge Type | Count | % of Total |
|-----------|-------|------------|
| Uses | 927 | 92.7% |
| Calls | 73 | 7.3% |

**Insight**: "Uses" edges (imports) dominate. Optimizing import processing has highest ROI.

---

## Part 2: Identified Bottlenecks

### Bottleneck #1: Sequential File Processing

```
Current Flow:
for file in files {           // Sequential loop
    read_file()               // Blocking I/O
    parse_with_treesitter()   // CPU-bound
    generate_entities()       // CPU-bound
    generate_edges()          // CPU-bound
    insert_to_db()            // I/O-bound
}
```

**Problem**: Each file waits for the previous to complete.
**Impact**: ~60% of time wasted in I/O waits.

### Bottleneck #2: Per-Entity Database Inserts

```
Current (v1.4.x): N round-trips for N entities
After v1.5.0:     1 round-trip (batch insert)
```

**Status**: ALREADY FIXED in v1.5.0 with `insert_entities_batch()`.
**Measured**: 2,356 entities in 2.2 seconds (self-test).

### Bottleneck #3: Single-Threaded Tree-Sitter Parsing

```
Current: 1 parser instance, processes files serially
```

**Problem**: Modern CPUs have 8-16 cores sitting idle.
**Impact**: ~70% CPU underutilization.

### Bottleneck #4: Synchronous File I/O

```rust
// Current
let content = std::fs::read_to_string(path)?;  // Blocks thread

// Better
let content = tokio::fs::read_to_string(path).await?;  // Async
```

**Problem**: Thread blocked during disk reads.
**Impact**: ~20% time in I/O wait.

### Bottleneck #5: No Incremental Processing

```
Current: Re-parse ALL files every time
Better:  Only parse CHANGED files (via content hash)
```

**Status**: ISGL1 v2 has content hashing, but not used for skip logic.
**Impact**: For re-indexing, could skip 95%+ unchanged files.

### Bottleneck #6: Language Parser Initialization

```rust
// Current: Initialize all 12 language parsers upfront
let parsers = initialize_all_parsers();  // ~100ms overhead

// Better: Lazy initialization per language
let parser = get_or_create_parser(language);
```

**Problem**: Loading parsers for unused languages.
**Impact**: Small (~100ms) but avoidable.

---

## Part 3: Optimization Strategies

### Strategy 1: Parallel File Processing with Rayon

```rust
use rayon::prelude::*;

files.par_iter()  // Parallel iterator
    .map(|file| process_file(file))
    .collect::<Vec<_>>()
```

**Expected Speedup**: 4-8x (depends on CPU cores)
**Complexity**: Low (add rayon dependency, change loop)

### Strategy 2: Async File I/O with Tokio

```rust
use tokio::fs;

let contents: Vec<String> = futures::future::join_all(
    files.iter().map(|f| fs::read_to_string(f))
).await;
```

**Expected Speedup**: 1.5-2x for I/O-heavy workloads
**Complexity**: Medium (async refactor)

### Strategy 3: Work-Stealing Thread Pool

```rust
// Separate concerns:
// - I/O threads: Read files from disk
// - CPU threads: Parse with tree-sitter
// - DB threads: Batch insert to CozoDB

crossbeam::scope(|s| {
    let (file_tx, file_rx) = crossbeam::channel::bounded(100);
    let (entity_tx, entity_rx) = crossbeam::channel::bounded(1000);

    // Producer: File reading
    s.spawn(|_| { read_files_to_channel(file_tx) });

    // Workers: Parsing (N threads)
    for _ in 0..num_cpus::get() {
        s.spawn(|_| { parse_files_from_channel(file_rx, entity_tx) });
    }

    // Consumer: Database insertion
    s.spawn(|_| { batch_insert_from_channel(entity_rx) });
});
```

**Expected Speedup**: 6-10x
**Complexity**: High (significant architecture change)

### Strategy 4: Memory-Mapped File Reading

```rust
use memmap2::MmapOptions;

let file = File::open(path)?;
let mmap = unsafe { MmapOptions::new().map(&file)? };
let content = std::str::from_utf8(&mmap)?;
```

**Expected Speedup**: 1.3-1.5x for large files
**Complexity**: Low

### Strategy 5: Parser Instance Pool

```rust
// Pool of pre-initialized parsers per language
struct ParserPool {
    rust_parsers: Vec<Parser>,    // Pre-warmed
    python_parsers: Vec<Parser>,
    // ... etc
}

fn get_parser(&self, lang: Language) -> &mut Parser {
    self.pools[lang].pop().unwrap_or_else(|| create_parser(lang))
}
```

**Expected Speedup**: 1.2x (eliminates parser init overhead)
**Complexity**: Medium

### Strategy 6: Incremental Skip Logic

```rust
fn should_process(file: &Path, db: &CozoDb) -> bool {
    let current_hash = compute_file_hash(file);
    let stored_hash = db.get_file_hash(file);

    current_hash != stored_hash  // Only process if changed
}
```

**Expected Speedup**: 10-100x for re-indexing unchanged codebases
**Complexity**: Low (ISGL1 v2 already has hashing)

### Strategy 7: Batch Edge Generation

```rust
// Current: Generate edges one-by-one during parsing
// Better: Collect all edges, deduplicate, batch insert

let all_edges: Vec<Edge> = files.par_iter()
    .flat_map(|f| extract_edges(f))
    .collect();

let unique_edges = deduplicate(all_edges);
db.insert_edges_batch(unique_edges);  // Single DB call
```

**Expected Speedup**: 2x for edge processing
**Complexity**: Low (already have `insert_edges_batch`)

---

## Part 4: Implementation Priority Matrix

| Strategy | Speedup | Complexity | Priority |
|----------|---------|------------|----------|
| **1. Rayon parallel files** | 4-8x | Low | **P0** |
| **6. Incremental skip** | 10-100x | Low | **P0** |
| 7. Batch edge generation | 2x | Low | P1 |
| 3. Work-stealing pool | 6-10x | High | P1 |
| 2. Async file I/O | 1.5-2x | Medium | P2 |
| 4. Memory-mapped files | 1.3x | Low | P2 |
| 5. Parser pool | 1.2x | Medium | P3 |

---

## Part 5: Recommended v1.5.1 Implementation

### Phase 1: Quick Wins (Expected: 5x speedup)

1. **Add Rayon for parallel file processing**
   ```toml
   # Cargo.toml
   rayon = "1.10"
   ```

2. **Implement incremental skip logic**
   - Check file modification time first (fast)
   - Fall back to content hash if mtime changed
   - Skip files with matching hash

### Phase 2: Architecture Improvements (Expected: additional 2x)

3. **Producer-consumer pipeline**
   - Decouple file reading from parsing
   - Decouple parsing from database writes

4. **Batch size tuning**
   - Current: Insert all entities at once
   - Better: Insert in chunks of 1000 (better memory)

### Combined Expected Result

| Phase | Speedup | Cumulative |
|-------|---------|------------|
| Baseline | 1x | 13.5 files/sec |
| Phase 1 (Rayon + Skip) | 5x | 67.5 files/sec |
| Phase 2 (Pipeline) | 2x | 135 files/sec |
| **Total** | **10x** | **135 files/sec** |

---

## Part 6: Performance Contracts

### REQ-PERF-001: Parallel Processing

```
WHEN processing 10,000 files on 8-core machine
THEN utilization SHALL be >= 70% on all cores
AND throughput SHALL be >= 100 files/second
```

### REQ-PERF-002: Incremental Re-indexing

```
WHEN re-indexing codebase with 5% changed files
THEN only changed files SHALL be processed
AND total time SHALL be <= 10% of full index time
```

### REQ-PERF-003: Memory Efficiency

```
WHEN processing 1GB codebase
THEN peak memory usage SHALL be <= 2GB
AND no memory leaks SHALL occur
```

---

## Part 7: Benchmarking Plan

### Benchmark 1: Parseltongue Self-Test

```bash
# Current baseline
time parseltongue pt01-folder-to-cozodb-streamer .

# Expected: ~2.2 seconds (already achieved in v1.5.0)
```

### Benchmark 2: Medium Codebase (10K files)

```bash
# Clone a medium OSS project
git clone https://github.com/example/medium-project
time parseltongue pt01-folder-to-cozodb-streamer ./medium-project

# Target: < 100 seconds
```

### Benchmark 3: Large Codebase (42K files, C#/JS)

```bash
# The user's actual use case
time parseltongue pt01-folder-to-cozodb-streamer ./large-codebase

# Current: 3100 seconds
# Target:  310 seconds
```

---

## Part 8: Risk Assessment

| Risk | Mitigation |
|------|------------|
| Rayon adds complexity | Well-tested library, minimal API |
| Race conditions in parallel code | Use thread-safe data structures |
| Memory pressure from parallelism | Limit concurrent files (bounded channel) |
| Parser not thread-safe | Create parser per thread (tree-sitter is thread-local) |
| Database contention | Batch inserts reduce contention |

---

## Part 9: Dependencies to Add

```toml
# Cargo.toml additions for v1.5.1

[dependencies]
rayon = "1.10"              # Parallel iterators
crossbeam = "0.8"           # Channels, scoped threads
num_cpus = "1.16"           # Detect CPU count
memmap2 = "0.9"             # Memory-mapped files (optional)
```

---

## Appendix: Parseltongue API Queries Used

```bash
# Complexity hotspots
curl http://localhost:7777/complexity-hotspots-ranking-view?top=30

# Entity distribution
curl http://localhost:7777/code-entities-list-all?limit=1000

# Edge distribution
curl http://localhost:7777/dependency-edges-list-all?limit=1000

# Search specific components
curl http://localhost:7777/code-entities-search-fuzzy?q=streamer
curl http://localhost:7777/code-entities-search-fuzzy?q=batch
```

---

*Research Date: February 7, 2026*
*Methodology: Parseltongue self-analysis (dogfooding)*
*Target Release: v1.5.1*
