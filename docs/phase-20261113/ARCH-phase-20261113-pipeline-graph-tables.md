# Architecture — phase-20261113: pipeline, full graph and HLD tables

Date: 2026-11-13. States what is in force now. It folds together the earlier table rules,
build architecture, preceding-table chain and haze review; those drafts are kept in
`archive-folded-draft-files/`.
Product intent: PRD-phase-20261113-dependency-graph-hld.md.
Contracts: SPEC-phase-20261113-executable-requirements.md.
First checked on pinned itoa 1.0.18 (`af77385d0daf4d0e949e81f2588be2e44f69f086`, 2 files, 488 lines).
Current verified implementation status is in the
[status board](../phase-20261111/current-milestone-status-board.md), not inferred from this design.

**Active scope:** the detailed full map and its source/semantic evidence. Higher-level
abstractions and unrelated decisions are in the [backlog](phase1-abstractions-decision-backlog.md).
One logical graph spans node, edge and linked evidence tables; it is not one physical mega-table.
Later abstractions are queries over a compatible frozen generation, optionally materialized.
They reuse these facts rather than reanalyzing source or inventing missing semantic information.
Retained HLD sections below describe deferred derivations, not active Phase 1 acceptance.

REQ-RUST-045.0 separates incomplete full-map checkpoint validation/storage from HLD eligibility.
The existing canonical node/edge types, codecs, tables and exact frozen source provenance are
reused; an L2 full-map store seam retains input completeness explicitly. Rust supplies body/type
resolution, DuckDB projects relational item/container facts, and Turso stores/freeze-checks them.
An incomplete checkpoint is never promoted by HLD assembly/publication. The CLI final stage
requires the bounded node/edge markers before publishing its workspace manifest, without HLD.

Every statement about the system carries one mark:

```text
 [code]      read in the current source on 2026-11-13 (file named beside it)
 [measured]  a number from an earlier run in this repository
 [proposal]  not built; a design choice in this document
```

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
   |  Rust   persist_impl_header_rows: persisted parser/namespace facts -> impl-header-index-table
   |         header-paths-only; not complete signature lowering
   X  complete signatures, body/call resolution and full-map production remain incomplete
```

The bounded original-top-level diagnostic connects source projection, frozen graph inputs,
materialization and publication. It does not supply the full semantic producer chain:

```text
 DuckDB  hld-materialization-duckdb-store.rs     takes HldMaterializationInputRecord (typed nodes + edges)
         -> five HLD record sets; retained as a deferred derived-output implementation
 Turso   hld-graph-publication-turso.rs          atomic publish of the five tables.
         current verification evidence is linked from the status board
```

Phase 1's offline itoa artifacts are a separate `LLM_PARSED_DRAFT` candidate inventory authored
by Devin from pinned source, a distinct
eligible local-only diagnostic subset, blocked/proposed rows, five derived expected-table files,
and one top-down table-contract manifest. They are symbolic and noncanonical; they are neither
production stage rows nor input to the mapper. The manifest distinguishes the typed materializer
interface from registered-but-unbuilt producers and publication controls. OD-2/OD-8 and the
file-module/area part of OD-7 remain open blockers where applicable. Original-item source
containment follows REQ-RUST-043.0; blocked draft rows still need source review. OD-9 is an
alternate proposal, not a suspension of the current
rule 10 six-kind collapse. Candidate coverage omits operators, body types, literals and repeated
same-line sites; the preserved inventory is not a whole-source completeness claim.

## 2. Four facts in the code that decide the design

```text
 C1  Body paths are never sent to name resolution.                             [code]
     syntax-namespace-input-extraction.rs :: check_exported_signature_path returns false
     for any path inside a fn body. NameResolutionPathKind has two values: Import, Signature.
     So `Buffer::new()` inside `default` is stored as text in body-site-reference-table
     with status "pending", and nothing downstream ever looks at it.

 C2  Header paths are resolved; complete signature types are not produced.     [code]
     lib.rs emits one field row per PARAM, SELF_PARAM, RET_TYPE, TYPE_PARAM, WHERE_PRED,
     TYPE_BOUND, ATTR node, and an IMPL row. The type paths inside them ARE resolved
     (they are Signature path sites). impl-header-source-records.rs now joins header
     paths to targets and preserves unresolved reasons; rich type/bound lowering remains missing.

 C3  itoa will trigger sysroot staging.                                        [code]
     check_sysroot_context_needed is true when a file contains the identifier `core` or `Clone`.
     itoa's lib.rs contains both. The latest native run through that path (not itoa)
     stopped at its 2,400 s limit (exit 124) with peak whole-tree RSS 9,234,448 KiB
     (8.81 GiB); last marker: round 3 import-bindings started.                 [measured]
     Whether itoa plus sysroot finishes is unknown. It has not been run.

 C4  The HLD builder reads records, not tables.                                [code]
     HldGraphAnalysisPort takes HldMaterializationInputRecord. HldGraphInputStorePort
     persists/assembles frozen inputs for the bounded structural path. This is not proof
     of complete semantic inputs or a full-map-only serving transition.
