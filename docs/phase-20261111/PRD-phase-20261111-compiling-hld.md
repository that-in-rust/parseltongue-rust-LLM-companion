# PRD — phase-20261111: HLD for Rust code that compiles

Status: draft · Executable specs: `SPEC-phase-20261111-executable-requirements.md` · E2E test SOP: `SOP-phase-20261111-hld-current-e2e-test.md` (run on all five repos) · Scope owner: human-architect-intent-author · Source design: `devin-research/Deep-Insight-08.md` (copied and narrowed here; this file stands alone)

## 0. How to read this document

```text
 §1  the phase in one paragraph        what is in, what is out, why this cut
 §2  glossary                          terms used below, defined once
 §3  the product                       the one table this phase must produce, and its row shape
 §4  the one design decision           parse once with the pinned parser, then run PORTED rust-analyzer logic on our tables;
                                       §4.1 architecture (table in → table out), §4.2 TursoDB constraint
 §5  the journey                       one line per step, codebase → files → spans → … → HLD
 §6  what is ported, from where        each rust-analyzer stage, its source location, what it reads, what we build
 §7  tables                            every table and who writes it
 §8  CPU tools                         every tool, four-word names
 §9  executable requirements           REQ-RUST-* contracts, WHEN … THEN … SHALL
 §10 verification matrix               req → test → assertion → metric
 §11 milestones                        the order of work, each ends with a measured gate
 §12 budgets                           time and memory targets (targets, not measurements)
 §13 out of scope                      what this phase deliberately does not do
 §14 risks and open questions
 §15 repositories we take from        pointer to the full list in the SPEC file
```

Conventions: four-word hyphenated names for every named thing (AGENTS.md). **M** = measured in this project; every other number is a target or an estimate and says so. `→` = "produces".

## 1. The phase in one paragraph

**Objective: a complete, exact `hld-current-dependency-edges` for a repository that compiles. Code that does not compile is the next phase, which builds on these tables (§13).**

Phase-20261111 builds `hld-current-dependency-edges` — public item → public item dependency rows — for **Rust workspaces that compile** (rust-analyzer itself, ripgrep, tokio-sized repositories). Because the code compiles, every file parses without error, every name has exactly one meaning, and rust-analyzer run offline gives an exact answer key. That removes the three hardest parts of the full design — broken-file recovery, grey spans, and guessing — and lets this phase answer one question cleanly: **can rust-analyzer's resolution logic, ported to run over our span and field tables one crate and one body at a time, reproduce rust-analyzer's dependency edges at a fraction of its memory?** **Every step is a CPU tool: no LLM of any size, no model training or post-training, no labels.** Anything the port cannot resolve is written as `unresolved` and counted — never guessed.

## 2. Glossary

```text
 workspace that compiles   `cargo check` succeeds on it (checked once, offline, when the corpus is chosen — never by Parseltongue)
 file row                  one file's bytes stored unchanged, plus a blake3 digest
 span                      one item (fn, struct, enum, trait, impl, const, static, type, use, mod, macro_rules, macro call) as
                           (file, start byte, end byte) — identity is bytes, never line/column
 fields                    the parts of a signature: params, return, generics, bounds, where-clauses, struct fields, variants,
                           trait items, impl header, attributes, cfg, docs
 body site                 a call, method call, path expression, construction, field access, operator, `?` inside a fn body
 library source root       source code Parseltongue reads but does not map as "yours": the toolchain's std/core/alloc sources
                           (`rustc --print sysroot`/lib/rustlib/src) and the dependency sources Cargo already downloaded
                           (~/.cargo/registry/src, git checkouts)
 crate graph               which crate depends on which, with features and editions — read from `cargo metadata --offline`
 module scope              for one module: every name visible in it (items, imports, globs, re-exports, prelude) → its target
 def map                   rust-analyzer's term for all module scopes of one crate; we call the table module-scope-binding-table
 inference                 working out the type of every expression in one fn body
 method lookup             given a receiver type and a method name: which impl's method is called (autoderef, autoref, traits in scope)
 trait solver              answers "does type X implement trait Y, via which impl, with which associated types"
 oracle                    rust-analyzer run offline on the same workspace; its answers are the ground truth for this phase
 extracted · unresolved    the two edge statuses in this phase: resolved by the port / not resolved (with a reason code)
 HLD node (D6)             an item another FILE can use: an item at a file's top level (or an inline `mod` in it), or an
                           associated item of a top-level impl/trait, with visibility pub, pub(crate), pub(super) or pub(in …).
                           Never a node: anything inside a fn body (even if `pub`); private top-level items (collapsed).
                           Private `use` = evidence of a file's outgoing dependencies, not a node; `pub use` = reexports edge.
                           Column `reachable_outside_crate` (yes/no) separates the crate's external API from the whole map
 HLD                       node → node edges with private chains collapsed (node A → private h → node B gives A → B)
```

