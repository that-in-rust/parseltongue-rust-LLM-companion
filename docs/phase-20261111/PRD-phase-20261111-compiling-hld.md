# PRD — phase-20261111: HLD for Rust code that compiles

Status: draft · Scope owner: human-architect-intent-author · Source design: `devin-research/Deep-Insight-08.md` (copied and narrowed here; this file stands alone)

## 0. How to read this document

```text
 §1  the phase in one paragraph        what is in, what is out, why this cut
 §2  glossary                          terms used below, defined once
 §3  the product                       the one table this phase must produce, and its row shape
 §4  the one design decision           parse once with the pinned parser, then run PORTED rust-analyzer logic on our tables
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
```

Conventions: four-word hyphenated names for every named thing (AGENTS.md). **M** = measured in this project; every other number is a target or an estimate and says so. `→` = "produces".

## 1. The phase in one paragraph

**Objective: a complete, exact `hld-current-dependency-edges` for a repository that compiles. Code that does not compile is the next phase, which builds on these tables (§13).**

Phase-20261111 builds `hld-current-dependency-edges` — public item → public item dependency rows — for **Rust workspaces that compile** (rust-analyzer itself, ripgrep, tokio-sized repositories). Because the code compiles, every file parses without error, every name has exactly one meaning, and rust-analyzer run offline gives an exact answer key. That removes the three hardest parts of the full design — broken-file recovery, grey spans, and guessing — and lets this phase answer one question cleanly: **can rust-analyzer's resolution logic, ported to run over our span and field tables one crate and one body at a time, reproduce rust-analyzer's dependency edges at a fraction of its memory?** No tiny LLM is used in this phase. Anything the port cannot resolve is written as `unresolved` and counted — never guessed.

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
 HLD                       public → public edges with private chains collapsed (public A → private h → public B gives A → B)
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

Hard rules kept from DI-08: no rust-analyzer server, no rust-analyzer analysis database (`ra_ap_hir*`, `ra_ap_ide*`, salsa) linked into Parseltongue, no `cargo build`/`cargo check`, no proc-macro execution at runtime. `cargo metadata --offline` is allowed: it reads manifests and the lockfile, compiles nothing.

## 5. The journey — one line per step

```text
 0  workspace   → crate graph        `cargo metadata --offline` → crates, deps, features, editions, cfg, source roots
 1  source roots → file rows         every .rs file of the workspace AND of the library source roots it depends on is stored byte-exact
                                     with a digest; library files are tagged origin std / external_crate; unchanged digests are skipped
 2  file        → spans + fields     one parse per file → span rows (kind, name, visibility, byte range, parent) and field rows;
                                     every byte is either in a span or in a classified gap (M: 3,370 files, 0 identity failures)
 3  span        → body sites         every call, method call, path, construction, field access, operator and `?` inside a body
                                     → a pending site row with its byte range
 4  crate       → module scopes      PORTED name resolution, one crate at a time in dependency order: mod tree, use trees, globs,
                                     re-exports, preludes, extern crates, macro_rules! scoping → module-scope-binding-table
 5  macro call  → expanded items     PORTED macro_rules! expansion for item-position calls; built-in derives from a fixed table;
                                     other proc macros → unresolved(proc_macro) → macro-expanded-item-table
 6  signature   → lowered types      every fn/struct/impl/trait signature's type paths resolved through module scopes → lowered
                                     signature rows; every impl header → impl-header-index-table (self type, trait, generics, where)
 7  body        → resolved sites     PORTED inference per fn body: local types, autoderef, method lookup over impl-header-index-table,
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
 STAGE 4 — NAME RESOLUTION                                       mode: copy-adapt
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
   reads           macro definition span + invocation token tree
   writes          macro-expanded-item-table: expanded text is parsed with the same parser; its items become spans with
                   origin = macro, pointing back at the invocation span
   built-in        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)] → a fixed table of the impls
                   they generate; serde/other proc-macro derives → unresolved(proc_macro), counted
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
   reads           one fn body's syntax (re-parsed from its span), lowered signatures of everything it touches,
                   impl-header-index-table rows for the receiver types it meets
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
 cpu-tool-oracle-edge-differ        stage 11, offline only; runs rust-analyzer (scip export / analysis-stats) as offline-teacher-rust-analyzer
 cpu-tool-find-search-query · cpu-tool-impact-graph-walk   unchanged from DI-08; read the tables built here
```

