# ARCH — pipeline, graph and tables for the itoa full item graph

## The answer

**One pipeline turns the pinned itoa source into one stored graph of items and their
dependencies. Rust produces facts, DuckDB does the relational work, Turso keeps the durable
frozen result. Every stage reads tables and writes tables; nothing else crosses a stage
boundary.**

Scope: itoa only, the full item graph only. No other repository, no oracle, no
higher-abstraction table.

Built today: source to resolved names, then a bounded graph step that
writes item nodes, `contains` edges and `calls` edges and reads them back from a frozen
generation. Not built: type lowering, body inference, method lookup, and the other four edge
kinds itoa needs (`uses_type`, `reads`, `implements`, `imports`).

```text
source files
  -> Rust    crate graph, ingest, one parse per file                  built
  -> DuckDB  name resolution rounds  <->  Rust item-macro expander    built
  -> Rust    impl headers                                             built: header paths only
  -> DuckDB  item nodes + contains edges                              built: bounded
  -> Rust    calls edges                                              built: type-relative paths only
  -> Turso   frozen full graph, read back and compared                built: marked incomplete
```

Scope: [PRD](PRD-itoa-full-item-graph-scope.md). Proof:
[SOP](SOP-itoa-full-item-graph-acceptance.md). Crates and files:
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
  4. The result is marked `incomplete`.

Three facts in the code that shape the remaining work:

- **C1** Name resolution takes only import and signature paths (`NameResolutionPathKind`). Body
  paths are stored as text in `body-site-reference-table`; only step 7 reads them.
- **C2** Header paths are resolved. Full signature types (parameters, returns, bounds) are not
  lowered, so nothing can type a receiver.
- **C3** itoa with the sysroot did not finish step 5 in 2,400 s. Development runs exclude it.

## 2 — The tables, top to bottom

Levels 1 and 2 are the full graph, the deliverable. Levels 3–7 are the facts it is built from.

```text
level  table                                  owner stage                        state
1      edge-evidence-record-table             semantic-edge-evidence-stage       bounded: contains, calls
2      item-effective-visibility-records      item-effective-visibility-stage    bounded: visibility unknown
3      impl-header-index-table                signature-type-lowering-stage      header paths only
       lowered-signature-type-table                                              no producer
4      path-resolution-site-records           name-resolution                    produced
       name-resolution-scope-records
       module-scope-binding-table
       macro-expanded-item-table
5      syntax-namespace-input-* (six)         syntax-namespace-input-records     produced
       lang-item-definition-table
6      field-signature-detail-table           syntax-spans                       produced; text only
       body-site-reference-table
7      span-reference-text-table              syntax-spans                       produced
```

Supporting tables:

- Inputs: `file-source-text-table`, `span-byte-ownership-ranges`,
  `crate-dependency-graph-table`, `workspace-config-state-records`.
- Work queues: `module-source-closure-requests`, `syntax-file-work-requests`.
- Control: `stage-run-generation-records`, `frozen-stage-publication-records`.
- Run receipts: `analysis-resolution-receipt-records`.

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
- **F5** `pub use` gives one `reexports` edge per resolved binding. itoa has none.
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

Open:

- **G10** Enum variants and struct fields: nodes or evidence (PRD, open).
- **F12** Does a `pub use` carry both an `imports` and a `reexports` edge?
- **F14** May `reads` and `uses_type` start at a non-function item (PRD, open)?
- **I4** How a macro-generated item maps back to its original file (PRD, open).

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
last  whole library equals the frozen expected graph     all of the above; targets in core as decided
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
4  reviewing and freezing the expected itoa graph
3  method call on a generic receiver (next target)
3  lowered signature types; uses_type and implements rules
3  effective visibility (E8)
2  use nodes, imports, contains, reads; body macros recorded unresolved
```
