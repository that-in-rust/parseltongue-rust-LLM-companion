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
  `test_pin_mismatch_blocks_parser`, `test_cli_imports_avoid_bytecode`.

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
  `test_toolchain_versions_are_recorded`, `test_rustup_shims_preserve_invocation`.

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

### REQ-SOP-RUNNER-019 — paginated row identity

- Contract: use endpoint-reported identities to reject missing identities, exact duplicates, and
  conflicting content across S2 pages before the collector accepts the sequence.
- SOP: S2 complete scope/missed-File pagination and §4 raw page evidence.
- SPEC: REQ-RUST-001.2 offline graph/process boundary; REQ-RUST-003.1 source coverage.
- Tests: `test_scope_duplicate_identity_blocks`, `test_scope_identity_missing_blocks`,
  `test_scope_content_conflict_blocks`, `test_missed_duplicate_identity_blocks`,
  `test_missed_identity_missing_blocks`.

Endpoint grounding: installed `codebase-memory-mcp 0.12.1` `query_graph --help` documents `--graph
<code|missed>`, `--max-rows`, `--cursor`, and compatibility `--offset`. A small local response from
the existing indexed repository (no corpus) returned `columns:["f.file_path"]`, `rows:[[path],...],
total:6, has_more:true, next_cursor:...`; a two-entry `check_index_coverage --scopes '["."]'` page
returned scope entries shaped as `{path, kind, detail}` with `scope_total:26` and `next_offset:2`.
Therefore the scope identity is `entry.path`; the missed query's only selected cell is `f.file_path`.
Fixtures now match those observed wire shapes, not guessed string rows.

STUB/RED: all four new identity regressions failed against the current validators (which only totaled
rows). Exact output:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_scope_duplicate_identity_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_scope_identity_missing_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_scope_content_conflict_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_missed_duplicate_identity_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_missed_identity_missing_blocks
Ran 5 tests in 0.004s
FAILED (failures=5)
AssertionError: CoverageInventoryBlockedError not raised
```

### REQ-SOP-RUNNER-020 — stage fail-fast state

- Contract: a failed untraced mapper run prevents process/target/status commands; a failed process
  audit prevents target/status commands. Persist observed outcomes and explicit `NOT STARTED` states,
  with no fabricated timing, memory, or publication evidence.
- SOP: S3 separate untraced measurement/process audit/target audit/status sequence; §3 verdict semantics.
- SPEC: REQ-RUST-015.1 target/build-script boundary; REQ-RUST-020.1 generation isolation.
- Tests: `test_mapper_failure_skips_audits`, `test_process_failure_skips_status`.

STUB/RED: tests use a precise fake `subprocess.run` sequence and real helper output-directory/result
files; no corpus or production executable is invoked. The helper continued after both observed failures
and returned no explicit stage statuses. Exact output:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_mapper_failure_skips_audits test_sop_qualification_runner.SopQualificationRunnerContractTests.test_process_failure_skips_status
Ran 2 tests in 0.005s
FAILED (failures=2)
AssertionError: None != 'FAIL'
AssertionError: None != 'FAIL'
```

GREEN: the measurement helper writes a per-repository artifact summary before execution and updates
it with observed command exits/statuses. A nonzero untraced mapper return now exits before process,
target, or status commands; a process-audit violation/incomplete result exits before target/status.
Skipped stages retain `NOT STARTED`, return codes/timing/RSS/publication remain null unless observed,
and the runner copies these statuses into its persistent manifest without relabeling skipped commands
as failed. Component status and S3 failure/blocking follow the recorded observation.

