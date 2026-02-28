# V216 - PRD - Parseltongue reborn as a Rust LLM companion

# Big Rocks

- Big-Rock-01: the primary-key
    - 







# Detailed notes


## Big-Rock-01: the primary-key
**Status**: Drafted for decision close  
**Date**: 2026-02-28  
**Intent**: Freeze canonical identity and pointer contracts so retrieval, graph analysis, and source reads stay consistent at scale.

### Why This Big Rock Exists
V216 must avoid identity drift between parsers, search sidecars, and graph analysis.
If primary keys are unstable, every downstream answer (blast radius, context packs, reasoning) becomes untrustworthy.

### Binding Decisions (Draft)
**BR01-PK-D1: Canonical entity identity is required and stable.**  
All extracted entities must map to:
`language|||kind|||scope|||name|||file_path|||discriminator`

**BR01-PK-D2: Retrieval and graph layers must share the same canonical key.**  
Any sidecar hit (lexical/vector/FTS) must resolve to canonical `entity_key` before graph reasoning.

**BR01-PK-D3: Chunk identity is separate from entity identity.**  
Use `chunk_key` for retriever storage and ranking, and explicit mapping:
`chunk_key -> entity_key`  
Baseline cardinality is many chunks to one entity, with overlap support when needed.

**BR01-PK-D4: Metadata-first pointer storage is default.**  
Store pointers and ranking metadata, not full source bodies:
1. `entity_key`
2. `chunk_key`
3. `file_path`
4. `start_line`
5. `end_line`
6. `file_hash` (or VCS blob hash)
7. retrieval features (`embedding`, `fts_terms`, `score_aux`)

**BR01-PK-D5: Source text is resolved on read, not cached as truth.**  
Code text is fetched on-demand from filesystem or pinned revision.

**BR01-PK-D6: Freshness checks gate trust-grade.**  
If hash mismatch is detected:
1. mark stale
2. trigger reindex for affected file/chunk
3. degrade confidence until refreshed
4. block `verified` truth-grade from stale rows

### Options Under Evaluation
**Option PK-A: Local sidecar-first**
1. Keep graph canonical in Parseltongue.
2. Use local retriever sidecar for candidate spans.
3. Resolve spans to `entity_key` before analysis.
4. Lower infrastructure overhead.

**Option PK-B: Turso/libSQL retrieval index**
1. Persist pointer metadata + ranking features in Turso/libSQL.
2. Keep source-on-read model (no full body persistence).
3. Better for shared/team retrieval at scale.
4. Higher operational complexity.

**Option PK-C: Hybrid**
1. Local-first default.
2. Turso mode as explicit opt-in.
3. Same canonical key schema in both modes.

### Open Questions
1. `OQ-PK-1`: What is the final schema and uniqueness contract for `entity_key` and `chunk_key`?
2. `OQ-PK-2`: How should overlaps be represented when one chunk maps to multiple entities?
3. `OQ-PK-3`: Should `entity_version_hash` be mandatory or optional in V216?
4. `OQ-PK-4`: Is Turso mode in V216 core scope or deferred behind a capability flag?
5. `OQ-PK-5`: What p95 freshness and retrieval latency SLOs are required for default enablement?

### Acceptance Criteria
1. 100% of queryable entities have canonical `entity_key`.
2. 100% of retriever hits can resolve to `entity_key` or explicit unresolved state.
3. No stale row can produce `verified` truth-grade output.
4. Pointer-based source retrieval returns explicit stale/missing errors, never silent fallback.
5. Both local and Turso modes (if enabled) preserve identical key semantics.