```

## 3. Target pipeline

Rule for who does what:

```text
 Rust     parse, expand macros, lower types, infer inside bodies
 DuckDB   every join, every recursion, dedup, roll-up
 Turso    every durable row, completion markers, frozen snapshots, publication
```

Stage numbers are the PRD section 5 numbers. Active work ends at the frozen full map and
linked evidence; stage 9 below is a deferred query/materialization layer. Rust owns semantic
rules and per-body inference, DuckDB owns relational joins/recursion, and Turso owns durable
generations. The current HLD-only serving transition must not be silently repurposed: an
independent full-map export/publication contract remains active under REQ-RUST-044.0.

```text
 stage 0-1   crate graph, source ingest       EXISTS                  Rust -> Turso
 stage 2/3   syntax walker                    EXISTS                  Rust -> Turso
 stage 4+5   name resolution + macros         EXISTS, not qualified   DuckDB SQL + Rust expander -> Turso
           macro output is stored as a virtual source file and goes through the
           same syntax walker, so generated impls get span, field and body-site rows [code]
        |
        v
 stage 6    signature-type-lowering-stage    [proposal] cpu-tool-port-signature-lowerer
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
 stage 7    semantic-edge-evidence-stage     [proposal] cpu-tool-port-body-inferencer
        body paths  today no stage resolves them (C1). Two ways to do it: PRD decision AD-2.
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
        The Turso registry and the publisher already name this stage; where its work is
        computed is PRD decision AD-3.
        DuckDB  recursive walk: declared visibility + enclosing scopes + re-exports
                -> effective visibility, owner kind, body ancestry, reachable-outside-crate
        Turso   item-effective-visibility-records. This is the full graph's node table: one row
                for EVERY item outside fn bodies (private ones too), one per `use`, and one
                file-public-area-node per source file. It no longer decides "node or not".
        |
        v
 DEFERRED: stage 9    hld-current-materialization-stage
        assembler  frozen input assembly exists for the bounded structural path;
                   complete semantic prerequisites remain required for full-semantic publication
        builder    select nodes, collapse allowed private paths, retain proof paths, count roll-ups
                   retained rules: accept the declared node/relation kinds;
                   never walk contains/imports during collapse
        Turso      atomic publish of the five HLD tables
```

Why stage 7is split between SQL and Rust: a path call is a join; a method call needs the
receiver's type, which depends on local `let` bindings and expression types. That is
inference over one function body at a time, which the PRD already assigns to Rust.

## 4. Tables, top (HLD) to bottom (spans)

Levels 1 and 2 together are the full dependency graph, a product in its own right. Level 0 is
derived from it. Levels 3 to 7 are pipeline internals.

REQ-RUST-040.0 starts with an explicit `original-top-level-containment` scope: source-backed,
cfg-active original-file top-level items only. A `FilePublicArea` key is compilation-unit identity
plus normalized relative path, separate from occurrence and namespace identities. The scope and
its incomplete source coverage travel in generation metadata; it does not satisfy the existing
full-semantic publication gate or claim call/type resolution.

```text
 REGISTERED+PRODUCED   table is in the Turso stage registry and a pipeline stage writes it
 PARTIAL               a stage writes it, but the stage is not qualified (M2 native run timed out on sysroot)
 REGISTERED ONLY       table name is in the registry; no source producer exists in crates/
 NOT RUN ON ITOA       applies to every row below: the pipeline has never been run on pinned itoa
