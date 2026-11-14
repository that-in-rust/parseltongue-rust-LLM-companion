# SOP — full-item-dependency-graph qualification

## The answer

**This procedure alone decides PASS. A milestone passes when every applicable step S1–S10
passes on every repository in scope, using one frozen release executable.**

- Phase 1 scope: pinned itoa 1.0.18. Phase 2 scope: serde, rust-analyzer, ripgrep, tokio,
  cargo-modules, in that order.
- Passing tests, a bounded checkpoint or a planned command are not a PASS.

Status on 2026-11-14: **no step has passed.**

```text
S1      NOT STARTED
S2      NOT STARTED
S3      BLOCKED       itoa with sysroot did not finish name resolution in 2,400 s
S4–S10  NOT STARTED
```

The S3 result was measured on 2026-11-13 (619 units parsed, exit 124, nothing published). Its
run report has since been deleted, so S3 must be re-measured before any work depends on it.

What exists is a bounded, sysroot-excluded checkpoint on itoa that the code itself marks
incomplete. Its numbers are in the top entry of [the log](phase1-log-p1.md).

Requirement ids and formulas are in the [SPEC](SPEC-phase-20261113-executable-requirements.md);
scope is in the [PRD](PRD-phase-20261113-dependency-graph-hld.md); graph rules are in the
[ARCH](ARCH-phase-20261113-pipeline-graph-tables.md).

## Rule 1 — Three witnesses, each with fixed authority

- **`parseltongue`** is the thing under test. It never grades itself.
- **`tools/oracle-semantic-export-harness`** is the answer key. It is built from the pinned
  rust-analyzer: upstream `2a362a4` plus `tools/parser-const-closure-statements.patch`, commit
  `bf6b2099d6351c37039b01447a9bb64f9a54a586`. An installed rust-analyzer is not the oracle.
  - PRIMARY export (build scripts off) is the only scoring denominator.
  - SECONDARY export (build scripts on) only identifies sites that depend on build scripts.
- **`codebase-memory-mcp`** is a structural cross-check. A gap or a heuristic edge in it never
  overrides the oracle and never excuses an oracle-proven error.
- For Phase 1 there is a fourth: the **frozen expected itoa graph** (Rule 2).

Production boundary, checked in S3: the mapper may start exactly three direct child commands,
`cargo metadata --offline`, `rustc --print cfg` and `rustc --print sysroot`. No compilation, no
build scripts, no network, no reads under the repository's `target/`.

## Rule 2 — Phase 1 is an exact comparison on itoa, built in this order

1. **Freeze inputs.** Lib target, no features, `cfg(test)` off, host and target
   `x86_64-unknown-linux-gnu`. Pin: [phase-one-benchmark-pin.json](../phase-20261111/phase-one-benchmark-pin.json).
   itoa ships no lockfile: create one once in a qualification copy outside production, then run
   `cargo check --offline --locked --lib --target x86_64-unknown-linux-gnu`.
2. **Freeze the expected graph before comparing.** Review every node, edge, unresolved site and
   inactive `cfg` branch of both source files (`src/lib.rs`, `src/u128_ext.rs`; 488 lines)
   against source and the oracle. Serialize, hash, freeze.
   [itoa-target-dataset](itoa-target-dataset/) is today's unreviewed draft, not that reference.
3. **Run production on the pinned source.** Expected rows live only in test assertions. Never
   feed them, oracle output or fixtures into production, and never hardcode an edge.
4. **Compare exactly.** Nodes, edges, evidence, unresolved and inactive sets must be equal. No
   unexplained extra, missing or wrong row. Never edit the expected graph to match the mapper;
   a correction to it needs a justified new freeze.
5. **Let the first failing real-source test choose the next work.** Fix only the producer it
   needs.

Phase 1 exit: frozen expected graph, exact agreement from a production run with sysroot
included, valid evidence on every row, and identical output at 1, 4 and 16 workers.

`--sysroot-source-scope-kind excluded` is a local diagnostic. Edges into `core` are then stored
unresolved with a reason. Such a run cannot pass Phase 1.

