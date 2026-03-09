# Minimal V200 Architecture Decisions - Complete Record

**Version:** v300.0.0
**Date:** 2026-03-09
**Status:** Compiled from conversation analysis
**Purpose:** Record all decisions, rationale, and stories from v200 planning discussions

---

# Executive Summary

**Core Thesis:** The 7-event journey (QUERY → SEARCH → ANCHOR → CLUSTER → ASK → CHOICE → DEEP DIVE) is the product. Everything else is infrastructure. Build only what serves this journey.

---

# Decision Log

## Decision 01: The Core Differentiation

**Date:** 2026-03-09
**Context:** Shreyas Doshi product thinking

### The 7-Event User Journey

```
Event 1: QUERY     →  LLM sends ~7 words
Event 2: SEARCH     →  RRF fusion finds 4 candidates (<10ms)
Event 3: ANCHOR     →  BFS to public interface (<50ms)
Event 4: CLUSTER    →  Ego network 1-hop (<100ms)
Event 5: ASK        →  Present 4 options (~200 tokens)
Event 6: CHOICE     →  LLM picks [1-4] or none
Event 7: DEEP DIVE  →  Full context up to 20k tokens (<500ms)
```

### Why This Wins

| Tool | Returns |
|------|---------|
| grep | Lines matching pattern |
| IDE search | Files containing text |
| Embedding search | Similar code chunks (guessed) |
| **Parseltongue** | Compiler-verified clusters + deep context |

**Token economics:**
- LLM pays 200 tokens to decide
- Then pays 20k for ONE deep dive (not 80k for all 4 candidates)

### Story

> A user asks "authentication flow in this codebase". Traditional tools dump files or guess with embeddings. Parseltongue:
> 1. Finds 4 candidates (auth::login, AuthProvider, etc.)
> 2. Anchors each to its public API boundary
> 3. Builds clusters with callers/callees
> 4. Presents options to LLM
> 5. LLM picks one
> 6. Returns compiler-verified deep context
>
> The user gets understanding, not lines.

---

## Decision 02: Remove CozoDB

**Date:** 2026-03-09
**Context:** Simplification for minimal v200

### The Rationale

| 7-Event Phase | What It Needs | Needs CozoDB? |
|---------------|---------------|----------------|
| Event 1: QUERY | 7 words | No |
| Event 2: SEARCH | FTS/trigram/RRF | No - SQLite works |
| Event 3: ANCHOR | Span → public interface | No - compiler/AST |
| Event 4: CLUSTER | 1-hop neighbors | No - simple edges table |
| Event 5-6: ASK/CHOICE | 200 tokens | No |
| Event 7: DEEP DIVE | CFG/DDG/types | No - rustc |

**The moat is NOT the graph database.**

The moat is:
1. span → public interface anchor
2. anchor → candidate cluster
3. choice → compiler-backed deep dive

### Current State (V161-V200)

CozoDB exists but is **underutilized**:

- `calculate_blast_radius()` exists in Cozo native
- HTTP endpoint uses Rust BFS + repeated Cozo queries
- Most graph analysis runs in Rust on top of Cozo lookups
- Smart-context, centrality, clustering all rebuild graphs in Rust

**Cozo is storage, not the computation engine.**

### Decision

**For minimal v200: Remove CozoDB from critical path.**

---

## Decision 03: Single libSQL/SQLite Store

**Date:** 2026-03-09
**Context:** Storage architecture

### The Schema

