# PRD — phase-20261113: full dependency graph and HLD for Rust code that compiles

Status: draft. Owner: human-architect-intent-author.
This file states what is in force now. It replaces `docs/phase-20261111/PRD-phase-20261111-compiling-hld.md`
and the earlier itoa-scoped PRD, both kept as history.
Executable contracts: SPEC-phase-20261113-executable-requirements.md.
Design: ARCH-phase-20261113-pipeline-graph-tables.md.
Procedure: `docs/phase-20261113/SOP-phase-20261113-full-item-dependency-graph.md`.
**M** marks observed facts; unmeasured budgets are targets, not results.

## 1. Objective

Development checkpoints may persist explicitly incomplete full-map facts (REQ-RUST-045.0), with
unknown visibility and unsupported resolution retained rather than guessed. They reuse the same
canonical graph and source evidence; they do not qualify complete semantics or bypass HLD gates.

Build, for compiling Rust workspaces, one detailed full-item-dependency-graph,
using CPU tools only: no LLM and no training in production.
The full graph holds every item outside function bodies, private ones included, and every
dependency proven from source. hld-current-dependency-edges and the roll-ups are views of it.
An ext-llm-coding-agent uses the full graph and its evidence to reason about dependencies
before editing. Interface/HLD abstractions are later queries over that same graph.

**Active Phase 1 scope:** the detailed full map, its evidence and independent comparison.
Higher-level selection, collapse, roll-ups and their projection-only decisions are deferred to
the [abstraction and decision backlog](phase1-abstractions-decision-backlog.md).
Retained HLD contracts describe later derived products, not Phase 1 exit requirements.

Port rust-analyzer algorithms over persisted rows rather than host its analysis database in production.
Aim for exact extracted dependencies, report unknowns without guessing, and enforce SPEC's explicit
precision/coverage gates rather than claiming that compilation guarantees a complete graph.

D1: cargo metadata --offline. D2: five repositories qualify equally: rust-analyzer, ripgrep, tokio, serde,
cargo-modules; development is driven first by one small pinned library, itoa (section 10).
D3: proc-macro-shape-table checked against real oracle expansions. D4/KL1: never execute build scripts
or read their saved target/ outputs. D5: rustc only --print cfg / --print sysroot during production.
D6: section 3.

## 2. Source closure, targets and context

Qualification is an external, offline cargo check of workspace lib+bin targets at pinned commits.
It is never invoked by production mapping. Compilation does not imply every .rs file in a checkout parses.
Map lib + bin workspace targets and normal dependency closure. Exclude tests, benches, examples,
dev-dependencies and build-script targets; cfg(test) is OFF. A test-enabled run is a Phase 2 option.
Record host proc-macro/build dependency contexts for provenance without mapping those as application targets.

The semantic source closure is files reachable from mapped targets' module trees under active cfg,
extended by ordinary includes and discovered macro expansions. Other .rs files are stored byte-exact only.
Parse errors/non-UTF-8 stop mapping only in that active closure; unreachable fixtures need not parse.
Inactive items encountered in reached files are counted as unresolved(cfg_inactive), never silently dropped.

Compilation-unit identity is Cargo package ID/source + target name/kind + host/target context.
Preserve dependency aliases separately. Enabled features, cfg and edition are per unit; sysroot uses target cfg.
Different versions/sources, lib+bin sharing source and dependency feature contexts never share semantic identity
merely because names or bytes match. Default features are resolved for the selected normal-dependency graph.
Record Cargo's resolver version; derive unit contexts rather than assuming metadata's unified feature list
is already target-specific. Record the target triple (host by default); explicit --target is print-cfg-only.
No std/no core and edition-specific extern preludes follow SPEC section 4.

## 3. Product: the full graph and the levels derived from it

This is one logical graph, not one giant physical table. Nodes and edges link to retained
source, signature, body, resolution and expansion evidence. Abstractions are DuckDB queries
over a compatible frozen generation; materialization may cache their results later. They
do not require separate source-analysis pipelines and cannot supply missing semantic facts.
Intent remains separately authored input, not something a query can infer from code.
The diagram includes the deferred query layer to preserve the longer-term architecture.