Constructs itoa does not contain (proc macros, async, re-exports) stay unexercised. Phase 2
covers them.

## Rule 3 — Verdicts are strict

- **PASS:** every applicable check and every per-repository threshold passes.
- **FAIL:** a contract fails, a sampled row is wrong, production spawns or reads something
  forbidden, output differs between runs, or a mandatory gate is missed.
- **BLOCKED:** a precondition, coverage page or oracle export is missing. Never counted as PASS.
- **NOT STARTED:** the stage does not exist yet. Never counted as PASS.
- An empty denominator is BLOCKED, not 100%.
- No pooling across repositories. Each repository passes on its own numbers.
- A PASS belongs to one executable digest. Any production change needs a new run.
- Do not patch production during a run. Fix, then start a new run.
- Memory is measured on every run with no interim cap. After all five repositories pass S5,
  whole-process-tree peak must be ≤ 500,000,000 bytes.
- Declare the expected wall time before each command; kill it at twice that.

## Reference

### Preconditions

- **P1** Pin sources: itoa for Phase 1; the five repositories in
  [corpus-source-pin-manifest.json](../phase-20261111/corpus-source-pin-manifest.json) for
  Phase 2. Record commit, lockfile digest and tool versions.
- **P2** Qualify the selected lib and bin targets once with offline `cargo check`, outside
  production.
- **P3** Provision dependencies, matching sysroot source and proc-macro artifacts offline. A
  missing proc-macro expansion is BLOCKED.
- **P4** Build the oracle harness from the pinned commit; record source commit, executable
  digest and build manifest.
- **P5** Record machine, OS, memory accounting method and the production executable digest.

Mapper and both oracle exports use the same targets, features, `cfg`, edition and host/target.
Tests, benches, examples and dev-dependencies are out of scope.

```bash
REPO=<absolute checkout>   CMP=<codebase-memory project>   OUT=<run output directory>
STORE="$OUT/workspace-store.db"
```

### S1 — Structural index

```bash
codebase-memory-mcp cli index_repository --repo-path "$REPO" --mode full
codebase-memory-mcp cli index_status --project "$CMP"
codebase-memory-mcp cli get_graph_schema --project "$CMP"
```

Pass: the index is `ready` for the pinned checkout; counts recorded. Re-index after any source
change.

### S2 — Coverage inventory

```bash
codebase-memory-mcp cli check_index_coverage --project "$CMP" --scopes '["."]' \
  --scope-limit 20 --scope-offset 0 --diagnostics full --format json
codebase-memory-mcp cli query_graph --project "$CMP" --graph missed \
  --query "MATCH (f:File) RETURN f.file_path" --max-rows 200 --format json
```

Pass: every page fetched (`has_more=false`), entries add up to `scope_total`, each page saved. A
missed file is missing cross-check evidence only.

### S3 — Gates and production run

Run the repository gates in [AGENTS.md](../../AGENTS.md) first. Then three separate runs, each
with its own store, so measurement and tracing do not interfere:

```bash
# 1 timing and memory (untraced)
python3 tools/process-tree-access-auditor.py sample \
  --memory-sample-output-path "$OUT/process-tree-memory-samples.json" -- \
  parseltongue map-workspace-full-run --workspace-root-path-value "$REPO" \
  --workspace-store-file-path "$STORE" --worker-thread-count-value 4 \
  --mapping-final-stage-kind full-item-dependency-graph

# 2 process audit
strace --seccomp-bpf -f -s 65535 -e trace=execve,connect,clone,clone3,fork,vfork \
  -o "$OUT/production-process-access-audit.txt" \
  parseltongue map-workspace-full-run --workspace-root-path-value "$REPO" \
  --workspace-store-file-path "$OUT/audit-store.db" --worker-thread-count-value 4 \
  --mapping-final-stage-kind full-item-dependency-graph

# 3 target/ read audit
strace --seccomp-bpf -f -s 65535 -e trace=openat -P "$REPO/target" \
  -o "$OUT/production-target-access-audit.txt" \
  parseltongue map-workspace-full-run --workspace-root-path-value "$REPO" \
  --workspace-store-file-path "$OUT/target-store.db" --worker-thread-count-value 4 \
  --mapping-final-stage-kind full-item-dependency-graph

parseltongue status-coverage-banner-show --workspace-store-file-path "$STORE" --json
parseltongue export-published-stage-rows --workspace-store-file-path "$STORE" \
  --stage-owned-table-name item-effective-visibility-records > "$OUT/full-graph-nodes.json"
parseltongue export-published-stage-rows --workspace-store-file-path "$STORE" \
  --stage-owned-table-name edge-evidence-record-table > "$OUT/full-graph-edges.json"
```

