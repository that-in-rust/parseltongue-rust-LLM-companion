# AGENTS.md — rules for any agent working in this repository

## Definition of done (read first)

```text
 A milestone is DONE only when its SOP-subset run report shows PASS on all five repos:
   hld-current-test-runs/<run-id>/hld-current-test-report.md
 Unit tests passing = progress, never done.
 Any production change after a run invalidates it: new run, all five repos, one frozen release executable digest.
 Status is reported per SOP step (PASS / FAIL / BLOCKED / not-started) with measured numbers — no bare percentages.
 Phase 1 development is driven by a real-source vertical acceptance test on pinned itoa (SOP §1.1).
 Its first checkpoint is the independently source-verified
 `<Buffer as Default>::default -> Buffer::new` relationship; this one-relation progress is not a
 whole-itoa/golden/SOP PASS. Phase 2 retains the original five-repository acceptance contract.
 Active Phase 1 is the detailed full map and exact source/evidence readback (REQ-RUST-044.0).
 HLD selection, collapse, roll-ups and projection-only decisions are deferred in BACKLOG.
 Do not require HLD output to prove a full-map relationship or discard private nodes.
```

## Canonical paths (PRD, SPEC, ARCH and SOP in phase-20261113; journal, status and pins in phase-20261111)

```text
 PRD      docs/phase-20261113/PRD-phase-20261113-dependency-graph-hld.md       (product intent, decisions, phases)
 SPEC     docs/phase-20261113/SPEC-phase-20261113-executable-requirements.md   (REQ-RUST ids + verification matrix)
 ARCH     docs/phase-20261113/ARCH-phase-20261113-pipeline-graph-tables.md     (pipeline, tables, graph and level rules)
 TARGET   docs/phase-20261113/itoa-target-dataset/                             (LLM_PARSED_DRAFT candidates authored by Devin from pinned source; draft, never production input)
 HISTORY  docs/phase-20261111/{PRD,SPEC}-phase-20261111-*.md and docs/phase-20261113/archive-folded-draft-files/ (not current)
 SOP      docs/phase-20261113/SOP-phase-20261113-full-item-dependency-graph.md (S1–S10, milestone subsets, revisions R1–R16)
 JOURNAL  docs/phase-20261111/TDD-phase-20261111-progress-checkpoints.md       (append-only TDD journal)
 STATUS   docs/phase-20261111/current-milestone-status-board.md                (rewritten in place; one page)
 BACKLOG  docs/phase-20261113/phase1-abstractions-decision-backlog.md         (deferred work, decisions, resume triggers)
 LOG      docs/phase-20261113/phase1-log-p1.md                                 (newest-first Phase 1 evolution log)
 PINS     docs/phase-20261111/{parser-source-pin,oracle-source-build,corpus-source-pin}-manifest.json
 RUNS     hld-current-test-runs/<run-id>/ (committed reports) · ~/hld-current-test-artifacts/<run-id>/ (large, gitignored, digest-recorded)
```

Source containment decision (2026-11-14, REQ-RUST-043.0): original impl members have one
lexical source container (file area or enclosing inline module), not an implementing-type
container. Impl/self-type/trait ownership stays separate metadata. Impl blocks remain records;
body-local items remain evidence. No contains/imports collapse or macro ownership is inferred.

A change to architecture or to any pin updates SPEC, ARCH, SOP and AGENTS.md in the same commit.
The PRD, SPEC and ARCH state only what is in force: edit the text in place, do not append revision sections.

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

### Single-VM execution only

Use only the current session VM for all implementation, integration, builds and tests.
Do not create or use separate-VM child sessions. A request to parallelize means concurrent
work on this same VM, not additional VMs. Preserve and integrate existing worker changes here;
continue all further work here unless the user explicitly changes this rule.

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

A RED/GREEN claim requires verified test discovery, a nonzero selected-test count, and evidence
that the test reached the intended behavior. A compile-only result, zero-test filter, or fixture/
setup failure is not behavioral RED/GREEN evidence.

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

- Phase 1 starts with the pinned-itoa real-source vertical acceptance test in SOP §1.1; component
  tests are supporting checks, not source acceptance. For Phase 2 qualification, run serde first;
  only then rust-analyzer, ripgrep, tokio and cargo-modules.
- Kill a correctness run only if it exceeds 2× its expected wall time; until all-five DuckDB S5 exists, do not set an interim DuckDB memory cap and measure/report effective engine limits plus whole-tree peak RSS. After S5, enforce the unchanged final target ≤500 MB whole process tree.
- 2026-11-12 revision: interim DuckDB cap removed by user decision; memory is measured and reported only until all-five S5 exists. Final ≤500 MB target remains unchanged.
- Keep Cargo package selection, features, profile, target, toolchain and lock inputs stable within a
  test cycle. Separate dependency compilation from test execution; avoid repeated arbitrary short
  timeouts that kill cold dependency compilation. Declare expected time, measure elapsed time and
  process-tree RSS, and do not add a memory cap against the current user override.