Focused fake-sequence checks:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_mapper_failure_skips_audits test_sop_qualification_runner.SopQualificationRunnerContractTests.test_process_failure_skips_status
Ran 2 tests in 0.005s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_runner_persists_skipped_states
Ran 1 test in 0.005s
OK
```

### REQ-SOP-RUNNER-021 — transition command metadata

- Contract: S1/S2 transition manifests persist each executed command's exact argv, actual cwd, and
  exit result; abbreviated display commands alone are insufficient. Page commands retain cursor,
  offset, and project flags.
- SOP: S1/S2 execution evidence and exact command provenance; §3 transition/measurement traceability.
- SPEC: REQ-RUST-001.2 command/process evidence and REQ-RUST-020.1 generation-bound evidence.
- Test: `test_repository_s3_directory_isolated` (also retains REQ-SOP-RUNNER-004 coverage).

API check: installed `codebase-memory-mcp cli index_repository --help` requires `--repo-path` and
supports `--mode`; it does not define `--manifest-path` or `--corpus-root`. Runner `--help` exposes
`--corpus-manifest` and `--project-root`, and a Python-source search found neither requested flag.
The implementation therefore records executed argv generically, including the supported S1
`--repo-path` and actual S2 flags, without inventing unsupported options. Exact help result:

```text
--repo-path <string> [required]  Repository path
--mode <full|moderate|fast|cross-repo-intelligence> [default: full]
```

STUB/RED: the integration test exercised real S1/S2 collection with fake subprocess results and
failed because the S1 transition had only shortened `commands`, with no `command_metadata`:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_repository_s3_directory_isolated
Ran 1 test in 0.011s
ERROR (errors=1)
KeyError: 'command_metadata'
```

Follow-up review RED expanded REQ-018 to fail closed for addressless sends, unresolved `execveat`,
failed/undecodable exec attempts, and demonstrated `-yy` decoding; REQ-020 now preserves genuine
telemetry from a failed mapping command, skips publication after a failed target audit, and normalizes
missing command exits as NOT STARTED. The existing S1/S2 harness test also checks transition metadata.
The combined focused run exposed each missing behavior:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_trace_captures_syscall_families test_sop_qualification_runner.SopQualificationRunnerContractTests.test_unaddressed_sends_require_proof test_sop_qualification_runner.SopQualificationRunnerContractTests.test_socketpair_local_send_passes test_sop_qualification_runner.SopQualificationRunnerContractTests.test_execveat_relative_path_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_failed_exec_payload_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_failed_forbidden_exec_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_mapper_failure_retains_samples test_sop_qualification_runner.SopQualificationRunnerContractTests.test_target_failure_skips_status test_sop_qualification_runner.SopQualificationRunnerContractTests.test_runner_persists_skipped_states test_sop_qualification_runner.SopQualificationRunnerContractTests.test_repository_s3_directory_isolated
Ran 10 tests in 0.024s
FAILED (failures=8, errors=1)
AssertionError: '-yy' not found in [... trace=%process,%network ...]
AssertionError: 0 != 3
AssertionError: False is not true
AssertionError: None != 2.5
AssertionError: 'PASS' != 'NOT STARTED'
AssertionError: 'FAIL' != 'NOT STARTED'
KeyError: 'command_metadata'
```

GREEN: scope pages now require a nonempty `path` identity and track complete entry content across
pages, distinguishing exact repeats from conflicting records. Missed-query rows must match the observed
single `f.file_path` column, contain one nonempty path, agree with the page's `returned` count and
`total_relation == "eq"`, and be unique across pages. The collector already validates each received
page before another cursor query; the new integration regression verifies a duplicate page is preserved
raw and stops collection after its second query. Because the SOP query projects only `f.file_path`,
conflicting extra payload for a missed-file identity is not representable in a valid response; unexpected
columns/row widths block, while conflicting record content is checked on actual scope `{path, kind,
detail}` entries.

Focused validator outputs:

```text
Ran 5 tests in 0.001s
OK

Ran 1 test in 0.004s
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

### CLI import hygiene correction — STUB / RED

Local REQ traceability: REQ-SOP-RUNNER-002 covers the runner/helper source identity frozen by execute mode.
The runner dynamically imports `corpus-index-coverage-inventory.py` before importing the measurement
helper that sets `sys.dont_write_bytecode`; the planned stub sets that flag before loading support modules.

