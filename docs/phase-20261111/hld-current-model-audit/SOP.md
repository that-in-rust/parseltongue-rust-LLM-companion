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
pin compatibility. Follow the corpus pin manifest; keep pinned sources
read-only.

The first bounded source scope is the serde 1.0.229 crate-root facade at
commit `aa7913ad0211fef960df68e7812a6211145a888c`: `serde/src/lib.rs` and the
corresponding `serde_core` crate-root declarations/manifests. The source
checkout is verified at that commit and clean. This limited scope is not a
complete serde HLD expectation.

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