```text
 source
   -> Rust: parse, expand, resolve, infer                 facts
   -> Turso: full-item-dependency-graph                   stored, never collapsed
        item-effective-visibility-records   one row per node
        edge-evidence-record-table          one row per edge fact
   -> DuckDB: levels                                      filter, collapse, count
        public-item-l2-interface    hld-current-interface-nodes, hld-current-dependency-edges,
                                    hld-edge-evidence-paths
        folder-file-l1-rollup       hld-file-level-rollup, hld-folder-level-rollup
```

Rules (D6):

```text
 1  The full graph is built first and is a product. A level adds no fact; it only filters
    nodes, collapses paths and counts.
 2  Full-graph nodes: every module, function, struct, enum, trait, type alias, const, static
    and associated item owned by the crate root, a module, a trait or an impl, private
    items included. Nothing inside a fn body. Impl blocks are not nodes.
 3  Each source file has one file-public-area-node, separate from its module node, kept at
    the interface level even when the module is private. It contains the file's top-level
    nodes.
 4  Each use outside fn bodies is a node, with one imports edge per resolved binding.
 5  An inline module is an ordinary module node with its own scope.
 6  Every extracted edge is stored, private -> private included. Relation kinds: calls,
    uses_type, implements, reexports, contains, derives, imports, reads (a read of a const,
    static or associated const).
 7  A call, type use or read targets the real item, never the use line; the use line is
    evidence. A function has edges only to what it uses, not to every import in scope.
 8  The full graph is the two existing tables above; no table is added.
 9  The interface level keeps the crate root, non-private items, file-public-area-nodes and
    use nodes; private items are dropped there and remain as steps in evidence paths.
 10 Collapse walks only dependency edges: calls, uses_type, implements, reexports, derives,
    reads. A contains or imports edge is never a step of a collapsed path; it shows at the
    interface level only as a direct edge to a kept or external target.
```

Visibility is effective visibility, not the `pub` token alone. Trait members, trait-impl members and
enum variants inherit their interface visibility; inherent associated items respect their own
restrictions; private-equivalent pub(self) does not qualify. External reachability is a separate
fact that follows ancestor modules and public reexport aliases. Public or restricted reexports
produce one edge per resolved binding, including aliases and globs, sourced at the nearest
selected enclosing module with the private lexical module kept in evidence. SPEC section 4 defines
the collapse algorithm: cycles terminate, diamonds retain provenance without duplicate HLD rows,
genuine self-edges are retained, and an unresolved intermediate link never creates an extracted
successor. File/folder roll-ups count deduplicated HLD rows, not the number of private paths.

Edges are extraction only. A generic or dynamic call targets the trait method once, never a
guessed implementation. Unknowns stay in evidence and site rows with one reason each.
hld-current-dependency-edges is written solely by cpu-tool-hld-table-builder, never by an agent
or a person. Its owned columns, defined with full semantics in SPEC section 5:

```text
run_generation_digest_value
edge_source_occurrence_key
edge_target_reference_value
edge_source_signature_text
edge_target_signature_text
edge_source_qualified_path
edge_target_qualified_path
edge_relation_kind_value
edge_collapse_via_kind
edge_evidence_row_count
edge_dependency_origin_kind
edge_resolution_status_kind
```

Via values: direct/collapsed. Origin values: same_crate/workspace_crate/std/external_crate.
Every node carries node_reachable_outside_crate; external targets preserve package source and target
identity, not just crate/version. Full span identity and snapshot are recoverable through the
generation/occurrence keys.

## 4. Architecture and storage

Two databases and no third. Turso holds durable rows, generation markers, frozen snapshots and
atomic publication, with BEGIN CONCURRENT/MVCC for concurrent stage writes. DuckDB is the
analysis engine: it receives a copy of one complete published generation through the Rust
Appender, with per-table count and digest equality checked before any analysis, and owns name
resolution rounds, signature and impl joins, method candidates, HLD collapse and roll-ups.
Name resolution uses complete, non-monotone stratified SQL rounds, not append-only accumulation.
Rust macro expansion alternates with SQL and writes its evidence to Turso. Only final product
tables copy back, with digest verification. DuckDB runs bundled, with no extension installed or
loaded, no external access, and its temp directory inside the run directory.

Every stage reads declared tables and writes declared tables. No memory crosses stage/unit boundaries.
Local lowering/fixed-point state may live inside a single work unit; outputs must be persisted before consumers run.
Name resolution and item-position macro expansion are ONE crate fixed point; macro expansion is its library,
not a separate inter-stage in-memory channel. Expression macros expand inside the body unit.
No hidden resolved-site buffer crosses stages.

