# SOP — phase-20261111 HLD validation

Name: hld-current-test-procedure. Status: planned executable procedure, not a record of passing runs.
Inputs: PRD-phase-20261111-compiling-hld.md and SPEC-phase-20261111-executable-requirements.md.
SPEC owns requirement/test names and scoring; PRD owns scope; this SOP applies those contracts.
Full S1–S10 runs apply at M8/releases. Intermediate milestone subsets are listed in §3.
Every applicable corpus gate runs on all five repos, never a tune/held-out split.

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

P1: pin clean corpus commits: rust-analyzer, ripgrep, tokio, serde, cargo-modules (SPEC §8).
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

Run in isolated oracle output/build locations, never allowing SECONDARY to alter PRIMARY's inputs.
Write normalized rows in oracle-primary-semantic-rows.jsonl and oracle-secondary-semantic-rows.jsonl.

```bash
oracle-semantic-export-harness export-workspace-semantic-rows \
  --workspace-root-path-value "$REPO" --oracle-export-mode-value PRIMARY \
  --semantic-row-output-path "$OUT/oracle-primary-semantic-rows.jsonl" \
  > "$OUT/oracle-primary-output-log.txt" 2> "$OUT/oracle-primary-error-log.txt"
oracle-semantic-export-harness export-workspace-semantic-rows \
  --workspace-root-path-value "$REPO" --oracle-export-mode-value SECONDARY \
  --semantic-row-output-path "$OUT/oracle-secondary-semantic-rows.jsonl" \
  > "$OUT/oracle-secondary-output-log.txt" 2> "$OUT/oracle-secondary-error-log.txt"
```

PRIMARY: build scripts disabled. SECONDARY: enabled. Both: proc macros enabled, identical selected targets,
features/cfg/editions, test cfg OFF (--no-test equivalent), offline inputs and recorded artifacts.
Export path resolutions, invocation-keyed expansion items/signatures/body sites, impl relations,
method/call targets, signature refs and effective visibility/reachability.
The per-site PRIMARY/SECONDARY diff is the KL1 set; analysis-stats summaries cannot identify those sites.
Optional `rust-analyzer scip` occurrence export must use a matching pinned binary/configuration and writes
oracle-secondary-occurrence-index.scip; never substitute it for semantic rows or agreement gates.

### S5 — canonical per-site comparison

```bash
parseltongue oracle-edge-differ-run \
  --workspace-store-file-path "$OUT/hld-current-workspace-store.db" \
  --primary-semantic-input-path "$OUT/oracle-primary-semantic-rows.jsonl" \
  --secondary-semantic-input-path "$OUT/oracle-secondary-semantic-rows.jsonl" \
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

SPEC M0–M8 is authoritative. All applicable corpus comparisons use all five pinned repos.
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
Do not patch production during a run: fix, then start a new applicable run on all five.

## 4. Persistent report

Write hld-current-test-runs/<run-id>/hld-current-test-report.md, append-only per run.
Artifact ordinal/worker/attempt suffixes are data values, not extra semantic name words.
Record:
- Corpus/source/lock/tool/config hashes, PRIMARY/SECONDARY manifests, harness source/executable hashes.
- Generation keys and workspace/shared-library manifests, completed stages and exact applicable tests.
- Index generation/counts, every coverage page and caveats; production exec/read audit and stderr.
- Per-repo numerators/denominators/exclusions, authoritative reason counts, KL1 site evidence, rates.
- Method PT-EXTRA/PT-BUG counts separately from deduplicated HLD/file-pair discrepancies and CM flags.
- Sample seed/algorithm/keys and confirmed count; node/reexport/collapse/roll-up results.
- Process-tree RSS telemetry, stage times, disk size, worker/attempt export comparisons.
- Per-repo PASS/FAIL/BLOCKED, first failed clause, overall verdict and follow-up defects.

A planned harness/test/command is not an observed passing result. Preserve output evidence for every claimed gate.
