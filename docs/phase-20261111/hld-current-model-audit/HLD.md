# HLD — source-grounded audit of HLD current

Scoped materials: [AGENTS](AGENTS.md), [SOP](SOP.md),
[record schema](hld-audit-record-schema.json), and
[fixture expectations](hld-audit-fixture-expectations.md).

## Rust Work Mode

Spec + Review/Reliability for this documentation packet; Delivery applies only
to a later, separately authorized validator or runner. This is an independent
audit of **HLD current**, not HLD future, intent rows, or production M8 code.

## Executable Requirements

`REQ-RUST-019.1` governs selected nodes, effective visibility and reexports;
`REQ-RUST-009.2` governs extracted edges, collapse, provenance and roll-ups;
`REQ-RUST-012.1` governs deterministic exports; `REQ-RUST-020.1` binds rows
to compatible generations. The new `REQ-RUST-025.0` specifies this optional,
non-authoritative source-grounded audit. Existing PRIMARY and SOP S8/S9 remain
the acceptance authority.

## Rust Design (L1/L2/L3 + Traits)

- **L1 facts:** canonical compilation-unit and occurrence keys, typed node and
  dependency candidates, evidence references, explicit unknown/exclusion
  reasons, and the frozen Pass A digest. No database or model dependency.
- **L2 reference model:** a pure deterministic transformation from the
  source/config evidence bundle to candidate HLD rows, provenance paths and
  roll-ups. Use an intentionally simple independent traversal; never call the
  production SQL builder or consume production HLD rows in Pass A.
- **L3 audit adapters:** pinned-source/config reader, optional code-graph
  navigator, frozen-production-export reader, artifact hasher, and optional
  zcode reviewer. Traits keep these replaceable and outside production.
  The model is never linked into the mapper binary.

### Two-pass boundary

**Pass A — source first.** For exactly one repository and declared source
scope, derive candidate nodes/dependencies from pinned source and the recorded
compilation configuration. Record each evidence file's path, source commit,
SHA-256, byte/line range, excerpt digest and claim; record canonical
compilation/occurrence keys, scopes examined, exclusions, unknowns, and the
coverage denominator. Production HLD rows are not an input. FTS/code-graph
results locate candidates but are not semantic evidence.

**Freeze.** Canonicalize and sort the Pass A artifact, hash its exact UTF-8
bytes with SHA-256, record the digest and freeze time, and make it immutable
before opening any production HLD export. Pass B rejects a changed digest or
source/config/tool identity.

**Pass B — compare.** Compare the frozen expectation with the HLD current rows
for the same repository, compilation identity and published generation.
Report false and missing edges separately; verify node selection/effective
visibility/reexports, private-chain cycles/diamonds/self-edges, full
provenance, evidence counts and deduplicated file/folder roll-ups. Unknown or
disputed model claims remain explicit and block the scoped audit until
resolved by source/PRIMARY evidence.

### Source-grounded evidence and authority

The first bounded pilot is the pinned serde facade crate-root reexport scope:
commit `aa7913ad0211fef960df68e7812a6211145a888c`, package `serde` 1.0.229,
`serde/src/lib.rs` plus the corresponding `serde_core` crate-root declarations
and their manifests. Record the exact feature/target context; the package's
manifest default is `std`, while workspace feature unification must not be
assumed. This is a partial source scope, not a complete serde HLD table.

Model output is candidate evidence only. Agreement is never PASS. A
source-confirmed false edge fails; an unresolved dispute is BLOCKED. The
deterministic PRIMARY comparison and unchanged SOP S8/S9 decide acceptance.
S8's existing output-key sample stays byte-for-byte governed by its current
algorithm; add separate source-selected samples to look for missing edges,
because sampling only emitted rows cannot find omissions. Report each of the
five repositories separately; never pool denominators. No complete-golden
claim is valid without exhaustive, independently verified scope coverage.

### Pass A / Pass B record

`hld-audit-record-schema.json` is the machine-readable contract. Its record
contains one repository, source/config/tool/executable identities, actual
reported model identity (nullable when not reported), prompt digest, code-graph
version/index coverage, Pass A evidence/candidates/unknowns, frozen expected
artifact digest, Pass B production-export content SHA-256, the separate
`run_generation_digest_value` and source-pin compatibility/compilation identity,
findings, both sampling records and independent acceptance statuses. The
export content hash is not the generation key. Missing production HLD export
means `BLOCKED` or `NOT_STARTED`, never an empty-success comparison.

The HLD edge key is `(generation, source occurrence, canonical target,
relation kind, via kind)`. Product rows carry the PRD/SPEC edge fields,
`extracted` status only, and a unique supporting evidence-row count; companion
evidence records preserve every distinct private chain. Roll-ups group the
deduplicated HLD rows, never path multiplicity. External targets retain package
source and target identity, not only crate/version.

## Verification Matrix

See `REQ-RUST-025.0` in SPEC and `hld-audit-fixture-expectations.md`. Every
named validator/runner test is **PENDING**; none is represented as implemented
or covered. This documentation packet is checked structurally, not by claiming
a Rust RED/GREEN cycle.

## Implementation Plan

1. Review this contract and the one-repository serde source-scope draft.
2. Implement the validator only after approval, using STUB → RED → GREEN →
   REFACTOR with journal evidence for every phase.
3. Run Pass A, freeze its artifact, then Pass B only if a matching production
   export exists. Keep all five repo results separate.
4. Reconcile every disagreement against source/PRIMARY/S8/S9 before any
   production change. A model opinion alone never changes code or a denominator.

## Quality Gate Results

Documentation-only checks are recorded in `JOURNAL.md`. No Rust build,
production HLD run, all-five comparison, S8/S9 pass, or complete audit is
claimed by this packet.

## Open Questions

- Which immutable publication/export supplies the production HLD current rows
  when the M8 builder exists?
- What exact file/folder roll-up counter columns will production publish?
- Which serde compilation target/feature set matches the eventual production
  generation after workspace feature resolution?
- Who performs final source adjudication for model-flagged edges and unknowns?
