# Comparator Parallel Work Journal

This append-only journal records the scoped path-comparison delivery. Shared SPEC, SOP, PRD, mapper, namespace pipeline, and CLI routing files remain outside this work except the existing comparator facade and its contract fixture.

## 2026-11-12 — Discovery and executable requirements

### Branch, index, and scope

- Repository: `/home/ubuntu/repos/parseltongue-rust-LLM-companion`.
- Branch: `devin/1794501872-comparator-parallel-work`, created directly at the requested base `31397b39deaa098fdcbb9fcf8e527e4de81d6efb`; no rebase, PR, or merge.
- Initial graph index: `codebase-memory-mcp 0.12.1`, project `home-ubuntu-repos-parseltongue-rust-LLM-companion`, generation `2026-11-12T16:46:48Z`, 22,870 nodes / 42,782 edges, ready, 0 unusable files, 6 parse-partial files. Coverage pagination returned 20 + 6 = 26/26 scopes, same generation, complete hash records. The six partials were namespace DuckDB SQL, an archived research markdown, and a failing SQL fixture; comparator source/tests were not among them. The graph located `run_oracle_agreement_comparison`, comparator contract tests, and SOP S4/S5, but did not expose `oracle-edge-agreement-table` as a table node.
- The initial worktree was clean at the requested base. No implementation changes had been made when this requirements checkpoint began.
- Memory policy: correctness first; no interim memory cap or memory-based optimization is applied to the isolated comparator DuckDB connection.

### Existing interface and persisted evidence

- `crates/parseltongue-cli-dispatcher-binary/src/oracle-path-comparison-facade.rs`: `run_oracle_agreement_comparison(source: &dyn StageTableStorePort, primary: &Path, secondary: &Path, agreement: &dyn StageTableStorePort, cancel: &RunCancelFlagValue)` is currently a stub returning `StoreBackendFailureError`.
- `crates/parseltongue-table-store-traits/src/lib.rs`: `StageTableStorePort` provides workspace publication scope, generation-scope preparation/publication, stage row reads/writes, and initialization; the comparator keeps this seam and the caller-provided agreement store.
- `crates/parseltongue-core-row-types/src/stage-pipeline-row-types.rs`: `StageRunScopeValue` is `(run_generation_digest_value, stage_attempt_sequence_value)`; `StageTableRowRecord` carries table name, row key, and JSON payload; `StageUnitWriteRequest` carries scope, stage owner, work key, input stages, and output rows. `RunCancelFlagValue` is shared cooperative cancellation.
- `crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs`: the published semantic unit identity is Cargo package ID (including source), target name/kind, compilation role, and a full BLAKE3 configuration digest over cfg, features, edition, and role. The publication manifest includes both generation and attempt.
- Source evidence is read from the published `crate-dependency-graph-table` and `path-resolution-site-records`; no mapper construction or namespace re-emission is in scope.
- `tools/oracle-semantic-export-harness/src/mapped-oracle-export-facade.rs` validates unique compilation identities, role/target policy, cfg/feature/edition digest, and source closure, then applies mapped-unit configuration by source root, package ID, target name, and target kind. Mapped target units and required sysroot contexts are retained; harness exports select mapped targets. Comparator matching must reject ambiguous unit matches and validate an optional semantic identity record rather than selecting a first candidate.

### Document rules and baseline test coverage

