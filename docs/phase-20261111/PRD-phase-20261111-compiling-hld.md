# PRD — phase-20261111: HLD for Rust code that compiles

Status: draft. Owner: human-architect-intent-author.
Executable contracts: SPEC-phase-20261111-executable-requirements.md.
Procedure: SOP-phase-20261111-hld-current-e2e-test.md.
Source design: devin-research/Deep-Insight-08.md, narrowed here. This phase's three document basenames
retain their phase-prefixed convention; new owned names follow AGENTS.md's four-word rule.
**M** marks observed facts; unmeasured budgets are targets, not results.

## 1. Objective

Build hld-current-dependency-edges for compiling Rust workspaces using CPU tools only, no LLMs/training.
Port rust-analyzer algorithms over persisted rows rather than host its analysis database in production.
Aim for exact extracted dependencies, report unknowns without guessing, and enforce SPEC's explicit
precision/coverage gates rather than claiming that compilation guarantees a complete graph.

D1: cargo metadata --offline. D2: all five repos equally: rust-analyzer, ripgrep, tokio, serde, cargo-modules.
D3: proc-macro-shape-table checked against real oracle expansions. D4/KL1: never execute build scripts
or read their saved target/ outputs. D5: rustc only --print cfg / --print sysroot during production.
D6: HLD nodes are non-private items another file can use, never fn-body items; reexports and external
reachability use effective visibility, not explicit pub tokens alone.

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
No std/no core and edition-specific extern preludes follow SPEC §2.3.

## 3. Product and row shape

hld-current-dependency-edges contains extracted selected-node dependencies only, written solely by
cpu-tool-hld-table-builder, never by an agent/human. Unknowns remain in evidence/site rows.

Owned column names, defined with full semantics in SPEC §3.2:

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

Relation values: calls, uses_type, implements, reexports, contains, derives.
Via values: direct/collapsed. Origin values: same_crate/workspace_crate/std/external_crate.
Every node carries node_reachable_outside_crate; external targets preserve package source and target identity,
not just crate/version. Full span identity and snapshot are recoverable through the generation/occurrence keys.

D6 selection includes file-top-level, inline-module and top-level impl/trait associated items with effective
non-private visibility, and excludes all fn-body descendants. Trait/trait-impl members and enum variants inherit
their interface visibility; inherent associated items respect their own restrictions.
External reachability additionally follows ancestor modules and public reexport aliases.
Private uses are evidence, not nodes; public/restricted reexports produce one edge per resolved binding,
including aliases/globs. SPEC §2.4 defines module/root endpoints and the private-chain collapse algorithm.
Use the nearest selected enclosing module as source, retaining a private lexical module in evidence;
the implicit externally addressable crate-root module qualifies as a node. Private-equivalent pub(self) does not.
Cycles terminate, diamonds retain provenance without duplicate HLD rows, genuine self-edges are retained,
and unresolved intermediate links never create extracted successors.
File/folder roll-ups count deduplicated HLD rows, not the number of private paths.

## 4. Architecture A and storage

Every stage reads declared tables and writes declared tables. No memory crosses stage/unit boundaries.
Local lowering/fixed-point state may live inside a single work unit; outputs must be persisted before consumers run.
Name resolution and item-position macro expansion are ONE crate fixed point; macro expansion is its library,
not a separate inter-stage in-memory channel. Expression macros expand inside the body unit.
Stage 8 is merged into stages 6/7, which write evidence directly; no hidden resolved-site buffer crosses stages.

Layers:
- L1 parseltongue-core-row-types: IDs, rows, errors; zero dependencies.
- L2 stage crates: synchronous logic over row-store traits, plus copy-adapted pure macro algorithms.
- L3 adapters: Turso runtime, parser/cargo bridges, optional next-solver adapter.
- parseltongue-cli-dispatcher-binary wires the production stages.
- oracle-semantic-export-harness is an external offline test artifact; ide/hir/salsa permitted there only.

Turso is the only database engine. Pin turso =0.9.0-pre.5; M: crates.io stable 0.8.3, maximum prerelease
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

## 5. Stage interfaces and ownership

Each entry lists declared inputs and outputs. Generation markers accompany every write.
Stage 0 bootstraps from the explicit filesystem/process boundary; ingest alone reads source bytes.
The parser adapter reads stored bytes; resolvers never read source files behind the table interface.

- Stage 0, cpu-tool-crate-graph-loader:
  reads manifests/lockfile plus allowed print/metadata commands; writes crate-dependency-graph-table,
  per-unit context and stage-run-generation-records. Verify lock presence before metadata and immutable bytes after it.
  Load proc-macro-shape-records.toml into proc-macro-shape-table as declared initialization input.
- Stage 1, cpu-tool-ingest-source-files:
  reads graph/source-root inventory and filesystem; writes file-source-text-table with raw bytes/path/digest.
  Read normal source includes through this stage when closure discovery requests them; never read target/ outputs.
- Stages 2/3, cpu-tool-syntax-tree-walker:
  reads file-source-text-table and graph contexts; writes span-reference-text-table,
  field-signature-detail-table and body-site-pending-table for active closure; records inactive encountered items.
- Stages 4+5, cpu-tool-port-name-resolver calling cpu-tool-port-macro-expander:
  reads graph/file/span/field/shape rows and completed dependency bindings;
  writes module-scope-binding-table, macro-expanded-item-table, lang-item-definition-table,
  expanded span/field/body-site rows and unresolved evidence rows. One crate is the atomic fixed-point unit.
  Closure-discovery requests return to persisted ingest/walker work; do not read unknown files directly.
  Requests and batch/final closure markers live in stage-run-generation-records, not an in-memory handoff.