Regression planned before implementation: `test_cli_imports_avoid_bytecode` copies the runner/support
scripts into an isolated temporary project, starts the CLI with default Python bytecode behavior using
`--help`, and asserts successful parser startup without creating `.pyc` files. It does not create a report
or run corpus work. RED output follows.

RED checkpoint:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_cli_imports_avoid_bytecode
Ran 1 test in 0.091s
FAILED (failures=1)
AssertionError: Lists differ: ['tools/__pycache__/corpus-index-coverage-inventory.cpython-312.pyc', 'tools/__pycache__/corpus-production-stage-measurements.cpython-312.pyc'] != []
```

### CLI import hygiene correction — GREEN

Set `sys.dont_write_bytecode = True` before loading dynamic support modules. GREEN checks:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_cli_imports_avoid_bytecode
Ran 1 test in 0.090s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py' -v
Ran 73 tests in 0.561s
OK

PYTHONDONTWRITEBYTECODE=1 python3 tools/process-tree-contract-tests.py
Ran 6 tests in 0.341s
OK

python3 tools/hld-current-qualification-runner.py --help >/dev/null && pyc_count=$(find tools -type f -name '*.pyc' | wc -l) && test "$pyc_count" -eq 0 && printf 'CLI --help: source-tree .pyc files=%s\n' "$pyc_count"
CLI --help: source-tree .pyc files=0
```

### Runner usage and acceptance boundary

Safe planning example; replace the run ID with a new unique value. Planning writes a truthful NOT STARTED
report under the default `hld-current-test-runs` root and invokes no mapper, oracle, or indexer:

```bash
python3 tools/hld-current-qualification-runner.py \
  --run-id m1-plan-unique-001 --milestone M1 --mode plan
```

Execute-mode invocation outline (not executed here):

```bash
python3 tools/hld-current-qualification-runner.py \
  --run-id m1-execution-unique-001 --milestone M1 --mode execute \
  --mapper-binary /absolute/path/to/frozen/mapper \
  --report-root hld-current-test-runs \
  --artifact-root "$HOME/hld-current-test-artifacts" \
  --indexer-path codebase-memory-mcp
```

Use a new run ID for every invocation. `--corpus-manifest` defaults to the pinned corpus manifest;
repeat `--project-root NAME=PATH` only for an intentional checkout override. M2 component inputs are
`--r4-evidence PATH` and `--r5-evidence PATH`.

This CLI is **not yet a turnkey full-S3 acceptance tool**. Full S3 remains BLOCKED until current-run,
frozen-identity-bound fixture-suite, build qualification, P2, dependency-audit, and release-provenance
evidence can be supplied and validated; those evidence-intake gates are not implemented here. R4/R5 are
structured component validators only, not independent authoritative exported-table or primary-JSONL
verification. S4/S5 remain unavailable/BLOCKED pending those integrations. No full-corpus qualification
run was performed.

### REFACTOR — no refactor needed

The import guard and isolated CLI regression are focused; no broader refactor is needed. GREEN code commit:
`9c538d6ff95888bd20a373955ee6fcedd35daabc`. This checkpoint is documentation-only; no code changed after
the GREEN test run.

### Scoped codebase-memory refresh

Refreshed once after the final code commit; the following no-refactor checkpoint changes only this journal
and will not trigger another reindex. Code commit analyzed: `9c538d6ff95888bd20a373955ee6fcedd35daabc`.

```text
~/.local/bin/codebase-memory-mcp cli index_repository --repo-path /home/ubuntu/repos/parseltongue-rust-LLM-companion --mode full
status=indexed; nodes=23089; edges=43614; parse_partial=6; not_indexed_files=17

~/.local/bin/codebase-memory-mcp cli index_status --project home-ubuntu-repos-parseltongue-rust-LLM-companion --diagnostics full --format json
status=ready; indexed_at=2026-11-12T18:20:09Z; nodes=23089; edges=43614; generation_matches=true

~/.local/bin/codebase-memory-mcp cli check_index_coverage --project home-ubuntu-repos-parseltongue-rust-LLM-companion --paths '["tools/hld-current-qualification-runner.py","tools/corpus-index-coverage-inventory.py","tools/corpus-production-stage-measurements.py","tools/test_sop_qualification_runner.py","docs/phase-20261111/sop-runner-work-journal.md"]' --diagnostics full --format json
path_total=5; path_returned=5; each status=no_recorded_issue; each freshness=metadata_match
```