```

Sources for the status words (checked 2026-11-13):

```text
 registry   crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs  STAGE_OWNED_TABLE_NAMES
 CLI        crates/parseltongue-cli-dispatcher-binary/src/main.rs  stage_implementation_status_values
            -> signature-lowering, call-resolution, visibility-hld: "not yet implemented"
 grep       the four semantic table names occur only in the registry and one test file
```

```text
 LEVEL 0 — HLD OUTPUTS                                   owner: hld-current-materialization-stage
   hld-current-interface-nodes
   hld-current-dependency-edges
   hld-edge-evidence-paths
   hld-file-level-rollup
   hld-folder-level-rollup
   status: DuckDB builder passes on typed fixtures; Turso publication not GREEN (9 passed, 2 failed)
        ^
        |  DuckDB: filter nodes, collapse private paths (never through contains or
        |  imports edges), keep proof paths, roll up. Adds no fact.
        |  The builder does NOT read tables. It takes HldMaterializationInputRecord:
        |  provenance records + node records + edge records. Something must assemble
        |  those records from levels 1 and 2. That assembler does not exist.
        |
 LEVEL 1 — FULL GRAPH EDGES                              owner: semantic-edge-evidence-stage
   edge-evidence-record-table                            REGISTERED ONLY
   one row per relation, private -> private included, never collapsed: source
   occurrence, resolved target, relation kind (calls / uses_type / implements /
   reexports / contains / derives / imports / reads), status, dispatch +
   substitution metadata, site occurrence, evidence range.
        ^
        |
 LEVEL 2 — FULL GRAPH NODES                              owner: item-effective-visibility-stage
   item-effective-visibility-records                     REGISTERED ONLY
   one row per item outside fn bodies (private included), per `use`, and one
   file-public-area-node per source file: semantic owner, effective visibility,
   body ancestry, external reachability. Every row is a full-graph node; the
   interface level filters on visibility.
        ^
        |
 LEVEL 3 — IMPLS AND TYPES                               owner: signature-type-lowering-stage
   impl-header-index-table                               REGISTERED ONLY
   impl occurrence, self type, trait target, substitutions, associated members.
   lowered-signature-type-table                          REGISTERED ONLY
   resolved parameter / return / self types, generic bounds, projections.
        ^
        |
 LEVEL 4 — NAME AND PATH RESOLUTION                      owner: name-resolution
   path-resolution-site-records                          PARTIAL
   name-resolution-scope-records                         PARTIAL
   module-scope-binding-table                            PARTIAL
   macro-expanded-item-table                             PARTIAL
        ^
        |
 LEVEL 5 — NAMESPACE INPUTS                              owner: syntax-namespace-input-records
   syntax-namespace-input-scopes
   syntax-namespace-input-items
   syntax-namespace-input-imports
   syntax-namespace-input-paths
   syntax-namespace-input-macros
   syntax-namespace-input-externs
   lang-item-definition-table
   legacy-macro-scope-records
   legacy-macro-source-events
   status: REGISTERED+PRODUCED
        ^
        |
 LEVEL 6 — SYNTAX FACTS CUT FROM SPANS                   owner: syntax-spans
   field-signature-detail-table                          REGISTERED+PRODUCED
   raw params, return, generics, bounds, impl trait/self text. Text, not resolved types.
   body-site-reference-table                             REGISTERED+PRODUCED
   raw call / path / method sites inside bodies. Stored pending, not resolved.
        ^
        |
 LEVEL 7 — SPANS  (stop here)                            owner: syntax-spans
   span-reference-text-table                             REGISTERED+PRODUCED
   one row per item span: digest id, kind, name, path, visibility, range, raw code, doc, cfg.
```

Not on the semantic chain:

These exist and are needed, but they carry context or control, not HLD meaning.

```text
 below the spans table
   file-source-text-table            source-ingest   raw file bytes the spans are cut from
   span-byte-ownership-ranges        syntax-spans    which span owns which bytes
   crate-dependency-graph-table      crate-graph     packages, targets, features, cfg
   workspace-config-state-records    crate-graph     workspace configuration, required stages

 work queues
   module-source-closure-requests    syntax-spans
   syntax-file-work-requests         syntax-spans

 control and publication
   stage-run-generation-records              completion markers per stage
   frozen-stage-publication-records          frozen snapshot revisions
   frozen-stage-unit-memberships             which units belong to a frozen revision
   workspace-generation-publication-manifest which generation is currently published

 not needed for HLD
   oracle-edge-agreement-table               offline comparison only
   analysis-resolution-receipt-records       name-resolution run receipts