## 3. The product

`hld-current-dependency-edges` — one row per (public item A depends on public item B):

```text
 column          meaning
 ─────────────── ──────────────────────────────────────────────────────────────────────────────────────
 node01_span_id  digest id of A's span          node02_span_id   digest id of B's span (or external item id)
 node01_sig      assembled signature of A       node02_sig       assembled signature of B
 node01_path     crate::module::Name of A       node02_path      same for B; external items keep crate@version
 edge_kind       calls · uses_type · implements · reexports · contains · derives
 via             direct | collapsed (through private items; the evidence rows say which)
 evidence_count  number of evidence rows behind this edge
 origin          same_crate · workspace_crate · std · external_crate
 status          extracted (this phase never writes inferred)
 snapshot        tree digest the row was derived from
```

Written only by `cpu-tool-hld-table-builder`. Never by ext-llm-coding-agent or a human. `hld-future-dependency-edges` and `hld-delta-dependency-view` keep their DI-08 definitions and are not built in this phase (§13).

## 4. The one design decision

**Parse every file once with the pinned parser library (`ra_ap_syntax` + `ra_ap_parser` on `ra-ap-rustc_lexer`), store spans and fields as rows, then run rust-analyzer's resolution algorithms — ported, not hosted — over those rows: name resolution per crate, `macro_rules!` expansion, signature lowering, body inference with method lookup, and trait solving. Results are written back as rows; nothing is kept in memory between crates or between bodies except what the current unit needs.**

Why port instead of running rust-analyzer: rust-analyzer's memory comes from its salsa database keeping every query result alive for the whole session (M, earlier in this research: 42k-line workspace, cold, 29 s, 2.3 GB). The algorithms themselves need only their inputs. Our inputs are already rows on disk.

What "ported" means, per stage (decided in §6):

```text
 copy-adapt       rust-analyzer's code is copied (MIT/Apache-2.0, attribution kept) and its inputs rewired from salsa
                  queries to table reads. Used where the code is self-contained: name resolution, macro_rules! expansion
 link-pinned      a published crate is linked unchanged, version-pinned: ra_ap_syntax, ra_ap_parser, ra-ap-rustc_lexer, and —
                  for the trait solver — ra-ap-rustc_type_ir + ra-ap-rustc_next_trait_solver (rustc's own solver)
 rewrite-from     our own implementation following rust-analyzer's structure, where its code is too entangled with salsa
                  and hir types to copy: signature lowering, body inference, method lookup, the solver adapter
```

Hard rules kept from DI-08: no rust-analyzer server, no rust-analyzer analysis database (`ra_ap_hir*`, `ra_ap_ide*`, salsa) linked into Parseltongue, no `cargo build`/`cargo check`, no proc-macro execution at runtime. **Build scripts (`build.rs`) are never run (D4)** — not by Parseltongue, not via `cargo check`, and their saved results in `target/` are not read either. Two print-only rustc calls are allowed (D5): `rustc --print cfg` (target cfg: unix, target_os, target_pointer_width …) and `rustc --print sysroot` (std source root). `cargo metadata --offline` is the decided source of the crate graph (decided 2026-11-05): it reads manifests and the lockfile, compiles nothing.

### 4.1 Architecture: every stage is table in → table out (option A, decided 2026-11-05)

```text
 up to the span table     rust-analyzer's parser crates as a pinned library: ra-ap-rustc_lexer, ra_ap_parser, ra_ap_syntax
                          (+ `cargo metadata --offline` for the crate graph) → file, span, field and body-site rows
 from the span table on   the WHOLE HLD is built from the tables: name resolution, macro_rules! expansion, signature lowering,
                          body inference, HLD derivation each read their input tables and write their output tables
 between stages           nothing is held in memory across stages; the tables are the only interface, so every stage can be
                          re-run, inspected and scored against rust-analyzer on its own
 parallelism              files, items and fn bodies: fully parallel workers reading and writing the same database;
                          name resolution: crates in dependency order, independent crates in parallel (Rust's rule, not ours)
 in-memory caches         none by default; a per-crate read cache for body inference is added ONLY if the body-inference step
                          misses its budget (§12), and the decision is recorded with the measurement
```

