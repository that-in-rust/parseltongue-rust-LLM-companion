# HLD-current SOP runner work journal

This scoped journal is the only phase documentation edited for the runner. It is not the shared TDD journal,
status board, SOP, SPEC, or a record of a corpus qualification pass.

## Authority, scope, and correction

- Read `AGENTS.md` at the repository root, the full `SOP-phase-20261111-hld-current-e2e-test.md` (including
  R1–R8), and the relevant SPEC scope, S1–S5/process-boundary requirements, verification crosswalk, DuckDB
  handoff, and M2 path-partition revisions.
- R9 from the task owner overrides prior memory language: the runner imposes no memory cap and no memory
  acceptance gate before all-five-repository S5. It records actual effective DuckDB `memory_limit` evidence
  and sampled whole-process-tree RSS only. The final ≤500,000,000-byte target remains deferred until after
  all five S5 results. Do not optimize or fail runs on memory before that point.
- Parent owns the future source-grounded S8/S9 audit on an isolated branch. This runner does not implement
  S8/S9 and does not change their existing deterministic sample or proof behavior. Missing later-stage
  exports remain NOT STARTED/BLOCKED; fixtures never establish corpus PASS.
- Scope is Python under `tools/` plus this journal. No Rust production/crate changes, production rebuild,
  full corpus mapping, or edits to shared root/phase SOP, SPEC, status board, TDD journal, or pin manifests.

## Codebase-memory correction and baseline

Read the installed skill directly at:
`/opt/.devin/plugins_cache/cache/account-upload_account-devin_playbooks-a7877c9b/24f64542e2d0bcba535f15332573cd75b45798be4e7fe1e58a9d9b9ab7f09fdd/skills/codebase-memory/SKILL.md`.
The skill directs installing `codebase-memory-mcp`, indexing with `index_repository`, and checking project,
status, schema, and scoped coverage. Its current CLI help documents `--repo-path` (the skill's raw JSON args
are deprecated). The initial local version check returned `MISSING: ~/.local/bin/codebase-memory-mcp`;
installed with checksum-verifying upstream installer and `--skip-config` to avoid agent config changes.
Installer and version output included `Checksum verified.` and `codebase-memory-mcp 0.12.1`.

Full-reindex command:

```text
codebase-memory-mcp cli index_repository --repo-path /home/ubuntu/repos/parseltongue-rust-LLM-companion --mode full
```

Result: `status=indexed`, project `home-ubuntu-repos-parseltongue-rust-LLM-companion`, 22,870 nodes,
42,782 edges, index mode full, `indexed_at=2026-11-12T16:49:31Z`, 6 parse-partial files, 0 unusable
files, and 17 purposely-not-indexed items (17 ignored files; `.claude`, `.git`, `.vscode` excluded).
The indexer result said `not_indexed_files` were excluded by ignore rules/suffixes and are not an error.
This is a fresh observation; no 13:51Z / 664-missed baseline was used.

Exact read/query commands and results:

```text
codebase-memory-mcp --version
  codebase-memory-mcp 0.12.1
codebase-memory-mcp cli list_projects
  projects: 1; home-ubuntu-repos-parseltongue-rust-LLM-companion
  root_path=/home/ubuntu/repos/parseltongue-rust-LLM-companion
  branch=devin/1794501865-sop-qualification-runner; total=1; has_more=false
codebase-memory-mcp cli index_status --project home-ubuntu-repos-parseltongue-rust-LLM-companion --diagnostics full --format json
  status=ready; nodes=22870; edges=42782; indexed_at=2026-11-12T16:49:31Z
  parse_partial=6; parse_unusable=0; not_indexed.files_count=17; skipped=0
codebase-memory-mcp cli get_graph_schema --project home-ubuntu-repos-parseltongue-rust-LLM-companion --format json --diagnostics full --limit 100 --offset 0
  exit 0; JSON schema returned node_labels and edge_labels (schema response inspected).
codebase-memory-mcp cli check_index_coverage --project home-ubuntu-repos-parseltongue-rust-LLM-companion --scopes '["tools"]' --scope-limit 100 --scope-offset 0 --diagnostics full --format json
  generation=2026-11-12T16:49:31Z; recording_status=complete; generation_matches=true;
  tools scope total=0; status=no_recorded_issue; signal=best_effort; has_more=false.
codebase-memory-mcp cli check_index_coverage --project home-ubuntu-repos-parseltongue-rust-LLM-companion --paths '["tools/corpus-index-coverage-inventory.py","tools/corpus-production-stage-measurements.py","tools/process-tree-access-auditor.py","tools/process-tree-contract-tests.py"]' --path-limit 100 --path-offset 0 --diagnostics full --format json
  generation=2026-11-12T16:49:31Z; all 4 paths returned, each status=no_recorded_issue,
  freshness=metadata_match, coverage=[]; path_total=4; path_has_more=false; signal=best_effort.
```

