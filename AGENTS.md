# AGENTS.md

Rules for any agent working in this repository. Read top to bottom: choose the phase,
then its answer and rules, then reference facts.

## Choose the phase

- **Graph producer:** `docs/phase-20261113/` governs the pinned-itoa mapper, its dataset
  and acceptance. "This phase" and graph-completion claims below refer to that work.
- **CLI consumer:** start at [phase-20261116/ReadMe.md](docs/phase-20261116/ReadMe.md).
  This phase uses the published dataset to inform coding agents via the CLI; it does NOT
  improve the dataset. Missing producer facts remain in the phase-20261113 backlog, not
  hidden extraction or guessed relationships in the CLI. Shared source-proof, spec-first,
  test-first and delivery rules still apply; a passing query never accepts the producer.
- **Backend release proposal:** start at [phase-20261117/ReadMe.md](docs/phase-20261117/ReadMe.md).
  This is a Linux/macOS packaging/release plan for the pinned `codebase-memory-mcp`
  non-Rust backend. Nothing is implemented or validated; it does not change phase-20261113
  data or the phase-20261116 consumer CLI.

## The answer

**Build one thing: the `full-item-dependency-graph` of pinned itoa 1.0.18, produced
deterministically by Rust code from the real source.**

It is DONE only when a production run equals the frozen expected itoa graph, as the SOP
defines. Until then, report it as incomplete.

Out of scope for this phase: every other repository, any rust-analyzer export or oracle, and
every higher-abstraction table (interface level, HLD, roll-ups). Do not build, extend or test
against them.

Where we are: status is kept in one place, the "Where we are" page at the top of
[the log](docs/phase-20261113/LOG-itoa-full-item-graph-progress.md). Read it before answering
"how far". Do not copy numbers from it into this file.

## Rule 1 — Only source-backed facts count

- A relationship is proven only when the production CLI derives it from pinned source, persists
  it, and a real-source test reads it back with exact file, range and digest.
- Unit tests, fixtures and builds passing are progress, never done.
- Never feed expected data or the target dataset into production, and never hardcode an
  expected edge.
- Never guess. Unknown visibility, unresolved calls, inactive `cfg` items and unsupported forms
  are stored explicitly with a typed reason.
- Source is often missing or broken, so every edge is in one of three states. **Proven:** every
  candidate was visible in source. **Proven, gap:** one target was found, and the edge names the
  source that was missing. **Unresolved:** zero or several targets were found, with one typed
  reason. Acceptance still needs zero gap edges, or an explicit accepted list.
- Any production change after a run invalidates that run.
- Status is given per SOP step (PASS / FAIL / BLOCKED / NOT STARTED) with measured numbers.
  No bare percentages, no "complete" for a bounded result.
- Progress on a part is expected rows matched, per relation kind: N of M. Until a row-by-row
  comparison exists, say that a count is stored rows, not matches, and never set a count of
  stored sites against a count of expected rows.
- A number in a part file or the log is measured, with its command, or marked predicted.
- `codebase-memory-mcp` and `clarity` show structure. They are navigation aids, not evidence.
- `docs/phase-20261113/itoa-target-dataset/` is the expected graph, frozen on
  2026-11-16 and checked against source by its author alone. A correction needs a source
  citation and a new freeze (SOP S1).

## Rule 2 — The itoa full graph only; everything else is backlog

Graph rules in force:

- Every item outside a function body is a node, private items included.
- Each source file has one file-area node. Each `use` is a node.
- A module contains the file area of its file: the crate root that of the root file, a
  `mod name;` module that of the file it names (SPEC part 12).
- Edges are stored uncollapsed. Relation kinds: `contains`, `calls`, `imports`, `uses_type`,
  `reads`, `implements`, `reexports`, `derives`, and for macros `generates` and `expands`.
- One edge row per source site. Two sites with the same source, relation and target are two
  rows, each with its own range. The expected graph lists one row per source, relation and
  target with its sites, so group stored rows the same way before comparing.
- An edge targets the real item, never the `use` line.
- Impl blocks are records, not nodes. Body-local items are evidence, not nodes.
- An impl member has one lexical container: its file area or enclosing inline module. Impl,
  self-type and trait ownership are separate metadata.

Ownership:

- Rust: parsing, macro expansion, call resolution, identities.
- DuckDB: name resolution, node projection, relational analysis.
- Turso: durable storage, frozen snapshots, atomic publication, serving.
- No third database.
- A DuckDB file of the finished graph is a copy written after the run by a separate crate
  from the exported JSON (user decision, 2026-11-17; SPEC part 9). It is not a store of
  the run.

Scope:

- This phase: the full graph and its evidence on pinned itoa 1.0.18. Nothing else.
- Interface levels, collapse and roll-ups are later queries over the stored graph. They are
  not part of this phase.
- Anything not needed for the itoa full graph goes in
  [the backlog](docs/phase-20261113/BACKLOG-deferred-work-open-decisions.md). Do not implement an
  undecided choice; ask, or backlog it.
- If most of a part's expected rows wait on an open decision, ask for that decision before
  starting the part.
- A target in `core` is a named external target, or a row without a target that the frozen
  expected graph lists (PRD, decided 2026-11-15). No node or edge of `core` itself is stored.
- A method call into `core` is named from the `core` source (PRD, decided 2026-11-16): the
  run passes `--sysroot-source-scope-kind declarations` and reads the declarations of
  `core`, which the SOP freezes by digest like the pinned source. The accepted run of
  2026-11-17 passed `declarations`; the earlier one, of 2026-11-16, passed `excluded`.
- The expected target of a call is written from source and frozen before the code that
  names it. A target the run names and the expected graph does not list is a FAIL.

## Rule 3 — One VM, one branch, test first, measure

- **One VM.** All work, builds and tests run on the current session VM. No child sessions, no
  extra worktrees or clones.
- **One branch.** `research/tiny-rust-code-mapper`. `git pull --rebase` before pushing (the user
  also commits from a Mac).
- **Focused commits.** One focused change per commit, at most about 1,000 changed lines, with a
  message that says what changed and why. Push after each. Do not hold work back for a full
  rewrite. Only a delete-only commit may be larger.
- **Spec first.** Before code for a new target, write its part file in the SPEC folder with
  the skill `rust-executable-specs-01` (seven sections), and update the
  index in the same commit. Parts not yet picked stay stubs.
- **Test first.** Write a failing real-source test on pinned itoa, make it pass with the smallest general change, refactor. A RED or GREEN
  claim needs a nonzero selected-test count and a test that reached the intended behaviour.
- **Gates before every push of code**, on each crate the commit touches, plus the real-source
  itoa tests:

```bash
cargo fmt --all -- --check
cargo clippy --release -p <crate> --all-targets --all-features -- -D warnings
cargo test  --release -p <crate> --all-targets --all-features
cargo test  --release -p parseltongue-cli-dispatcher-binary --test itoa-source-acceptance-tests -- --test-threads=1
```

- **Workspace gates once per step of a part**, before its log entry:

```bash
cargo fmt --all -- --check
cargo clippy --release --workspace --all-targets --all-features -- -D warnings
cargo test  --release --workspace --all-targets --all-features -- --skip test_lang_items_in_sysroot --test-threads=1
cargo build --release --workspace --all-targets --all-features
```

  The skipped test is an open defect (native sysroot timeout), not a waiver. The workspace gate
  is not PASS while it is skipped.
- **Measure.** Record wall time and whole-process-tree peak memory for every run. No memory
  cap in this phase.
- **Stop.** Kill a run that exceeds 2× its expected wall time. After 3 failed attempts at one
  step, stop, log what was tried, and cut the step smaller.
- **Log.** After each GREEN or REFACTOR commit, add an entry at the bottom of the log, and update its "Where we are" page: test names, exact command, pass/fail counts, commit, next target, and the codebase-memory queries run (Rule 4).

## Rule 4 — Query codebase-memory at every step

Use the skill `codebase-memory` (CLI `codebase-memory-mcp`, commands under Navigation aids)
as the first tool for every question about the code, not once per part. grep and sed are for
reading the lines the graph points to, and for text it does not hold: string literals, table
names, JSON keys.

Query it at each of these points:

- **Session start, and after any context reset:** `index_status`. Re-index if `indexed_at`
  is older than the last code commit.
- **Before answering "where are we" or reviewing work:** re-index, then query the functions
  the answer is about.
- **Before writing a part file:** `trace_path` from the stage entry point to the code the
  target needs. The design section names the callers and callees found.
