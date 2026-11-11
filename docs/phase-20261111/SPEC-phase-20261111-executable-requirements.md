# Executable specs — phase-20261111: HLD for Rust code that compiles

Status: draft · Source: `PRD-phase-20261111-compiling-hld.md` (decisions D1–D6, KL1) · Skill: rust-executable-specs (spec-first architecture)

This file turns the PRD into contracts that can be tested before code exists. Where it and the PRD disagree, the PRD's
decisions win and this file is fixed. **M** = measured; every other number is a target. No code exists yet.

```text
 §1  Rust work mode
 §2  executable requirements            REQ-RUST-<NNN>.<REV>, WHEN … THEN … SHALL
 §3  Rust design                        L1 / L2 / L3 crates, core types, trait seams, errors, concurrency
 §4  verification matrix                req → test → type → assertion → metric
 §5  implementation plan                TDD order, one milestone at a time, each with its RED tests named
 §6  quality gate results
 §7  open questions
```

## 1. Rust work mode

```text
 modes            Spec + Delivery + Reliability
 component        batch CLI (parseltongue-cli-dispatcher-binary) over a workspace of library crates; parser / analysis
 main risks       1 boundaries — which stage owns which table (architecture A: table in → table out)
                  2 memory — ≤ 2 GB peak on rust-analyzer's own repository (target)
                  3 concurrency — many workers writing one TursoDB file under MVCC, output must stay deterministic
                  4 exactness — every extracted row must agree with rust-analyzer run offline; unknowns are counted, never guessed
 fixed rules      CPU tools only (no LLM, no training) · no salsa / ra_ap_hir* / ra_ap_ide* · no cargo build / check ·
                  no build.rs (D4) · rustc only --print cfg / --print sysroot (D5) · cargo metadata --offline (D1) · TursoDB only
```

## 2. Executable requirements

Revision `.0` = as in the PRD; `.1` = tightened here (the change is stated). 014–022 are new.

