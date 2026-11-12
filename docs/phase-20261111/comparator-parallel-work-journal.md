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

Refactor commit: `6c2420a64` (`refactor: isolate oracle comparison storage`), following GREEN commit `42db82b29`. No push was made.

### Comparator correction follow-up

Artifact ownership cleanup: commit `e6548c694` removes the three generated `.codebase-memory` files from the repository. The codebase-memory graph and ADR remain external. Observed external index status before this follow-up was full index, 23,022 nodes, 43,739 edges, seven changed comparator paths with `generation_matches=true` and `metadata_match`, and ADR update status `updated` with complete 27-line content.

REQ-RUST-025.1 RED
  - Added `test_role_changes_semantic_digest`, deriving expected digests directly from the producer formula `[cfg, features, edition, role]` and asserting host/target digest distinction.
  - `cargo test -p parseltongue-duckdb-analysis-adapter test_role_changes_semantic_digest -- --nocapture`
  - RED output: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 93 filtered out`; the assertion showed derived `67e2ddbc3bd177b5876c703e99e2bcba1d3b0a668a6004aa39824ce14c4721bb` vs expected `e87c01979f2fba8b139cc3363aa8d4fc0226e0c9f4f46982eee99c805853e3f5`.

REQ-RUST-025.1 GREEN
  - `derive_semantic_unit_identity` and the persisted integration fixture now hash exactly `[cfg, features, edition, role]`; no alternate digest shape is accepted. The formula matches `crates/parseltongue-syntax-span-extractor/src/syntax-namespace-input-extraction.rs:119-129` and `crates/parseltongue-cli-dispatcher-binary/src/published-stage-export-facade.rs:155-158`.
  - `cargo test -p parseltongue-duckdb-analysis-adapter test_role_changes_semantic_digest -- --nocapture`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 93 filtered out`.
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests test_exact_comparison_publishes_turso -- --nocapture`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out`.

REQ-RUST-025.2 RED
  - Added PRIMARY/production/SECONDARY cfg reason regressions, a wrong-target cfg case, unknown-classification cfg case, and a positive PRIMARY-only cfg exclusion case.
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests cfg_ -- --nocapture`
  - RED output: `test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 14 filtered out`. `test_production_cfg_keeps_mismatch`, `test_secondary_cfg_preserves_match`, and `test_cfg_wrong_target_mismatch` observed zero eligible rows instead of one; `test_unknown_cfg_remains_blocked` observed `N/A` instead of `BLOCKED`. The positive PRIMARY-only exclusion test passed.

REQ-RUST-025.2 GREEN
  - The score CASE now prioritizes unknown/deferred classification and nonempty target mismatches before cfg exclusion. Only `primary_reason='cfg_inactive'` can produce `cfg-excluded`; production/SECONDARY reasons remain evidence, and proc-macro empty-answer behavior remains distinct.
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests cfg_ -- --nocapture`: `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out`.

REQ-RUST-024.2 RED
  - Added `test_duckdb_rows_preserve_scope` and `test_wrong_scope_prevents_matching` for generation+attempt fields on copied/canonical/evidence rows and scope-qualified full-outer joins.
  - `cargo test -p parseltongue-duckdb-analysis-adapter scope_ -- --nocapture`
  - RED output: `test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 87 filtered out`; `test_wrong_scope_prevents_matching` failed because `source_scope_generation_value` is absent from `production_oracle_site_rows`.
  - `cargo test -p parseltongue-duckdb-analysis-adapter test_duckdb_rows_preserve_scope -- --nocapture`
  - RED output: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 93 filtered out`; the binder reported `source_scope_generation_value` absent from `oracle_source_copy_rows`.

REQ-RUST-024.2 GREEN
  - The source-copy Appender, production/PRIMARY/SECONDARY canonical rows, and comparison evidence rows now all carry exact source generation and attempt values. Source-copy verification checks total and scoped counts plus per-table framed digests that include generation, attempt, table, key, and payload. Canonical joins, result filters, and row ordering include both scope fields.
  - Requirement: `docs/phase-20261111/SPEC-phase-20261111-executable-requirements.md:773` (every DuckDB row carries its generation key; this comparator also carries source attempt as required by the scoped handoff).
  - Tests: `test_duckdb_rows_preserve_scope` verifies exact fields and scope-framed digest; `test_wrong_scope_prevents_matching` proves a different generation cannot join by site alone.
  - `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_comparison -- --nocapture`: `test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out`.

Follow-up verification
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests -- --nocapture`: `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
  - `cargo fmt --all -- --check`: exit code 0 (after `cargo fmt --all` normalized the touched Rust files).
  - `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.79s`.
  - `cargo build --workspace --release`: `Finished release profile [optimized] target(s) in 6.03s`.
  - `git diff --check`: rerun after this journal update.
  - Workspace tests were not rerun; the previously reported namespace-pipeline stub failures are unchanged and explicitly out of scope.
  - Generated `.codebase-memory` files remain removed by targeted commit `e6548c694`. The graph and ADR remain external; the prior observed external index/ADR status above predates this follow-up and was not refreshed. No generated index artifact is part of the comparator deliverable.

### Proc-macro empty PRIMARY correction

REQ-RUST-025.2 RED
  - Added `test_proc_empty_primary_exclusions` for PRIMARY proc-dependent empty targets with an empty production row and with production missing; `test_proc_wrong_target_mismatch` remains the wrong-target guard.
  - `cargo test -p parseltongue-duckdb-analysis-adapter test_proc_empty_primary_exclusions -- --nocapture`
  - RED output: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 94 filtered out`; the empty-production case reported proc exclusions `0` instead of `1`.