Pass:

- All stages for the milestone are marked complete and published. Limit today: the banner
  reports publication for the first four stages only and hard-codes signature lowering, call
  resolution and visibility as "not yet implemented". For a full-graph run, read
  `workspace_required_stage_names` and `workspace_full_map_completeness_kind` from
  `workspace-config-state-records`; the value `incomplete` is not a pass.
- Direct children are exactly the three allowed commands. Beneath `cargo metadata`, only
  `rustc -vV` and Cargo's stdin `--print` discovery probe are allowed.
- No file under `target/` is read except Cargo's `.rustc_info.json`.
- The lockfile digest is unchanged; a missing lockfile is an explicit validation error.
- Time and memory come only from the untraced run.
- Before DuckDB analysis starts: only a published, non-cancelled generation is copied; row
  counts and content digests match on both sides; DuckDB runs with extension install/load and
  external access off, a run-local temp directory, and its effective `memory_limit` reported.

### S4 — Oracle exports

```bash
parseltongue export-mapped-unit-manifest --workspace-store-file-path "$STORE" > "$OUT/mapped-unit-manifest.json"

oracle-semantic-export-harness --workspace-source-root-path "$REPO" \
  --mapped-unit-manifest-path "$OUT/mapped-unit-manifest.json" \
  --oracle-artifact-cache-output "$OUT/oracle-artifact-cache.json"

for KIND in PRIMARY SECONDARY; do
  oracle-semantic-export-harness --workspace-source-root-path "$REPO" \
    --mapped-unit-manifest-path "$OUT/mapped-unit-manifest.json" \
    --oracle-macro-cache-manifest "$OUT/oracle-artifact-cache.json" \
    --oracle-configuration-kind "$KIND" \
    --oracle-path-export-output "$OUT/oracle-$KIND-path-export.json"
done
```

Pass: both exports come from the same manifest, cache and source generation (digests recorded),
with proc macros on and test `cfg` off.

Limit today: the harness exports path-resolution sites only. Macro expansion items, impl
relations, method and call targets, signature references and visibility are required for full
S4 and are not exported yet. Until they are, S4 and S5 cannot pass beyond path sites.

### S5 — Per-site comparison

```bash
parseltongue oracle-edge-differ-run --workspace-store-file-path "$STORE" \
  --primary-semantic-input-path "$OUT/oracle-PRIMARY-path-export.json" \
  --secondary-semantic-input-path "$OUT/oracle-SECONDARY-path-export.json" \
  --agreement-store-file-path "$OUT/oracle-edge-agreement-store.db"
```

Per-repository gates, with PRIMARY sites as the denominator:

```text
REQ-RUST-005  path resolution                         ≥ 99.5%
REQ-RUST-006  macro expansion items and body sites    ≥ 99%
REQ-RUST-013  proc-macro shape expansions             = 100%
REQ-RUST-018  impl relations                          ≥ 99.5%
REQ-RUST-007  method targets, precision               ≥ 99%
REQ-RUST-008  method targets, coverage                ≥ 95%   (mandatory at M8)
REQ-RUST-019  selection, visibility, reachability     = 100%
```

