# SOP — phase-20261113 full-item-dependency-graph validation

> The PRD and SPEC this procedure executes are now
> `docs/phase-20261113/PRD-phase-20261113-dependency-graph-hld.md` and
> `docs/phase-20261113/SPEC-phase-20261113-executable-requirements.md`. Requirement ids are
> unchanged. SPEC section numbers cited below (§2.1-§2.4, §3.2, §4, §8) are the earlier
> numbering; the SPEC's id-history section maps them.

Name: full-item-dependency-graph. Status: planned executable procedure, not a record of passing runs.
Inputs: the phase-20261113 PRD and SPEC named above.
SPEC owns requirement/test names and scoring; PRD owns scope; this SOP applies those contracts.
Full S1–S10 runs apply at M8/releases. Intermediate milestone subsets are listed in §3.
Phase 1 development is driven by the pinned-itoa real-source vertical acceptance test in §1.1;
component tests support it but do not substitute for source acceptance.
Phase 2 retains every applicable corpus gate on all five repos, never a tune/held-out split.
The initial one-relation checkpoint is progress only, not whole-itoa/golden/SOP PASS or an all-five
milestone/release PASS.

## 0. Witnesses and boundaries

- Parseltongue is the production mapper under test: CPU-only, tables between stages.
- oracle-semantic-export-harness is the answer key, built from rust-analyzer's pinned checkout in SPEC §8.
  Its offline ide/hir/salsa dependencies are allowed ONLY in the harness. Installed rust-analyzer 1.99.1
  (58cf8f9) is not the oracle for checkout 2a362a41a.
- codebase-memory-mcp 0.12.1 is a structural/coverage cross-check, not an answer key.
  Heuristic and missing edges never override PRIMARY oracle evidence.
- SCIP is optional secondary occurrence evidence; method references are useful, but SCIP's empty
  relationship export does not prove impl/expansion/visibility relations.
- The direct-child three-command allowlist applies to production mapping only:
  cargo metadata --offline, rustc --print cfg, rustc --print sysroot.
  Only beneath metadata Cargo, permit rustc -vV or the stdin ___ probe with the six fixed
  --crate-type selectors, approved --print= flags and -Wwarnings, without output artifacts.
  All other descendants, compilation, build scripts and network fail.
  Corpus qualification, harness builds/exports, version collection and test drivers are outside it.

## 1. Preconditions

P1: Phase 1 pins itoa using §1.1. Phase 2 pins clean corpus commits:
rust-analyzer, ripgrep, tokio, serde, cargo-modules (SPEC §8), running serde first.
Record full hashes, lockfile digest, source snapshot and all tool/config versions.
P2: qualify selected workspace lib+bin targets once with offline cargo check outside production.
Do not infer that unrelated fixtures or inactive modules parse.
P3: provision downloaded normal dependencies, matching sysroot sources and real proc-macro artifacts offline.
PRIMARY needs proc macros available without executing build scripts in its export. Record artifact hashes;
missing proc-macro expansions BLOCK gates, never fall back to the installed oracle binary.
P4: build oracle-semantic-export-harness from SPEC §8's full source commit; record source commit,
executable digest and build manifest. Both export modes have proc macros enabled and test cfg OFF.
P5: record machine/OS, memory accounting method and production executable digest.
Use identical lib+bin/default-feature/per-unit cfg/edition/host-target policy for mapper and both oracle exports.
Tests/benches/examples/dev-dependencies are out; build contexts are provenance only.

Placeholders: $REPO absolute checkout, $CMP indexed project, $OUT persistent output directory:
hld-current-test-runs/<run-id>/<repo-name>. Create it before commands.
Owned command/flag names below are planned contracts, not claims that implementations already exist.
External tool flags/property names keep their upstream spelling.

### 1.1 Phase 1 — pinned-source vertical HLD acceptance

Canonical procedure location is now phase-20261113; the old phase-20261111 SOP is a redirect.
REQ-RUST-045.0 permits explicitly incomplete full-map development checkpoints. Their source,
node and edge snapshots must remain compatible, with incomplete state preserved on readback;
unknown visibility and unsupported calls are not guessed. No HLD table or eligibility pass is
required for this bounded checkpoint, and existing HLD publication gates remain unchanged.