```text
 ── INPUTS ──────────────────────────────────────────────────────────────────────────────────────────────────────────

 REQ-RUST-001.1  crate graph, offline, no compilation                         (.1: names every allowed process)
   WHEN cpu-tool-crate-graph-loader runs on a workspace with a Cargo.lock and downloaded dependencies
   THEN it SHALL write one crate-dependency-graph-table row per package/target in `cargo metadata --offline` output
        (name, version, edition, enabled features, root file, dependency edges with renames)
   AND  the only child processes Parseltongue SHALL spawn in the whole run are: `cargo metadata --offline`,
        `rustc --print cfg`, `rustc --print sysroot`; any other spawn SHALL be a test failure
   AND  when metadata fails (no lockfile, missing download) it SHALL return CrateGraphLoadError with the cargo stderr,
        and write nothing

 REQ-RUST-002.0  byte-exact ingest
   WHEN a file is ingested
   THEN its stored bytes SHALL equal the bytes on disk (no newline, whitespace or encoding change)
   AND  a re-run on an unchanged file (same blake3 digest) SHALL write nothing

 REQ-RUST-003.0  byte coverage
   WHEN a file of a compiling workspace is walked
   THEN item spans + classified gaps SHALL sum to the file length, sorted, contiguous, non-overlapping, 0 bytes of kind "other"

 REQ-RUST-004.1  no parse errors in scope                                     (.1: non-UTF-8 named)
   WHEN a workspace is accepted into phase-20261111
   THEN every workspace file SHALL parse with 0 parser errors
   AND  a file with parse errors or non-UTF-8 bytes SHALL stop the run with WorkspaceNotCompilingError { path, first error },
        not be recovered (recovery is the next phase)

 REQ-RUST-014.0  target cfg from rustc (D5)
   WHEN the run starts
   THEN the active cfg set SHALL be the output of `rustc --print cfg` plus the enabled features from cargo metadata,
        stored once per run with the snapshot
   AND  an item under an inactive cfg SHALL be written with status unresolved(cfg_inactive) and counted, never dropped silently

 REQ-RUST-015.0  build scripts are never run, never read (D4, KL1)
   WHEN a crate has a build.rs
   THEN Parseltongue SHALL NOT execute it and SHALL NOT read anything under target/
   AND  `include!(concat!(env!("OUT_DIR"), …))` and env!/option_env! of a variable not set by cargo metadata SHALL be
        written unresolved(build_script)
   AND  CARGO_PKG_* and CARGO_MANIFEST_DIR SHALL be filled from cargo metadata
   AND  a cfg only a build script could set SHALL be OFF
   AND  the banner SHALL report the unresolved(build_script) count per crate

 ── PORTED STAGES ───────────────────────────────────────────────────────────────────────────────────────────────────

 REQ-RUST-005.0  name resolution agreement
   WHEN cpu-tool-port-name-resolver has run on a crate
   THEN for every path in a use tree or signature, its target SHALL equal rust-analyzer's resolution in ≥ 99.5 % of paths
   AND  every disagreement SHALL be listed in oracle-edge-agreement-table with a reason

 REQ-RUST-016.0  name resolution and macro expansion share one fixed point
   WHEN a crate is resolved
   THEN imports and macro expansion SHALL run in one loop until neither adds a binding or an item
        (the shape of rust-analyzer's DefCollector::resolution_loop)
   AND  the loop SHALL stop after FIXED_POINT_LIMIT rounds with NameResolutionStuckError { crate, round, pending count }
        instead of running forever

 REQ-RUST-006.1  macro_rules! and built-in macro agreement                    (.1: built-ins and body position added;
   WHEN a macro_rules! or built-in macro invocation is expanded, at item or body position    proc macros moved to 013)
   THEN the set of (kind, name) of the produced items, and the call sites inside a body expansion, SHALL equal
        rust-analyzer's expansion for ≥ 99 % of invocations
   AND  every expanded item SHALL carry origin = macro and the invocation span id

 REQ-RUST-013.0  proc-macro shape agreement (D3)
   WHEN a proc-macro invocation matches a proc-macro-shape-table row (macro crate, version from Cargo.lock, macro name)
   THEN the (kind, name, signature) set of the items it adds SHALL equal rust-analyzer's expansion on the five repos for
        100 % of shape rows; a mismatching row SHALL fail the test with macro, version and diff
   AND  an invocation with no matching row (any other macro, or another version) SHALL be written unresolved(proc_macro)

 REQ-RUST-017.0  lang items and incoherent impls
   WHEN core / alloc / std from the sysroot are resolved
   THEN lang-item-definition-table SHALL contain every #[lang = "…"] item rust-analyzer's lang_item.rs knows that exists in
        that sysroot (M: 169 names listed in lang_item.rs)
   AND  impl-header-index-table SHALL include the #[rustc_has_incoherent_inherent_impls] impls on primitives (f32, str, [T] …)

 REQ-RUST-018.0  signature lowering agreement
   WHEN cpu-tool-port-signature-lowerer has run
   THEN every type path in a signature or impl header SHALL be resolved or unresolved with a reason
   AND  the uses_type and implements edges SHALL agree with rust-analyzer in ≥ 99.5 %

 REQ-RUST-007.1  method-call precision                                        (.1: build_script reason added)
   WHEN cpu-tool-port-body-inferencer writes a method-call site as extracted
   THEN its target SHALL equal rust-analyzer's target in ≥ 99 % of extracted method-call sites (precision first)
   AND  a site it cannot resolve SHALL be written unresolved with exactly one reason: unknown_receiver · trait_solver_needed ·
        proc_macro · build_script · cfg_inactive · external_source_missing

 REQ-RUST-008.0  method-call coverage reported per sub-step
   WHEN 7a, 7b and 7c are complete
   THEN the banner SHALL report the extracted share of method-call sites after each sub-step; target ≥ 95 % after 7c

 ── THE PRODUCT ─────────────────────────────────────────────────────────────────────────────────────────────────────

 REQ-RUST-019.0  HLD node selection (D6)
   WHEN cpu-tool-hld-table-builder selects nodes
   THEN a span SHALL be a node iff it is (a) an item at a file's top level or in an inline mod, or (b) an associated item of
        a top-level impl/trait — AND its visibility is pub, pub(crate), pub(super) or pub(in …)
   AND  an item inside a fn body SHALL never be a node, whatever its visibility
   AND  a private `use` SHALL be evidence only; a `pub use` SHALL produce a reexports edge
   AND  every node SHALL carry reachable_outside_crate = yes | no, computed from module-scope-binding-table

 REQ-RUST-009.1  HLD derivation                                               (.1: "public item" → HLD node per 019)
   WHEN cpu-tool-hld-table-builder runs
   THEN every row SHALL connect two HLD nodes (or a node and an external item)
   AND  every row with via = collapsed SHALL have evidence rows naming the private items it passed through
   AND  no row SHALL have status other than extracted

 ── RUNTIME BEHAVIOUR ───────────────────────────────────────────────────────────────────────────────────────────────

 REQ-RUST-010.1  no hidden analysis engine, small core                        (.1: L1 rule added)
   WHEN the Parseltongue binary is built
   THEN its dependency tree SHALL contain no ra_ap_hir*, ra_ap_ide*, ra_ap_base_db or salsa crate (`cargo tree`)
   AND  parseltongue-core-row-types SHALL have zero dependencies

 REQ-RUST-020.0  stages are independent and re-runnable (architecture A)
   WHEN any one stage is run again on an unchanged database
   THEN it SHALL read only its declared input tables, rewrite only its declared output tables, and produce byte-identical rows
   AND  a stage SHALL refuse to start with StageInputMissingError when an input table has no completed run marker

 REQ-RUST-021.0  concurrent writes under TursoDB MVCC
   WHEN N workers write the same database with BEGIN CONCURRENT
   THEN a write conflict SHALL be retried up to a bounded count with backoff, then surface as StoreWriteConflictError
   AND  no row SHALL be lost or duplicated (row count and digest equal to a 1-worker run)
   AND  the number of in-flight units SHALL never exceed the configured worker count (bounded, no unbounded queues)

 REQ-RUST-022.0  cancellation leaves a consistent database
   WHEN the run is interrupted (Ctrl-C / SIGTERM) during any stage
   THEN every committed unit SHALL be complete, no partial unit SHALL be visible, the stage SHALL have no completed run marker,
        and re-running SHALL finish with the same output as an uninterrupted run

 REQ-RUST-011.0  memory
   WHEN the full pipeline runs on the rust-analyzer repository (workspace + library source roots)
   THEN peak resident memory SHALL stay ≤ 2 GB (target) and SHALL be reported with the run

 REQ-RUST-012.0  determinism
   WHEN the pipeline runs twice on the same tree with the same or different thread counts
   THEN hld-current-dependency-edges SHALL be byte-identical when exported in sorted order
```