The coverage API explicitly warns that no recorded issue does not prove completeness. At this baseline,
the new runner and its tests did not exist yet; refresh/re-query them after implementation. No `b3sum` or
Python `blake3` provider is present; real-run preflight must report BLOCKED rather than invent a BLAKE3 digest.

## Local runner contracts and test traceability

These local identifiers describe Python-runner contracts only; they neither amend SPEC nor claim Rust
requirement coverage. Each entry maps to the existing SOP and SPEC clauses it operationalizes.

### REQ-SOP-RUNNER-001

- Contract: exclusive persistent runs, truthful initial states, planning, and harness-only identity.
- SOP: §3 milestone verdicts and §4 persistent report.
- SPEC: REQ-RUST-020.1 stage-generation isolation; REQ-RUST-022.1 interrupted-stage status.
- Tests: `test_report_initializes_repo_states`, `test_existing_run_refuses_overwrite`,
  `test_planning_creates_truthful_report`, `test_fake_execution_is_harness`,
  `test_persisted_harness_stays_harness`.

### REQ-SOP-RUNNER-002

- Contract: frozen executable, manifest, lock, source identity, pin agreement, and process boundaries.
- SOP: §1 P1–P5; R1 oracle pin and R2 stale-evidence binding.
- SPEC: REQ-RUST-015.1 target/build-script boundary; REQ-RUST-020.1 generation identity.
- Tests: `test_missing_executable_blocks_preflight`, `test_binary_mutation_fails_boundary`,
  `test_lock_mutation_fails_boundary`, `test_source_mutation_fails_boundary`,
  `test_pin_mismatch_blocks_parser`.

### REQ-SOP-RUNNER-003

- Contract: fresh S1/S2 indexing, complete stable pagination, totals, and missed-File evidence.
- SOP: S1, S2, and §4 index-generation/page reporting.
- SPEC: REQ-RUST-001.2 offline graph/process boundary; REQ-RUST-003.1 source coverage;
  REQ-RUST-020.1 generation identity.
- Tests: `test_stale_generation_blocks_scope`, `test_scope_cursor_cycle_blocks`,
  `test_missed_cursor_cycle_blocks`, `test_malformed_coverage_page_blocks`,
  `test_complete_coverage_pages_validate`, `test_empty_missed_pages_validate`,
  `test_scope_total_mismatch_blocks`, `test_boolean_scope_total_blocks`,
  `test_boolean_missed_total_blocks`, `test_serde_executes_before_ripgrep`.

### REQ-SOP-RUNNER-004

- Contract: reused S3 measurement/audit helpers, canonical executable, complete trace/status, markers.
- SOP: §0 production-child boundary and S3.
- SPEC: REQ-RUST-010.2 production dependency boundary; REQ-RUST-015.1 target/build-script boundary;
  REQ-RUST-020.1 stage isolation.
- Tests: `test_forbidden_network_trace_fails`, `test_incomplete_trace_blocks_measurement`,
  `test_clean_trace_validates_root`, `test_unfinished_process_trace_blocks`,
  `test_measurement_uses_frozen_path`, `test_measurement_guard_wraps_commands`,
  `test_repository_s3_directory_isolated`.

### REQ-SOP-RUNNER-005

- Contract: per-repository/per-step outcomes, exceptions, serde fail-fast, and separate future gates.
- SOP: §3 verdict semantics and §4 persistent report; R2 stale-evidence rule.
- SPEC: REQ-RUST-020.1 generation isolation; REQ-RUST-022.1 cancellation semantics.
- Tests: `test_serde_failure_stops_corpus`, `test_error_persists_repo_states`,
  `test_preflight_failure_persists_report`, `test_artifact_exception_persists_report`,
  `test_m1_s10_stays_unstarted`, `test_m2_exports_remain_blocked`.

### REQ-SOP-RUNNER-006

- Contract: structured R4/R5 extension validators, generation/table/settings/temp checks and row partitions.
- SOP: S4/S5; R4 Turso-to-DuckDB handoff; R5 M2 denominator.
- SPEC: REQ-RUST-024.1 published-generation copy equality; REQ-RUST-024.4 effective settings;
  REQ-RUST-024.8 projected handoff; §2.1 path partition rule.