Layers:

```text
 L1 core      row and id types (SpanId, CrateId, ItemPath, EdgeStatus, UnresolvedReason), typed errors per stage; no dependencies
 L2 std       stage logic written against row-store traits; dependency-ordered crate scheduler; bounded worker pool
 L3 external  Turso adapter · ra_ap_syntax / ra_ap_parser / ra-ap-rustc_lexer · cargo_metadata · (M7 only) ra-ap-rustc_next_trait_solver
```

### 4.2 Constraint: the database is TursoDB (decided 2026-11-05)

```text
 engine          TursoDB — in-process SQL database written in Rust, SQLite-compatible (dialect and file format); Rust crate `turso`
 why             concurrent writes and reads: `BEGIN CONCURRENT` uses MVCC so several workers can write in parallel, and readers do
                 not block writers; this is what lets every stage in §4.1 run its workers in parallel against one file
 file            one database file per mapped workspace (plus one shared file for library source roots, reused by digest)
 rule            no other database (no CozoDB, no RocksDB, no rusqlite/SQLite C library) in phase-20261111
 status (checked 2026-11-05, upstream README)  `BEGIN CONCURRENT` is listed as a feature, not experimental; Turso is pre-1.0
                 (crates.io max version 0.9.0-pre.5) → pin one version for the phase; write conflicts under MVCC are retried
                 by the worker, and REQ-RUST-012 (determinism) must hold regardless of commit order
```

## 5. The journey — one line per step

```text
 0  workspace   → crate graph        `cargo metadata --offline` → crates, deps, features, editions, cfg, source roots
 1  source roots → file rows         every .rs file of the workspace AND of the library source roots it depends on is stored byte-exact
                                     with a digest; library files are tagged origin std / external_crate; unchanged digests are skipped
 2  file        → spans + fields     one parse per file → span rows (kind, name, visibility, byte range, parent) and field rows;
                                     every byte is either in a span or in a classified gap (M: 3,370 files, 0 identity failures)
 3  span        → body sites         every call, method call, path, construction, field access, operator and `?` inside a body
                                     → a pending site row with its byte range
 4+5 crate     → module scopes      ONE PORTED fixed-point per crate, in dependency order: imports and macro expansion are
                    + expanded items   resolved in the same loop (an expanded macro can add items and imports; a new import can
                                     make a macro resolvable) → module-scope-binding-table + macro-expanded-item-table;
                                     macro_rules! and built-in macros expanded; proc macros via proc-macro-shape-table (D3), else unresolved(proc_macro)
 6  signature   → lowered types      every fn/struct/impl/trait signature's type paths resolved through module scopes → lowered
                                     signature rows; every impl header → impl-header-index-table (self type, trait, generics, where)
 7  body        → resolved sites     PORTED body lowering (desugar for / ? / .await, expand body macros) then inference per fn body: local types, autoderef, method lookup over impl-header-index-table,
                                     trait solving for bounds/associated types → each site extracted with its target, or unresolved
 8  sites       → evidence edges     every resolved site and signature reference → edge-evidence-record-table
 9  evidence    → HLD                public → public rows, private chains collapsed, origin tagged → hld-current-dependency-edges;
                                     file and folder roll-ups by trigger
 10 tables      → banner             per-level coverage, resolved share per site kind, unresolved by reason, oracle agreement when known
 11 offline     → score              cpu-tool-oracle-edge-differ compares step 8–9 rows with rust-analyzer's answers on the same tree
```

## 6. What is ported, from where

Source locations are from the rust-analyzer checkout indexed with codebase-memory-mcp (project `home-ubuntu-repos-rust-tools-rust-analyzer`, 38,467 nodes, 229,944 edges); line counts exclude tests (M).

