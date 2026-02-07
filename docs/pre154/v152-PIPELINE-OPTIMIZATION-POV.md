# Pipeline Optimization POV: Three-Stage Ingestion Architecture

**Date**: 2026-02-07
**Version**: v1.5.2 Research
**Author**: Claude Code (Anthropic)
**Analysis Method**: Parseltongue HTTP API (no grep/glob)

---

## Executive Summary

Analysis of the proposed three-stage pipeline optimization for the Parseltongue ingestion system. The proposal suggests converting the current sequential file processing into a parallel pipeline with three stages: Discovery, Parsing, and Database Writes.

---

## Architecture Analysis via Parseltongue

### Current State (Verified via HTTP API)

| Component | What Parseltongue Found | Evidence |
|-----------|------------------------|----------|
| **File Discovery** | Uses `walkdir` (sequential) | `rust:module:WalkDir:external-dependency-walkdir:0-0` |
| **Main Processing** | `stream_directory` → `stream_file` chain | Blast radius shows 14 affected entities |
| **Database Layer** | `CozoDbStorage` with batch methods | `insert_entities_batch`, `insert_edges_batch` |
| **Hotspot** | `streamer.rs` - 136 outbound edges | Rank #6 in complexity hotspots |

### Dependency Flow (from `/forward-callees-query-graph`)

```
run_folder_to_cozodb_streamer
    └─→ stream_directory (streamer.rs)
        └─→ stream_file (streamer.rs:627)
            └─→ insert_entities_batch (cozo_client.rs)
            └─→ insert_edges_batch (cozo_client.rs)
```

### Current Batch Behavior

Batch insert exists but is **per-file**, not **cross-file**. Each `stream_file` call writes immediately.

---

## The Original Proposal (LLM Suggestion)

### The Mental Model: Factory Assembly Line

Current pipeline as a single worker doing everything sequentially:

```
Pick up file → Read it → Parse it → Extract entities → Write to DB → Next file
```

One file at a time, 42,000 times. The worker's hands are idle while waiting for DB writes. The parser sits idle while files are being read.

### The New Model: Three Stations Connected by Conveyor Belts

