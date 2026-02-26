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

## Big-Rock-01: Ingestion Truth Loop (Accuracy-First)
**Status**: Drafted for immediate use (pending final sign-off)  
**Date**: 2026-02-25  
**Intent**: Make ingestion trustworthy before adding more graph algorithms.

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

### Acceptance Criteria For This Policy
1. A failed run can answer: exactly what was missed, where, and why.
2. Same input + same tool versions => same output graph and ledger.
3. Post-MVP optimization can be done without changing query contracts.
4. Time-to-ready and throughput are measurable from built-in telemetry.
