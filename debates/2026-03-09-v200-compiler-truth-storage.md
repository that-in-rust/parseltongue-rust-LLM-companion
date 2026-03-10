# Debate: Where should rich Rust compiler information live in Parseltongue v200?

**Created:** 2026-03-09
**Agent 1:** Codex-Store-Split
**Agent 2:** Codex-Minimal-SingleDB
**Agent 3:** -
**Max Rounds:** 2
**Status:** CONVERGED

## Context

Decision for v200: keep codemogger/Turso-style chunk search, but decide whether rich Rust compiler information should live in a separate truth store keyed by ISG_L1_V3 or in the same local SQL database as search tables. Debate scope is storage architecture only, not UI or ranking.

### Evidence

- codemogger chunks schema stores file_path, chunk_key, language, kind, name, signature, snippet, start_line, end_line, file_hash, embedding; it has no graph edge or compiler-fact tables (CR09/codemogger/src/db/schema.ts:10-31).
- Parseltongue Cozo schema currently mixes Current_Code/Future_Code blobs with interface_signature, TDD_Classification, lsp_meta_data, file_path, entity_type, semantic_path in one CodeGraph relation (crates/parseltongue-core/src/storage/cozo_client.rs:95-117).
- Parseltongue already models graph edges separately in DependencyEdges with from_key/to_key/edge_type (crates/parseltongue-core/src/storage/cozo_client.rs:153-164).
- The shipped blast-radius endpoint computes BFS in Rust and issues repeated Cozo raw_query calls rather than delegating the whole traversal to a graph DB query, showing graph algorithms can live in Rust over relational storage (crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs:176-276).

### Relevant Files

- CR09/codemogger/src/db/schema.ts:10-31
- CR09/codemogger/src/db/store.ts:90-147
- crates/parseltongue-core/src/storage/cozo_client.rs:95-164
- crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs:176-276

### Constraints

- v200 is CPU-only.
- Primary bridge key is ISG_L1_V3.
- Search UX follows the 7-event converge flow.
- Pinned toolchain is acceptable; upgrade churn is out of scope for this cycle.

---

## Proposal

STATUS: CONVERGED

[A1-R1] Initial proposal: use **two logical persistence lanes** and keep rich compiler truth out of the codemogger-style chunk rows.

1. **Search lane**: keep codemogger-style chunk/index tables for Event 2 search and code preview.
   - Evidence: codemogger `chunks` stores `file_path`, `chunk_key`, `language`, `kind`, `name`, `signature`, `snippet`, `start_line`, `end_line`, `file_hash`, and optional embeddings, but no graph edges or compiler facts at [CR09/codemogger/src/db/schema.ts:12-30](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/CR09/codemogger/src/db/schema.ts:12).
   - Evidence: codemogger batch writes are file-centric and optimized for reindexing chunks and file hashes, not entity graph maintenance, at [CR09/codemogger/src/db/store.ts:125-180](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/CR09/codemogger/src/db/store.ts:125).
   - Required addition: add `entity_key` and `anchor_entity_key` columns so search hits bridge into `ISG_L1_V3`.

2. **Truth lane**: persist compiler-derived entities and edges separately, keyed by `ISG_L1_V3`.
   - Evidence: Parseltongue already separates node-ish data and edge data in Cozo through `CodeGraph` and `DependencyEdges` at [crates/parseltongue-core/src/storage/cozo_client.rs:95-117](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/parseltongue-core/src/storage/cozo_client.rs:95) and [crates/parseltongue-core/src/storage/cozo_client.rs:153-164](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/parseltongue-core/src/storage/cozo_client.rs:153).
   - Evidence: the current `CodeGraph` relation mixes code blobs (`Current_Code`, `Future_Code`) with hot metadata (`interface_signature`, `semantic_path`, `entity_type`), which is exactly the muddle v200 should avoid at [crates/parseltongue-core/src/storage/cozo_client.rs:97-115](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/parseltongue-core/src/storage/cozo_client.rs:97).
   - Proposed truth tables: `truth_entities`, `truth_edges`, `truth_aliases`, `compiler_artifact_index`.

3. **Cold evidence cache**: store heavy CFG/DFG/MIR artifacts as compressed files, referenced by DB rows.
   - Reason: deep-dive artifacts are large and only needed in Event 7, while Events 2-5 need hot summaries and edges only.

4. **Execution model**: graph algorithms may still run in Rust over truth tables; a graph DB is not required for v200.
   - Evidence: the shipped blast-radius endpoint performs BFS in Rust and queries `DependencyEdges` incrementally, instead of relying on a database-native graph algorithm, at [crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs:188-289](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs:188).

