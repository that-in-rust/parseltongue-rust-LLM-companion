# itoa HLD-current — how to build it, worked backwards

Date: 2026-11-13. Phase folder: `docs/phase-20261113/`.
Scope: the full dependency graph (two tables) and the five HLD tables derived from it, for `dtolnay/itoa` 1.0.18
(commit `af77385d0daf4d0e949e81f2588be2e44f69f086`, 2 files, 488 lines).
Out of scope here: serde, the other four repositories, sysroot qualification, tiny LLMs.

Companion file: `itoa-hld-preceding-tables.md` (the table list this design builds).

Revised 2026-11-13 after the full-graph-first decisions (`Table-Specs-01.md`, parent PRD §17,
SPEC REQ-RUST-032.0 / 019.2 / 030.1). Sections 3 to 7 below are updated; sections 1 and 2
describe the code and did not change.

This is a design. Nothing in it has been run on itoa. Each statement is marked:

```text
 [code]      read in the current source on 2026-11-13 (file named beside it)
 [measured]  a number from an earlier run in this repository
 [proposal]  not built; a design choice in this document
```

Exploration used codebase-memory-mcp (project `home-ubuntu-repos-parseltongue-integration`,
re-indexed today; 6 files only partly parsed, so every claim below was also checked by reading the file).

## 1. What the pipeline does today

```text
 parseltongue-cli-dispatcher-binary :: run_workspace_mapping_pipeline          [code] main.rs
   |
   |  Rust   parseltongue-crate-graph-loader      load_workspace_crate_graph
   |         -> Turso: crate-dependency-graph-table, workspace-config-state-records
   |
   |  Rust   parseltongue-crate-graph-loader      ingest_workspace_source_files
   |         -> Turso: file-source-text-table
   |
   |  Rust   parseltongue-syntax-span-extractor   extract_file_syntax_rows  (one ra_ap_syntax parse per file)
   |         -> Turso: span-reference-text-table, span-byte-ownership-ranges,
   |                   field-signature-detail-table, body-site-reference-table,
   |                   syntax-namespace-input-* (6 tables), lang-item-definition-table,
   |                   legacy-macro-scope-records, legacy-macro-source-events
   |
   |  Rust   prepare_sysroot_staging_scope        runs when a file mentions std / core / alloc / Clone / ...
   |         -> ingests and parses the sysroot crates as extra units
   |
   |  DuckDB parseltongue-duckdb-analysis-adapter  run_namespace_analysis_pipeline
   |         copy frozen Turso generation -> DuckDB file
   |         SQL fixed point: scopes, bindings, imports, path candidates
   |         Rust  parseltongue-port-macro-expander expands each resolved item macro,
   |               new unit `crate/macros/NNNNNNNN` goes back through the syntax stage, repeat
   |         -> Turso: module-scope-binding-table, name-resolution-scope-records,
   |                   path-resolution-site-records, macro-expanded-item-table
   |
   X  stops here. CLI status: signature-lowering, call-resolution, visibility-hld = "not yet implemented"
```

Separately, and not connected to the above:

```text
 DuckDB  hld-materialization-duckdb-store.rs     takes HldMaterializationInputRecord (typed nodes + edges)
         -> five HLD record sets.  15 fixture tests passed.                    [measured]
 Turso   hld-graph-publication-turso.rs          atomic publish of the five tables.
         last run 9 passed, 2 failed (fixture setup).                          [measured]
```

## 2. Four facts in the code that decide the design

```text
 F1  Body paths are never sent to name resolution.                             [code]
     syntax-namespace-input-extraction.rs :: check_exported_signature_path returns false
     for any path inside a fn body. NameResolutionPathKind has two values: Import, Signature.
     So `Buffer::new()` inside `default` is stored as text in body-site-reference-table
     with status "pending", and nothing downstream ever looks at it.

 F2  Signature and impl facts are raw syntax, not types.                       [code]
     lib.rs emits one field row per PARAM, SELF_PARAM, RET_TYPE, TYPE_PARAM, WHERE_PRED,
     TYPE_BOUND, ATTR node, and an IMPL row. The type paths inside them ARE resolved
     (they are Signature path sites). Nothing joins the two into
     "impl self type = Buffer, trait = Default".

 F3  itoa will trigger sysroot staging.                                        [code]
     check_sysroot_context_needed is true when a file contains the identifier `core` or `Clone`.
     itoa's lib.rs contains both. The latest native run through that path (not itoa)
     stopped at its 2,400 s limit (exit 124) with peak whole-tree RSS 9,234,448 KiB
     (8.81 GiB); last marker: round 3 import-bindings started.                 [measured]
     Whether itoa plus sysroot finishes is unknown. It has not been run.

 F4  The HLD builder reads records, not tables.                                [code]
     HldGraphAnalysisPort takes HldMaterializationInputRecord. No code builds that
     record from Turso tables.
```

