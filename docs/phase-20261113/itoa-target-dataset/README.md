# itoa reference table bundle

This is an offline candidate reference authored by Devin from pinned itoa source and classified as `LLM_PARSED_DRAFT`. No independent compiler/oracle verification has frozen it. It is not production input or canonical HLD qualification. It preserves the current candidate row set beside a distinct local-only eligibility subset and blocker classifications. It does not cover the whole source universe: operators, body types, literals and repeated same-line sites are known omissions.

Candidate labels remain symbolic and noncanonical; the local-only subset is a decided diagnostic subset, not the complete target. Selection follows effective item interface visibility, distinct from outside-crate reachability: public `private::Sealed` and its public trait members remain selected despite their private ancestor module.

The five expected HLD outputs use the current rule 10 collapse set: `calls`, `uses_type`, `implements`, `reexports`, `derives`, and `reads`. `contains` and `imports` are excluded at every collapsed hop. OD-9 remains an alternate proposal and does not suppress these outputs; applicable OD-2/OD-7/OD-8 assumptions remain blocked. The current eligible subset yields no collapsed rows; this is its candidate result, not an OD-9 filter or a completeness claim. The focused fixture test exercises collapse for all six kinds.

File/folder roll-ups are partial diagnostics: edges without known source/target file mappings are listed in `excluded_rollup_edges` in both outputs, rather than assigned an invented bucket. Builtin targets use `BUILTIN_BUCKET_MAPPING_NOT_IMPLEMENTED`. These roll-ups exclude those edges pending mapping, and their status is `BLOCKED` whenever exclusions remain.

## Regenerate

From the repository root, with the pinned itoa checkout available:

```sh
python3 docs/phase-20261113/itoa-target-dataset/itoa-target-dataset-builder.py \
  --pin-file docs/phase-20261111/phase-one-benchmark-pin.json \
  --source-root /path/to/rust-tools/itoa \
  --output-directory docs/phase-20261113/itoa-target-dataset
python3 -m unittest discover -s docs/phase-20261113/itoa-target-dataset -p 'test_*.py' -v
python3 docs/phase-20261113/itoa-target-dataset/itoa-backward-table-ladder.py \
  --manifest-file docs/phase-20261113/itoa-target-dataset/itoa-table-contract-manifest.json \
  --export-directory /path/to/exported-table-rows \
  --source-root /path/to/rust-tools/itoa \
  --pin-file docs/phase-20261111/phase-one-benchmark-pin.json \
  --output-directory /path/to/report-output
```

The verifier reads JSONL exports and source bytes only; it never opens or writes the baseline database and never mutates the static manifest. Overall status remains `BLOCKED` until canonical mapping and complete HLD comparison are implemented.

The review ZIP is regenerated from the repository root after the tests and builder complete:

```sh
zip -FS -r /home/ubuntu/hld-current-test-artifacts/itoa-reference-review-20261114.zip \
  AGENTS.md \
  docs/phase-20261113/itoa-target-dataset \
  docs/phase-20261113/PRD-itoa-full-item-graph-scope.md \
  docs/phase-20261113/SPEC-phase-20261113-executable-requirements.md \
  docs/phase-20261113/ARCH-phase-20261113-pipeline-graph-tables.md \
  docs/phase-20261113/SOP-itoa-full-item-graph-acceptance.md \
  docs/phase-20261111/itoa-hld-current-benchmark/itoa-hld-backward-design.md \
  docs/phase-20261111/TDD-phase-20261111-progress-checkpoints.md \
  docs/phase-20261111/phase-one-benchmark-pin.json \
  crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs \
  crates/parseltongue-core-row-types/src/stage-pipeline-row-types.rs \
  crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs \
  crates/parseltongue-duckdb-analysis-adapter/src/hld-materialization-duckdb-store.rs \
  crates/parseltongue-core-row-types/src/name-resolution-stage-records.rs \
  crates/parseltongue-syntax-span-extractor/src/syntax-namespace-input-persistence.rs \
  crates/parseltongue-syntax-span-extractor/src/syntax-namespace-record-codec.rs \
  -x '*/__pycache__/*' '*.pyc'
```
