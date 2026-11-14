# ARCH — pipeline, full graph and tables

## The answer

**One pipeline turns source into one stored graph. Rust produces facts, DuckDB does the
relational work, Turso keeps the durable frozen result. Every stage reads tables and writes
tables; nothing else crosses a stage boundary.**

Built today: source to resolved names for whole workspaces, then a bounded graph step that
writes item nodes, `contains` edges and `calls` edges and reads them back from a frozen
generation. Not built: type lowering, body inference, method lookup, and six of the eight edge
kinds.

```text
source files
  -> Rust    crate graph, ingest, one parse per file                  built
  -> DuckDB  name resolution rounds  <->  Rust item-macro expander    built, not qualified
  -> Rust    impl headers                                             built: header paths only
  -> DuckDB  item nodes + contains edges                              built: bounded
  -> Rust    calls edges                                              built: type-relative paths only
  -> Turso   frozen full graph, read back and compared                built: marked incomplete
  -> DuckDB  derived levels (interface, roll-ups)                     deferred; fixtures only
```

Product: [PRD](PRD-phase-20261113-dependency-graph-hld.md). Contracts:
[SPEC](SPEC-phase-20261113-executable-requirements.md). Proof:
[SOP](SOP-phase-20261113-full-item-dependency-graph.md). Crates and files:
[crate map](current-codebase-crate-map.md).

## 1 — The pipeline as built

Entry: `run_workspace_mapping_pipeline` in `parseltongue-cli-dispatcher-binary/src/main.rs`.

```text
step  function (crate)                                          writes
1     load_workspace_crate_graph (crate-graph-loader)           crate-dependency-graph-table,
                                                                workspace-config-state-records
2     ingest_workspace_source_files (crate-graph-loader)        file-source-text-table
3     extract_file_syntax_rows (syntax-span-extractor)          span-reference-text-table,
                                                                span-byte-ownership-ranges,
                                                                field-signature-detail-table,
                                                                body-site-reference-table,
                                                                six syntax-namespace-input-* tables,
                                                                lang-item-definition-table
4     prepare_sysroot_staging_scope (cli)                       sysroot crates as extra units
5     run_namespace_analysis_pipeline (duckdb-analysis-adapter  module-scope-binding-table,
      + port-macro-expander)                                    name-resolution-scope-records,
                                                                path-resolution-site-records,
                                                                macro-expanded-item-table
6     persist_impl_header_rows (syntax-span-extractor)          impl-header-index-table
7     publish_full_graph_checkpoint (cli)                       item-effective-visibility-records,
                                                                edge-evidence-record-table
```

- Step 4 runs when a file mentions `std`, `core`, `alloc`, `Clone` and similar, unless
  `--sysroot-source-scope-kind excluded` is passed. itoa triggers it.
- Step 5 copies one frozen Turso generation into a DuckDB file and runs SQL rounds for scopes,
  bindings, imports and path candidates. Each resolved item macro is expanded in Rust; its
  output becomes a virtual source file that goes back through step 3. Repeat to a fixed point.
- Step 7 runs only with `--mapping-final-stage-kind full-item-dependency-graph`:
  1. `derive_original_graph_input` (DuckDB) projects item nodes and `contains` edges.
  2. `resolve_original_call_edges` (Rust) resolves body calls against namespace and impl rows.
  3. `FullGraphInputStorePort` writes the graph to Turso, reads the frozen copy back, and fails
     unless it equals what was derived.
  4. The result is marked `incomplete`. No derived level is published from it.

Four facts in the code that shape the remaining work:

- **C1** Name resolution takes only import and signature paths (`NameResolutionPathKind`). Body
  paths are stored as text in `body-site-reference-table`; only step 7 reads them.
- **C2** Header paths are resolved. Full signature types (parameters, returns, bounds) are not
  lowered, so nothing can type a receiver.
- **C3** itoa with the sysroot did not finish step 5 in 2,400 s. Development runs exclude it.
- **C4** The derived-level builder takes typed records, not tables. It works on fixtures; no
  complete real-source input reaches it.

