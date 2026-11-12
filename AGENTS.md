# AGENTS.md — rules for any agent working in this repository

## Definition of done (read first)

```text
 A milestone is DONE only when its SOP-subset run report shows PASS on all five repos:
   hld-current-test-runs/<run-id>/hld-current-test-report.md
 Unit tests passing = progress, never done.
 Any production change after a run invalidates it: new run, all five repos, one frozen release executable digest.
 Status is reported per SOP step (PASS / FAIL / BLOCKED / not-started) with measured numbers — no bare percentages.
```

## Canonical paths (phase-20261111)

```text
 PRD      docs/phase-20261111/PRD-phase-20261111-compiling-hld.md
 SPEC     docs/phase-20261111/SPEC-phase-20261111-executable-requirements.md   (REQ-RUST ids + verification matrix)
 SOP      docs/phase-20261111/SOP-phase-20261111-hld-current-e2e-test.md       (S1–S10, milestone subsets, revisions R1–R8)
 JOURNAL  docs/phase-20261111/TDD-phase-20261111-progress-checkpoints.md       (append-only TDD journal)
 STATUS   docs/phase-20261111/current-milestone-status-board.md                (rewritten in place; one page)
 PINS     docs/phase-20261111/{parser-source-pin,oracle-source-build,corpus-source-pin}-manifest.json
 RUNS     hld-current-test-runs/<run-id>/ (committed reports) · ~/hld-current-test-artifacts/<run-id>/ (large, gitignored, digest-recorded)
```

A change to architecture or to any pin updates SPEC, SOP and AGENTS.md in the same commit.

## Naming rule: four words, hyphenated, for everything

Every component, artefact, table, tool, model, role and view named in this repository's documents and code gets an explicit **four-word hyphenated name** — the way the HTTP API already names its endpoints (`blast-radius-impact-analysis`, `reverse-callers-query-graph`). Enum *values* (status words such as `extracted`, `abstained`) are values, not names, and stay single words.

Why: a four-word name is unambiguous in prose, greppable, and reads as a sentence fragment; two-word names collide and one-word names drift.

The four words must describe what the thing IS (its contents, scope, kind) — never a claim about it ("product"), a role suffix ("role"), or filler ("driver", "table" where the kind is already clear). Bad: `hld-current-product-table`. Good: `hld-current-dependency-edges`.

The external coding agent (Claude Code, Codex, Cursor, …) is `ext-llm-coding-agent`; `ext-coding-agent` is its accepted short form in prose.