**Station 1 — Discovery** (the `ignore` crate's thread pool)

Multiple workers walk the filesystem in parallel. The moment a file is found, it goes on a conveyor belt (crossbeam bounded channel). Nobody waits for the full directory walk to finish. Files flow continuously.

**Station 2 — Parsing** (Rayon bursts)

Workers at this station pull files off the conveyor belt. They accumulate a small batch (say 100 files), then use Rayon to parse all 100 in parallel across all CPU cores. Parsed results — entities and edges — go onto a second conveyor belt. The key insight: this station doesn't need to know how many total files exist. It just keeps pulling, parsing, sending.

**Station 3 — Database Writes** (single dedicated thread)

One worker pulls parsed results off the second conveyor belt, accumulates them into batches of 300 rows, and writes to CozoDB. This worker never waits for parsing to finish. It writes continuously as results arrive.

### Why This Is Faster

All three stations run **simultaneously**:

```
Time →
Station 1: [walk walk walk walk walk walk ...]
Station 2:    [parse parse parse parse parse ...]
Station 3:       [write write write write ...]
```

In the current model, the total time is:

```
walk_time + read_time + parse_time + extract_time + write_time = 3100s
```

In the pipeline model, the total time is:

```
max(walk_time, parse_time, write_time) + startup_drain ≈ 1500s + overhead
```

The slowest station (parsing at 1500s) determines throughput. Everything else happens *during* parsing, essentially for free.

---

## Proposal Validity Assessment

### What the LLM Got RIGHT

1. **Sequential bottleneck is real** - `walkdir` is sequential, `stream_file` waits for DB writes
2. **Batch writes already exist** - Can leverage existing `insert_entities_batch` / `insert_edges_batch`
3. **Backpressure via bounded channels is sound** - Prevents memory explosion on 42K files
4. **Three-stage model is clean** - Discovery → Parse → Write is a natural separation

### What Needs Adjustment

1. **Tree-sitter Parser Thread Safety** - The LLM correctly notes `thread_local!` need, but Parseltongue shows parsing happens in `stream_file` which would need refactoring
2. **CozoDbStorage is NOT thread-safe for concurrent writes** - Single writer is correct, but need to verify CozoDB's internal locking
3. **`ignore` crate requires new dependency** - Currently using `walkdir`, not `ignore`

---

## Three Implementation Possibilities

### Option 1: Conservative Pipeline (Lowest Risk)

```
Changes: streamer.rs only
Parallelism: Parsing only (Rayon within file)
Timeline: ~2 days

                  ┌─────────────────────┐
                  │  walkdir (current)  │ ← Keep sequential
                  └─────────────────────┘
                            │
                            ▼
              ┌──────────────────────────┐
              │   stream_file (Rayon)    │ ← Parallelize entity extraction
              │   - Parse in parallel    │    within each file
              │   - Batch per-file       │
              └──────────────────────────┘
                            │
                            ▼
              ┌──────────────────────────┐
              │   insert_batch (single)  │ ← Keep single-threaded
              └──────────────────────────┘
```

**Expected Speedup**: 1.3-1.5x (parsing only, still file-sequential)
**Risk**: Low - minimal architectural change
**Files Changed**: `streamer.rs` only

---

### Option 2: Full Pipeline (The LLM's Proposal)

```
Changes: streamer.rs + new pipeline module
Dependencies: Add `ignore`, `crossbeam`
Timeline: ~1 week

  ┌──────────────────┐   Channel A   ┌──────────────────┐   Channel B   ┌──────────────────┐
  │   ignore crate   │   (512 cap)   │   Rayon parse    │   (512 cap)   │   Single writer  │
  │  parallel walk   │ ────────────▶ │   thread-local   │ ────────────▶ │   batch 300 rows │
  │  (filesystem)    │               │   Parsers        │               │   CozoDbStorage  │
  └──────────────────┘               └──────────────────┘               └──────────────────┘
```

**Expected Speedup**: 2-3x on large codebases
**Risk**: Medium - significant refactor of `stream_directory`
**New Dependencies**: `ignore`, `crossbeam-channel`

**Key Refactoring Needed** (from Parseltongue):
- `stream_directory` (blast radius: 4 hop-2 dependents) must become a spawner
- `stream_file` must return results instead of writing directly
- Error handling needs channel-based propagation

---

### Option 3: Async Pipeline (Future-Proof)

```
Changes: Major refactor to async
Dependencies: tokio channels (already have tokio)
Timeline: ~2 weeks

  ┌──────────────────┐   tokio mpsc  ┌──────────────────┐   tokio mpsc  ┌──────────────────┐
  │   tokio::spawn   │   (bounded)   │  spawn_blocking  │   (bounded)   │   Single async   │
  │  fs walk tasks   │ ────────────▶ │   parse tasks    │ ────────────▶ │   writer task    │
  └──────────────────┘               └──────────────────┘               └──────────────────┘
```

**Expected Speedup**: 2-4x with async I/O benefits
**Risk**: High - requires async refactor of `CozoDbStorage` methods
**Benefit**: Aligns with existing tokio usage in HTTP server

**Parseltongue shows**: `CozoDbStorage` methods are already async (`insert_entities_batch`, `insert_edges_batch`), but called from sync context in streamer.

---

## Comparison Matrix

| Factor | Option 1 | Option 2 | Option 3 |
|--------|----------|----------|----------|
| Speedup | 1.3-1.5x | 2-3x | 2-4x |
| Risk | Low | Medium | High |
| Time | 2 days | 1 week | 2 weeks |
| Complexity | Minimal | Moderate | Significant |
| Dependencies | None | +2 crates | None (tokio exists) |

---

## Key Parseltongue Findings

### Entities in Critical Path

From `/code-entities-list-all`:
- `rust:mod:streamer` - Main streaming module
- `rust:struct:StreamerConfig` - Configuration
- `rust:method:stream_file` - Per-file processing
- `rust:struct:CozoDbStorage` - Database layer

### Database Methods (from cozo_client.rs)

Already async-capable:
- `insert_entities_batch` - Batch entity insertion
- `insert_edges_batch` - Batch edge insertion
- `get_all_entities` - Query all entities
- `get_all_dependencies` - Query all edges

### Complexity Hotspots (from `/complexity-hotspots-ranking-view`)

| Rank | Entity | Coupling |
|------|--------|----------|
| 6 | `streamer.rs` | 136 outbound |
| 13 | `cozo_client.rs` | 77 outbound |
| 12 | `incremental_reindex_core_logic.rs` | 82 outbound |

---

## Recommendation

**Initial Recommendation**: Option 2 (Full Pipeline) for best balance of risk/reward.

**However**: User requested research into **Option 3 (Async Pipeline)** first.

---

## Next Steps

1. **Research Option 3** - Deep dive into async pipeline feasibility
2. **Analyze tokio integration points** - Current async usage patterns
3. **Identify blocking operations** - Tree-sitter parsing, file I/O
4. **Design async-compatible streamer** - Without breaking existing API

---

*Analysis performed using Parseltongue HTTP API endpoints only (no grep/glob)*
*Server: http://localhost:7777*