```

The gap:

```text
 levels 7, 6, 5     produced today
 level 4            produced, not qualified
 levels 3, 2, 1     four tables, zero producers   <-- the missing work
 assembler          tables -> HldMaterializationInputRecord, missing
 level 0            builder works on fixtures only
```

`body-site-reference-table` holds the text `Buffer::new()`. Turning that text into
the occurrence `Buffer::new` is the job of levels 4 and 3. No amount of work at
level 0 produces it.

## 5. Graph rules

Rule tags:

```text
 [code]      in the builder code today
 [decided]   decided by the user on 2026-11-13; in the SPEC, not in the code
 [open]      not decided; listed in PRD section 13
```

```text
 Full graph tables (bounded original-top-level producer exists; full semantic coverage does not)
   item-effective-visibility-records    one row per graph node, with visibility and owner
   edge-evidence-record-table           one row per edge fact, with status and source site
 Level tables (deferred; bounded structural real-source output is not full-map qualification)
   hld-current-interface-nodes · hld-current-dependency-edges · hld-edge-evidence-paths
   hld-file-level-rollup · hld-folder-level-rollup

 T1 [decided]   The full graph is those two existing tables, not two new ones.
 T2 [deferred]  Consumer-facing denormalization moved to backlog BD-04.
                Canonical frozen full-map readback/export and evidence remain active.
```

### Nodes

```text
 G1 [decided]   Every item is a node, whatever its visibility: module, function, struct, enum,
                trait, type alias, const, static, associated function, associated const,
                associated type. Private helpers such as divmod100 are nodes here.
 G2 [code]      Owners: crate root, module, trait, impl. Nothing inside a function body is a
                node, including a module nested in a body. Body contents are proof only.
 G3 [code]      The crate root is a node.
 G4 [code]      Each node stores effective visibility: public, restricted-non-private
                (pub(crate), pub(super), pub(in path)), or private (pub(self) counts as private).
                Trait members and trait-impl members take the trait's visibility; inherent
                impl members keep their own.
 G5 [code]      Reachable-from-outside-the-crate is a separate column from visibility.
 G6 [code]      Unknown visibility never silently means private; it is a typed error.
 G7 [decided]   Each source file has one file-public-area-node, separate from the file's module
                node. Identity: compilation unit plus file path. New item kind file_public_area.
 G8 [decided]   Each `use` declaration is a node (item kind use statement), private or public.
 G9 [decided]   An inline module (`mod private { }`) is an ordinary module node inside its
                file's area. It has its own scope and does not see the file's imports.
 G11 [code]     Impl blocks are not nodes (REQ-RUST-029.0). An impl member's row names its impl
                block as owner evidence. `impl Integer for u8` is stored as an implements fact
                whose source is the builtin type; no local node is invented for `u8`.
 G10 [open]     Enum variants and struct fields. The builder rejects them as owners today.
                itoa has no enum; its two fields (Buffer.bytes, DecimalPairs.0) are private.
                Does not block level 2 on itoa.
