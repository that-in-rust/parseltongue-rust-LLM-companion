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

### REQ-SOP-RUNNER-022 — RED

The first synthetic helper tests were added before validator implementation. Actual command/output:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_s3_prerequisite_evidence_intake.py' -v
Ran 6 tests in 0.002s
FAILED (errors=6)
NotImplementedError at parse_s3_evidence_json, validate_s3_evidence_receipt, and validate_s3_prerequisite_evidence
```

The failures reproduce the unimplemented strict JSON, numeric receipt, timeout, and duplicate-reference
contracts; no Cargo, mapper, corpus, or native Rust command was run.

### REQ-SOP-RUNNER-022 — schema and later evidence collection

Schema v1 requires `schema_version`, `run_id`, `milestone`, exact runtime `identity_binding`, four global
prerequisite families (`fixture_suite`, separate `fmt`/`clippy` build receipts, `dependency_audit`, and
`release_provenance`), and a per-pinned-repository `repository_binding` plus distinct P2 metadata/check
receipts. Each reference is `{path,size_bytes,sha256,blake3}`. Receipt JSON binds family, scope, exact
argv/cwd/environment, finite elapsed/timeout, completion/timeout/exit fields, and complete stdout/stderr
descriptors. The runner loads and freezes inputs in prepared context; it does not create receipts or expose
an identity-freezer command. Missing inputs stay BLOCKED; observed nonzero/identity/hash contradictions are
FAIL. Hashes establish byte consistency, not independent proof that a command ran as claimed.

Truthful nonqualifying example (intentionally incomplete; never evidence of a successful run):

```json
{"schema_version":1,"run_id":"example-not-run","milestone":"M1","identity_binding":{},"global_prerequisites":{},"repositories":{}}
```

Later collection recipe: first prepare/freeze the exact mapper, runner source, lockfiles, parser pin/bundle,
R1 patch, toolchain, config/environment, repository snapshots and P2 unit policy. Do not derive a new
identity from the receipt bundle. Run each required command separately from its bound workspace cwd; for
example production checks use `cargo fmt --all -- --check`, `cargo clippy --message-format=json --workspace
--all-targets --all-features -- -D warnings`, unfiltered `cargo test --workspace`, the pinned normal-edge
`cargo tree`, and the separate release build. For each pinned repository, run offline+locked metadata and
the exact `cargo check --workspace --lib --bins --message-format=json` policy. Preserve raw stdout/stderr
outside project/corpus/output roots; record actual argv, cwd, non-secret relevant environment, elapsed,
timeout and exit result. Compute every descriptor from the retained bytes with the runner's existing
SHA-256/BLAKE3 identity helpers, then finalize manifest references only after commands finish. Do not
substitute a hand-authored PASS flag or example bundle.

Validator GREEN after subsequent P2, source-inventory and publication-token integration changes:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_s3_prerequisite_evidence_intake.py'
Ran 31 tests in 0.086s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py'
Ran 121 tests in 1.231s
OK

PYTHONPYCACHEPREFIX=/tmp/s3-intake-pycache python3 -m py_compile tools/hld-current-qualification-runner.py tools/s3-prerequisite-evidence-intake.py tools/test_sop_qualification_runner.py tools/test_s3_prerequisite_evidence_intake.py
exit 0
```

No-refactor checkpoint: the current review batch changes only the scoped runner, helper, tests and this
journal; no Cargo, mapper, corpus, native Rust test, production crate, or shared documentation was run or
edited. Full external-receipt authenticity remains outside this validator's claim.

## REQ-SOP-RUNNER-022 — append-only implementation checkpoint

This REQ-022 overview and historical STUB checkpoint are now placed after all pre-existing HEAD content;
the original journal bytes are unchanged. The former STUB is superseded by the implementation and results
below. This local runner requirement adds no Cargo evidence and no mapper, corpus, or native-Rust command.
It traces to SPEC §4 REQ 001–004, 010, 014–017, 020–022 at M0/M1/M2 applicability and SOP §1 P1–P5,
§2 S3, and §3 M0–M2. Shared SPEC/SOP/AGENTS remain unchanged.

### REQ-SOP-RUNNER-022 — historical STUB checkpoint

The initial planned pure API was `parse_s3_evidence_json`, `validate_s3_evidence_bundle`, command-family
validators, and source-backed fixture inventory, with a runner I/O adapter. The initial test plan covered
duplicate/malformed JSON, receipt identity/file integrity, command grammar, source fixture accounting and
conditional ignores, gate aggregation, freeze mutation, and pre-mapper stop. The RED implementation stub
was replaced by the strict helper and adapter exercised in the recorded GREEN results below.

The optional input is `--s3-prerequisite-evidence PATH`. The helper is pure over strict-decoded documents,
receipt/log bytes, runtime identity, source-derived repository bindings, and frozen Rust fixture inventory;
it never executes manifest-supplied commands. The runner freezes manifest/receipt/log bytes and present and
absent identities, checks those identities at boundaries, records five gate families, and blocks/fails
before S3 output or mapping unless all prerequisites and the internal run/repository/identity token pass.
Public PASS maps cannot promote a run; HARNESS ONLY synthetic evidence leaves the overall run
`HARNESS ONLY`. Top-level wrong run/milestone/identity violations propagate FAIL to fallback gate rows,
while a later repository's P2 result remains repository-scoped and does not fail `serde`.

### Exact schema v1 fields

The manifest has exactly `schema_version`, `run_id`, `milestone`, `identity_binding`,
`global_prerequisites`, and `repositories`. `schema_version` is integer 1 (not bool). The runtime identity
binds the run/milestone, mapper path and SHA-256/BLAKE3, runner/source and lock identities, parser/R1
identities, toolchain/configuration, and relevant secret-free environment. `global_prerequisites` contains
`fixture_suite`, `build_evidence` with separate `fmt`/`clippy`, `dependency_audit`, and `release_provenance`.
The repository map is exactly the pinned repository set; each value has exactly `repository_binding` and
`p2_offline_qualification`, containing two independent P2 receipt references (`p2_metadata`, `p2_check`).

