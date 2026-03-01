# V216 - PRD - Parseltongue reborn as a Rust LLM companion

# Big Rocks






- Big-Rock-01: the scope and dependencies
    - language Rust 21
    - rust-analyzer
    - more options
        - rust compiler
    - 


- Big-Rock-02: the primary-key
    - language|||kind|||scope|||name|||file_path|||discriminator
    - language: rust
    - kind: fn
    - scope: auth::service
    - name: authenticate_user
    - file_path: src/auth/service.rs
    - discriminator: sig_v3

- Big-Rock-03: code-graph-building
    - parse folder names
    - folders become entities of type folder,  
    - rust-ecosystem files
        - rust code
        - rust config
            - toml
            - 
        - rust tests
    - non-rust files






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

### Rust Analyzer Superset Mapping
Assessment:
1. Rust Analyzer inputs are a practical superset of our read-pointer format.
2. RA carries richer identity/range context (file identity + typed symbol context + ranges), which can be projected to our compact read key.

Canonical layers:
1. Stable identity layer:
   - `language|||kind|||scope|||name|||file_path|||discriminator`
2. Read-pointer layer (for code fetch):
   - `file_path|||entity_key|||start_line|||end_line`

Projection contract:
1. Resolve RA symbol/definition to canonical `entity_key`.
2. Convert RA range/offsets to line-based span.
3. Emit read pointer exactly as:
   - `filepath|||entity|||StartLine|||EndLine`
4. Use full entity span for default code read; keep narrower selection ranges optional for focused context.

Example:
1. `entity_key = rust|||fn|||auth::service|||authenticate_user|||src/auth/service.rs|||sig_v3`
2. `read_pointer = src/auth/service.rs|||rust|||fn|||auth::service|||authenticate_user|||src/auth/service.rs|||sig_v3|||40|||96`

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

### Happy Path Example (End-to-End)
User ask:
1. "Where is authentication implemented and what breaks if I change it?"

System flow:
1. Retriever returns top chunks with pointers:
   - `chunk_key=ch_01`, `file_path=src/auth/service.rs`, `start_line=40`, `end_line=96`, `score=0.91`
   - `chunk_key=ch_17`, `file_path=src/auth/token.rs`, `start_line=10`, `end_line=58`, `score=0.87`
2. Resolve chunks to canonical entities:
   - `ch_01 -> entity_key=rust|||fn|||auth::service|||authenticate_user|||src/auth/service.rs|||sig_v3`
   - `ch_17 -> entity_key=rust|||fn|||auth::token|||verify_token|||src/auth/token.rs|||sig_v1`
3. Freshness check passes (`file_hash` matches current file state).
4. Graph analysis runs on resolved entity key:
   - callers
   - callees
   - blast radius (hops=2)
5. Response returns:
   - ranked entities
   - trusted pointer ranges
   - dependency impact summary
   - confidence/truth-grade annotations

### Classification Model (Code + Non-Code)
Every discovered file must terminate in one classification:
1. `code-graph`
   - parseable source files used for entities/edges
2. `identifiable-tests`
   - test-only files (unit/integration/e2e/fixtures)
   - for mixed files, test tagging moves to entity-level (see below)
3. `docs`
   - markdown/rst/adoc and other documentation artifacts
4. `non-eligible-text`
   - unsupported language/extensions, generated blobs, binaries, or irrelevant text

Entity-level sub-classification in `code-graph`:
1. `implementation-entity`
   - functions/classes/types participating in runtime behavior
2. `test-entity`
   - test blocks inside otherwise non-test files (e.g., inline module tests, cfg(test), test functions)
   - excluded from default production blast-radius unless explicitly included
3. `comment-entity` (optional, low-trust)
   - extracted comments/docstrings as advisory metadata only
   - never promoted to dependency truth
4. `unparsable-entity`
   - file is eligible but parse failed/partial; store with degrade reason and non-verified grade

Handling rules:
1. Incompatible/unsupported files -> `non-eligible-text` with reason code.
2. Parse failures in supported files -> keep ledger row, mark `partial` or `failed`, never silently drop.
3. Mixed files stay `code-graph` at file level; only test nodes are marked `test-entity`.
4. Tests are visible and queryable with explicit filter flags, but excluded from default blast-radius unless requested.
5. Comments/docstrings can improve retrieval ranking, but cannot create verified edges without parser/LSP evidence.