```text
 STAGES 4+5 ARE ONE STAGE. rust-analyzer's DefCollector::resolution_loop (collector.rs) nests 'resolve_imports inside
 'resolve_macros inside 'resolve_attr: imports reach a fixed point, then macros are expanded, then imports again, until both
 settle. So macro expansion is called FROM name resolution as a library; it is not a separate table-in/table-out stage.

 STAGE 4 — NAME RESOLUTION (calls stage 5 inside its loop)       mode: copy-adapt
   rust-analyzer   crates/hir-def/src/nameres/ (4.5k lines)
                   collector.rs: collect_defs (l.63), DefCollector::resolve_imports (l.792), resolve_macros (l.1306)
                   path_resolution.rs: resolve_path_fp_with_macro (l.154) · entry: nameres.rs crate_def_map (l.384)
   reads           mod tree, use trees, item names + visibility, extern crates, preludes, macro_rules! defs, crate graph
   from our rows   span-reference-text-table, field-signature-detail-table (use trees), crate-dependency-graph-table
   writes          module-scope-binding-table: (crate, module path, name, namespace) → target span id | external item id
   unit of work    one crate; crates in dependency order; independent crates in parallel (same scheduling idea as
                   ide-db/src/prime_caches.rs parallel_prime_caches)
   why exact       name resolution is a fixed-point over imports inside one crate plus finished def maps of its deps; every
                   input is a row

 STAGE 5 — MACRO_RULES! EXPANSION                                mode: copy-adapt (or link ra_ap_mbe if it builds standalone)
   rust-analyzer   crates/mbe/src/ (3.0k lines): expander.rs expand_rules (l.17), expander/transcriber.rs transcribe (l.124)
   reads           macro definition span + invocation token tree; called by stage 4 for item-position calls and by stage 7
                   body lowering for expression/statement-position calls (format!, vec!, assert_eq!, matches!, write! …)
   writes          macro-expanded-item-table: expanded text is parsed with the same parser; its items become spans with
                   origin = macro, pointing back at the invocation span
   built-in        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)] → a fixed table of the impls
                   they generate
   proc macros     (D3) proc-macro-shape-table: per (macro crate, version, macro name) the SHAPE of what it adds — items,
                   impls, methods and their signatures — not the full expansion. E.g. derive(Serialize) → impl Serialize for T;
                   salsa::input struct S → S::new + one getter per field; salsa::tracked fn f → f stays callable.
                   Versions come from Cargo.lock. Every shape is checked against rust-analyzer's real expansion on the
                   five repos (REQ-RUST-013). First set: serde, salsa, thiserror, tokio::main/test, derives used in
                   rust-analyzer itself. A macro without a shape row → unresolved(proc_macro), counted
   built-in fn-like hir-expand/src/builtin/fn_macro.rs registers 29: format_args, concat, stringify, include, include_str,
                   include_bytes, env, option_env, cfg, cfg_select, line, column, file, module_path, assert, panic,
                   unreachable, compile_error, asm … — ported as written; include!/include_str! read files through
                   file-source-text-table; env!/option_env! need values (see Q4)
   built-in attrs  attr_macro.rs: derive, derive_const, test, bench, test_case, cfg_accessible, cfg_eval,
                   global_allocator, define_opaque — ported as written
   not ported      hir-expand's proc-macro server (needs compiled .so files → a build)

 STAGE 6 — SIGNATURE LOWERING AND IMPL INDEX                     mode: rewrite-from
   rust-analyzer   crates/hir-ty/src/lower.rs: impl_self_ty_query (l.1825), callable_item_signature (l.3041),
                   generic predicates; method_resolution.rs: TraitImpls/InherentImpls for_crate (l.578, l.694)
   reads           field rows + module-scope-binding-table
   writes          lowered-signature-type-table (every type path → resolved item), impl-header-index-table
                   (impl span, self-type head, trait, generics, where-clauses, associated type bindings)
   why rewrite     lower.rs is tied to salsa ids and hir-def types; the logic (resolve each path in its scope) is simple

 STAGE 7 — BODY INFERENCE, METHOD LOOKUP, TRAIT SOLVING          mode: rewrite-from + link-pinned solver
   rust-analyzer   crates/hir-ty/src/infer/ (14.5k lines): infer.rs infer_query (l.120), expr.rs infer_method_call (l.1773),
                   autoderef.rs; method_resolution.rs lookup_method_including_private (l.136) (3.7k lines with
                   method_resolution/); next_solver/ (19.8k lines) = adapter onto ra-ap-rustc_next_trait_solver + rustc_type_ir
   step 0          BODY LOWERING — hir-def/src/expr_store/lower.rs (+ lower/, ~5.2k lines): collect_for_loop (IntoIterator /
                   Iterator::next), collect_try_operator (Try::branch), desugar_try_block, .await, coroutines, and
                   collect_macro_call for body macros. Inference runs on the lowered body, not on syntax. The lowered body
                   lives only inside the one-body unit of work (it is not a table)
   reads           one fn body's syntax (re-parsed from its span), lowered signatures of everything it touches,
                   impl-header-index-table rows for the receiver types it meets, traits in scope at the body (from
                   module-scope-binding-table), lang-item-definition-table, and infer.rs's other inputs (trait environment
                   = where-clauses of the enclosing items, value types, impl self types, target data layout)
   lang items      autoderef (Deref), for (IntoIterator), ? (Try), operators (Add …), closures (Fn*) are found through
                   #[lang = "…"] items in core/alloc/std; hir-def/src/lang_item.rs lists 169 → lang-item-definition-table
   incoherent      std puts inherent impls on primitives (f32, str, slices) via #[rustc_has_incoherent_inherent_impls];
                   method_resolution.rs with_incoherent_inherent_impls searches sysroot crates for them → impl index must
                   include them
   solver adapter  next_solver/interner.rs: `impl Interner for DbInterner` spans l.875–2019 (~1.1k lines, ~240 fn/type items)
                   — that is the interface our table-backed adapter must implement for M7 (R1)
   writes          resolved site rows: (site span, target span id | external item id, receiver type, via-trait) or
                   unresolved(reason)
   order of build  7a local inference + inherent methods + autoderef table, NO solver
                   7b trait methods via traits in scope + impl-header-index-table lookup by self-type head
                   7c link rustc's next trait solver behind our own adapter for generics, blanket impls, associated types
                   — each sub-step is gated by oracle agreement (§11) before the next starts
   unit of work    one fn body; bodies in parallel; memory freed after each body
```