Each reference has exactly `{path,size_bytes,sha256,blake3}` and names a unique retained regular file by a
relative non-alias path. A receipt has exactly `{schema_version,receipt_id,family,scope,identity_binding,
argv,cwd,relevant_environment,elapsed_seconds,timeout_seconds,completion_status,timed_out,exit_status,
stdout,stderr}`; P2 receipts additionally require `repository_binding`. The `stdout` and `stderr`
descriptors each have exactly `{path,size_bytes,sha256,blake3,complete}`; `complete` must be true. Receipt
and descriptor sizes/hashes are checked against retained bytes. Duplicate JSON keys/nonfinite numbers,
unsupported shapes, absent data, and incomplete streams cannot produce PASS; observed command/test/build/
dependency/hash/identity violations are FAIL. A receipt hash proves byte consistency, not who produced it.

### Required command argv table

| Receipt family / scope | Required argv after frozen PATH-dispatched Cargo |
| --- | --- |
| `fixture_suite` / production | `test --workspace` (unfiltered) |
| `fmt` / production | `fmt --all -- --check` |
| `clippy` / production | `clippy --message-format=json --workspace --all-targets --all-features -- -D warnings` |
| `dependency_audit` / production | `tree --package parseltongue-cli-dispatcher-binary --edges normal --prefix none --format {p}` |
| `release_build` / production | `build --release --package parseltongue-cli-dispatcher-binary --bin parseltongue --message-format=json` |
| `p2_metadata` / each pinned repo | `metadata --format-version 1 --offline --locked` |
| `p2_check` / each pinned repo | `check --workspace --lib --bins --offline --locked --message-format=json` |

The actual argv[0] must be the current PATH-dispatched Cargo executable whose frozen file identity is in
the runtime binding; an arbitrary same-basename executable is not accepted. P2 source inventory includes
canonical `src_path`; Cargo metadata target sources and check-artifact `target.src_path` plus
`manifest_path` are compared to the expected unit. Missing path data is BLOCKED, observed mismatch is FAIL,
and unsupported external/ambiguous target paths stay BLOCKED.

### Bounded future receipt collection

Prepare the ordinary runner context first and keep its identity helpers and repository/unit bindings; never
derive expected identity or argv from receipts. Run each required command separately in the bound cwd.
Capture monotonic start/end readings and calculate elapsed with `time.monotonic()`, recording configured
timeout, completion, and actual exit. A timed-out command with no complete result remains BLOCKED; exit 124
alone is not an assertion failure. A separately observed failed test, explicit build-finished false, or
completed nonzero remains FAIL even if a sibling stream/receipt is incomplete. Retain exact raw stdout and
stderr bytes outside project/corpus/report/artifact roots; do not normalize, truncate, or fabricate them.
After stream completion, compute byte count, SHA-256 and BLAKE3 with the runner's identity helpers, then
write receipt and manifest references. The runner never executes an arbitrary manifest command or offers an
evidence-creation shortcut. Do not fabricate real command/test/dependency/build evidence.

External receipt honesty is not authenticated by hashes, and command output alone cannot authenticate that
a historical command ran. Missing exact M2 crosswalk names in THIS checkout remain required-but-unproven
and BLOCKED; no aliases/waivers are inferred. The child-ignore exception is conditional only: accept the
exact `test_child_fixture_waits_release` ignored source/reason only when source-backed and all three named
active child tests passed; absent child support is not required, and an unsupported claimed ignored row is
BLOCKED. The three manual-profile ignores remain source-backed only.

### Actual final integration RED and GREEN logs

Before implementation, P2 source/manifest regressions recorded:

```text
Ran 5 tests in 0.027s
FAILED (failures=4)
test_p2_metadata_source_fails: BLOCKED != FAIL
test_p2_metadata_manifest_fails: BLOCKED != FAIL
test_p2_artifact_source_fails: BLOCKED != FAIL
test_p2_artifact_manifest_fails: BLOCKED != FAIL
test_p2_missing_source_blocks: ok
```

The initial runner adapter RED was `Ran 3 tests in 0.241s` / `FAILED (failures=1, errors=2)`: wrong
identity raised `KeyError: 'validation_error'`; intake priority raised
`AttributeError: module 'sop_qualification_runner' has no attribute 'merge_s3_intake_error'`; external
source path reported `RunnerPreconditionBlockedError not raised`. The wrong-identity regression was then
extended through `serde` S3 pre-mapper stop.

```text
Focused helper GREEN: Ran 5 tests in 0.027s / OK
Focused runner GREEN: Ran 4 tests in 0.235s / OK

Helper suite: Ran 70 tests in 0.290s / OK
Runner suite: Ran 136 tests in 2.408s / OK
Process-tree suite: Ran 6 tests in 0.342s / OK
```

### Test traceability and final checkpoint

The complete method-name list for tests previously unreferenced in the journal is appended next; every
test has a `REQ-SOP-RUNNER-*` docstring tag and four underscore-delimited words after `test_`. The final
AST/name/tag/traceability and whitespace command outputs follow that list. No-refactor checkpoint: only
the scoped runner, pure intake helper, tests and this journal changed; no Cargo, native Rust, mapper,
corpus, or production evidence command was run, and no commit was made.

Tests previously absent from this journal, now cross-listed for traceability:

Runner:
`test_s3_identity_bound_harness` `test_s3_intake_failure_priority` `test_s3_identity_failure_stops`
`test_s3_intake_error_priority` `test_s3_native_skip_blocks` `test_workspace_inherited_proc_macro`
`test_workspace_custom_crate_blocks` `test_workspace_external_source_blocks`
`test_s3_component_cannot_promote` `test_s3_fail_precedes_blocked` `test_s3_prerequisite_blocks_mapper`
`test_intake_mutated_bytes_fail` `test_parent_symlink_replacement_fails`
`test_external_member_overlap_blocks` `test_config_appears_pre_freeze` `test_config_appears_post_freeze`
`test_receipt_appears_pre_freeze` `test_intake_fail_over_blocks` `test_frozen_evidence_mutation_fails`
`test_frozen_evidence_deletion_fails` `test_evidence_symlink_retarget_fails`
`test_duplicate_page_stops_collection`

Helper:
`test_json_rejects_duplicate_keys` `test_json_rejects_nonfinite_numbers`
`test_receipt_rejects_boolean_version` `test_receipt_rejects_boolean_elapsed`
`test_bad_reference_shape_blocks` `test_missing_receipt_schema_blocks` `test_missing_receipt_exit_blocks`
`test_nested_receipt_identity_fails` `test_receipt_rejects_timeout_success`
`test_manifest_rejects_duplicate_references` `test_manifest_schema_version_blocks`
`test_valid_bundle_returns_tokens` `test_identity_mismatch_fails_bundle` `test_hash_mismatch_fails_bundle`
`test_missing_log_blocks_bundle` `test_truncated_log_blocks_bundle` `test_fixture_filter_blocks_bundle`
`test_native_skip_blocks_bundle` `test_forbidden_tree_fails_gate` `test_analysis_tree_dependencies_fail`
`test_pinned_parser_tree_passes` `test_release_mismatch_fails_gate` `test_p2_target_override_blocks`
`test_p2_feature_override_blocks` `test_manual_ignore_passes_bundle` `test_unknown_ignore_blocks_fixture`
`test_child_absent_ignore_blocks` `test_child_exception_active_passes` `test_failed_fixture_sets_fail`
`test_nonzero_then_missing_fails` `test_fixture_failure_beats_truncated`
`test_forbidden_tree_lacks_p2` `test_missing_then_nonzero_fails` `test_m2_missing_crosswalk_blocks`
`test_source_macro_ambiguity_blocks` `test_duplicate_receipt_id_fails`
`test_duplicate_receipt_contents_fail` `test_p2_feature_mismatch_fails`
`test_async_source_inventory_matches` `test_detached_test_attribute_blocks`
`test_wrong_runtime_run_fails` `test_wrong_pinned_repository_fails` `test_selected_unit_policy_fails`
`test_command_target_selector_blocks` `test_fixture_section_counts_block` `test_fixture_measured_rows_block`
`test_clippy_warning_stderr_fails` `test_tree_full_rows_block` `test_tree_server_families_fail`
`test_release_missing_artifact_blocks` `test_release_identity_context_blocks`
`test_release_package_locator_fails` `test_release_version_only_pkgid`
`test_timeout_fixture_receipt_blocks` `test_timeout_fixture_failure_fails`
`test_unknown_target_override_blocks` `test_wrong_dispatched_cargo_blocks`
`test_release_binary_source_fails` `test_release_malformed_target_blocks`
`test_release_success_beats_truncated` `test_release_missing_success_blocks`
`test_p2_missing_finish_blocks` `test_p2_failure_beats_truncated`
`test_metadata_nested_malformed_blocks` `test_malformed_receipt_id_blocks`

### Receipt recipe correction — read-only identity preparation

“Prepare the ordinary runner context first” means assemble the expected binding with read-only identity
helpers only: capture the source snapshots, lock/config/toolchain identities and registry, then call
`build_s3_expected_identity` and `build_s3_repository_bindings`. Choose a fresh run ID and calculate the
exact PLANNED runner-owned report/artifact run-directory paths, but exclude those exact paths from the
identity registry; do not create them. Do not invoke `prepare_execution_context` or
`execute_qualification_run` and do not precreate either owned run directory before the receipt manifest is
complete. Retain the expected identity and repository bindings externally, run the fixed bounded receipt
commands using those bindings, retain/hash their raw streams, and assemble the complete manifest. Only
then invoke the runner once with that manifest and the fresh run ID. Never reuse a run ID or owned
directory from a failed or incomplete preparation. No automatic identity freezer or receipt-collection
command is added.

### REQ-SOP-RUNNER-022.R — explicit standard release-profile receipts

Policy review before implementation:

- `docs/phase-20261111/SOP-phase-20261111-hld-current-e2e-test.md` §S3, lines 77–82, requires the
  applicable fixture/build suite, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, production `cargo tree`, and a separate release build of the production binary from
  the oracle harness. It does not prescribe a debug profile for fixture or Clippy commands, or prohibit
  an explicit standard `--release` on either command.
- The same SOP's gate summary at line 392 and `AGENTS.md` lines 64–66 require Clippy, tests, and a
  release build; neither explicitly prohibits release-profile fixture/Clippy receipts. The separate
  production release-build requirement remains in force.

No policy prohibition was found. For `fixture_suite` and `clippy`, exactly one literal Cargo-level
`--release` is accepted only after the command name and before `--`. The receipt's original argv remains
unchanged and is the profile evidence; the validator does not infer or relabel a debug invocation as
release. Workspace/all-target/all-feature coverage, Clippy JSON output and `-D warnings`, and the
unfiltered fixture contract remain exact. Package filters, `--skip`, positional test filters, custom
`--profile`, unknown options, duplicate/misplaced `--release`, and environment identity overrides remain
rejected. P2 command and feature matching was not changed.

Traceability for this subclause (each test has the `REQ-SOP-RUNNER-022` doc tag):
`test_fixture_release_profile_passes`, `test_clippy_release_profile_passes`,
`test_release_profile_misuse_blocks`.