## 3. Target architecture

The Turso + DuckDB split of PRD-phase-20261111 §16 is kept as it is. Rule for who does what:

```text
 Rust     parse, expand macros, lower types, infer inside bodies
 DuckDB   every join, every recursion, dedup, roll-up
 Turso    every durable row, completion markers, frozen snapshots, publication
```

Stage numbers below are the PRD-phase-20261111 §5 numbers. Bottom (spans) to top (HLD).

```text
 PRD 0-1   crate graph, source ingest       EXISTS                  Rust -> Turso
 PRD 2/3   syntax walker                    EXISTS                  Rust -> Turso
 PRD 4+5   name resolution + macros         EXISTS, not qualified   DuckDB SQL + Rust expander -> Turso
           macro output is stored as a virtual source file and goes through the
           same syntax walker, so generated impls get span, field and body-site rows [code]
        |
        v
 PRD 6     signature-type-lowering-stage    [proposal] cpu-tool-port-signature-lowerer
        Rust    walk each fn / impl / trait / struct span; turn type syntax into
                a small type tree (path, reference, pointer, array, tuple, generic parameter)
        DuckDB  join type-path leaves to the already resolved Signature path sites
        Turso   lowered-signature-type-table   one row per (item, slot): param n, return, self, bound
                impl-header-index-table        one row per impl: self type, trait target, generics,
                                               member occurrences
                edge-evidence-record-table     uses_type and implements rows (through the table's
                                               owner stage, semantic-edge-evidence-stage)
        |
        v
 PRD 7     semantic-edge-evidence-stage     [proposal] cpu-tool-port-body-inferencer
        body paths  today no stage resolves them (F1). Two ways to do it: decision AD-2 in the PRD.
        DuckDB  path calls:  body site JOIN resolved body path -> calls
                `Type::x`:   first segment resolved as a type, then member `x` looked up in
                             impl-header-index-table (inherent impls, then trait impls) -> calls
        Rust    method calls `recv.name(..)`: receiver type from the parameter / self / local,
                then inherent-impl lookup (7a), then trait-method lookup through bounds (7b).
                A generic receiver (`I: Integer`) yields ONE edge to the trait method,
                with the bound chain written into dispatch metadata.
        Turso   edge-evidence-record-table calls and reads rows; body-site status moves from
                pending to extracted or unresolved(reason). Never guessed.
                Every edge is stored, private -> private included; none is collapsed here.
        also    [proposal] this stage owns the table, so it also writes the two cheap kinds:
                imports   DuckDB: each resolved Import path site of a use node -> its target
                contains  DuckDB: span ownership -> file-public-area-node / module / trait
                A call that resolved through an import targets the item; the use node's
                occurrence goes into the edge's evidence.
        |
        v
 (registry) item-effective-visibility-stage [proposal] DuckDB SQL only
        The PRD lists no separate stage for this; the Turso registry and the publisher
        already name one, so this design follows the code.
        DuckDB  recursive walk: declared visibility + enclosing scopes + re-exports
                -> effective visibility, owner kind, body ancestry, reachable-outside-crate
        Turso   item-effective-visibility-records. This is the full graph's node table: one row
                for EVERY item outside fn bodies (private ones too), one per `use`, and one
                file-public-area-node per source file. It no longer decides "node or not".
        |
        v
 PRD 9     hld-current-materialization-stage
        assembler  [proposal] in the DuckDB adapter: read the edge and visibility rows of the
                   frozen generation, decode into HldMaterializationInputRecord, refuse if any
                   of the eight input stage markers is missing
        builder    EXISTS on fixtures: select nodes, collapse private paths, proof paths, roll-ups
                   must change: admit file_public_area and use_statement nodes at the interface
                   level; accept imports and reads; stop walking contains and imports edges
                   during collapse (HLD_RECURSIVE_PATH_SQL walks every kind today)
        Turso      atomic publish of the five HLD tables
```