REQ-RUST-025.2 GREEN
  - Restored `WHEN primary_empty AND proc_macro_dependent THEN 'proc-excluded'` after the nonempty mismatch and PRIMARY cfg branches, before generic unresolved handling. This covers empty and missing production while preserving wrong-target mismatch precedence.
  - `cargo test -p parseltongue-duckdb-analysis-adapter test_proc_ -- --nocapture`: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 92 filtered out`; this includes empty-production, missing-production, and proc wrong-target cases.

Proc-macro empty PRIMARY follow-up verification
  - `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_comparison -- --nocapture`: `test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out`.
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests -- --nocapture`: `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
  - `cargo fmt --all -- --check`: exit code 0.
  - `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.56s`.
  - `cargo build --workspace --release`: `Finished release profile [optimized] target(s) in 4.42s`.
  - External index refresh used `index_repository --repo-path /home/ubuntu/repos/parseltongue-rust-LLM-companion --mode full --persistence false`: `status=indexed`, `nodes=23033`, `edges=43906`, `expected_nodes=23033`, `expected_edges=43906`, `artifact_present=false`.
  - `index_status --project home-ubuntu-repos-parseltongue-rust-LLM-companion --diagnostics summary --format json`: `status=ready`, matching 23,033 nodes and 43,906 edges, six best-effort parse-partial files, zero parse-unusable files, and doc-links status `ok`; 17 intentionally ignored files were classified by design. The index coverage report is best-effort, not a completeness guarantee.
  - `manage_adr --project home-ubuntu-repos-parseltongue-rust-LLM-companion --mode update` returned `status=updated`; read-back reported `content_complete=true`, `total_lines=29`.
  - Indexing used persistence false; `.codebase-memory` remains absent. The final graph refresh is repeated after this journal update so the external graph covers this record too.

### Comparator requirement ownership correction: REQ-RUST-026

The independent model audit reserves `REQ-RUST-025.0` (audit branch commit `24fc7466`). Comparator requirements are `REQ-RUST-026.0` through `REQ-RUST-026.5`. Active comparator test doc-tags in `oracle-path-comparison-core.rs`, `oracle-path-comparison-storage.rs`, and `oracle-comparison-contract-tests.rs` are updated to the comparator-owned identifiers below. Earlier journal entries and their original 025 identifiers remain unchanged as historical records; this appended matrix is authoritative for current comparator test tags. No shared SPEC was edited.

#### Authoritative comparator requirements

- **REQ-RUST-026.0 — persisted evidence validation:** Validate source scope, mapped unit configuration and context resolution, persisted oracle envelope and required arrays, actual `oracle_proc_macro_rows` producer entries, typed enum/boolean/namespace values, and required evidence strings. Reject malformed/failed macro evidence, ambiguous or unmapped site units, invalid ranges/provenance, and invalid reason or classification values before publication; never fabricate a production classification.
- **REQ-RUST-026.1 — semantic identity and canonicalization:** Match the producer semantic-unit digest exactly as `[cfg, features, edition, role]` with no alternate digest shapes; distinguish host/target roles. Normalize package-relative paths from persisted roots, preserve site context and expansion provenance, canonicalize target kind/namespaces and builtin identity, merge split namespace evidence for identical targets, collapse exact duplicates, and reject conflicting duplicates.
- **REQ-RUST-026.2 — PRIMARY partition and score semantics:** Assign each PRIMARY raw row one explicit partition/reason while reconciling duplicates. Score only known module-scope sites; keep deferred and unknown classifications explicit, with unknown blocked even when a reason exists. Only PRIMARY `cfg_inactive` excludes from the PRIMARY denominator; production and SECONDARY reasons remain evidence. Preserve mismatch precedence over exclusions. Actual proc-macro evidence excludes an empty PRIMARY answer only for empty or absent production, never a wrong nonempty target. An empty denominator reports N/A.
- **REQ-RUST-026.3 — non-PRIMARY evidence visibility:** Keep production-only paths separately visible without changing the PRIMARY denominator. Preserve SECONDARY-only KL1 and same-site changed-target evidence. A production-only site that also appears in SECONDARY remains visible in the production-only count as well as in SECONDARY evidence.
- **REQ-RUST-026.4 — scoped, stable, cancellation-safe publication:** Carry source generation and attempt through copied, canonical, and evidence DuckDB rows; qualify joins, filters, and ordering by both; verify per-table counts and scope-framed digests. Recheck source stability during reads and before publication. Validate/cancel before publication and ensure source changes or in-flight write cancellation preserve the prior agreement generation. Persist to the distinct registered Turso agreement store, never mutating source evidence; repeated oracle changes must not expose stale rows.
- **REQ-RUST-026.5 — deterministic semantic export:** Produce deterministic semantic rows and summaries across supported thread counts, separating operational attempt identity from semantic export identity.

#### Authoritative REQ-RUST-026 test matrix

Each listed active test doc-tag is located under every matching requirement identifier. Tests with duplicate names in different test modules/crates are separately active tests; the shared name is listed once per requirement.

**REQ-RUST-026.0**
- `test_ambiguous_unit_contexts_fail`
- `test_invalid_context_kind_rejected`
- `test_invalid_reason_enum_rejected`
- `test_invalid_proc_flag_rejected`
- `test_invalid_namespace_enum_rejected`
- `test_unit_configuration_contracts_fail`
- `test_unmapped_site_units_rejected`
- `test_site_provenance_entries_validated`
- `test_site_evidence_required_values`
- `test_macro_failure_blocks_publish`
- `test_malformed_macro_record_rejected`
- `test_scope_oracle_errors_block`

**REQ-RUST-026.1**
- `test_site_context_identity_distinct`
- `test_exact_duplicates_collapse_rows`
- `test_conflicting_duplicate_payloads_fail`
- `test_expansion_provenance_separates_sites`
- `test_package_paths_normalize_lexically`
- `test_malformed_target_ranges_fail`
- `test_target_namespaces_are_canonicalized`
- `test_builtin_target_names_canonicalize`
- `test_builtin_namespace_changes_identity`
- `test_split_target_namespaces_merge`
- `test_role_changes_semantic_digest`
- `test_exact_comparison_publishes_turso`

**REQ-RUST-026.2**
- `test_unknown_scope_is_blocked`
- `test_proc_wrong_target_mismatch`
- `test_proc_empty_answer_excluded`
- `test_proc_empty_primary_exclusions` — latest regression covers empty and absent production; wrong-target guard remains `test_proc_wrong_target_mismatch`.
- `test_blocked_partition_reconciles_duplicates`
- `test_enum_variant_module_scored`
- `test_primary_classes_partition_once`
- `test_empty_denominator_is_na`
- `test_unresolved_targets_remain_mismatch`
- `test_persisted_known_answer_matrix`
- `test_production_cfg_keeps_mismatch`
- `test_secondary_cfg_preserves_match`
- `test_cfg_wrong_target_mismatch`
- `test_unknown_cfg_remains_blocked`
- `test_primary_cfg_excludes_scored`

**REQ-RUST-026.3**
- `test_extra_production_secondary_counts`
- `test_production_only_rows_visible`
- `test_changed_targets_persist_semantically`
- `test_persisted_known_answer_matrix`

**REQ-RUST-026.4**
- `test_exact_comparison_publishes_turso`
- `test_persisted_known_answer_matrix`
- `test_scope_oracle_errors_block`
- `test_oracle_agreement_registry_roundtrip`
- `test_cancel_prevents_agreement_publish`
- `test_source_changes_after_reads`
- `test_source_changes_before_publish`
- `test_write_cancellation_preserves_generation`
- `test_source_unpublished_generation_fails`

**REQ-RUST-026.5**
- `test_thread_counts_export_identically`
- `test_persisted_known_answer_matrix`

This correction changes requirement doc-tags and this journal only; it makes no comparator behavior changes. Structural traceability checks and `git diff --check` are recorded with the follow-up commit; no builds or tests are run for a doc-tag-only change.

### REQ-RUST-026.4 R4 DuckDB lifecycle and telemetry correction

REQ-RUST-026.4 RED
  - Added `test_comparison_connection_settings_are_verified` and `test_comparison_telemetry_survives_publication` before implementation changes.
  - `cargo test -p parseltongue-duckdb-analysis-adapter test_comparison_connection_settings_are_verified -- --nocapture`: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 95 filtered out`; both connections read the shared `.tmp` path instead of unique spill directories.
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests test_comparison_telemetry_survives_publication -- --nocapture`: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 19 filtered out`; returned operational identity lacks `memory_limit` settings/telemetry.

