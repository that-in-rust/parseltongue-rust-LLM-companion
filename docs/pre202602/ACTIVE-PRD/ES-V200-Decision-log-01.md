# v200-Decision-log-01
Status: Active — restructured 2026-02-22 using Minto Pyramid Principle
Purpose: Record binding decisions and open questions for V200.

## Top-of-Stack Decision Focus (2026-03-01)
**Status**: Priority stack for BR02/BR03 execution order  
**Intent**: Lock the pipeline `LLM -> SearchLayer -> CodeGraph -> HighROILLMContext -> LLM` as the default architecture focus.

Top-of-stack architecture priorities:
1. **Intent Router First**  
   Route each query into intent classes (`bug`, `refactor`, `explain`, `safety`, `migrate`) before retrieval policy is selected.
2. **Dual-Lane Search**  
   Run lexical/symbol and semantic retrieval in parallel, then fuse with graph-aware reranking.
3. **Mandatory Entity Wrap Gate**  
   No raw span can be returned without resolving to canonical wrapping entity and key.
4. **Typed Expansion Profiles**  
   Expansion edge priorities must vary by intent (calls/called_by/impl/type/cfg/dataflow).
5. **High-ROI Context Compiler**  
   Emit packetized context (anchor, key neighbors, minimal flow slices, risk hints), not file dumps.
6. **Proof-Carrying Context**  
   Every packet item must include provenance (`file:start:end`, source tool, hash freshness, confidence).
7. **Counterfactual Impact Mode**  
   First-class query mode for “what breaks if X changes,” driven by blast radius + typed edges + flow slices.
8. **Ambiguity Loop by Default**  
   If top candidates are close, return disambiguation payload instead of forced single answer.
9. **Freshness-Aware Trust Gate**  
   Stale/hash-mismatch evidence degrades confidence and triggers selective re-compute before `verified` output.
10. **Rustc Deep-Mode Trigger**  
   Escalate to rustc-sidecar only for high-risk intents; keep fast path lightweight for routine queries.

Top-of-stack open questions:
1. `OQ-BR02-24`: What are the intent taxonomy and routing rules that become V200 defaults?
2. `OQ-BR02-25`: What is the exact fusion strategy for dual-lane search (RRF/weighted sum/learned rerank)?
3. `OQ-BR03-6`: What minimal proof payload is required for a response to be considered actionable?
4. `OQ-BR07-11`: What p95 latency and freshness SLO define acceptable fast-mode vs rustc deep-mode switching?

List of Big Rocks
- BR01: Ingestion (Accuracy-First) that includes
  - segregating files and entities into 4 categories : 
    - docs
    - non-eligible-text (languages or extensions we do not support)
    - identifiable-tests (test files or entities)
    - code-graph (source code files)
  - enriching via tree-sitter, LSPs and common sense
    - basic metadata for entities, files and edges via tree-sitter
    - entities with meta-data from LSPs like Rust-Analyzer and others wherver possible
    - dependency edges with meta-data from LSPs like Rust-Analyzer and others wherver possible
    - dependency edges with new relationships that we can creatively infer
      - 2 functions in the same file are likely to be related by common context sharing
      - 2 statements in the same file at public interface level are likely to be related by common context sharing
- BR02: Query-First Graph Design
  - Following list of ideas
    - FIND ENTITIES
      - GET /code-entities-list-all                                  → all entities (data.total_count, data.entities[])
      - GET /code-entities-list-all?entity_type=function             → filter by type
      - GET /code-entities-search-fuzzy?q=PATTERN                    → fuzzy name search (data.total_count, data.entities[])
      - GET /code-entity-detail-view?key=ENTITY_KEY                  → full source code of one entity
    - TRACE DEPENDENCIES
      - GET /dependency-edges-list-all                               → all edges (from_key, to_key, edge_type)
      - GET /reverse-callers-query-graph?entity=ENTITY_KEY           → who calls this entity
      - GET /forward-callees-query-graph?entity=ENTITY_KEY           → what does this entity call
      - GET /blast-radius-impact-analysis?entity=ENTITY_KEY&hops=N   → transitive impact (default hops=2)
    - ANALYZE ARCHITECTURE
      - GET /circular-dependency-detection-scan                      → circular dependency cycles
      - GET /complexity-hotspots-ranking-view?top=N                  → most coupled entities (default top=10)
      - GET /semantic-cluster-grouping-list                           → module groupings
      - GET /strongly-connected-components-analysis                   → Tarjan SCC cycle detection
      - GET /technical-debt-sqale-scoring                             → ISO 25010 SQALE debt scores
      - GET /kcore-decomposition-layering-analysis                    → core/mid/peripheral layers
      - GET /centrality-measures-entity-ranking?method=pagerank       → entity importance (method: pagerank|betweenness)
      - GET /entropy-complexity-measurement-scores                    → Shannon entropy per entity
      - GET /coupling-cohesion-metrics-suite                          → CK metrics: CBO, LCOM, RFC, WMC
      - GET /leiden-community-detection-clusters                      → Leiden community detection
    - Explore Arxiv and other common sense things knowing you have a code-graph heavily enriched with useful information
- BR03: Bi-Directional Workflow reference where LLM queries the above list of APIs to generate insights
- BR04: Tauri App as a UI to manage & monitor parseltongue ingestion across your system
- BR05: 


---

## Scope Preservation Rule
Everything below BR01-BR04 is additive expansion.  
No original BR01-BR04 scope is reduced, replaced, or deprecated by these sections.

---

## Big-Rock-01: Ingestion Truth Loop (Accuracy-First)
**Status**: Drafted for immediate use (pending final sign-off)  
**Date**: 2026-02-25  
**Intent**: Make ingestion trustworthy before adding more graph algorithms.

### Scope Lock (Original Statement — Unchanged)
- BR01: Ingestion (Accuracy-First) that includes
  - segregating files and entities into 4 categories:
    - `docs`
    - `non-eligible-text (languages or extensions we do not support)`
    - `identifiable-tests (test files or entities)`
    - `code-graph (source code files)`
  - enriching via tree-sitter, LSPs and common sense
    - basic metadata for entities, files and edges via tree-sitter
    - entities with meta-data from LSPs like Rust-Analyzer and others wherever possible
      - dependency edges with meta-data from LSPs like Rust-Analyzer and others wherever possible
      - dependency edges with new relationships that we can creatively infer
      - 2 functions in the same file are likely to be related by common context sharing
      - 2 statements in the same file at public interface level are likely to be related by common context sharing

### Why This Big Rock Exists
We agreed the core product risk is not query UX or algorithm count.  
The core risk is silent or ambiguous ingestion: what got parsed, what got skipped, what was guessed, and what is actually true.

If this is weak, every downstream feature (blast radius, context packing, architecture checks) is weak.

### Binding Decisions (Current Draft)
**BR01-D1: Full file accountability is mandatory.**  
Every non-gitignored file must end in exactly one terminal classification:
1. `docs`
2. `non-eligible-text`
3. `identifiable-tests`
4. `code-graph`

No file can disappear from the pipeline without an explicit reason.

**BR01-D2: Accuracy tiering is explicit at fact level.**  
Every extracted fact must carry a truth grade:
1. `verified` (compiler/semantic tool grounded)
2. `heuristic` (pattern-based, confidence scored)
3. `rejected` (failed confidence or contradictory)

Only `verified` and high-confidence `heuristic` facts are queryable by default.

**BR01-D3: Provenance is first-class metadata.**  
Every file/entity/edge stores:
1. parser/tool used
2. parser version
3. extraction timestamp
4. confidence
5. degrade reason (if partial)
6. source anchor (path + line span when available)