Actual RED, before the command validator change:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_s3_prerequisite_evidence_intake.S3PrerequisiteEvidenceIntakeTests.test_fixture_release_profile_passes test_s3_prerequisite_evidence_intake.S3PrerequisiteEvidenceIntakeTests.test_clippy_release_profile_passes test_s3_prerequisite_evidence_intake.S3PrerequisiteEvidenceIntakeTests.test_release_profile_misuse_blocks
test_fixture_release_profile_passes ... FAIL
test_clippy_release_profile_passes ... FAIL
test_release_profile_misuse_blocks ... ok
AssertionError: 'BLOCKED' != 'PASS' (fixture_suite)
AssertionError: 'BLOCKED' != 'PASS' (build_evidence)
Ran 3 tests in 0.053s
FAILED (failures=2)
```

Actual GREEN after the minimal allowlist change:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v test_s3_prerequisite_evidence_intake.S3PrerequisiteEvidenceIntakeTests.test_fixture_release_profile_passes test_s3_prerequisite_evidence_intake.S3PrerequisiteEvidenceIntakeTests.test_clippy_release_profile_passes test_s3_prerequisite_evidence_intake.S3PrerequisiteEvidenceIntakeTests.test_release_profile_misuse_blocks
test_fixture_release_profile_passes ... ok
test_clippy_release_profile_passes ... ok
test_release_profile_misuse_blocks ... ok
Ran 3 tests in 0.083s
OK
```

The positive tests assert that `--release` is present in the retained argv. The negative matrix covers
package selection, custom profiles, unknown flags, a fixture `--skip`, duplicate or misplaced release
flags, and an environment field not matching the frozen binding. No Cargo command was run for this
implementation.

### Real frozen source-inventory observation

The existing pure `derive_production_source_test_inventory` was run on the real tracked `crates/` source
at base commit `36f4042644285a2c2f26593a06ed4a6882ce74ef`; no compilation or target command was run.
`capture_frozen_source_snapshot` was called before and after inventory derivation with
`project_root / "crates"`, the same tracked source prefix enumerated by the derivation:

```text
source root: /home/ubuntu/repos/parseltongue-rust-LLM-companion/crates
HEAD before/after: 36f4042644285a2c2f26593a06ed4a6882ce74ef
source_tree_sha256 before/after: 456adfe775f377b59d11eb248c923db472f6a3e2ef4c996d99373af6834c3637
tracked source files before/after: 94
snapshot_unchanged: true
source inventory tests: 208
ambiguity status: AMBIGUOUS
```

A repository-root snapshot attempt is unsupported because tracked `.claude/skills` is a directory
symlink and the snapshot helper requires regular file inputs (`frozen input is not a regular file:
/home/ubuntu/repos/parseltongue-rust-LLM-companion/.claude/skills`). The source-only `crates/` snapshot
avoids that unrelated path and covers the exact tracked source prefix read by the inventory derivation.

Ambiguity reasons returned by the existing derivation (no names were inferred or aliased):

- macro-generated test inventory is ambiguous in
  `crates/parseltongue-port-macro-expander/src/macro-expansion-rule-parts/macro-expansion-match-parts/macro-token-transcription-logic.rs`
- macro-generated test inventory is ambiguous in
  `crates/parseltongue-port-macro-expander/src/macro-expansion-rule-parts/rule-match-expansion-logic.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-cli-dispatcher-binary/tests/oracle-comparison-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-cli-dispatcher-binary/tests/stage-export-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-cli-dispatcher-binary/tests/workspace-cli-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-core-row-types/src/lib.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-crate-graph-loader/tests/graph-stage-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-crate-graph-loader/tests/sysroot-context-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-duckdb-analysis-adapter/src/bounded-constructor-policy-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-duckdb-analysis-adapter/src/namespace-binding-round-strata.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-duckdb-analysis-adapter/src/namespace-import-endpoint-rounds.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-duckdb-analysis-adapter/src/namespace-path-candidate-records.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-duckdb-analysis-adapter/src/namespace-scope-relation-records.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-duckdb-analysis-adapter/src/published-generation-copy-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-duckdb-analysis-adapter/src/row-resolution-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-duckdb-analysis-adapter/src/sibling-namespace-accuracy-fixtures.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-port-macro-expander/src/builtin-expansion-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-port-macro-expander/src/declarative-expansion-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-stage-worker-scheduler/tests/worker-stage-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-syntax-span-extractor/src/legacy-macro-scope-records.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-syntax-span-extractor/src/namespace-legacy-source-fixtures.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-syntax-span-extractor/src/syntax-namespace-record-codec.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-syntax-span-extractor/tests/syntax-stage-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-table-store-traits/src/lib.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-turso-store-adapter/tests/stage-store-contract-tests.rs`
- test function could not be inventoried unambiguously in
  `crates/parseltongue-turso-store-adapter/tests/workspace-dependency-build-gates.rs`

Required-name comparison used the existing `M0_TEST_NAMES`, `M1_TEST_NAMES`, and `M2_TEST_NAMES` sets
against the 208 derived source names:

```text
missing required M1 names: test_no_analysis_crates_linked
missing required M2 names (M0 + M1 + M2):
  test_body_macro_sites_match
  test_macro_rules_items_match
  test_nameres_matches_oracle_paths
  test_no_analysis_crates_linked
  test_oracle_disagreements_keep_reasons
missing additional M2 names:
  test_body_macro_sites_match
  test_macro_rules_items_match
  test_nameres_matches_oracle_paths
  test_oracle_disagreements_keep_reasons
```