## 7. Tables

```text
 table                              written by                          row = …
 ────────────────────────────────── ─────────────────────────────────── ──────────────────────────────────────────────────────
 crate-dependency-graph-table       cpu-tool-crate-graph-loader         one crate: name, version, edition, features, cfg, root file, deps
 file-source-text-table             cpu-tool-ingest-source-files        one file: path, origin, digest, raw bytes
 span-reference-text-table          cpu-tool-syntax-tree-walker         one span: id, kind, name, visibility, byte range, parent, origin
 field-signature-detail-table       cpu-tool-syntax-tree-walker         one field of a span: params, return, generics, bounds, use tree…
 body-site-pending-table            cpu-tool-syntax-tree-walker         one body site: span, byte range, site kind, name text
 proc-macro-shape-table             human-written, checked by test      (macro crate, version, macro name) → items/impls/methods it adds
 lang-item-definition-table         cpu-tool-port-name-resolver         one lang item: name (deref, into_iter, branch, add …) → item span
 module-scope-binding-table         cpu-tool-port-name-resolver         (crate, module, name, namespace) → target
 macro-expanded-item-table          cpu-tool-port-macro-expander        one expanded item: invocation span, expanded span, status
 lowered-signature-type-table       cpu-tool-port-signature-lowerer     one type path in a signature → resolved target
 impl-header-index-table            cpu-tool-port-signature-lowerer     one impl: self-type head, trait, generics, where, assoc types
 edge-evidence-record-table         cpu-tool-port-body-inferencer       one evidence row: from span, to target, kind, site range, status
                                    and the signature lowerer
 hld-current-dependency-edges       cpu-tool-hld-table-builder          §3
 hld-file-level-rollup · hld-folder-level-rollup   triggers on HLD rows file → file / folder → folder counters
 per-level-coverage-banner          cpu-tool-status-coverage-banner     published counts
 oracle-edge-agreement-table        cpu-tool-oracle-edge-differ         offline only: per site, port answer vs rust-analyzer answer
```

## 8. CPU tools

```text
 cpu-tool-crate-graph-loader        runs `cargo metadata --offline`, writes crate-dependency-graph-table, lists library source roots
 cpu-tool-ingest-source-files       walks workspace + library source roots, stores bytes verbatim, digests, skips unchanged
 cpu-tool-syntax-tree-walker        one pinned parse per file → spans, fields, body sites; byte-coverage identity checked
 cpu-tool-port-name-resolver        stage 4
 cpu-tool-port-macro-expander       stage 5
 cpu-tool-port-signature-lowerer    stage 6
 cpu-tool-port-body-inferencer      stage 7 (includes method lookup and the solver adapter)
 cpu-tool-hld-table-builder         stage 9
 cpu-tool-status-coverage-banner    stage 10
 cpu-tool-oracle-edge-differ        stage 11, offline only; runs rust-analyzer (scip export / analysis-stats) only to score the port (an answer key for tests, never training data)
 cpu-tool-find-search-query · cpu-tool-impact-graph-walk   unchanged from DI-08; read the tables built here
```

## 9. Executable requirements