Scoped coverage is explicitly a best-effort signal, not a completeness guarantee. The first index command
attempt mistakenly passed unsupported `--format json` and exited 1; the exact command above succeeded.

### Toolchain shim dispatch correction — STUB / RED

REQ-SOP-RUNNER-009: execute `cargo --version` and `rustc --version` through the PATH-selected invocation
paths so rustup-style symlink dispatch is preserved, while recording each resolved canonical executable
path separately. The focused regression `test_rustup_shims_preserve_invocation` uses real cargo/rustc
symlinks to one executable that prints its invoked basename; it is REQ-tagged and four words after `test_`.
The expected RED result will show the current canonical-path invocation returning `rustup` rather than
the requested `cargo`/`rustc` identity.

RED checkpoint:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_rustup_shims_preserve_invocation
Ran 1 test in 0.004s
FAILED (failures=1)
AssertionError: 'rustup shim-version' != 'cargo shim-version'
```

### Toolchain shim dispatch correction — GREEN

`probe_toolchain_versions` now executes the PATH-selected path and records it in `path`, with the resolved
target separately in `canonical_path`. The real symlink regression observes cargo/rustc identity through
the shim and checks both returned paths.

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_rustup_shims_preserve_invocation
Ran 1 test in 0.004s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py' -v
Ran 74 tests in 0.580s
OK
```

### Toolchain shim dispatch correction — REFACTOR

No refactor needed: the change is limited to `probe_toolchain_versions`, and the symlink regression covers
both the invoked version name and separately recorded canonical path. GREEN commit:
`b22b92358bb187a5e8ab5e24d8d401d4a9d2dd1d`; this checkpoint changes only the journal.

### Output, lock, and publication safeguards — local REQ traceability and STUB

This bounded batch adds only output-placement validation, explicit selected-lock freezing, and
published-stage-map validation. These local identifiers document the tool behavior; they do not
modify or claim to add rows to the shared SPEC verification matrix.

- `REQ-SOP-RUNNER-015` maps to SOP revision R3 (persistent report/artifact locations) and SPEC
  source snapshot exclusions at §0 / the source snapshot contract. Outputs must be canonical and
  disjoint from project/corpus inputs except the exact project `hld-current-test-runs` report root;
  source snapshots exclude only newly owned report/artifact run directories.
- `REQ-SOP-RUNNER-016` maps to SPEC lock existence/hash checks at lines 41 and 58 and the frozen
  source snapshot contract at line 337. Only selected serde/tokio top-level `Cargo.lock` files are
  permitted as ignored/untracked inputs, with exact-byte identity and provenance retained.
- `REQ-SOP-RUNNER-017` maps to SPEC publication/generation state at lines 390, 397, and 417–418.
  A complete published-stage map and nonempty generation identity are required; legacy static
  implementation statuses are not publication evidence.

STUB plan: add pre-write canonical output-root validation and safe stderr failure, pass exact owned
run directories to snapshots, whitelist only selected top-level locks in the snapshot/identity
registry, and switch the publication validator to `stage_publication_status_values`. RED tests are
tagged with the corresponding local REQ above and use four-word `test_` names. No mapper, source
manifest, corpus, lockfile, or production file is changed.

### Output, lock, and publication safeguards — RED