Scope of the rule (clarified 2026-11-11): it covers Rust identifiers too — crates (`parseltongue-core-row-types`), types,
traits and error types (`SpanDigestIdValue`, `SpanRowReadStore`, `CrateGraphLoadError`), functions, CLI verbs, and test names
(four words after the `test_` prefix: `test_span_gap_tiling_identity`). Exempt: enum variants and status values (`Extracted`,
`unresolved`), and names owned by other projects (rust-analyzer's `DefCollector`). The fixed prefixes `cpu-tool-` and
`tiny-llm-` count as one word, as in the nomenclature below (`cpu-tool-syntax-tree-walker`).

## Rust code style: idiomatic functional Rust (always apply)

- Prefer pure functions, immutable data, expressions, and small total functions with typed inputs/outputs.
- Prefer iterator combinators (`map`, `filter`, `fold`, `try_fold`), `Option`/`Result` combinators and `?` over mutation and loops with flags.
- Keep side effects (I/O, DB, processes) at L3 edges behind traits.
- No `unwrap`/`expect` outside tests; no `unsafe`; no global mutable state.
- Make ownership/borrowing clear rather than clever; readability and reasoning beat speed.
- Accept small-value clones or a second pass for clarity; optimise only after a measured budget miss.
- Implement with TDD stub → red → green → refactor (`tdd-workflow-for-rust-stub-red-green-refactor`).
- Use repo skills in `.agents/skills`: `rust-coder-02`, `idiomatic-rust-coder-01`, `rust-executable-specs-01`.

## Working method (always apply)

### TDD loop with a journal — every code change, every time

Use `tdd-workflow-for-rust-stub-red-green-refactor` constantly; no code change skips it.

```text
 1 pick REQ-RUST ids from SPEC (add/revise the REQ first if missing)
 2 STUB      signatures + todo!(); cargo check                       → journal entry
 3 RED       failing tests named test_<four_words>, doc-tagged /// REQ-RUST-NNN.R; paste failing output → journal entry
 4 GREEN     minimum code; tests pass                                 → journal entry + commit
 5 REFACTOR  combinators, newtypes, typed errors, dead code; or an explicit "no refactor needed" note → journal entry + commit
 6 gates     cargo fmt --check · clippy --all-targets -D warnings · cargo test · release build
 7 push      git pull --rebase origin <branch> first (the user also commits from a Mac)
 8 milestone end → SOP-subset run → run report → update status board
```

Journal entry (append to JOURNAL after EVERY phase, never edit old entries):

```text
 phase · REQ ids · test names + status · exact command + pasted output (pass/fail lines)
 commit sha · decisions + rejected options · next step · blockers
```

### Requirements, errors, layering

- Every test names its REQ; a check fails when the SPEC verification matrix lists a REQ or test id that does not exist.
- L1 core is dependency-free with hand-written error enums; L2/L3 errors are `thiserror` enums. A free-text-only error variant is a refactor defect.
- Side effects only in L3 adapters behind L2 traits.

### Budgets before big runs

- Always run serde first; only then the other four repos.
- Kill any run that exceeds its memory budget (final target ≤500 MB whole process tree; interim DuckDB cap ≈1.2 GiB) or 2× its expected wall time; record the peak and stop to re-plan.
- Measure memory and time before arguing design.

### Stop rule

After 3 failed attempts at the same step: stop, journal what was tried, re-plan or cut the step smaller. No further tuning.

### Status messages

Milestone · per-SOP-step PASS/FAIL/BLOCKED · measured numbers · next action. Read STATUS and the latest run report before answering "how far".

Research rubric, source ledger, negative-knowledge registry and skill packaging: [agent-working-method-playbook](docs/agent-working-method-playbook.md).

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

 TABLES AND VIEWS (Turso is the write/serving store; phase-20261111 analysis tables live in DuckDB, see SPEC)
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

NEXT PHASE (not phase-20261111; do not build now)
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
 lowered-signature-type-table · impl-header-index-table · oracle-edge-agreement-table · lang-item-definition-table · proc-macro-shape-table · hld-current-e2e-test (SOP)
 cpu-tool-crate-graph-loader · cpu-tool-port-macro-expander · cpu-tool-port-signature-lowerer
 cpu-tool-port-body-inferencer · cpu-tool-oracle-edge-differ

 RETIRED 2026-11-12 (in-memory resolver exceeded memory gate; replaced by DuckDB SQL fixed point) — do not reuse
 cpu-tool-port-name-resolver (crate parseltongue-port-name-resolver) → parseltongue-duckdb-analysis-adapter
 hld-current-test-procedure → hld-current-e2e-test (SOP file name)

 LEVELS
 crate-level-l0-context · folder-file-l1-rollup · public-item-l2-interface · body-site-l3-evidence

 OTHER
 per-level-coverage-banner · snapshot-tree-digest-stamp · public-span-feed-export · agent-working-method-playbook
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

Project name `home-ubuntu-repos-parseltongue-rust-LLM-companion`; re-index after large changes (counts go stale — read them from `index_status`, not from here). It is an exploration aid for agents working on this repo; it is not part of the Parseltongue runtime (it uses tree-sitter and LSP, which the product excludes — see Deep-Insight-07 §2).

Gotchas: it only partly parses DuckDB `.sql` files (check `parse_partial` in the index output); `search_graph --name_pattern` rejects inline regex flags such as `(?i)`.

## Environment gotchas

- DuckDB's bundled C++ build is slow and memory-heavy: `CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CXXFLAGS=-g0`; pause the VS Code rust-analyzer process (SIGSTOP) during native builds and SIGCONT it afterwards.
- The oracle harness incremental cache (`tools/oracle-semantic-export-harness/target`) can reach ~10 GiB and is safe to delete; keep its executables' digests recorded.
- The VM environment blueprint still references the deleted `experiments/spancut`.

## Revision note — mapped-path-export-capability

REQ-RUST-027.0 records that `mapped_semantic_paths` describes only the current
mapped PRIMARY/SECONDARY path-envelope exporter. Use the live ordered CLI and
single-envelope JSON format documented in SOP S4/S5. The value does not claim
full S4 family support or qualification; keep the broader S4 requirements and
historical capability records intact.