Layers:
- L1 parseltongue-core-row-types: IDs, rows, errors; zero dependencies.
- L2: write and analysis ports, scheduler, copy-adapted pure macro algorithms.
- L3 adapters: Turso runtime, DuckDB analysis, parser/cargo bridges, optional next-solver adapter.
- parseltongue-cli-dispatcher-binary wires the production stages.
- oracle-semantic-export-harness is an external offline test artifact; ide/hir/salsa permitted there only.

Turso is the write and serving engine. Pin turso =0.9.0-pre.5; M: crates.io stable 0.8.3, maximum prerelease
0.9.0-pre.5. M: the inspected API is async (query/execute/prepare/build) and supports
TransactionBehavior::Concurrent / BEGIN CONCURRENT. The L3 store adapter owns the runtime; L2 remains sync.
Source API availability is not measured throughput or a concurrency guarantee.
Conflict policy: initial attempt plus 5 retries, delays 10/20/40/80/160 ms, rollback and reread/recompute.
Nonconflict errors do not retry; cancellation interrupts backoff. Maximum in-flight units equals worker count.

One workspace DB plus one shared library DB, both Turso. stage-run-generation-records records compatibility
keys derived from source snapshot/config/tool versions. Every row/marker carries run_generation_digest_value.
Publish an immutable library generation first, then atomically switch the workspace manifest referencing it.
Readers pin the manifest; GC cannot remove referenced library generations. A crash leaves unreferenced
staging rows, never a partly published workspace. Crate bindings/expansions publish atomically within staging;
downstream stages require matching completed unit markers. Old published generations remain readable.
Identical-key reruns use separate staging attempts, never overwrite published rows; semantic exports omit
operational attempt sequences. Source snapshots exclude .git, target/ and oracle output directories.
No cache by default; a stage-local per-crate read cache requires a recorded measured budget miss.
SpanDigestIdValue is content-only, truncated to 16 bytes with full 32-byte digest collision detection.
SemanticOccurrenceKeyValue includes compilation unit, path/range and expansion invocation/hygiene ancestry.

## 5. Stages and ownership

Every stage reads declared tables and writes declared tables, with generation markers on every
write. The Turso registry gives each table one owner stage. Stage 0 bootstraps from the explicit
filesystem/process boundary; ingest alone reads source bytes; resolvers never read source files
behind the table interface.

```text
 stage                               engine          writes                                    state
 0  crate graph                      Rust            crate-dependency-graph-table,             built
                                                     workspace-config-state-records
 1  source ingest                    Rust            file-source-text-table                    built
 2/3 syntax walker                   Rust            span-reference-text-table, span-byte-     built
                                                     ownership-ranges, field-signature-detail-
                                                     table, body-site-reference-table,
                                                     namespace input tables, lang-item-
                                                     definition-table, macro scope records
 4+5 name resolution + item macros   DuckDB + Rust   module-scope-binding-table, name-         built, not
                                     expander        resolution-scope-records, path-resolution- qualified
                                                     site-records, macro-expanded-item-table
 6  signature-type-lowering-stage    Rust + DuckDB   lowered-signature-type-table,             not built
                                                     impl-header-index-table
 7  semantic-edge-evidence-stage     Rust + DuckDB   edge-evidence-record-table: every edge    not built
                                                     of the full graph, all eight kinds
 -  item-effective-visibility-stage  open (AD-3)     item-effective-visibility-records: every  not built
                                                     node of the full graph
 9  hld-current-materialization-     DuckDB          the five HLD tables; reads only the two   built on
    stage                                            full-graph tables and unit provenance     fixtures
 10 cpu-tool-status-coverage-banner  Rust            per-level-coverage-banner                 partial
 11 cpu-tool-oracle-edge-differ      DuckDB, offline oracle-edge-agreement-table               built for
                                                                                               paths
```

Stage 7 has three sub-steps: 7a locals, inherent methods and autoderef; 7b traits in scope;
7c the optional next-solver adapter. Stage 0 verifies lock presence before metadata and immutable
bytes after it, and loads proc-macro-shape-records.toml into proc-macro-shape-table. Closure-discovery
requests return to persisted ingest/walker work; no stage reads unknown files directly. Stage 11
never spawns analysis engines from inside a production mapping run.

## 6. Porting sources