- Tests: `test_missing_duckdb_settings_blocks`, `test_handoff_digest_mismatch_fails`,
  `test_negative_handoff_count_blocks`, `test_valid_handoff_records_settings`,
  `test_partition_rejects_duplicate_rows`, `test_partition_identity_mismatch_blocks`,
  `test_malformed_row_identity_blocks`, `test_valid_partition_counts_rows`,
  `test_empty_partition_checks_denominator`.

### REQ-SOP-RUNNER-007

- Contract: pre-S5 memory reporting is measurement-only; no 500 MB or interim cap acceptance gate.
- SOP: R8 and S10 memory revision.
- SPEC: REQ-RUST-024.5 final target, which supersedes REQ-RUST-024.4 and REQ-RUST-011.1 for this
  pre-correctness milestone; user R9 defers acceptance until all-five S5.
- Tests: `test_large_rss_never_fails`, `test_memory_target_remains_deferred`.

### REQ-SOP-RUNNER-008

- Contract: external large-artifact BLAKE3 and harness-only fixture identity.
- SOP: R3 artifact location/digest and §4 evidence retention.
- SPEC: REQ-RUST-020.1 run-generation identity.
- Tests: `test_real_run_requires_blake3`, `test_fake_execution_is_harness`,
  `test_persisted_harness_stays_harness`.

### REQ-SOP-RUNNER-009

- Contract: S3 component results never imply full S3; missing fixtures, build/dependency proof, P2, or
  release provenance keeps qualification BLOCKED. Record rustc/cargo and configuration versions or
  explicit unavailability.
- SOP: §1 P1–P5; S3 fixture/build/dependency and production-map gates; §3 verdict.
- SPEC: REQ-RUST-010.2, REQ-RUST-015.1, REQ-RUST-020.1.
- Tests: `test_s3_component_never_qualifies`, `test_component_completion_continues_corpus`,
  `test_toolchain_versions_are_recorded`.

### REQ-SOP-RUNNER-010

- Contract: validate each live pagination page before another subprocess, reject all cursor cycles, preserve
  received response bytes and native generations, and place BLAKE3-digested raw responses externally.
- SOP: S1/S2 and R3 artifact boundary.
- SPEC: REQ-RUST-001.2, REQ-RUST-020.1.
- Tests: `test_missed_cycle_stops_collection`, `test_stale_missed_generation_stops`,
  `test_boolean_scope_stops_collection`, `test_empty_pages_cannot_loop`,
  `test_raw_pages_stay_external`, `test_invalid_digest_blocks_manifest`,
  `test_repository_s3_directory_isolated`.

### REQ-SOP-RUNNER-011

- Contract: post-freeze mutation is FAIL, terminal preflight/finalization status affects the aggregate,
  and evidence digesting is best-effort without re-demanding invalid frozen inputs.
- SOP: R2, R3, §3 FAIL/BLOCKED semantics, §4 report fields.
- SPEC: REQ-RUST-020.1 generation identity; REQ-RUST-022.1 interrupted run state.
- Tests: `test_dirty_snapshot_fails_boundary`, `test_preflight_failure_affects_overall`,
  `test_artifact_digest_affects_overall`, `test_run_writes_artifact_manifest`.

### REQ-SOP-RUNNER-012

- Contract: R4/R5 validators are extension components only; their self-asserted counts/digests cannot
  establish corpus S4/S5 without independent authoritative table/JSONL validation and complete classes.
- SOP: R4, R5, S4, and S5.
- SPEC: REQ-RUST-024.1, REQ-RUST-024.8, §2.1 path partition rule.
- Tests: `test_m2_component_stays_blocked`, `test_m2_validator_fail_aggregates`.

### REQ-SOP-RUNNER-013

- Contract: hash every tracked byte and relevant ignored Rust/Cargo input; bind symlink targets safely or
  BLOCK when dependency closure cannot be frozen.
- SOP: §1 P1/P2/P3 and S3 source/build-script boundary.
- SPEC: REQ-RUST-014.1 compilation context; REQ-RUST-015.1 generated/build-script inputs;
  REQ-RUST-020.1 generation identity.
- Tests: `test_vendor_build_sources_block`, `test_tracked_binary_changes_snapshot`,
  `test_symlink_target_change_fails`, `test_directory_symlink_blocks_snapshot`.

### REQ-SOP-RUNNER-014

- Contract: exact `.rustc_info.json` basename exemption only; unfinished target traces block, and the
  exact-path-only filter remains explicitly non-recursive.