REQ-RUST-026.4 GREEN
  - `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_comparison -- --nocapture`: `test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out`.
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests -- --nocapture --test-threads=1`: `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
  - Runtime read-back confirmed `memory_limit`, `temp_directory`, `enable_external_access`, `autoload_known_extensions`, `autoinstall_known_extensions`, `allow_community_extensions`, and `threads`; no memory limit override was added.

### R4 authoritative test mapping update

**REQ-RUST-026.4 — isolated lifecycle, settings, telemetry, and copy evidence**
- `test_comparison_connection_settings_are_verified`
- `test_comparison_context_cleanup_after_sql_error`
- `test_comparison_context_cleanup_after_cancel`
- `test_comparison_telemetry_survives_publication`
- `test_write_cancellation_preserves_generation`
- `test_duckdb_rows_preserve_scope`

**REQ-RUST-026.5 — semantic determinism across supported thread counts**
- `test_thread_counts_export_identically`
- `test_published_comparisons_threads_are_deterministic`

### REQ-RUST-026.2/026.3 PRIMARY oracle-source-blocking correction

REQ-RUST-026.2/026.3 RED
  - Added `test_external_source_missing_blocks_match_rate`, `test_primary_valid_oracle_missing_production`, `test_primary_source_missing_beats_production_cfg`, `test_production_reason_does_not_excuse_wrong_target`, and `test_secondary_reasons_do_not_affect_primary`; updated `test_primary_classes_partition_once` to require distinct oracle-source and unknown-blocked partitions.
  - `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_comparison -- --nocapture`: `test result: FAILED. 38 passed; 4 failed; 0 ignored; 0 measured; 62 filtered out`.
  - The RED failures showed PRIMARY `external_source_missing` scored as mismatch, failed to override production cfg, remained eligible in the class-partition test, and SECONDARY `cfg_inactive` incorrectly became the authoritative reason. The valid-oracle/missing-production and wrong-target regressions already passed, preserving their nonmatch/mismatch behavior.

