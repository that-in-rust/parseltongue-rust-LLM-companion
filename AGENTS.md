# AGENTS.md — rules for any agent working in this repository

## Naming rule: four words, hyphenated, for everything

Every component, artefact, table, tool, model, role and view named in this repository's documents and code gets an explicit **four-word hyphenated name** — the way the HTTP API already names its endpoints (`blast-radius-impact-analysis`, `reverse-callers-query-graph`). Enum *values* (status words such as `extracted`, `abstained`) are values, not names, and stay single words.

Why: a four-word name is unambiguous in prose, greppable, and reads as a sentence fragment; two-word names collide and one-word names drift.

The external coding agent (Claude Code, Codex, Cursor, …) is `ext-coding-agent-llm`; `ext-coding-agent` is its accepted short form in prose.

## Nomenclature (keep in sync with devin-research/Deep-Insight-08.md)

```text
 ROLES
 ext-coding-agent-llm              the external coding agent (Claude, Codex…) that uses Parseltongue; writes intent, never facts
 human-architect-reviewer-role     the person; writes intent, reviews rows
 parseltongue-cli-dispatcher-binary one binary; routes one verb to one CPU tool; the only thing the agent calls
 offline-teacher-rust-analyzer     rust-analyzer/rustc labelling training data off-line; never at runtime
 offline-judge-heldout-evaluation  scoring the tiny models on held-out repositories

 ENTITIES
 multi-level-code-map-entity       what we build: the tables below
 multi-level-query-engine-entity   what the agent uses: find · impact · status · propose

 TABLES AND VIEWS (one Turso / SQLite-compatible file)
 file-source-text-table            raw files: path, folder, digest, text — written first, by a CPU tool
 span-reference-text-table         one row per item span: digest id, kind, name, path, visibility, range, raw code, doc, cfg
 name-token-index-table            sub-tokens of names and paths, for joins and BM25
 field-signature-detail-table      per-span fields: params, return, generics, bounds, flags, derives, impl trait/self
 edge-evidence-record-table        every L2/L3 edge with status, confidence, evidence span, snapshot
 hld-current-product-table         THE PRODUCT: public item → public item rows, extraction-only
 hld-future-intent-table           intended rows; written only by ext-coding-agent-llm or human-architect-reviewer-role
 hld-delta-derived-view            full outer join of the two; nobody writes it: to_build · to_remove · realized · unplanned
 hld-file-level-rollup             file → file counters maintained on write
 hld-folder-level-rollup           folder → folder counters maintained on write
 agent-memory-prose-table          what the agent learned about the map; agent-written; the only table with prose

 CPU TOOLS (decide nothing; move bytes, run SQL)
 cpu-tool-ingest-source-files      walk tree · digest · write file-source-text-table
 cpu-tool-span-cutting-driver      feed file text to the span tagger; grey rows to the clarifier; write span rows
 cpu-tool-text-copy-indexer        copy span text/doc into columns · name tokens · FTS
 cpu-tool-field-extraction-driver  feed span text to the field tagger; write field rows
 cpu-tool-name-join-edges          SQL: use-paths × names → extracted edges or candidate lists
 cpu-tool-bind-resolve-edges       feed candidates to the binder; feed bodies to the site tagger; write inferred/abstained
 cpu-tool-hld-table-builder        derive hld-current-product-table and the roll-ups from edge + span (DI-07 §12.6)
 cpu-tool-find-search-query        trie / FTS / BM25 → ranked rows + span text
 cpu-tool-impact-graph-walk        recursive CTE over edges: callers, callees, blast radius, cycles
 cpu-tool-propose-future-rows      validate and write hld-future-intent-table rows for the agent/human
 cpu-tool-status-coverage-banner   per-level coverage, stale rows, pending L3

 TINY MODELS (batch of text in → labels + confidence out; never touch the table)
 tiny-model-span-boundary-tagger   2–5 MB; every token; where public items start/end, kind, name, visibility
 tiny-model-grey-span-clarifier    ~100 MB; grey spans only; fix boundary/kind/visibility
 tiny-model-field-detail-tagger    30–100 MB; span text only; generics, bounds, flags, cfg, impl
 tiny-model-candidate-edge-binder  ~100 MB; site + candidates → one target or abstain
 tiny-model-result-rank-scorer     optional; reranks find results
 tiny-model-body-site-tagger       2–5 MB; every body token; call/construct/field/await/unsafe sites (L3)

 LEVELS
 crate-level-l0-context · folder-file-l1-rollup · public-item-l2-interface · body-site-l3-evidence

 OTHER
 per-level-coverage-banner · snapshot-tree-digest-stamp · public-span-feed-export
```

## Research conventions

- `devin-research/` is append-only: new thinking is a new section or a new file; earlier text is never replaced. Standalone Deep-Insight files are mirrored into `Deep-Insight-01.md` as new Parts.
- Vertical sections and fenced `text` blocks; no wide side-by-side markdown tables.
- Separate measured numbers from envelope estimates; say which is which.