Use pinned rust-analyzer sources from SPEC section 10, retaining MIT OR Apache-2.0 headers for copied code.
- Copy-adapt hir-def/src/nameres/ collector/path resolution over tables.
- Copy-adapt mbe expander/transcriber into parseltongue-port-macro-expander; mbe has a salsa dependency.
  If span is linked, default-features=false and the full production tree must remain salsa-free.
- Copy-adapt hir-expand/src/builtin/ declarative/built-in macro behavior and invocation provenance.
- Rewrite-from hir-ty/src/lower.rs, infer/, method_resolution.rs and hir-def/src/expr_store/lower.rs:
  signatures, body lowering, inference, method lookup. Desugaring and operators require lang items.
- M: lang_item.rs has 169 declaration sym rows; discover those present in the selected sysroot, not all 169 blindly.
- Preserve incoherent inherent impl handling: flagged types, primitive self heads and sysroot-origin search;
  distinguish rustc_has_incoherent_inherent_impls from rustc_allow_incoherent_impl.
- M7 only: link pinned ra-ap-rustc_type_ir / ra-ap-rustc_next_trait_solver through a table-backed adapter.
- Link pinned parser crates; never link ra_ap_hir*, ra_ap_ide*, ra_ap_base_db or salsa into production.

SPEC section 4 requires fixtures for #[path]/cfg_attr, nested/ordinary includes, hygiene/$crate,
glob precedence/ambiguity, aliases, edition-specific extern preludes and no_std/no_core.

## 7. Oracle and evidence

oracle-semantic-export-harness is built from rust-analyzer@2a362a41a8eb6e090f1a12d9b712df85af6f0c33,
not the installed 1.99.1 binary. Record build commit, executable digest, toolchain and configuration.
It exports normalized per-site path targets, expansion items with invocation IDs, impl relations,
call/method targets, signature refs and effective visibility/reachability.
PRIMARY disables build scripts; SECONDARY enables them; both enable proc macros and use identical
lib+bin/default-feature/per-unit cfg/edition policy with test cfg OFF.
A per-site diff identifies KL1; analysis-stats aggregates cannot establish that attribution.
Only PRIMARY is the answer key for gates. SCIP is secondary occurrence evidence, not a complete semantic oracle.
Harness prerequisites (including prebuilt proc-macro artifacts for PRIMARY) are provisioned offline outside
production; a missing real expansion blocks REQ 013 rather than pretending a shape passed.

## 8. Process and unresolved-reason rules

The production allowlist is only cargo metadata --offline, rustc --print cfg, rustc --print sysroot.
External harness builds, qualification, tool-version collection and oracle exports are outside it.
Production never builds/checks, runs proc macros, executes build.rs or reads saved target/ outputs.
CARGO_PKG_* / CARGO_MANIFEST_DIR come from metadata; unavailable env and generated includes are build_script.
Build-script-only cfg is OFF, counted as cfg_inactive, attributed to KL1 only with paired export evidence.

SiteUnresolvedReasonKind is authoritative; one reason per site in causal precedence:
cfg_inactive, build_script, proc_macro, external_source_missing, unknown_receiver, trait_solver_needed.
Inactive/PRIMARY-absent KL1 sites are counted exclusions. If PRIMARY sees a site, an unresolved production
answer remains a nonmatch, including unknown_receiver/proc_macro/trait_solver_needed.
Missing oracle evidence blocks a gate. Every mismatch/exclusion and its evidence is retained.
The same rules apply in SPEC section 4 and the SOP; a reason never excuses an extracted wrong target.

## 9. Requirements, verification and scoring

SPEC section 3 is the single executable wording. Do not maintain an abbreviated conflicting
WHEN/THEN copy here. The requirements cover input/process integrity (001–004/014–015), resolution
and macros (005–008/010.3–010.4/011.2/013/016–018), the full graph and HLD levels (009/019/029–032),
engines and generations (005.2/016.2/020–022/024), memory (011/024.5/024.9), deterministic export
(012), the comparator and oracle (021.2/025–027) and diagnostics (024.6–024.7/028).

Use SPEC section 6's requirement → test → SOP-step crosswalk and exact four-word test names.
Every percentage is computed separately per repo with raw numerator/denominator/exclusions.
Empty denominator is N/A and blocks a corpus gate, not 100%; empty fixtures assert zero output.
Keys include compilation unit, relative source path, byte range, site kind and expansion provenance.
005 paths ≥99.5%; 006 invocation sets ≥99%; 013 matching-shape invocations 100%;
018 relation intersection/union ≥99.5%; 007 correct/extracted method sites ≥99%;
008 correct/eligible PRIMARY method sites ≥95% mandatory at M8; 019 selection/reachability 100%.
Wrong extracted sites count against precision even with a purported reason.
SOP PT-EXTRA is site-level rate, distinct from deduplicated HLD/file-pair counts; sampled-row misses still fail.

