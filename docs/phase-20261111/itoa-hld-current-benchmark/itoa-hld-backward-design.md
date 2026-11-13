# itoa HLD backward design

## 1. Rust Work Mode

**Spec Mode + Delivery Mode + Reliability Mode.** Component: Rust analysis
library and CLI, with Turso persistence and DuckDB relational analysis.
The risk is semantic ambiguity and false acceptance, not insufficient indexing.

Work backwards from a source-derived expected graph. Do not replace the
production mapper with a benchmark-specific extractor. The companion reference
is currently a reviewed-in-progress semantic draft, not a frozen canonical
golden. Unavailable compiler, expansion, and external-target evidence remains a
freeze blocker; it is not permission to fabricate identity.

## 2. Executable Requirements

These requirements refine REQ-RUST-009.2, 012.1, 019.1, 020.1 and 022.1.
The canonical SPEC owns the requirement identifiers and verification matrix.

### REQ-RUST-029.0 — source-derived reference separation

WHEN the itoa reference is constructed, THEN every source-written item SHALL
have source-relative byte evidence and every generated item SHALL distinguish
its template, invocation, substitution, and declaration/implementation owner.
Different implementations SHALL NOT share a single trait-declaration identity.
Expected data SHALL NOT be an input to production mapping.

WHEN canonical compilation context, expansion identity, or a target remains
unverified, THEN the reference SHALL remain DRAFT with that gap enumerated.
A graph family or approximate endpoint SHALL NOT count as an accepted
occurrence-level row.

### REQ-RUST-030.0 — typed HLD materialization

WHEN a complete compatible semantic-input snapshot is provided, THEN the HLD
builder SHALL select nodes using REQ 019 and materialize REQ 009 edges, complete
evidence paths, and deduplicated roll-ups. It SHALL reject malformed identities,
missing local endpoints, conflicting duplicate records, and incompatible or
incomplete input generations before publishing anything.

WHEN a path contains an unresolved relation, THEN traversal SHALL stop there.
The unresolved observation remains evidence; it SHALL NOT become an extracted
HLD relation. Inferred/proposed facts SHALL NOT enter HLD-current.

WHEN a private cycle or diamond is encountered, THEN canonical simple paths
SHALL terminate and preserve distinct provenance without multiplying HLD rows.
Direct and collapsed rows remain separate when their via-kind differs.

### REQ-RUST-031.0 — transactional HLD serving

WHEN HLD output is persisted, THEN selected nodes, edges, evidence paths, and
both roll-ups SHALL be one stage-owned atomic work unit in Turso. Downstream
readers SHALL require its compatible completion marker, not merely a table.
Cancellation or failure SHALL leave the previous publication readable and
SHALL NOT expose any part of the new result.

WHEN the same frozen inputs are processed in different orders or operational
attempts, THEN sorted semantic outputs SHALL be identical after excluding the
attempt identity. Empty fixtures are valid only with explicit complete input
markers; missing tables or stages SHALL NOT masquerade as an empty graph.

## 3. Rust Design (L1/L2/L3 + Traits)

### Start with actual itoa relationships

The following are source-level expectations, not claims that production
currently resolves them. Canonical occurrence fields will replace these
readable labels at freeze.

```text
<Buffer as Default>::default --calls--> Buffer::new
<Buffer as Clone>::clone     --calls--> Buffer::new
Buffer::format              --calls--> private::Sealed::write
Integer                     --uses_type--> private::Sealed

<u128 as Sealed>::write
    --> <u128 as Unsigned>::fmt       [private implementation body]
    --> div_rem_1e16                 [private helper]
    --> u128_ext::mulhi              [selected pub(crate) endpoint]
```

The last chain is a source-derived static-dispatch expectation needing target
verification. It is NOT a call chain from the bodyless `Unsigned::fmt` trait
declaration. The generic `Buffer::format<I>` call does NOT create twelve
invented concrete writer calls.

`Sealed`, its associated items, and `mulhi` qualify as local interface nodes.
A private enclosing module affects external reachability, not the separate
non-private item-visibility test. Private `Unsigned` implementations and helper
functions remain evidence. Collapse stops at selected `mulhi`; it must not skip
over it to an external function called by `mulhi`.

