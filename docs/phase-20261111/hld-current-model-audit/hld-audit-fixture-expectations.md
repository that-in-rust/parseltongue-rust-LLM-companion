# Fixture expectations — source-grounded HLD-current audit

Every item below is a planned validator fixture, not an existing test. Its
REQ-RUST-025.0 status is **PENDING** until implemented and run.

| test_id | fixture input | required observation | status |
| --- | --- | --- | --- |
| `test_pass_a_source_only` | source/config facts plus a production export path | Pass A ignores production rows and every candidate cites pinned line evidence | PENDING |
| `test_frozen_artifact_digest_integrity` | frozen expected artifact, then one changed byte | Pass B is rejected before comparison | PENDING |
| `test_false_edge_has_evidence` | one extra actual edge absent from fully inspected source scope | source-confirmed false-edge key fails the scoped audit | PENDING |
| `test_missing_edge_source_first` | a source-grounded candidate omitted from actual rows | source-selected sample finds the missing key | PENDING |
| `test_unresolved_dispute_blocks_audit` | contradictory or insufficient source evidence | BLOCKED with reason; no guessed edge and no PASS | PENDING |
| `test_source_digest_mismatch_blocks` | source/config digest differs after Pass A freeze | BLOCKED before Pass B | PENDING |
| `test_cycle_no_fake_self` | private cycle with no direct self-edge | traversal terminates; no cycle-invented self-edge | PENDING |
| `test_diamond_dedup_keeps_paths` | two resolved private paths to the same selected target | one HLD key, all distinct evidence paths retained | PENDING |
| `test_rollup_counts_unique_rows` | duplicate paths plus a distinct HLD edge | file/folder counts equal deduplicated HLD rows | PENDING |
| `test_missing_export_blocks_audit` | no matching-generation HLD current export | BLOCKED/NOT STARTED; zero comparisons is not success | PENDING |
| `test_s8_sample_algorithm_unchanged` | fixed product keys and recorded seed | exact existing S8 algorithm/keys remain unchanged | PENDING |
| `test_config_bundle_digest_contract` | canonical immutable configuration bundle and exact per-unit mapper inputs | `source_pin.config_sha256` equals bundle SHA-256; both per-unit BLAKE3 values use the exact loader tuple | PENDING |

Additional source fixtures must include positive/negative effective visibility,
body-item exclusion, private parents, aliases and globs; all expectations need
source byte/line evidence. Test disagreements become deterministic fixtures
only after source-grounded adjudication, before any production change.

## Validator responsibilities beyond JSON Schema (PENDING)

The schema checks record shape and selected cross-field gates. A future
deterministic validator must also check the following; none is implemented:

- Byte offsets are zero-based and half-open; `0 <= byte_start <= byte_end <=`
  source-file byte length. One-based inclusive line ranges are ordered and
  agree with the exact source bytes, file SHA-256 and excerpt SHA-256.
- Evidence IDs are unique. Every node, edge and finding reference resolves to
  one evidence record; compilation and occurrence keys agree with source,
  target and generation identity.
- Recomputed canonical Pass A bytes equal the frozen expected digest; freeze
  precedes any Pass B export read. Export content SHA-256, generation key,
  compatibility facts and compilation identity are independently checked and
  never substituted for one another.
- All compared rows belong to one compatible generation. Edge/evidence keys,
  cycle paths, diamond provenance and deduplicated file/folder roll-ups match
  deterministic source/PRIMARY definitions.
- S8's recorded seed, existing algorithm and keys equal the unchanged SOP
  output-selected sample; source-selected supplements are separately seeded
  and cannot replace S8 or alter its denominator.
- Final repository PASS requires a matching production export, frozen source
  expectation, compatible generation, and PRIMARY/S8/S9 PASS. Schema shape
  alone cannot establish these semantic facts.