REQ-RUST-026.2/026.3 GREEN
  - `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_comparison -- --nocapture`: `test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out`.
  - `git diff --check`: exit code 0.
  - The 201-site regression publishes 200 exact matches plus one PRIMARY `external_source_missing`; status and denominator status are BLOCKED, rates are null, both blocked partitions remain distinct, and raw partition totals reconcile. Production-missing remains an eligible zero-match nonmatch; PRIMARY source-missing overrides production cfg; SECONDARY-only cfg/build reasons remain separately recorded without changing PRIMARY authoritative reason.

### PRIMARY source-blocking authoritative test mapping update

**REQ-RUST-026.2 — oracle availability and PRIMARY-visible reason precedence**
- `test_external_source_missing_blocks_match_rate`
- `test_primary_valid_oracle_missing_production`
- `test_primary_source_missing_beats_production_cfg`
- `test_production_reason_does_not_excuse_wrong_target`
- `test_secondary_reasons_do_not_affect_primary`
- `test_primary_classes_partition_once`
- `test_production_cfg_keeps_mismatch`
- `test_secondary_cfg_preserves_match`
- `test_unknown_cfg_remains_blocked`
- `test_cfg_wrong_target_mismatch`

**REQ-RUST-026.3 — disjoint blocked partitions and raw reconciliation**
- `test_external_source_missing_blocks_match_rate`
- `test_primary_classes_partition_once`
- `test_blocked_partition_reconciles_duplicates`

### REQ-RUST-026.0 strict oracle-envelope boolean correction

REQ-RUST-026.0 RED
  - Added `test_oracle_envelope_flags_strict`, covering missing, null, wrong-boolean, and wrong-type values for both boolean fields in PRIMARY and SECONDARY envelopes. Each case requires typed `InvalidEnvelope` and verifies the prior agreement scope and seeded row remain unchanged.
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests test_oracle_envelope_flags_strict -- --nocapture --test-threads=1`: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 20 filtered out`.
  - The first malformed case showed PRIMARY `oracle_build_scripts_flag` could be omitted and comparison still returned `Ok(...)`, published a new agreement generation, and replaced the prior publication.

REQ-RUST-026.0 GREEN
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests test_oracle_envelope_flags_strict -- --nocapture --test-threads=1`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out`.
  - All 16 invalid envelope-field cases now return `InvalidEnvelope` with the expected configuration and field before publication; the existing generation scope and seeded agreement row remain readable after every case.

**REQ-RUST-026.0 — strict envelope and prepublication validation test mapping update**
- `test_oracle_envelope_flags_strict`
- `test_macro_failure_blocks_publish`
- `test_malformed_macro_record_rejected`

### Final focused comparator gate set

- `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_comparison -- --nocapture`: `test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out`.
- `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests -- --nocapture --test-threads=1`: `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
- `cargo fmt --all -- --check`: `Exit code: 0`.
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile [unoptimized + debuginfo] target(s) in 2.14s`; `Exit code: 0`.
- `cargo build --workspace --release`: `Finished release profile [optimized] target(s) in 6.65s`; `Exit code: 0`.
- `git diff --check`: `Exit code: 0`.
- Workspace tests were not rerun; the previously reported stage-export namespace stub remains outside comparator ownership.

REQ-RUST-026.4 REFACTOR
  - Workspace Clippy initially found needless extra references in comparator `Connection` calls and the lifecycle settings test. Removed only those redundant borrows; no scoring behavior or public interface changed. The final workspace Clippy gate passed.

### Final authoritative REQ-RUST-026 requirement/test matrix

**REQ-RUST-026.0**
- `test_ambiguous_unit_contexts_fail`
- `test_invalid_context_kind_rejected`
- `test_invalid_reason_enum_rejected`
- `test_invalid_proc_flag_rejected`
- `test_invalid_namespace_enum_rejected`
- `test_unit_configuration_contracts_fail`
- `test_unmapped_site_units_rejected`
- `test_site_provenance_entries_validated`
- `test_site_evidence_required_values`
- `test_oracle_envelope_flags_strict`
- `test_scope_oracle_errors_block`
- `test_malformed_macro_record_rejected`
- `test_macro_failure_blocks_publish`