## 3. Rust design (L1 / L2 / L3 + traits)

### 3.1 Crates

```text
 layer  crate (four-word name)                 holds                                                   may depend on
 ────── ────────────────────────────────────── ─────────────────────────────────────────────────────── ───────────────────
 L1     parseltongue-core-row-types            ids, row structs, enums, error enums                    nothing
 L2     parseltongue-table-store-traits        read / write traits per table (§3.3), run markers        L1
 L2     parseltongue-stage-worker-scheduler    dependency-ordered crate scheduler, bounded worker pool  L1, L2 traits
 L2     parseltongue-port-name-resolver        stage 4+5 fixed point (copy-adapt of nameres/)           L1, L2 traits, mbe
 L2     parseltongue-port-macro-expander       macro_rules! (copy-adapt mbe), built-ins, shape table    L1, L2 traits
 L2     parseltongue-port-signature-lowerer    stage 6 (rewrite-from lower.rs)                         L1, L2 traits
 L2     parseltongue-port-body-inferencer      stage 7: body lowering, inference, method lookup        L1, L2 traits
 L2     parseltongue-hld-edge-builder          stage 9 node selection + collapse + roll-ups            L1, L2 traits
 L3     parseltongue-turso-store-adapter       implements every §3.3 trait on TursoDB                  turso (pinned)
 L3     parseltongue-syntax-span-extractor     ra_ap_syntax / ra_ap_parser → span, field, site rows    ra_ap_* (pinned)
 L3     parseltongue-crate-graph-loader        cargo metadata --offline + rustc --print cfg / sysroot  cargo_metadata
 L3     parseltongue-trait-solver-adapter      M7 only: DbInterner-shaped adapter over tables          ra-ap-rustc_next_trait_solver
 L3     parseltongue-oracle-edge-differ        offline test-only: compares rows with rust-analyzer      dev-dependency only
 bin    parseltongue-cli-dispatcher-binary     one verb → one tool; wires L3 adapters into L2 stages    all
```