- Measure memory and time before arguing design.

### Stop rule

After 3 failed attempts at the same step: stop, journal what was tried, re-plan or cut the step smaller. No further tuning.

### Status messages

Milestone · per-SOP-step PASS/FAIL/BLOCKED · measured numbers · next action. Read STATUS and the latest run report before answering "how far".

For Phase 1, lead with the pinned-source relationship's expected/actual/missing evidence and the
first blocking production stage; aggregate unit counts are secondary and never acceptance evidence.

Research rubric, source ledger, negative-knowledge registry and skill packaging: [agent-working-method-playbook](docs/agent-working-method-playbook.md).

## Nomenclature (keep in sync with docs/devin-research/Deep-Insight-08.md)

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
 full-item-dependency-graph        item-effective-visibility-records (nodes) + edge-evidence-record-table (edges); private items included; never collapsed
 file-public-area-node             one per source file, separate from its module node; contains the file's top-level nodes
 hld-current-dependency-edges      deferred query result: current public item → public item rows, extraction-only
 hld-future-dependency-edges       intended rows; written only by ext-llm-coding-agent or human-architect-intent-author
 hld-delta-dependency-view         full outer join of the two; nobody writes it: to_build · to_remove · realized · unplanned
 hld-file-level-rollup             deferred query/materialization: file → file counters
 hld-folder-level-rollup           deferred query/materialization: folder → folder counters
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

 PHASE-20261111 ADDITIONS (docs/phase-20261113/PRD-phase-20261113-dependency-graph-hld.md; compiling code only, ported rust-analyzer logic)
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

- `docs/devin-research/` is append-only: new thinking is a new section or a new file; earlier text is never replaced. Standalone Deep-Insight files are mirrored into `docs/devin-research/Deep-Insight-01.md` as new Parts.
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

## Revision note — itoa graph-first construction

Phase 1 follows SPEC REQ-RUST-029.0–031.0 and
`docs/phase-20261111/itoa-hld-current-benchmark/itoa-hld-backward-design.md`:
development is driven by a real-source vertical acceptance test, not by
completing downstream table contracts first. The first checkpoint independently
verifies `<Buffer as Default>::default -> Buffer::new` in the pinned source and
asserts it through the production pipeline. Full semantic identities and source
evidence belong in test assertions; do not feed pre-resolved facts to production
or hardcode the expected edge there. The first observed missing stage determines
which general-purpose producer path is needed. Reference families/display names
are not frozen canonical occurrences; never feed expected data into production.
Builder fixtures are not a real-source comparison. Keep `body-site-reference-table` as
the body-site store. Turso owns durable semantic facts and atomic HLD serving;
DuckDB owns relational analysis. Halted large-repository/native work remains
separate from this focused slice; all-five milestone completion is unchanged.

The Phase 1 reference rebuild is offline dataset/contract work only: preserve the
complete source-reviewed itoa candidate inventory, show a separate eligible
local-only subset plus blockers. Keep five derived expected HLD outputs as deferred artifacts,
distinct from the active full-map inventory. These symbolic candidates are not canonical extracted
rows or production input. Verify top-down contracts in the single manifest;
do not settle OD-2/OD-8/OD-9 or the remaining file-module/area part of OD-7, infer macro
file ownership, or turn external
labels into extracted targets. OD-9 and other projection-only decisions live in BACKLOG;
the retained six-kind rule still applies if collapse is later invoked, never to full-map
storage. Source coverage limits stay explicit. Baseline
checks are read-only, report-only and fail closed.

### Historical revision note — direct HLD materializer checkpoint

The bounded DuckDB materializer slice selects nodes and projects direct edges
in SQL only. It is explicitly DirectProjectionOnly; recursive private-chain
collapse, roll-ups, frozen-input marker checks, Turso publication and
real-source producers remain separate increments. Reject extracted private
traversal rather than implying a complete graph. Dependency origin uses
explicit unit provenance and full compilation-unit identity, never labels or
paths. Keep canonical edge fields and full evidence-row keys.

The validation increment accepts only namespace-seeded primitive Builtin type
names. ExternalDefinition records cannot share a full unit key with a LocalItem
or CrateRoot; evidence sites must share the owning source's full unit key.
Unknown owner semantics and inconsistent body ancestry fail closed, while
nested modules inside bodies remain excluded rather than invalidated.

### Historical revision note — recursive HLD path/collapse checkpoint

The current DuckDB materializer recursively traverses extracted edges in SQL
from selected sources through nonselected LocalItems to the first selected or
canonical external/builtin terminal. Per-path full occurrence identities
prevent repeated private endpoints while permitting one terminal return to
the source. Paths retain ordered endpoint and evidence-row identities; the
terminal relation labels the output. SQL groups by generation/source/target/
relation/via and counts the distinct evidence-key union with UNNEST. The
typed output is GraphPathsOnly, sorted after decoding, with both roll-ups
uncomputed. Roll-ups, frozen-stage-marker validation, Turso publication,
real-source producers and CLI complete-stage status remain separate work.

