# Deep Insight 08

*Names follow the four-word rule in AGENTS.md.*

1. **hld-current-dependency-edges is the product.**
2. **It is read on every task and rebuilt only where the tree changed.**
   - One row = a public item depending on a public item: node01 (signature, file, folder, kind, span id) · node02 (same) · edge type · status · confidence [min, max] · evidence count · snapshot. No bodies, no private items — those are evidence behind a row, in edge-evidence-record-table and span-reference-text-table.
   - Raw code is not in the row. It lives once in span-reference-text-table, keyed by digest, where full-text search also lives; search starts there and lands in HLD rows by one join.
   - **Written only by extraction** — cpu-tool-hld-table-builder deriving from edge and span rows that CPU tools and tiny models produced. Never by ext-llm-coding-agent, never by human-architect-intent-author.
   - **hld-future-dependency-edges** holds the intended design. Written only by ext-llm-coding-agent or human-architect-intent-author through cpu-tool-propose-future-rows, which validates and never authors. Same row shape, status `proposed`, plus creator and reason.
   - **hld-delta-dependency-view** is the gap: a full outer join of the two. Nobody writes it. States: to_build · to_remove · realized · unplanned.
   - **hld-file-level-rollup** and **hld-folder-level-rollup** are counters (weight, kinds, confidence range) updated by trigger on every HLD row — the folder map is never stale relative to the item map.
   - **agent-learned-notes-table** is the only place prose is allowed: what ext-llm-coding-agent learned about the map, keyed to span ids. Agent-written; never read by extraction.

## End-to-end workflow, with roles

```text
 step  what happens                                                       who does it                                   writes
 ───── ───────────────────────────────────────────────────────────────── ───────────────────────────────────────────── ──────────────────────────────
 0     tree is walked, hashed, saved                                      cpu-tool-ingest-source-files                  file-source-text-table
 1     files cut into public spans; grey spans re-read                   cpu-tool-span-cutting-driver                  span-reference-text-table
                                                                           → tiny-model-span-boundary-tagger
                                                                           → tiny-model-grey-span-clarifier
 2     span text/doc copied, names tokenised, FTS built                   cpu-tool-text-copy-indexer                    span, name-token-index-table
 3     signature details extracted                                        cpu-tool-field-extraction-driver              field-signature-detail-table
                                                                           → tiny-model-field-detail-tagger
 4     names joined to targets; unique = extracted, else candidates       cpu-tool-name-join-edges                      edge-evidence-record-table
 5     candidates resolved; L3 sites found and queued                     cpu-tool-bind-resolve-edges                   edge-evidence-record-table
                                                                           → tiny-model-candidate-edge-binder
                                                                           → tiny-model-body-site-tagger
 6     HLD rows and roll-ups derived                                      cpu-tool-hld-table-builder                    hld-current-dependency-edges,
                                                                                                                         hld-file/folder-level-rollup
 7     coverage per level published                                       cpu-tool-status-coverage-banner               per-level-coverage-banner
 ───── first map ends here: ≤ 30 min for a Linux-sized tree on an 8 GB Air; L3 pending rows counted, draining in background
 8     agent asks: where is X · what breaks if X changes                  ext-llm-coding-agent                          —
       answered from the tables                                           cpu-tool-find-search-query, cpu-tool-impact-graph-walk
 9     intent written as rows: new node, new edge, retire edge            ext-llm-coding-agent or human-architect-intent-author
                                                                           via cpu-tool-propose-future-rows             hld-future-dependency-edges
 10    gap read: to_build rows, each with file, folder, neighbours, sig   ext-llm-coding-agent (reads hld-delta-dependency-view)   —
 11    code written for exactly those rows                                ext-llm-coding-agent                          the working tree
 12    changed files re-run steps 0–7 for those files only                cpu tools + tiny models                       all extraction tables
 13    matching rows appear in hld-current → delta flips to realized      hld-delta-dependency-view (by definition)        —
 14    review: rows of intent vs rows of fact                             human-architect-intent-author                 —
 ───── always in background: L3 drain (steps 5–6 per region), labels, re-map on edit; offline-teacher-rust-analyzer and
       offline-judge-heldout-evaluation train and score the tiny models, never at runtime
```

Roles in one line each:
- **ext-llm-coding-agent** — reads the product, writes intent and code; never writes facts.
- **human-architect-intent-author** — writes intent, reviews rows; never writes facts.
- **CPU tools** — move bytes, run SQL, derive tables; decide nothing.
- **tiny models** — make every judgment; see text, return labels; never touch a table.
- **offline teacher and judge** — rust-analyzer/rustc label and score; absent at runtime.