Rule: L2 stage crates never name TursoDB, ra_ap_syntax or cargo; they see traits only, so every stage is unit-tested
with an in-memory fake store (`MemoryRowStoreFake`, test-only). REQ-RUST-010 is checked by a `cargo tree` gate.

### 3.2 L1 core types (invalid states unrepresentable)

```rust
pub struct SpanDigestIdValue([u8; 16]);          // blake3(file digest, start, end, kind) truncated — identity is bytes (REQ 003, 012)
pub struct FileContentDigestValue([u8; 32]);
pub struct CrateDenseIndexValue(u32);              // dense index assigned in sorted (name, version) order → deterministic
pub struct SpanByteRangeValue { start: u32, end: u32 }   // constructor rejects start > end
pub struct ItemQualifiedPathText(Box<str>);        // crate::module::Name; external items keep crate@version

pub enum NameResolutionNamespaceKind { Types, Values, Macros }
pub enum SpanDeclaredVisibilityKind { Private, Public, PubCrate, PubSuper, PubIn(ItemQualifiedPathText) }
pub enum HldNodePlacementKind { FileTopLevel, InlineModule, TopLevelAssocItem, InsideFnBody }   // REQ 019
pub enum EdgeTargetReferenceValue { Span(SpanDigestIdValue), External { krate: ItemQualifiedPathText, item: ItemQualifiedPathText } }
pub enum HldEdgeRelationKind { Calls, UsesType, Implements, Reexports, Contains, Derives }
pub enum HldEdgeViaKind { Direct, Collapsed }
pub enum HldEdgeOriginKind { SameCrate, WorkspaceCrate, Std, ExternalCrate }

pub enum SiteUnresolvedReasonKind {           // the ONLY reasons allowed (REQ 007, 014, 015, 013)
    UnknownReceiver, TraitSolverNeeded, ProcMacro, BuildScript, CfgInactive, ExternalSourceMissing,
}
pub enum BodySiteResolutionOutcome {             // no "maybe": a site is one or the other
    Extracted { target: EdgeTargetReferenceValue, via_trait: Option<ItemQualifiedPathText> },
    Unresolved(SiteUnresolvedReasonKind),
}
```

`HldNodeCandidateSpan::try_from(span) -> Option<HldSelectedNodeRow>` is the single place that applies REQ 019, so a fn-body item cannot
become a node by construction.

### 3.3 Trait seams (L2), one pair per table

```text
 trait                              methods (shape)                                          used by
 ────────────────────────────────── ──────────────────────────────────────────────────────── ─────────────────────────
 CrateGraphReadStore                crates_in_dependency_order() · deps_of(CrateDenseIndexValue)          scheduler, name resolver
 SpanRowReadStore                   spans_of_file / spans_of_crate / span(SpanDigestIdValue)             every stage after 2
 FieldDetailReadStore               fields_of(SpanDigestIdValue)                                        name resolver, lowerer
 BodySiteReadStore                  sites_of_body(SpanDigestIdValue)                                    body inferencer
 ScopeBindingReadStore              resolve(CrateDenseIndexValue, module, name, NameResolutionNamespaceKind) -> EdgeTargetReferenceValue  lowerer, inferencer
 ImplHeaderLookupIndex              impls_for_self_head(head) · impls_of_trait(trait)        inferencer, solver adapter
 LangItemReadStore                  lang_item(name) -> Option<SpanDigestIdValue>                         inferencer
 ProcMacroShapeLookup               shape(macro crate, version, name) -> Option<ProcMacroShapeRow>   macro expander
 StageRowWriteSink<Row>             write_unit(WorkUnitKeyValue, Vec<Row>) — one transaction per unit  every stage
 StageRunMarkerStore                begin(stage) · complete(stage) · is_complete(stage)       REQ 020, 022
```