**BR01-D4: Canonical identity stays stable across tools.**  
All tool outputs (tree-sitter, rust-analyzer, external analyzers) must map to one canonical entity address:
`language|||kind|||scope|||name|||file_path|||discriminator`

If a tool cannot map cleanly, its facts are not merged.

**BR01-D5: No silent merge of inconsistent facts.**  
Conflicting facts from different tools must be:
1. resolved by precedence rules (`verified` > `heuristic`)
2. or quarantined with conflict markers
3. never auto-presented as truth

**BR01-D6: Ingestion observability is a product surface, not debug-only.**  
Users can query coverage and status directly:
1. total files seen
2. files by terminal classification
3. parse success/failure by language/tool
4. fact quality distribution (`verified/heuristic/rejected`)
5. top degrade reasons

**BR01-D7: FUJ depends on this loop.**  
Final User Journey must start from ingestion visibility and confidence, then move to graph analysis.

### Proposed Data Contract (V200 Baseline)
Each file receives one immutable ingestion ledger row per run:

```text
IngestionLedgerRow {
  run_id: string,
  file_path: string,                  // normalized relative path
  gitignored: bool,
  terminal_class: enum,               // docs | non_eligible_text | identifiable_tests | code_graph
  language_detected: option<string>,
  parser_used: option<string>,
  parser_version: option<string>,
  extraction_status: enum,            // success | partial | failed | skipped
  verified_fact_count: u32,
  heuristic_fact_count: u32,
  rejected_fact_count: u32,
  degrade_reason: option<string>,
  started_at: datetime,
  finished_at: datetime
}
```

### Open Questions To Close Next
1. `OQ-BR01-1`: Should `identifiable-tests` also be dual-tagged under `code-graph` when test entities are extracted?
2. `OQ-BR01-2`: What is the default confidence threshold for heuristic facts to become query-visible?
3. `OQ-BR01-3`: Do we freeze a run snapshot per ingest or allow in-place mutation for always-on watch mode?
4. `OQ-BR01-4`: Which filetypes in `docs` are in-scope for doc-level entity extraction (`md`, `adoc`, `rst`, others)?

### Acceptance Criteria For Big-Rock-01
1. 100% of non-gitignored files appear in the ingestion ledger.
2. 0 silent drops (`seen_files == classified_files` invariant holds).
3. 100% of queryable edges/facts have provenance and truth grade.
4. Coverage endpoint returns deterministic totals for same run input.
5. Conflicting multi-tool facts are visible as conflicts, not hidden.

### FUJ v2 Simplification Impact
Once BR01 is locked, FUJ can be simplified into:
1. Setup
2. Ingest with observability proof
3. Query only trusted facts
4. Explain confidence on every answer

---

## Big-Rock-02: Query Trust Surface (Query-First Graph Design)
**Status**: Drafted for decision close  
**Date**: 2026-02-26  
**Intent**: Turn graph data into trustworthy, explainable answers with stable contracts.

### Scope Lock (Original Statement — Unchanged)
- BR02: Query-First Graph Design
- FIND ENTITIES
- TRACE DEPENDENCIES
- ANALYZE ARCHITECTURE
- Explore Arxiv and other common sense things knowing you have a code-graph heavily enriched with useful information

### Additive Note
The decisions below add response contracts and trust annotations only.  
They do not remove any endpoint family or reduce research/Arxiv exploration scope.

### Why This Big Rock Exists
Great ingestion without great query contracts still fails users.  
The user does not want "all data"; the user wants "answer I can trust in one screen."  
BR02 defines exactly what can be asked, how answers are shaped, and how confidence is exposed.

### Binding Decisions (Current Draft)
**BR02-D1: Query families are fixed and explicit.**
1. `Find` — discovery (`list`, `search`, `detail`)
2. `Trace` — dependency traversal (`callers`, `callees`, `blast_radius`)
3. `Analyze` — graph reasoning (`cycles`, `hotspots`, `SCC`, `centrality`, `clusters`, `debt`)

**BR02-D2: Every response is trust-annotated.**
1. `truth_grade` summary (`verified`, `heuristic`, `rejected`)
2. `confidence` summary and `uncertain=true` counts
3. provenance summary (tools that produced returned facts)

**BR02-D3: Default query mode is safe-by-default.**
1. include `verified` and high-confidence `heuristic`
2. hide `rejected` by default
3. include `uncertain` facts only on explicit opt-in

**BR02-D4: Single getter contract is non-negotiable (G2).**
All read paths (MCP, HTTP, CLI) pass through one store getter layer with identical semantics.

**BR02-D5: Entity detail is live-source grounded (G3).**
`/code-entity-detail-view` reads current disk source by path + line range.  
If file moved or unreadable, return explicit error contract (never stale cached source).

**BR02-D6: HTTP and MCP are parity surfaces.**
Transport differs; business semantics do not differ.  
One query contract, two adapters.

**BR02-D7: Source retrieval is pointer-based, not body-cached.**
1. Canonical graph storage keeps source addresses (`file_path`, `line_start`, `line_end`, optional `revision/hash`), not full code bodies.
2. `/code-entity-detail-view` resolves source on-demand from local filesystem for current workspace state.
3. If a pinned revision/hash is requested, retrieval may use `git show REV:PATH` semantics for deterministic historical reads.
4. If source moved, changed, or is unreadable, return explicit stale/missing-source contract (never silently return old cached text).

### Canonical Endpoint Pack (V200)
1. Find Entities
2. `GET /code-entities-list-all`
3. `GET /code-entities-list-all?entity_type=function`
4. `GET /code-entities-search-fuzzy?q=PATTERN`
5. `GET /code-entity-detail-view?key=ENTITY_KEY`
6. Trace Dependencies
7. `GET /dependency-edges-list-all`
8. `GET /reverse-callers-query-graph?entity=ENTITY_KEY`
9. `GET /forward-callees-query-graph?entity=ENTITY_KEY`
10. `GET /blast-radius-impact-analysis?entity=ENTITY_KEY&hops=N`
11. Analyze Architecture
12. `GET /circular-dependency-detection-scan`
13. `GET /complexity-hotspots-ranking-view?top=N`
14. `GET /semantic-cluster-grouping-list`
15. `GET /strongly-connected-components-analysis`
16. `GET /technical-debt-sqale-scoring`
17. `GET /kcore-decomposition-layering-analysis`
18. `GET /centrality-measures-entity-ranking?method=pagerank`
19. `GET /entropy-complexity-measurement-scores`
20. `GET /coupling-cohesion-metrics-suite`
21. `GET /leiden-community-detection-clusters`

### Acceptance Criteria For Big-Rock-02
1. Same query via MCP and HTTP returns semantically equivalent payload.
2. 100% query responses include trust and provenance summaries.
3. No endpoint bypasses single getter contract.
4. Entity detail returns live source or explicit file error, never stale body.
5. Endpoints are stable enough for FUJ and Tauri integration without per-release rewiring.

### Open Questions To Close Next (BR02)
1. `OQ-BR02-1`: What is the canonical retrieval strategy for V200 candidate generation before graph expansion?
   - Baseline flow under discussion: `top_k search candidates -> entity span resolution (file_path + start_line + end_line) -> dependency graph expansion -> selective source read -> LLM judgment`.