- `docs/phase-20261111/SPEC-phase-20261111-executable-requirements.md` §2.1: site key includes normalized workspace/package-relative path, UTF-8 byte range, compilation-unit key, site kind, and deterministic expansion provenance/invocation. Target keys preserve package source ID, target name/kind, and qualified item path. Exact duplicate occurrences collapse by key; different compilation units do not. PRIMARY is the denominator, SECONDARY-only sites are KL1 observations, unknown oracle evidence blocks, and empty denominators are N/A (not 100%).
- `docs/phase-20261111/SOP-phase-20261111-hld-current-e2e-test.md` §S5 specifies per-site match/mismatch/unresolved/exclusion evidence, unit-context separation, expansion provenance, and duplicate-key canonicalization. It does not define conflicting same-key rows with different target/classification/reason payloads. Existing tests cover exact duplicate collapse only; the comparator will reject conflicting duplicates with a typed error.
- `REQ-RUST-005.1` requires path-site scoring and M2's temporary proc-macro exclusion to be counted separately; M3 removes that exclusion. A wrong extracted target remains a mismatch even if a proc marker is present (covered by the existing test).
- `REQ-RUST-005.2` requires Turso writes and DuckDB analysis, with no third database/runtime RA. `REQ-RUST-021.2` requires exact mapped-unit and PRIMARY/SECONDARY policy, an exhaustive partition, module-scope path scoring (including module-bound primitive names and enum variants), and deferred type-relative sites. `REQ-RUST-024.1` requires a complete published source generation and per-table row-count/content-digest agreement before analysis, without extensions/ATTACH; `REQ-RUST-024.3` requires total ordering and byte-identical single-/multi-thread exports.
- Existing integration tests in `crates/parseltongue-cli-dispatcher-binary/tests/oracle-comparison-contract-tests.rs`: `test_exact_comparison_publishes_turso`, `test_empty_primary_is_not_success`, `test_unresolved_and_wrong_targets_fail`, and `test_scope_and_oracle_errors_block`. They use a dynamic `Leaf`/`Uses(Leaf)` Cargo fixture and an exported path row. Exact duplicate PRIMARY rows collapse to one denominator; empty PRIMARY is N/A and retains one secondary-only row; unresolved sites do not match; wrong target remains a mismatch despite the proc flag; bad scope/config/path and source-as-agreement are errors. The fixture must explicitly tag the known-answer module-scope site; no production fallback classification is permitted.
- Proc-macro limitation: the `site_proc_macro_dependent_flag` currently appears only in the comparator test fixture, not in the production producer/harness. No detector is inferred or added here. The CLI comparator route supplies a fresh default cancel flag; the existing Ctrl-C handler is only in mapping flow. This work preserves the cancellation seam and tests cancellation at the port boundary without changing `main.rs` signal wiring.
- Documentation pin mismatch discovered but not changed: PRD §7 and the SPEC baseline refer to rust-analyzer `2a362a41a8eb6e090f1a12d9b712df85af6f0c33`; SOP revision R1 says the supported patched fork is `bf6b2099d6351c37039b01447a9bb64f9a54a586` with `tools/parser-const-closure-statements.patch` SHA-256 `89146042a1c1629ef67c730c0e180dec15264577f675de1c090dca2d87da6bfa`, superseding old SOP §0 references. Shared docs are not edited in this task.

### Scoped comparator requirements and test traceability

The following comparator-specific requirements supplement, but do not edit, the shared SPEC verification matrix.

| req_id | WHEN | THEN SHALL | linked tests |
| --- | --- | --- | --- |
| REQ-RUST-025.0 | Either oracle artifact is malformed, has incompatible configuration/scope, contains unknown required evidence, or cannot be mapped to exactly one source compilation context | Return a typed comparator error before publishing any agreement generation | `test_invalid_oracle_configuration_fails`, `test_attempt_mismatch_rejected_early`, `test_ambiguous_unit_contexts_fail` |
| REQ-RUST-025.1 | Sites/targets are normalized | Use package-relative lexical paths, full semantic unit identity, ranges, site kind, expansion provenance, canonical target sets, and deterministic namespace ordering; exact duplicates collapse and conflicting same-key payloads error | `test_site_identity_keeps_contexts_distinct`, `test_exact_duplicates_collapse_rows`, `test_conflicting_duplicate_payloads_fail`, `test_expansion_provenance_separates_sites` |
| REQ-RUST-025.2 | PRIMARY rows are classified and compared with production/SECONDARY evidence | Partition every raw PRIMARY row exactly once; score module-scope rows, defer type-relative rows, block unknown/missing/conflicting classification; apply M2 proc exclusions only to otherwise unresolved/missing/matching answers, never to a wrong extracted target; empty denominator is N/A | `test_partition_primary_site_classes`, `test_proc_exclusion_keeps_wrong_target_mismatch`, `test_zero_denominator_is_not_applicable`, `test_module_variants_are_scored` |
| REQ-RUST-025.3 | Evidence has SECONDARY-only, changed-target, or production-only sites | Retain each as a deterministic evidence row without changing the PRIMARY denominator; compute site equality and aggregate metrics in DuckDB SQL full-outer/set comparisons | `test_secondary_orphans_are_retained`, `test_changed_targets_keep_both_evidence`, `test_production_only_rows_are_visible` |
| REQ-RUST-025.4 | Source/oracles are ready for comparison | Pin the frozen published source generation/attempt; verify source row counts and framed BLAKE3 digests before SQL; include source generation/attempt and both oracle digest identities in agreement rows; write then publish one complete deterministic Turso agreement generation; recheck source publication before publish; cancellation/failure leaves no usable partial snapshot | `test_source_unpublished_generation_fails`, `test_source_change_blocks_publication`, `test_comparison_cancellation_prevents_publication`, `test_agreement_generation_is_published` |
| REQ-RUST-025.5 | Equivalent evidence is analyzed with DuckDB thread count 1 or 2 | Produce byte-identical sorted agreement exports and summaries | `test_thread_counts_export_identically` |