- SOP: S3 filtered target audit and explicit recursive-audit limitation.
- SPEC: REQ-RUST-015.1 target-output read prohibition.
- Tests: `test_rustc_suffix_path_fails`, `test_exact_rustc_basename_allowed`,
  `test_unfinished_target_trace_blocks`, `test_exit_only_target_passes`,
  `test_exit_only_requires_wrapper`, `test_empty_target_trace_blocks`,
  `test_truncated_target_trace_blocks`, `test_orphan_resumed_target_blocks`.

Support scripts remain the source of S1/S2 and S3 mechanics: extend
`tools/corpus-index-coverage-inventory.py` with optional indexer/project/configuration and strict pagination
validation; extend `tools/corpus-production-stage-measurements.py` for explicit run-local output and guarded
before/after subprocess callbacks; preserve `tools/process-tree-access-auditor.py` trace and nominal-10ms
RSS sampling behavior. No S8/S9 code or behavior is in scope.

## STUB phase — API and plan

Added the runner API stub only; no implementation exists yet. Planned CLI inputs: explicit milestone M1/M2,
unique run ID, optional frozen mapper path (required for execution; plan mode records NOT STARTED without it),
manifest/report/artifact roots, optional indexer path and corpus-root overrides, plan/execute mode, and
structured M2 evidence paths. Plan mode writes a persistent NOT STARTED/BLOCKED report and never invokes
mapper/oracle. Execute mode freezes pins, source snapshot, lock, manifests, runner/helper identities and
canonical executable before stage work; every child command is guarded before/after. New runs are
exclusive, fail-fast in serde-first order, and all outputs/report transitions survive failures/interruption.

R4/R5 evidence is data-only JSON; no arbitrary command string is accepted. Any unresolved precondition is
BLOCKED, an observed contract violation is FAIL, and full milestone PASS is impossible while M1 S10 or M2
S4/S5 is missing. Real execution and full corpus mapping are not being run in this implementation task.

### STUB verification

Command:

```text
python3 - <<'PY'
from pathlib import Path
path = Path('tools/hld-current-qualification-runner.py')
compile(path.read_text(), str(path), 'exec')
print(f'Syntax OK: {path}')
PY
```

Output after the full typed API stub was added: `Syntax OK: tools/hld-current-qualification-runner.py`
(exit 0). The API compile check preceded test addition and implementation.

The tests were written before implementation. Deliberate STUB-phase RED command:

```text
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py' -v
```

Actual output: 28 tests errored against the unimplemented API stub; command exit status 1. Representative
exact failure and summary lines:

```text
ERROR: test_binary_mutation_fails_boundary (test_sop_qualification_runner.SopQualificationRunnerContractTests.test_binary_mutation_fails_boundary)
  File "/home/ubuntu/repos/parseltongue-rust-LLM-companion/tools/test_sop_qualification_runner.py", line 116, in test_binary_mutation_fails_boundary
    frozen = runner.validate_runner_executable(executable)
  File "/home/ubuntu/repos/parseltongue-rust-LLM-companion/tools/hld-current-qualification-runner.py", line 90, in validate_runner_executable
    raise NotImplementedError
NotImplementedError
Ran 28 tests in 0.011s
FAILED (errors=28)
```

The captured output showed all tests reaching `NotImplementedError`, except the BLAKE3 finder test, which
also exposed that its planned provider-finder symbol was not yet in the stub. No implementation existed at
this RED checkpoint.

## Initial GREEN phase — tools-only implementation before parent review

Implemented the Python runner in `tools/hld-current-qualification-runner.py`, with S1/S2 indexing and
pagination in `tools/corpus-index-coverage-inventory.py`, S3 measurement/audit orchestration in
`tools/corpus-production-stage-measurements.py`, REQ-tagged tests in `tools/test_sop_qualification_runner.py`,
and this scoped journal. The runner writes an exclusive persistent report/manifest before preflight, keeps all
five repositories NOT STARTED until work is attempted, persists per-step commands/outcomes and first clauses,
and fails fast in serde-first order. It pins canonical mapper bytes, manifests, runner/support sources, source
trees, lock files, and process-boundary identities. It never uses historic corpus check codes as qualification.