Impl blocks are header/evidence containers, not extra D6 interface nodes.
Trait-impl members have separate occurrences and inherit interface visibility.
Relations such as external primitive type implements local `Integer` remain
lower-level impl facts unless they have a valid selected-source HLD projection;
do not synthesize a selected local primitive node to force them into HLD.

Only existing relation kinds are used:
`calls`, `uses_type`, `implements`, `reexports`, `contains`, `derives`.
Generic versus concrete dispatch is evidence metadata, not a new edge kind.
Source-explicit operations and compiler-desugared operations must be accounted
for separately; incomplete oracle coverage cannot silently remove either from
an applicable acceptance denominator.

### Backward dependency chain

```text
EXPECTED source-derived itoa graph                    [offline test input]
                         |
                         v
                  exact comparison
                         ^
                         |
Turso: published HLD nodes + edges + paths + roll-ups
                         ^
                         |
DuckDB: select nodes, collapse paths, deduplicate, aggregate
                         ^
                         |
Turso: effective item visibility + semantic edge evidence
                         ^
                         |
Rust body inference + DuckDB signature/impl/method joins
                         ^
                         |
existing namespace, macro, signature-field and body-site rows
                         ^
                         |
existing source bytes, spans, crate/configuration identity
```

No reference JSON is copied into production tables. Hand-authored semantic
fixtures test the last stage but are explicitly not an itoa source-to-HLD run.

### Existing durable tables: reuse, do not replace

* `crate-dependency-graph-table` and `workspace-config-state-records`: package,
  target, role, edition, cfg/features and source identity.
* `file-source-text-table`: byte-exact source and digest, including expansion
  sources through the existing provenance mechanism.
* `span-reference-text-table`: occurrence identity, item kind, raw visibility,
  source range/text and cfg activity. Extend owner/ancestry metadata here when
  needed rather than introducing another span store.
* `field-signature-detail-table`: syntactic parameters, returns, bounds and
  impl headers; raw strings are not resolved types.
* `body-site-reference-table`: ordinary/method/path/record/field/macro sites.
  This remains the single body-site store, including pending/unresolved sites.
* `module-scope-binding-table`, `name-resolution-scope-records`,
  `path-resolution-site-records`, `macro-expanded-item-table`,
  `lang-item-definition-table`: resolved namespace and expansion evidence.
* `stage-run-generation-records`: compatible input completion and publication.

The existing Turso adapter implements these as generation/attempt-qualified
stage rows with payloads. Register new tables through the same stage owner
inventory; do not create an unrelated database or bypass the stage write port.

### Missing semantic inputs: build as their producers become available

**`lowered-signature-type-table` — Turso durable result, DuckDB analysis.**
Resolve type paths, associated projections, generic bounds, returns and impl
self/trait types from signature fields and namespace bindings. Distinguish
`I::Buffer`, `<u128 as Sealed>::Buffer`, and the concrete array type.

**`impl-header-index-table` — Turso durable facts, DuckDB lookup indexes.**
Keep impl occurrence, self type, trait target, substitutions and associated
member occurrences. This is required to distinguish twelve generated writer
bodies from one trait declaration and to resolve pointer-sized delegation.

**`item-effective-visibility-records` — Turso durable semantic facts.**
One item occurrence references its lexical/semantic owner, effective item
visibility, fn-body ancestry, external reachability, and the evidence establishing
those facts. Unknown effective visibility is not silently private. Derive
selection in DuckDB from these facts; do not trust a caller-supplied selected bit.

**`edge-evidence-record-table` — Turso durable semantic relations.**
Keep source occurrence, optional resolved local/external target, relation kind,
status, original site occurrence, dispatch/substitution metadata, source range,
and exact upstream identity. Resolved rows have an extracted target; unresolved
rows carry one authoritative reason. A source occurrence and a body-site
occurrence are distinct. Lowered signature/impl facts and body resolution feed
this shared contract; no second call-site store is introduced.

Semantic fact publication must declare all required upstream stages. A partial
fact set can be inspected but cannot claim a complete source-derived graph.

### HLD output: computed in DuckDB, served durably from Turso

**`hld-current-interface-nodes`**

Preserve selected nodes even when their degree is zero. Store the complete
semantic occurrence reference, qualified path, assembled signature, normalized
source location, effective visibility and external reachability.

**`hld-current-dependency-edges`**

Use the SPEC §3.2 columns. The unique key is:

```text
(generation, selected source, selected/external target, relation kind, via kind)
```

Every row has `edge_resolution_status_kind = extracted`. The evidence count
is the number of unique original evidence-row keys across retained paths, not
the number of paths or traversal steps. Direct and collapsed edges may coexist.

**`hld-edge-evidence-paths`**

Keep each canonical ordered simple path, its HLD edge key, ordered occurrence
and evidence-row references, and original macro/source provenance. Do not
replace provenance with only a count or concatenate ambiguous display names.

**`hld-file-level-rollup` and `hld-folder-level-rollup`**

Aggregate deduplicated HLD rows, not paths. File keys include compilation-unit
identity and normalized source-relative path; folder keys use their parent
directories, with root normalized to `.`. External endpoints retain their
canonical external identity. If their definition file is unavailable, use an
explicit external endpoint bucket, not a guessed local path; distinguish it
from a verified external definition file. Each HLD edge contributes once at
each level. Report relation/via/origin dimensions without multiplying counts.

The explicit external bucket policy must be shared by reference and actual
exports and tested; it is not an exclusion from edge comparison.

### DuckDB working relations

Use the existing extension-free, file-backed frozen-generation copy mechanism.
Normalize typed HLD node/evidence inputs into
`hld-analysis-node-records` and `hld-analysis-edge-records`. Project canonical
paths through `hld-analysis-path-records` using a recursive CTE. Reuse frozen
table manifests for count/digest checks and require complete semantic-input
markers. Temporary working relations are not a third source of truth.

Traversal starts from selected nodes, follows extracted edges through private
occurrences, and stops at the first selected or external endpoint. Track visited
occurrences per path, preserve terminal relation kind, allow genuine explicit
closing self-edges, and stop private cycles without fabricating self-edges.
Sort all semantic outputs by full canonical keys, never transient dense IDs.

### Layer boundaries and typed seams

* **L1 `parseltongue-core-row-types`:** dependency-free validated node, relation,
  target, visibility, path and result types. Reuse `SemanticOccurrenceKeyValue`
  and compilation identities; no alternate display-name identity scheme.
  The existing `NameSymbolTargetKey::CrateRoot` represents the implicit root
  without a fabricated item span; `Builtin` represents primitive identities
  without a fabricated sysroot definition. Ordinary `core` functions and
  methods are external definitions, NOT `Builtin` symbols merely because
  they belong to the standard library. Generic parameters and trait
  declarations must not be relabelled as concrete implementation occurrences.
  Hand-written structured validation errors; no JSON or SQL dependencies.
* **L2 `parseltongue-table-store-traits`:** a narrow `HldGraphAnalysisPort`
  alongside existing stage store/marker ports. It accepts a frozen compatible
  scope and returns typed results/errors; test doubles can prove orchestrator
  rollback and missing-stage behavior.
* **L3 `parseltongue-duckdb-analysis-adapter`:** JSON/SQL codecs, normalized
  projections, recursive CTE and deterministic export. Structured `thiserror`
  variants distinguish malformed input, incomplete snapshot, generation mismatch,
  duplicate conflict, missing endpoint, cancellation and storage failure.
* **L3 `parseltongue-turso-store-adapter`:** registered ownership, transactional
  HLD output unit and immutable generation publication.
* **CLI:** orchestrate the existing stages and expose actual published rows.
  Until semantic producers are connected, a request for complete HLD reports
  the missing stage; do not mark `visibility-hld` complete because DDL exists.

No unsafe, unwrap/expect in production, new runtime rust-analyzer analysis
dependency, runtime DuckDB extension, third database, or interim memory cap.

## 4. Verification Matrix

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| 029.0 | source evidence audit | reference validation | all selected nodes have source/template+invocation evidence; no pooled implementation owners | missing/duplicate count = 0 |
| 030.0 / 019.1 | local visibility fixtures | unit/integration | public-in-private-parent and pub(crate) selected; body-local/pub(self) excluded | exact node set |
| 030.0 / 009.2 | direct/private/selected-stop fixtures | integration | terminal kind retained; first selected target stops traversal | exact edge/path set |
| 030.0 / 009.2 | cycle/diamond/self fixtures | integration | finite canonical paths; unique evidence union; correct self-edges | exact counts |
| 030.0 | malformed/conflicting/unresolved fixtures | negative integration | typed failure or stopped path; no fabricated extracted target | zero invalid rows |
| 031.0 / 020.1 | missing/stale-input fixtures | integration | table presence alone is insufficient | no publication |
| 031.0 / 022.1 | cancellation/failure fixtures | integration | atomic rollback; previous generation readable | unchanged prior digest |
| 031.0 / 012.1 | reordered-input/repeated-attempt fixtures | integration | identical sorted semantic rows | byte equality |
| 031.0 / 009.2 | file/folder/external fixtures | integration | each deduplicated HLD row counted once | sum = HLD edge count |
| 029.0–031.0 | pinned itoa source comparison | end-to-end | real mapper output equals independently frozen reference | missing/wrong/extra = 0 |