These missing names remain BLOCKED in this checkout; the observed inventory ambiguity is not waived.
The Python suite budget was `<30s` expected and `120s` maximum per suite; final measured results after the
code change were helper `Ran 73 tests in 0.383s / OK`, runner `Ran 136 tests in 2.532s / OK`, and process
tree `Ran 6 tests in 0.343s / OK`. No-refactor checkpoint: only the helper, helper tests, and this scoped
journal changed in this follow-up; no production Rust source, P2 matching logic, SOP, or AGENTS file was
edited, and no commit or push was made.

### Supplemental test traceability

The final static traceability check also identified earlier runner/helper tests that were not yet
cross-listed in this append-only journal. Their existing REQ-tagged docstrings remain authoritative;
this list records the names without changing prior checkpoints:

Runner:
`test_output_checkout_root_blocks` `test_output_source_ancestor_blocks`
`test_output_source_subtree_blocks` `test_corpus_ancestor_root_blocks`
`test_corpus_root_output_blocks` `test_output_symlink_bypass_blocks`
`test_output_symlink_alias_blocks` `test_output_safe_paths_validate`
`test_outputs_block_before_writes` `test_output_exclusions_detect_mutations`
`test_selected_locks_are_frozen` `test_selected_lock_modification_fails`
`test_selected_lock_deletion_fails` `test_unselected_lock_stays_blocked`
`test_publication_map_accepts_complete` `test_publication_map_requires_stages`
`test_publication_map_requires_generation` `test_static_status_cannot_publish`
`test_unaddressed_sends_require_proof` `test_socketpair_local_send_passes`
`test_annotated_unix_fd_allowed` `test_stale_socket_proof_rejected`
`test_socketpair_inode_fd_untrusted` `test_execveat_relative_path_blocks`
`test_failed_exec_payload_blocks` `test_failed_forbidden_exec_blocks`
`test_mapper_failure_retains_samples` `test_target_failure_skips_status`
`test_runner_persists_skipped_states`

Helper:
`test_p2_metadata_source_fails` `test_p2_metadata_manifest_fails`
`test_p2_artifact_source_fails` `test_p2_artifact_manifest_fails`
`test_p2_missing_source_blocks`

### Authorized resolver-v2 fixture exception — HARNESS ONLY

The only Cargo invocations authorized for this fixture were run in the external, dependency-free,
two-package resolver-v2 workspace at
`/home/ubuntu/parseltongue-resolver-v2-fixture-1794529914/workspace`. Both commands used a 20-second
timeout and completed successfully:

```text
argv: cargo metadata --offline --format-version 1
elapsed_monotonic_seconds=0.037953; timed_out=false; exit_status=0

argv: cargo check --offline --locked --workspace --lib --bins --message-format=json
elapsed_monotonic_seconds=0.157697; timed_out=false; exit_status=0
```

Exact cwd, argv arrays, timeout, stream sizes/hashes, and unmodified raw stdout/stderr are retained in
`/home/ubuntu/parseltongue-resolver-v2-fixture-1794529914/observations/command-records.json` and the
adjacent `metadata.stdout`, `metadata.stderr`, `check.stdout`, and `check.stderr`; the short report is
`feature-resolution-observation.md` in the same directory. Both commands took less than the expected
10 seconds. The `cargo check` build-finished record succeeded.

In `resolve.nodes`, `resolver-v2-shared` has `baseline`, `default`, and `dev-extra`; its emitted lib
artifact has only `baseline` and `default`. The consumer node and its emitted lib/bin artifacts all have
`default`. Metadata lists the consumer lib, default bin, feature-gated bin, and shared lib; the gated
bin was not emitted because its required `gated-bin` feature was disabled. This exact feature mismatch
is recorded without a subset or union shortcut. This fixture is **HARNESS ONLY**, not qualification:
P2 feature matching was not changed and no S3/P2 qualification is claimed. Current P2 remains BLOCKED
pending a parent decision on retaining exact package-level equality or defining future unit-attributed
Cargo evidence.

### Source-inventory absence wording clarification

The listed “missing required” names mean only that the existing pure inventory derivation did not
return those names. Because that inventory reports `AMBIGUOUS`, this does not establish that any of
these tests are absent from Rust source. They remain unproven and BLOCKED; no alias or waiver was used:

```text
M1 derived-name gap: test_no_analysis_crates_linked
M2 derived-name gap: test_body_macro_sites_match, test_macro_rules_items_match,
  test_nameres_matches_oracle_paths, test_no_analysis_crates_linked,
  test_oracle_disagreements_keep_reasons
```

The full-checkout frozen-source limitation also remains: tracked `.claude/skills` is a symlink to a
directory, which the snapshot helper cannot capture as a regular file. The before/after `crates/`
source snapshot covers the exact tracked source prefix read by the inventory; no claim is made that the
full checkout snapshot succeeded.

### Runner-owned dependency-tree cross-check and P2 classification — HARNESS ONLY

The external dependency-audit receipt remains required in schema v1. No receipt schema field, CLI
option, or manifest reference was added. `prepare_s3_evidence_qualification`,
`validate_s3_prerequisite_evidence`, and the dependency-audit evaluator accept an optional internal
reference; absence or an unsupported/serialized value is BLOCKED. The reference is an immutable
in-process capability minted through the runner collection path, bound to the canonical expected
identity digest, fixed argv, frozen project root, and relevant environment. Its private authority
guard protects the manifest boundary, not arbitrary Python code executing inside the process.

Collection is placed after registry, expected identity, and guard creation and before helper
validation, outside the mapper-traced process. It uses the frozen Cargo dispatch path and package
name, and no receipt-supplied argv, flags, or cwd:

```text
cargo tree --package <frozen production package> --edges normal --prefix none --format {p} --offline --locked
cwd: frozen PROJECT_ROOT
```