[A2-R1] ~~use two logical persistence lanes~~

[A2-R1] Counterpoint: for v200, the minimum viable shape is **one physical local SQL database with two logical table groups**, not two separate databases or engines.

- Evidence: codemogger already proves that a single local SQL store can hold search chunks, indexed files, and per-codebase FTS successfully at [CR09/codemogger/src/db/schema.ts:12-85](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/CR09/codemogger/src/db/schema.ts:12).
- Evidence: blast-radius computation already runs in Rust and only needs keyed edge lookups from storage, so truth tables do not need a separate engine to support v200 traversal at [crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs:223-269](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/blast_radius_impact_handler.rs:223).
- Evidence: the actual Cozo mistake is the wide mixed schema in `CodeGraph`, not the mere fact that hot metadata and code live in one database file at [crates/parseltongue-core/src/storage/cozo_client.rs:95-117](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/crates/parseltongue-core/src/storage/cozo_client.rs:95).

[A2-R1] Proposed refinement:

1. Keep **one physical local libSQL/SQLite file** in v200.
2. Split it into logical groups:
   - `search_chunks`, `search_indexed_files`, `search_fts_*`
   - `truth_entities`, `truth_edges`, `truth_aliases`
   - `artifact_index`
3. Keep heavy compiler artifacts on disk, addressed by `ISG_L1_V3` plus snapshot/toolchain hash.
4. Put only `entity_key` / `anchor_entity_key` bridges into search rows. Do not replicate rich compiler facts inside `search_chunks`.

[A1-R2] Verified concession: the key product constraint is **separation of concerns**, not necessarily **two engines**.

- Evidence: none of the cited files show a current need for cross-engine isolation, distributed graph queries, or graph-native transactional semantics.
- Evidence: the strongest concrete problems are schema shape and write-path coupling, both solvable with one physical DB and strict table boundaries.

[A1-R2] Converged proposal:

1. **Adopt one physical embedded SQL database for v200**, preferably local libSQL/SQLite for Tauri-friendly packaging.
2. **Enforce two logical stores inside it**:
   - Search store: codemogger-style search rows, FTS, file hashes, chunk metadata.
   - Truth store: `ISG_L1_V3` entities, edges, aliases, public-interface anchors, compiler summaries.
3. **Keep deep compiler artifacts off the hot path**:
   - store pointers in `artifact_index`
   - store large CFG/DFG/MIR payloads as compressed files on disk
4. **Do not add more than bridge columns to search rows**:
   - required: `entity_key`, `anchor_entity_key`
   - optional: short `signature_summary`
   - forbidden in `search_chunks`: full MIR/CFG/DFG/type graph payloads
5. **Defer a second physical truth database** unless evidence appears for:
   - write amplification from mixed search/truth workloads
   - need for graph-native query language as a product surface
   - sync/replication constraints that force split deployment

[A2-R2] Independent verification complete. This is the smallest architecture that preserves the user journey:

- Event 2 stays fast because search rows remain codemogger-like.
- Events 3-4 get `ISG_L1_V3` anchoring from truth tables.
- Event 7 gets rich compiler depth from artifact files plus truth summaries.

---

## Parking Lot

- [A2-R1] Whether the truth tables should later move to Cozo, Kuzu, or remain in libSQL is parked until v200 proves the 7-event journey.
- [A1-R2] Whether to persist precomputed PageRank/Leiden/k-core outputs is parked until ranking quality is benchmarked on real v200 queries.

---

## Dispute Log

| Round | Agent | Section | What Changed | Why | Status |
|-------|-------|---------|--------------|-----|--------|
| 1 | Codex-Store-Split | Proposal | Proposed split between search lane, truth lane, and cold artifact cache | codemogger chunk schema lacks graph/compiler facts; current Cozo schema mixes blobs with hot metadata | CLOSED |
| 1 | Codex-Minimal-SingleDB | Proposal | Challenged separate physical stores; argued for one physical DB with logical separation | no evidence yet that v200 needs dual engines, while current traversal already runs in Rust over relational lookups | CLOSED |
| 2 | Codex-Store-Split | Proposal | Conceded physical split is unnecessary for v200 and rewrote proposal around one DB with two logical stores | minimum viable architecture should optimize product flow, not storage ideology | CLOSED |
| 2 | Codex-Minimal-SingleDB | Proposal | Accepted converged design after independent verification | proposal keeps search rows lean and preserves compiler-truth separation | CLOSED |

**Status values:** `OPEN` = unresolved, `CLOSED` = resolved, `PARKED` = deferred and not blocking convergence.