2. `OQ-BR02-2`: Do we require a dedicated search service/indexer, or is filesystem search (`rg`/BM25/fuzzy rerank) sufficient for V200 scope?
3. `OQ-BR02-3`: Where does cosine/vector similarity add measurable value versus lexical/fuzzy search for real user tasks?
4. `OQ-BR02-4`: What is the minimum metadata we store persistently (`entity key + span + edge + hash`) while keeping source bodies on disk only?
5. `OQ-BR02-5`: For large codebases, what is the acceptable indexing latency budget before graph+search utility is considered not worth the cost?
6. `OQ-BR02-6`: What is our formal "LSP for LLMs" contract for V200?
   - Candidate architecture under evaluation: `MCP transport layer + precise symbol/graph truth layer + metadata-first span retrieval (file_path/start_line/end_line)`.
   - Required decision: which part is mandatory in V200 core vs optional sidecar integration.
7. `OQ-BR02-7`: Should V200 include a control-flow layer for statement-order reasoning inside entities (not only dependency edges)?
   - Candidate edge set: `NEXT`, `BRANCH`, `LOOP`, `RETURN`, `THROW`.
   - Required decision: whether control-flow is V200 core scope or deferred after dependency-graph baseline.
8. `OQ-BR02-8`: Do we adopt a SCIP-like transmission contract for cross-tool interoperability while keeping Parseltongue storage/query runtime independent?
9. `OQ-BR02-9`: Do we integrate CocoIndex-style semantic search as a sidecar candidate generator tied to Parseltongue canonical entity keys?
   - Candidate stance: sidecar for discovery (`top_k spans`), not canonical truth replacement.
   - Required decision: V200 mandatory vs optional integration milestone.

### External Precedent Addendum (SCIP, sourcegraph/scip)
**Status**: Added for BR02 decision support  
**Date**: 2026-02-27

Key takeaways from SCIP codebase review:
1. SCIP is designed as a **transmission format**, not a query storage engine.
2. Source text is optional in index payloads; consumers are expected to read source from filesystem by root + relative path.
3. Path/range contracts are strict and explicit (canonical relative paths, half-open ranges, position encoding).
4. Symbol identity uses a formal grammar with canonical formatting/parsing to reduce ambiguity.
5. Relationship semantics are typed (`is_reference`, `is_implementation`, `is_type_definition`, `is_definition`) instead of one generic edge.
6. `enclosing_range` is used to provide local AST context around occurrences (useful for context slicing and call hierarchy-like UX).
7. Streaming, document-granular parsing is a first-class design choice for large codebases.
8. Quality gates (`lint`, `stats`, `snapshot`, `test`) are part of protocol health, not optional tooling.

Candidate carry-forwards for V200:
1. Keep Parseltongue source retrieval filesystem-first by default (no full source body persistence required in core graph store).
2. Freeze a canonical symbol format + parser/formatter contract and validate at ingest time.
3. Add typed relationship semantics to query surface and trust/provenance outputs.
4. Add enclosing-range metadata to improve selective context assembly for LLM workflows.
5. Add ingest quality gates equivalent to lint/canonicalize/snapshot checks before accepting index data as trusted.

### External Precedent Addendum (CocoIndex-style sidecar retrieval)
**Status**: Added for BR02/BR07 decision support  
**Date**: 2026-02-27

Summary:
1. A lightweight semantic MCP search layer can produce `file_path/start_line/end_line` candidates fast.
2. This is strong for direct discovery and natural-language recall.
3. It is not sufficient alone for dependency-truth reasoning (no canonical graph contract by default).

Carry-forward for V200:
1. Use sidecar search to generate top candidate spans.
2. Resolve those spans to canonical Parseltongue entity keys.
3. Expand via dependency graph and trust-grade filters before LLM output.

### Indexing Time Expectations (planning ranges; verify with benchmark probes)
Assumptions:
1. Medium/high-end developer machine.
2. AST/chunking + embeddings + local SQLite/vector store.
3. Embedding provider latency dominates when remote API is used.

Cold index (first run):
1. ~100k LOC: ~2-8 minutes
2. ~500k LOC: ~10-45 minutes
3. ~1M+ LOC: ~20-90 minutes

Incremental index (typical daily edits):
1. <100 changed files: ~3-60 seconds
2. ~100-500 changed files: ~1-5 minutes
3. Full refactor/branch switch: can approach cold-index range

Candidate V200 SLO targets to evaluate:
1. Incremental update p95 under 60 seconds for <100 changed files.
2. Cold index under 30 minutes for ~500k LOC with local embeddings.
3. Explicit degrade messaging when embedding provider latency exceeds SLO envelope.

### External Research Addendum (Entity Search + Context Retrieval)
**Status**: Added for BR02/BR07 decision support  
**Date**: 2026-02-28  
**Source Note**: `/Users/amuldotexe/Desktop/notebook-gh/Notes2026/deep-research-entity-search-context-202602280026.md`

Key carry-forwards from the 2026-02-28 research note:
1. Entity boundary extraction is tractable with Tree-sitter/LSP spans and should remain metadata-first (`file_path`, `line_start`, `line_end`, entity type/name/signature/docstring).
2. Pure vector similarity is insufficient for V200-quality retrieval; candidate ranking should be multi-signal.
3. Dependency-graph proximity should be a first-class rerank signal after initial candidate generation.
4. Context payload should stay compact: top entity packet + a small related-neighborhood, never whole-file default.
5. Indexing economics must be tiered by repository size and usage pattern (one-off vs repeated multi-session use).
6. Incremental updates (Merkle/hash-based changed-file detection) are mandatory if any semantic sidecar index is adopted.

Proposed V200 retrieval contract extension:
1. Stage A (`retrieve`): lexical-first (`rg`/BM25/fuzzy) plus optional semantic sidecar to produce top-N candidate spans.
2. Stage B (`resolve`): map each span to canonical Parseltongue entity key.
3. Stage C (`rerank`): combine weighted signals:
   - lexical overlap
   - semantic similarity (when enabled)
   - graph proximity (call/import/type edges)
   - entity-type intent match
   - reference/popularity score
4. Stage D (`assemble`): return top-3 entity packets and max 3-5 related entities with explicit confidence + provenance.

Proposed response packet baseline (for BR02 APIs and MCP parity):
1. `entity_key`
2. `entity_name`
3. `entity_type`
4. `file_path`
5. `line_start`
6. `line_end`
7. `why_ranked` (signal contribution summary)
8. `confidence`
9. `truth_grade`
10. `related_entities[]` (bounded list)

Open questions to close from this addendum:
1. `OQ-BR02-10`: What are the default rerank weights and how are they calibrated against a benchmark query set?
2. `OQ-BR02-11`: Is semantic/vector search V200 core or an optional sidecar behind capability flags?
3. `OQ-BR02-12`: What is the default neighborhood expansion budget (`top_k` + `hops` + token cap) for LLM context assembly?
4. `OQ-BR07-1`: What repo-size threshold triggers semantic indexing recommendation vs lexical-only mode?
5. `OQ-BR07-2`: Which incremental index freshness SLO is required to keep reranking trustworthy after code changes?

### External Research Addendum (CocoIndex Engine vs Codemogger Turso Storage)
**Status**: Added for BR02/BR07 decision support  
**Date**: 2026-02-28  
**Source Notes**:
1. `/Users/amuldotexe/Downloads/cocoindex-io-cocoindex-code-8a5edab282632443.txt`
2. `/Users/amuldotexe/Downloads/glommer-codemogger-8a5edab282632443.txt`