```

### Edges

```text
 F1 [code]      Relation kinds today: calls, uses_type, implements, reexports, contains, derives.
 F2 [code]      Only facts proven from source are edges (status extracted). An unresolved site
                is stored with its reason and is never an edge. Nothing is guessed.
 F3 [code]      An edge target is a graph node, a canonical external item (core, std, another
                crate), or a builtin primitive type.
 F4 [code]      A generic or trait call targets the trait method once, with dispatch and
                substitution in metadata. It never fans out to guessed implementations.
 F5 [code]      `pub use` and `pub(crate) use` give one reexports edge per resolved binding.
 F6 [decided]   Every extracted edge is stored, including private -> private. No collapsing
                in the full graph.
 F7 [decided]   file-public-area-node --contains--> each top-level node of its file.
 F8 [decided]   New relation kind `imports`: use node -> imported item, one per resolved
                binding (a glob gives one per binding it brings in). Goes with G8.
 F9 [decided]   New relation kind `reads`: function -> const or static it reads, for example
                Buffer::new -> <i128 as Integer>::MAX_STR_LEN.
 F10 [decided]  Access is not use. A function gets an edge only to items it actually uses,
                never to every import its file can see.
 F11 [decided]  A call or type edge targets the real item, not the `use` line. The use node it
                resolved through is stored as proof on the edge.
 F13 [decided]  One lexical source container per original non-body item. File-level declarations
                and impl members use the file area; nested declarations use the enclosing
                inline module or trait. Impl members skip the impl as a graph container.
                Impl occurrence/self type/trait remain separate ownership records, not extra
                contains parents. No contains/imports collapse. The file-module/area relation
                and macro file attribution remain open (REQ-RUST-043.0).
 F14 [open]     May `reads` and `uses_type` start at a non-function item (a const initializer,
                an associated type, a struct field type, an array length)? itoa has several.
 F12 [open]     A `pub use` line: does it carry both an imports edge from its use node (F8) and
                the module's reexports edge (F5), or only one? itoa has no `pub use`.
```

### Identity

```text
 I1 [code]      Every node and edge endpoint is a full semantic occurrence key
                (compilation unit, file, byte range, syntax kind, expansion chain).
                No display names, no guessed paths, no invented sysroot spans.
 I2 [code]      Builtin endpoints use only the primitive type vocabulary.
 I3 [code]      Malformed identity, missing local endpoint, conflicting duplicate, wrong
                generation or missing completion marker: typed error before publication.
 I4 [open]      Macro-generated items: their keys point into the expansion's virtual file.
                The rule mapping them back to src/lib.rs for proof and roll-ups is not written.
```

### Levels: each is a filter over the full graph

```text
 LEVEL 2  interface    hld-current-interface-nodes, hld-current-dependency-edges,
                       hld-edge-evidence-paths

 L1 [code]      Keep a node when its effective visibility is public or restricted-non-private,
                plus the crate root. Kept nodes with no edges stay.
 L2 [decided]   Always keep file-public-area-nodes, even when the file's module is private.
 L3 [decided]   Keep use nodes at this level.
 L4 [code]      Collapse: from a kept source, walk extracted edges through dropped (private)
                nodes to the first kept or external target. The last step's relation kind labels
                the edge; via kind = collapsed. A direct edge has via kind = direct.
 L5 [code]      Every distinct path is kept as proof. Cycles stop per path; a private cycle
                alone invents no edge. Diamonds merge the edge, not the paths. A path that
                meets an unresolved step ends with no edge.
 L6 [code]      Evidence row count = unique evidence rows across all paths of the edge.
 L11 [open]     `private::Sealed` sits in `mod private`, which this level drops. Under L10 the
                lib.rs area node then has no contains edge to it here. May a contains-only
                chain collapse through a dropped module?
 L7 [open]      Private items visible to several files (private items at the crate root, such
                as divmod100) are dropped by L1. Under D2 this can change later as a filter.
 L10 [decided]  Collapse (L4) never walks a `contains` or `imports` edge, as a first step or a
                later one. It walks calls, uses_type, implements, reexports, derives, reads.
                So the lib.rs area node gets no collapsed calls edges from the private
                functions it contains. A contains or imports edge shows at this level only as
                a direct edge to a kept or external target. The builder's SQL walks every
                relation kind today.

 LEVEL 1  file and folder    hld-file-level-rollup, hld-folder-level-rollup

 L8 [code]      Count level 2 edges grouped by source bucket, target bucket, relation kind and
                via kind. Buckets: source file (or folder), crate root, one canonical external
                item. Counts are edges, not path multiplicity.
 L9 [decided]   A file-public-area-node counts in its own file.
