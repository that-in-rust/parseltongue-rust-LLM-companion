# SOP — itoa full item dependency graph

## The answer

**PASS means one thing: a production run on pinned itoa 1.0.18 writes a full item dependency
graph that equals the frozen expected itoa graph, row for row, and does so identically on
every run.**

- Scope is itoa only, and the full item dependency graph only: nodes, edges, evidence and
  unresolved sites.
- Out of scope for this phase: every other repository, any rust-analyzer export or oracle
  comparison, and every higher-abstraction table (interface level, file and folder roll-ups).
- Passing unit tests, a bounded checkpoint or a planned command are not a PASS.

Status on 2026-11-16: **all four steps passed on one frozen executable, built from commit
`822da80cf`** (run report in [the log](LOG-itoa-full-item-graph-progress.md)). Production
has changed since (SPEC part 8), so these results do not cover the branch head. No step has
been run on it.

```text
S1  freeze inputs and expected graph   PASS   digests equal the pin and the freeze record
S2  gates and production run           PASS   process audit clean; one workspace test skipped (below)
S3  exact comparison                   PASS   58 tests; 151 nodes and 682 edge rows equal the frozen graph
S4  determinism, time, memory          PASS   six runs at 1, 4 and 16 workers store identical rows
```

- The graph is marked `incomplete`: 133 method calls into `core` have no target. Each is
  an expected row (Rule 3).
- The workspace gate skips `test_lang_items_in_sysroot`, which fails when run. It reads the
  native sysroot, which itoa runs exclude. Accepted by the user on 2026-11-16 as a known
  open defect.
- The expected graph was written and checked by one author (Rule 1).
- The scope value `declarations` (SPEC part 8) is outside this SOP: see the last section.

Scope and decisions are in the [PRD](PRD-itoa-full-item-graph-scope.md); graph rules
and tables are in the [ARCH](ARCH-itoa-full-item-graph-pipeline.md); requirement ids and their
tests are in the [SPEC](SPEC-itoa-full-item-graph/README.md).

## Rule 1 — The expected graph comes from source, and is frozen before any comparison

- The expected graph is [itoa-target-dataset](itoa-target-dataset/): every item node, every
  dependency edge and every unresolved site of `src/lib.rs` and `src/u128_ext.rs` (488 lines),
  each with a source range.
- It is written by reading the pinned source. No tool output is its authority.
- It is frozen (2026-11-16). The digests of its files are in
  `itoa-expected-graph-freeze.json`; the comparison test fails when a file differs from them.
- It was written and checked against the source by one author, with no second reviewer
  (user decision, 2026-11-16). An error made the same way in the expected graph and in the
  code is not caught.
- Never edit the expected graph to match the mapper. A correction needs a source citation and
  a new freeze.

## Rule 2 — Production reads only the pinned source

- Expected rows live only in test assertions. Never feed them or fixtures into production, and
  never hardcode an edge.
- Production may start exactly three child commands: `cargo metadata --offline`,
  `rustc --print cfg` and `rustc --print sysroot`. No compilation, no build scripts, no
  network, no reads under the repository's `target/`.
- The first failing real-source test chooses the next work. Fix only the producer it needs.

## Rule 3 — Verdicts are strict

- **PASS:** nodes, edges, evidence and unresolved sites all equal the frozen expected graph. No
  extra, missing or wrong row.
- **FAIL:** any row differs, production spawns or reads something forbidden, or two runs
  differ.
- **BLOCKED:** a precondition is missing, for example the expected graph is not frozen.
- **NOT STARTED:** the producer or the check does not exist yet.
- A run the code marks `incomplete` can pass only when every row without a target is an
  expected row of the frozen graph. On pinned itoa these are the 133 method calls whose
  receiver type is declared in `core`. The report states the mark and the count.
- A PASS belongs to one executable digest. Any production change needs a new run.
- Memory and time are measured and reported on every run. This phase sets no memory cap.
- Declare the expected wall time before each command; kill it at twice that.

## Reference

### S1 — Freeze inputs and the expected graph

- Source: itoa 1.0.18, commit `af77385d0daf4d0e949e81f2588be2e44f69f086`, at
  `/home/ubuntu/repos/rust-tools/itoa`. Pin:
  [phase-one-benchmark-pin.json](../phase-20261111/phase-one-benchmark-pin.json).
- Configuration: lib target, no features, `cfg(test)` off, host and target
  `x86_64-unknown-linux-gnu`.
- itoa ships no lockfile. Create one once in a copy outside production, then run
  `cargo check --offline --locked --lib --target x86_64-unknown-linux-gnu`.
- Regenerate and test the expected graph:

```bash
python3 docs/phase-20261113/itoa-target-dataset/itoa-expected-graph-builder.py \
  --pin-file docs/phase-20261111/phase-one-benchmark-pin.json \
  --source-root /home/ubuntu/repos/rust-tools/itoa \
  --output-directory docs/phase-20261113/itoa-target-dataset
python3 -m unittest discover -s docs/phase-20261113/itoa-target-dataset -p 'test_*.py' -v
```

Pass: source commit and file digests match the pin; the expected-data tests pass; the frozen
files' digests equal the freeze record.