```sql
-- Entities (ISG_L1_V3 keyed)
CREATE TABLE entities (
    entity_key TEXT PRIMARY KEY,  -- ISG_L1_V3
    file_path TEXT,
    start_line INTEGER,
    end_line INTEGER,
    visibility TEXT,             -- pub/private
    entity_type TEXT,            -- fn/struct/trait/impl
    semantic_path TEXT,          -- module hierarchy
    language TEXT,
    signature_summary TEXT,
    content_hash TEXT
);

-- Edges
CREATE TABLE edges (
    from_entity_key TEXT,
    to_entity_key TEXT,
    edge_kind TEXT,              -- calls/uses/implements
    source TEXT,                 -- rustc/treesitter
    confidence REAL,
    PRIMARY KEY (from_entity_key, to_entity_key, edge_kind)
);

-- Chunks (codemogger-style + entity mapping)
CREATE TABLE chunks (
    chunk_key TEXT PRIMARY KEY,  -- file_path:start_line:end_line
    file_path TEXT,
    start_line INTEGER,
    end_line INTEGER,
    kind TEXT,
    name TEXT,
    signature TEXT,
    snippet_preview TEXT,
    file_hash TEXT,
    entity_key TEXT,             -- ISG_L1_V3 if known
    anchor_entity_key TEXT       -- Public interface key
);

-- Search index
CREATE VIRTUAL TABLE chunks_fts USING fts5(
    name, signature, snippet_preview
);

-- File tracking
CREATE TABLE indexed_files (
    file_path TEXT PRIMARY KEY,
    file_hash TEXT,
    indexed_at INTEGER
);

-- Aliases for fuzzy search
CREATE TABLE entity_aliases (
    alias_text TEXT,
    entity_key TEXT,
    alias_type TEXT,             -- name/path/signature
    PRIMARY KEY (alias_text, entity_key)
);
```

### Why One Store

- Single transaction boundary
- No dual-write synchronization
- No drift between stores
- Simpler debugging
- One file to backup/move

### Story

> The user searches "auth login". SQLite FTS returns chunks. Each chunk has an `anchor_entity_key` column that points to the public interface. The system loads that entity + its edges from the same database. No network calls, no cross-database joins, no complexity.

---

## Decision 04: Validated Graph Algorithms

**Date:** 2026-03-09
**Context:** Replace homegrown implementations

### The Problem

Current graph algorithms in `parseltongue-core/src/graph_analysis/`:
- Internally tested: yes
- Validated against external truth: **no**
- Production-scale validation: **no**

This is "hoping they're right."

### The Solution: Trustworthy Libraries

| Need | Library | Status | Why Trust |
|------|---------|--------|-----------|
| Graph structure | **petgraph** | 3,773 stars | Industry standard |
| BFS/DFS | petgraph | Built-in | Standard implementation |
| Shortest path | petgraph | Built-in | Dijkstra, A* |
| SCC (Tarjan) | petgraph | Built-in | Standard algorithm |
| PageRank | petgraph | Built-in | Standard algorithm |
| Dominators | petgraph | Built-in | CFG support |
| Betweenness | **rustworkx-core** | Qiskit ecosystem | Validated |
| k-core | rustworkx-core | Built-in | Validated |
| Connectivity | rustworkx-core | Built-in | Validated |
| Leiden/Louvain | **graphrs** | Specialized | Analytics crate |
| **Validation oracle** | **NetworkX** | Ground truth | Python standard |

### Algorithm Replacement Map

| Current File | Replace With | Validate Against |
|--------------|--------------|------------------|
| `centrality_measures_algorithm.rs` | petgraph + rustworkx-core | NetworkX |
| `kcore_decomposition_algorithm.rs` | rustworkx-core | NetworkX |
| `leiden_community_algorithm.rs` | graphrs | NetworkX |
| `tarjan_scc_algorithm.rs` | petgraph | NetworkX |
| Blast radius (BFS) | petgraph | Unit tests |

### Story

> We had 70+ algorithms documented but not validated. The other LLM pointed this out. The fix: stop writing graph algorithms. Use petgraph (3,773 stars, battle-tested). Use rustworkx-core (from Qiskit, validated). Use NetworkX as the oracle in tests. Now when PageRank returns 0.0247, we KNOW it's correct because NetworkX agrees.

---

## Decision 05: rustc_private + Version Pinning

**Date:** 2026-03-09
**Context:** Compiler integration

### The Fear

> "rustc_private breaks every nightly!"

### The Reality

| API Category | Change Frequency |
|-------------|------------------|
| Core queries (`type_of`, `fn_sig`, `visibility`) | **0%** |
| MIR structure | **1%** |
| HIR structure | **3%** |
| Helper methods | **5%** |

### The Solution

```toml
# rust-toolchain.toml
[toolchain]
channel = "nightly-2025-03-01"
components = ["rustc-dev", "llvm-tools-preview"]
```

**This file NEVER changes. Problem solved.**

### What You Get With rustc_private