```text
 REQ-RUST-001.0  crate graph
   WHEN cpu-tool-crate-graph-loader runs on a workspace with a Cargo.lock and downloaded dependencies
   THEN it SHALL write one crate-dependency-graph-table row per crate in `cargo metadata --offline` output
   AND  it SHALL NOT invoke cargo build, cargo check or a build script, nor rustc except `--print cfg` and `--print sysroot`

 REQ-RUST-002.0  byte-exact ingest
   WHEN a file is ingested
   THEN its stored bytes SHALL equal the bytes on disk (no newline, whitespace or encoding change)
   AND  a re-run on an unchanged file SHALL write nothing

 REQ-RUST-003.0  byte coverage
   WHEN a file of a compiling workspace is walked
   THEN item spans + classified gaps SHALL sum to the file length, sorted, contiguous, non-overlapping, with 0 bytes of kind "other"

 REQ-RUST-004.0  no parse errors in scope
   WHEN a workspace is accepted into phase-20261111
   THEN every workspace file SHALL parse with 0 parser errors; a file with errors SHALL stop the run with its path, not be recovered

 REQ-RUST-005.0  name resolution agreement
   WHEN cpu-tool-port-name-resolver has run on a crate
   THEN for every path in a use tree or signature, its target SHALL equal rust-analyzer's resolution of the same path
        in ≥ 99.5 % of paths, and every disagreement SHALL be listed in oracle-edge-agreement-table with a reason

 REQ-RUST-006.0  macro_rules! expansion agreement
   WHEN an item-position macro_rules! invocation is expanded
   THEN the set of (kind, name) of the produced items SHALL equal rust-analyzer's expansion for ≥ 99 % of invocations
   AND  a proc-macro invocation SHALL go through proc-macro-shape-table (REQ 013) or be written unresolved(proc_macro), never guessed

 REQ-RUST-007.0  method-call precision
   WHEN cpu-tool-port-body-inferencer writes a method-call site as extracted
   THEN its target SHALL equal rust-analyzer's target in ≥ 99 % of extracted method-call sites (precision first)
   AND  sites it cannot resolve SHALL be written unresolved with one reason code: unknown_receiver · trait_solver_needed ·
        proc_macro · cfg_inactive · external_source_missing

 REQ-RUST-008.0  method-call coverage (reported per sub-step)
   WHEN 7a, 7b and 7c are complete
   THEN the banner SHALL report the extracted share of method-call sites after each sub-step; the phase target is ≥ 95 % after 7c
        (target, not measured)

 REQ-RUST-009.0  HLD derivation
   WHEN cpu-tool-hld-table-builder runs
   THEN every row SHALL connect two HLD nodes (§2, D6) (or a node and an external item), and every collapsed row SHALL have
        evidence rows naming the private items it passed through

 REQ-RUST-010.0  no hidden analysis engine
   WHEN the Parseltongue binary is built
   THEN its dependency tree SHALL contain no ra_ap_hir*, ra_ap_ide*, ra_ap_base_db or salsa crate (checked by `cargo tree`)

 REQ-RUST-011.0  memory
   WHEN the full pipeline runs on the rust-analyzer repository itself (workspace + library source roots)
   THEN peak resident memory SHALL stay ≤ 2 GB (target, not measured) and SHALL be reported with the run

 REQ-RUST-013.0  proc-macro shape agreement
   WHEN a proc-macro invocation matches a proc-macro-shape-table row
   THEN the (kind, name, signature) set of the items it adds SHALL equal rust-analyzer's expansion on the five repos for 100 %
        of shape rows; a mismatching row SHALL fail the test with the macro, version and diff

 REQ-RUST-012.0  determinism
   WHEN the pipeline runs twice on the same tree with the same thread count or different thread counts
   THEN hld-current-dependency-edges SHALL be byte-identical when exported in sorted order
```

## 10. Verification matrix