**REQ-RUST-026.1**
- `test_site_context_identity_distinct`
- `test_exact_duplicates_collapse_rows`
- `test_conflicting_duplicate_payloads_fail`
- `test_expansion_provenance_separates_sites`
- `test_package_paths_normalize_lexically`
- `test_malformed_target_ranges_fail`
- `test_target_namespaces_are_canonicalized`
- `test_builtin_target_names_canonicalize`
- `test_builtin_namespace_changes_identity`
- `test_split_target_namespaces_merge`
- `test_role_changes_semantic_digest`
- `test_exact_comparison_publishes_turso`

**REQ-RUST-026.2**
- `test_unknown_scope_is_blocked`
- `test_proc_wrong_target_mismatch`
- `test_proc_empty_answer_excluded`
- `test_proc_empty_primary_exclusions`
- `test_blocked_partition_reconciles_duplicates`
- `test_enum_variant_module_scored`
- `test_primary_classes_partition_once`
- `test_external_source_blocked_rate`
- `test_primary_missing_production_nonmatch`
- `test_source_blocked_preserves_cfg`
- `test_production_reason_keeps_mismatch`
- `test_secondary_reason_nonimpact_primary`
- `test_empty_denominator_is_na`
- `test_unresolved_targets_remain_mismatch`
- `test_production_cfg_keeps_mismatch`
- `test_secondary_cfg_preserves_match`
- `test_cfg_wrong_target_mismatch`
- `test_unknown_cfg_remains_blocked`
- `test_primary_cfg_excludes_scored`
- `test_persisted_known_answer_matrix`

**REQ-RUST-026.3**
- `test_extra_production_secondary_counts`
- `test_production_only_rows_visible`
- `test_primary_classes_partition_once`
- `test_external_source_blocked_rate`
- `test_changed_targets_persist_semantically`
- `test_persisted_known_answer_matrix`

**REQ-RUST-026.4**
- `test_published_comparison_is_deterministic`
- `test_comparison_settings_are_verified`
- `test_context_cleanup_sql_error`
- `test_context_cleanup_after_cancel`
- `test_exact_comparison_publishes_turso`
- `test_comparison_telemetry_survives_publication`
- `test_persisted_known_answer_matrix`
- `test_scope_oracle_errors_block`
- `test_oracle_agreement_registry_roundtrip`
- `test_cancel_prevents_agreement_publish`
- `test_source_changes_after_reads`
- `test_source_changes_before_publish`
- `test_write_cancellation_preserves_generation`
- `test_source_unpublished_generation_fails`

**REQ-RUST-026.5**
- `test_published_comparison_is_deterministic`
- `test_thread_counts_export_identically`

REQ-RUST-026.0 REFACTOR
  - Reviewed the strict-envelope boolean check and table-driven regression after GREEN; the presence-plus-exact-boolean expression and typed prepublication error need no further refactor. GREEN implementation and regression commit: `dd94be791b4320e3b89797af26ba0b4acf039015`.

### Review corrections: causal precedence and invocation-local cancellation

REQ-RUST-026.2 RED
  - `test_source_blocked_preserves_cfg` now checks that PRIMARY external-source absence still blocks with a null rate, while the authoritative causal reason follows SOP precedence (`cfg_inactive`) and the explicit oracle-source partition reason remains `external_source_missing`.
  - `cargo test -p parseltongue-duckdb-analysis-adapter test_source_blocked_preserves_cfg -- --nocapture`: `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 103 filtered out`; assertion output was `left: String("external_source_missing")`, `right: "cfg_inactive"`.

REQ-RUST-026.2 GREEN
  - Removed only the PRIMARY external-source special case from authoritative-reason precedence; the blocked status and explicit `oracle-source-blocked` partition/reason remain independent. PRIMARY-visible causal precedence uses PRIMARY plus production only and leaves SECONDARY reasons non-authoritative.
  - `cargo test -p parseltongue-duckdb-analysis-adapter test_source_blocked_preserves_cfg -- --nocapture`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 103 filtered out`.
  - `test_primary_missing_production_nonmatch` covers both absent production and an empty production row carrying `external_source_missing`; with valid PRIMARY evidence both remain eligible nonmatches (`eligible=1`, `blocked=0`, rate `0.0`). `cargo test -p parseltongue-duckdb-analysis-adapter test_primary_missing_production_nonmatch -- --nocapture`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 103 filtered out`.

REQ-RUST-026.4 REVIEW CORRECTION
  - Removed the shared before/after TMPDIR directory-set snapshot from `test_write_cancellation_preserves_generation`; it still checks real write-path cancellation and preservation of the prior agreement generation. `test_context_cleanup_after_cancel` separately observes this invocation's spill path while its live context is cancelled, then verifies cleanup after the context drops. No global TMPDIR mutation or suite serialization is used.

### Final review-correction gates

- `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_comparison -- --nocapture`: `test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out; finished in 4.01s`.
- `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests -- --nocapture --test-threads=1`: `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 32.26s`.
- `cargo fmt --all -- --check`: `Exit code: 0`.
- `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile [unoptimized + debuginfo] target(s) in 10.71s`; `Exit code: 0`.
- `cargo build --workspace --release`: `Finished release profile [optimized] target(s) in 7.36s`; `Exit code: 0`.
- Workspace tests were not rerun; the previously reported namespace-pipeline stub remains outside comparator ownership.