Implementation boundary: pure parsing/identity/canonicalization is internal L1/L2-style Rust in comparator-only modules; an isolated DuckDB L3 connection runs classification, full-outer/set comparison, and aggregates; the existing `StageTableStorePort` adapter writes the agreement table through Turso. No new dependency, runtime RA, mapper/namespace changes, or shared-doc changes are allowed. A single registration tuple in `STAGE_OWNED_TABLE_NAMES` is authorized because the current port rejects undeclared tables.

### TDD checkpoints

Each checkpoint below is append-only and records its exact command/output and commit after completion. Required sequence: STUB (`cargo check`), RED (demonstrate missing behavior), GREEN, REFACTOR, then required quality gates. No phase is claimed complete before its check has run.

## TDD checkpoint log

### Follow-up source-root and dependency verification

- The source package-root field is persisted: `crates/parseltongue-crate-graph-loader/src/lib.rs:391` emits both `unit_source_root_path` and `unit_package_root_path`; line 496 consumes the package root, and `sysroot-compilation-context-rows.rs:44-45` emits it for sysroot units. Use the persisted `unit_package_root_path` directly for lexical package-relative normalization; do not derive it from Cargo package IDs.
- The initially missing sibling parser checkout was restored following SPEC §8/SOP R1: clone `https://github.com/rust-lang/rust-analyzer.git` into `/home/ubuntu/repos/rust-tools/rust-analyzer`, fetch `tools/parser-source-fork-bundle.bundle` HEAD, then detach at `bf6b2099d6351c37039b01447a9bb64f9a54a586`. Verified `git rev-parse HEAD` equals that manifest pin, the checkout is clean, and the patch digest is `89146042a1c1629ef67c730c0e180dec15264577f675de1c090dca2d87da6bfa`.

### Traceability naming correction

Keep new `test_` identifiers to exactly four underscore-separated words after the prefix. The earlier planning table is historical and append-only; use these corrected names in RED and subsequent traceability:

```text
REQ-RUST-025.0
  test_invalid_oracle_configuration_fails
  test_attempt_mismatch_rejected_early
  test_ambiguous_unit_contexts_fail
REQ-RUST-025.1
  test_site_context_identity_distinct
  test_exact_duplicates_collapse_rows
  test_conflicting_duplicate_payloads_fail
  test_expansion_provenance_separates_sites
REQ-RUST-025.2
  test_partition_primary_site_classes
  test_proc_wrong_target_mismatch
  test_empty_denominator_is_na
  test_module_variants_are_scored
REQ-RUST-025.3
  test_secondary_orphans_are_retained
  test_changed_targets_keep_evidence
  test_production_only_rows_visible
REQ-RUST-025.4
  test_source_unpublished_generation_fails
  test_source_change_blocks_publication
  test_cancel_prevents_agreement_publish
  test_agreement_generation_is_published
  test_oracle_agreement_registry_roundtrip
REQ-RUST-025.5
  test_thread_counts_export_identically
```

Subsequent traceability stays in vertical per-requirement sections; do not add wide requirement/test tables.

### STUB checkpoint — 2026-11-12

- Requirements: REQ-RUST-025.0–025.5 signatures/adapter boundary only; no behavior is claimed.
- Exact command: `set -o pipefail && cargo check -p parseltongue-cli-dispatcher-binary 2>&1 | tee /home/ubuntu/comparator-work-logs/stub-cargo-check.log`
- Output:
  ```text
  Checking duckdb v1.20000.1
  Checking parseltongue-duckdb-analysis-adapter v0.1.0 (/home/ubuntu/repos/parseltongue-rust-LLM-companion/crates/parseltongue-duckdb-analysis-adapter)
  Checking parseltongue-cli-dispatcher-binary v0.1.0 (/home/ubuntu/repos/parseltongue-rust-LLM-companion/crates/parseltongue-cli-dispatcher-binary)
  Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 16s
  ```