- Every site gets exactly one result and, if unresolved, the earliest reason in this order:
  `cfg_inactive`, `build_script`, `proc_macro`, `external_source_missing`,
  `unknown_receiver`, `trait_solver_needed`.
- Graph edge rows currently carry a different reason type (`HldUnresolvedReasonKind`). Until the
  PRD open point "Edge reasons" is settled, unresolved edges cannot be scored and that part of
  the comparison is BLOCKED.
- Inactive and PRIMARY-absent sites are exclusions. A PRIMARY-visible unresolved site is a miss.
- A wrong extracted target is never excused by a reason.
- Report raw counts, exclusions and rates.

### S6 — Structural cross-check

Compare `.rs` file, item and impl counts from codebase-memory with production for the same
active source set. Explain every difference (inactive `cfg`, body items, macro expansion). A
difference prompts an oracle check; it does not decide pass or fail by itself.

### S7 — Edge triage

Use codebase-memory `CALLS` edges only to locate candidates, then score production rows against
PRIMARY per site:

```text
AGREE      correct
PT-EXTRA   extracted but wrong              method rate must be ≤ 1%
PT-BUG     PRIMARY-visible, not extracted   counts against REQ-RUST-008
KNOWN-GAP  excluded for a verified reason
```

### S8 — Sampled evidence review

- Sample `min(50, row count)` edge rows without replacement, seed 42: rank row keys by
  BLAKE3-256 of seed, key length and key bytes; take the smallest.
- Open the stored source and evidence for each. One false edge or missing evidence is FAIL.
- Zero rows gives zero samples and BLOCKED, not a pass.

### S9 — Graph proof

Full graph, active now:

- Every item outside a function body is a node, private items included.
- One file-area node per source file; one node per active `use`.
- Every extracted edge is present uncollapsed, including private to private.
- An edge through an import targets the real item and keeps the `use` as evidence.
- Visibility and outside-crate reachability match PRIMARY, or are stored as unknown.

Derived levels, applied only once they are built: every derived node, edge and path hop exists
in the full graph, and no collapsed path contains a `contains` or `imports` hop.

### S10 — Determinism, memory, time

- Run production twice at each of 1, 4 and 16 workers. Export the node and edge tables as in S3
  and compare with `cmp`. Any difference is FAIL.
- Every query whose output is consumed has a total `ORDER BY`.
- Record whole-process-tree peak memory (10 ms samples), wall time per stage and disk size.

### Milestone subsets

```text
M0   S3 build and identity fixtures; S4 harness build only
M1   S1, S2, S3 input/store fixtures; S10 input-row determinism
M2   S1–S3; S4/S5 path gates, proc-macro sites excluded and reported
M3   M2 plus proc-macro shape gates, exclusions removed
M4   M3 plus signatures and impl headers; S6 impl triage
M5   M4 plus body inference precision; S7
M6   M5 with traits in scope; ≥ 95% coverage on all five allows skipping M7
M7   M6 with the solver adapter
M8   full S1–S10 and every SPEC contract
```

`--mapping-final-stage-kind` accepts `syntax-spans` (M1-only run), `name-resolution` (the
default), `full-item-dependency-graph`, and `original-top-level-containment` (an earlier
checkpoint that publishes HLD tables for top-level containment only; not used for any gate).

### Runner and report

```bash
python3 tools/hld-current-qualification-runner.py --run-id <run-id> --milestone M2 \
  --mode execute --mapper-binary target/release/parseltongue
```

- The runner supports milestones M1 and M2 only. Later milestones are run by hand from the
  steps above until it is extended.
- Report: `hld-current-test-runs/<run-id>/hld-current-test-report.md`, committed. Large
  artifacts: `~/hld-current-test-artifacts/<run-id>/`, not committed; the report records each
  path and digest. No run exists in either folder today.
- The report states: phase; source, lock, tool and executable digests; stages completed; per
  step PASS / FAIL / BLOCKED / NOT STARTED with counts, rates and exclusions; the first failing
  check; time and peak memory; the sample seed and keys.
- A gate with no report entry was not observed.