Focused RED command (Python fixtures only; no Cargo, corpus, mapper, or production execution):

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_checkout_root_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_source_ancestor_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_source_subtree_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_corpus_ancestor_root_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_corpus_root_output_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_symlink_bypass_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_symlink_alias_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_safe_paths_validate test_sop_qualification_runner.SopQualificationRunnerContractTests.test_outputs_block_before_writes test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_exclusions_detect_mutations test_sop_qualification_runner.SopQualificationRunnerContractTests.test_selected_locks_are_frozen test_sop_qualification_runner.SopQualificationRunnerContractTests.test_selected_lock_modification_fails test_sop_qualification_runner.SopQualificationRunnerContractTests.test_selected_lock_deletion_fails test_sop_qualification_runner.SopQualificationRunnerContractTests.test_unselected_lock_stays_blocked test_sop_qualification_runner.SopQualificationRunnerContractTests.test_publication_map_accepts_complete test_sop_qualification_runner.SopQualificationRunnerContractTests.test_publication_map_requires_stages test_sop_qualification_runner.SopQualificationRunnerContractTests.test_publication_map_requires_generation test_sop_qualification_runner.SopQualificationRunnerContractTests.test_static_status_cannot_publish
```

Observed RED output:

```text
test_output_checkout_root_blocks ... FAIL
test_output_source_ancestor_blocks ... FAIL
test_output_source_subtree_blocks ... FAIL
test_corpus_ancestor_root_blocks ... FAIL
test_corpus_root_output_blocks ... FAIL
test_output_symlink_bypass_blocks ... FAIL
test_output_symlink_alias_blocks ... FAIL
test_output_safe_paths_validate ... ok
test_outputs_block_before_writes ... FAIL
test_output_exclusions_detect_mutations ... ERROR
test_selected_locks_are_frozen ... ERROR
test_selected_lock_modification_fails ... ERROR
test_selected_lock_deletion_fails ... ERROR
test_unselected_lock_stays_blocked ... ok
test_publication_map_accepts_complete ... FAIL
test_publication_map_requires_stages ... ok
test_publication_map_requires_generation ... ok
test_static_status_cannot_publish ... FAIL
Ran 18 tests in 0.278s
FAILED (failures=10, errors=4)
AssertionError: RunnerPreconditionBlockedError not raised
AttributeError: module 'sop_qualification_runner' has no attribute 'build_owned_run_exclusions'
RunnerPreconditionBlockedError: ignored untracked source inputs found in /tmp/tmptgu4nk9e/lock-serde: ['Cargo.lock']
AssertionError: False is not true : coverage banner has no stage implementation statuses
AssertionError: True is not false
```

The output-path RED confirms preflight accepted checkout/corpus ancestors, source descendants, and
symlink aliases and wrote the fixture report. The selected-lock RED is the generic ignored-source
rejection; the publication RED confirms legacy implementation statuses passed while the new map was
ignored. Green implementation follows below.

### Output, lock, and publication safeguards — GREEN

Output roots are canonicalized and checked against the project root plus all five selected corpus
roots before `create_persistent_run_report` or artifact-directory creation. Any symlink component
blocks the root, reports are permitted only externally or at the exact project
`hld-current-test-runs` root, and artifacts must be disjoint from every input root. Unsafe roots
produce a BLOCKED JSON record on stderr with exit 2 and no report/artifact writes. Snapshot exclusions
now contain only the exact new report and artifact run directories; a post-freeze ignored Rust source
elsewhere beneath the report parent is detected.

Snapshot/registry evidence now admits only the top-level `Cargo.lock` selected for serde or tokio.
The exact file identity is included in both the lock registry and source snapshot digest, with Git
provenance recorded as `ignored` or `untracked`; modification and deletion raise frozen-input FAIL.
Other ignored/untracked inputs remain blocked. Publication validation now requires
`stage_publication_status_values` with applicable stages marked `complete` and a nonempty published
generation digest. Missing publication status (including legacy static implementation statuses) is
BLOCKED, and incomplete stage values cannot PASS. The mapper and all non-scoped files remain
untouched; full S3 prerequisites remain conservative.

Focused GREEN command and output:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_checkout_root_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_source_ancestor_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_source_subtree_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_corpus_ancestor_root_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_corpus_root_output_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_symlink_bypass_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_symlink_alias_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_safe_paths_validate test_sop_qualification_runner.SopQualificationRunnerContractTests.test_outputs_block_before_writes test_sop_qualification_runner.SopQualificationRunnerContractTests.test_output_exclusions_detect_mutations test_sop_qualification_runner.SopQualificationRunnerContractTests.test_selected_locks_are_frozen test_sop_qualification_runner.SopQualificationRunnerContractTests.test_selected_lock_modification_fails test_sop_qualification_runner.SopQualificationRunnerContractTests.test_selected_lock_deletion_fails test_sop_qualification_runner.SopQualificationRunnerContractTests.test_unselected_lock_stays_blocked test_sop_qualification_runner.SopQualificationRunnerContractTests.test_publication_map_accepts_complete test_sop_qualification_runner.SopQualificationRunnerContractTests.test_publication_map_requires_stages test_sop_qualification_runner.SopQualificationRunnerContractTests.test_publication_map_requires_generation test_sop_qualification_runner.SopQualificationRunnerContractTests.test_static_status_cannot_publish
Ran 18 tests in 0.372s
OK
```