- The initial attempt before restoring the sibling parser checkout failed at manifest loading because `/home/ubuntu/repos/rust-tools/rust-analyzer/crates/parser/Cargo.toml` was absent; no Rust compilation was reached. The prescribed SOP R1/SPEC §8 bundle checkout was then installed at the exact parser pin described above, and this STUB check completed successfully. Full output is retained at `/home/ubuntu/comparator-work-logs/stub-cargo-check.log`.
- Commit: none. Next: add the explicit fixture classification and behavior-focused RED tests.

### RED fixture attempt — not a comparator result

- Exact command: `set -o pipefail && cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests test_exact_comparison_publishes_turso -- --exact --nocapture 2>&1 | tee /home/ubuntu/comparator-work-logs/red-cargo-test.log`
- Output ended with `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out` and the error `DuckDB name-resolution stage not implemented`.
- Reachability check: the panic was at the fixture's `mapped.status.success()` assertion immediately after invoking `map-workspace-full-run`; the comparator CLI command was never reached. This is not valid RED evidence for the comparator. Do not modify mapper/namespace/main paths; replace this helper with a direct persisted Turso source fixture, then rerun the targeted test. Full output is retained at `/home/ubuntu/comparator-work-logs/red-cargo-test.log`.

### RED checkpoint — direct published fixture

- The helper now seeds a minimal published source generation through `StageTableStorePort`, with an explicit known-answer `module-scope` PRIMARY row; it no longer invokes the mapper/namespace pipeline.
- Exact command: `set -o pipefail && cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests test_exact_comparison_publishes_turso -- --exact --nocapture 2>&1 | tee /home/ubuntu/comparator-work-logs/red-fixture-cargo-test.log`
- Output:
  ```text
  running 1 test
  Error: StoreBackendFailureError { error_context_detail_text: "oracle path comparison not implemented" }
  test test_exact_comparison_publishes_turso ... FAILED
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out
  ```
- This failure reaches the comparator adapter after the direct published source fixture succeeds; it is the intended missing-behavior RED. Full output is retained at `/home/ubuntu/comparator-work-logs/red-fixture-cargo-test.log`.
- Environment self-check after the repeated parser-manifest warning: repo blueprint contains no Rust toolchain setup; no repository toolchain override exists; active `rustc`/`cargo` are 1.99.1 stable, matching `oracle-source-build-manifest.json`, and the external parser checkout is the pinned clean commit. The non-fatal warning `unused manifest key `lints.cargo` (may be supported in a future version)` is emitted by the pinned rust-analyzer parser/syntax manifests because that Cargo feature is nightly-only. Switching toolchains would diverge from the recorded stable toolchain, so it is retained as a warning; both STUB compilation and test binary compilation completed.

### Follow-up correctness GREEN checkpoint — 2026-11-12

Historical proposals above are append-only. This section is authoritative for the current implementation and uses only existing test identifiers.

REQ-RUST-025.0
  - `test_macro_failure_blocks_publish`: actual `oracle_proc_macro_rows` failure shape is rejected.
  - `test_malformed_macro_record_blocks_publish`: malformed actual producer record is rejected.
  - `test_scope_oracle_errors_block`: generation and attempt mismatches are rejected before publish.
  - `test_ambiguous_unit_contexts_fail`, `test_invalid_context_kind_rejected`, `test_invalid_reason_enum_rejected`, `test_invalid_proc_flag_rejected`, `test_invalid_namespace_enum_rejected`.

REQ-RUST-025.1
  - `test_package_paths_normalize_lexically`, `test_site_context_identity_distinct`, `test_exact_duplicates_collapse_rows`, `test_conflicting_duplicate_payloads_fail`, `test_expansion_provenance_separates_sites`.
  - `test_target_namespaces_are_canonicalized`, `test_builtin_namespace_changes_identity`, `test_split_target_namespaces_merge`.

REQ-RUST-025.2
  - `test_primary_classes_partition_once`, `test_proc_wrong_target_mismatch`, `test_proc_empty_answer_excluded`, `test_enum_variant_module_scored`, `test_empty_denominator_is_na`, `test_blocked_partition_reconciles_duplicates`.