## 10. Phases and milestones

### Phase 1 — one small pinned library

Development is driven by a real-source test on one library that a person can read in full.
Phase 1 produces its detailed full graph and linked semantic evidence from real source through
the production pipeline and compares them against an independently reviewed target. It does
not require interface selection, collapsed paths, roll-ups or any of the five HLD outputs.

```text
 repository   dtolnay/itoa 1.0.18
 commit       af77385d0daf4d0e949e81f2588be2e44f69f086
 target       lib, edition 2021, x86_64-unknown-linux-gnu, no features, no build script
 size         src/lib.rs 466 lines, src/u128_ext.rs 22 lines
 full graph   item-effective-visibility-records (nodes), edge-evidence-record-table (edges)
 evidence     source/spans, signatures, bodies, resolutions, expansions, unresolved/inactive sites
 readback     compatible frozen full-map generation, independently of derived HLD tables
```

The retained diagnostic containment slice uses the explicit
`original-top-level-containment` final stage. It is limited to cfg-active items owned by their
original file at top level, with a graph-only file-area identity and direct `contains` edges. It
does not claim semantic dependency completeness; full semantic publication prerequisites remain
unchanged. This mode is not the primary product or the full-map implementation path; do not
silently widen it to admit methods. Build the missing full-map producers separately.

Narrower than the whole product:

```text
 one repository instead of five
 one compilation unit plus whatever `core` handling AD-1 selects
 not needed by itoa, so not built now: proc macros, build scripts, features,
 third-party dependencies, async, dyn dispatch, the optional trait solver (7c)
```

Acceptance:

The Phase 1 target is the complete scoped full-graph node/edge inventory, its linked semantic
evidence and unresolved/inactive inventories, with a machine-readable upstream contract manifest.
The source/signature/body/resolution tables are part of the detailed evidence contract, not
optional presentation data. Keep a separate eligible local-only subset for diagnosis only.
The five expected HLD files are retained deferred artifacts, not Phase 1 acceptance inputs.
The existing candidate inventory is an `LLM_PARSED_DRAFT` authored by Devin from pinned
source, with symbolic, noncanonical identities; no independent compiler/oracle verification has
frozen it and it is not production input. Its node/edge counts are observations only. The preserved
candidate rows are not whole-source coverage: operators, body
types, literals and repeated same-line sites remain known omissions. OD-2/OD-8 and the remaining
file-module/area part of OD-7 block affected
assumptions; OD-9 is deferred because this full-map checkpoint performs no collapse.

```text
 A1  the production binary runs on the pinned source with no pre-resolved input
 A2  every expected full-graph node and edge is present with its full identity and
     evidence range, private items and private -> private edges included
 A3  no unexplained extra, missing or wrong node, edge or evidence row is present
 A4  unresolved/inactive sites match the reviewed inventory; a reason alone does not excuse
     a resolvable missing edge, and diagnostic sysroot exclusion is not complete coverage
 A5  source/configuration/executable digests, run time, peak process-tree RSS and disk are recorded
 A6  frozen full-map readback preserves exact evidence and compatible generation identity;
     cancellation/failure preserves prior serving output; semantic output agrees at 1/4/16 workers
```

Relationships the reference must contain (from source review):

```text
 <Buffer as Default>::default --calls--> Buffer::new
 <Buffer as Clone>::clone     --calls--> Buffer::new
 Buffer::format               --calls--> private::Sealed::write     one generic edge, not twelve
 Integer                      --uses_type--> private::Sealed
 <u128 as Sealed>::write --> <u128 as Unsigned>::fmt --> div_rem_1e16 --> u128_ext::mulhi
     three direct edges in the full graph, with private intermediates and exact evidence
 Buffer::new                  --reads--> <i128 as Integer>::MAX_STR_LEN
 Buffer::format               --calls--> core::hint::unreachable_unchecked
     the `use core::hint` node is proof, not the target
 use nodes of lib.rs          --imports--> core::hint, core::mem, core::mem::MaybeUninit, core::str
 area node of lib.rs, area node of u128_ext.rs  --contains--> their top-level nodes
```