After the publication-missing reason was made explicitly BLOCKED, its four focused regressions passed:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_publication_map_accepts_complete test_sop_qualification_runner.SopQualificationRunnerContractTests.test_publication_map_requires_stages test_sop_qualification_runner.SopQualificationRunnerContractTests.test_publication_map_requires_generation test_sop_qualification_runner.SopQualificationRunnerContractTests.test_static_status_cannot_publish
Ran 4 tests in 0.001s
OK
```

Final scoped suites:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py'
Ran 92 tests in 0.944s
OK

PYTHONDONTWRITEBYTECODE=1 python3 tools/process-tree-contract-tests.py
Ran 6 tests in 0.341s
OK
```

No Cargo command, lock regeneration, corpus run, mapper run, production build, or graph/index operation was performed.

### Output, lock, and publication safeguards — REFACTOR checkpoint

No refactor needed: the output validation and owned-run exclusion helpers are narrowly scoped, lock
permission is explicit at the serde/tokio source-root boundary, and publication validation reuses
the existing generation/tiling/count checks with the new publication map. GREEN commit:
`c9d4c18da843e38c0b663f8abc6bd05667df8e69`. This is a separate journal-only checkpoint; subsequent
review requests about other audit helpers remain out of scope.

### REQ-SOP-RUNNER-018 — process/network syscall audit

- Contract: trace the complete process and network syscall families; enforce execveat and Internet
  send-family violations while retaining the SOP's permitted local AF_UNIX behavior.
- SOP: S3 separate process-access audit and §0 production-child/network boundary.
- SPEC: REQ-RUST-001.2 offline graph and process boundary; reject disallowed descendants/network.
- Tests: `test_trace_captures_syscall_families`, `test_execveat_spawn_is_forbidden`,
  `test_internet_send_families_fail`, `test_unix_sends_remain_allowed`.

STUB/RED: tests were added before code changes. The real S3 command builder still named only
`execve,connect,clone,clone3,fork,vfork`; successful `execveat` was classified merely as incomplete,
and `sendmmsg` was not rejected. The AF_UNIX preservation regression already passed.

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_trace_captures_syscall_families test_sop_qualification_runner.SopQualificationRunnerContractTests.test_execveat_spawn_is_forbidden test_sop_qualification_runner.SopQualificationRunnerContractTests.test_internet_send_families_fail test_sop_qualification_runner.SopQualificationRunnerContractTests.test_unix_sends_remain_allowed
Ran 4 tests in 0.003s
FAILED (failures=3)
test_unix_sends_remain_allowed ... ok
AssertionError: 'trace=%process,%network' not found in [... trace=execve,connect,clone,clone3,fork,vfork ...]
AssertionError: False is not true
AssertionError: 2 != 3
```

GREEN: the S3 audit now requests strace's full `%process,%network` classes with `-f`, the parser
classifies successful `execveat` with the same full-argv allowlist as `execve`, and socket/connect/
sendto/sendmsg/sendmmsg events carrying AF_INET or AF_INET6 fail. AF_UNIX connect/sendmsg remains
allowed. Focused output:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_trace_captures_syscall_families test_sop_qualification_runner.SopQualificationRunnerContractTests.test_execveat_spawn_is_forbidden test_sop_qualification_runner.SopQualificationRunnerContractTests.test_internet_send_families_fail test_sop_qualification_runner.SopQualificationRunnerContractTests.test_unix_sends_remain_allowed
Ran 4 tests in 0.003s
OK
```