Decision clarification:
1. CocoIndex is an indexing/transformation engine runtime, not an RLM and not a canonical graph truth system by itself.
2. In the observed `cocoindex-code` implementation, search rows include `file_path`, `start_line`, `end_line`, `embedding`, and cosine score in SQLite/sqlite-vec.
3. In the observed codemogger implementation, storage uses Turso/libSQL-flavored SQLite with chunk rows including `chunk_key`, `file_path`, `start_line`, `end_line`, `embedding`, and `file_hash`.

Comparative framing for V200:
1. CocoIndex-sidecar method strengths:
   - fast path to semantic candidate spans (`file_path:start_line:end_line`)
   - local-first workflow (simple developer setup)
   - clear fit as non-canonical retriever before graph resolution
2. CocoIndex-sidecar method risks:
   - Python/runtime integration boundary with Rust core
   - pre-release dependency surface in current ecosystem examples
   - requires explicit staleness/consistency checks before graph trust
3. Codemogger/Turso-style method strengths:
   - persistent SQL layer with chunk-level metadata, hashes, and strong incremental mechanics
   - easier team/remote sharing if Turso sync/deployment is used
   - single query surface for FTS + vector + metadata filters
4. Codemogger/Turso-style method risks:
   - more infra and product surface area for V200 core scope
   - remote DB dependence can increase latency and operational complexity
   - temptation to drift into body-caching/data-warehouse behavior

Primary-key compatibility decision:
1. Both approaches can store Parseltongue canonical identity (`language|||kind|||scope|||name|||file_path|||discriminator`) as metadata.
2. Required addition for either store:
   - `entity_key` (canonical)
   - `file_hash` (staleness guard)
   - optional `entity_version_hash` (deterministic revision pin)
3. For chunk-to-entity mapping, prefer explicit mapping table:
   - `chunk_key -> entity_key` (many-to-one baseline, many-to-many allowed for overlap)

Recommended V200 call (current draft):
1. Keep Parseltongue graph store as canonical truth layer.
2. Adopt semantic retrieval as optional sidecar first (CocoIndex-style), returning spans only.
3. Resolve spans to canonical entity keys before any graph reasoning output.
4. Defer Turso-backed shared retrieval store to post-V200 unless collaboration scale requires it immediately.
5. Never bypass BR01 truth-grade/provenance contracts regardless of retrieval backend.

Metadata-only storage refinement (accepted direction for evaluation):
1. If Turso/libSQL retrieval index is used, store pointer metadata and ranking features, not full source bodies.
2. Minimum pointer schema:
   - `entity_key`
   - `chunk_key`
   - `file_path`
   - `start_line`
   - `end_line`
   - `file_hash` (or VCS blob hash)
   - retrieval features (`embedding`, `fts_terms`, `score_aux`)
3. Source text is resolved on-demand from filesystem or pinned VCS revision using stored pointers.
4. If hash mismatch is detected at read time, mark candidate stale, trigger reindex for that file/chunk, and degrade confidence until refreshed.
5. `verified` truth-grade answers must not be emitted from stale pointer rows.

Open questions to close from this addendum:
1. `OQ-BR02-13`: Do we ship V200 with local-only semantic sidecar by default and Turso as explicit opt-in?
2. `OQ-BR02-14`: What is the canonical schema for `chunk_key -> entity_key` mapping and overlap handling?
3. `OQ-BR07-3`: What p95 latency budget must sidecar retrieval meet (local vs remote Turso mode) to remain default-enabled?
4. `OQ-BR07-4`: What freshness SLO and hash mismatch behavior are mandatory before returning sidecar-derived candidates?

### External Research Addendum (Lucene/ES Data-Flow Limits + Architecture Rubber-Duck)
**Status**: Added for BR02/BR03/BR07 decision support  
**Date**: 2026-02-28  
**Source Notes**:
1. `docs/ACTIVE-Reference/06-LUCENE-ES-FOR-DATA-FLOW-ANALYSIS.md`
2. `docs/ACTIVE-Reference/07-ARCHITECTURE-RUBBER-DUCK-DEBUG.md`

Thesis (integration direction):
1. Search systems are strong for retrieval, weak for program analysis.
2. Graph systems are required for dependency/data/control-flow reasoning.
3. Parseltongue should explicitly separate:
   - Retrieval layer (candidate generation/ranking)
   - Analysis layer (graph traversal, blast radius, semantic relationships)
4. Product moat should be "context extraction intelligence" (what minimal context an LLM needs for a task), not just storage choice.

Options under evaluation:
1. `OPT-V200-RETRIEVAL-A` — In-core lexical retrieval upgrade:
   - Add BM25 + Jaccard + RRF fusion locally in Rust.
   - Keep optional vectors as sidecar.
   - Strength: low infra complexity, deterministic local behavior.
2. `OPT-V200-RETRIEVAL-B` — Sidecar semantic retrieval:
   - Use CocoIndex/Turso-style chunk retrieval for top-N spans.
   - Resolve to canonical `entity_key` before graph analysis.
   - Strength: strong natural-language discovery and fast adoption.
3. `OPT-V200-RETRIEVAL-C` — Intelligent context server path:
   - Multi-depth extraction (tree-sitter always, semantic depth where available).
   - Rule-based context extraction with optional LLM fallback for novel tasks.
   - Strength: highest differentiation if evaluated and tuned well.
4. `OPT-V200-INTERFACE-D` — Dual interface mode:
   - Context injection for zero-friction workflows.
   - MCP/HTTP explicit tools for deterministic operator control.

Open questions to close from this addendum:
1. `OQ-BR02-15`: Do we implement BM25/Jaccard/RRF in-core for V200 baseline, or require sidecar retrieval from day one?
2. `OQ-BR02-16`: Which retrieval+analysis split is mandatory in V200 API contracts (and which is implementation detail)?
3. `OQ-BR03-1`: Should "context extraction intelligence" be a formal BR03 scope item with acceptance tests per task type?
4. `OQ-BR03-2`: Do we add task-intent classes (e.g., signature change, field add, trait impl) that drive deterministic context rules?
5. `OQ-BR07-5`: Which freshness model is V200 default — static snapshots, watch+incremental, or hybrid with explicit stale markers?
6. `OQ-BR07-6`: What benchmark suite is required to validate the token-compression claim (e.g., equivalent accuracy at lower context budget)?
7. `OQ-BR07-7`: What is V200 success threshold for first-pass correctness improvement (compile/test pass rate) versus naive file-reading workflows?

### External Research Addendum (Rust-Analyzer Idiomatic Pattern Pack)
**Status**: Added for BR01/BR02/BR03/BR07 decision support  
**Date**: 2026-03-01  
**Source Folder Copied**: `docs/ACTIVE-Reference/rust-analyzer/`

