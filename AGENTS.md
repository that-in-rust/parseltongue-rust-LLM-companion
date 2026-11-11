# AGENTS.md — rules for any agent working in this repository

## Naming rule: four words, hyphenated, for everything

Every component, artefact, table, tool, model, role and view named in this repository's documents and code gets an explicit **four-word hyphenated name** — the way the HTTP API already names its endpoints (`blast-radius-impact-analysis`, `reverse-callers-query-graph`). Enum *values* (status words such as `extracted`, `abstained`) are values, not names, and stay single words.

Why: a four-word name is unambiguous in prose, greppable, and reads as a sentence fragment; two-word names collide and one-word names drift.

The four words must describe what the thing IS (its contents, scope, kind) — never a claim about it ("product"), a role suffix ("role"), or filler ("driver", "table" where the kind is already clear). Bad: `hld-current-product-table`. Good: `hld-current-dependency-edges`.

The external coding agent (Claude Code, Codex, Cursor, …) is `ext-llm-coding-agent`; `ext-coding-agent` is its accepted short form in prose.

## Nomenclature (keep in sync with devin-research/Deep-Insight-08.md)

```text
 ROLES
 ext-llm-coding-agent              the external coding agent (Claude, Codex…) that uses Parseltongue; writes intent, never facts
 human-architect-intent-author     the person; writes intent, reviews rows
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
 hld-current-dependency-edges      the product: current public item → public item rows, extraction-only
 hld-future-dependency-edges       intended rows; written only by ext-llm-coding-agent or human-architect-intent-author
 hld-delta-dependency-view         full outer join of the two; nobody writes it: to_build · to_remove · realized · unplanned
 hld-file-level-rollup             file → file counters maintained on write
 hld-folder-level-rollup           folder → folder counters maintained on write
 agent-learned-notes-table         what the agent learned about the map; agent-written; the only table with prose

 CPU TOOLS (decide nothing; move bytes, run SQL)
 cpu-tool-ingest-source-files      walk tree · store bytes verbatim · digest · write file-source-text-table
 cpu-tool-syntax-tree-walker       one pinned ra_ap_syntax parse per file → span, field, name-token and pending L3 rows; recovery re-cut on error files
 cpu-tool-name-join-edges          SQL: use-paths × names → extracted edges or candidate lists; publishes the join rate
 cpu-tool-bind-resolve-edges       CPU resolver first; only ambiguous candidates to the binder; macro items to the guesser; write inferred/abstained
 cpu-tool-hld-table-builder        derive hld-current-dependency-edges and the roll-ups from edge + span (DI-07 §12.6)
 cpu-tool-find-search-query        trie / FTS / BM25 → ranked rows + span text
 cpu-tool-impact-graph-walk        recursive CTE over edges: callers, callees, blast radius, cycles
 cpu-tool-propose-future-rows      validate and write hld-future-dependency-edges rows for the agent/human
 cpu-tool-status-coverage-banner   per-level coverage, stale rows, pending L3

 TINY LLMS (only where grammar cannot decide; text in → labels + confidence out; never touch the table; see DI-08 "When a tiny LLM is called")
 tiny-llm-span-clarifier           grey spans only (parse error + recovery rule conf < 1); fix boundary/kind/visibility
 tiny-llm-edge-binder              site + several candidates → one target or abstain; only after the CPU resolver fails
 tiny-llm-macro-guesser            macro invocation at item position → which item(s) it defines; status inferred
 tiny-llm-result-ranker            optional; reranks find results beyond the agent's budget

 RETIRED 2026-11-09 (DI-07 §16: boundaries, fields and sites come from the parser) — do not reuse the names
 tiny-llm-span-boundary-tagger · tiny-llm-field-detail-tagger · tiny-llm-body-site-tagger
 cpu-tool-span-cutting-driver · cpu-tool-text-copy-indexer · cpu-tool-field-extraction-driver

 PHASE-20261111 ADDITIONS (docs/phase-20261111/PRD-phase-20261111-compiling-hld.md; compiling code only, ported rust-analyzer logic)
 crate-dependency-graph-table · body-site-pending-table · module-scope-binding-table · macro-expanded-item-table
 lowered-signature-type-table · impl-header-index-table · oracle-edge-agreement-table
 cpu-tool-crate-graph-loader · cpu-tool-port-name-resolver · cpu-tool-port-macro-expander · cpu-tool-port-signature-lowerer
 cpu-tool-port-body-inferencer · cpu-tool-oracle-edge-differ

 LEVELS
 crate-level-l0-context · folder-file-l1-rollup · public-item-l2-interface · body-site-l3-evidence

 OTHER
 per-level-coverage-banner · snapshot-tree-digest-stamp · public-span-feed-export
```

## Research conventions

- `devin-research/` is append-only: new thinking is a new section or a new file; earlier text is never replaced. Standalone Deep-Insight files are mirrored into `Deep-Insight-01.md` as new Parts.
- Vertical sections and fenced `text` blocks; no wide side-by-side markdown tables.
- Separate measured numbers from envelope estimates; say which is which.

## Codebase exploration tool: codebase-memory-mcp

Use DeusData/codebase-memory-mcp (MIT, native binary, 17 MCP tools, vendored tree-sitter grammars + hybrid LSP) to explore this or any repository before reading files by hand.

```bash
# install (macOS / Linux); the installer refuses a group-writable ~/.local/bin — chmod 755 ~/.local/bin first if it complains
curl -fsSL https://raw.githubusercontent.com/DeusData/codebase-memory-mcp/main/install.sh | bash
codebase-memory-mcp --version

# index a repository (once; re-run after large changes)
codebase-memory-mcp cli index_repository --repo_path /abs/path/to/repo
codebase-memory-mcp cli list_projects
codebase-memory-mcp cli index_status --project <name>

# query (flag-based; `codebase-memory-mcp cli <tool> --help` lists flags)
codebase-memory-mcp cli get_architecture --project <name> --aspects all
codebase-memory-mcp cli query_graph --project <name> ...        # callers, callees, dependencies, impact
codebase-memory-mcp cli detect_changes --project <name>          # change-aware test selection
codebase-memory-mcp cli manage_adr --project <name> --mode update   # persist architecture decisions across sessions

# MCP for a coding agent: run the binary with no args (stdio); optional graph UI: daemon start --open --port=9749
```

Indexed on 2026-11-09 for this repo: 24,681 nodes, 50,558 edges; project name `home-ubuntu-repos-parseltongue-rust-LLM-companion`. It is an exploration aid for agents working on this repo; it is not part of the Parseltongue runtime (it uses tree-sitter and LSP, which the product excludes — see Deep-Insight-07 §2).