Concrete Rust test names and exact commands must be added to the canonical
SPEC before their implementation. This matrix is the design-level inventory,
not a report that tests already exist or pass.

## 5. Implementation Plan

1. Correct and expand the reference to individual semantic occurrences and
   relation evidence. Verify the entire two-file source, generated substitutions,
   selections and exclusions. Resolve scope/identity gaps without copying mapper
   answers into the expected graph.
2. Implement the final HLD stage against typed complete semantic-input fixtures:
   STUB interfaces, RED contract tests, GREEN SQL/transactions, REFACTOR, gates.
   First materialize typed inputs in DuckDB; then add frozen-Turso marker
   validation and atomic publication. The first slice alone does not prove
   persisted generation completeness or transaction rollback.
3. Work backwards into visibility, signature/impl and body-resolution producers,
   extending existing persisted rows and ports. Tests name the specific expected
   relation each new producer enables.
4. Connect the real CLI and run pinned itoa. Emit actual rows and a discrepancy
   report, even when the result is blocked/incomplete; never substitute fixtures.
5. Freeze verified context/reference when all required proof exists, repeat exact
   comparison, and measure time/RSS and deterministic exports. Phase 2 remains
   serde → rust-analyzer → ripgrep → tokio → cargo-modules.

Do not restart the halted native diagnostic or old child assignments as part
of this work. No broad optimisation or unrelated proc-macro support precedes
the itoa slice.

## 6. Quality Gate Results

Not yet run for this design. Reference checks are recorded in the companion
reference. Production implementation, Rust gates and source-to-HLD acceptance
are pending. Passing isolated HLD fixtures will be implementation progress,
not Phase 1 acceptance or all-five milestone completion.

Required code gates: scoped `cargo fmt --check`, Clippy with warnings denied,
tests, and a release build using the existing pinned toolchain/lock.
Record expected wall time before long runs, stop at twice that expectation,
measure whole-process-tree RSS, and append each TDD phase's exact evidence to
the canonical journal.

## 7. Open Questions

* Reference review correction pending: the current call-inventory generator
  incorrectly labels ordinary external `core` functions/methods as `Builtin`.
  Their target identities are unverified external definition candidates.
  Correct this before any fixture conversion or freeze; do not copy the
  draft's classification into production.
* Canonical external endpoint/source identity and compiler-verified generated
  occurrence evidence are not yet captured for the reference.
* The oracle currently exports mapped semantic paths, not complete calls,
  visibility or macro-produced inventories; no broader capability is assumed.
* Full implicit/desugared operation coverage remains an explicit semantic
  inventory gap, not a reason to declare a source-explicit subset complete.

These are evidence/implementation blockers to resolve, not requests to weaken
the existing acceptance requirements.

## Implementation checkpoint — recursive paths and collapse

The current bounded DuckDB slice recursively materializes ordered HLD paths
and collapsed rows from explicit typed inputs. A recursive CTE seeds selected
sources, extends through nonselected LocalItems with per-path full-identity
cycle guards, and terminates at selected or canonical external/builtin
endpoints. Terminal relation kind is retained; one return to the selected
source is permitted as a finite self-edge, while private cycles alone produce
no row. Ordered evidence identities distinguish paths with equal endpoint
sequences, and SQL uses UNNEST plus COUNT(DISTINCT ...) for the evidence union.
Output is sorted and marked GraphPathsOnly; file/folder roll-ups remain None.

This does not validate frozen Turso stage markers, publish durable generations,
run real-source producers, expose CLI completion status, or establish the
pinned itoa source-to-HLD comparison. CompleteGraph remains reserved until
roll-ups exist.