Per-file insight items (one per copied file):
1. `01-hir-ty-patterns.md`: Cycle-safe incremental query design is a hard requirement for semantic depth; V200 option is to formalize cycle-recovery metadata in query provenance. `OQ-RA-01`: Which query families require explicit cycle fallback in V200 core?
2. `02-hir-def-patterns.md`: Interned definition identities map directly to Parseltongue key-stability goals; option is to enforce canonical key derivation from definition-loc tuples. `OQ-RA-02`: Which loc fields are mandatory for deterministic key derivation across tools?
3. `03-syntax-parser-patterns.md`: Typed wrappers over untyped trees reduce parser ambiguity; option is a strict typed AST adapter boundary before graph writes. `OQ-RA-03`: Should untyped parse results ever be query-visible outside diagnostics?
4. `04-ide-features-patterns.md`: Snapshot + cancellation patterns should shape MCP/HTTP consistency semantics; option is immutable analysis snapshots per request. `OQ-RA-04`: What is the snapshot isolation contract across concurrent requests?
5. `05-ide-assists-patterns.md`: Structured assist handlers are a template for future write-actions; option is to define mutation-intent handlers as typed contracts, even if V200 stays read-first. `OQ-RA-05`: Do we reserve an assists-style contract in V200 for V217+ safe edits?
6. `06-ide-completion-patterns.md`: Two-phase speculative completion suggests retrieval should support hypothesis-first ranking; option is staged candidate expansion before expensive reads. `OQ-RA-06`: Which query intents justify speculative candidate generation in V200?
7. `07-ide-db-patterns.md`: Symbol index + configurable query builders align with retrieval configurability; option is explicit retrieval profiles (`fast`, `balanced`, `deep`). `OQ-RA-07`: What are the default profile thresholds and tuning knobs?
8. `08-hir-facade-expand-patterns.md`: Bidirectional source mapping is essential for line-span trust; option is mandatory syntax<->entity backpointers in all queryable facts. `OQ-RA-08`: Is reverse mapping coverage a release gate for V200?
9. `09-lsp-server-patterns.md`: Event-loop dispatcher + typed routing informs protocol adapter design; option is one internal command bus for MCP/HTTP parity. `OQ-RA-09`: Do we standardize one dispatcher model before adding new endpoints?
10. `10-vfs-base-db-patterns.md`: Path canonicalization and bounded file IDs are critical for stable hashes/keys; option is path normalization policy as a first-class ingest contract. `OQ-RA-10`: What is the canonical normalization spec across macOS/Linux path edge cases?
11. `11-project-model-patterns.md`: Workspace discovery fallback logic should be deterministic and observable; option is ordered project-root resolution with ledger traces. `OQ-RA-11`: Which root-discovery precedence order is frozen for V200?
12. `12-token-tree-macro-patterns.md`: Macro/token-tree span compression indicates macro-aware spans need dedicated storage shapes; option is macro-expansion provenance edges separate from regular call edges. `OQ-RA-12`: What macro-derived facts qualify as `verified` versus `heuristic`?
13. `13-utility-crates-patterns.md`: Invariant-oriented utility patterns suggest stronger type-level guarantees in core contracts; option is non-empty and constrained wrappers in API schemas. `OQ-RA-13`: Which response fields should be modeled as non-empty by type, not convention?
14. `14-lib-arena-lsp-patterns.md`: Arena/sparse-map patterns support custom high-performance graph memory layouts; option is to benchmark arena-based in-memory indices for hot traversal paths. `OQ-RA-14`: Which graph queries warrant arena specialization in V200 timeframe?
15. `15-ide-diagnostics-patterns.md`: Diagnostics taxonomies map cleanly to truth-grade/degrade reasons; option is a strict diagnostic-code namespace for ingest/query errors. `OQ-RA-15`: What minimum diagnostic code set is mandatory for V200 operator trust?
16. `16-proc-macro-server-patterns.md`: Versioned external process protocols inform sidecar capability negotiation; option is explicit sidecar capability/version handshake before use. `OQ-RA-16`: Which capability mismatches should hard-fail versus soft-degrade?
17. `17-test-infrastructure-patterns.md`: Fixture mini-DSL and minimal-core stubs are strong evaluation primitives; option is a standardized fixture harness for retrieval+graph regressions. `OQ-RA-17`: Which benchmark fixture corpus becomes the canonical V200 quality gate?
18. `18-xtask-codegen-patterns.md`: Tooling/codegen pipelines can prevent contract drift; option is automated generation/validation for endpoint schemas and key contracts. `OQ-RA-18`: Which contracts are code-generated versus handwritten in V200?
19. `19-cross-cutting-architecture-patterns.md`: Layered DB traits and cache budgets align with BR07 performance governance; option is explicit cache-capacity policy with telemetry per layer. `OQ-RA-19`: What cache budgets are fixed defaults for medium/large repos?
20. `20-ssr-span-cfg-patterns.md`: Two-phase matching and span-focused workflows reinforce metadata-first retrieval; option is enforce span-first context assembly before full entity reads. `OQ-RA-20`: Which tasks may bypass span-first and read full entities immediately?
21. `ANALYSIS_STATE.md`: Meta-analysis tracking should become a recurring quality ledger; option is a decision-log companion state file with readiness scoring per architecture option. `OQ-RA-21`: Do we formalize a readiness scorecard for BR decisions before implementation starts?

### External Research Addendum (Three-Layer Retrieval-to-Graph Context Loop)
**Status**: Added for BR02/BR03 decision support  
**Date**: 2026-03-01

Decision thesis:
1. V200 query flow should be a strict 3-layer contract:
   - Layer 1 (`search`): find top candidate spans from lexical/fuzzy (and optional semantic sidecar) search.
   - Layer 2 (`wrap`): map each span to canonical entity key, then recover enclosing interface entity + local control/data-flow clues.
   - Layer 3 (`expand`): pull related entities from dependency graph and assemble a token-budgeted context packet (default `<10k` tokens).
2. The LLM should never receive whole-file default context when a span/entity packet is available.
3. Returned packet should prioritize "high-context-per-token":
   - direct entity span
   - why this entity was selected
   - direct callers/callees + critical type/impl links
   - minimal code excerpts required for a decision

Three creative examples (operator-visible workflow):
1. Example A — "Bug from vague symptom text"
   - User prompt: "Why does login expire immediately after deploy?"
   - Layer 1 finds spans matching symptom terms (`expire`, `ttl`, `clock skew`, `jwt`) across config + auth code.
   - Layer 2 wraps to canonical entity keys (for example: token validation fn, env config loader), and identifies the exact branch where TTL is interpreted.
   - Layer 3 returns compact packet: validator entity + config entity + one test entity + nearest callers, with control-flow branch evidence and confidence.
2. Example B — "Intent-first feature change"
   - User prompt: "Add dry-run mode to deploy command."
   - Layer 1 finds spans in CLI parsing, deploy orchestration, and side-effecting execution calls.
   - Layer 2 maps to enclosing interface entities and marks write-path vs read-path control flow.
   - Layer 3 returns under-10k packet containing parse->plan->execute chain, affected interfaces, and blast radius of entities likely to break.
3. Example C — "Cross-cutting rename without reading whole repo"
   - User prompt: "Rename customer_tier to subscription_plan safely."
   - Layer 1 finds textual spans in API schema, validation, persistence, and analytics usage.
   - Layer 2 resolves each span to owning entities and classifies role (`definition`, `read`, `write`, `serialization`).
   - Layer 3 returns a migration packet with ordered related entities (schema first, adapters second, call-sites third) plus confidence and stale-hash warnings.

Open questions introduced:
1. `OQ-BR02-17`: What is the default top-k span count per query before wrap+graph expansion?
2. `OQ-BR02-18`: Which control/data-flow signals are mandatory in V200 context packets vs optional enrichments?
3. `OQ-BR03-3`: What is the strict token budgeting policy for context packets (`entity core`, `graph neighbors`, `evidence`) under `<10k` tokens?

### External Research Addendum (Rubber-Duck 3-Layer Context Compressor Thesis)
**Status**: Added for BR02/BR03/BR07 decision support  
**Date**: 2026-03-01

Carry-forward thesis:
1. Parseltongue should be framed as a semantic context compressor, not as a raw code retrieval dump system.
2. The 3-layer flow remains mandatory:
   - `Layer 1 (search)`: free-form query -> semantic anchors (non-keyword identifiers) -> top candidate spans/entities.
   - `Layer 2 (anchor)`: bind candidate to canonical entity + containment context (`module`, `signature`, `file_path:start_line:end_line`).
   - `Layer 3 (expand)`: typed graph traversal with budget controls to return only high-context neighbors.