Targets inside `core` depend on AD-1; under its option B those edges are recorded unresolved
for bounded diagnosis only. That option does not establish complete selected-source semantics.

Passing Phase 1 is not a pass of M8 or of the five-repository SOP.

Implementation status and run evidence are recorded in the
[current milestone status board](../phase-20261111/current-milestone-status-board.md)
and its linked run reports, not inferred from this product plan. A typed fixture passing,
a producer being written, and source-backed publication passing are separate claims.

Build order:

One real-source test per slice. Expected rows live only in the test.

```text
 slice 0   retained baseline: time, peak RSS and last stage reached; AD-1 remains explicit
 slice 0a  retained diagnostic: original top-level area:src/lib.rs --contains--> Buffer
 slice 1   original item/member nodes + source containers; separate impl ownership;
           <Buffer as Default>::default --calls--> Buffer::new with frozen full-map readback
 slice 2   <Buffer as Clone>::clone --calls--> Buffer::new; implements and uses_type edges
 slice 3   Buffer::format --calls--> private::Sealed::write (one generic edge; REQ-RUST-042.0)
 slice 4   the u128 chain through macro-generated impls: three direct full-graph edges
 slice 5   use nodes, imports edges, file-public-area-nodes, reads edges
 slice 6   whole library compared against the frozen reference (A1 to A6)
```

A slice passes on the full-graph rows and their exact evidence. No interface-level output
is required; visibility remains a semantic fact, not a filter that discards private nodes.

Each slice adds only the general behaviour that relationship needs. Nothing is keyed on
itoa names.

The hardest bounded next target is slice 3, not a promise to implement all Rust method lookup
at once. Its prerequisites are the closed containment loop, original item/member identities,
impl/signature records, lexical body paths and the simpler default-to-new call. The producer must
prove `i: I -> Integer -> private::Sealed -> write`, retain the associated projection `I::Buffer`,
and produce exactly one generic target at that call site, never a guessed concrete integer impl.
Distractor methods, lexical shadowing, ambiguous candidates, cyclic bounds and source renames
are required negative/mutation checks. Completion includes persisted and published evidence,
not only resolver unit tests. Unknown external dependencies remain reasoned unresolved facts.

Only one acceptance target is active at a time: member nodes/source containers toward
default-to-new, then generic dispatch. For each, verify backwards from the frozen full map
and evidence to source; fix
the earliest observed incorrect or absent producer and rerun the same target. Supporting component
work must be necessary for that target. Structural-only output cannot carry semantic calls or
stand in for complete semantic coverage; publication prerequisites remain explicit and enforced.

### Phase 2 — five repositories, SPEC M0–M8 is authoritative

Order: serde, rust-analyzer, ripgrep, tokio, cargo-modules.

- M0: workspace skeleton, core IDs/errors/fake store, dependency gates, oracle harness/build manifest.
- M1: graph/ingest/parser, Turso runtime, generation markers/scheduler; 001–004/014–015/020–022.
- M2: combined name resolution/macros, built-ins/lang items; 005/006/016 and discovery part of 017.
  Proc-macro-dependent sites excluded-and-reported only until M3.
- M3: checked proc-macro shapes; 013, rerun 005/006 with temporary exclusions removed.
- M4: signature lowering/impl index; 018 and incoherent impl-header fixture.
- M5: 7a locals/inherent/autoderef; 007/017 lookup, report 008.
- M6: 7b trait lookup; 007/report 008. All five ≥95% permits skipping M7.
- M7: optional 7c solver; 007/010/report 008.
- M8: HLD/collapse/roll-ups/banner; 009/019/011/012 and mandatory final 008, all contracts green.

Intermediate milestones use SOP §3 subsets, not a premature full-HLD verdict.
M8/releases require full S1–S10 on all five with the same production hash and toolchain.

## 11. Budgets and measurements

Final target: peak resident memory of the whole production process tree at most 500,000,000 bytes,
enforced only after working end-to-end results exist for all five repositories. Until then memory is
measured and reported, with no interim cap, and correctness comes first.
Target: body inference under 30 minutes on 4 cores for rust-analyzer plus selected dependencies.
Measure summed resident bytes at 10 ms intervals with child accounting and record telemetry limitations;
exclude harness/indexer processes. Disk size and stage wall time are reported, not invented thresholds.
The full graph is larger than the HLD on every repository; its size is measured and reported.
No budget or correctness result is claimed achieved by this document.