REQ-RUST-025.3
  - `test_production_only_rows_visible`, `test_extra_production_secondary_counts`, `test_changed_targets_persist_semantically`.

REQ-RUST-025.4
  - `test_source_unpublished_generation_fails`, `test_cancel_prevents_agreement_publish`, `test_oracle_agreement_registry_roundtrip`, `test_exact_comparison_publishes_turso`, `test_scope_oracle_errors_block`.
  - Source-change-during-read and in-flight cancellation delegating-port regressions, plus the combined persisted match/wrong/missing/extra/deferred matrix and repeated-oracle stale-row test, remain outstanding; the pre-cancel test is not evidence for in-flight cancellation.

REQ-RUST-025.5
  - `test_thread_counts_export_identically` currently compares sorted SQL evidence/summary with one and two DuckDB threads. A persisted-export comparison excluding operational attempt diagnostics remains outstanding.

RED evidence:
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests test_macro_failure_blocks_publish -- --exact --nocapture` failed on the assertion that the failed producer row must not publish.
  - `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_normalization -- --nocapture` failed six new regressions for builtin namespace identity/merging and strict context/reason/flag/namespace validation.

GREEN evidence:
  - `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_ -- --nocapture`: `test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out`.
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests -- --nocapture`: `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
- Full command output is retained under `/home/ubuntu/comparator-work-logs/` in `sql-regression-tests.log` and `cli-contract-regression-tests.log`. Commit follows this GREEN checkpoint; refactor/gates are separate.

### Follow-up REFACTOR checkpoint — 2026-11-12

The GREEN checkpoint is committed as `42db82b29` (`fix: validate oracle comparison evidence`). The comparator storage module is now `oracle-path-comparison-storage.rs`; the statistics tuple is a named struct, SQL table identifiers and new Rust identifiers follow the naming rule, and SQL tests follow the storage functions.

REQ-RUST-025.0
  - `test_macro_failure_blocks_publish`, `test_malformed_macro_record_rejected`, `test_scope_oracle_errors_block`.
  - `test_ambiguous_unit_contexts_fail`, `test_invalid_context_kind_rejected`, `test_invalid_reason_enum_rejected`, `test_invalid_proc_flag_rejected`, `test_invalid_namespace_enum_rejected`.

REQ-RUST-025.1
  - `test_package_paths_normalize_lexically`, `test_site_context_identity_distinct`, `test_exact_duplicates_collapse_rows`, `test_conflicting_duplicate_payloads_fail`, `test_expansion_provenance_separates_sites`.
  - `test_target_namespaces_are_canonicalized`, `test_builtin_namespace_changes_identity`, `test_split_target_namespaces_merge`.

REQ-RUST-025.2
  - `test_primary_classes_partition_once`, `test_proc_wrong_target_mismatch`, `test_proc_empty_answer_excluded`, `test_enum_variant_module_scored`, `test_empty_denominator_is_na`, `test_blocked_partition_reconciles_duplicates`.

REQ-RUST-025.3
  - `test_production_only_rows_visible`, `test_extra_production_secondary_counts`, `test_changed_targets_persist_semantically`.

REQ-RUST-025.4
  - `test_persisted_known_answer_matrix` covers persisted match/wrong/missing/extra/deferred evidence, counts, rate, repeated oracle changes without stale rows, and changed target persistence.
  - `test_source_unpublished_generation_fails`, `test_scope_oracle_errors_block`, `test_cancel_prevents_agreement_publish`, `test_oracle_agreement_registry_roundtrip`, `test_exact_comparison_publishes_turso`.
  - Source-change-during-read/before-publish through a delegating port and in-flight cancellation remain unimplemented; the pre-cancel test does not cover them.

REQ-RUST-025.5
  - `test_thread_counts_export_identically` covers one- and two-thread SQL result parity.
  - `test_persisted_known_answer_matrix` compares complete persisted exported rows while excluding publication-attempt diagnostics and checks the semantic digest remains stable across attempts.

Focused checks after refactor:
  - `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_ -- --nocapture`: `test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out`.
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests -- --nocapture`: `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
  - `cargo fmt --all -- --check`: exit 0.
  - `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile` and exit 0.
  - `cargo build --workspace --release`: `Finished release profile` and exit 0.
  - `cargo test --workspace` is blocked by four unchanged `stage-export-contract-tests` cases; each reaches the out-of-scope `namespace-pipeline-publication-facade.rs` stub and reports `DuckDB name-resolution stage not implemented`. These tests do not reach the comparator. Full logs are in `/home/ubuntu/comparator-work-logs/`.

### Final follow-up test and gate update — 2026-11-12

This entry supersedes the earlier REFACTOR checkpoint's explicitly pending publication cases; prior entries remain historical and unchanged.

REQ-RUST-025.0
  - `test_unit_configuration_contracts_fail`, `test_unmapped_site_units_rejected`, `test_site_provenance_entries_validated`, `test_site_evidence_required_values` cover unit cfg/features/edition/role/mapped scope, provenance strings, and required site values.

REQ-RUST-025.4
  - `test_source_changes_after_reads` and `test_source_changes_before_publish` use a delegating port to change reported source scope at the read and pre-publication rechecks and assert the preexisting agreement generation remains published.
  - `test_write_cancellation_preserves_generation` requests cancellation from the delegated agreement write and asserts the existing generation remains published.
  - `test_persisted_known_answer_matrix` covers combined classifications, repeated changed oracle inputs with no stale rows, persisted changed target evidence, and attempt-independent persisted export rows.

Final focused results after the added regressions:
  - Adapter `oracle_path_` tests: `test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out`.
  - CLI comparator contract tests: `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
  - `cargo fmt --all -- --check`: exit 0.
  - `cargo clippy --workspace --all-targets -- -D warnings`: exit 0; output ends `Finished dev profile`.
  - `cargo build --workspace --release`: exit 0, after the last production-source edit; remaining subsequent edits only added tests and journal text.
  - Final `cargo test --workspace` again completes the comparator suite (14 passing) and is then blocked by the same four out-of-scope stage-export tests and namespace pipeline stub described above.