Writes are per unit of work (one file, one crate, one fn body) and atomic: the unit's rows commit together or not at all
(REQ 022). Rows are sorted by key inside the unit, and ids are content digests, so commit order cannot change the output
(REQ 012, 021).

### 3.4 Errors (typed, one enum per boundary, no `anyhow` in libraries)

```text
 CrateGraphLoadError            metadata failed · rustc print failed · forbidden process requested
 WorkspaceNotCompilingError     path + first parser error | non-UTF-8 at byte offset
 NameResolutionStuckError       crate, round, pending imports/macros
 StageInputMissingError         stage, missing table
 StoreWriteConflictError        table, unit key, attempts
 StoreBackendFailureError       wraps the Turso error at the L3 boundary only
```

Unresolved sites are rows, not errors: an error stops the run, an unresolved site is data and is counted.

### 3.5 Concurrency and cancellation

```text
 workers          fixed pool of N threads (default = cores); a unit of work is one file / crate / fn body
 name resolution  crate starts only when all its deps are complete; independent crates run in parallel
 backpressure     at most N units in flight; a worker writes its own unit (no shared unbounded queue)
 TursoDB          BEGIN CONCURRENT per unit; conflict → bounded retry with backoff (REQ 021)
 async boundary   if the pinned turso crate exposes an async API, the L3 adapter owns one runtime and offers blocking trait
                  methods; L2 stays synchronous, so no lock or guard is ever held across .await (to check in M1, Q1)
 cancellation     one shared RunCancelFlagShared checked between units; an interrupted unit is rolled back; the stage marker is not
                  completed (REQ 022)
 unsafe           none planned; any unsafe needs a written justification and a test
```

## 4. Verification matrix