- **Before writing a test:** `search_graph` for the fixtures and helpers that already exist,
  and `trace_path` inbound on the function under test to see which tests cover it.
- **Before changing a function:** `trace_path` on it in both directions, and read its
  callers before changing what it returns.
- **Before adding a function, type or module:** `search_graph` for one that already does
  the job. Extend it instead of writing a parallel copy.
- **Before each commit:** `trace_path` inbound on every function the diff changed, and run
  the tests it names.
- **After each code commit:** `index_repository`. The CLI index is not watched and never
  refreshes itself.
- **In the refactor step:** list the longest and most complex functions in the files touched
  (`query_graph` on `complexity`) and say for each whether it was split.

Two limits:

- **Record the queries.** Each log entry lists the codebase-memory queries run for that step.
  A step whose entry lists none did not follow this rule.
- **Confirm in source.** Its call edges are matched by name and some are wrong. Check every
  claim taken from it against the source lines before writing it down or acting on it
  (Rule 1: navigation, not evidence).

## Rule 5 — retired 2026-11-16 (user decision)

No GLM-5.3 or other second-opinion review is run. The check for pinned itoa is the expected
graph in `docs/phase-20261113/itoa-target-dataset/`, compared by
`test_itoa_declarations_graph_equality` (scope `declarations`) and
`test_itoa_whole_graph_equality` (scope `excluded`).
Earlier part files cite reviews made under this rule; those records stay as history.

## Rule 6 — An open graph design choice goes to the form with the most information

- The user's rule (2026-11-15): "our intention should be to provide maximum information which
  helps the coding agent code better."
- When a choice about what the graph stores is open, pick the form that tells a coding agent
  reading the graph the most (a named target over an unresolved one, an edge and a field
  over either alone, a node over a hidden record). Do not ask the user again for this kind
  of choice.
- Record each such choice as decided in the PRD ("Decisions in force") and the backlog in the
  same commit, with its date and what it costs in rows.
- More information never means a guessed fact (Rule 1). A name is stored only as far as the
  source proves it.

## Reference

### Documents

`docs/phase-20261113/` is current documentation for the graph producer. The bounded CLI
consumer phase starts at [its entry document](docs/phase-20261116/ReadMe.md); it does not
change the producer's scope, frozen expected graph or acceptance SOP.
The proposal-only Linux/macOS packaging/release plan for the pinned `codebase-memory-mcp`
non-Rust backend starts at [phase-20261117/ReadMe.md](docs/phase-20261117/ReadMe.md).
Nothing is implemented or validated; phase-20261113 data and the phase-20261116 consumer
CLI remain unchanged.

```text
SOP      SOP-itoa-full-item-graph-acceptance.md         steps S1–S4 that decide PASS
PRD      PRD-itoa-full-item-graph-scope.md              what is built and what is out of scope
ARCH     ARCH-itoa-full-item-graph-pipeline.md          pipeline, tables, graph rules
TARGET   itoa-target-dataset/                           the expected itoa graph and its builder
LOG      LOG-itoa-full-item-graph-progress.md           the progress record: a "Where we are" page, then entries (the log up to step 7.8 is in git at cb40f3d2f)
BACKLOG  BACKLOG-deferred-work-open-decisions.md        deferred work and open decisions
MAP      MAP-parseltongue-crates-as-measured.md         crates, files, verbs, stages as measured
SPEC     SPEC-itoa-full-item-graph/                     README.md is the index; one part file per
                                                        target, in build order
```

- These files state only what is in force. Edit in place; no revision sections. History goes
  in the log.
- Every document is written answer first (pyramid): the conclusion and current state on top,
  then the few points that support it, then reference detail. Short, specific, no repetition.
- One document at a time, in commits of at most about 1,000 changed lines, each pushed to
  `research/tiny-rust-code-mapper` before the next starts.
- A change to architecture or to a pin updates ARCH, SOP and this file in the same commit.
- Outside these phase directories, `docs/` is history. Exception: code and tools still read these files in
  `docs/phase-20261111/`, so do not move or delete them without updating the readers:
  `phase-one-benchmark-pin.json` (the itoa pin) and `parser-source-pin-manifest.json`. Three
  more are read only by the older five-repository tools under `tools/`:
  `corpus-source-pin-manifest.json`, `oracle-source-build-manifest.json`,
  `SPEC-phase-20261111-executable-requirements.md`.