### REQ-SOP-RUNNER-018 — UNIX-STREAM descriptor evidence RED

The `-yy` annotated descriptor form `UNIX-STREAM:[...]` is positive evidence for an AF_UNIX endpoint,
including addressless sends on inherited descriptors. The parser initially recognized only `UNIX:`;
this focused regression was added before the format matcher changed.

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_annotated_unix_fd_allowed
Ran 1 test in 0.007s
FAILED (failures=1)
AssertionError: Lists differ: ['100: unproven network send'] != []
```

The first full process-tree contract run then exposed one stale assertion that expected the former
hand-selected syscall list rather than the new complete strace classes. This was a test-contract
failure, not an audit implementation failure:

```text
PYTHONDONTWRITEBYTECODE=1 python3 tools/process-tree-contract-tests.py
Ran 6 tests in 0.360s
FAILED (failures=1)
AssertionError: 'execve' not found in 'trace=%process,%network'
```

### Final bounded corrections — GREEN

The production trace now asks strace for `trace=%process,%network` with `-yy`; strace's installed
help documents `-yy, --decode-fds=all` as printing available file-descriptor information. The auditor
rejects IPv4/IPv6 activity regardless of syscall success, unproven addressless network sends, unresolved
relative/empty `execveat`, malformed exec payloads, and failed forbidden exec attempts. It preserves
proved AF_UNIX socket/socketpair descriptors across the traced process ancestry and accepts documented
`UNIX:` and `UNIX-STREAM:` descriptor annotations. The process-contract test now asserts the complete
strace classes and `-yy`, replacing its obsolete selected-syscall expectation.

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_trace_captures_syscall_families test_sop_qualification_runner.SopQualificationRunnerContractTests.test_execveat_spawn_is_forbidden test_sop_qualification_runner.SopQualificationRunnerContractTests.test_internet_send_families_fail test_sop_qualification_runner.SopQualificationRunnerContractTests.test_unix_sends_remain_allowed test_sop_qualification_runner.SopQualificationRunnerContractTests.test_unaddressed_sends_require_proof test_sop_qualification_runner.SopQualificationRunnerContractTests.test_socketpair_local_send_passes test_sop_qualification_runner.SopQualificationRunnerContractTests.test_annotated_unix_fd_allowed test_sop_qualification_runner.SopQualificationRunnerContractTests.test_execveat_relative_path_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_failed_exec_payload_blocks test_sop_qualification_runner.SopQualificationRunnerContractTests.test_failed_forbidden_exec_blocks
Ran 10 tests in 0.004s
OK

PYTHONDONTWRITEBYTECODE=1 python3 tools/process-tree-contract-tests.py
Ran 6 tests in 0.356s
OK
```

S3 reads and persists any genuine memory sample before handling the mapper exit. A failed mapper remains
FAIL while valid telemetry is retained; missing telemetry is BLOCKED/unavailable. Process-audit failure
stops before target/status, target-audit failure stops before status/publication, and absent exits for
unattempted runner components remain NOT STARTED rather than becoming FAIL. Focused fake-command results:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_mapper_failure_skips_audits test_sop_qualification_runner.SopQualificationRunnerContractTests.test_process_failure_skips_status test_sop_qualification_runner.SopQualificationRunnerContractTests.test_mapper_failure_retains_samples test_sop_qualification_runner.SopQualificationRunnerContractTests.test_target_failure_skips_status
Ran 4 tests in 0.010s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_runner_persists_skipped_states
Ran 1 test in 0.005s
OK
```

S1 and S2 transition manifests now carry exact captured `argv`, child `cwd`, and exit result, including
each S2 page request. The actual S1 indexer uses `--repo-path`; installed help has no `--manifest-path`
or `--corpus-root`. S2 scope pages use the supported `--scope-offset` (not `--offset`), project flags,
and missed-page `--cursor`. An initially over-specific regression assertion for `--offset` was corrected
after source inspection; the implementation and actual API shape were unchanged by that test correction.

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_repository_s3_directory_isolated
Ran 1 test in 0.011s
OK
```