S1 is fresh full indexing followed by list/status/schema checks; S2 requires stable generation/totals,
strictly advancing pagination, complete row counts, final status stability, and retained response-page digests.
S3 reuses the process-tree auditor and separate traced/untraced measurements, requires complete traces, exit
statuses, and publication markers, and reports exact-target filter non-recursion plus nominal 10-ms RSS
sampling limitations. R9 remains measurement-only before all-five S5. The M2 R4/R5 validators accept only
structured data bound to run, repository, binary, input, artifact path and size/digests; they do not mark S4/S5
or full milestone qualification PASS. M1 S10 and M2 S4/S5 remain BLOCKED/not started at the integration
boundary. A fake executable may be run only with `--harness-only`, which records HARNESS ONLY, never PASS.

Example planning command (does not need mapper/oracle and does not execute the corpus):

```text
python3 tools/hld-current-qualification-runner.py --run-id review-plan-001 --milestone M1 --mode plan
```

Real execution requires `--mapper-binary PATH`, optional `--corpus-manifest PATH`, `--indexer-path PATH`, and
repeatable `--project-root NAME=PATH`; defaults are the pinned corpus manifest, repository
`hld-current-test-runs/`, and `~/hld-current-test-artifacts/`. A real run is blocked when no external BLAKE3
provider exists. No real execute-mode corpus run was attempted.

### Initial GREEN verification (superseded by parent-review checkpoint)

Static syntax and local REQ/name traceability command:

```text
python3 -B -c 'import ast; from pathlib import Path; paths = [Path("tools/hld-current-qualification-runner.py"), Path("tools/corpus-index-coverage-inventory.py"), Path("tools/corpus-production-stage-measurements.py"), Path("tools/test_sop_qualification_runner.py")]; [compile(path.read_text(), str(path), "exec") for path in paths]; tree = ast.parse(paths[-1].read_text()); tests = [node for node in ast.walk(tree) if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name.startswith("test_")]; assert all(len(node.name[5:].split("_")) == 4 for node in tests); assert all((ast.get_docstring(node) or "").startswith("REQ-SOP-RUNNER-") for node in tests); print("Syntax and REQ traceability OK:", len(tests), "tests; exactly-four-word names and tagged docstrings")'
```

Output: `Syntax and REQ traceability OK: 43 tests; four-word names and tagged docstrings` (exit 0).

Runner unittest command:

```text
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py' -v
```

Final output:

```text
Ran 43 tests in 0.068s
OK
```

Existing process-tree contract command:

```text
python3 tools/process-tree-contract-tests.py
```

Output:

```text
......
Ran 6 tests in 0.334s
OK
```

The first implementation unittest attempt exposed that the binary-mutation fixture lacked executable
permissions; the fixture now marks the file executable before testing byte-mutation detection. Post-GREEN
edge supplements also cover malformed M2 row/count evidence and boolean pagination totals. The final runner
suite above passes. No Rust build/gate or full corpus execution was run.

### Post-implementation codebase-memory refresh

Skill-following full reindex command:

```text
codebase-memory-mcp cli index_repository --repo-path /home/ubuntu/repos/parseltongue-rust-LLM-companion --mode full
```

Output: `status=indexed`, project `home-ubuntu-repos-parseltongue-rust-LLM-companion`, 23,007 nodes,
43,379 edges, `indexed_at=2026-11-12T17:14:50Z`, 6 parse-partial files, 0 unusable files, and 17
intentionally ignored files. The index-status query returned `status=ready`, the same node/edge counts and
generation. `list_projects` returned one project at the requested repository root on
`devin/1794501865-sop-qualification-runner`; `get_graph_schema` returned 47 schema labels/types.

Scoped coverage command:

```text
codebase-memory-mcp cli check_index_coverage --project home-ubuntu-repos-parseltongue-rust-LLM-companion --paths '["tools/hld-current-qualification-runner.py","tools/test_sop_qualification_runner.py","tools/corpus-index-coverage-inventory.py","tools/corpus-production-stage-measurements.py","tools/process-tree-access-auditor.py"]' --path-limit 100 --path-offset 0 --diagnostics full --format json
```

Output: `path_total=5`, all 5 returned with `status=no_recorded_issue`, `freshness=metadata_match`, empty
coverage arrays, generation match and `recording_status=complete`; signal is `best_effort`, not proof of
completeness. A final reindex is performed after this journal checkpoint so the index also reflects these
results.

## Parent-review correction checkpoint

The initial GREEN above is historical and does not cover the nine issues identified in the parent review.
The parent requested focused regressions, a correction journal, and a GREEN/no-refactor checkpoint; no push,
PR, production build, full-corpus run, Rust change, S8/S9 work, or shared-document edit is in scope.

### RED — focused regressions before corrections