| Data | Charon | rustc_private |
|------|--------|---------------|
| Functions, types, traits | ✅ | ✅ |
| Resolved calls | ✅ | ✅ |
| Signatures | ✅ | ✅ |
| Visibility | ✅ | ✅ |
| **MIR bodies** | ⚠️ Partial | ✅ Full |
| **Polonius borrow facts** | ❌ | ✅ |
| **Type inference** | ⚠️ Partial | ✅ Full |
| **Lifetime information** | ❌ | ✅ |
| **Monomorphization** | ❌ | ✅ |
| **Data flow analysis** | ❌ | ✅ |

### Tools That Prove This Works

- Miri (undefined behavior detection)
- Flowistry (ownership analysis)
- Aquascope (visualizations)
- Prusti (verification)
- Creusot (verification)
- Rudra (security analysis)
- Kani (model checking)

**All use rustc_private. All pin versions. All work.**

### Story

> The "instability" narrative is fear-mongering. Pin `nightly-2025-03-01`. Write code against `TyCtxt`, `optimized_mir()`, `fn_sig()`. These don't change. If you upgrade in 6 months, 95% works, 5% needs trivial renames. You control the upgrade. You get full compiler truth. Just pin and ship.

---

## Decision 06: Compiler Truth Storage

**Date:** 2026-03-09
**Context:** Where to store rustc information

### Hot Facts (In libSQL)

```sql
-- Entities (already defined)
-- Edges (already defined)

-- Type summaries
CREATE TABLE type_summaries (
    entity_key TEXT PRIMARY KEY,
    return_type TEXT,
    arg_types TEXT,              -- JSON array
    generic_params TEXT,         -- JSON array
    trait_bounds TEXT            -- JSON array
);

-- Impl summaries
CREATE TABLE impl_summaries (
    impl_key TEXT PRIMARY KEY,
    trait_key TEXT,
    self_type_key TEXT,
    method_keys TEXT             -- JSON array
);

-- Anchor metadata
CREATE TABLE anchor_metadata (
    entity_key TEXT PRIMARY KEY,
    nearest_public_ancestor TEXT,
    module_hierarchy TEXT,
    crate_ownership TEXT
);
```

### Cold Facts (Compressed Files)

```
.parseltongue/
  compiler-cache/
    <snapshot_id>/
      <isg_l1_v3_key>/
        cfg.json.zst          # Control flow graph
        mir_summary.json.zst  # MIR body snapshot
        borrow_facts.json.zst # Polonius facts
```

### Index Table

```sql
CREATE TABLE compiler_artifact_index (
    snapshot_id TEXT,
    entity_key TEXT,
    artifact_kind TEXT,         -- cfg/mir/borrow_facts
    artifact_path TEXT,
    content_hash TEXT,
    toolchain_version TEXT,
    created_at INTEGER,
    PRIMARY KEY (snapshot_id, entity_key, artifact_kind)
);
```

### Why This Split

- Hot facts: Queried constantly in Events 3, 4, 7
- Cold facts: Only accessed on deep dive
- Don't bloat the DB with heavy artifacts

---

## Decision 07: Two-Layer Architecture

**Date:** 2026-03-09
**Context:** Product architecture

### Layer 1: Search Sidecar

**Responsibility:** Find likely code spans quickly

- FTS/trigram search
- RRF fusion (exact + fuzzy + git-recency)
- File hash incremental indexing
- Codemogger-style chunk storage

**Output:** Top 4 candidate spans

### Layer 2: Parseltongue Truth Layer

**Responsibility:** Map spans to understanding

- Span → public interface anchor
- Anchor → candidate cluster (callers/callees)
- Choice → compiler-backed deep dive

**Output:** Understanding units

### The Mapping

```
Event 1: query     →  User/LLM sends short query
Event 2: search     →  Search sidecar (codemogger-inspired)
Event 3: anchor     →  Parseltongue truth layer
Event 4: cluster    →  Parseltongue truth layer
Event 5: ask        →  Present [1][2][3][4][none]
Event 6: choice     →  LLM picks
Event 7: deep dive  →  Parseltongue + rustc_private
```

### Story

> Codemogger helps you get to useful spans fast. Parseltongue adds the thing codemogger doesn't have: interface boundary, graph context, compiler trust. The product line becomes: codemogger-style search finds where to look, Parseltongue decides what it means.

---

## Decision 08: Tauri as Shell Only

**Date:** 2026-03-09
**Context:** UI architecture

### The Stack

```
Tauri UI (shell)
    ↓
Rust backend (core logic)
    ↓
libSQL local store
    ↓
rustc_private (compiler sidecar)
```