The collector snapshots the current relevant secret-free environment and checks it against frozen
identity; identity guards run before and after collection and retained stdout/stderr file identities
are checked at later boundaries. Full stdout and stderr are spooled to exclusive files in the owned
artifact run directory, without the event recorder's 16 KiB limit or a row/RAM acceptance cap. The
observation retains argv, cwd, relevant environment, monotonic elapsed time, completion/timeout/exit
state, stream byte counts, SHA-256, and BLAKE3. Timeout is 60 seconds: `cargo tree --offline --locked`
does not compile and is normally a short local resolution, while this cap allows for slow filesystem
or machine contention and still bounds a hung child. On timeout, the collector kills the process
group and waits no more than the additional 5-second cleanup bound, preserving partial streams.

The helper scans forbidden rows in both retained supplied and runner-owned output before
completeness/timeout handling, so an observed forbidden row remains FAIL even when a sibling receipt
is malformed or incomplete. It binds the root name and version exactly to frozen identity and
compares complete successful output as ordered raw rows after CRLF-to-LF normalization only. It does
not trim, sort, deduplicate, or discard blank rows. Valid repeated rows pass only when present in the
same reference output. Missing, unsupported, stale, incomplete, or timed-out current reference is
BLOCKED; a completed nonzero reference command is FAIL; valid wrong root/version or full-row mismatch
is FAIL. A successful internal reference is required for a qualification token.

P2 remains strict: exact metadata/artifact feature equality and target coverage are unchanged; there
is no subset/union acceptance, metadata-as-unit-graph claim, or nightly-policy change. The only P2
change is classification of metadata/artifact feature mismatch as BLOCKED with reason
`P2 metadata/artifact feature mismatch leaves per-unit feature context unproven`. A missing
required-features artifact remains BLOCKED. No Cargo or production qualification command was run for
this implementation. The resolver-v2 mismatch remains an unsupported context, not a qualification.

### RED/GREEN regressions

The initial RED run showed `test_tree_reference_required_blocks` receiving PASS without an internal
reference (expected BLOCKED) and `test_p2_feature_context_blocks` receiving FAIL for unsupported
metadata/artifact context (expected BLOCKED). After implementation, helper and runner regressions
passed. The helper references are synthetic HARNESS ONLY and use an independent baseline, never the
mutated receipt's stdout. Coverage includes root-only and wrong-version output; interior/suffix
omission with rehashed receipts; inserted duplicate/order mismatch; exact matching including actual
duplicate rows; CRLF-only equivalence and other-whitespace mismatch; absent/forged/stale references;
timeout, partial, and completed-nonzero references; forbidden-row precedence over malformed or
incomplete receipts; retained-output mutation; fixed argv despite a malicious receipt flag; skipped
collection for absent/unsupported evidence; and collector-to-adapter-to-helper/token integration
with only the process boundary mocked.

Regression methods: `test_p2_feature_context_blocks`, `test_tree_reference_required_blocks`,
`test_tree_root_only_fails`, `test_tree_root_version_fails`, `test_tree_suffix_omission_fails`,
`test_tree_interior_omission_fails`, `test_tree_order_mismatch_fails`,
`test_tree_duplicate_reference_passes`, `test_tree_crlf_only_passes`,
`test_tree_whitespace_mismatch_fails`, `test_tree_forged_reference_blocks`,
`test_tree_stale_reference_blocks`, `test_tree_timeout_output_blocks`,
`test_tree_nonzero_reference_fails`, `test_tree_partial_forbidden_fails`,
`test_tree_incomplete_forbidden_fails`, `test_tree_reference_mutation_fails`,
`test_dependency_collector_token_harness`, and `test_dependency_tree_timeout_reaps`.

Exact suite results:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_s3_prerequisite_evidence_intake.py'
Ran 89 tests in 0.559s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py'
Ran 138 tests in 2.584s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 tools/process-tree-contract-tests.py
Ran 6 tests in 0.374s
OK
```

The runner collector's subprocess and BLAKE3 process boundaries were mocked in HARNESS ONLY tests;
the exact Cargo command was not invoked. No qualification is claimed from these synthetic tests, and
no commit or push is part of this checkpoint.

### Final static verification

Four Python files parsed with `compile`/AST. The four-word naming, REQ docstring, and journal
traceability check printed:

```text
Python AST syntax: 4 files parsed
test methods=227; non-four-word names=[]; missing REQ tags=[]; missing journal traceability=[]
```

`git diff --check`: exit 0, no output.

### Bounded collector review corrections — HARNESS ONLY

The collector now reads from still-open `x+b` output streams and compares each descriptor's
`fstat` device/inode with the path's `lstat` identity before and after identity capture and hashing.
It compares captured bytes with the path identity before registering the output for later boundary
checks. A detected mutation is persisted in the observation and raised as FAIL, not downgraded to
missing authority. Retained partial stdout is scanned and the forbidden-row finding is persisted
before incomplete authority is discarded; forbidden rows still raise FAIL when hashing fails or
timeout cleanup cannot reap the child. An after-collection identity guard runs from `finally` for
every attempted spawn, including Popen failures. Unreaped output cannot mint an internal reference.

Internal stdout/stderr stream handles retain and validate their `complete` boolean. Incomplete
streams cannot authorize ordered comparison, while retained forbidden rows and exact root
name/version findings remain observable. Root validation precedes parsing suffix rows, so a wrong
root followed by malformed text remains FAIL without a reference. External schema-v1 dependency
receipts preserve the prior optional `--offline`/`--locked` argv acceptance; unknown supplied flags
remain BLOCKED. The runner-owned command remains fixed in code with both flags, independent of
receipt argv. No schema or P2 matching-policy change was made.

RED runs before the collector/helper corrections:

```text
helper focused regressions: Ran 4 tests in 0.025s; FAILED (failures=2)
test_dependency_legacy_receipt_passes: BLOCKED != PASS
test_tree_reference_incomplete_blocks: PASS != BLOCKED