### Default-parallel CLI verification and refactor checkpoint

- Command: `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests -- --nocapture` (default test-harness parallelism; no `--test-threads=1`).
- Output: `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.85s`.
- `test_write_cancellation_preserves_generation` passed with its invocation-wide TMPDIR snapshot removed; `test_context_cleanup_after_cancel` retains the direct live-context cleanup assertion.

REQ-RUST-026.4 REFACTOR
  - No further refactor needed. Default-parallel publication cancellation passed, the prior agreement generation remains protected, and per-invocation spill cleanup is checked by the isolated context guard test. No global TMPDIR mutation or suite serialization was introduced.

### R9 file-backed comparator context

REQ-RUST-026.4 STUB
  - Added the four-word test scaffold `test_database_backing_uses_file` without changing the comparison interface or production path. The regression will read DuckDB's `PRAGMA database_list` result rather than infer backing from configuration.
  - `cargo check -p parseltongue-duckdb-analysis-adapter --tests`: `Finished dev profile [unoptimized + debuginfo] target(s) in 1.38s`; `Exit code: 0`.

REQ-RUST-026.4 RED
  - The regression was made to require a non-null DuckDB-reported main database file, an existing file under this context's owned directory, and cleanup after drop. Current in-memory backing reports NULL for the `file` column.
  - `cargo test -p parseltongue-duckdb-analysis-adapter test_database_backing_uses_file -- --nocapture`: panic `DuckDB main database has no backing file`; `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 104 filtered out`.

REQ-RUST-026.4 GREEN
  - Replaced the in-memory open with `Connection::open_with_flags` for a database file inside the invocation-owned `TempDir`. DuckDB `PRAGMA database_list` read-back is checked against that file and its live existence; backing evidence is added only to `agreement_operational_identity`.
  - Preserved security/spill configuration and effective setting checks, left `memory_limit` at DuckDB's default, and retained connection-before-TempDir field order.
  - `cargo test -p parseltongue-duckdb-analysis-adapter test_database_backing_uses_file -- --nocapture`: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 104 filtered out; finished in 0.12s`.

REQ-RUST-026.4 REFACTOR
  - Isolated DuckDB's actual database identity/path validation in `read_duckdb_database_backing`, with a typed `DuckdbDatabaseBackingMismatch` error. No further refactor was needed; no facade, scoring, source/agreement interface, or shared document changed.
  - The checked-in SOP snapshot lists R1-R8; the parent-provided R9 file-backed requirement also appears in SPEC §REQ-RUST-024.8. Shared SPEC/SOP files were not edited.

REQ-RUST-026.4 R9 test matrix addition
  | Requirement | Test | Evidence |
  | --- | --- | --- |
  | REQ-RUST-026.4 | `test_database_backing_uses_file` | DuckDB-reported file path is non-null, matches operational identity, exists under the owned directory while live, and is removed after drop. |
  | REQ-RUST-026.4 | `test_comparison_settings_are_verified` | Two live contexts have distinct existing database files; security, thread, spill, and default memory settings remain verified. |
  | REQ-RUST-026.4 | `test_context_cleanup_sql_error`, `test_context_cleanup_after_cancel` | The owned database file and directory are removed after SQL error and cancellation. |
  | REQ-RUST-026.4 | `test_comparison_telemetry_survives_publication` | Actual backing metadata is nested under operational identity and survives persisted publication. |
  | REQ-RUST-026.4, REQ-RUST-026.5 | `test_published_comparison_is_deterministic` | Thread-1/thread-2 backing paths differ and are cleaned up while semantic export bytes and digest remain identical. |

REQ-RUST-026.4 R9 final verification
  - `cargo test -p parseltongue-duckdb-analysis-adapter oracle_path_comparison -- --nocapture`: `test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out; finished in 2.88s`.
  - `cargo test -p parseltongue-cli-dispatcher-binary --test oracle-comparison-contract-tests -- --nocapture`: `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.54s` (default test-harness parallelism).
  - `cargo fmt --all -- --check`: no diagnostics; `Exit code: 0`.
  - `cargo clippy --workspace --all-targets -- -D warnings`: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.89s`; `Exit code: 0`.
  - `cargo build --workspace --release`: `Finished release profile [optimized] target(s) in 6.08s`; `Exit code: 0`.
  - Clippy initially rejected `expect()` in the new test (`clippy::expect-used`); the regression now asserts the read-back path is nonempty without `expect`, and the workspace clippy gate passed. The initial fmt check found formatting-only differences, fixed with `cargo fmt --all`, followed by the passing fmt check above.

### REQ-RUST-027.0 — mapped path capability metadata

REQ-RUST-027.0 STUB
  - Retained the existing `build_oracle_source_manifest` API, manifest fields, source pin, and exporter flags. Read-only source inspection confirmed the mapped PRIMARY/SECONDARY path-export and artifact-cache routes; the manifest value alone did not describe that bounded capability accurately.
  - Before assigning the requirement, repository-wide tracked and worktree searches for `REQ-RUST-027` returned no matches.