### Revision note — DuckDB file/folder roll-ups

The bounded materializer now groups a SQL materialized/deduplicated HLD edge
relation for both file and folder roll-ups. Path/evidence multiplicity never
inflates counts; relation and collapse-via kind remain grouping keys.
SourceFile and SourceFolder buckets preserve full unit identity and
normalized paths; CrateRoot and canonical external identities remain
separate typed buckets. Missing external file metadata is never inferred.
CompleteGraph with present roll-up options describes only supplied typed
facts, not source-pipeline completion or durable Turso publication.

### Revision note — transactional Turso HLD publication

HldGraphPublicationPort is the synchronous L2 seam for publishing only
typed CompleteGraph output with both roll-up options present. The Turso L3
adapter uses the generic stage-owned schema and one transaction for the five
semantic HLD tables, hld/current unit/global completion markers, snapshot,
and workspace-generation manifest. Required source markers must be visible
through the exact frozen revision; table presence and later mutable-base
markers do not qualify.
For every required input owner, any current-scope completion/work-unit marker
not present in the supplied frozen revision returns typed StaleInputSnapshot.
This exact-closure rule is HLD-specific; generic snapshot semantics do not
change. Generic writes by the HLD owner are rejected once any of its markers
is frozen. Publication clears all unsealed HLD rows/markers in the destination
scope before writing, preventing stale work units from leaking into the view.

The global HLD completion-marker payload records the input snapshot alias,
resolved base attempt and revision as persistence metadata, separate from
semantic row payloads. Registered semantic inputs belong to
signature-type-lowering-stage, item-effective-visibility-stage, and
semantic-edge-evidence-stage. Existing workspace_required_stage_names remain
enforced. Canonical semantic JSON excludes attempt IDs and Debug strings.
Cancellation, backend failure, and sealed-unit rejection preserve prior
publications. This is publication compatibility only; producers, CLI
integration, and source/corpus qualification remain outside scope.

## Revision note — full graph first, levels derived (2026-11-13)

REQ-RUST-045.0 permits explicitly incomplete full-map development checkpoints using existing
canonical records and frozen storage, independently of HLD. Unknown visibility is not guessed;
HLD completeness/publication gates remain strict. Required node/edge markers describe the bounded
checkpoint only. The full-map final stage must preserve incomplete state on frozen readback.

User decision. SPEC REQ-RUST-032.0, 019.2 and 030.1; parent PRD §17; SOP R16;
rules and itoa example in ARCH section 5 and 6.

- Build the full-item-dependency-graph first: every item outside fn bodies is a
  node, private items included, and every extracted edge is stored uncollapsed.
- HLD levels are DuckDB derivations of it. A level filters, collapses and
  counts; it never adds a fact.
- Active Phase 1 stops at verified full-map facts and evidence. Abstractions are later
  queries over one compatible frozen generation, optionally materialized; they do not
  introduce separate semantic pipelines. Queries cannot recover missing upstream facts.
- Each source file has a file-public-area-node. Each `use` is a node with
  `imports` edges. `reads` is the relation for const and static reads.
- An edge targets the real item, never the `use` line.
- Collapse never walks a `contains` or `imports` edge.
- Turso + DuckDB, identities and publication are unchanged.
- Implementation/verification status belongs in STATUS, not this rule. Do not implement
  undecided full-map choices without a decision; defer irrelevant choices to BACKLOG.

### Revision note — original-file top-level containment scope

REQ-RUST-040.0 adds one real-source stage, `original-top-level-containment`, without changing the
default. It admits only cfg-active items owned by their original file at top level; it does not
admit nested, associated, body, or expanded items or semantic dependency edges. File-area identity
is graph-only (unit plus normalized relative path), never a fabricated occurrence or namespace
symbol. Source coverage remains distinct from typed HLD output completeness; do not claim semantic
completeness or weaken the existing full-semantic publication gate.
This mode stays diagnostic. It is not the detailed-map product and must not be silently
expanded to include members or semantic calls. Full-map nodes/evidence are the active work.

### Revision note — frozen HLD graph input storage

REQ-RUST-041.0 adds only the L2 `HldGraphInputStorePort` and Turso persistence/assembly plus
scope-aware HLD publication. Resolve canonical units from frozen root namespace scopes mapped to
raw crate graph unit keys; never derive `unit_configuration_digest_full` from the raw graph hash.
Structural graph markers carry exact `mapping_scope_kind` and `source_coverage_kind` fields.
Analysis input snapshots do not move the workspace-serving pointer; final HLD publication remains
the only serving transition and retains its single-transaction rollback boundary. Source SQL
production and CLI integration are explicitly deferred.