Final scoped checks:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py'
Ran 113 tests in 1.172s
OK

Python AST syntax: 6 files parsed; test methods=119; non-four-word names=[]; missing journal traceability=[]; missing REQ tags=[]

diff checks: exit 0, no output
```

### REQ-SOP-RUNNER-018 — no-refactor-needed checkpoint

The fail-closed network correction and its regression tests were committed as GREEN in
`6fd7c5a08` (`fix SOP audit and fail-fast evidence`). Review found no additional refactor needed: the
local-IPC proof is per-event, socketpair data is not treated as FD provenance, and ambiguous sends remain
unproven. No production, corpus, or unrelated files were changed. This checkpoint is journal-only; the
runner/process tests above were run against the committed code before this entry.

No corpus, mapper, production build, Rust gate, Cargo/lock regeneration, or graph refresh was run.
Overall full milestone never PASS merely from S1-S3; M1 S10 and M2 S4/S5 are separate not-started/blocked
gates until supplied and validated. Full S3 acceptance remains blocked pending current-run frozen-identity-bound
fixture/build/P2/dependency/release evidence intake; R4/R5 remain component validators and S4/S5 unavailable.

User R9 overrides memory: NO cap imposed by runner and NO memory acceptance gate until all-five S5; only
measure actual effective DuckDB setting and sampled whole-tree RSS; final <=500 MB deferred.

Future independent zcode source-grounded HLD-current audit S8/S9 is owned by parent on isolated branch.
Do NOT implement it or edit shared docs. Existing deterministic S8 sample and S9 proofs unchanged. Keep
S1–S3/M2 extension scope, don't wait for audit work. Missing later-stage exports always NOT STARTED/BLOCKED,
never fixture-based corpus PASS.

### REQ-SOP-RUNNER-018 — stale descriptor proof RED

Final review identified that recursively trusting a descriptor created by `socketpair` can falsely allow
an addressless send after close/reuse, and parsing the socketpair array is unnecessary. A new regression
requires per-event AF_UNIX sockaddr or `-yy` UNIX annotation; bare socketpair numbers and a later reused
number must remain unproven.

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_stale_socket_proof_rejected
Ran 1 test in 0.002s
FAILED (failures=1)
AssertionError: 0 != 2
```

### REQ-SOP-RUNNER-018 — per-event local IPC proof GREEN

This final review correction supersedes the earlier descriptor-provenance wording above: no persistent
FD map or ancestor lookup is used. AF_UNIX socket/socketpair creation does not authorize later sends.
An addressless send is allowed only when that event itself has a `-yy` UNIX/UNIX-STREAM descriptor
annotation; an explicit non-null AF_UNIX address remains allowed. The socketpair argument array is no
longer parsed as descriptor identity, so close/reuse and decorated inode digits cannot become proof.

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_stale_socket_proof_rejected
Ran 1 test in 0.002s
OK
```

Supplemental regression for decorated socketpair arguments confirms that inode digits are never parsed
as FD identities:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_sop_qualification_runner.SopQualificationRunnerContractTests.test_socketpair_inode_fd_untrusted
Ran 1 test in 0.002s
OK
```

Final verification after the stale-provenance and decorated-socketpair regressions:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py'
Ran 115 tests in 1.015s
OK

PYTHONDONTWRITEBYTECODE=1 python3 tools/process-tree-contract-tests.py
Ran 6 tests in 0.342s
OK

Python AST syntax: 6 files parsed; test methods=121; non-four-word names=[]; missing journal traceability=[]; missing REQ tags=[]

diff checks: exit 0, no output
```