REQ-RUST-027.0 RED — BLOCKED BEFORE ASSERTION
  - Added the tagged four-word test `test_oracle_capability_reports_paths` first. The focused command did not reach Rust compilation or the assertion because the offline registry lacks a locked rust-analyzer dependency. This is not an actual assertion RED.
  - `cargo metadata --offline --locked --manifest-path tools/oracle-semantic-export-harness/Cargo.toml --format-version 1 > /dev/null`: `Exit code: 101`.
  - `cargo test --offline --locked --manifest-path tools/oracle-semantic-export-harness/Cargo.toml test_oracle_capability_reports_paths -- --nocapture`: `Exit code: 101`; test not executed.
  - Both commands reported:

```text
error: no matching package named `la-arena` found
location searched: crates.io index
required by package `hir v0.0.0 (/home/ubuntu/repos/rust-tools/rust-analyzer/crates/hir)`
    ... which satisfies path dependency `hir` (locked to 0.0.0) of package `oracle-semantic-export-harness v0.1.0 (/home/ubuntu/repos/parseltongue-rust-LLM-companion/tools/oracle-semantic-export-harness)`
note: offline mode (via `--offline`) can sometimes cause surprising resolution failures
help: if this error is too confusing you may wish to retry without `--offline`
```

REQ-RUST-027.0 GREEN IMPLEMENTATION — RUNTIME UNVERIFIED
  - Changed only `oracle_export_support_kind` from `provenance_only` to `mapped_semantic_paths`; updated the existing pinned-manifest assertion without changing the pinned hash. The new assertion remains in place, but no Rust test result is claimed.
  - `cargo fmt --manifest-path tools/oracle-semantic-export-harness/Cargo.toml -- --check`: no diagnostics; `Exit code: 0`.
  - `cargo clippy --offline --locked --manifest-path tools/oracle-semantic-export-harness/Cargo.toml --all-targets -- -D warnings`: `Exit code: 101` before linting, with the same missing-`la-arena` error above. Cargo also emitted this upstream warning for pinned rust-analyzer manifests:

```text
warning: /home/ubuntu/repos/rust-tools/rust-analyzer/crates/hir-expand/Cargo.toml: unused manifest key `lints.cargo` (may be supported in a future version)
```

REQ-RUST-027.0 REFACTOR
  - Source/diff review found no additional refactor needed. The SOP uses the live manifest/cache/export/comparator commands, ordered mapped-export flags, separate stage targets/logs, frozen paired inputs, and one pretty JSON envelope per oracle export. Full S4 families remain explicit requirements; current path capability does not implement them or per-site expansion provenance/proc-macro dependence and does not establish qualification.
  - Source-only structural output: `PASS: REQ-RUST-027.0 was unused in HEAD and is now declared/test-mapped`; `PASS: live cache/export flags, order, pretty-JSON format, and S4/S5 paths match source`; `PASS: new four-word test and unchanged full source pin assertion`; `PASS: only AGENTS, SOP target block, SPEC additions, and harness lib.rs changed so far`; `PASS: scorer, static build manifest, parser/corpus pins unchanged; AGENTS/SPEC history is additions-only`; `SOURCE-ONLY: stale value in HEAD vs mapped_semantic_paths implementation/test expectation; not Rust runtime evidence`.
  - The static `oracle-source-build-manifest.json`, executable digest, source pins, lockfiles, scoring, resolver, and provenance implementation were not changed. No corpus or workspace build was run. A lock-compatible offline Cargo registry containing `la-arena` is required to execute the actual RED/GREEN contract test and clippy gate.

### S4 fail-fast and build-budget review correction

- Added `set -e` as the first statement in the S4 shell block so a failed manifest/cache command, paired export, or frozen-input checksum stops before comparison.
- Prelaunch build budget: estimated 10 minutes expected, hard maximum 20 minutes per compilation invocation. The metadata, focused test, and clippy commands in this run stopped at dependency resolution; no compilation or build was launched. The execution tool's 4m40 timeout clamp is a tool wait limit, not evidence that a 20-minute build limit was enforced.
- Use an explicit shell `timeout 1200s` for any future compilation invocation. No command was run under that timeout for this documentation-only follow-up; no dependency fetch, corpus run, or other gate was repeated.

### REQ-RUST-027.0 — post-handoff runtime validation

Dependency preparation (outside qualification)
  - Ran the expressly authorized network preparation command `cargo fetch --locked --manifest-path tools/oracle-semantic-export-harness/Cargo.toml`, wrapped by `/usr/bin/timeout 1200s` and `/usr/bin/time`. Result: `elapsed_seconds=0.56 exit_code=0`; output included `Updating crates.io index`, `Downloading crates ...`, and `Downloaded la-arena v0.3.1`. Cargo emitted the known upstream `lints.cargo` manifest warnings; this fetch did not compile or qualify the harness.
  - Before fetch and after fetch, HEAD was `f1e61976064008f1b001862b3a92cfa23242d8f4`, branch was `devin/1794501872-comparator-parallel-work`, and `git status --porcelain=v1` was empty. The following SHA-256 digests were identical before fetch, after fetch, and after all gates:

```text
07792bcb211b2d137af8012e6ad110a6b8087d7868ca136db545c9596dff8e45  Cargo.lock
289cebd2b9d53b1030f32f0d91a701f9df799acd6c6412df410cb217df666e1e  tools/oracle-semantic-export-harness/Cargo.lock
e2c96a147e2096544fd4ef8d23477bd435a5c2aa972b9a226d7dbb20847d65c5  docs/phase-20261111/parser-source-pin-manifest.json
4eed63352738951e1f3ac5de03769ae184409c6e8d7ae470948c4ff0ec98fc03  docs/phase-20261111/corpus-source-pin-manifest.json
64444e2abaaecd1a9688af595c70d7ad51e28686e6ae9446f0e6f561e9497781  docs/phase-20261111/oracle-source-build-manifest.json
```

REQ-RUST-027.0 STUB / RED setup
  - Kept the `build_oracle_source_manifest` API and tagged `test_oracle_capability_reports_paths` from `f1e61976064008f1b001862b3a92cfa23242d8f4`. Created `/home/ubuntu/repos/oracle-capability-red-worktree` detached at that exact commit; its `../../../rust-tools/rust-analyzer` resolves to `/home/ubuntu/repos/rust-tools/rust-analyzer`.
  - In that isolated worktree only, reverted the production `oracle_export_support_kind` literal to `provenance_only`; retained the new test's expected `mapped_semantic_paths` value. The verified RED worktree diff was one line in `tools/oracle-semantic-export-harness/src/lib.rs`. The pinned rust-analyzer checkout remained clean.
  - Reused the existing shared `/home/ubuntu/repos/parseltongue-rust-LLM-companion/target` (`CARGO_TARGET_DIR`); no competing Cargo/rustc process was present when selected. RED, GREEN, all-tests, clippy, and release-build invocations each used `/usr/bin/timeout 1200s` and `/usr/bin/time`. No memory limit or build tuning was added.

REQ-RUST-027.0 RED — actual post-handoff assertion failure
  - Command: `cargo test --offline --locked --manifest-path tools/oracle-semantic-export-harness/Cargo.toml test_oracle_capability_reports_paths -- --nocapture` in the isolated RED worktree.
  - Actual runtime output:

```text
assertion `left == right` failed
  left: String("provenance_only")
 right: "mapped_semantic_paths"
test oracle_manifest_contract_tests::test_oracle_capability_reports_paths ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 33 filtered out; finished in 0.00s
error: test failed, to rerun pass `--lib`
elapsed_seconds=60.92 exit_code=101
```

REQ-RUST-027.0 GREEN — unchanged `f1e6197` source
  - Focused command: `cargo test --offline --locked --manifest-path tools/oracle-semantic-export-harness/Cargo.toml test_oracle_capability_reports_paths -- --nocapture`.
  - Output: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out; finished in 0.00s`; `/usr/bin/time`: `elapsed_seconds=3.89 exit_code=0`.
  - All harness tests: `cargo test --offline --locked --manifest-path tools/oracle-semantic-export-harness/Cargo.toml`.
  - Output: `test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 157.61s`; main binary and doc-test targets each reported zero tests and success. `/usr/bin/time`: `elapsed_seconds=158.13 exit_code=0`.

REQ-RUST-027.0 final harness gates
  - `cargo fmt --manifest-path tools/oracle-semantic-export-harness/Cargo.toml -- --check`: no diagnostics; exit 0; `/usr/bin/time` `elapsed_seconds=0.12`.
  - `cargo clippy --offline --locked --manifest-path tools/oracle-semantic-export-harness/Cargo.toml --all-targets -- -D warnings`: `Finished dev profile [unoptimized + debuginfo] target(s) in 32.72s`; exit 0; `/usr/bin/time` `elapsed_seconds=32.79`.
  - `cargo build --offline --locked --release --manifest-path tools/oracle-semantic-export-harness/Cargo.toml`: `Finished release profile [optimized] target(s) in 1m 29s`; exit 0; `/usr/bin/time` `elapsed_seconds=89.46`.
  - Every compilation invocation completed below the enforced 1200-second shell limit. Cargo emitted this repeated upstream warning:

```text
warning: /home/ubuntu/repos/rust-tools/rust-analyzer/crates/hir-expand/Cargo.toml: unused manifest key `lints.cargo` (may be supported in a future version)
```

    The active toolchain was `stable-x86_64-unknown-linux-gnu`, Cargo/rustc 1.99.1; the pinned rust-analyzer checkout remained clean. No toolchain or pinned checkout change was made.

REQ-RUST-027.0 REFACTOR / scope checkpoint
  - No further refactor was needed; no production source, scoring, resolver, provenance behavior, pins, lockfiles, static executable manifest, or digest changed. The only main-worktree change for this validation is this append-only journal evidence.
  - This validates the harness manifest capability contract only. No production mapper, serde corpus/export, final executable freeze, or S4/S5 qualification run was performed. Complete stdout/stderr and timing files are retained at `/home/ubuntu/oracle-capability-validation-f1e6197`.