```text
 req       test                                       type                         assertion                                         metric
 ───────── ────────────────────────────────────────── ──────────────────────────── ───────────────────────────────────────────────── ─────────────────
 001.1     test_crate_graph_offline_only              integration                  rows = metadata packages; spawned set ⊆ 3 allowed  exact
 001.1     test_metadata_failure_writes_nothing        integration                  no lockfile → CrateGraphLoadError, 0 rows          —
 002.0     test_ingest_bytes_stay_identical           property (proptest)          stored == disk for random bytes                    100 %
 002.0     test_unchanged_digest_skips_write          unit                         second run writes 0 rows                           0
 003.0     test_span_gap_tiling_identity              integration, corpus          tiling identity on every file                      0 failures
 004.1     test_parse_error_stops_run                 unit                         error file → WorkspaceNotCompilingError(path)      —
 004.1     test_non_utf8_stops_run               unit                         invalid UTF-8 → error with byte offset             —
 014.0     test_cfg_from_rustc_print              integration                  active cfg = rustc --print cfg + features          exact
 014.0     test_inactive_cfg_items_counted            unit                         #[cfg(any())] item → unresolved(cfg_inactive)      counted
 015.0     test_build_script_never_executed           integration                  fixture build.rs that writes a file → file absent  0 runs
 015.0     test_out_dir_include_unresolved            unit                         include!(…OUT_DIR…) → unresolved(build_script)     —
 015.0     test_cargo_env_from_metadata           unit                         env!("CARGO_PKG_VERSION") = manifest version       exact
 005.0     test_nameres_matches_oracle_paths          integration + oracle         port target == rust-analyzer target                ≥ 99.5 %
 016.0     test_macro_import_fixed_point         unit                         macro-generated `use` resolves in the same loop    —
 016.0     test_fixed_point_limit_errors              unit                         self-feeding macro → NameResolutionStuckError      —
 006.1     test_macro_rules_items_match               integration + oracle         expanded (kind, name) set equal                    ≥ 99 %
 006.1     test_body_macro_sites_match                integration + oracle         call sites inside format!/vec!/assert! equal       ≥ 99 %
 013.0     test_macro_shapes_match_oracle        integration + oracle         shape (kind, name, signature) set equal            100 % of rows
 013.0     test_unknown_proc_macro_unresolved         unit                         no shape row / other version → unresolved          —
 017.0     test_lang_items_in_sysroot           integration                  every lang_item.rs name present in sysroot found   100 %
 017.0     test_incoherent_primitive_impls_indexed    integration                  f32::abs, str::len resolve to std impls            exact
 018.0     test_signature_edges_match_oracle          integration + oracle         uses_type / implements edges equal                 ≥ 99.5 %
 007.1     test_method_target_precision_oracle        integration + oracle         extracted target == oracle target                  ≥ 99 %
 007.1     test_unresolved_has_one_reason             property                     every unresolved row has exactly one reason        100 %
 008.0     test_method_coverage_per_substep           benchmark report             extracted share after 7a / 7b / 7c                 ≥ 95 % after 7c
 019.0     test_body_items_never_nodes             unit                         `pub fn` inside a fn body → not a node             —
 019.0     test_pub_use_reexport_edge           unit                         `pub use a::B` → reexports edge; `use` → evidence  —
 019.0     test_reachable_outside_crate_flag          integration + oracle         flag == item reachable in rust-analyzer            100 %
 009.1     test_hld_rows_connect_nodes                integration                  both ends nodes; collapsed rows have evidence      100 %
 010.1     test_no_analysis_crates_linked             build gate (cargo tree)      forbidden crates absent; L1 has 0 deps             0
 020.0     test_stage_rerun_is_identical              integration                  re-run one stage → byte-identical output tables    identical
 020.0     test_stage_refuses_missing_input           unit                         no input marker → StageInputMissingError           —
 021.0     test_concurrent_writers_lose_nothing       integration (N = 1, 4, 16)   row count + digest equal to N = 1                  equal
 021.0     test_conflict_retry_is_bounded             unit (fault-injecting fake)  always-conflict store → error after K attempts     K attempts
 021.0     test_in_flight_units_bounded               unit                         observed max in-flight ≤ N                         ≤ N
 022.0     test_cancel_then_rerun_identical           integration                  cancel at random unit → rerun output == clean run  identical
 011.0     test_peak_memory_within_budget             benchmark (/usr/bin/time -v) peak RSS on rust-analyzer repo                     ≤ 2 GB target
 012.0     test_hld_export_is_deterministic           integration                  sorted export equal for 1 and N threads, twice     byte-identical
 gates     cargo fmt --check · cargo clippy --all-targets -- -D warnings · cargo test · cargo build --release · cargo tree gate
```

Oracle tests run rust-analyzer offline on the five repos (rust-analyzer, ripgrep, tokio, serde, cargo-modules) and write
oracle-edge-agreement-table; they are slow and run as a separate `--ignored` suite plus nightly, never inside the binary.

## 5. Implementation plan (TDD: RED test first, then GREEN, then refactor)