```

### Columns

```text
 full graph nodes   item-effective-visibility-records — column list not yet in the SPEC.
                    Needed by the builder's input record: target reference, category, item
                    kind, owner kind, body ancestry, effective visibility, reachable outside
                    crate, source relative path, qualified path, signature text.
 full graph edges   edge-evidence-record-table — column list not yet in the SPEC.
                    Needed: source reference, target reference, relation kind, resolution
                    status, evidence row key, source site, dispatch metadata.
 level 2 nodes      generation, node target reference, qualified path, source relative path,
                    effective visibility, reachable outside crate, signature text
 level 2 edges      generation, source occurrence, target reference, relation kind, via kind,
                    source and target qualified path and signature text, status (extracted
                    only), evidence row count, origin (same crate / workspace crate /
                    standard library / external crate)
 level 2 paths      edge key, ordered endpoints, ordered evidence row keys, ordered sites
 level 1 roll-ups   source bucket, target bucket, relation kind, via kind, edge count
```

## 6. itoa

### What each stage must handle

From the `LLM_PARSED_DRAFT` candidate dataset (`itoa-target-dataset/`: 73 nodes, 368 edges; symbolic and not frozen):

```text
 stage 4+5  three item-position macro_rules: impl_Integer (5 calls), impl_Integer_size
          (6 calls written, 2 active on a 64-bit target: the rest are cfg-gated off),
          impl_Unsigned (4 calls). They generate 12 Integer impls, 12 private::Sealed impls
          and 4 Unsigned impls: 12 const, 12 type and 12 method occurrences that are interface-level nodes.
 stage 6   5 hand-written impl blocks (Default, Copy, Clone for Buffer; inherent Buffer;
          Unsigned for u128) plus the generated ones; trait Integer: private::Sealed;
          Buffer::format<I: Integer>
 stage 7   path calls in bodies (Buffer::new, Unsigned::fmt, mulhi, div_rem_1e16, core paths); method calls on
          self, on a generic `I`, and on core types (MaybeUninit, pointers, slices)
 visibility  full graph: every item outside fn bodies is a node, private helpers included.
          interface level: `private::Sealed` and `pub(crate) mulhi` are kept although their
          modules are private; private helpers and private trait impl methods are dropped
          and stay as proof steps on collapsed edges
 new rows 3 use nodes with 4 imports edges (the two `no_panic` uses are cfg-off), 2
          file-public-area-nodes with their contains edges, reads edges into MAX_STR_LEN
 stage 9   one collapse chain: <u128 as Sealed>::write -> ... -> u128_ext::mulhi
```

Not needed by itoa, so not built now: proc macros, build scripts, features, cross-crate
dependencies other than `core`, async, closures as HLD nodes, dyn dispatch.

### Expected under the rules (not produced)

```text
 FULL GRAPH adds, compared with level 2
   nodes   mod private, mod u128_ext, DecimalPairs, DECIMAL_PAIRS, divmod100,
           slice_buffer_to_str, trait Unsigned and its fmt methods, enc_16lsd, div_rem_1e16
   edges   <u128 as Sealed>::write --calls--> <u128 as Unsigned>::fmt
           <u128 as Unsigned>::fmt --calls--> div_rem_1e16
           div_rem_1e16 --calls--> u128_ext::mulhi
           ... every other private call, type use and const read

 LEVEL 2
   file-public-area-node  src/lib.rs        (beside the crate root node)
     contains  use core::hint                       imports  core::hint
     contains  use core::mem::{self, MaybeUninit}   imports  core::mem, core::mem::MaybeUninit
     contains  use core::str                        imports  core::str
     contains  Buffer, Integer
     contains  members of file-level original impls (F13); generated-file attribution remains open
     private::Sealed is a node of this file, but its container `mod private` is dropped
               here, so the area has no contains edge to it unless L11 is decided yes
   file-public-area-node  src/u128_ext.rs   (kept although module u128_ext is private)
     contains  mulhi                                pub(crate), reachable outside crate = no
   edges     <Buffer as Default>::default --calls--> Buffer::new
             Buffer::format --calls--> private::Sealed::write              (one edge, F4)
             Buffer::new --reads--> <i128 as Integer>::MAX_STR_LEN         (F9)
             Buffer::format --calls--> core::hint::unreachable_unchecked   (F11)
                                         proof: the `use core::hint` node
             <u128 as Sealed>::write --calls, collapsed--> u128_ext::mulhi
                                         proof: the three full-graph edges above (L4)
   not here  the cfg-gated `use no_panic::no_panic` lines (feature off; not in either layer)
