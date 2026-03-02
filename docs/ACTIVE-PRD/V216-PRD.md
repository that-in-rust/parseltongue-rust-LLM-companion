# V216 - PRD - Parseltongue reborn as a Rust LLM companion

# User Segment x Diferentiation

- Rust Open Source Library
    - Maintainers
    - Contributors



# Core flow for 216



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
    - folders become entities of type folder, with distance from  
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