## 12. Out of scope

Broken active-closure code recovery; all LLMs/training; full proc-macro execution in production;
build-script execution/output ingestion; test/bench/example/dev targets; all-feature/test-cfg exploration;
intent tables hld-future-dependency-edges and hld-delta-dependency-view; incremental remap.
Dynamic/generic calls target the applicable trait method, not a guessed runtime implementation.
Library-source reuse is compatibility-keyed, never a source-digest-only semantic cache.

Not in Phase 1: the five Phase 2 repositories, the native sysroot timeout except as far as AD-1
option A requires it, CLI query verbs, the coverage banner, and memory or speed optimisation.

## 13. Decisions

### Decided

D-A: external normalized semantic harness, PRIMARY gates, SECONDARY KL1 attribution, SCIP secondary.
D-B: lib+bin, test cfg OFF, per-unit contexts. D-C: ≥99% site precision, sample misses fail.
D-D: ≥95% final coverage at M8. D-E: authoritative M0–M8 and milestone SOP subsets.
D-F: copy-adapt mbe, optional span without defaults, no salsa in production.
D-G: pin async Turso prerelease, runtime owned by L3.
D-H: content digest versus semantic occurrence identity, full collision detection, Cargo target identity.
D-I: compatibility generations, atomic publication and shared-library manifests.
D-J: one reason enum/scoring policy. D-K: formulas, 8192 rounds, six total write attempts.
D-L: clause tests, module/visibility/collapse fixtures. D-M: executable SOP query/coverage/sampling fixes.
D-N: four-word owned identifiers/artifacts; phase document basenames retained.
KL1 remains a known limitation, not permission to exempt arbitrary oracle-proven bugs.
Unmeasured risks: solver adapter scope, macro-shape coverage, disk lookup cost, MVCC throughput and RSS.
Remaining corpus hashes are pinned at M1; harness executable hash is recorded after its pinned build.

Decided on 2026-11-13: rules 1 to 10 of section 3.

### Open, graph and level rules

Active correctness decisions are OD-2 provenance, OD-4, the remainder of OD-7 and OD-8.
OD-2 roll-up attribution, OD-3 presentation copies, OD-5 for a future `pub use` source scope,
OD-6 and OD-9 have moved to backlog BD-01–BD-05.

```text
 OD-2  Generated-item identity and source/container provenance, including `contains`.
 OD-4  Enum variants and struct fields as nodes versus retained evidence. itoa's private
       fields and their type/array-length facts still require an explicit policy.
 OD-7  Source-container rule decided on 2026-11-14: each original non-body item has one
       lexical source container. File-level items and impl members belong to the file area;
       nested items belong to their enclosing inline module or trait. An impl is skipped as
       a graph container: its member uses the impl's lexical source container. Impl identity,
       self type and trait ownership are separate metadata, never a second contains parent.
       No contains-only collapse is permitted. The file-module-to-area relation remains open.
 OD-8  May `reads` and `uses_type` start at a non-function item? itoa needs it for
       `const MAX_STR_LEN: usize = <u64 as Integer>::MAX_STR_LEN` (isize, usize) and for the
       array length in `Buffer`'s field. Lean: yes, the source is the item that holds the
       expression.
```

OD-7's source-container decision needs source-reviewed reference updates; its remaining
file-module/area question and OD-8 still block affected rows from being frozen.

### Open, pipeline choices

Each one states what the code does today, the options, and what each option costs.
A recommendation is given where the evidence supports one. None is decided.

### AD-1  How itoa's references into `core` are handled

```text
 today      a file that mentions `core`, `Clone` and similar makes the pipeline ingest and
            parse the sysroot crates as extra units. itoa's lib.rs does.

 measured   2026-11-13, slice 0: itoa plus sysroot (619 files parsed) did not finish name
            resolution within its declared 2,400 s limit; exit 124, parent peak RSS 8.32 GiB,
            nothing published. Report:
            hld-current-test-runs/20261113T151000Z-itoa-unchanged-pipeline/
            (removed from the tree on 2026-11-14; last present at commit a2e13a3bd)

 in force   option B, as an explicit interim, by the rule under "recommend" below. Taken by
            the agent after the measurement; the user was told and may reverse it.
            Switch: SPEC REQ-RUST-038.0.

 option A   keep sysroot ingestion
            + targets into core get real identities
            - first itoa result waits on the unqualified name-resolution run

 option B   first itoa pass without sysroot units
            + most itoa-to-itoa edges are buildable now
            - not all: the signed `write` calls `Unsigned::fmt(self.unsigned_abs(), ..)`; which
              impl that reaches depends on the return type of a core method
            - edges into core (trait impls of Default / Clone / Copy, calls on MaybeUninit,
              pointers, slices) are recorded unresolved, reason: external source missing
            - the comparison must be labelled local-only until A is done

 option C   name core targets by canonical path without parsing the sysroot
            - conflicts with "no invented sysroot identities"; needs a change to section 3

 recommend  measure first: run today's pipeline on itoa once (slice 0). If it finishes
            within a stated time, take A. If it does not, take B as an explicit interim.
```