```

### What each relationship needs

```text
 <Buffer as Default>::default --calls--> Buffer::new
   span-reference-text-table          the two fn spans and the impl span
   body-site-reference-table          the `Buffer::new()` site inside default
   level 5 + level 4                  `Buffer` resolves to the struct occurrence
   impl-header-index-table            inherent impl of Buffer has member `new`;
                                      the caller's impl is Default for Buffer
   edge-evidence-record-table         the calls row with its evidence range
   item-effective-visibility-records  both ends are nodes, at both layers
   lowered-signature-type-table       NOT needed for this one

 Buffer::format<I: Integer> --calls--> private::Sealed::write
   everything above, plus
   lowered-signature-type-table       I: Integer, Integer: Sealed, so i.write() is Sealed::write
   must stay ONE generic edge; the twelve integer impls go in dispatch metadata

 <u128 as Sealed>::write -> <u128 as Unsigned>::fmt -> div_rem_1e16 -> u128_ext::mulhi
   everything above, plus
   macro-expanded-item-table          the impls come from itoa's declarative macros
   item-effective-visibility-records  Unsigned::fmt and div_rem_1e16 are private:
                                      full-graph nodes, dropped and collapsed at the
                                      interface level by the HLD builder
```

### What a walk through the source found

```text
 E1  Slice 1 is a type-relative lookup, not a `Self` rule.
     itoa's `default` and `clone` bodies call `Buffer::new()`, not `Self::new()`.
     So slice 1 does not need a `Self` rule. It needs a type-relative lookup:
     resolve `Buffer` to the struct, then find `new` among Buffer's inherent impls.

 E2  Type-relative paths are a stage of their own, and nothing does them.
     The name-resolution SQL handles module-scope paths. The SPEC defers
     type-relative and associated paths. itoa uses five shapes:
       Buffer::new                             struct, inherent impl
       i128::MAX_STR_LEN                       primitive, trait impl
       Self::MAX_STR_LEN                       `Self` inside an impl
       I::MAX_STR_LEN                          generic parameter, through the bound I: Integer
       <$primitive as Integer>::MAX_STR_LEN    fully qualified, inside a macro template

 E3  A reference to an associated const is a `reads` edge.
     `Buffer::new` reads `i128::MAX_STR_LEN`. Open: may the source be a const initializer or
     a type, not only a function (OD-8).

 E4  "Without sysroot, all local edges still work" was false.
     The signed `write` calls `Unsigned::fmt(self.unsigned_abs(), ..)`. Choosing
     `<u8 as Unsigned>::fmt` needs the return type of `i8::unsigned_abs`, a core method.
     So some itoa-to-itoa edges depend on core (PRD AD-1 option B).

 E5  Two call shapes with a concrete receiver are not covered.
     `Unsigned::fmt(self, buf)` inside `impl Sealed for u8` must reach `<u8 as Unsigned>::fmt`,
     not the bodyless trait method. That needs the argument's type and an impl pick.
     `(self as $primitive).write(buf)` in the isize/usize impls is a method call on a
     cast to a primitive; it must reach `<i64 as Sealed>::write`. That needs the cast's
     type and a trait-method lookup on a primitive.
     `Sealed` is not imported there. A rustc check on a four-line copy of the pattern
     showed the call compiles inside `impl private::Sealed for ..` and fails outside it:
     the trait being implemented is in scope in its own impl. The lookup must copy that rule.

 E6  Macro-generated items: where is their evidence?
     Expanded impls are parsed from a virtual source file, so their spans point into
     generated text. Exact generated/original provenance is needed for full-map evidence
     and containment. Token-origin rows exist; the mapping rule is not written. A display/
     roll-up bucket is a separate deferred decision (BD-03), not permission to guess provenance.

 E7  Body-position macros are not connected, but itoa barely depends on them.
     itoa has 15 assert expansions (three `assert!` lines in a macro template) and one
     `debug_assert!`. Their arguments name only primitive consts and a local variable,
     so no itoa-to-itoa edge comes from them. They are defined in core, and body macros
     are not yet joined to body sites (open task from the earlier phase). For itoa
     they only need to be recorded as unresolved sites, not expanded.

 E8  Visibility is not "SQL over scopes only".
     A trait impl member takes its visibility from the trait (`Sealed` public,
     `Unsigned` private). So the visibility stage reads impl headers too.

 E9  The assembler has unstated outputs.
     Node rows need a qualified path text and a signature text
     (for example `<u8 as private::Sealed>::write`), a crate-root node and `contains` edges.
     Section 3 proposes the edge stage for `contains`; path and signature text are open (T2).