Added targeted cases for S3 component-versus-qualification status, terminal aggregate accounting, actual
orchestration/artifact finalization, live pagination cycles and native generation fidelity, external raw-page
and BLAKE3 manifest placement, M2 validation-component status, tracked/ignored/symlink source bytes, exact
target audit exceptions, unfinished traces, and toolchain-version availability. The earlier RED-phase
static assertion used `>= 4`; the correction check now requires exactly four words after `test_`.

Command before the corrections:

```text
set -o pipefail && PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py' -v 2>&1 | tail -n 180
```

Actual RED result (exit 1):

```text
Ran 61 tests in 0.470s
FAILED (failures=9, errors=9)
```

Representative exact failures:

```text
sop_qualification_runner.RunnerPreconditionBlockedError: source checkout is not clean: /tmp/tmpavr31tog/dirty-source: [' M src/lib.rs']
AssertionError: 2 != 1
AssertionError: coverage collector requested another page after its cursor cycle
TypeError: collect_index_coverage_inventory() got an unexpected keyword argument 'raw_response_directory_path'
KeyError: 'validator_component_status'
AttributeError: module 'sop_qualification_runner' has no attribute 'probe_toolchain_versions'
AssertionError: False is not true
Ran 61 tests in 0.470s
FAILED (failures=9, errors=9)
```

The RED run demonstrated that dirty Git state was misclassified as BLOCKED, pagination invoked further
commands on malformed/cyclic pages, raw artifact intake and toolchain/component APIs were absent, and the
orchestration path did not reach its final artifact manifest. This is a checkpoint record, not a passing test.

### Correction plan

- Keep mapping/measurement/audit component states separate from S3 qualification; S3 remains BLOCKED with
  explicit P2, fixture/build/dependency audit/qualification, and release-provenance gates not supplied by this
  runner. Continue across all five repositories only when component execution succeeds.
- Validate scope/missed response shape, generation and stable totals before each next query; maintain a set of
  all seen cursors; keep exact response text unchanged and record expected/boundary generations separately.
- Write all response pages to the artifact root, create BLAKE3 digests there, and publish a small per-repo
  digest/summary manifest directly at `<run-id>/<repo-name>` plus a compact manifest copy in the report.
- Convert post-freeze source/lock mutation or dirty-state inability to verify into FAIL. Aggregate preflight,
  input-integrity and artifact-digest terminal statuses; best-effort artifact preservation must not call an
  identity check that can no longer pass.
- Return the selected BLAKE3 tool identity from execution context and cover the actual execute/finalize path.
- Keep R4/R5 validation-component PASS distinct from authoritative corpus gates, both BLOCKED pending actual
  exported table/primary JSONL checks and completeness of required classification coverage.
- Hash all tracked bytes, inspect ignored Rust/Cargo inputs except known generated targets/run outputs, and
  block dependency closure when directory symlinks or external inputs cannot be frozen.
- Exempt only exact `.rustc_info.json` basenames and reject unfinished target traces; preserve the stated
  exact-path-only/non-recursive limitation.
- Preserve vertical REQ documentation and map each local ID to the existing SPEC REQ/SOP clauses above;
  do not update shared specifications. The final static check will require exactly four test-name words.

### GREEN / REFACTOR checkpoint

The first full correction run exposed three issues: the live A-B-A test fixture's total was inconsistent
with its page limit, the harness probe stored byte fields in JSON, and frozen-identity reporting assumed
indexer/strace identities had a mapper-only `canonical_path` field. The cycle fixture now has enough
pages for the cycle path, harness JSON retains decoded output only, and reporting uses the captured path
for non-mapper tools.

Initial correction-run output (before those fixes; this was a diagnosis run, not the earlier RED checkpoint):

```text
Ran 65 tests in 0.447s
FAILED (failures=1, errors=2)
```

The earlier RED checkpoint remains the actual pre-correction result: 61 tests, 9 failures, and 9 errors,
with representative failures recorded above. The final suite below includes 65 tests; no separate RED
output was captured for late-added test refinements after that historical checkpoint.

Static four-word name and journal traceability check:

```text
python3 -c 'import pathlib,re,sys; p=pathlib.Path("tools/test_sop_qualification_runner.py"); s=p.read_text(); names=re.findall(r"^\s+def (test_[A-Za-z0-9_]+)\(",s,re.M); bad=[n for n in names if len(n[5:].split("_")) != 4]; j=pathlib.Path("docs/phase-20261111/sop-runner-work-journal.md").read_text(); missing=[n for n in names if f"`{n}`" not in j]; print(f"test methods={len(names)}; non-four-word names={bad}; missing journal traceability={missing}"); sys.exit(bool(bad or missing))'
```