## 9. Executable requirements

```text
 REQ-RUST-001.0  crate graph
   WHEN cpu-tool-crate-graph-loader runs on a workspace with a Cargo.lock and downloaded dependencies
   THEN it SHALL write one crate-dependency-graph-table row per crate in `cargo metadata --offline` output
   AND  it SHALL NOT invoke rustc, cargo build, cargo check or a build script

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
   AND  a proc-macro invocation SHALL be written as unresolved(proc_macro), never expanded or guessed

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
   THEN every row SHALL connect two public items (or a public item and an external item), and every collapsed row SHALL have
        evidence rows naming the private items it passed through

 REQ-RUST-010.0  no hidden analysis engine
   WHEN the Parseltongue binary is built
   THEN its dependency tree SHALL contain no ra_ap_hir*, ra_ap_ide*, ra_ap_base_db or salsa crate (checked by `cargo tree`)

 REQ-RUST-011.0  memory
   WHEN the full pipeline runs on the rust-analyzer repository itself (workspace + library source roots)
   THEN peak resident memory SHALL stay ≤ 2 GB (target, not measured) and SHALL be reported with the run

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
 012      test_hld_export_is_deterministic         integration               sorted exports equal across 1 and N threads   byte-identical
 gates    cargo fmt · cargo clippy --all-targets -- -D warnings · cargo test · cargo build --release
```

Oracle corpus (each must pass `cargo check` once, offline, when chosen): rust-analyzer, ripgrep, tokio, serde, cargo-modules. Split by repository: rules may be tuned on the first two only; the last three are held out.

## 11. Milestones — each ends with a measured gate

```text
 M1  crate graph + ingest + walker on workspace AND library source roots      gate: REQ 001–004 green; file/span counts published
 M2  cpu-tool-port-name-resolver                                              gate: REQ 005 on held-out repos
 M3  cpu-tool-port-macro-expander (macro_rules! + built-in derive table)      gate: REQ 006
 M4  cpu-tool-port-signature-lowerer + impl-header-index-table                gate: every signature type path resolved or unresolved
                                                                              with a reason; signature edges (uses_type, implements)
                                                                              agree with oracle ≥ 99.5 %
 M5  body inference 7a (locals, inherent methods, autoderef table)            gate: REQ 007 precision; coverage reported (REQ 008)
 M6  body inference 7b (trait methods by traits in scope + impl index)        gate: REQ 007; coverage reported
 M7  7c: link ra-ap-rustc_next_trait_solver behind our adapter                gate: REQ 007 + 008 target; REQ 010 still green
 M8  cpu-tool-hld-table-builder + roll-ups + banner                           gate: REQ 009, 011, 012
```

Decision point after M6: if 7a + 7b already reach ≥ 95 % method-call coverage on the held-out repos, M7 is optional for this phase.

## 12. Budgets (targets, to be measured — none of these is measured yet)

```text
 corpus size             rust-analyzer repo ≈ 1 M lines including tests (estimate) + its dependency and std sources
 parse                   M: 789 k lines/s/core for a full lossless tree → seconds, not minutes
 name resolution         target: minutes for the whole crate graph, crates in parallel
 body inference          target: < 30 min on 4 cores for the rust-analyzer repo including dependencies
 peak memory             target: ≤ 2 GB (REQ 011); the reference point is rust-analyzer itself (M: 2.3 GB for 42 k lines)
 disk                    target: one Turso / SQLite file; size reported
```

## 13. Out of scope for this phase

```text
 broken or partially written code      NEXT PHASE: recovery rule, grey spans, tiny-llm-span-clarifier and the other tiny LLMs
                                       are added on top of this phase's tables and ported resolvers
 all tiny LLMs                         no binder, guesser or ranker; unresolved rows are counted, not guessed
 proc-macro expansion                  unresolved(proc_macro) except the built-in derive table
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
 R4  disk-backed lookups may be slow for method lookup. Mitigation: per-body caches and an in-memory impl index per crate;
     measure before optimising
 R5  copied code keeps rust-analyzer's licence headers and attribution (MIT OR Apache-2.0)
 Q1  is `cargo metadata --offline` acceptable as the only Cargo call, or must the crate graph come from parsing Cargo.toml
     and Cargo.lock ourselves?
 Q2  which corpus beyond rust-analyzer is the acceptance set?
```