```text
 req      test (four-word name)                    type                      assertion                                   metric
 ──────── ──────────────────────────────────────── ───────────────────────── ─────────────────────────────────────────── ─────────────────────
 001      test_crate_graph_offline_only            integration               rows = cargo metadata crates; no rustc spawn  count equal
 002      test_ingest_bytes_stay_identical         property (proptest)       stored == disk for random byte files          100 %
 003      test_span_gap_tiling_identity            integration, corpus       identity holds on every file                  0 failures (M today)
 004      test_parse_error_stops_run               unit                      erroring file → typed error with path         —
 005      test_nameres_matches_oracle_paths        integration + oracle      port target == rust-analyzer target           ≥ 99.5 %
 006      test_macro_rules_items_match             integration + oracle      expanded (kind, name) set equal               ≥ 99 %
 007      test_method_target_precision_oracle      integration + oracle      extracted target == oracle target             ≥ 99 %
 008      test_method_coverage_per_substep         benchmark report          extracted share after 7a / 7b / 7c            reported; ≥ 95 % after 7c
 009      test_hld_rows_public_endpoints           integration               both ends public; collapsed rows have evidence 100 %
 010      test_no_analysis_crates_linked           build gate (cargo tree)   forbidden crates absent                       0
 011      test_peak_memory_within_budget           benchmark (/usr/bin/time) peak RSS on rust-analyzer repo               ≤ 2 GB target
 013      test_proc_macro_shapes_match_oracle      integration + oracle      shape (kind, name, signature) set equal       100 % of shape rows
 012      test_hld_export_is_deterministic         integration               sorted exports equal across 1 and N threads   byte-identical
 gates    cargo fmt · cargo clippy --all-targets -- -D warnings · cargo test · cargo build --release
```

Oracle corpus (each must pass `cargo check` once, offline, when chosen): rust-analyzer, ripgrep, tokio, serde, cargo-modules. All five are test cases on equal terms (decided 2026-11-05); there is no tune/held-out split, because phase-20261111 has CPU rules only and no trained models.

## 11. Milestones — each ends with a measured gate

```text
 M1  crate graph + ingest + walker on workspace AND library source roots      gate: REQ 001–004 green; file/span counts published
 M2  cpu-tool-port-name-resolver                                              gate: REQ 005 on all five repos
 M3  cpu-tool-port-macro-expander (macro_rules! + built-in derive table)      gate: REQ 006
 M4  cpu-tool-port-signature-lowerer + impl-header-index-table                gate: every signature type path resolved or unresolved
                                                                              with a reason; signature edges (uses_type, implements)
                                                                              agree with oracle ≥ 99.5 %
 M5  body inference 7a (locals, inherent methods, autoderef table)            gate: REQ 007 precision; coverage reported (REQ 008)
 M6  body inference 7b (trait methods by traits in scope + impl index)        gate: REQ 007; coverage reported
 M7  7c: link ra-ap-rustc_next_trait_solver behind our adapter                gate: REQ 007 + 008 target; REQ 010 still green
 M8  cpu-tool-hld-table-builder + roll-ups + banner                           gate: REQ 009, 011, 012
```

Decision point after M6: if 7a + 7b already reach ≥ 95 % method-call coverage on all five repos, M7 is optional for this phase.

## 12. Budgets (targets, to be measured — none of these is measured yet)

```text
 corpus size             rust-analyzer repo ≈ 1 M lines including tests (estimate) + its dependency and std sources
 parse                   M: 789 k lines/s/core for a full lossless tree → seconds, not minutes
 name resolution         target: minutes for the whole crate graph, crates in parallel
 body inference          target: < 30 min on 4 cores for the rust-analyzer repo including dependencies
 peak memory             target: ≤ 2 GB (REQ 011); the reference point is rust-analyzer itself (M: 2.3 GB for 42 k lines)
 disk                    target: one TursoDB file per workspace (§4.2); size reported
```

## 13. Out of scope for this phase

```text
 broken or partially written code      NEXT PHASE: recovery rule, grey spans, tiny-llm-span-clarifier and the other tiny LLMs
                                       are added on top of this phase's tables and ported resolvers
 all LLMs and all training            no tiny LLM (binder, guesser, ranker, clarifier), no training or post-training, no
                                       labelled datasets; unresolved rows are counted, not guessed
 full proc-macro expansion             only shapes from proc-macro-shape-table (D3); every other proc macro → unresolved(proc_macro)
 build-script output                  (D4) never run, never read. Consequences: `include!(concat!(env!("OUT_DIR"), …))` →
                                       unresolved(build_script); `env!` of a build-script variable → unresolved(build_script);
                                       `CARGO_PKG_*` / `CARGO_MANIFEST_DIR` are filled from cargo metadata; a cfg that only a
                                       build script sets (`cargo:rustc-cfg=…`) is treated as OFF, as rust-analyzer does without
                                       build scripts, and the items under it are counted
 cfg combinations                      one cfg set per run: the default features from cargo metadata; inactive code counted
 intent tables                         hld-future-dependency-edges, hld-delta-dependency-view, cpu-tool-propose-future-rows
 incremental re-map on edit            full runs only; digests are stored so the next phase can add it
 runtime dispatch                      `dyn Trait` and generic calls point to the trait method, not to an implementation
```