```text
test methods=65; non-four-word names=[]; missing journal traceability=[]
```

Python syntax check:

```text
python3 -c 'import ast,pathlib; paths=[pathlib.Path(p) for p in ("tools/hld-current-qualification-runner.py","tools/corpus-index-coverage-inventory.py","tools/corpus-production-stage-measurements.py","tools/test_sop_qualification_runner.py")]; [ast.parse(p.read_text(),filename=str(p)) for p in paths]; print("Python AST syntax: 4 files parsed")'
```

```text
Python AST syntax: 4 files parsed
```

Final correction runner suite:

```text
set -o pipefail && PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py' -v 2>&1 | tail -n 180
```

```text
Ran 65 tests in 0.463s
OK
```

Existing process-tree contract suite:

```text
PYTHONDONTWRITEBYTECODE=1 python3 tools/process-tree-contract-tests.py
```

```text
......
----------------------------------------------------------------------
Ran 6 tests in 0.341s

OK
```

No refactor was performed; the final correction is a focused GREEN/no-refactor checkpoint.

Final whitespace checks:

```text
git diff --check
git diff --cached --check
```

Both exited 0 with no output. Final branch/status review:

```text
devin/1794501865-sop-qualification-runner
AM docs/phase-20261111/sop-runner-work-journal.md
MM tools/corpus-index-coverage-inventory.py
MM tools/corpus-production-stage-measurements.py
AM tools/hld-current-qualification-runner.py
AM tools/test_sop_qualification_runner.py
```

`git diff HEAD --name-only` contains only those five scoped files. Shared SOP/SPEC, root AGENTS,
status/TDD journal, pin manifests, S8/S9 work, production/crates, and unrelated paths are unchanged. No full
corpus run or Rust gate was run. The original request's GREEN commit versus the latest no-refactor
checkpoint was not resolved by the lead; no commit was created. Existing staged and unstaged changes remain
uncommitted pending lead review; no push or PR was made.

#### Final codebase-memory refresh

CLI version: `codebase-memory-mcp 0.12.1`. Full reindex command:

```text
codebase-memory-mcp cli index_repository --repo-path /home/ubuntu/repos/parseltongue-rust-LLM-companion --mode full
```

Result: `status=indexed`, project `home-ubuntu-repos-parseltongue-rust-LLM-companion`, 23,070 nodes,
43,545 edges, 6 parse-partial files, 0 parse-unusable files, and 17 intentionally not-indexed files.
The follow-up status is `ready` at `2026-11-12T17:57:50Z` with matching counts.

```text
codebase-memory-mcp cli list_projects
  projects: 1; root=/home/ubuntu/repos/parseltongue-rust-LLM-companion
  branch=devin/1794501865-sop-qualification-runner; has_more=false
codebase-memory-mcp cli index_status --project home-ubuntu-repos-parseltongue-rust-LLM-companion --diagnostics full --format json
  status=ready; nodes=23070; edges=43545; indexed_at=2026-11-12T17:57:50Z
  parse_partial=6; parse_unusable=0; not_indexed.files_count=17
codebase-memory-mcp cli get_graph_schema --project home-ubuntu-repos-parseltongue-rust-LLM-companion --format json --diagnostics full --limit 100 --offset 0
  exit 0; schema returned 48 node-label/edge-type entries; has_more=false
codebase-memory-mcp cli check_index_coverage --project home-ubuntu-repos-parseltongue-rust-LLM-companion --paths '["tools/hld-current-qualification-runner.py","tools/test_sop_qualification_runner.py","tools/corpus-index-coverage-inventory.py","tools/corpus-production-stage-measurements.py","tools/process-tree-access-auditor.py","tools/process-tree-contract-tests.py"]' --path-limit 100 --path-offset 0 --diagnostics full --format json
  path_total=6; all 6 status=no_recorded_issue, freshness=metadata_match, coverage=[]
  recording_status=complete; signal=best_effort, not a completeness guarantee
```

The reindex includes the final code/test changes; the only subsequent change is this journaled command
and result record. Coverage is a best-effort signal, not proof of completeness.

### Follow-up integration corrections — STUB / RED

Local REQ traceability: REQ-SOP-RUNNER-004 covers S3 output isolation and target-trace terminal/wrapper
proof; REQ-SOP-RUNNER-010 covers exact S1/S2 command argv and before/after outcome logging; REQ-SOP-RUNNER-012
covers R4/R5 validator FAIL propagation while authoritative corpus gates remain BLOCKED for otherwise-PASS
components.