Why PRD 7 is split between SQL and Rust: a path call is a join; a method call needs the
receiver's type, which depends on local `let` bindings and expression types. That is
inference over one function body at a time, which the PRD already assigns to Rust.

## 4. What itoa actually needs from each stage

From the draft source inventory (48 selected occurrences, 197 call-site candidates; draft, not golden):

```text
 PRD 4+5  three item-position macro_rules: impl_Integer (5 calls), impl_Integer_size
          (6 calls written, 2 active on a 64-bit target: the rest are cfg-gated off),
          impl_Unsigned (4 calls). They generate 12 Integer impls, 12 private::Sealed impls
          and 4 Unsigned impls: 12 const, 12 type and 12 method occurrences that are interface-level nodes.
 PRD 6    5 hand-written impl blocks (Default, Copy, Clone for Buffer; inherent Buffer;
          Unsigned for u128) plus the generated ones; trait Integer: private::Sealed;
          Buffer::format<I: Integer>
 PRD 7    path calls in bodies (Buffer::new, Unsigned::fmt, mulhi, div_rem_1e16, core paths); method calls on
          self, on a generic `I`, and on core types (MaybeUninit, pointers, slices)
 visibility  full graph: every item outside fn bodies is a node, private helpers included.
          interface level: `private::Sealed` and `pub(crate) mulhi` are kept although their
          modules are private; private helpers and private trait impl methods are dropped
          and stay as proof steps on collapsed edges
 new rows 3 use nodes with 4 imports edges (the two `no_panic` uses are cfg-off), 2
          file-public-area-nodes with their contains edges, reads edges into MAX_STR_LEN
 PRD 9    one collapse chain: <u128 as Sealed>::write -> ... -> u128_ext::mulhi
```

Not needed by itoa, so not built now: proc macros, build scripts, features, cross-crate
dependencies other than `core`, async, closures as HLD nodes, dyn dispatch.

## 5. Build order — one relationship at a time, on real source

Each slice is one real-source test on the pinned itoa files. The expected rows live only in the test.

```text
 slice 0  run today's pipeline on itoa as it is; record time, peak RSS, which stage ends it.
          This answers F3 before anything is designed around it.

 slice 1  <Buffer as Default>::default --calls--> Buffer::new
          needs  impl header rows only from PRD 6, the `Type::x` rule from PRD 7,
                 minimal visibility rows, the assembler, the builder
          does not need  type lowering of params, method calls, macros

 slice 2  <Buffer as Clone>::clone --calls--> Buffer::new;  implements and uses_type edges
          needs  full PRD 6

 slice 3  Buffer::format --calls--> private::Sealed::write   (one generic edge, not twelve)
          needs  PRD 7 method calls with a generic receiver (7b, through the bound)

 slice 4  <u128 as Sealed>::write -> <u128 as Unsigned>::fmt -> div_rem_1e16 -> u128_ext::mulhi
          needs  macro-generated impls through PRD 6 and 7; collapse in PRD 9

 slice 5  use nodes, imports edges, file-public-area-nodes with contains edges, reads edges
          needs  the node table writing use and area rows; the two cheap edge kinds above

 slice 6  whole library: full-graph nodes and edges, then the derived HLD tables, compared
          against the independently reviewed target dataset
```

Each slice is checked on the full-graph row first; the interface-level row follows once the
visibility stage exists.

Slice 1 is the "next task" already written in the SOP. This file only adds which stages it touches.

## 6. Decisions that are open

`PRD-phase-20261113-itoa-hld-current.md` §6 holds them: §6.1 what was decided on 2026-11-13
(DD-1 to DD-9), §6.2 what those decisions left open (OD-2 to OD-8), §6.3 the pipeline choices
with options (AD-1 to AD-5).

## 7. What this design does not claim

```text
 - No PRD 6, PRD 7, visibility or assembler code exists.
 - No itoa run has happened; slice 0 is the first.
 - The 48 / 197 inventory is a draft, not an accepted reference.
 - A passing slice 1 is one relationship, not a Phase 1 PASS.
 - The file_public_area item kind, the imports and reads relation kinds and the collapse
   change are documents only; the code has none of them.
```