### AD-2  Who resolves paths inside function bodies

```text
 today      the syntax walker stores body paths as text; name resolution takes only
            import paths and signature paths, and skips anything inside a fn body.

 option A   send body paths to DuckDB as a third kind of path site, resolved in one SQL
            pass after the name fixed point has finished
            + reuses the existing path-candidate SQL; joins stay in DuckDB (section 4)
            - more rows in the analysis copy; needs a SPEC requirement change

 option B   resolve them in Rust inside the body inferencer, reading binding rows
            + matches section 5's stage 7 wording
            - a second path resolver to keep consistent with the SQL one

 recommend  A. Looking up an item path is a join the SQL already does; the Rust
            inferencer then only drops names that a local variable shadows.
```

### AD-3  Where effective visibility is computed

```text
 today      the Turso registry and the HLD publisher already name a separate stage,
            item-effective-visibility-stage, with its own table and required marker.
            Section 5 shows the registry's stage names without deciding where the work
            is computed.

 option A   separate stage, DuckDB SQL only (what the code already expects)
 option B   inside the builder; the registry and the publisher then have to change

 note       under rule 8 of section 3 this table holds every full-graph node, not only selected ones, so
            the stage must also write private items, use nodes and file-public-area-nodes.

 recommend  A. It is the smaller change.
```

### AD-4  How much body inference the first version has

```text
 itoa needs  receiver types from parameters, `self`, and simple `let` bindings;
             inherent methods (7a); trait methods through a direct bound (7b)

 option A    exactly that; anything else is recorded unresolved with a reason
 option B    port the wider inference first (closures, match ergonomics, autoderef chains)

 recommend   A. B is needed for the larger repositories, not for itoa.
```

### AD-5  What the pipeline's output is compared against

This decision concerns the full graph and detailed evidence. Expected HLD rows, collapsed paths
and roll-up counters are deferred under BD-06, not prerequisites for freezing full-map facts.

```text
 today      the reference is a source-review draft: 48 selected occurrences, 197 call-site
            candidates, status DRAFT_NOT_FROZEN. The rust-analyzer oracle export is incomplete.

 option A   reference tables written by LLM parsing of the pinned source, then reviewed
            by a person, kept outside production, each row marked observed / inferred / unresolved
 option B   reference exported by the rust-analyzer oracle harness run on itoa
 option C   A as the reference, B as a cross-check where the harness can export

 note       A is what was asked for on 2026-11-13. In every option the reference is
            comparison data only and is never read by the production pipeline.
```

## 14. Provenance and licensing

SPEC section 10 lists repositories, exact known commits, modes (copy-adapt/link-pinned/rewrite-from/
oracle/idea-only) and licenses. Copied code retains headers and third-party-license-notices.md in its
crate. cargo-modules MPL-2.0 code is idea-only, never copied.

## 15. What this document does not claim

```text
 no complete end-to-end source producer chain after name resolution has been verified
 no canonical whole-itoa full-map comparison has been made; current evidence is in STATUS
 the first semantic call remains unverified; structural/header evidence is narrower
 the itoa target dataset is an `LLM_PARSED_DRAFT` authored by Devin from pinned source; no
   independent compiler/oracle verification has frozen it, and it is not whole-source coverage
   known omissions include operators, body types, literals and repeated same-line sites
 the production pipeline remains separate from the offline reference builder and does not
   consume candidate rows; implemented subsets do not establish the complete graph contract
 OD-7 source containment is decided, but its file-module/area relationship remains open;
   OD-2 provenance, OD-4 and OD-8 remain open; projection-only choices are in the backlog
 the recommendations in section 13 are design judgement, not measurements
```