3. High-density context defaults:
   - signatures over bodies
   - typed relationships over raw text
   - focused call/data-flow slices over full graphs
4. Whole-file reads are fallback behavior only, never default.

Retrieval and expansion priority contract (candidate draft):
1. Priority P1 (always include when available):
   - `calls`
   - `uses_type`
   - `implements`
2. Priority P2 (include if budget allows):
   - `called_by`
   - `contains`
   - `same_module`
3. Priority P3 (deep context, intent-gated):
   - `transitive_calls`
   - `trait_hierarchy`
   - `control_flow`/`data_flow` slices

Proposed rerank and confidence model:
1. Candidate score is weighted multi-signal, not pure fuzzy or pure cosine:
   - `score = w_lexical + w_semantic + w_graph_proximity + w_entity_type_intent + w_freshness`
2. Freshness is first-class:
   - stale hashes reduce score and confidence
   - stale-only sets cannot produce `verified` answers
3. Missing-confidence behavior:
   - return top spans + uncertainty markers + next-query suggestions
   - do not force a deterministic-looking answer

Token budget policy (draft baseline):
1. Target context packet under `10k` tokens, with expected practical range `500-4k`.
2. Suggested split:
   - `entity anchor + signature`: 5-15%
   - `P1 relationships`: 40-50%
   - `P2 relationships`: 20-30%
   - `P3 relationships`: 10-20%
   - `format/provenance overhead`: 5-10%
3. Stop expansion on either budget exhaustion or relevance-threshold drop.

V216/V200 architecture direction reinforced:
1. rust-analyzer already computes most needed structure (`ItemTree`, `DefMap`, trait impl maps).
2. V200/v216 core work is extraction + canonicalization + persistence + query contract, not re-implementing compiler semantics.
3. Control-flow/data-flow should start as selective slices tied to anchor intent, not full-program IR ambitions.

Open questions introduced:
1. `OQ-BR02-19`: What is the anchor-extraction algorithm for "non-keyword semantic tokens" from natural-language and code-fragment queries?
2. `OQ-BR02-20`: What are the initial production weights for multi-signal scoring, and how are they calibrated against benchmark tasks?
3. `OQ-BR03-4`: Which edge types are mandatory in P1 for each intent class (`bug`, `refactor`, `explain`, `migrate`)?
4. `OQ-BR07-8`: What freshness SLO is required before semantic candidates are allowed into default response packets?
5. `OQ-BR07-9`: What minimum information-density KPI do we enforce (for example, compression ratio and task success lift)?

### External Research Addendum (V216 Semantic Context Compressor Thesis, V2 Upgrades)
**Status**: Added for BR02/BR03/BR07 decision support  
**Date**: 2026-03-01  
**Source Note**: `docs/ACTIVE-Reference/14-V216-SEMANTIC-CONTEXT-COMPRESSOR-THESIS.md`

Key decisions/options extracted:
1. Retrieval ranking should be query-intent aware, not one global static formula.
2. The V2 rerank formula is promoted as candidate baseline:
   - `score = lexical + semantic + graph_proximity + entity_type_intent + freshness`
3. V200 should ship confidence-gated search outcomes:
   - High confidence (`>=0.80`): auto-proceed to Layer 2 anchor.
   - Medium confidence (`>=0.50` and margin gate): proceed with explicit uncertainty marker.
   - Low confidence (`<0.50` or weak margin): return candidate set + clarification/follow-up suggestions.
4. Token packing should be staged with explicit value-per-token prioritization:
   - Stage 1: anchor packet
   - Stage 2: essential neighbors
   - Stage 3: contextual neighbors
   - Stage 4: optional deep edges
5. Progressive disclosure must be a first-class response mode:
   - default compact packet
   - on-demand expansion of call/data-flow slices
6. Rust-analyzer extraction remains the architecture center:
   - `ItemTree` for signatures/containment
   - `DefMap`/`PerNs` for scope/visibility
   - trait/inherent impl maps for type capability edges
7. Explicit non-goals remain valid for V200:
   - full type inference persistence
   - full macro pipeline recreation
   - full LSP server/event-loop reimplementation

Proposed V200 contract extensions from this thesis:
1. Introduce a typed retrieval outcome enum:
   - `resolved_high_confidence`
   - `resolved_medium_confidence`
   - `ambiguous_candidates`
   - `no_confident_match`
2. Introduce margin-aware ambiguity contract:
   - include `top_score`, `second_score`, and `score_gap`
   - include per-candidate `why_ranked` explanation
3. Introduce value-per-token telemetry:
   - response stores selected packet items and dropped items with reason (`budget`, `low_relevance`, `stale`)

Open questions introduced:
1. `OQ-BR02-21`: What are V200 default high/medium/low confidence thresholds and score-gap margins?
2. `OQ-BR02-22`: Should intent-aware rerank profiles (`bug`, `explain`, `refactor`, `migrate`) be part of V200 core or behind feature flag?
3. `OQ-BR02-23`: What exact schema do we use for `why_ranked` and ambiguity payloads to keep MCP/HTTP parity?
4. `OQ-BR03-5`: Which deterministic evidence is minimum-required before medium-confidence responses can be used for LLM action suggestions?
5. `OQ-BR07-10`: What value-per-token KPI should gate regressions in context packing quality?

### External Research Addendum (rustc Release-Cadence Risk Assessment for V200)
**Status**: Added for BR01/BR03 decision support  
**Date**: 2026-03-02  
**Intent**: Resolve: "If Parseltongue is a static tool and we stay on rustc-relevant versions, how serious is API break risk?"

Release-tag evidence used (not `main` churn):
1. `1.90.0 -> 1.91.0`: `rustc_public` exported-signature removals = `3`
2. `1.91.1 -> 1.92.0`: removals = `1`
3. `1.92.0 -> 1.93.0`: removals = `10`
4. `1.93.0 -> 1.93.1`: removals = `0`
5. Removals are concentrated (Pareto-like), not uniform; majority observed in `compiler/rustc_public/src/ty.rs`.

Assessment (decision-quality summary):
1. This is **not** "everything breaks every release."
2. This is also **not** "zero risk."
3. Patch releases are usually low-risk; minor releases carry most compatibility movement.
4. A static binary does not imply static compiler-internal API compatibility over time.
5. **Pinned-version clarifier**: while the toolchain and adapter are pinned (no upgrade in progress), APIs are operationally stable for that running version; brittleness is primarily an **upgrade-time** risk, not a runtime-query risk.

Decision direction (draft to execute):
1. **Version-coupled support policy**: support `N` and `N-1` stable toolchains only; reject unknown versions explicitly.
2. **Adapter boundary mandate**: isolate rustc-facing extraction behind one compatibility adapter so graph/query contracts stay stable.
3. **Risk tiering for planning**:
   - pinned exact stable version: low operational risk
   - `N/N-1` stable coverage: medium recurring adapter work
   - nightly tracking: high risk (not default for V200)
4. **Graceful degradation contract**: if rustc capability mismatch occurs, downgrade to parsable subset + explicit `partial` capability marker, never silent success.
5. **Strategic positioning note**: retaining a real rustc deep mode is also a deliberate capability signal — proving Parseltongue can operate at true compiler depth, not only at syntax/LSP depth.