## 2 — The tables, top to bottom

Levels 1 and 2 are the full graph, the product. Level 0 is derived from it. Levels 3–7 are the
facts it is built from.

```text
level  table                                  owner stage                        state
0      hld-current-interface-nodes            hld-current-materialization-stage  deferred
       hld-current-dependency-edges
       hld-edge-evidence-paths
       hld-file-level-rollup
       hld-folder-level-rollup
1      edge-evidence-record-table             semantic-edge-evidence-stage       bounded: contains, calls
2      item-effective-visibility-records      item-effective-visibility-stage    bounded: visibility unknown
3      impl-header-index-table                signature-type-lowering-stage      header paths only
       lowered-signature-type-table                                              no producer
4      path-resolution-site-records           name-resolution                    produced, not qualified
       name-resolution-scope-records
       module-scope-binding-table
       macro-expanded-item-table
5      syntax-namespace-input-* (six)         syntax-spans                       produced
       lang-item-definition-table
6      field-signature-detail-table           syntax-spans                       produced; text only
       body-site-reference-table
7      span-reference-text-table              syntax-spans                       produced
```

Supporting tables:

- Inputs: `file-source-text-table`, `span-byte-ownership-ranges`,
  `crate-dependency-graph-table`, `workspace-config-state-records`.
- Control: `stage-run-generation-records`, `frozen-stage-publication-records`,
  `frozen-stage-unit-memberships`, `workspace-generation-publication-manifest`.
- Offline only: `oracle-edge-agreement-table`.

The registry of table owners is `STAGE_OWNED_TABLE_NAMES` in
`parseltongue-turso-store-adapter/src/stage-table-store-backend.rs`.

Columns the two graph tables carry:

- **Node:** target reference, item kind, owner kind, body ancestry, effective visibility,
  reachable outside crate, source path, qualified path, signature text.
- **Edge:** source reference, target reference, relation kind, resolution status, evidence row
  key, source site, dispatch metadata.

## 3 — The graph rules

Nodes:

- **G1** Every item is a node whatever its visibility: module, function, struct, enum, trait,
  type alias, const, static, associated function, const and type.
- **G2** Nothing inside a function body is a node. Body contents are evidence.
- **G3** The crate root is a node.
- **G4** Each node stores effective visibility: public, restricted, or private. Trait members and
  trait-impl members take the trait's visibility; inherent members keep their own.
- **G5** Reachable from outside the crate is a separate column.
- **G6** Unknown visibility never silently means private.
- **G7** One file-area node per source file, separate from the file's module node.
- **G8** Each `use` declaration is a node.
- **G9** An inline module is an ordinary module node with its own scope.
- **G11** Impl blocks are not nodes. `impl Integer for u8` is an `implements` fact whose source
  is the builtin type.

Edges:

- **F2** Only facts proven from source are edges. An unresolved site is stored with a reason.
- **F3** A target is a graph node, a canonical external item, or a builtin primitive type.
- **F4** A generic or trait call targets the trait method once, with the bound chain in dispatch
  metadata. It never fans out to implementations.
- **F5** `pub use` gives one `reexports` edge per resolved binding.
- **F6** Every extracted edge is stored, private to private included. Nothing is collapsed.
- **F7** A file-area node `contains` each top-level node of its file.
- **F8** A `use` node `imports` each item it binds.
- **F9** A function `reads` each const or static it reads.
- **F10** A function has edges only to what it uses, never to every import in scope.
- **F11** A call or type edge targets the real item; the `use` it came through is evidence.
- **F13** One lexical container per item. File-level items and impl members belong to the file
  area; nested items belong to their inline module or trait. Impl identity, self type and trait
  are metadata, never a second parent.

Identity:

- **I1** Every node and edge endpoint is a full occurrence key: compilation unit, file, byte
  range, syntax kind, expansion chain. No display names, no invented sysroot spans.
- **I3** Malformed identity, a missing local endpoint, a conflicting duplicate or a wrong
  generation is a typed error before publication.