```text
 M0  workspace skeleton       crates of §3.1 empty; L1 types; MemoryRowStoreFake
                              RED: test_no_analysis_crates_linked, test_unresolved_has_one_reason
 M1  inputs                   crate graph loader, ingest, span extractor, Turso adapter, run markers, worker pool
                              RED: 001 · 002 · 003 · 004 · 014 · 015 · 020 · 021 · 022 tests
                              also measure: Turso write throughput at N = 1/4/16 (R6), file/span counts → published
 M2  name resolution + macros stage 4+5 fixed point, macro_rules!, built-ins, lang items
                              RED: 005 · 016 · 006 · 017 tests
 M3  proc-macro shapes        shape table for serde, salsa, thiserror, tokio main/test, rust-analyzer's derives
                              RED: 013 tests
 M4  signatures + impl index  RED: 018 · test_incoherent_primitive_impls_indexed
 M5  body 7a                  body lowering + locals + inherent methods + autoderef        RED: 007 · 008 (7a)
 M6  body 7b                  trait methods via traits in scope                             RED: 008 (7b)
                              decision point: ≥ 95 % after 7b on all five → M7 optional
 M7  body 7c                  trait-solver adapter (R1)                                     RED: 008 (7c), 010 still green
 M8  HLD                      node selection, collapse, roll-ups, banner                    RED: 019 · 009 · 011 · 012
```

Every milestone ends with all gates in §4 green for the requirements it owns, and the measured numbers written into the
PRD as **M**.

## 6. Quality gate results

```text
 cargo fmt / clippy / test / build    not run — no code exists yet; this is a specification
 requirement coverage                 every REQ in §2 has ≥ 1 test in §4 (checked by hand)
```

## 7. Open questions

```text
 Q1  the pinned turso crate's API: async or blocking? If async, §3.5's "runtime inside the L3 adapter" applies; to check
     in M0 against the pinned version
 Q2  macro_rules! expansion: copy-adapt crates/mbe, or link ra_ap_mbe? Linking is allowed only if its dependency tree has
     no salsa (ra_ap_span may pull it in — unchecked); decide in M2 with `cargo tree`
 Q3  where proc-macro-shape-table's source lives: a checked-in data file (e.g. TOML) loaded into the table at run start is
     proposed, so humans edit text and REQ 013 tests it
 Q4  cfg set: one run = default features only (PRD §13). Is a second run with --all-features wanted for the five repos?
```

## 8. Repositories we take from

How each repository is used: **copy-adapt** = code copied, licence headers and attribution kept, inputs rewired to our
tables · **link-pinned** = published crate linked unchanged at one version · **rewrite-from** = our own code following its
structure, nothing copied · **oracle** = run offline only, in tests, as the answer key · **idea-only** = design studied,
no code copied or linked · **tooling** = used by agents working on this repo, never part of the Parseltongue binary.
Commits marked (M) are the checkouts on this VM on 2026-11-05; "pin at M0/M1" means the exact commit is fixed then.