Open questions introduced:
1. `OQ-BR01-18`: What exact V200 toolchain matrix do we publish (`N` only vs `N/N-1`)?
2. `OQ-BR03-7`: What is the minimum adapter conformance test suite required before certifying a new rustc minor version?
3. `OQ-BR07-12`: What SLA do we commit for new stable rustc support after release day (for example, `<7 days`)?

### External Research Addendum (Context-Query-Converge Score Architecture + Profile Variations)
**Status**: Added for BR02/BR03/BR07 decision support  
**Date**: 2026-03-02  
**Intent**: Consolidate recent research journals/thesis notes into one executable scoring model for the single endpoint workflow.

Single-endpoint contract direction:
1. Keep one user-facing endpoint: `POST /context-query-converge`.
2. Keep one state machine for all clients (human CLI + agents):
   - `resolved`
   - `disambiguate`
   - `no_match`
3. Keep progressive disclosure as the default UX:
   - quick graph computation first
   - option-card disambiguation when needed
   - deep dive only for selected/auto-resolved cluster

Why this direction is reinforced:
1. The thesis/workflow notes prioritize ranked option cards + scoped deep dives over up-front intent questionnaires and over full deep-dive-on-all candidates.
2. Research synthesis indicates combined structural representations outperform single-mode retrieval.
3. Build/buy guidance supports centrality/community/impact algorithms now, while deferring heavy GNN-first architectures.

Unified score model (draft baseline):
1. `score(candidate)` should be a weighted multi-signal composition:
   - `search_fusion` (`exact_symbol` + `fuzzy` + `RRF`)
   - `anchor_quality` (public-interface anchor quality)
   - `structural_relevance` (call/type/control/data-flow proximity)
   - `centrality_importance` (PageRank, betweenness, k-core)
   - `community_fit` (Leiden/module cohesion)
   - `impact_relevance` (blast radius / influence propagation)
   - `temporal_relevance` (recent changes/version-locality, when available)
   - `freshness`
   - `provenance_confidence`
2. Apply hard penalties:
   - stale hash / stale index
   - weak or missing provenance
   - token-overflow pressure
   - low margin ambiguity
3. Use margin-aware finalization:
   - auto-resolve only when confidence and score-gap gates are satisfied
   - otherwise return compact `disambiguate` payload with 2-4 labeled clusters

Algorithm priority for V200 implementation:
1. **P0 (ship first):**
   - exact symbol lookup + trigram fuzzy + `RRF`
   - BFS anchoring to nearest public interface
   - ego-network (`k`-hop) cluster extraction
   - PageRank + betweenness for ranking
   - influence/BFS blast-radius expansion
   - token-budget context selector (value-per-token)
   - freshness + provenance gating before `resolved`
2. **P1 (next iteration):**
   - Leiden community detection for option-card grouping
   - k-core as dense-core signal
   - shortest-path distance as proximity feature
   - lightweight temporal recency signal (change-aware scoring)
3. **P2 (defer):**
   - GNN-based rankers as default scoring path
   - full CPG-first mandatory pipeline for all queries

Profile-driven weight shifts (same endpoint, different policy):
1. `balanced`: even blend of search + anchor + structure + importance.
2. `auto_fast`: high exact/anchor weights with strict auto-resolve gap.
3. `safe_strict`: high freshness/provenance/impact, conservative auto-resolve.
4. `debug_error`: boost control/data-flow and local blast radius around error seeds.
5. `refactor_impact`: maximize reverse-callers + impact propagation + core-density cues.
6. `learn_codebase`: maximize community coverage and representative central nodes.
7. `pr_review`: add recency/change-locality and dependency-risk emphasis.
8. `symbol_direct`: near-direct resolution when exact canonical symbol exists.
9. `compare_two`: preserve top-2 clusters when margin is narrow.
10. `agent_json`: deterministic ordering + explicit score breakdown + provenance fields.

Draft guardrails:
1. Do not claim "zero hallucination"; claim evidence-backed confidence.
2. No `resolved` outcome without freshness/provenance gates passing.
3. Always emit `why_ranked` fields for top candidates in `disambiguate`.
4. Default option-card count remains `2-4` for decision speed.

Open questions introduced:
1. `OQ-BR02-26`: What default weight vector should V200 ship for `balanced` profile?
2. `OQ-BR02-27`: What exact auto-resolve thresholds (`top_score`, `score_gap`, freshness floor) should gate `auto_fast`?
3. `OQ-BR03-8`: What minimum structural evidence bundle is required per profile before returning `resolved`?
4. `OQ-BR07-13`: What p95 latency budget per phase is acceptable while preserving 2-4 candidate disambiguation quality?

---

## Big-Rock-03: Compiler Truth + LLM Judgment Loop
**Status**: Drafted for decision close  
**Date**: 2026-02-26  
**Intent**: Use deterministic semantic tools for facts, and reserve LLM for judgment.

### Scope Lock (Original Statement — Unchanged)
- BR03: Bi-Directional Workflow reference where LLM queries the above list of APIs to generate insights

### Additive Note
The decisions below clarify where deterministic extraction ends and LLM interpretation begins.  
They are an expansion of BR03, not a narrowing of BR03.

### Why This Big Rock Exists
LLMs are strong at interpretation, weak at pretending to be compilers.  
If we let the LLM guess types/dispatch/ownership when tools can prove them, we lose trust and speed.

### Binding Decisions (Current Draft)
**BR03-D1: Deterministic-first rule.**  
Questions with objectively correct answers must be answered by compiler/LSP/semantic engines first.

**BR03-D2: LLM is called for ambiguity and prioritization, not base extraction.**  
Examples: cycle intentionality, refactor prioritization, migration strategy ordering.

**BR03-D3: Judgment outputs must cite evidence.**  
Every LLM judgment payload includes supporting entity keys, edges, and confidence slices.

**BR03-D4: Typed semantic edges are first-class for Rust.**  
Direct/TraitMethod/DynDispatch/ClosureInvoke must flow into graph reasoning paths.

**BR03-D5: If deterministic evidence is unavailable, output must degrade explicitly.**  
Use capability markers (`full`, `partial`, `heuristic`) instead of pretending certainty.

### Acceptance Criteria For Big-Rock-03
1. No LLM-only path can emit `verified` truth grade without deterministic evidence.
2. Judgment endpoints always return supporting evidence keys.
3. Rust semantic queries show typed edge kinds in output.
4. Ambiguous cases are tagged clearly, not silently averaged.
5. Response time for judgment workflows is bounded by using pre-computed evidence, not raw code rereads.

---

## Big-Rock-04: Tauri App Operator Surface
**Status**: Drafted for decision close  
**Date**: 2026-02-26  
**Intent**: Build Tauri as a clean operator UI to manage and monitor Parseltongue ingestion/processes.

### Scope Lock (Original Statement — Unchanged)
- BR04: Tauri App as a UI to manage and monitor parseltongue ingestion across your system

### Additive Note
The section below operationalizes BR04 only.  
No language-depth work is included in BR04.

### Why This Big Rock Exists
If runtime operations are fragile, product trust collapses regardless of graph quality.  
Tauri should reduce friction, not become a second product to maintain.

### Binding Decisions (Current Draft)
**BR04-D1: Tauri is instance manager only.**  
No graph explorer scope in V200 Tauri.

**BR04-D2: Three core actions only.**
1. Start/Stop HTTP server per workspace
2. Write MCP config entry
3. Show exact CLI commands for ingest/serve

**BR04-D3: Ingestion monitoring is first-class in Tauri.**
1. current run state
2. coverage totals from BR01
3. parse/degrade error summaries
4. process port and lifecycle status

