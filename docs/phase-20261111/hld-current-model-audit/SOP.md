# SOP — independent source-grounded audit of HLD current

This SOP supplements, but never replaces or weakens, the phase-20261111
production SOP. It audits **current production HLD rows**, not HLD future,
human intent, or the production M8 builder. The audit is optional evidence;
PRIMARY and the existing S8/S9 rules remain the acceptance authority.

## Inputs and per-repository boundary

Use one pinned corpus repository per audit record. Record source/lock/config
digests, compilation target/features, tool/executable hashes, code-graph
version and index coverage, zcode version, requested model and actual reported
model identity (or `null`), prompt digest, seeds and output hashes. Never pool
the five repository denominators. Keep the production export's content SHA-256
separate from `run_generation_digest_value`, compilation identity and source-
pin compatibility. Follow the Phase 1 or Phase 2 pin manifest as applicable; keep pinned sources
read-only.

## Phase 1 benchmark selection — 2026-11-13

The active first benchmark is now **itoa 1.0.18**, commit
`af77385d0daf4d0e949e81f2588be2e44f69f086`, as pinned in
[phase-one-benchmark-pin.json](../phase-one-benchmark-pin.json).
Follow [production SOP §1.1](../SOP-phase-20261111-hld-current-e2e-test.md#11-phase-1--one-small-repeatedly-tested-hld-current-benchmark):
both complete library source files (488 lines), lib target, empty features,
test cfg OFF, 64-bit Linux target, all active declarative expansions and private
evidence chains. Keep canonical external `core` targets, without treating the
whole sysroot as benchmark-local source. Tests/benches/examples/fuzz and the
optional `no-panic` proc macro are excluded by target/configuration, not by
observed mapper failures.

Build source-only Pass A before the production HLD exporter is ready; the absence
of that exporter blocks Pass B, not source reading or reference drafting.
Do not reuse the serde draft as itoa expectations. A model may assist, but full
scope/evidence/identity verification and the frozen reference are required for
Phase 1 acceptance whether or not an optional model reviewer runs. Preserve raw
model output separately. Compare all expected/actual semantic rows, not just an
S8 sample; unresolved reference claims cannot become an accepted empty graph.
This selection does not authorize a new zcode/agent run by itself.

### Historical serde pilot

The earlier bounded source scope was the serde 1.0.229 crate-root facade at
commit `aa7913ad0211fef960df68e7812a6211145a888c`: `serde/src/lib.rs` and the
corresponding `serde_core` crate-root declarations/manifests. The source
checkout is verified at that commit and clean. This limited scope is not a
complete serde HLD expectation. Retain its draft and provenance as historical
evidence. Phase 2 still uses the original five corpus repositories.

## Pass A — independent source derivation

1. Freeze the repository/target/configuration identity and define the scope
   before inspecting any production HLD export.
2. Derive candidate nodes/dependencies from source and configuration only.
   For each claim record source path, commit, full-file SHA-256, exact byte and
   line range, excerpt SHA-256, canonical compilation/occurrence keys and a
   concise rationale. Code-graph is discovery only, never the claim's evidence.
3. Record source evidence with zero-based half-open byte ranges, one-based
   inclusive line ranges, full-file and excerpt SHA-256 digests. Record all
   examined scopes, excluded/unknown scopes, unresolved claims,
   denominator definition, and whether coverage is exhaustive. Model-only
   guesses stay unknown; do not silently turn absence of evidence into an edge
   or a negative fact.
4. Canonically sort/serialize the expected artifact. Hash its exact UTF-8
   bytes using SHA-256. Record digest/time and freeze it before Pass B. Any
   source/config/tool change invalidates the draft and requires a new freeze.

## Pass B — frozen comparison

1. Obtain the production `hld-current-dependency-edges`, node/reachability
   facts, evidence references and file/folder roll-ups from the **same** pinned
   generation and compilation context. Hash every input/export. If no HLD
   current export exists, record BLOCKED/NOT STARTED and stop: empty output is
   not a successful audit.
2. Compare frozen source expectations with actual rows. Separate false/extra
   and missing edges. Check selected nodes, effective visibility, reexports,
   canonical external identity, resolved private chains, cycles/diamonds/
   genuine self-edges, complete provenance, unique evidence counts and exact
   roll-ups over deduplicated HLD rows.
3. Keep model disagreement as a candidate. Confirm a false edge against source
   and the applicable deterministic evidence before FAIL; unresolved disputes
   BLOCK the scoped audit. Model agreement alone never produces PASS.
4. Record per-repository counts, denominator, exclusions, unknowns, sample
   limitations, expected/actual hashes, all candidate findings, and the
   authoritative PRIMARY/S8/S9 disposition using
   `hld-audit-record-schema.json`.

## Sampling and acceptance

The existing S8 sample is unchanged: min(50, published HLD rows), without
replacement, default seed 42; rank canonical UTF-8 keys by BLAKE3-256 of the
u64 little-endian seed, key byte length (u64 little-endian), then key bytes;
smallest hashes win and row key breaks ties. Record overridden seed and keys.
Supplement it with source-selected expectations to expose missing rows that
output-only sampling cannot see. Report each sampling denominator and
limitation. S9 node/reexport/collapse checks remain unchanged. No model sample
substitutes for S8/S9; no model-derived rate changes a PRIMARY denominator.

Audit disposition is `DRAFT_ONLY`, `REVIEW_REQUIRED`, `BLOCKED`, or
`SOURCE_CONFIRMED_FALSE_EDGE`; only the regular SOP runner records acceptance
PASS/FAIL. Complete golden coverage may be claimed only after every declared
scope and denominator is exhaustively and independently verified.

## Planned fixture gates (PENDING)

See `hld-audit-fixture-expectations.md` and REQ-RUST-025.0. Expected fixtures
cover source-only Pass A, frozen-digest mismatch, false/missing edges,
unresolved evidence, effective visibility/reexports, cycles/diamonds/self-
edges, roll-up deduplication and missing production export. These are planned,
not observed tests. Follow the scoped `AGENTS.md` TDD journal loop before any
validator/runner implementation.

## Revision — optional independent reviewer at S8/S9 (2026-11-12)

An independent zcode review may provide source-grounded candidate evidence as
a supplement to S8/S9. It SHALL NOT change existing thresholds, sampling,
PRIMARY denominators, or per-repository verdicts. Record source/config/tool/
executable/prompt/model identities and hashes; model identity is the actual
reported value or `null`. A model-confirmed claim still requires deterministic
source/PRIMARY adjudication. A missing HLD export or unresolved dispute is
BLOCKED/NOT STARTED, never a model PASS.

## Serde configuration bundle

For the bounded serde pilot, `source_pin.config_sha256` is SHA-256 of the
separate immutable audit configuration bundle, not a per-unit BLAKE3 digest or
`RunGenerationDigestValue`. Canonical bytes are UTF-8 JSON with recursively
lexicographic object keys, no insignificant whitespace, `ensure_ascii=false`,
and one final LF. Sort unit arrays by the source `CrateCompilationUnitKey`
field order; sort unordered cfg and feature arrays lexicographically.

The bundle records the pinned commit and Cargo.lock SHA-256, resolver, exact
`rustc -vV` and `cargo -V`, host/target policy, and the selected per-unit key
and configuration fields. Per-unit `unit_configuration_digest_full` remains
BLAKE3 of the compact serde JSON tuple `[unit_target_cfg_values,
unit_enabled_feature_values, unit_rust_edition_value,
unit_compilation_role_kind]`. Collect metadata only with
`cargo metadata --offline --locked --no-deps` and `rustc --print cfg`; do not
run build scripts or read target outputs.

This pilot derives only the normal default-feature `serde` lib to `serde_core`
lib closure: `serde` enables `std`; its normal `serde_core` dependency enables
`result`, and `serde/std` enables `serde_core/std`. Tests, dev dependencies,
examples, build-script targets, `serde_derive`, `serde_derive_internals`,
`test_suite`, and other workspace units are excluded. The bundle does not
establish compiler-dependent build-script or macro expansion output and does
not create occurrence keys. Keep candidate rows empty until each source
occurrence and its provenance are independently proved.