### Authoritative final REQ-to-test mapping

Use only these current identifiers for final traceability; earlier proposed names are historical.

REQ-RUST-025.0
  - `test_macro_failure_blocks_publish`, `test_malformed_macro_record_rejected`, `test_scope_oracle_errors_block`.
  - `test_ambiguous_unit_contexts_fail`, `test_invalid_context_kind_rejected`, `test_invalid_reason_enum_rejected`, `test_invalid_proc_flag_rejected`, `test_invalid_namespace_enum_rejected`.
  - `test_unit_configuration_contracts_fail`, `test_unmapped_site_units_rejected`, `test_site_provenance_entries_validated`, `test_site_evidence_required_values`.

REQ-RUST-025.1
  - `test_package_paths_normalize_lexically`, `test_site_context_identity_distinct`, `test_exact_duplicates_collapse_rows`, `test_conflicting_duplicate_payloads_fail`, `test_expansion_provenance_separates_sites`.
  - `test_target_namespaces_are_canonicalized`, `test_builtin_namespace_changes_identity`, `test_split_target_namespaces_merge`, `test_builtin_target_names_canonicalize`, `test_malformed_target_ranges_fail`.

REQ-RUST-025.2
  - `test_primary_classes_partition_once`, `test_proc_wrong_target_mismatch`, `test_proc_empty_answer_excluded`, `test_enum_variant_module_scored`, `test_empty_denominator_is_na`, `test_blocked_partition_reconciles_duplicates`, `test_unknown_scope_is_blocked`.
  - `test_persisted_known_answer_matrix` asserts persisted match, wrong, missing, extra, and deferred cases together.

REQ-RUST-025.3
  - `test_production_only_rows_visible`, `test_extra_production_secondary_counts`, `test_changed_targets_persist_semantically`, `test_persisted_known_answer_matrix`.

REQ-RUST-025.4
  - `test_source_unpublished_generation_fails`, `test_scope_oracle_errors_block`, `test_source_changes_after_reads`, `test_source_changes_before_publish`, `test_write_cancellation_preserves_generation`, `test_cancel_prevents_agreement_publish`.
  - `test_oracle_agreement_registry_roundtrip`, `test_exact_comparison_publishes_turso`, `test_persisted_known_answer_matrix`.

REQ-RUST-025.5
  - `test_thread_counts_export_identically`, `test_persisted_known_answer_matrix`.
