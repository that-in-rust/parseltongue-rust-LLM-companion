# AGENTS.md

Rules for any agent working in this repository. Read top to bottom: the answer first,
then the three rules that follow from it, then reference facts.

## The answer

**Build one thing: the `full-item-dependency-graph` of pinned itoa 1.0.18, produced
deterministically by Rust code from the real source.**

It is DONE only when a production run equals the frozen expected itoa graph, as the SOP
defines. Until then, report it as incomplete.

Out of scope for this phase: every other repository, any rust-analyzer export or oracle, and
every higher-abstraction table (interface level, HLD, roll-ups). Do not build, extend or test
against them.

Where we are (2026-11-14): a bounded checkpoint on pinned itoa, marked `incomplete` by the
code itself. Current numbers and the next target are the top entry of
[the log](docs/phase-20261113/phase1-log-p1.md). Read it before answering "how far".

## Rule 1 — Only source-backed facts count

- A relationship is proven only when the production CLI derives it from pinned source, persists
  it, and a real-source test reads it back with exact file, range and digest.
- Unit tests, fixtures and builds passing are progress, never done.
- Never feed expected data or the target dataset into production, and never hardcode an
  expected edge.
- Never guess. Unknown visibility, unresolved calls, inactive `cfg` items and unsupported forms
  are stored explicitly with a typed reason.
- Any production change after a run invalidates that run.
- Status is given per SOP step (PASS / FAIL / BLOCKED / NOT STARTED) with measured numbers.
  No bare percentages, no "complete" for a bounded result.
- `codebase-memory-mcp` and `clarity` show structure. They are navigation aids, not evidence.
- `docs/phase-20261113/itoa-target-dataset/` is the expected graph. Today it is an unreviewed
  draft; it becomes the reference only after review against source and a freeze (SOP S1).

## Rule 2 — The itoa full graph only; everything else is backlog

Graph rules in force:

- Every item outside a function body is a node, private items included.
- Each source file has one file-area node. Each `use` is a node.
- Edges are stored uncollapsed. Relation kinds: `contains`, `calls`, `imports`, `uses_type`,
  `reads`, `implements`, `reexports`, `derives`.
- An edge targets the real item, never the `use` line.
- Impl blocks are records, not nodes. Body-local items are evidence, not nodes.
- An impl member has one lexical container: its file area or enclosing inline module. Impl,
  self-type and trait ownership are separate metadata.

Ownership:

- Rust: parsing, macro expansion, call resolution, identities.
- DuckDB: name resolution, node projection, relational analysis.
- Turso: durable storage, frozen snapshots, atomic publication, serving.
- No third database.

Scope:

- This phase: the full graph and its evidence on pinned itoa 1.0.18. Nothing else.
- Interface levels, collapse and roll-ups are later queries over the stored graph. They are
  not part of this phase.
- Anything not needed for the itoa full graph goes in
  [the backlog](docs/phase-20261113/phase1-abstractions-decision-backlog.md). Do not implement an
  undecided choice; ask, or backlog it.
- `--sysroot-source-scope-kind excluded` is how runs work today. How edges into `core` are
  judged is an open decision in the PRD; until decided, such a run cannot pass.

## Rule 3 — One VM, one branch, test first, measure

- **One VM.** All work, builds and tests run on the current session VM. No child sessions, no
  extra worktrees or clones.
- **One branch.** `research/tiny-rust-code-mapper`. `git pull --rebase` before pushing (the user
  also commits from a Mac).
- **Small commits.** One focused change per commit, at most about 200 changed lines, with a
  message that says what changed and why. Push after each. Do not hold work back for a full
  rewrite. Only a delete-only commit may be larger.
- **Test first.** Write a failing real-source test on pinned itoa, make it pass with the smallest general change, refactor. A RED or GREEN
  claim needs a nonzero selected-test count and a test that reached the intended behaviour.
- **Gates before every push of code:**

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
- **Log.** After each GREEN or REFACTOR commit, add a newest-first entry to the log: test names, exact command, pass/fail counts, commit, next target.

## Reference

### Documents

`docs/phase-20261113/` is the only current documentation.

```text
SOP      SOP-itoa-full-item-graph-acceptance.md    steps S1–S4 that decide PASS
PRD      PRD-itoa-full-item-graph-scope.md          what is built and what is out of scope
ARCH     ARCH-itoa-full-item-graph-pipeline.md        pipeline, tables, graph rules
TARGET   itoa-target-dataset/                                the expected itoa graph and its builder
LOG      phase1-log-p1.md                                    the single progress record, newest first
BACKLOG  phase1-abstractions-decision-backlog.md             deferred work and open decisions
MAP      current-codebase-crate-map.md                       crates, files, verbs, stages as measured
SPEC     SPEC-wider-scope-requirements-not-current.md      NOT current: written for the wider
                                                             scope; to be rewritten after ARCH
```

- These files state only what is in force. Edit in place; no revision sections. History goes
  in the log.
- Every document is written answer first (pyramid): the conclusion and current state on top,
  then the few points that support it, then reference detail. Short, specific, no repetition.
- One document at a time, in commits of at most about 200 changed lines, each pushed to
  `research/tiny-rust-code-mapper` before the next starts.
- A change to architecture or to a pin updates ARCH, SOP and this file in the same commit.
- Everything else under `docs/` is history. Exception: code and tools still read these files in
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
  --mapping-final-stage-kind full-item-dependency-graph

# read it back
parseltongue export-published-stage-rows --workspace-store-file-path <store.db> \
  --stage-owned-table-name item-effective-visibility-records     # nodes
parseltongue export-published-stage-rows --workspace-store-file-path <store.db> \
  --stage-owned-table-name edge-evidence-record-table            # edges

# the real-source tests that decide progress
cargo test --release -p parseltongue-cli-dispatcher-binary \
  --test itoa-source-acceptance-tests -- --test-threads=1
```

- The binary has other final-stage values and verbs from earlier work (HLD publication,
  coverage banner, oracle comparison). This phase does not use them.

### Code

- Ten crates under `crates/`, all named `parseltongue-*`. The crate map lists each one.
- `parseltongue-core-row-types` has no dependencies and stays that way.
- Side effects (files, databases, processes) live only in adapter crates, behind traits from
  `parseltongue-table-store-traits`.
- Errors are typed enums. No free-text-only variant.
- No `unwrap`/`expect` outside tests, no `unsafe`, no global mutable state.
- Prefer pure functions, immutable data and iterator combinators. Optimise only after a
  measured miss.
- Repo skills: `.agents/skills/{rust-coder-02,idiomatic-rust-coder-01,rust-executable-specs-01}`.

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
codebase-memory-mcp cli trace_path     --project home-ubuntu-repos-parseltongue-rust-LLM-companion --function-name <fn>
codebase-memory-mcp cli query_graph    --project home-ubuntu-repos-parseltongue-rust-LLM-companion --query "<cypher>"
clarity show crates --collapse
clarity show <file> --reach both --depth 2
```

Re-index after large changes. `search_graph --name_pattern` rejects inline regex flags such
as `(?i)`.