```

### Build order — one relationship at a time, on real source

Each slice is one real-source test on the pinned itoa files. The expected rows live only in the test.

```text
 slice 0  run today's pipeline on itoa as it is; record time, peak RSS, which stage ends it.
          This answers F3 before anything is designed around it.

 slice 1  <Buffer as Default>::default --calls--> Buffer::new
          needs  original item/member nodes and source-container edges; impl header rows
                 from PRD 6, the `Type::x` rule from PRD 7, full-map persistence and exact evidence
          does not need  type lowering of params, method calls, macros

 slice 2  <Buffer as Clone>::clone --calls--> Buffer::new;  implements and uses_type edges
          needs  full PRD 6

 slice 3  Buffer::format --calls--> private::Sealed::write   (one generic edge, not twelve)
          needs  PRD 7 method calls with a generic receiver (7b, through the bound)

 slice 4  <u128 as Sealed>::write -> <u128 as Unsigned>::fmt -> div_rem_1e16 -> u128_ext::mulhi
          needs  macro-generated impls through PRD 6 and 7; no collapse

 slice 5  use nodes, imports edges, file-public-area-nodes with contains edges, reads edges
          needs  the node table writing use and area rows; the two cheap edge kinds above

 slice 6  whole library: full-graph nodes, direct edges and detailed evidence compared
          against the independently reviewed target dataset, without HLD tables
```

Each slice is checked through frozen full-map rows and exact source evidence. Interface-level
selection is deferred; visibility remains graph metadata, not an inclusion filter.

Slice 1 is the next task written in the SOP (R15).

## 7. How unclear each task still is

Scale: 1 = clear, could start now. 3 = approach known, details open.
5 = not understood well enough to start; needs a decision or an experiment.

```text
 haze  task                                               what is still unclear
 ----  -------------------------------------------------  ------------------------------------------------
  2    original member nodes and source containers        canonical occurrences; enforce REQ-RUST-043.0
  2    impl-header negative/ownership coverage            headers exist; full signature types do not
  2    body-position macros (E7)                          only: record them unresolved, add no edges
  2    reads edges (E3)                                   the relation exists; non-function source open
  2    use nodes, imports and contains edges              original source containers decided (F13)
  3    body paths, first segment (AD-2)                   local shadowing; generic-parameter heads
  3    lowered signature types                            how deep the type tree goes for itoa
  3    uses_type and implements edge rules                which signature positions emit an edge
  3    method call on a generic receiver (i.write)        walking the supertrait Integer: Sealed
  3    effective visibility                               join with impl headers (E8); AD-3
  3    frozen full-map readback/export                     exact generation/evidence contract, without HLD
  4    type-relative lookup (E2)                          five shapes, primitives, bounds; no code today
  4    calls with a concrete receiver (E5)                argument and cast typing; impl selection
  4    macro-generated identity and evidence (E6)         mapping generated spans back to lib.rs
  4    freezing the full-map reference                     active graph choices, independent review and complete evidence
  5    core handling (AD-1, E4)                           native path unqualified; no fabricated external identities
```

HLD builder/consumer-copy work and projection choices moved to BL-01–BL-06/BD-01–BD-06.
Prior baseline/fixture results are status evidence, not outstanding design work in this list.

## 8. Reference data

Production rows come only from deterministic Rust and SQL. The itoa target dataset in
`itoa-target-dataset/` contains `LLM_PARSED_DRAFT` symbolic candidates authored by Devin from
pinned source. No independent compiler/oracle verification has frozen them. They are comparison
data, not canonical or production input, and the production pipeline must never read them.

## 9. What this file does not claim

```text
 - The complete signature/body/semantic producer chain is not verified.
 - Structural and header runs do not establish complete full-map or HLD equality.
 - The 48 / 197 inventory is a draft, not an accepted reference.
 - A passing slice 1 is one relationship, not a Phase 1 PASS.
 - Deferred query plans are not implemented or passing merely because they are specified.
```