### S2 — Gates and production run

Run the repository gates in [AGENTS.md](../../AGENTS.md) first. Then:

```bash
REPO=/home/ubuntu/repos/rust-tools/itoa   OUT=<run output directory>
STORE="$OUT/workspace-store.db"

# 1 timing and memory (untraced)
python3 tools/process-tree-access-auditor.py sample \
  --memory-sample-output-path "$OUT/process-tree-memory-samples.json" -- \
  parseltongue map-workspace-full-run --workspace-root-path-value "$REPO" \
  --workspace-store-file-path "$STORE" --worker-thread-count-value 4 \
  --mapping-final-stage-kind full-item-dependency-graph

# 2 process and target/ audit (separate store, so tracing does not affect timing)
strace --seccomp-bpf -f -s 65535 -e trace=execve,connect,clone,clone3,fork,vfork,openat \
  -o "$OUT/production-process-access-audit.txt" \
  parseltongue map-workspace-full-run --workspace-root-path-value "$REPO" \
  --workspace-store-file-path "$OUT/audit-store.db" --worker-thread-count-value 4 \
  --mapping-final-stage-kind full-item-dependency-graph

# 3 export the graph
parseltongue export-published-stage-rows --workspace-store-file-path "$STORE" \
  --stage-owned-table-name item-effective-visibility-records > "$OUT/full-graph-nodes.json"
parseltongue export-published-stage-rows --workspace-store-file-path "$STORE" \
  --stage-owned-table-name edge-evidence-record-table > "$OUT/full-graph-edges.json"
```

Pass:

- The run publishes. `workspace_full_map_completeness_kind` in
  `workspace-config-state-records` is recorded in the report (Rule 3).
- Direct children are exactly the three allowed commands. Beneath `cargo metadata`, only
  `rustc -vV` and Cargo's stdin `--print` probe are allowed.
- No file under `target/` is read except Cargo's `.rustc_info.json`.
- Time and memory come only from the untraced run.

Add `--sysroot-source-scope-kind excluded` to both runs. A target in `core` is then stored
as a named external target, or without a target and with a reason (below).

### S3 — Exact comparison

Compare the exported nodes and edges with the frozen expected graph:

- **Nodes:** every item outside a function body, private items included; one file-area node
  per source file; one node per active `use`; items generated by a macro carry the invocation
  they came from.
- **Edges:** every dependency between items, uncollapsed, private to private included, of the
  kinds `contains`, `calls`, `uses_type`, `reads`, `implements`, `imports`, and for macros
  `generates` and `expands`. itoa has no `derives` or `reexports`; the expected count for
  both is zero.
- **Evidence:** every edge cites the exact source range of the site that proves it. An edge
  through an import targets the real item and keeps the `use` as evidence.
- **Unresolved sites:** each is stored with a typed reason and matches the expected list.

The comparison runs as real-source tests in
`crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs`:

```bash
PARSELTONGUE_FROZEN_EXECUTABLE_PATH=<the frozen executable> \
cargo test --release -p parseltongue-cli-dispatcher-binary \
  --test itoa-source-acceptance-tests -- --test-threads=1
```

- The variable is required. Without it the tests run the executable Cargo builds for the
  test, whose digest differs from the `cargo build --release` executable (measured
  2026-11-16).
- `test_itoa_whole_graph_equality` compares the whole graph: every node, every edge site by
  byte range and record, and `contains` and `implements` by source, relation and target.
- The expected graph names items by symbolic label; the test derives the same label from
  each stored identity.

### S4 — Determinism, time, memory

- Run production twice at each of 1, 4 and 16 workers. Export nodes and edges as in S2 and
  compare every line after the first. The first line of an export is its header and carries
  that run's attempt id. Any other difference is FAIL.
- Record wall time, whole-process-tree peak memory (10 ms samples) and store size.

### Report

Each run writes one entry at the bottom of [the log](LOG-itoa-full-item-graph-progress.md): source and executable
digests; each step PASS / FAIL / BLOCKED / NOT STARTED with row counts; the first differing
row; time and peak memory. A step with no entry was not observed.

### Targets outside itoa (decided 2026-11-15)

**Targets outside itoa.** Resolving a `core` target to an item needs the sysroot source in the run; on
2026-11-13 that run did not finish name resolution in 2,400 s. Decided: the sysroot stays out of
the run. An edge into `core` matches when production stores the same named external target
as the expected row: the name as far as itoa's source proves it, the crate, and the flag that
the target's source was not read. A site whose target name source cannot prove matches an
expected unresolved row with the same reason.

### The scope value `declarations` (SPEC part 8): not covered

`--sysroot-source-scope-kind declarations` reads the `core` source under
`<sysroot>/lib/rustlib/src/rust/library`, syntax only, to name method calls into `core`. A run
with it cannot pass this SOP as written:

- **Rule 2** names the pinned source as the only source production reads.
- **S1** does not freeze the `core` files, and the run id hashes the compiler version, not
  those files.
- **S2** has no rule for reads under the sysroot source directory.
- **The expected graph** lists those method calls without a target.

Before such a run is judged, this SOP is changed in the same commit as the ARCH and
`AGENTS.md`, and the expected graph is re-frozen from source.