Derived levels (deferred):

- **L1–L3** The interface level keeps the crate root, non-private nodes, file-area nodes and
  `use` nodes.
- **L4–L6** Collapse walks from a kept node through dropped private nodes to the first kept or
  external target. Every distinct path is kept as evidence; cycles stop; an unresolved step
  ends the path with no edge.
- **L10** Collapse never walks a `contains` or `imports` edge.
- **L8–L9** Roll-ups count interface edges by file and folder.

Open:

- **G10** Enum variants and struct fields: nodes or evidence (PRD OD-4).
- **F12** Does a `pub use` carry both an `imports` and a `reexports` edge?
- **F14** May `reads` and `uses_type` start at a non-function item (PRD OD-8)?
- **I4** How a macro-generated item maps back to its original file (PRD OD-2).
- **L7, L11** Interface-level treatment of crate-root private items and of items in a dropped
  module. Deferred with the levels.

## 4 — What remains, in build order

Each target is one real-source test on pinned itoa. Fix the earliest missing producer.

```text
target                                                   what it needs that is not built
done  <Buffer as Default>::default -> Buffer::new        -
next  Buffer::format -> private::Sealed::write           signature bounds (I: Integer, Integer:
                                                         Sealed); method call on a generic
                                                         receiver; one edge, not twelve
      <Buffer as Clone>::clone -> Buffer::new            implements and uses_type edges
      u128 write -> Unsigned::fmt -> div_rem_1e16        calls inside macro-generated impls;
        -> u128_ext::mulhi                               concrete-receiver impl selection
      use nodes -> imports; reads of MAX_STR_LEN         imports and reads producers
last  whole library equals the frozen reference          all of the above, sysroot included
```

What reading itoa's source showed:

- **E1** `default` and `clone` call `Buffer::new()`, not `Self::new()`. The lookup is
  type-relative: resolve `Buffer`, then find `new` in its inherent impls.
- **E2** itoa uses five type-relative shapes: `Buffer::new`, `i128::MAX_STR_LEN`,
  `Self::MAX_STR_LEN`, `I::MAX_STR_LEN`, `<$primitive as Integer>::MAX_STR_LEN`. Only the first
  is handled.
- **E3** A reference to an associated const is a `reads` edge.
- **E4** Some itoa-to-itoa edges need `core`. The signed `write` calls
  `Unsigned::fmt(self.unsigned_abs(), ..)`; choosing the impl needs the return type of a core
  method.
- **E5** Two concrete-receiver shapes are uncovered: `Unsigned::fmt(self, buf)` inside
  `impl Sealed for u8` must reach `<u8 as Unsigned>::fmt`, and `(self as $primitive).write(buf)`
  must reach `<i64 as Sealed>::write`. A trait being implemented is in scope inside its own impl.
- **E6** Macro-generated impls have spans in a virtual file. The rule mapping them to
  `src/lib.rs` is not written (I4).
- **E7** Body macros (`assert!`, `debug_assert!`) add no itoa-to-itoa edge. Record them
  unresolved; do not expand them.
- **E8** A trait-impl member's visibility comes from the trait, so the visibility step reads impl
  headers.
- **E9** Node rows need a qualified path and signature text, for example
  `<u8 as private::Sealed>::write`.

itoa's shape: three item macros (`impl_Integer`, `impl_Integer_size`, `impl_Unsigned`) generate
12 `Integer` impls, 12 `private::Sealed` impls and 4 `Unsigned` impls; five impls are
hand-written.

How unclear each piece still is (1 clear, 5 needs a decision or an experiment):

```text
5  sysroot (core) handling: the native path does not finish (C3, E4)
4  type-relative lookup beyond Buffer::new (E2)
4  calls with a concrete receiver (E5)
4  macro-generated identity and evidence (E6)
4  freezing the expected itoa graph
3  method call on a generic receiver (next target)
3  lowered signature types; uses_type and implements rules
3  effective visibility (E8)
2  use nodes, imports, contains, reads; body macros recorded unresolved
```