```text
 ── CODE WE COPY OR LINK ────────────────────────────────────────────────────────────────────────────────────────────

 rust-lang/rust-analyzer          https://github.com/rust-lang/rust-analyzer        MIT OR Apache-2.0   commit 6e37b2d62 (M)
   copy-adapt     crates/hir-def/src/nameres/ (collector.rs resolution_loop, path_resolution.rs) → stage 4+5
                  crates/mbe/src/ (expander.rs, transcriber.rs) → macro_rules! (unless ra_ap_mbe links without salsa, Q2)
                  crates/hir-expand/src/builtin/ (fn_macro.rs, attr_macro.rs, derive) → built-in macros
                  crates/hir-def/src/lang_item.rs → lang-item-definition-table names
   rewrite-from   crates/hir-def/src/expr_store/lower.rs (body lowering) · crates/hir-ty/src/lower.rs (signatures) ·
                  crates/hir-ty/src/infer/ + method_resolution.rs (inference, method lookup) ·
                  crates/hir-ty/src/next_solver/interner.rs (shape of the solver adapter, M7)
                  crates/project-model/ (how rustc --print cfg / sysroot and cargo metadata are read)
                  crates/ide-db/src/prime_caches.rs (dependency-ordered parallel crate scheduling)
   link-pinned    ra_ap_syntax · ra_ap_parser · ra-ap-rustc_lexer (parser up to the span table), the versions that
                  rust-analyzer@6e37b2d62's Cargo.toml uses (M: ra-ap-rustc_lexer 0.180; ra_ap_syntax 0.0.362 in registry)
   oracle         rust-analyzer binary at the same commit, offline, for every *_matches_oracle test
   never          ra_ap_hir*, ra_ap_ide*, ra_ap_base_db, salsa, the proc-macro server (REQ 010)

 rust-lang/rust                   https://github.com/rust-lang/rust                 MIT OR Apache-2.0   pin at M7
   link-pinned    rustc_type_ir + rustc_next_trait_solver, as published ra-ap-rustc_type_ir / ra-ap-rustc_next_trait_solver
                  (M7 only, the versions rust-analyzer@6e37b2d62 uses)
   input          the toolchain's library/ sources (core, alloc, std) found via `rustc --print sysroot`; parsed, not compiled

 tursodatabase/turso              https://github.com/tursodatabase/turso            MIT                 pin at M0
   link-pinned    crate `turso` (TursoDB, BEGIN CONCURRENT / MVCC) — the only database (PRD §4.2)

 oli-obk/cargo_metadata           https://github.com/oli-obk/cargo_metadata          MIT                 pin at M0
   link-pinned    typed parsing of `cargo metadata --offline` JSON (M: 0.19.2 and 0.23.1 in registry)

 BLAKE3-team/BLAKE3               https://github.com/BLAKE3-team/BLAKE3              CC0 / Apache-2.0    pin at M0
   link-pinned    crate `blake3` for file digests and SpanDigestIdValue

 proptest-rs/proptest             https://github.com/proptest-rs/proptest            MIT OR Apache-2.0   dev-dependency
   link-pinned    property tests (REQ 002, 007)

 ── ORACLE CORPUS (each passes `cargo check` once when chosen; Parseltongue never builds them) ────────────────────────

 rust-lang/rust-analyzer          https://github.com/rust-lang/rust-analyzer        commit 6e37b2d62 (M)
 BurntSushi/ripgrep               https://github.com/BurntSushi/ripgrep              pin at M1
 tokio-rs/tokio                   https://github.com/tokio-rs/tokio                  pin at M1
 serde-rs/serde                   https://github.com/serde-rs/serde                  pin at M1
 regexident/cargo-modules         https://github.com/regexident/cargo-modules        commit 48455ca59 (M)

 ── IDEAS ONLY (studied; no code copied or linked) ──────────────────────────────────────────────────────────────────

 regexident/cargo-modules         MPL-2.0 (file-level copyleft → we do NOT copy its code)
   idea-only      module tree and visibility rendering on top of rust-analyzer → how D6 nodes and
                  reachable_outside_crate are shown
 cargo-public-api/cargo-public-api https://github.com/cargo-public-api/cargo-public-api  MIT    commit 68b218915 (M)
   idea-only      what "the public API of a crate" is, as rustdoc JSON lists it → cross-check for reachable_outside_crate
 obi1kenobi/cargo-semver-checks   https://github.com/obi1kenobi/cargo-semver-checks  MIT OR Apache-2.0  commit 161450e93 (M)
   idea-only      queries over public items and re-exports (import paths) → pub use / reexports edge rules
 praezi/rust (RustPräzi)          https://github.com/praezi/rust                     MIT OR Apache-2.0  commit ec5aa0ed2 (M)
   idea-only      crate-level call graphs over a whole ecosystem → evidence rows and the public graph feed later
 willcrichton/flowistry           https://github.com/willcrichton/flowistry          MIT               commit 693ceda92 (M)
   idea-only      information flow inside bodies; needs a rustc driver (a build) → not usable this phase; noted for later

 ── TOOLING (agents only, never in the binary) ──────────────────────────────────────────────────────────────────────

 DeusData/codebase-memory-mcp     https://github.com/DeusData/codebase-memory-mcp    MIT
   tooling        structural exploration of rust-analyzer while porting (AGENTS.md)
 that-in-rust/agent-room-of-requirements  commit 117afc24b — the Rust skills installed in .agents/skills/
```

Rule: anything copied keeps its original licence header and an entry in a `THIRD-PARTY-NOTICES` file in the copying crate;
MPL-2.0 code (cargo-modules) is never copied.