collector focused regressions: Ran 5 tests in 0.226s; FAILED (failures=5)
test_collector_stream_race_fails: mutation was not raised
test_unreaped_timeout_forbidden_fails: forbidden partial row was not raised as FAIL
test_collector_hash_failure_forbidden: forbidden partial row was not raised as FAIL
test_collector_spawn_failure_guards: after guard was not called
```

The first source-mutation test harness also intercepted the registry guard's real Git subprocess
while mocking BLAKE3. It was corrected to mock only the fixed Cargo/BLAKE3 boundaries and delegate
other subprocesses; `test_dependency_source_mutation_fails` then passed using the real registry
guard. The root-order test was corrected to call validation without an auto-generated reference;
with the early root check disabled, the regression produced `BLOCKED` instead of its required
`FAIL` (one test, one failure).

GREEN suites after corrections:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_s3_prerequisite_evidence_intake.py'
Ran 93 tests in 0.510s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py'
Ran 143 tests in 3.117s
OK
```

Coverage added for legacy and unknown supplied argv, incomplete internal streams, root-before-suffix
failure, a genuine swap of distinct interior rows, output path replacement between open-stream read
and identity capture, real-registry source mutation during the mocked child, unreaped timeout plus
forbidden partial output, hash failure plus forbidden partial output, and after-guard on spawn
failure. The subprocess tests are HARNESS ONLY; no Cargo command or qualification was run. No commit
or push was made.

Review regression methods: `test_tree_root_malformed_fails`,
`test_dependency_legacy_receipt_passes`, `test_dependency_unknown_flag_blocks`,
`test_tree_reference_incomplete_blocks`, `test_collector_stream_race_fails`,
`test_dependency_source_mutation_fails`, `test_unreaped_timeout_forbidden_fails`,
`test_collector_hash_failure_forbidden`, and `test_collector_spawn_failure_guards`.

### Resolver-v2 fixture collector verification attempt — BLOCKED before collection

Only the existing `/home/ubuntu/parseltongue-resolver-v2-fixture-1794529914/workspace` fixture was
inspected. Its workspace manifest declares resolver 2 and members `consumer`/`shared`; the frozen
requested package is `resolver-v2-consumer v0.1.0`, with local normal dependency
`resolver-v2-shared v0.1.0`. `dev-extra` is only on the dev-dependency declaration and the gated
bin requires `gated-bin`. Its lock lists those two packages. Cargo dispatch lookup found
`/home/ubuntu/.cargo/bin/cargo`; no Cargo version probe was run, so the exact version/context remains
HARNESS ONLY / unprobed. No `.cargo/config` appeared in the workspace file listing.

The provider discovery command
`command -v b3sum || true; command -v b3 || true; command -v cargo || true; python3 -c 'import importlib.util; print("python_blake3", importlib.util.find_spec("blake3"))'`
printed only `/home/ubuntu/.cargo/bin/cargo` and `python_blake3 None`; the `b3sum` and `b3` lookups
were empty. A search of existing system/user bin locations found no BLAKE3 executable, and Node
reported `node_blake3 unavailable`. The existing runner preflight returned exactly
`RunnerPreconditionBlockedError large artifact BLAKE3 provider is unavailable`. No provider was
installed or substituted. Because this production prerequisite blocks collection, no identity
registry was frozen, no collector or Cargo command ran, no before/after guards ran, and no stream
hashes or output comparison exist. The fixed command that was not executed was:

```text
/home/ubuntu/.cargo/bin/cargo tree --package resolver-v2-consumer --edges normal --prefix none --format {p} --offline --locked
cwd: /home/ubuntu/parseltongue-resolver-v2-fixture-1794529914/workspace
```

The requested 20-second harness timeout was not applied because no invocation occurred; actual
fixture timeout is N/A. Production defaults remain 60 seconds plus a 5-second cleanup bound. No
raw streams or omission comparison were produced. Full details are retained at
`/home/ubuntu/resolver-v2-tree-collector-verification.md`.

Requested follow-up checks:

```text
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile tools/hld-current-qualification-runner.py tools/s3-prerequisite-evidence-intake.py tools/test_sop_qualification_runner.py tools/test_s3_prerequisite_evidence_intake.py
Exit code 0; no output

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 tools/process-tree-contract-tests.py
Ran 6 tests in 0.416s
OK

git diff --check
Exit code 0; no output
```

No production code was changed, no Cargo/production/corpus command ran, and no qualification is
claimed. No commit or push was made.

### Resolver-v2 fixture collector run completed — HARNESS ONLY

After the initial BLAKE3-provider blocker, the explicitly authorized binary-only install succeeded
outside the repo:

```text
python3 -m pip install --only-binary=:all: --target /home/ubuntu/resolver-v2-blake3-wheel-1.0.8 blake3==1.0.8
Successfully installed blake3-1.0.8
```

The runner detected `python-blake3:1.0.8` from that target; the package reported version 1.0.8 and
the known empty-input vector `af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262`.
No synthetic stream hashes were used.

The external HARNESS ONLY driver at `/home/ubuntu/resolver-v2-tree-collector-harness.py` used the
existing resolver-v2 path-only workspace and fixed command twice, with
`DEPENDENCY_TREE_TIMEOUT_SECONDS=20.0` patched in-process (production remains 60 seconds plus 5
seconds cleanup):

```text
PYTHONPATH=/home/ubuntu/resolver-v2-blake3-wheel-1.0.8:/home/ubuntu/repos/parseltongue-rust-LLM-companion/tools PYTHONDONTWRITEBYTECODE=1 python3 /home/ubuntu/resolver-v2-tree-collector-harness.py

/home/ubuntu/.cargo/bin/cargo tree --package resolver-v2-consumer --edges normal --prefix none --format {p} --offline --locked
cwd: /home/ubuntu/parseltongue-resolver-v2-fixture-1794529914/workspace
```

