# Table-Specs-01 — full dependency graph first, HLD levels derived from it

Date: 2026-11-13. Checked first on pinned itoa (`af77385d0daf4d0e949e81f2588be2e44f69f086`).
Status: a rules document. Nothing in it has been run on itoa. The rules are now in the SPEC
(REQ-RUST-032.0, 019.2, 030.1); the code is unchanged. See section 8.

Every rule carries one tag:

```text
 [spec]      in SPEC-phase-20261111 and the builder code today
 [decided]   decided by the user in chat on 2026-11-13; written into the SPEC, not in the code
 [open]      not decided
```

Sources for [spec]: SPEC §2.4, REQ-RUST-009.2, 019.1, 030.0;
`hld-materialization-core-records.rs`; `hld-materialization-duckdb-store.rs` (node selection SQL).

## 1. Shape

```text
 D1 [decided]  Build one full dependency graph first. Every HLD level is derived from it.
 D2 [decided]  A level never adds a fact. It only filters nodes, collapses paths and counts.
               Changing what a level shows is a change to a filter, not to the pipeline.
 D3 [spec]     Rust extracts the facts; Turso stores them; DuckDB derives the levels.
               No third database. No LLM-derived fact in production rows.
```

```text
 source  ->  Rust: parse, expand macros, resolve names, types, impls, calls
                |
                v
 FULL GRAPH     every item outside function bodies, private included        section 2
                every extracted edge between them                            section 3
                |   DuckDB only: filter, collapse, count. No re-parsing.
                v
 LEVEL 2  interface    nodes usable by another file; private chains collapsed, proof kept
 LEVEL 1  file         file -> file edge counts
 LEVEL 1  folder       folder -> folder edge counts                          section 5
```

```text
 Full graph tables (both registered, neither has a producer yet)
   item-effective-visibility-records    one row per graph node, with visibility and owner
   edge-evidence-record-table           one row per edge fact, with status and source site
 Level tables (builder works on typed fixtures; no real-source input yet)
   hld-current-interface-nodes · hld-current-dependency-edges · hld-edge-evidence-paths
   hld-file-level-rollup · hld-folder-level-rollup

 T1 [decided]   The full graph is those two existing tables, not two new ones.
 T2 [open]      Does the agent query the full graph through these two tables directly, or
                through a published copy with path and signature text added?
```

## 2. Full graph: node rules

```text
 G1 [decided]   Every item is a node, whatever its visibility: module, function, struct, enum,
                trait, type alias, const, static, associated function, associated const,
                associated type. Private helpers such as divmod100 are nodes here.
 G2 [spec]      Owners: crate root, module, trait, impl. Nothing inside a function body is a
                node, including a module nested in a body. Body contents are proof only.
 G3 [spec]      The crate root is a node.
 G4 [spec]      Each node stores effective visibility: public, restricted-non-private
                (pub(crate), pub(super), pub(in path)), or private (pub(self) counts as private).
                Trait members and trait-impl members take the trait's visibility; inherent
                impl members keep their own.
 G5 [spec]      Reachable-from-outside-the-crate is a separate column from visibility.
 G6 [spec]      Unknown visibility never silently means private; it is a typed error.
 G7 [decided]   Each source file has one file-public-area-node, separate from the file's module
                node. Identity: compilation unit plus file path. New item kind file_public_area.
 G8 [decided]   Each `use` declaration is a node (item kind use statement), private or public.
                Reverses the SPEC line "No use statement becomes a node" (REQ 019.1).
 G9 [decided]   An inline module (`mod private { }`) is an ordinary module node inside its
                file's area. It has its own scope and does not see the file's imports.
 G11 [spec]     Impl blocks are not nodes (REQ-RUST-029.0). An impl member's row names its impl
                block as owner evidence. `impl Integer for u8` is stored as an implements fact
                whose source is the builtin type; no local node is invented for `u8`.
 G10 [open]     Enum variants and struct fields. The builder rejects them as owners today.
                itoa has no enum; its two fields (Buffer.bytes, DecimalPairs.0) are private.
                Does not block level 2 on itoa.
```

## 3. Full graph: edge rules