STUB/API plan before implementation:

- Pass an explicit `<artifact-repository>/production` directory to S3 command construction, helper output,
  and status paths; keep report summaries under `<run-id>/<repo-name>`.
- Extend target-trace audit input with the verified wrapper return code. A complete no-access trace must
  contain a clean traced-process exit and wrapper exit; empty, truncated, unfinished, or orphan-resumed
  traces remain blocked.
- Add a bounded external `commands.jsonl` recorder. Each command gets exact argv plus before/after events;
  completion events contain return code and output byte counts, never raw stdout/stderr. The recorder does
  not invoke the identity guard.
- Set corpus R4/R5 status to FAIL when an extension validator observes a contract violation; only missing
  independent artifact verification keeps an otherwise-PASS component BLOCKED.

Focused regression cases added before implementation:

- `test_repository_s3_directory_isolated`
- `test_exit_only_target_passes`
- `test_exit_only_requires_wrapper`
- `test_empty_target_trace_blocks`
- `test_truncated_target_trace_blocks`
- `test_orphan_resumed_target_blocks`
- `test_m2_validator_fail_aggregates`

RED checkpoints before implementation:

- `PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_exit_only_target_passes test_sop_qualification_runner.SopQualificationRunnerContractTests.test_exit_only_requires_wrapper test_sop_qualification_runner.SopQualificationRunnerContractTests.test_empty_target_trace_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_truncated_target_trace_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_orphan_resumed_target_blocks`:
  `Ran 5 tests in 0.002s`; `FAILED (errors=4)`. Four calls failed with
  `TypeError: audit_target_path_trace() got an unexpected keyword argument 'wrapper_return_code'`; the
  no-wrapper negative case already passed.
- `PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_repository_s3_directory_isolated`:
  `Ran 1 test in 0.008s`; `FAILED (failures=1)`. S1/S2 completed, but the real helper's nonempty-directory
  guard prevented its fake measurement invocation (`AssertionError: 0 != 1`).
- `PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_m2_validator_fail_aggregates`:
  `Ran 1 test in 0.005s`; `FAILED (failures=1)`. Observed R4 validator FAIL was reported as
  `BLOCKED` (`AssertionError: 'BLOCKED' != 'FAIL'`).

GREEN implementation and final verification outputs follow below.

### GREEN

Implementation outcomes:

- S3 command construction and the real measurement helper use a new, empty
  `<artifact-repository>/production` directory. The measurement summary is additionally retained at the
  small per-repository report path. The end-to-end harness-only regression reached the real helper
  directory check and verified the fresh path plus separate external command ledger.
- Target-trace completeness now requires a clean traced-process exit and wrapper return code 0; empty,
  truncated, unfinished, orphan-resumed, and forbidden-open evidence cannot pass. Exact
  `.rustc_info.json` basename handling remains unchanged.
- R4/R5 contract validator FAIL is propagated to the corresponding evidence result, S4/S5 gate, and
  aggregate. An otherwise-PASS component still has a BLOCKED corpus gate pending independent artifact
  verification.
- `commands.jsonl` is external under each artifact repository, stores exact S1/S2 argv and before/after
  outcomes (return code and stdout/stderr byte counts only), and is bounded to 16 KiB per event, 512
  events, and 8 MiB total. The recorder performs no identity-guard callback.

Runner suite:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py' -v
Ran 72 tests in 0.857s
OK
```

Existing process-tree contract suite:

```text
PYTHONDONTWRITEBYTECODE=1 python3 tools/process-tree-contract-tests.py
Ran 6 tests in 0.393s
OK
```

Static syntax/name/REQ traceability:

```text
Python AST syntax: 4 files parsed; test methods=72; non-four-word names=[]; missing journal traceability=[]; missing REQ tags=[]
```

The first static-command attempt used an over-escaped regex and reported test tags missing; the corrected
`REQ-SOP-RUNNER-[0-9]+` check above passed. No production build, real corpus run, or Rust gate was run.

Final diff checks before commit:

```text
git diff --check && git diff --cached --check && git diff HEAD --check
Exit code 0; no output.
```

### REFACTOR — no refactor needed

Reviewed the focused change set against the correction brief. The isolated production directory, target
trace proof, M2 failure propagation, and bounded command ledger are covered by direct regressions; no
additional refactor is warranted. GREEN commit: `0dfbfa14c`. This checkpoint contains only this journal
entry; no runner behavior changed after the GREEN test run.