### Commands

```bash
# build the graph for itoa
parseltongue map-workspace-full-run \
  --workspace-root-path-value /home/ubuntu/repos/rust-tools/itoa \
  --workspace-store-file-path <store.db> \
  --worker-thread-count-value 2 \
  --mapping-final-stage-kind full-item-dependency-graph \
  --sysroot-source-scope-kind declarations

# read it back; use this to study stored rows, not a temporary test
parseltongue export-published-stage-rows --workspace-store-file-path <store.db> \
  --stage-owned-table-name item-effective-visibility-records     # nodes
parseltongue export-published-stage-rows --workspace-store-file-path <store.db> \
  --stage-owned-table-name edge-evidence-record-table            # edges

# copy the two exported files into one DuckDB file (a separate binary; SPEC part 9)
parseltongue-graph-dataset-loader --node-export-file-path <nodes.json> \
  --edge-export-file-path <edges.json> --dataset-output-file-path <graph.duckdb>

# the real-source tests that decide progress
cargo test --release -p parseltongue-cli-dispatcher-binary \
  --test itoa-source-acceptance-tests -- --test-threads=1
```

- The binary has other final-stage values and verbs from earlier work (HLD publication,
  coverage banner, oracle comparison). This phase does not use them.

### Code

- Eleven crates under `crates/`, all named `parseltongue-*`. The crate map lists each one.
- `parseltongue-core-row-types` has no dependencies and stays that way.
- Side effects (files, databases, processes) live only in adapter crates, behind traits from
  `parseltongue-table-store-traits`.
- Errors are typed enums. No free-text-only variant.
- No `unwrap`/`expect` outside tests, no `unsafe`, no global mutable state.
- Write idiomatic functional Rust wherever possible: pure functions, immutable data and
  iterator pipelines, not loops that push into accumulators. Keep a loop only where a chain
  would be less clear. Optimise only after a measured miss.
- Repo skills: `.agents/skills/{rust-coder-02,idiomatic-rust-coder-01,rust-executable-specs-01}`,
  and `.agents/skills/rust-agent-mega-01` (the one-file Rust guide; read its section 1 first).

### Names

- New crates, tables, stages, CLI verbs, types, functions and tests get a four-word name that
  says what the thing is: `full-item-dependency-graph`, `FullGraphInputStorePort`,
  `test_default_new_call_roundtrip` (four words after `test_`).
- Exempt: enum variants, status values, and names owned by other projects.
- Every test carries a doc comment naming its REQ-RUST id.
- Use only names that exist in the code or in the phase-20261113 documents. The older
  `cpu-tool-*` and `tiny-llm-*` vocabulary names nothing in this code.

### Machine

```text
~/repos/parseltongue-rust-LLM-companion   this repository
~/repos/rust-tools/itoa                   the pinned source under test
~/repos/rust-tools/rust-analyzer          parser crates the build depends on
~/repos/clarity-cli                       shallow clone of the clarity CLI
```

- The build depends on `../rust-tools/rust-analyzer`: upstream commit `2a362a4` plus
  `tools/parser-const-closure-statements.patch` (commit `bf6b209`). Do not move it to a newer
  upstream commit.
- DuckDB's bundled C++ build is slow and memory-heavy. Use
  `CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CXXFLAGS=-g0`.

### Navigation aids

```bash
codebase-memory-mcp cli index_repository --repo_path /home/ubuntu/repos/parseltongue-rust-LLM-companion
codebase-memory-mcp cli index_status   --project home-ubuntu-repos-parseltongue-rust-LLM-companion
codebase-memory-mcp cli trace_path     --project home-ubuntu-repos-parseltongue-rust-LLM-companion --function-name <fn> --direction both --depth 1
codebase-memory-mcp cli search_graph   --project home-ubuntu-repos-parseltongue-rust-LLM-companion --label Function --name-pattern '.*<word>.*'
codebase-memory-mcp cli query_graph    --project home-ubuntu-repos-parseltongue-rust-LLM-companion --query "<cypher>"
clarity show crates --collapse
clarity show <file> --reach both --depth 2
```

When to run them is Rule 4. Output is a tree by default; add `--format json` before parsing
it. `search_graph --name-pattern` rejects inline regex flags such as `(?i)`.