```text
 F1 [spec]      Relation kinds today: calls, uses_type, implements, reexports, contains, derives.
 F2 [spec]      Only facts proven from source are edges (status extracted). An unresolved site
                is stored with its reason and is never an edge. Nothing is guessed.
 F3 [spec]      An edge target is a graph node, a canonical external item (core, std, another
                crate), or a builtin primitive type.
 F4 [spec]      A generic or trait call targets the trait method once, with dispatch and
                substitution in metadata. It never fans out to guessed implementations.
 F5 [spec]      `pub use` and `pub(crate) use` give one reexports edge per resolved binding.
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
 F13 [open]     Containment detail: does each node have exactly one container (the area node for
                top-level items and impl members, the inline module or trait for nested ones)?
                What relates a file's module node to its area node?
 F14 [open]     May `reads` and `uses_type` start at a non-function item (a const initializer,
                an associated type, a struct field type, an array length)? itoa has several.
 F12 [open]     A `pub use` line: does it carry both an imports edge from its use node (F8) and
                the module's reexports edge (F5), or only one? itoa has no `pub use`.
```

## 4. Identity rules (both layers)

```text
 I1 [spec]      Every node and edge endpoint is a full semantic occurrence key
                (compilation unit, file, byte range, syntax kind, expansion chain).
                No display names, no guessed paths, no invented sysroot spans.
 I2 [spec]      Builtin endpoints use only the primitive type vocabulary.
 I3 [spec]      Malformed identity, missing local endpoint, conflicting duplicate, wrong
                generation or missing completion marker: typed error before publication.
 I4 [open]      Macro-generated items: their keys point into the expansion's virtual file.
                The rule mapping them back to src/lib.rs for proof and roll-ups is not written.
```

## 5. Levels: each is a filter over the full graph

```text
 LEVEL 2  interface    hld-current-interface-nodes, hld-current-dependency-edges,
                       hld-edge-evidence-paths

 L1 [spec]      Keep a node when its effective visibility is public or restricted-non-private,
                plus the crate root. Kept nodes with no edges stay.
 L2 [decided]   Always keep file-public-area-nodes, even when the file's module is private.
 L3 [decided]   Keep use nodes at this level.
 L4 [spec]      Collapse: from a kept source, walk extracted edges through dropped (private)
                nodes to the first kept or external target. The last step's relation kind labels
                the edge; via kind = collapsed. A direct edge has via kind = direct.
 L5 [spec]      Every distinct path is kept as proof. Cycles stop per path; a private cycle
                alone invents no edge. Diamonds merge the edge, not the paths. A path that
                meets an unresolved step ends with no edge.
 L6 [spec]      Evidence row count = unique evidence rows across all paths of the edge.
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

 L8 [spec]      Count level 2 edges grouped by source bucket, target bucket, relation kind and
                via kind. Buckets: source file (or folder), crate root, one canonical external
                item. Counts are edges, not path multiplicity.
 L9 [decided]   A file-public-area-node counts in its own file.
```

## 6. Columns

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

## 7. itoa under these rules (expected, not produced)

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
     contains  members of the impls written in the file and generated by its macros (F13 open)
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

## 8. What has changed, and what has not

```text
 written 2026-11-13, documents only
 SPEC        REQ-RUST-032.0 full graph in the two existing tables (D1, G1, G7, G8, G9, F6-F11)
             REQ-RUST-019.2 interface level as a filter (D2, L1-L3); replaces 019.1
             REQ-RUST-030.1 relation kinds imports and reads (F8, F9)
             fourteen named RED test contracts, none written; open points listed
 parent PRD  D6 revised; section 17 added
 itoa PRD    section 6.1 decided, 6.2 open; acceptance A6; relationships and slices extended
 SOP         revision R16: S9 checks the full graph first, then the level as a filter
 AGENTS.md   nomenclature and a revision note

 not done
 SPEC        column lists for the two full-graph tables (section 6 here)
 core types  HldRelationKindValue gains two variants; a file_public_area item kind
 DuckDB      level 2 filter admits file_public_area and use_statement always (L2, L3)
             collapse skips contains and imports edges (L10)
 producers   unchanged in difficulty: resolving every call, type and impl is still the work,
             and none of the four tables between name resolution and the builder has a producer
 reference   the itoa answer key must list full-graph edges, private ones included
```

No code has changed and nothing has been run on itoa.