### What Tauri Does

- Window management
- File picker dialogs
- Settings UI
- Progress visualization
- MCP server management

### What Tauri Does NOT Do

- Graph algorithms (Rust core)
- Compiler analysis (rustc_private)
- Database operations (libSQL)
- Search logic (Rust core)

### Why This Matters

Tauri is a **thin shell**. The real work happens in Rust. This keeps:
- Core logic testable (unit tests)
- Core logic portable (could be CLI, could be library)
- Tauri focused on UI concerns

---

## Decision 09: ISG_L1_V3 as Canonical Key

**Date:** 2026-03-09
**Context:** Entity identity

### The Key Format

```
language|||kind|||scope|||name|||file_path|||discriminator

Example:
rust|||fn|||auth::service|||authenticate_user|||src/auth/service.rs|||sig_v3
```

### What It Bridges

- **Chunks table** → Has `entity_key` and `anchor_entity_key` columns
- **Entities table** → Primary key
- **Edges table** → Both ends use ISG_L1_V3
- **Type summaries** → Keyed by ISG_L1_V3
- **Compiler artifacts** → Files named by ISG_L1_V3

### Why This Matters

One key format. Everywhere. No translation layers. No ambiguity.

---

# Not Borrowed from Codemogger

## What NOT to Steal

| Item | Reason |
|------|--------|
| Tree-sitter as final truth for Rust | Use rustc_private instead |
| Vector embeddings as default | Not needed for MVP |
| "chunk result = answer" thinking | We add anchor + cluster |
| Bun/TypeScript runtime | We're Rust + Tauri |
| Default embedding dependency | Optional, not required |

## What TO Steal

| Item | How We Use It |
|------|---------------|
| `chunk_key = file_path:start_line:end_line` | Direct use |
| File-hash incremental indexing | Direct use |
| Query preprocessing (stopwords) | Adapt to Rust |
| RRF merge pattern | Adapt lanes |
| Thin MCP surface | Copy structure |
| Per-codebase search DB | Adopt pattern |
| Schema structure | Extend with entity keys |

---

# Implementation Priority

## Phase 1: Foundation

1. Create new minimal crate structure
2. Add libSQL/SQLite storage
3. Implement chunk ingestion (codemogger-inspired)
4. Add ISG_L1_V3 key generation

## Phase 2: Search

5. FTS5 setup
6. Trigram indexing
7. RRF fusion
8. Event 2 endpoint

## Phase 3: Graph

9. petgraph integration
10. Entity/edge loading into memory
11. BFS for anchoring
12. Events 3-4 endpoints

## Phase 4: Compiler

13. rustc_private integration
14. rust-toolchain.toml pinning
15. Type extraction
16. Call edge extraction
17. Event 7 endpoint

## Phase 5: Tauri

18. Tauri app scaffold
19. UI for ingestion
20. UI for search
21. UI for cluster selection
22. MCP server integration

---

# Success Criteria

## Minimal V200 Ships When

- [ ] One libSQL file stores all data
- [ ] Search finds 4 candidates in <10ms
- [ ] Anchoring completes in <50ms
- [ ] Clusters build in <100ms
- [ ] Deep dive returns in <500ms
- [ ] All graph algorithms use petgraph/rustworkx-core
- [ ] All algorithms validated against NetworkX
- [ ] rustc_private pinned and working
- [ ] Tauri app is functional

## Quality Gates

- [ ] Zero homegrown graph algorithms (use libraries)
- [ ] Zero CozoDB dependency
- [ ] 100% test coverage on critical path
- [ ] NetworkX validation fixtures for all algorithms

---

# References

- `docs/v300/PRD-v300.md` - Core 7-event journey
- `docs/v300/algorithm_endpoint_journey_matrix_202603091430.md` - Algorithm mapping
- `docs/v300/rustc_private_stability_rationale_202603091530.md` - Version pinning rationale
- `CR09/codemogger/` - Reference implementation
- [petgraph](https://github.com/petgraph/petgraph)
- [rustworkx-core](https://github.com/Qiskit/rustworkx)
- [graphrs](https://github.com/freemed/rustgraphrs)
- [NetworkX](https://networkx.org/)

---

**Document Version:** 1.0.0
**Generated:** 2026-03-09
**Total Decisions Documented:** 9
**Total Stories:** 5