No Cargo version, metadata, check, build, or other Cargo command was run. Exact toolchain version
remains HARNESS ONLY / unprobed. The fixture is not a Git checkout, so the production Git-index
snapshot helper cannot apply. The external harness transparently used an explicit full non-`target`
fixture path inventory with the existing `capture_frozen_file_identity`, standard identity registry,
`verify_identity_registry`, and a path-set guard. It also froze Cargo shim and resolved executable,
Cargo config candidates, runner/helper sources, and the installed BLAKE3 provider files. Guards
before and after both collector calls passed with no mutation.

The collector retained observations and raw stdout/stderr in
`/home/ubuntu/resolver-v2-tree-collector-run-1/` and
`/home/ubuntu/resolver-v2-tree-collector-run-2/`. Both fixed commands completed and were reaped with
exit 0, no timeout, and complete streams. Elapsed times were 0.678797331s and 0.679927683s. Both
produced identical 208-byte stdout:

```text
resolver-v2-consumer v0.1.0 (/home/ubuntu/parseltongue-resolver-v2-fixture-1794529914/workspace/consumer)
resolver-v2-shared v0.1.0 (/home/ubuntu/parseltongue-resolver-v2-fixture-1794529914/workspace/shared)
```

Each stdout SHA-256 was `e9b68ecebfa3a2453f2493095a2545acad2d7743a8042d766b38209b48c1c11d` and BLAKE3
was `a5f13aa41a60e006c8bdad14e3171ad124ff9717172f9aae8c27cd955002e632`. Each stderr was empty
(0 bytes), SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`, BLAKE3
`af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262`.

The helper gate comparing run 1's real stream with run 2's independent reference returned PASS.
Removing the final `resolver-v2-shared` row from run 1 returned FAIL with
`supplied dependency rows differ from runner-owned ordered rows`. This fixture has two rows, so a
suffix omission was testable but there was no interior row. Full report:
`/home/ubuntu/resolver-v2-tree-collector-verification.md`.

No production code changed, no qualification is claimed, and no commit or push was made.

### Dependency-tree command timing excludes guards

The collector now records `elapsed_seconds` only from immediately before `Popen` through wait and
any bounded process-group cleanup. A separate `collection_elapsed_seconds` records the broader
collector duration, including identity guards and stream capture/hashing. The hard child timeout
remains 60 seconds with a bounded 5-second cleanup; command construction, authority, and evidence
policy are unchanged.

The mocked regression `test_collector_slow_guard_timing` uses a 50ms command timeout and 60ms sleeps
in the before/after identity guards. RED on the old implementation recorded
`elapsed_seconds=0.06293139699846506`, exceeding the 50ms limit. GREEN asserts the bounded command
elapsed stays within the timeout, full collection elapsed exceeds it, and the completed reference
still passes the helper gate. The runner suite passed all 144 tests in 3.242s; `py_compile` exited 0.

The fixture's earlier observation values 0.678797331s and 0.679927683s were recorded before this
timing correction and included collection guards/capture/hashing. They remain historical observed
collection durations and are not reinterpreted as subprocess-only command elapsed times. No Cargo
command was run for this fix.

### Dependency-tree timeout kill permission failure

The first run of `test_collector_killpg_error_blocks` was RED: mocked `os.killpg` raised
`PermissionError`, and the handler raised `UnboundLocalError: cannot access local variable 'error'
where it is not associated with a value` instead of recording the command failure.

The catch now binds the exception as `error`. GREEN confirms the 60-second wait times out, the
mocked process-group kill permission failure is recorded, the bounded 5-second cleanup wait still
runs and reaps the child, and the collector returns no dependency reference. Observation remains
timed out with command error; no authority is minted. The full runner suite passed:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py'
Ran 145 tests in 3.161s
OK

PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile tools/hld-current-qualification-runner.py tools/test_sop_qualification_runner.py
Exit code 0; no output
```

No Cargo command or policy change was made.

### Runner-owned nonzero dependency tree acquisition is BLOCKED

The focused RED run showed the old evaluator returned `FAIL` for a runner-owned completed tree
reference with exit 7. The evaluator now classifies that acquisition as `BLOCKED`; supplied external
receipt nonzero exits remain `FAIL`. Independent forbidden-row, exact-root, and retained-stream
mutation observations keep `FAIL` precedence. The renamed REQ-linked test verifies a plain nonzero
reference yields no tokens and retains exit 7 plus stderr. A nonzero reference containing a forbidden
row remains `FAIL`, also with no tokens and retained exit/stderr. A separate regression confirms a
nonzero supplied receipt remains `FAIL`; existing root and mutation precedence regressions remain
unchanged.

Focused RED output:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_s3_prerequisite_evidence_intake.py' -k nonzero
Ran 5 tests in 0.030s
AssertionError: 'FAIL' != 'BLOCKED'
FAILED (failures=1)
```

Checks, each run with a 60-second command timeout:

```text
PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_s3_prerequisite_evidence_intake.py' -k nonzero
Ran 5 tests in 0.028s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_s3_prerequisite_evidence_intake.py'
Ran 95 tests in 0.523s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools -p 'test_sop_qualification_runner.py'
Ran 145 tests in 3.229s
OK

PYTHONPATH=tools PYTHONDONTWRITEBYTECODE=1 python3 tools/process-tree-contract-tests.py
Ran 6 tests in 0.341s
OK
```

Budget correction: historical E<30s / 120s-ceiling figures remain historical. Future Python gates
use E=30s with a 60-second ceiling (exact 2x).
