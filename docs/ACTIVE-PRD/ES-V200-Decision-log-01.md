# v200-Decision-log-01
Status: Active — restructured 2026-02-22 using Minto Pyramid Principle
Purpose: Record binding decisions and open questions for V200.

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