**BR04-D4: Every UI action has a CLI equivalent.**  
Power users can graduate from UI to terminal without hidden logic.

**BR04-D5: Tauri must never become an analysis engine.**  
All heavy graph logic stays in gateway/store/reasoning crates.

### Acceptance Criteria For Big-Rock-04
1. A new workspace can be launched, connected, and queried without manual log hunting.
2. Process crash/death is detected and reflected in UI state.
3. MCP config write path is deterministic and auditable.
4. Users can copy/paste CLI commands from Tauri and reproduce behavior exactly.
5. No graph algorithm logic lives in Tauri client layer.

---

## Proposed Big-Rock-05: Tiered Language Depth for Rust Migration
**Status**: Drafted for decision close  
**Date**: 2026-02-26  
**Intent**: Build the deepest truthful graph for languages that matter for Rust rewrite journeys.

### Scope Note
Original BR05 was open/blank.  
This is a proposed fill for BR05 focused on migration-grade language capability depth.

### Why This Big Rock Exists
Your end goal is Rust migration, not generic language vanity coverage.  
So capability depth must follow migration value, not feature count.

### Binding Decisions (Current Draft)
**BR05-D1: Tiered depth is mandatory and explicit.**
1. Tier 1: Rust (`.rs`) — full semantic depth + highest confidence
2. Tier 2: TypeScript, JavaScript, C, C++, Ruby, Rails — medium-to-high depth where tooling supports it
3. Tier 3: remaining supported languages — structural extraction + explicit capability limits

**BR05-D2: Dataflow is capability-scored per language.**
1. Rust: full target in V200
2. Tier 2: partial where symbol resolution works
3. Tier 3: heuristic only, default hidden unless opted in

**BR05-D3: Migration-focused outputs are required.**
1. cross-language boundary map (HTTP/FFI/WASM/PyO3/Ruby FFI)
2. API contract mismatch report across Rust and non-Rust nodes
3. blast-radius + public-module-context to estimate rewrite impact
4. capability report so users know exactly what to trust per language

**BR05-D4: No language gets fake precision.**
When tooling cannot prove semantics, output must remain heuristic and visibly marked.

### Acceptance Criteria For Big-Rock-05
1. Every language in ingest has a declared capability tier in output metadata.
2. Tier 2 languages produce useful migration-grade relationship edges with confidence.
3. Tier 3 languages never appear as high-confidence semantic truth by accident.
4. Rewrite planning queries can cross Rust and Tier 2 codebases in one graph walk.
5. FUJ examples include at least one cross-tier migration trace.

---

## Proposed Big-Rock-06: External Evidence Federation (Truthful-by-Construction)
**Status**: Drafted for decision close  
**Date**: 2026-02-26  
**Intent**: Ingest strong external analyzer signals without corrupting canonical graph truth.

### Scope Note
This is additive after BR01-BR05 and can be sequenced after core ingestion/query contracts.

### Why This Big Rock Exists
There is rich value in external analyzers (Semgrep, CodeQL, Brakeman, clang tooling),  
but blind merge of their output into core edges creates false confidence.

### Binding Decisions (Current Draft)
**BR06-D1: External findings enter as evidence, not canonical truth.**
Default representation is `evidence` relation with provenance fields.

**BR06-D2: Canonical promotion is gated.**
Promote evidence to first-class edge only if:
1. entity mapping is exact
2. replay tests pass on pinned corpus
3. conflict checks pass

**BR06-D3: Mapping contract is explicit.**
Minimum mapping tuple:
1. `path`
2. `start_line:start_col`
3. `end_line:end_col`
4. `rule_id/check_id`

**BR06-D4: Conflicts are quarantined, not auto-resolved.**
Ambiguous or contradictory evidence must be query-visible as conflict state.

### Acceptance Criteria For Big-Rock-06
1. 100% external findings keep tool/version/rule provenance.
2. No external finding becomes canonical without passing promotion gate.
3. Ambiguous mappings are visible as `ambiguous`/`unmapped`, not silently dropped.
4. Query layer can filter by `canonical_only` vs `canonical_plus_evidence`.
5. FUJ includes one evidence-to-canonical promotion example.

---

## Big-Rock-07: Performance Envelope and Incremental Scale
**Status**: Drafted for decision close  
**Date**: 2026-02-26  
**Intent**: Keep "time-to-ready" fast while preserving BR01 truth guarantees.

### Why This Big Rock Exists
Correct but slow feels broken. Fast but inconsistent is worse.  
BR07 defines performance work that does not erode truth contracts.

### Binding Decisions (Current Draft)
**BR07-D1: One ingest command, multi-lane internal pipeline.**  
User sees one flow; system may run parse/evidence/enrichment lanes in parallel.

**BR07-D2: Incremental skip + stale cleanup is mandatory.**
1. hash unchanged files and skip safely
2. remove stale entities/edges for deleted files
3. keep deterministic run ledger updates

**BR07-D3: Parallelism is bounded and observable.**
Bound worker count per phase and emit per-phase timings.

**BR07-D4: Performance cannot bypass truth checks.**  
Any optimization that hides provenance/conflict/coverage is out of scope.

### Acceptance Criteria For Big-Rock-07
1. Re-ingest of unchanged repo is materially faster than cold ingest.
2. Deleted/renamed files are reflected correctly after incremental run.
3. Per-phase telemetry exists for every run (`discover`, `parse`, `evidence`, `merge`, `write`).
4. Same input + same versions still yields deterministic graph outputs.
5. FUJ latency targets are measured against these phase metrics, not anecdotal timings.

---

## Implementation Policy (V200): Build What Works First, Optimize Next
**Status**: Accepted draft  
**Date**: 2026-02-25  
**Intent**: Ship dependable ingestion truth first, then optimize throughput safely.

### Binding Policy Decisions
**POL-D1: Correctness-first delivery.**  
V200 implementation priority is:
1. deterministic ingestion outputs
2. explicit provenance + truth grades
3. conflict visibility
4. zero silent drops

Performance optimization is not phase-1 gating unless it threatens usability.

**POL-D2: Keep performance hooks from day 1 (do not defer these).**  
The following are mandatory in initial implementation:
1. batch writes
2. content-hash skip for unchanged files
3. parallel file processing where safe
4. per-phase timing telemetry (`discover`, `parse`, `evidence`, `merge`, `write`)

These keep optimization optional later, not painful rewrites.

**POL-D3: Do not defer contract stability.**  
These contracts are frozen early:
1. canonical entity key format
2. ingestion ledger schema
3. provenance schema
4. truth-grade model (`verified`, `heuristic`, `rejected`)

Changing these late is considered high-risk and requires explicit migration plan.

**POL-D4: One command, one run ledger, explicit phase states.**  
User executes one ingest flow; internally pipeline may have multiple lanes.  
Each file must record phase outcome:
1. `discovered`
2. `parsed`
3. `evidenced`
4. `merged`
5. `written`
6. terminal state (`completed` | `failed` | `skipped`) with reason

**POL-D5: No full source-body persistence in V200 graph core.**
1. Store canonical entity/edge facts and source addresses only.
2. Resolve code text at read time from filesystem (and optionally pinned VCS revision when requested).
3. This keeps graph storage focused on truth relationships while preserving deterministic source access.

### Acceptance Criteria For This Policy
1. A failed run can answer: exactly what was missed, where, and why.
2. Same input + same tool versions => same output graph and ledger.
3. Post-MVP optimization can be done without changing query contracts.
4. Time-to-ready and throughput are measurable from built-in telemetry.