- Stage 6, cpu-tool-port-signature-lowerer:
  reads graph/span/field/binding/lang-item rows; writes lowered-signature-type-table,
  impl-header-index-table and signature/impl edge-evidence-record-table.
- Stage 7, cpu-tool-port-body-inferencer:
  reads graph/file/span/field/body-site/binding/lowered-signature/impl/lang-item/shape rows;
  writes edge-evidence-record-table plus body-macro expansion/span/field/site rows within its body unit.
  7a locals/inherent/autoderef; 7b traits in scope; optional 7c next solver. No distinct stage 8.
- Stage 9, cpu-tool-hld-table-builder:
  reads graph/span/field/binding/expansion/evidence rows; writes hld-current-dependency-edges
  and hld-file-level-rollup / hld-folder-level-rollup atomically with evidence references.
- Stage 10, cpu-tool-status-coverage-banner:
  reads completed stage outputs/markers and optional oracle-edge-agreement-table; emits per-level-coverage-banner.
- Stage 11, cpu-tool-oracle-edge-differ:
  offline only; reads production rows and normalized harness artifacts; writes oracle-edge-agreement-table.
  It does not spawn analysis engines from inside a production mapping run.

## 6. Porting sources

Use pinned rust-analyzer sources from SPEC §8, retaining MIT OR Apache-2.0 headers for copied code.
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

SPEC §2.3 requires fixtures for #[path]/cfg_attr, nested/ordinary includes, hygiene/$crate,
glob precedence/ambiguity, aliases, edition-specific extern preludes and no_std/no_core.

## 7. Oracle and evidence

oracle-semantic-export-harness is built from rust-analyzer@6e37b2d62127d445d77069031b416c48c9b0b4f2,
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
The same rules apply in SPEC §2.2 and SOP; a reason never excuses an extracted wrong target.

## 9. Executable requirements

SPEC §2 is the single executable wording; its REQ-RUST-001–022 contracts are incorporated here.
Do not maintain an abbreviated conflicting WHEN/THEN copy.
They cover input/process integrity (001–004/014–015), resolution/macros (005–008/013/016–018),
D6/HLD (009/019), dependency isolation (010), generations/concurrency/cancellation (020–022),
memory (011) and deterministic export (012).

## 10. Verification and scoring

Use SPEC §4's requirement → test → SOP-step crosswalk and exact four-word test names.
Every percentage is computed separately per repo with raw numerator/denominator/exclusions.
Empty denominator is N/A and blocks a corpus gate, not 100%; empty fixtures assert zero output.
Keys include compilation unit, relative source path, byte range, site kind and expansion provenance.
005 paths ≥99.5%; 006 invocation sets ≥99%; 013 matching-shape invocations 100%;
018 relation intersection/union ≥99.5%; 007 correct/extracted method sites ≥99%;
008 correct/eligible PRIMARY method sites ≥95% mandatory at M8; 019 selection/reachability 100%.
Wrong extracted sites count against precision even with a purported reason.
SOP PT-EXTRA is site-level rate, distinct from deduplicated HLD/file-pair counts; sampled-row misses still fail.

## 11. Milestones — SPEC M0–M8 is authoritative

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

## 12. Budgets and measurements

Targets: body inference <30 minutes on 4 cores for rust-analyzer plus selected dependencies;
production process-tree peak RSS ≤2 GiB (2 × 1024³ bytes), not parent-only time -v RSS.
Measure summed resident bytes at 10 ms intervals with child accounting and record telemetry limitations;
exclude harness/indexer processes. Disk size and stage wall time are reported, not invented thresholds.
Measure Turso throughput/conflicts at 1/4/16 workers before adding caches.
No budget or correctness result is claimed achieved by this document.

## 13. Out of scope

Broken active-closure code recovery; all LLMs/training; full proc-macro execution in production;
build-script execution/output ingestion; test/bench/example/dev targets; all-feature/test-cfg exploration;
intent tables hld-future-dependency-edges and hld-delta-dependency-view; incremental remap.
Dynamic/generic calls target the applicable trait method, not a guessed runtime implementation.
Library-source reuse is compatibility-keyed, never a source-digest-only semantic cache.

## 14. Risks and decisions

D-A: external normalized semantic harness, PRIMARY gates, SECONDARY KL1 attribution, SCIP secondary.
D-B: lib+bin, test cfg OFF, per-unit contexts. D-C: ≥99% site precision, sample misses fail.
D-D: ≥95% final coverage at M8. D-E: authoritative M0–M8 and milestone SOP subsets.
D-F: copy-adapt mbe, optional span without defaults, no salsa in production.
D-G: pin async Turso prerelease, runtime owned by L3.
D-H: content digest versus semantic occurrence identity, full collision detection, Cargo target identity.
D-I: compatibility generations, atomic publication and shared-library manifests.
D-J: one reason enum/scoring policy. D-K: formulas, 8192 rounds, six total write attempts, 2 GiB process-tree RSS.
D-L: clause tests, module/visibility/collapse fixtures. D-M: executable SOP query/coverage/sampling fixes.
D-N: four-word owned identifiers/artifacts; phase document basenames retained.
KL1 remains a known limitation, not permission to exempt arbitrary oracle-proven bugs.
Unmeasured risks: solver adapter scope, macro-shape coverage, disk lookup cost, MVCC throughput and RSS.
Remaining corpus hashes are pinned at M1; harness executable hash is recorded after its pinned build.

## 15. Provenance and licensing

SPEC §8 lists repositories, exact known commits, modes (copy-adapt/link-pinned/rewrite-from/oracle/idea-only)
and licenses. Copied code retains headers and third-party-license-notices.md in its crate.
cargo-modules MPL-2.0 code is idea-only, never copied. No repository-wide AGENTS.md change is made by this phase.