The heading is retained for existing links; the active checkpoint is now the **detailed full
map**, not HLD. Higher-level abstraction work and irrelevant decisions are in the
[backlog](../phase1-abstractions-decision-backlog.md). Retained HLD rules apply when that work
resumes, not to Phase 1 exit. This scope change does not weaken semantic coverage or publication
safety. **Use only [dtolnay/itoa 1.0.18](https://github.com/dtolnay/itoa/tree/af77385d0daf4d0e949e81f2588be2e44f69f086)
for Phase 1.** Its immutable source/configuration policy is
[phase-one-benchmark-pin.json](../phase-20261111/phase-one-benchmark-pin.json). Keep the five-repository
[corpus manifest](../phase-20261111/corpus-source-pin-manifest.json) unchanged for Phase 2.

**First vertical checkpoint — one source-verified relationship.** Independently
verify in the pinned itoa source that `<Buffer as Default>::default` delegates
to `Buffer::new`. Then implement a real-source integration regression that feeds
the pinned source through the production pipeline and asserts the relationship
using full compilation-unit and occurrence identities plus source evidence
(source path, ranges, digests, and applicable expansion/invocation provenance).

The retained bounded diagnostic increment is REQ-RUST-040.0's explicit
`original-top-level-containment` stage: cfg-active original-file top-level ownership only, with a
separate file-area identity. It excludes nested/body/associated/expanded items and semantic
dependency relations. Its coverage marker must remain orthogonal to typed HLD materialization
completeness; this increment is not a whole-source or semantic-completeness pass.
Do not widen it into the detailed-map product. The next active producer slice is original
item/member nodes with one source container and separate impl ownership, then the call above.
The expected relation belongs only in test assertions: production must consume
actual pinned source, not pre-resolved semantic fixture inputs, and must not
hardcode the expected edge. This first relation is a diagnostic progress
checkpoint only; it does not establish that the production pipeline emits it,
complete itoa coverage, or qualify the HLD.

Before calling a run RED or GREEN, list the selected test IDs, verify the filter
selects at least one test, and confirm the real-source fixture reached the
intended behavior. A compile-only build, zero-test filter, or setup failure is
not behavioral evidence. Keep Cargo packages, features, profile, target,
toolchain, and lock inputs stable within a test cycle; compile dependencies
separately from direct test execution, avoid repeated arbitrary short cold-build
timeouts, and record expected/actual time and process-tree RSS. Do not add a
memory cap against the current user override.

Measured at commit `af77385d0daf4d0e949e81f2588be2e44f69f086`:
- Entire library source: `src/lib.rs` (466 lines) and `src/u128_ext.rs` (22 lines):
  **488 lines / 17,599 bytes**, including comments and blank lines.
- One Rust 2021 library target. The selected empty feature set has no enabled
  third-party normal dependencies, no build script and no enabled proc macro.
  Optional `no-panic` is OFF; tests, benches, examples, fuzz and dev-dependencies
  are outside the semantic target. Cargo lock resolution may still need their
  manifests/packages; do not claim that offline provisioning is already complete.
- Small enough to read in full, but nontrivial: public `Buffer` methods, the
  `Integer` trait, sealed/private helpers, declarative macro-generated impls,
  primitive types, cfg-dependent pointer-sized impls and external `core` targets.
  Macros, generics and unsafe syntax are **not** excluded to make the graph easier.

**Fixed scope and preflight**

1. Read both library files in full, including private bodies. Use the lib target,
   empty enabled features, `cfg(test)=false`, and x86_64-unknown-linux-gnu for both
   host and target. Enumerate all active declarative expansions, including the
   64-bit pointer-width branches; account for inactive cfg branches explicitly.
   This is the complete selected library, not a handpicked source excerpt.
2. Before an accepted run, freeze rustc/Cargo versions, sysroot source identity,
   full per-unit cfg/features/edition/role, source digests and Cargo lock digest.
   There is no tracked upstream lockfile: provision and retain one once in a
   qualification copy outside production, then use it unchanged with
   `cargo check --offline --locked --lib --target x86_64-unknown-linux-gnu`.
   A missing lock/cache, unresolved per-unit context or mismatched identity is
   BLOCKED. Do not alter the upstream manifest or enable `no-panic` to provision it.
3. Keep `core`/primitive dependencies as canonical external endpoints. External
   definition evidence and sysroot identity are required, but the expected full map
   does not recursively enumerate all sysroot internals as benchmark-local nodes.
   This scope does not authorize stubbing production resolution or bypassing its
   process/dependency rules. A small source repo does not itself fix the native
   resolver timeout.

**Build the expected table first — source-only Pass A**

An explicitly partial comparison against independently source-verified facts
is allowed during diagnosis, but it must be labeled partial diagnostic evidence,
never a frozen reference or golden. Freeze the complete expected graph only
after the full selected source universe and evidence have been reviewed.

The current itoa rebuild under `docs/phase-20261113/itoa-target-dataset/` is
reference-only: retain the complete proposed candidate inventory beside a
separate eligible local-only diagnostic subset and explicit blocked/proposed
rows. The five expected HLD outputs are retained deferred artifacts, not active comparison
targets. Do not turn unknown visibility, external labels,
inferred core-dependent calls, macro file assignment or unresolved OD-2/OD-7/OD-8-dependent
rows into extracted expectations. OD-9 and roll-up attribution are deferred; actual generated
identity and source/container provenance remain active. Keep one table-contract manifest
top-down from impl/signature through namespace/name resolution, body/signature
sites and spans; unbuilt schemas are `BLOCKED/NOT_IMPLEMENTED`.

OD-7's original-item source containment was decided on 2026-11-14 (REQ-RUST-043.0):
file-level impl members use their file area; nested impl members use the enclosing inline
module. Impl/self-type/trait ownership stays separate metadata, not a second contains parent.
This does not decide the file-module/area relationship or generated-file attribution.
Previously blocked reference rows require source review before promotion; do not bulk-unblock
them. The ban on contains/imports collapse and the all-five qualification gates are unchanged.

Backward verification uses immutable baseline export artifacts, never a
database write. Require one compatible generation header and explicit stage
markers; an export error is `ERROR`, a missing marker is incomplete, and row
counts or heuristic matches cannot pass. The default-to-new checkpoint records
the exact pinned source byte range and separately reports syntax-site evidence
versus a missing semantic resolution row. This rebuild does not freeze the
candidate draft or make a production comparison.

4. An LLM may propose the expected full-map nodes, direct relations and source evidence
   from the pinned source/configuration; codebase-memory is navigation only.
   Start this before waiting for the production M8 exporter. Do not feed mapper
   output into reference construction. Apply REQ-RUST-032.0/043.0/044.0: private/public nodes,
   canonical endpoints, source containers, direct facts and exact evidence. Preserve visibility
   as metadata; do not filter nodes, collapse private paths or construct roll-ups for this gate.
5. For the current reference-only rebuild, keep
   `itoa-full-graph-nodes.json` and `itoa-full-graph-edges.json` as the complete
   candidate inventory, `itoa-local-verification-subset.json` as the separate
   eligible local-only slice, `itoa-blocked-candidate-index.json` as blocked
   expectations, and `itoa-table-contract-manifest.json` beside them. Retain the five
   `itoa-expected-*.json` outputs as deferred history, not active acceptance inputs. These are symbolic draft
   artifacts, not canonical identities or production schemas. Preserve exact
   source byte ranges/digests where verified; macro invocation ancestry and file
   assignment remain blockers until their policy/evidence is resolved.
6. Check every declared source scope, generated item/site and claim against source
   and deterministic pinned-oracle evidence where applicable. Account for the
   entire selected universe, including non-selected items; an output-only sample
   cannot establish completeness. Unsupported identity/expansion mappings or
   unresolved claims remain BLOCKED/DRAFT, not an empty accepted table. Preserve
   raw model output separately from reviewed corrections and record model/prompt
   provenance. The model is not acceptance authority.
7. Canonically serialize, hash and freeze the reviewed reference **before** Pass B.
   Report expected-set counts and scope/exclusions. Do not rewrite expected rows
   to match a failing mapper. A source/config/semantic-reference correction needs
   a separately justified reference revision and a new freeze; a mapper-only
   change reuses the same frozen reference.

**Constant regression loop — frozen Pass B**

The graph-first architecture and table ownership are specified in the
[current architecture](../phase-20261113/ARCH-phase-20261113-pipeline-graph-tables.md)
and SPEC REQ-RUST-032.0/043.0/044.0. For Phase 1 implementation, run the real-source
regression first and let its first missing production stage determine the next
general-purpose producer path. Fix only the producer work required to emit that
relation with evidence; do not complete unrelated downstream infrastructure
first. Synthetic semantic-input fixtures may test a component, but are not Pass
B and cannot substitute for a production run on the pinned source. The
source-reference draft is not accepted output until step 7's complete freeze.
Keep local item visibility distinct from outside-crate reachability; public
`Sealed` members and `pub(crate) mulhi` are not excluded by their private modules.
An absent external definition file is not a guessed local file and does not remove the edge.
Query-only bucket/roll-up decisions are deferred; endpoint identity and evidence are not.

8. After each production change affecting mapping, identities or evidence, run the
   affected focused tests and the available M0–M8 subset below on this same pin.
   After each integration checkpoint, rerun all currently implemented applicable
   stages on itoa. Retain one new report and executable digest per run. This
   defines the required loop; no scheduler/CI job or runner support is claimed
   to be implemented by this documentation change.
9. Compare intermediate sets and, once available, the complete scoped full-map node/edge/
   evidence and unresolved/inactive sets against the frozen reference. For this small golden
   benchmark require exact semantic equality: no unexplained extra/missing/wrong
   row, no invented extracted edge and no unaccounted scope. HLD equality and file/folder
   counters are deferred, not alternative evidence for full-map correctness. Differences require source-grounded adjudication, not
   a relaxed denominator. Retain the existing PRIMARY and S8/S9 authority.
10. Report each SOP step as PASS / FAIL / BLOCKED / NOT STARTED with counts,
    discrepancies, hashes, time and whole-process-tree RSS. Declare expected time
    E before each command and kill at 2E; memory remains measured-only under R9.
    Unsupported future stages stay NOT STARTED; missing prerequisite evidence
    is BLOCKED. No fake empty rows, placeholder keys or zero-denominator PASS.

**Phase 1 exit:** a reviewed, fully accounted frozen expected full graph plus a
same-context production full-map export with exact agreement and valid detailed evidence,
and deterministic semantic output at the supported 1/4/16 worker counts.
The one-relation checkpoint above is not this full-library exact comparison and
must not be called a golden, Phase 1 exit, or SOP PASS. Phase 2's later all-five
repository gates remain unchanged.
Run applicable source/semantic/evidence gates and REQ-RUST-044.0; HLD-specific S-step
portions are deferred, not PASS. Constructs absent from itoa are explicitly
unexercised, never evidence of general proc-macro/async/reexport coverage.
Do not require unrelated Phase 2 shapes to label this bounded benchmark passed,
and do not call that M3–M8 all-five completion. The final 500 MB release target
remains deferred per R9, not silently satisfied by this small repository.

**Phase 2:** only after Phase 1, expand qualification in order:
serde → rust-analyzer → ripgrep → tokio → cargo-modules. Existing thresholds,
fixtures and all-five milestone/release rules remain mandatory. Preserve prior
serde drafts as historical partial evidence; they are not the Phase 1 reference.
At adoption, the itoa source pin is verified but its lock/config freeze, expected
graph, build qualification and production comparisons are all NOT STARTED.

## 2. Steps S1–S10

### S1 — fresh structural index

```bash
codebase-memory-mcp cli index_repository --repo-path "$REPO" --mode full
codebase-memory-mcp cli list_projects
codebase-memory-mcp cli index_status --project "$CMP"
codebase-memory-mcp cli get_graph_schema --project "$CMP"
```

Require succeeded generation at pinned checkout; record counts and generation. Re-index after source changes.
Never reuse an older source generation just because a project name exists.

### S2 — complete coverage inventory

```bash
codebase-memory-mcp cli check_index_coverage --project "$CMP" --scopes '["."]' \
  --scope-limit 20 --scope-offset 0 --diagnostics full --format json
codebase-memory-mcp cli query_graph --project "$CMP" --graph missed \
  --query "MATCH (f:File) RETURN f.file_path" --max-rows 200 --format json
```

Save each response as codebase-index-coverage-page-<ordinal>.json or codebase-missed-source-page-<ordinal>.json.
For check_index_coverage, keep one stable generation and repeat with --scope-offset from next_offset
until has_more=false. Require strictly increasing offsets and accumulated entries == scope_total.
scope_truncated describes an individual page window and is informational, not a gate.
For query_graph, pass next_cursor via --cursor with the same project/graph/query until has_more=false.
Restrict to File: the missed graph also contains folders, which are not missed files.
A partial/unusable/missed file is missing CROSS-CHECK evidence only; CM-GAP NEVER excuses an oracle-proven error.

### S3 — fixture gates and production map

Run the applicable SPEC §4 fixture/build suite first, including failure/empty cases in isolated fixtures.
Use cargo fmt --check, cargo clippy --all-targets -- -D warnings, cargo test and production cargo tree;
release-build the production binary separately from the oracle harness.
Audit production's dependency tree, not the harness's permitted ide/hir/salsa tree.

```bash
python3 tools/process-tree-access-auditor.py sample \
  --memory-sample-output-path "$OUT/process-tree-memory-samples.json" -- \
  parseltongue map-workspace-full-run --workspace-root-path-value "$REPO" \
  --workspace-store-file-path "$OUT/production-untraced-workspace-store.db" --worker-thread-count-value 4 \
  > "$OUT/production-stage-output-log.txt" 2> "$OUT/production-stage-error-log.txt"
strace --seccomp-bpf -f -s 65535 -e trace=execve,connect,clone,clone3,fork,vfork \
  -o "$OUT/production-process-access-audit.txt" \
  parseltongue map-workspace-full-run --workspace-root-path-value "$REPO" \
  --workspace-store-file-path "$OUT/production-audit-workspace-store.db" --worker-thread-count-value 4
strace --seccomp-bpf -f -s 65535 -e trace=openat -P "$REPO/target" \
  -o "$OUT/production-target-access-audit.txt" \
  parseltongue map-workspace-full-run --workspace-root-path-value "$REPO" \
  --workspace-store-file-path "$OUT/production-target-workspace-store.db" --worker-thread-count-value 4
parseltongue status-coverage-banner-show \
  --workspace-store-file-path "$OUT/production-untraced-workspace-store.db" --json \
  > "$OUT/hld-current-coverage-banner.json"
# HLD export applies at M8/releases, not M1/M2.
parseltongue hld-current-export-sorted \
  --workspace-store-file-path "$OUT/hld-current-workspace-store.db" \
  > "$OUT/hld-current-dependency-export.tsv"
```

The sampler/strace are external supervisors: filter the allowlist to the mapper and its descendants,
not supervisor startup. Resolve audited paths/symlinks; forbid build-script execution and target/ output reads.
The repository's tools/process-tree-access-auditor.py sample command is the executable equivalent of
the planned process-tree-memory-sampler wrapper. It records mapper PID, exit, elapsed seconds and sampled
sum of live mapper/descendant VmRSS in bytes at nominal 10 ms intervals; supervisor RSS is excluded.
tools/corpus-production-stage-measurements.py runs sampling, audit and JSON status for the pinned corpus.
Timing/RSS SHALL come only from the untraced run. Do not put the sampler beneath strace: its /proc
opens can dominate the trace and stop the mapper. Separate stores prevent measurement/audit interference.
Use strace --seccomp-bpf so excluded syscall classes do not incur ptrace stops.
N bounds CPU parsing workers and resident file inputs/results. The CLI loads at most N file inputs,
parses them in parallel without store calls, then writes their atomic file units in stable order.
The adapter retains Concurrent transactions and bounded retries for independent concurrent callers.
The sampler traverses children of every thread in the mapper tree, not every host process, and records
its own CPU seconds separately. A real child-residency fixture checks that child memory is included.
M1 first discovers the complete source closure into small request rows; no span/field/body outputs or
global syntax completion are published by this pass. After hashing persisted source/config tables,
prepare the final generation scope while it contains only metadata/control rows. Full extraction is
a second bounded parsing pass that replaces discovery units under that canonical generation key.
No AST crosses these passes. This extra CPU pass avoids a corpus-sized MVCC key rewrite: final
publication validates stage completion and commits only the manifest pointer. Prepare rejects late
calls containing syntax outputs. Profile load/parse/write seconds separately; final S3 timing includes
both passes and coverage aggregation, not just the faster one. Status streams large tables and uses
SQL counts rather than materializing complete body/field datasets.
The lightweight execve/connect audit retains clone/fork metadata solely to prove direct-child versus
Cargo-descendant ancestry; it traces no file opens. Filtered openat is a separate run. strace -P matches
the exact repo/target path, not all descendants: recursive target rejection is enforced by canonical-path
source guards and fixture tests, and the report SHALL distinguish this limit from full access tracing.
Reconstruct fork/clone/exec ancestry and check complete argv: direct children remain exactly the three
commands; Cargo descendants are restricted to rustc -vV and the approved stdin ___ discovery probe.
That probe has exactly one each of --crate-type bin/rlib/dylib/cdylib/staticlib/proc-macro,
approved --print=file-names/sysroot/split-debuginfo/crate-name/cfg flags and -Wwarnings.
The approved order is Cargo's observed order; no -o, --emit, --out-dir or other flags are accepted.
Pure print-only probes without selectors remain accepted; they cannot compile or write artifacts.
Reject unknown probe flags, output paths, real crate inputs, other executables and network activity.
A metadata Cargo .rustc_info.json cache is compiler discovery provenance, not a build-script output;
permit only that exact basename under target, never other target files.
A rustup shim re-exec of the identical approved argv is the same logical command, not a new permission.
Run the trace checker on the saved audit; any violation makes S3 FAIL, regardless of mapper exit status.
Check lock existence BEFORE metadata; compare full lock digest before/after; failed/mutated metadata
publishes no graph rows and preserves original bytes. A missing lock is explicit validation, not assumed Cargo behavior.
Production errors include stderr; require successful stage markers/publication for the selected milestone.
Do not require M8 HLD exports before M8: at earlier milestones export only available stage tables.
The S3 fixture suite exercises SPEC §4's negative/stale/collision/cancellation cases without corrupting corpus runs.

### S4 — paired semantic oracle exports

The current harness capability is `mapped_semantic_paths`: it exports mapped path-resolution sites only.
The complete S4 families below remain requirements; this capability label does not imply their
implementation or establish qualification. Freeze the published source generation before starting.
Run from the repository root with the existing `parseltongue` and `oracle-semantic-export-harness`
binaries; do not use `cargo run` per export. Use one persistent, invocation-unique `$OUT` and
capture stdout/stderr for every command. The mapped manifest and artifact cache must be identical
for both exports; do not change them or the source generation between PRIMARY and SECONDARY.
The manifest command's stdout is the JSON artifact itself; its stderr is retained separately.

```bash
set -e
SOURCE_STORE="$OUT/hld-current-workspace-store.db"
MANIFEST="$OUT/mapped-unit-manifest.json"
CACHE_OUT="$OUT/oracle-cache"
PRIMARY_OUT="$OUT/oracle-primary"
SECONDARY_OUT="$OUT/oracle-secondary"
CACHE="$CACHE_OUT/oracle-artifact-cache.json"
mkdir -p "$CACHE_OUT" "$PRIMARY_OUT" "$SECONDARY_OUT" \
  > "$OUT/oracle-stage-directories.stdout.log" 2> "$OUT/oracle-stage-directories.stderr.log"

parseltongue export-mapped-unit-manifest \
  --workspace-store-file-path "$SOURCE_STORE" \
  > "$MANIFEST" 2> "$OUT/mapped-unit-manifest.stderr.log"

CARGO_TARGET_DIR="$CACHE_OUT/cargo-target" oracle-semantic-export-harness \
  --workspace-source-root-path "$REPO" \
  --mapped-unit-manifest-path "$MANIFEST" \
  --oracle-artifact-cache-output "$CACHE" \
  > "$CACHE_OUT/stdout.log" 2> "$CACHE_OUT/stderr.log"

sha256sum "$MANIFEST" "$CACHE" \
  > "$OUT/oracle-paired-inputs.sha256" 2> "$OUT/oracle-paired-inputs.sha256.stderr.log"

CARGO_TARGET_DIR="$PRIMARY_OUT/cargo-target" oracle-semantic-export-harness \
  --workspace-source-root-path "$REPO" \
  --mapped-unit-manifest-path "$MANIFEST" \
  --oracle-path-export-output "$PRIMARY_OUT/oracle-path-export.json" \
  --oracle-configuration-kind PRIMARY \
  --oracle-macro-cache-manifest "$CACHE" \
  > "$PRIMARY_OUT/stdout.log" 2> "$PRIMARY_OUT/stderr.log"

CARGO_TARGET_DIR="$SECONDARY_OUT/cargo-target" oracle-semantic-export-harness \
  --workspace-source-root-path "$REPO" \
  --mapped-unit-manifest-path "$MANIFEST" \
  --oracle-path-export-output "$SECONDARY_OUT/oracle-path-export.json" \
  --oracle-configuration-kind SECONDARY \
  --oracle-macro-cache-manifest "$CACHE" \
  > "$SECONDARY_OUT/stdout.log" 2> "$SECONDARY_OUT/stderr.log"

sha256sum --check "$OUT/oracle-paired-inputs.sha256" \
  > "$OUT/oracle-paired-inputs-check.stdout.log" 2> "$OUT/oracle-paired-inputs-check.stderr.log"
```

PRIMARY: build scripts disabled. SECONDARY: enabled. Both: proc macros enabled, identical selected targets,
features/cfg/editions, test cfg OFF (--no-test equivalent), offline inputs and recorded artifacts.
The current mapped route fixes target context to the compiler default, enables no extra features,
keeps default features enabled, and disables test targets. Retain `$CACHE_OUT/cargo-target` and its
proc-macro dylibs through both exports; the cache command internally runs `cargo check --offline --locked`.
Each `oracle-path-export.json` is one pretty-printed JSON envelope, not JSONL. The comparator consumes
each file as a single JSON value.
Export path resolutions, invocation-keyed expansion items/signatures/body sites, impl relations,
method/call targets, signature refs and effective visibility/reachability.
The current `mapped_semantic_paths` route does not implement those full families or per-site expansion
provenance and proc-macro-dependence fields, and does not establish S4/S5 qualification.
The per-site PRIMARY/SECONDARY diff is the KL1 set; analysis-stats summaries cannot identify those sites.
Optional `rust-analyzer scip` occurrence export must use a matching pinned binary/configuration and writes
oracle-secondary-occurrence-index.scip; never substitute it for semantic rows or agreement gates.

### S5 — canonical per-site comparison

```bash
parseltongue oracle-edge-differ-run \
  --workspace-store-file-path "$SOURCE_STORE" \
  --primary-semantic-input-path "$PRIMARY_OUT/oracle-path-export.json" \
  --secondary-semantic-input-path "$SECONDARY_OUT/oracle-path-export.json" \
  --agreement-store-file-path "$OUT/oracle-edge-agreement-store.db" \
  > "$OUT/hld-current-oracle-difference.json" 2> "$OUT/oracle-differ-error-log.txt"
```

Write oracle-edge-agreement-table with each match/mismatch/unresolved/exclusion and causal evidence.
Use compilation-unit identity, normalized path/range/site kind and expansion invocation/provenance keys.
Target identity preserves Cargo source/target; duplicate keys canonicalize, different unit contexts stay separate.
PRIMARY is the sole agreement denominator; SECONDARY-only sites are KL1 observations, not production misses.

Per-repo gates (SPEC §2.1 gives full formulas):
- 005 exact path matches / eligible PRIMARY path sites ≥99.5%.
- 006 exact invocation item+body-site sets / eligible PRIMARY invocations ≥99%.
- 013 matching shape-expansion sets / matching-shape PRIMARY invocations =100%; every shape has a fixture.
- 018 relation intersection / relation union ≥99.5%.
- 007 correct extracted method targets / all extracted method sites ≥99%.
- 008 correct extracted method targets / eligible PRIMARY method sites ≥95% mandatory at M8,
  after the last selected inference sub-step, whether 7b or 7c.
- 019 correct selection/visibility/reachability / union of candidate occurrences =100%.

Report raw counts, exclusions and rates; no pooled-five-repo success. Empty denominator is N/A/BLOCKED for
corpus gates, not 100%; empty fixtures assert zero output. Missing oracle evidence BLOCKS comparison.
M2 alone excludes-and-reports proc-macro-dependent sites. M3 and later remove that temporary exemption.

Authoritative reasons and scoring, identical to SPEC §2.2:
cfg_inactive → build_script → proc_macro → external_source_missing → unknown_receiver → trait_solver_needed.
Exactly one earliest causal reason per site. Count all reasons separately.
Inactive/PRIMARY-absent KL1 are exclusions; PRIMARY-visible unresolved sites remain nonmatches, including
unknown_receiver and build_script. Missing oracle source blocks; missing production source alone is a nonmatch.
A wrong extracted target is never excused by an unresolved reason.

### S6 — structural cross-check

```bash
codebase-memory-mcp cli query_graph --project "$CMP" \
  --query "MATCH (f:File) RETURN f.file_path"
codebase-memory-mcp cli query_graph --project "$CMP" \
  --query "MATCH (n) WHERE n:Function OR n:Method OR n:Struct OR n:Enum OR n:Interface OR n:Type RETURN labels(n), count(*)"
codebase-memory-mcp cli query_graph --project "$CMP" \
  --query "MATCH (a)-[r:IMPLEMENTS]->(b) RETURN a.qualified_name, b.qualified_name"
```

Paginate row queries. Filter file paths to .rs and compare the same active closure/context, not the index's
HTML/docs/unrelated fixture count. Explain bytes-only files, inactive cfg, body items and expansion differences.
Impl graph differences prompt PRIMARY checks, not automatic pass/fail from codebase-memory.
Compare file/folder roll-ups exactly with published HLD aggregation (REQ 009), separately from index counts.

### S7 — edge triage without heuristic authority

```bash
codebase-memory-mcp cli query_graph --project "$CMP" --max-rows 99998 --format json \
  --query "MATCH (a)-[r:CALLS]->(b) WHERE r.strategy STARTS WITH 'lsp_' AND r.strategy <> 'lsp_short_name_unique' RETURN a.qualified_name, a.file_path, a.start_line, b.qualified_name, b.file_path, r.strategy, r.confidence"
```

Page with --cursor until has_more=false. STARTS WITH is supported in 0.12.1.
Index CALLS edges are not occurrence-complete: use them to locate candidates, then score the normalized
PRIMARY/production site rows. Never calculate method precision from deduplicated graph/file pairs.
Assign each compared semantic site one result: AGREE (correct), PT-EXTRA (incorrect extracted),
PT-BUG (PRIMARY-visible but not extracted), KNOWN-GAP (verified excluded cause).
Record CM-ONLY/CM-GAP as separate cross-check evidence flags, not semantic exemptions.
PT-EXTRA method rate = incorrect extracted method sites / all extracted method sites, must be ≤1%;
PT-BUG method sites affect mandatory coverage under REQ 008.
Record deduplicated HLD/file-pair discrepancies separately, without using them as precision denominators.
Sample misses in S8 still fail even when the aggregate precision threshold passes.

### S8 — sampled evidence review

Sample min(50, published HLD row count) WITHOUT replacement using explicit recorded seed 42 by default,
never derived from the run date. Rank canonical UTF-8 row keys by BLAKE3-256 of the seed (u64 little-endian),
key byte length (u64 little-endian) and key bytes; select the smallest hashes, breaking ties by row key.
Record algorithm/version, any overridden integer seed and all sampled row keys.
Zero rows yields zero samples and N/A, not 50/50; empty fixtures pass emptiness assertions, while nonempty
corpus gates still require SPEC's observations and cannot claim full success from no samples.
For each sample, inspect its stored source/evidence, optionally locating code with:

```bash
codebase-memory-mcp cli get_code_snippet --project "$CMP" --qualified-name "<qualified-name>" --source-mode full
```

The reviewer (human-architect-intent-author or ext-llm-coding-agent) validates actual code/evidence,
not just a snippet match. A missing index node uses stored source, not automatic failure.
Every sampled false edge/missing required evidence FAILS, independent of the ≥99% aggregate gate.

### S9 — node, reexport and collapse proof

Compare effective visibility and node_reachable_outside_crate with PRIMARY for all candidate occurrences.
Validate body-ancestry exclusion; trait/trait-impl/enum inherited visibility; restricted visibility;
private parents and public aliases/globs. One reexports edge per resolved exported binding;
source endpoints follow SPEC §2.4's module/root rule.
Validate collapsed paths against stored evidence rows: each hop resolved, endpoints valid, private
intermediates named, provenance/counts complete. No graph trace_path depth limit or CALLS-only traversal.
Run cycles/diamonds/genuine self-edge/unresolved-link fixtures; roll-ups count HLD rows, not paths.
Only extracted status enters HLD; an unresolved intermediate stops that path.

### S10 — determinism, memory and time

Run production twice at each worker count 1, 4, 16 under the same compatibility key; export stable sorted
semantic rows to hld-current-dependency-export-<workers>-<attempt>.tsv and compare with cmp.
Use fresh staging attempts; also run isolated concurrent-write and cancel/rerun fixtures.
For rust-analyzer, process-tree peak RSS ≤2 GiB (2 × 1024³ bytes), including allowed children, excluding
external samplers/indexers/harness. Record 10 ms summed-RSS samples, short-lived child accounting,
the larger observed process-tree peak and sampling limits. Parent-only /usr/bin/time -v is supplementary.
Report stage wall times and disk size; budget measurements are M, not assumed achievements.
Final 008 coverage is mandatory here at M8 even when M7 was skipped.

## 3. Milestone subsets and verdict

SPEC M0–M8 is authoritative. Phase 1 repeatedly applies the available subset to itoa
under §1.1; Phase 2 applies corpus comparisons to all five originally pinned repos.
Unavailable future stages are NOT silently marked passed.

- M0: S3 skeleton/build/dependency/identity fixtures; S4 harness build/provenance smoke fixtures only.
- M1: S1/S2, S3 input/store/generation/concurrency/cancellation fixtures; S10 input-row determinism and
  write-throughput measurements only. No HLD, final coverage or full-pipeline memory verdict.
- M2: S1/S2/S3, S4/S5 path/declarative/built-in/lang-item gates; proc-macro sites excluded-and-reported.
- M3: M2 subset plus S4/S5 shape gates and paths/macros with temporary exclusions removed.
- M4: M3 subset plus S4/S5 signatures/impl headers and S6 impl triage.
- M5: M4 subset plus S4/S5/S7 body 7a precision, primitive lookup and coverage report.
- M6: M5 subset for 7b; ≥95% on all five permits skipping M7.
- M7: M6 subset for 7c, production no-salsa gate remains mandatory.
- M8/releases: full S1–S10, all SPEC §4 contracts, final ≥95% coverage and ≤2 GiB rust-analyzer memory gate.

PASS: every applicable clause/test and per-repo threshold passes. Full HLD works only when all five
pass S1–S10 with the same production hash/toolchain.
FAIL: contract failure, sampled miss, forbidden production access/spawn, nondeterminism, missed mandatory
coverage/memory/agreement gate; PT-EXTRA is rate-gated, not automatically zero-tolerance.
BLOCKED: missing precondition, incomplete coverage pagination or unavailable oracle evidence; never PASS.
A codebase-memory omission cannot turn an oracle-proven error into KNOWN-GAP.
Do not patch production during a run: fix, then start a new applicable run.
Use itoa in Phase 1; all five in Phase 2. Never reuse an old executable's PASS.

## 4. Persistent report

Write hld-current-test-runs/<run-id>/hld-current-test-report.md, append-only per run.
Artifact ordinal/worker/attempt suffixes are data values, not extra semantic name words.
Record:
- Phase (1 or 2); Phase 1 reference revision/digest, complete expected/actual counts,
  all graph differences and unexercised constructs. Never aggregate itoa with Phase 2.
- Lead with the real-source checkpoint's expected/actual/missing semantic identities and source
  evidence, followed by the first blocking production stage; aggregate unit counts are secondary.
- Corpus/source/lock/tool/config hashes, PRIMARY/SECONDARY manifests, harness source/executable hashes.
- Generation keys and workspace/shared-library manifests, completed stages and exact applicable tests.
- Index generation/counts, every coverage page and caveats; production exec/read audit and stderr.
- Per-repo numerators/denominators/exclusions, authoritative reason counts, KL1 site evidence, rates.
- Method PT-EXTRA/PT-BUG counts separately from deduplicated HLD/file-pair discrepancies and CM flags.
- Sample seed/algorithm/keys and confirmed count; node/reexport/collapse/roll-up results.
- Process-tree RSS telemetry, stage times, disk size, worker/attempt export comparisons.
- Per-repo PASS/FAIL/BLOCKED, first failed clause, overall verdict and follow-up defects.

A planned harness/test/command is not an observed passing result. Preserve output evidence for every claimed gate.

### M1 parser requalification scope

When re-running S3 for a parser pin while M2 is incomplete, pass
--mapping-final-stage-kind syntax-spans. The default remains name-resolution.
The banner must show only the three M1 stages complete and leave later stages
not yet implemented; do not count that run as M2/S5 evidence.
Freeze one executable across all repositories and record its digest.
The process auditor must bind that exact configured path, not infer identity from
the filename. Reanalyse saved traces only with that explicit identity; never rewrite
traces or ignore unfinished events. A repeat audit may replace an incomplete capture
only if the failed capture and the repeat's evidence are both retained.

### S10 memory revision — deferred until correctness

The final whole-process-tree peak RSS target is ≤500 MB (500,000,000 bytes),
superseding the earlier 2 GiB acceptance gate. Enforce the revised target
after working DuckDB M2 and S5 results exist for all five repositories.
Before that milestone, prioritize correctness: do not optimize RAM, retain
the approximately 1.2 GiB DuckDB cap, and record actual process-tree peaks,
wall time and resolution rounds. Historical 2 GiB checks and current engine
limits are interim settings, not evidence of final memory acceptance.

### SOP revision — process compliance and DuckDB analysis (2026-11-12)

Appended after a compliance audit. Earlier text stays; where it conflicts, this revision wins.

R1 oracle pin. The answer key and the production parser are built from the
patched fork commit bf6b2099d6351c37039b01447a9bb64f9a54a586 plus
tools/parser-const-closure-statements.patch (SHA-256
89146042a1c1629ef67c730c0e180dec15264577f675de1c090dca2d87da6bfa).
References to checkout 2a362a41a in §0 are superseded.

R2 stale evidence. Every run binds one frozen release executable digest.
Evidence from an older digest does not carry forward: the M1 S1–S3 PASS
recorded before the DuckDB migration is historical only. The next applicable
milestone subset re-runs S1/S2/S3 on all five repositories.

R3 persistent report location.
```text
hld-current-test-runs/<run-id>/hld-current-test-report.md   committed, append-only
hld-current-test-runs/<run-id>/<repo-name>/                 small manifests, committed
~/hld-current-test-artifacts/<run-id>/<repo-name>/          large traces/exports, gitignored;
                                                            report records path + BLAKE3 digest
```
A gate without a report entry is not observed.

R4 Turso → DuckDB handoff gate (new, applies from M2). Before any analysis:
only a published, non-cancelled Turso generation is copied; per-table row
counts and content digests match on both sides; the DuckDB connection reports
extension autoload/autoinstall off, external access off, the configured
memory_limit and a run-local temp directory. Any mismatch is FAIL, not retry-and-pass.

R5 M2 S5 denominator. Oracle path sites are partitioned by a published rule:
module-scope paths are scored at M2; type-relative/associated paths are counted
and reported, scored from M4–M6. Every oracle row lands in exactly one class
with a reason; partition sums equal the raw export count per repository.

R6 deterministic analysis. Every consumed query has ORDER BY a total key.
S10 compares exports from DuckDB threads=1 and threads=N byte-for-byte.

R7 development-process gates (checked per run, recorded in the report):
- STUB → RED → GREEN → REFACTOR; each GREEN group is followed by a REFACTOR
  commit or an explicit "no refactor needed" checkpoint note.
- The SPEC verification matrix maps every REQ-RUST id to test ids; a check
  fails if a listed REQ or test id does not exist. Tests name their REQ.
- L2/L3 errors are typed (thiserror enums); free-text-only error variants are
  a refactor defect.
- cargo fmt --check, clippy --all-targets -D warnings, cargo test, release build.

R8 memory: see "S10 memory revision" — ≤500 MB whole-tree peak, enforced
after all-five DuckDB S5 exists; interim DuckDB cap ≈1.2 GiB.

### SOP revision — optional source-grounded HLD-current model audit (2026-11-12)

The independent, two-pass HLD-current audit in
`docs/phase-20261111/hld-current-model-audit/SOP.md` may supplement S8/S9.
It never reads production HLD rows in Pass A; it freezes a source/config-derived
expected artifact before Pass B. It does not change S8's deterministic sample,
PRIMARY denominators, thresholds, or per-repository verdicts. Model findings
are non-authoritative; source-confirmed false edges fail, unresolved disputes
block, and missing production HLD output is BLOCKED/NOT STARTED, not success.
This audit-only section does not change root AGENTS.md or production architecture.

### SOP revision R9 — interim memory-cap removal (2026-11-12)

User decision supersedes R4's configured-memory-cap language and R8's interim
cap. Until all-five DuckDB S5 results exist, do not set any interim DuckDB
memory_limit and do not impose a memory cap on correctness runs. Measure and
report the effective DuckDB current_setting('memory_limit'), duckdb_memory()
tags, phase elapsed time, and whole-process-tree peak RSS for every run. Keep
run-local spill directories, disable extension install/load and external
access, and retain file-backed databases. After all-five S5 exists, enforce
the unchanged ≤500,000,000-byte whole-process-tree peak RSS target.

### SOP revision R10 — direct-projection checkpoint (2026-11-13)

The typed DuckDB materializer's current implementation checkpoint covers only
SQL-based node selection and direct extracted-edge projection. It reports
DirectProjectionOnly; roll-ups, recursive private-chain collapse,
frozen-input marker validation, durable Turso publication, real-source
producers and source-to-HLD acceptance remain NOT STARTED. Extracted edges
through nonselected local items fail with the explicit unsupported-traversal
error. This checkpoint is implementation progress only, not HLD-complete or
SOP-subset PASS evidence.

### SOP revision R11 — HLD identity validation (2026-11-13)

The typed input validator accepts only the primitive Builtin type vocabulary
seeded by namespace records, rejects local-unit ExternalDefinition aliases and
cross-unit evidence sites, and fails closed for unsupported owner/item semantics
or inconsistent body ancestry. Same-unit macro evidence may use a different
source file. Identical duplicates remain legal; conflicting unit, node and
evidence duplicates fail. Inferred, Proposed and Unresolved evidence remain
non-extracted. This increment does not implement recursive traversal,
roll-ups or publication.

### SOP revision R12 — recursive DuckDB paths (2026-11-13)

The current bounded materializer replaces direct-only traversal rejection
with a recursive DuckDB CTE over extracted edges. It carries ordered full
endpoint/evidence identities, follows only nonselected LocalItems, stops at
the first selected or canonical external/builtin terminal, and uses the
terminal edge relation. Per-path guards prohibit repeated private endpoints
while allowing a finite return to the selected source; private cycles alone
emit no edge. Paths are distinct by ordered proof identity, and SQL counts the
distinct evidence union per generation/source/target/relation/via row.
The union is computed in SQL by UNNESTing ordered evidence lists; Rust only
decodes the records and sorts the typed result.
Output completeness is GraphPathsOnly; file/folder roll-ups remain None.
Frozen-stage checks, Turso publication, real-source producers, CLI completion
status and source-to-HLD acceptance remain NOT STARTED. This is not an HLD
complete or SOP-subset PASS claim.

### SOP revision R13 — DuckDB file/folder roll-ups (2026-11-13)

The current explicit-input DuckDB materializer now materializes/reuses the
deduplicated HLD edge relation in SQL and groups that relation for file and
folder roll-ups. Paths and unnested evidence are not roll-up inputs; each
full HLD key contributes once, with relation and Direct/Collapsed via kept
as dimensions.

File buckets preserve full compilation-unit identity and normalized source-
relative path. CrateRoot uses its explicit bucket; an ExternalDefinition
without verified file metadata and every Builtin use full canonical target
identity. Folder buckets use immediate parent paths, including `.` for
root-level source files, and retain unit identity. CrateRoot and canonical
external buckets remain distinct. HldFolderEndpointBucket is a separate L1
type from file buckets.

The materializer reports CompleteGraph with Some(file roll-ups) and
Some(folder roll-ups), including Some(empty) for empty typed input. This
means only that all supplied typed facts were materialized. Frozen-stage
checks, Turso publication, real-source producers, CLI status and source/
corpus qualification remain NOT STARTED. The focused matrix covers
test_hld_rollup_edge_counts, test_hld_rollup_bucket_context and
test_hld_empty_input_rollups; determinism includes roll-ups.

### SOP revision R14 — durable Turso HLD publication (2026-11-13)

Typed CompleteGraph output from the explicit-input DuckDB materializer is
published through the synchronous L2 HldGraphPublicationPort and its Turso L3
implementation. This adds neither source producers nor CLI integration,
corpus qualification, or an end-to-end status claim.

Publication writes the five semantic HLD tables in one generic stage unit at
hld/current, writes unit/global completion markers, seals a snapshot, and
switches workspace-generation-publication-manifest in the same transaction.
The source scope must resolve from frozen-stage-publication-records; required
legacy and semantic completion markers must be visible at that exact frozen
revision. The destination must match the resolved base attempt and generation.
For each required input owner, publication also rejects any current-scope
completion/work-unit marker absent from that frozen revision with typed
StaleInputSnapshot. This closure check is HLD-specific and does not alter
generic snapshot semantics. Generic writes by the HLD owner are rejected once
any HLD marker is frozen, including global completion and non-current units.
All unsealed HLD-owned rows and markers in the destination scope are cleared
before writing the new unit, preventing stale units from entering the view.
Configured workspace_required_stage_names remain enforced by the existing
generic publication gate. The global HLD completion-marker payload records
the exact input snapshot alias, base attempt and revision as persistence
metadata, separate from semantic row payloads.

Canonical JSON row identities preserve full typed identities and ordered
proof identities, including optional signatures. Attempt IDs and Debug text
do not enter semantic keys/payloads. Sealed HLD units cannot be overwritten;
failed/cancelled transactions preserve prior published views. The registered
semantic input tables are only stage-owned here; their decoders/producers are
not implemented. Required tests:
test_hld_missing_input_markers, test_hld_generation_input_isolation,
test_hld_cancel_preserves_publication, test_hld_failed_write_rollback,
test_hld_complete_empty_snapshot and test_hld_publication_round_trip.
These are component fixtures, not itoa qualification.

### SOP revision R15 — source-first Phase 1 acceptance (2026-11-13)

Phase 1 development is acceptance-test-driven on the pinned itoa source. The
first required checkpoint is independently verifying
`<Buffer as Default>::default -> Buffer::new`, then exercising actual pinned
source through the production pipeline and asserting full semantic identities
and source evidence. Expected rows remain in test assertions only; no
pre-resolved fixture inputs or production hardcoding are allowed. Component
checks remain useful but are not end-to-end acceptance.

**Next task — NOT STARTED:** implement the real-source integration regression
for that exact relationship; run it to identify the first production failure;
then fix only the required general-purpose producer path until the relation is
emitted with evidence. Do not start by completing unrelated downstream
infrastructure, and do not claim this relationship is currently emitted by
production. Confirm test discovery, nonzero selection, and intended behavior
before recording RED/GREEN.

An initial comparison may use explicitly partial, independently source-verified
expectations for diagnosis, but never label a partial reference as golden. The
one-relation checkpoint is not a whole-itoa result or SOP PASS. The later
complete-library exact comparison, all applicable quality gates, five-repo
qualification, and all-five milestone/release requirements remain intact.
Existing L1/L2/L3 boundaries, Turso durable atomic publication, DuckDB
relational analysis, provenance, cancellation, and stage contracts are not
weakened by this development-order rule.

### SOP revision R16 — full graph first, levels derived (2026-11-13)

SPEC REQ-RUST-032.0, 019.2 and 030.1 apply. Nothing in this revision has been
run; no producer of the full graph exists.

S9 now has two parts, in this order:

1. Full graph. Compare item-effective-visibility-records and
   edge-evidence-record-table with PRIMARY and the source-reviewed reference:
   every item outside fn bodies is a node, private items included; one
   file-public-area-node per source file; one node per cfg-active use outside
   fn bodies; every extracted edge present uncollapsed, private -> private
   included; imports and reads edges present; an edge resolved through an
   import targets the item and keeps the use node as evidence; no edge from a
   function to an import it does not use.
2. Interface level as a filter. Every hld-current-interface-nodes row, every
   hld-current-dependency-edges row and every hld-edge-evidence-paths hop
   exists in the full graph. File-public-area-nodes and use nodes are selected,
   including in a file whose module is private. Private items are unselected
   and named as intermediates. No collapsed path has a `contains` or `imports`
   hop. The existing S9 visibility, reexport, collapse,
   cycle, diamond and roll-up checks then run unchanged.

For pinned itoa (§1.1) the preserved candidate row set lists full-graph candidates,
private ones included, beside a separately derived eligible subset and five
interface/evidence/roll-up outputs. In §1.1 step 5, the full-graph files are
`LLM_PARSED_DRAFT` candidate inventories authored by Devin from pinned source, not whole-source
coverage (operators, body types, literals and repeated same-line sites remain omitted); no
independent compiler/oracle verification has frozen them. The five expected outputs are separate
and derived only from eligible rows. The draft remains symbolic and noncanonical. The first
checkpoint is unchanged:
`<Buffer as Default>::default -> Buffer::new`, asserted first as a full-graph
edge from real source. The SPEC's "Not decided in this revision" points are
reported as open in a run report, not scored.

Order (user-approved 2026-11-13): before the `default -> Buffer::new` checkpoint, run a
skeleton slice on the same real source: node rows for every item, `use` and file area, plus
`imports` and `contains` edges, assembled, built and published. It proves storage, assembly
and publication on real source with the least new code. It is not the checkpoint and not a
PASS. Build order and contracts: SPEC REQ-RUST-033.0–037.0 and ARCH section 6.

Interim scope (2026-11-13, PRD AD-1 option B, SPEC REQ-RUST-038.0): the unchanged pipeline on
itoa plus sysroot did not finish name resolution in 2,400 s. Until that path works, Phase 1
development runs use `--sysroot-source-scope-kind excluded`. Such a run is local-only: edges
into `core` are reported unresolved with a reason, never invented; step 3 above (external
definition evidence and sysroot identity) is then BLOCKED, not waived; and the run cannot be a
Phase 1 PASS.

### SOP revision R15 — frozen full-graph storage and scoped publication

REQ-RUST-041.0 owns the L2 typed graph-input store and Turso L3 persistence/assembly seam. It
reads canonical unit identity from the empty-path root rows in
`syntax-namespace-input-scopes`, mapped through `scope_cargo_unit_key` to the raw crate graph
unit. Structural config and graph-owner markers must carry the exact mapping and coverage scope.
The input snapshot is analysis-only; only the final HLD transaction updates the serving manifest.
This storage checkpoint adds no source SQL producer or CLI wiring and does not qualify pinned itoa.