## 14. Risks and open questions

```text
 R1  the solver adapter is large: rustc's solver asks for generics, predicates and impls through an interner interface; porting
     next_solver/ (19.8k lines) may be most of the phase. Mitigation: M7 is gated and optional (§11)
 R2  ra-ap-* crates are republished weekly with 0.0.x APIs. Mitigation: pin one version for the whole phase
 R3  library source roots are large (std + all dependencies can exceed the workspace). Mitigation: map them once, reuse rows
     across workspaces by digest
 R6  TursoDB is pre-1.0 and evolving fast; MVCC write conflicts under heavy parallel writes are unmeasured. Mitigation: pin
     one version, retry on conflict, measure write throughput in M1
 R4  disk-backed lookups may be slow for method lookup. Mitigation: per-body caches and an in-memory impl index per crate;
     measure before optimising
 R7  rust-analyzer's OWN code is proc-macro heavy (M, grep of crates/, tests included): 196 `#[salsa::tracked|interned|input|db|
     query_group]` uses and ~170 Serialize / ~167 Deserialize derives. salsa attributes generate the query methods that most of
     rust-analyzer calls through `db.…()`; without proc-macro expansion those call sites are unresolved(proc_macro), so the
     ≥ 95 % method-call target (REQ 008) depends on proc-macro-shape-table covering salsa and serde (D3)
 R8  codebase-memory's graph for rust-analyzer has no CALLS edges out of DefCollector methods (trace_path: 0 callees); the
     stage structure above was checked against the source, not the graph
 KL1 KNOWN LIMITATION — build scripts (D4). This phase never runs build.rs and never reads its output, so code that only
     exists after a build script runs is outside the HLD. Measured on this VM's ~/.cargo/registry (sample biased to the
     tools cloned here, not a crates.io survey):
       13 % of crates have a build.rs (112 of 847 crate names)
       of the 203 versioned crate dirs with one: 58 % (117) emit rustc-cfg, mostly compiler-version probes;
                                                  7 % (15) include generated code from OUT_DIR — the case that loses edges
       rust-analyzer's own 5 build.rs only set env values (commit hash, version) or check-cfg
     Reported per run as the unresolved(build_script) count. Phase 2 will address build scripts (for example by reading
     saved results in target/, or another approach decided then).
 R5  copied code keeps rust-analyzer's licence headers and attribution (MIT OR Apache-2.0)
 D3  decided 2026-11-05: proc macros via proc-macro-shape-table (shape, not full expansion), checked against rust-analyzer
 D4  decided 2026-11-05: build scripts are never run and their target/ output is never read. Estimate (M, this VM's
     ~/.cargo/registry, a sample biased to the tools cloned here): 112 of 847 crate names (13 %) have a build.rs; of the 203
     versioned crate dirs with one, 117 (58 %) emit rustc-cfg (mostly compiler-version probes) and 15 (7 %) include generated
     code from OUT_DIR. rust-analyzer's own 5 build.rs only set env values (commit hash, version) or check-cfg → effect on
     the HLD of the five test repos expected small; measured by the unresolved(build_script) count
 D5  decided 2026-11-05: `rustc --print cfg` and `rustc --print sysroot` are allowed (print-only, compile nothing)
 D6  decided 2026-11-05: HLD node = item another file can use (§2 "HLD node"); fn-body items never; reachable_outside_crate column
 D1  decided 2026-11-05: the crate graph comes from `cargo metadata --offline` (the only Cargo call)
 D2  decided 2026-11-05: test set = rust-analyzer, ripgrep, tokio, serde, cargo-modules, all scored equally
```

## 15. Repositories we take from

The full list — each repository, what is taken from it, how (copy-adapt · link-pinned · rewrite-from · oracle · idea-only · tooling), licence and pinned commit — is §8 of `SPEC-phase-20261111-executable-requirements.md`. In short: rust-analyzer (copy-adapt name resolution and macros, rewrite-from types and inference, parser crates linked, offline oracle), rust-lang/rust (std sources; the next trait solver in M7), tursodatabase/turso, cargo_metadata, blake3, proptest; corpus ripgrep, tokio, serde, cargo-modules; ideas only from cargo-modules (MPL-2.0, never copied), cargo-public-api, cargo-semver-checks, RustPräzi, Flowistry.
